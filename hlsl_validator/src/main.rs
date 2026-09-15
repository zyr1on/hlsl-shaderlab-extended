// hlsl_validator - main.rs
// High-performance HLSL, Unity ShaderLab, and Unreal Engine Language Server powered by Microsoft DXC

mod docs;
mod signature;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShaderContext {
    PureHlsl,
    UnityShaderLab,
    UnityHlsl,
    UnrealEngine,
}

pub fn detect_shader_context(uri: &str, content: &str) -> ShaderContext {
    // 1. Unity ShaderLab: .shader extension or top-level non-commented Shader keyword
    let is_shaderlab = uri.ends_with(".shader") || content.lines().any(|l| {
        let t = l.trim();
        !t.starts_with("//") && !t.starts_with("/*") && t.starts_with("Shader \"")
    });
    if is_shaderlab {
        return ShaderContext::UnityShaderLab;
    }

    // 2. Unreal Engine USF/USH: extension, engine paths, or UE shader markers
    if uri.ends_with(".usf")
        || uri.ends_with(".ush")
        || uri.contains("/Engine/")
        || uri.contains("\\Engine\\")
        || uri.contains("/Shaders/Private")
        || uri.contains("\\Shaders\\Private")
        || uri.contains("/Shaders/Public")
        || uri.contains("\\Shaders\\Public")
        || content.contains("#include \"/Engine/")
        || content.contains("#include \"Common.ush\"")
        || content.contains("FMaterialPixelParameters")
        || content.contains("FMaterialVertexParameters")
        || content.contains("ResolvedView.")
        || content.contains("SceneTexturesStruct")
    {
        return ShaderContext::UnrealEngine;
    }

    // 3. Unity HLSL / CG / URP / HDRP / 2D: includes, macros, or built-in variables
    let in_unity_dir = uri.ends_with(".cginc")
        || uri.contains("com.unity.render-pipelines")
        || uri.contains("/Assets/")
        || uri.contains("\\Assets\\")
        || uri.contains("/Packages/")
        || uri.contains("\\Packages\\");

    if in_unity_dir
        || content.contains("HLSLPROGRAM")
        || content.contains("CGPROGRAM")
        || content.contains("CBUFFER_START(UnityPerMaterial)")
        || content.contains("TransformObjectToHClip")
        || content.contains("TransformObjectToWorld")
        || content.contains("UnityObjectToClipPos")
        || content.contains("UnityShaderVariables")
        || content.contains("UnityCG")
        || content.contains("unity_ObjectToWorld")
        || content.contains("unity_WorldToObject")
        || content.contains("unity_MatrixVP")
        || content.contains("UNITY_MATRIX_MVP")
        || content.contains("_Time")
        || content.contains("_SinTime")
        || content.contains("_CosTime")
        || content.contains("unity_DeltaTime")
        || content.contains("SAMPLE_TEXTURE2D")
    {
        return ShaderContext::UnityHlsl;
    }

    // 4. Default: Pure HLSL (DirectX 11/12, DXC compute/graphics shaders)
    ShaderContext::PureHlsl
}

pub fn is_inside_properties_block(doc: &str, target_line: usize) -> bool {
    let mut in_props = false;
    let mut props_depth = 0;
    for (idx, line) in doc.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            if idx == target_line {
                return in_props && props_depth > 0;
            }
            continue;
        }
        if trimmed.starts_with("Properties") {
            in_props = true;
        }
        if in_props {
            let open_b = line.chars().filter(|&c| c == '{').count();
            let close_b = line.chars().filter(|&c| c == '}').count();
            props_depth += open_b;
            if props_depth > 0 && close_b >= props_depth {
                if idx == target_line {
                    return true;
                }
                in_props = false;
                props_depth = 0;
            } else {
                props_depth = props_depth.saturating_sub(close_b);
            }
        }
        if idx == target_line && in_props && props_depth > 0 {
            return true;
        }
    }
    false
}

pub fn is_inside_tags_block(doc: &str, target_line: usize) -> bool {
    let mut in_tags = false;
    let mut tags_depth = 0;
    for (idx, line) in doc.lines().enumerate() {
        let code_part = line.split("//").next().unwrap_or("").trim();
        if code_part.is_empty() || code_part.starts_with("/*") {
            if idx == target_line {
                return in_tags && tags_depth > 0;
            }
            continue;
        }
        if !in_tags && (code_part.starts_with("Tags") || code_part.contains("Tags {") || code_part.contains("Tags{")) {
            in_tags = true;
        }
        if in_tags {
            let open_b = code_part.chars().filter(|&c| c == '{').count();
            let close_b = code_part.chars().filter(|&c| c == '}').count();
            tags_depth += open_b;
            if tags_depth > 0 && close_b >= tags_depth {
                if idx == target_line {
                    return true;
                }
                in_tags = false;
                tags_depth = 0;
            } else {
                tags_depth = tags_depth.saturating_sub(close_b);
            }
        }
        if idx == target_line && in_tags && tags_depth > 0 {
            return true;
        }
    }
    false
}

pub fn is_unity_2d_context(uri: &str, content: &str) -> bool {
    content.contains("Universal2D")
        || content.contains("CanUseSpriteAtlas")
        || content.contains("PreviewType\"=\"Plane")
        || content.contains("UnityPixelSnap")
        || content.contains("[PerRendererData]")
        || content.contains("Sprite")
        || content.contains("UnityGet2DClipping")
        || uri.contains("Sprite")
        || uri.contains("2D")
        || uri.contains("UI")
}

const UNITY_COMPAT_PREAMBLE: &str = r#"
#ifndef __UNITY_BUILTIN_STUBS__
#define __UNITY_BUILTIN_STUBS__
#define fixed4 float4
#define fixed3 float3
#define fixed2 float2
#define fixed float
#define half4 float4
#define half3 float3
#define half2 float2
#define half float

// Texture sampling macros
struct UnitySampler2D { Texture2D t; SamplerState s; };
#define sampler2D UnitySampler2D
#define tex2D(tex, uv) (tex.t.Sample(tex.s, uv))
#define tex2Dlod(tex, uv) (tex.t.SampleLevel(tex.s, (uv).xy, (uv).w))
#define TRANSFORM_TEX(tex,name) ((tex.xy) * name##_ST.xy + name##_ST.zw)

#define TEXTURE2D(name) Texture2D name
#define SAMPLER(name) SamplerState name
#define SAMPLE_TEXTURE2D(name, samplerName, coord2) name.Sample(samplerName, coord2)

// Global Unity Matrices (3D & 2D)
float4x4 UNITY_MATRIX_MVP;
float4x4 UNITY_MATRIX_MV;
float4x4 UNITY_MATRIX_V;
float4x4 UNITY_MATRIX_P;
float4x4 UNITY_MATRIX_VP;
float4x4 unity_ObjectToWorld;
float4x4 unity_WorldToObject;
float4x4 unity_MatrixVP;
float4x4 unity_MatrixV;
float4x4 unity_MatrixInvV;
float4x4 unity_MatrixP;
float4x4 unity_MatrixInvP;

// Global Unity Parameters (Time, Screen, Fog, Camera, Lighting)
float4 _Time;
float4 _SinTime;
float4 _CosTime;
float4 unity_DeltaTime;
float4 _ScreenParams;
float4 _ScaledScreenParams;
float4 _ProjectionParams;
float4 _ZBufferParams;
float4 unity_OrthoParams;
float3 _WorldSpaceCameraPos;
float4 _WorldSpaceLightPos0;
float4 _LightColor0;
float4 _MainLightPosition;
float4 _MainLightColor;
float4 _AdditionalLightsCount;

// Unity 2D Pixel Snapping (Pixel-perfect 2D Sprites)
inline float4 UnityPixelSnap(float4 pos) {
    float2 hpc = _ScreenParams.xy * 0.5;
    float2 temp = floor(pos.xy / pos.w * hpc + 0.5) / hpc;
    pos.xy = temp * pos.w;
    return pos;
}

// Unity 2D UI Canvas Rect Clipping
inline float UnityGet2DClipping(float2 position, float4 clipRect) {
    float2 inside = step(clipRect.xy, position.xy) * step(position.xy, clipRect.zw);
    return inside.x * inside.y;
}

// Coordinate Transforms (Built-in Pipeline)
inline float4 UnityObjectToClipPos(float3 pos) { return mul(UNITY_MATRIX_MVP, float4(pos, 1.0)); }
inline float4 UnityObjectToClipPos(float4 pos) { return mul(UNITY_MATRIX_MVP, pos); }
inline float4 UnityWorldToClipPos(float3 pos) { return mul(UNITY_MATRIX_VP, float4(pos, 1.0)); }
inline float3 UnityObjectToViewPos(float3 pos) { return mul(UNITY_MATRIX_MV, float4(pos, 1.0)).xyz; }
inline float3 UnityObjectToWorldNormal(float3 norm) { return normalize(mul(norm, (float3x3)unity_WorldToObject)); }
inline float3 UnityObjectToWorldDir(float3 dir) { return normalize(mul((float3x3)unity_ObjectToWorld, dir)); }
inline float3 UnityWorldSpaceViewDir(float3 worldPos) { return _WorldSpaceCameraPos - worldPos; }

// Coordinate Transforms (Universal Render Pipeline - URP 2D & 3D)
inline float4 TransformObjectToHClip(float3 pos) { return mul(UNITY_MATRIX_MVP, float4(pos, 1.0)); }
inline float4 TransformObjectToHClip(float4 pos) { return mul(UNITY_MATRIX_MVP, pos); }
inline float3 TransformObjectToWorld(float3 posOS) { return mul(unity_ObjectToWorld, float4(posOS, 1.0)).xyz; }
inline float3 TransformWorldToObject(float3 posWS) { return mul(unity_WorldToObject, float4(posWS, 1.0)).xyz; }
inline float4 TransformWorldToHClip(float3 posWS) { return mul(UNITY_MATRIX_VP, float4(posWS, 1.0)); }
inline float3 TransformObjectToWorldNormal(float3 norm) { return normalize(mul(norm, (float3x3)unity_WorldToObject)); }
inline float3 TransformObjectToWorldDir(float3 dir) { return normalize(mul((float3x3)unity_ObjectToWorld, dir)); }
inline float3 TransformWorldToView(float3 posWS) { return mul(unity_MatrixV, float4(posWS, 1.0)).xyz; }

// URP 2D Lighting Structs
struct SurfaceData2D {
    half4 albedo;
    half3 normalTS;
    half4 mask;
};

// Unity Instancing & Multi-compile stubs
#define UNITY_VERTEX_INPUT_INSTANCE_ID
#define UNITY_VERTEX_OUTPUT_STEREO
#define UNITY_SETUP_INSTANCE_ID(v)
#define UNITY_TRANSFER_INSTANCE_ID(v, o)
#define UNITY_INITIALIZE_VERTEX_OUTPUT_STEREO(o)
#define UNITY_ACCESS_INSTANCED_PROP(arr, var) var

// Constant Buffer macros (URP / HDRP / SRP)
#define CBUFFER_START(name) cbuffer name {
#define CBUFFER_END };
#endif
"#;

const UNREAL_COMPAT_PREAMBLE: &str = r#"
#ifndef __UNREAL_BUILTIN_STUBS__
#define __UNREAL_BUILTIN_STUBS__
struct FViewUniformShaderParameters {
    float3 WorldCameraOrigin;
    float GameTime;
    float RealTime;
    float DeltaTime;
    float4 ViewSizeAndInvSize;
    float4x4 WorldToClip;
    float4x4 ClipToWorld;
    float4x4 TranslatedWorldToClip;
    float4 ScreenPositionScaleBias;
};
static const FViewUniformShaderParameters ResolvedView = (FViewUniformShaderParameters)0;
static const FViewUniformShaderParameters View = (FViewUniformShaderParameters)0;

struct FMaterialPixelParameters {
    float3 WorldPosition;
    float3 WorldPosition_CamRelative;
    float3 WorldNormal;
    float4 ScreenPosition;
    float2 TexCoords[4];
};
struct FMaterialVertexParameters {
    float3 WorldPosition;
    float3 WorldNormal;
    float4 TangentToWorld[3];
};
struct FPixelMaterialInputs {
    float3 EmissiveColor;
    float3 BaseColor;
};
inline float3 Luminance(float3 LinearColor) {
    return dot(LinearColor, float3(0.3, 0.59, 0.11));
}
inline float3 RotateAboutAxis(float4 NormalizedRotationAxisAndAngle, float3 PivotPoint, float3 Position) {
    float3 Axis = NormalizedRotationAxisAndAngle.xyz;
    float Angle = NormalizedRotationAxisAndAngle.w;
    return Position + sin(Angle) * cross(Axis, Position - PivotPoint);
}
inline float3 GetWorldPosition(FMaterialVertexParameters Parameters) {
    return Parameters.WorldPosition;
}
inline float CalcPixelDepth(FMaterialPixelParameters Parameters) {
    return Parameters.ScreenPosition.w;
}
#endif
"#;

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: u8,
    pub message: String,
    pub source: String,
}

pub struct ValidationTask {
    pub uri: String,
    pub content: String,
}

pub fn path_to_uri(path: &Path) -> String {
    let s = path.to_string_lossy().replace('\\', "/");
    let clean = s.strip_prefix("//?/").unwrap_or(&s);
    if clean.starts_with('/') {
        format!("file://{clean}")
    } else {
        format!("file:///{clean}")
    }
}

pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    if !uri.starts_with("file://") {
        return None;
    }
    let path_str = uri.strip_prefix("file://")?;
    #[cfg(windows)]
    {
        let trimmed = path_str.trim_start_matches('/');
        let decoded = urlencoding_decode(trimmed);
        Some(PathBuf::from(decoded))
    }
    #[cfg(not(windows))]
    {
        let decoded = urlencoding_decode(path_str);
        Some(PathBuf::from(decoded))
    }
}

fn urlencoding_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(b) = u8::from_str_radix(&hex, 16) {
                out.push(b as char);
            } else {
                out.push('%');
                out.push_str(&hex);
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn find_dxc_path() -> String {
    if let Ok(p) = env::var("DXC_PATH") {
        let trimmed = p.trim();
        if !trimmed.is_empty() && Path::new(trimmed).is_file() {
            return trimmed.to_string();
        }
    }

    #[cfg(windows)]
    let binary_name = "dxc.exe";
    #[cfg(not(windows))]
    let binary_name = "dxc";

    if let Ok(path_var) = env::var("PATH") {
        for dir in env::split_paths(&path_var) {
            let candidate = dir.join(binary_name);
            if candidate.is_file() {
                return candidate.to_string_lossy().to_string();
            }
        }
    }

    binary_name.to_string()
}

static INCLUDE_CACHE: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<PathBuf, Vec<PathBuf>>>> = std::sync::OnceLock::new();

pub fn discover_include_paths(uri: &str, workspace_root: Option<&Path>) -> Vec<PathBuf> {
    let parent_path = uri_to_path(uri).and_then(|p| p.parent().map(|p| p.to_path_buf()));
    if let Some(ref parent) = parent_path {
        let cache = INCLUDE_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
        if let Ok(guard) = cache.lock() {
            if let Some(cached) = guard.get(parent) {
                return cached.clone();
            }
        }
    }

    let mut paths = Vec::new();
    if let Some(ref parent) = parent_path {
        paths.push(parent.clone());

        let mut current = parent.as_path();
        while let Some(up) = current.parent() {
            if up.join("Assets").is_dir() && up.join("ProjectSettings").is_dir() {
                if !paths.contains(&up.to_path_buf()) {
                    paths.push(up.to_path_buf());
                }
                let assets = up.join("Assets");
                if !paths.contains(&assets) {
                    paths.push(assets);
                }
                let packages = up.join("Packages");
                if !paths.contains(&packages) {
                    paths.push(packages);
                }
                let pkg_cache = up.join("Library").join("PackageCache");
                if pkg_cache.is_dir() && !paths.contains(&pkg_cache) {
                    paths.push(pkg_cache);
                }
                break;
            }
            if up.join("Source").is_dir() || up.join("Config").is_dir() {
                if !paths.contains(&up.to_path_buf()) {
                    paths.push(up.to_path_buf());
                }
                let shaders_dir = up.join("Shaders");
                if shaders_dir.is_dir() && !paths.contains(&shaders_dir) {
                    paths.push(shaders_dir);
                }
                break;
            }
            current = up;
        }
    }

    if let Some(ws) = workspace_root {
        if !paths.contains(&ws.to_path_buf()) {
            paths.push(ws.to_path_buf());
        }
        let assets = ws.join("Assets");
        if assets.is_dir() && !paths.contains(&assets) {
            paths.push(assets);
        }
        let packages = ws.join("Packages");
        if packages.is_dir() && !paths.contains(&packages) {
            paths.push(packages);
        }
        let pkg_cache = ws.join("Library").join("PackageCache");
        if pkg_cache.is_dir() && !paths.contains(&pkg_cache) {
            paths.push(pkg_cache);
        }
    }

    if let Some(parent) = parent_path {
        let cache = INCLUDE_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
        if let Ok(mut guard) = cache.lock() {
            guard.insert(parent, paths.clone());
        }
    }

    paths
}

pub fn is_inside_hlsl_block(doc: &str, target_line: usize) -> bool {
    let mut in_block = false;
    for (idx, line) in doc.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == "HLSLPROGRAM" || trimmed == "CGPROGRAM" || trimmed == "HLSLINCLUDE" || trimmed == "CGINCLUDE" {
            in_block = true;
        } else if trimmed == "ENDHLSL" || trimmed == "ENDCG" {
            in_block = false;
        }
        if idx == target_line {
            return in_block;
        }
    }
    false
}

pub fn should_stub_include(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with("#include") {
        return false;
    }
    trimmed.contains("Packages/")
        || trimmed.contains("/Engine/")
        || trimmed.contains("/Plugin/")
        || trimmed.contains("UnityCG.cginc")
        || trimmed.contains("UnityUI.cginc")
        || trimmed.contains("AutoLight.cginc")
        || trimmed.contains("Lighting.cginc")
        || trimmed.contains("HLSLSupport.cginc")
        || trimmed.contains("TerrainEngine.cginc")
        || trimmed.contains("UnityShaderVariables.cginc")
        || trimmed.contains("UnityStandard")
        || trimmed.contains("UnityInstancing.cginc")
}

pub fn validate_shaderlab_properties_and_cbuffer(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut in_properties = false;
    let mut prop_brace_depth = 0;
    let mut properties = Vec::new();

    for (line_idx, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();

        if trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("Properties") && !in_properties {
            in_properties = true;
            prop_brace_depth = 0;
        }

        let open_b = trimmed.chars().filter(|&c| c == '{').count();
        let close_b = trimmed.chars().filter(|&c| c == '}').count();

        if in_properties {
            prop_brace_depth += open_b;

            if prop_brace_depth > 0 && !trimmed.starts_with("Properties") && !trimmed.starts_with('{') && !trimmed.starts_with('}') && !trimmed.is_empty() {
                // Check 1: Trailing semicolon warning (common HLSL habit)
                if trimmed.ends_with(';') {
                    let col_semi = raw_line.rfind(';').unwrap_or(raw_line.len().saturating_sub(1));
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_idx, character: col_semi },
                            end: Position { line: line_idx, character: col_semi + 1 },
                        },
                        severity: 2, // Warning
                        message: "ShaderLab property declarations do not use trailing semicolons ';'. Remove ';'".to_string(),
                        source: "shaderlab".to_string(),
                    });
                }

                let line_clean = trimmed.trim_end_matches(';').trim();
                let mut attr_slice = line_clean;
                let mut unclosed_attr = false;
                let mut parsed_attrs: Vec<&str> = Vec::new();

                while attr_slice.starts_with('[') {
                    if let Some(end_bracket) = attr_slice.find(']') {
                        let inside = attr_slice[1..end_bracket].trim();
                        for single in inside.split(',') {
                            let clean = single.trim();
                            if !clean.is_empty() {
                                parsed_attrs.push(clean);
                            }
                        }
                        attr_slice = attr_slice[end_bracket + 1..].trim();
                    } else {
                        unclosed_attr = true;
                        break;
                    }
                }

                if unclosed_attr {
                    let col_b = raw_line.find('[').unwrap_or(0);
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_idx, character: col_b },
                            end: Position { line: line_idx, character: raw_line.len() },
                        },
                        severity: 1, // Error
                        message: "Unclosed attribute bracket '['. Expected closing ']'.".to_string(),
                        source: "shaderlab".to_string(),
                    });
                }
                let without_attr = attr_slice;

                // Check for missing property name before '(' (e.g. `("Float", Float) = 0.0`)
                if without_attr.starts_with('(') {
                    let col = raw_line.find('(').unwrap_or(0);
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_idx, character: col },
                            end: Position { line: line_idx, character: col + without_attr.len() },
                        },
                        severity: 1, // Error
                        message: "Missing property name before '('. Expected syntax: _PropertyName (\"DisplayName\", Type) = defaultValue".to_string(),
                        source: "shaderlab".to_string(),
                    });
                } else if without_attr.contains('(') {
                    let name = without_attr.split('(').next().unwrap_or("").trim();
                    if !signature::is_valid_identifier(name) {
                        let col = raw_line.find(name).unwrap_or(0);
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: line_idx, character: col },
                                end: Position { line: line_idx, character: col + name.len().max(1) },
                            },
                            severity: 1, // Error
                            message: format!("Invalid property name '{name}'. Property names must start with an underscore or letter."),
                            source: "shaderlab".to_string(),
                        });
                    } else {
                        // Check for duplicate property names
                        if properties.iter().any(|(p_name, ..): &(String, _, _, _)| p_name == name) {
                            let col = raw_line.find(name).unwrap_or(0);
                            diagnostics.push(Diagnostic {
                                range: Range {
                                    start: Position { line: line_idx, character: col },
                                    end: Position { line: line_idx, character: col + name.len().max(1) },
                                },
                                severity: 1, // Error
                                message: format!("Duplicate property name '{name}' in Properties block."),
                                source: "shaderlab".to_string(),
                            });
                        }

                        if let Some(paren_idx) = without_attr.find('(') {
                            let after_paren = &without_attr[paren_idx + 1..];
                            let mut p_type = String::new();

                            // Display name quote check
                            if let Some(first_quote) = after_paren.find('"') {
                                if let Some(second_quote) = after_paren[first_quote + 1..].find('"') {
                                    let after_second = &after_paren[first_quote + 1 + second_quote + 1..];
                                    if let Some(comma_idx) = after_second.find(',') {
                                        let rest = after_second[comma_idx + 1..].trim();
                                        let mut paren_count = 0;
                                        let mut type_end = rest.len();
                                        for (idx, ch) in rest.char_indices() {
                                            if ch == '(' {
                                                paren_count += 1;
                                            } else if ch == ')' {
                                                if paren_count == 0 {
                                                    type_end = idx;
                                                    break;
                                                } else {
                                                    paren_count -= 1;
                                                }
                                            }
                                        }
                                        p_type = rest[..type_end].trim().to_string();
                                    }
                                } else {
                                    let col_q = raw_line.find('"').unwrap_or(0);
                                    diagnostics.push(Diagnostic {
                                        range: Range {
                                            start: Position { line: line_idx, character: col_q },
                                            end: Position { line: line_idx, character: raw_line.len() },
                                        },
                                        severity: 1,
                                        message: "Unclosed string quote in property display name.".to_string(),
                                        source: "shaderlab".to_string(),
                                    });
                                }
                            } else if let Some(close_paren) = after_paren.find(')') {
                                let inside_paren = &after_paren[..close_paren];
                                let parts: Vec<&str> = inside_paren.split(',').collect();
                                p_type = parts.get(1).map(|s| s.trim().to_string()).unwrap_or_default();
                            }

                            if !p_type.is_empty() {
                                let col = raw_line.find(name).unwrap_or(0);
                                properties.push((name.to_string(), p_type.clone(), line_idx, col));

                                let base_type = p_type.split('(').next().unwrap_or(&p_type).trim();
                                const VALID_TYPES: &[&str] = &["Color", "Vector", "Float", "Int", "Integer", "Range", "2D", "3D", "Cube", "2DArray", "Rect"];

                                // Validate property type
                                if !VALID_TYPES.contains(&base_type) {
                                    let col_t = raw_line.find(&p_type).unwrap_or(col);
                                    let suggestion = match base_type {
                                        "float4" | "half4" | "Vector4" | "Vector3" | "float3" => " Did you mean 'Vector' or 'Color'?",
                                        "Texture" | "Texture2D" | "sampler2D" => " Did you mean '2D'?",
                                        "TextureCube" | "samplerCUBE" => " Did you mean 'Cube'?",
                                        "bool" | "Boolean" => " ShaderLab has no 'bool' property type. Use 'Float' with a [Toggle] attribute.",
                                        "float" | "half" | "double" => " Did you mean 'Float'?",
                                        "int" | "uint" => " Did you mean 'Int'?",
                                        "float4x4" | "Matrix" | "matrix" => " ShaderLab Properties block does not support matrices (float4x4). Declare it in HLSL cbuffer and set via C# material.SetMatrix().",
                                        s if s.contains('[') || s.contains("Array") => " ShaderLab Properties block does not support arrays. Declare the array in HLSL and set via C# material.SetFloatArray() / material.SetMatrixArray().",
                                        _ => " Valid ShaderLab types: Color, Float, Int, Range(min, max), 2D, 3D, Cube, 2DArray, Vector, Rect.",
                                    };
                                    diagnostics.push(Diagnostic {
                                        range: Range {
                                            start: Position { line: line_idx, character: col_t },
                                            end: Position { line: line_idx, character: col_t + p_type.len() },
                                        },
                                        severity: 1,
                                        message: format!("Unknown property type '{p_type}'.{suggestion}"),
                                        source: "shaderlab".to_string(),
                                    });
                                }

                                // Validate attribute compatibility
                                const KNOWN_ATTRS: &[&str] = &[
                                    "MainColor", "MainTexture", "HDR", "HideInInspector", "NoScaleOffset",
                                    "Normal", "SingleLineTexture", "PerRendererData", "MaterialToggle",
                                    "Toggle", "KeywordEnum", "Enum", "Space", "Header", "IntRange", "Tooltip"
                                ];
                                for attr in &parsed_attrs {
                                    let attr_ident = attr.split('(').next().unwrap_or(attr).trim();
                                    let col_attr = raw_line.find(attr_ident).unwrap_or(col);

                                    if !KNOWN_ATTRS.contains(&attr_ident) {
                                        let typo_hint = if attr_ident.eq_ignore_ascii_case("MainColour") {
                                            " Did you mean '[MainColor]'?"
                                        } else if attr_ident.eq_ignore_ascii_case("MainTex") {
                                            " Did you mean '[MainTexture]'?"
                                        } else {
                                            ""
                                        };
                                        diagnostics.push(Diagnostic {
                                            range: Range {
                                                start: Position { line: line_idx, character: col_attr },
                                                end: Position { line: line_idx, character: col_attr + attr_ident.len() },
                                            },
                                            severity: 2, // Warning
                                            message: format!("Unknown ShaderLab attribute '[{attr_ident}]'.{typo_hint}"),
                                            source: "shaderlab".to_string(),
                                        });
                                    }

                                    if attr_ident == "Normal" && base_type != "2D" {
                                        diagnostics.push(Diagnostic {
                                            range: Range {
                                                start: Position { line: line_idx, character: col_attr },
                                                end: Position { line: line_idx, character: col_attr + attr_ident.len() },
                                            },
                                            severity: 2,
                                            message: format!("[Normal] attribute can only be applied to 2D texture properties, but '{name}' is of type '{p_type}'."),
                                            source: "shaderlab".to_string(),
                                        });
                                    } else if attr_ident == "HDR" && base_type != "Color" {
                                        diagnostics.push(Diagnostic {
                                            range: Range {
                                                start: Position { line: line_idx, character: col_attr },
                                                end: Position { line: line_idx, character: col_attr + attr_ident.len() },
                                            },
                                            severity: 2,
                                            message: format!("[HDR] attribute can only be applied to Color properties, but '{name}' is of type '{p_type}'."),
                                            source: "shaderlab".to_string(),
                                        });
                                    } else if attr_ident == "IntRange" && base_type != "Range" {
                                        diagnostics.push(Diagnostic {
                                            range: Range {
                                                start: Position { line: line_idx, character: col_attr },
                                                end: Position { line: line_idx, character: col_attr + attr_ident.len() },
                                            },
                                            severity: 2,
                                            message: format!("[IntRange] attribute can only be applied to Range properties, but '{name}' is of type '{p_type}'."),
                                            source: "shaderlab".to_string(),
                                        });
                                    } else if (attr_ident == "SingleLineTexture" || attr_ident == "NoScaleOffset") && !["2D", "3D", "Cube", "2DArray", "Rect"].contains(&base_type) {
                                        diagnostics.push(Diagnostic {
                                            range: Range {
                                                start: Position { line: line_idx, character: col_attr },
                                                end: Position { line: line_idx, character: col_attr + attr_ident.len() },
                                            },
                                            severity: 2,
                                            message: format!("[{attr_ident}] attribute can only be applied to texture properties (2D, 3D, Cube), but '{name}' is of type '{p_type}'."),
                                            source: "shaderlab".to_string(),
                                        });
                                    } else if attr_ident == "MainColor" && base_type != "Color" {
                                        diagnostics.push(Diagnostic {
                                            range: Range {
                                                start: Position { line: line_idx, character: col_attr },
                                                end: Position { line: line_idx, character: col_attr + attr_ident.len() },
                                            },
                                            severity: 2,
                                            message: format!("[MainColor] attribute is intended for Color properties, but '{name}' is of type '{p_type}'."),
                                            source: "shaderlab".to_string(),
                                        });
                                    } else if attr_ident == "MainTexture" && base_type != "2D" {
                                        diagnostics.push(Diagnostic {
                                            range: Range {
                                                start: Position { line: line_idx, character: col_attr },
                                                end: Position { line: line_idx, character: col_attr + attr_ident.len() },
                                            },
                                            severity: 2,
                                            message: format!("[MainTexture] attribute is intended for 2D texture properties, but '{name}' is of type '{p_type}'."),
                                            source: "shaderlab".to_string(),
                                        });
                                    }
                                }

                                // Range syntax and bounds checking
                                let mut range_bounds: Option<(f64, f64)> = None;
                                if base_type == "Range" {
                                    if !p_type.contains('(') || !p_type.contains(')') {
                                        let col_t = raw_line.find(&p_type).unwrap_or(col);
                                        diagnostics.push(Diagnostic {
                                            range: Range {
                                                start: Position { line: line_idx, character: col_t },
                                                end: Position { line: line_idx, character: col_t + p_type.len() },
                                            },
                                            severity: 1,
                                            message: format!("Range property '{name}' requires min and max limits: Range(min, max). Example: Range(0.0, 1.0)"),
                                            source: "shaderlab".to_string(),
                                        });
                                    } else if let Some(r_open) = p_type.find('(') {
                                        if let Some(r_close) = p_type[r_open..].find(')') {
                                            let r_content = &p_type[r_open + 1..r_open + r_close];
                                            let r_parts: Vec<&str> = r_content.split(',').map(|s| s.trim()).collect();
                                            if r_parts.len() != 2 {
                                                let col_t = raw_line.find(&p_type).unwrap_or(col);
                                                diagnostics.push(Diagnostic {
                                                    range: Range {
                                                        start: Position { line: line_idx, character: col_t },
                                                        end: Position { line: line_idx, character: col_t + p_type.len() },
                                                    },
                                                    severity: 1,
                                                    message: format!("Invalid Range syntax '{p_type}'. Expected exactly two limits: Range(min, max)."),
                                                    source: "shaderlab".to_string(),
                                                });
                                            } else {
                                                let min_res = r_parts[0].parse::<f64>();
                                                let max_res = r_parts[1].parse::<f64>();
                                                match (min_res, max_res) {
                                                    (Ok(min_v), Ok(max_v)) => {
                                                        if min_v > max_v {
                                                            let col_t = raw_line.find(&p_type).unwrap_or(col);
                                                            diagnostics.push(Diagnostic {
                                                                range: Range {
                                                                    start: Position { line: line_idx, character: col_t },
                                                                    end: Position { line: line_idx, character: col_t + p_type.len() },
                                                                },
                                                                severity: 1,
                                                                message: format!("Invalid Range limits for '{name}': min ({min_v}) is greater than max ({max_v})."),
                                                                source: "shaderlab".to_string(),
                                                            });
                                                        } else {
                                                            range_bounds = Some((min_v, max_v));
                                                        }
                                                    }
                                                    _ => {
                                                        let col_t = raw_line.find(&p_type).unwrap_or(col);
                                                        diagnostics.push(Diagnostic {
                                                            range: Range {
                                                                start: Position { line: line_idx, character: col_t },
                                                                end: Position { line: line_idx, character: col_t + p_type.len() },
                                                            },
                                                            severity: 1,
                                                            message: format!("Non-numeric bounds in Range '{p_type}'. Both min and max must be valid numbers."),
                                                            source: "shaderlab".to_string(),
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Validate '=' and default value
                                if let Some(eq_idx) = without_attr.find('=') {
                                    let val_str = without_attr[eq_idx + 1..].trim();
                                    if val_str.is_empty() {
                                        let col_eq = raw_line.rfind('=').unwrap_or(raw_line.len().saturating_sub(1));
                                        diagnostics.push(Diagnostic {
                                            range: Range {
                                                start: Position { line: line_idx, character: col_eq },
                                                end: Position { line: line_idx, character: raw_line.len() },
                                            },
                                            severity: 1, // Error
                                            message: format!("Missing default value after '=' for {p_type} property '{name}'."),
                                            source: "shaderlab".to_string(),
                                        });
                                    } else {
                                        let col_val = raw_line.find(val_str).unwrap_or(col);

                                        if base_type == "Color" || base_type == "Vector" {
                                            if val_str.starts_with('"') {
                                                diagnostics.push(Diagnostic {
                                                    range: Range {
                                                        start: Position { line: line_idx, character: col_val },
                                                        end: Position { line: line_idx, character: col_val + val_str.len() },
                                                    },
                                                    severity: 1,
                                                    message: format!("Type mismatch: {p_type} property '{name}' cannot take texture default value '{val_str}'. Expected vector literal (x, y, z, w)."),
                                                    source: "shaderlab".to_string(),
                                                });
                                            } else if !val_str.starts_with('(') || !val_str.ends_with(')') {
                                                diagnostics.push(Diagnostic {
                                                    range: Range {
                                                        start: Position { line: line_idx, character: col_val },
                                                        end: Position { line: line_idx, character: col_val + val_str.len() },
                                                    },
                                                    severity: 1,
                                                    message: format!("Malformed {p_type} literal '{val_str}'. Expected format: (x, y, z, w)"),
                                                    source: "shaderlab".to_string(),
                                                });
                                            } else {
                                                let inner = val_str[1..val_str.len() - 1].trim();
                                                let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                                                let has_empty = parts.iter().any(|s| s.is_empty());
                                                if has_empty {
                                                    diagnostics.push(Diagnostic {
                                                        range: Range {
                                                            start: Position { line: line_idx, character: col_val },
                                                            end: Position { line: line_idx, character: col_val + val_str.len() },
                                                        },
                                                        severity: 1,
                                                        message: format!("Malformed {p_type} literal '{val_str}'. Trailing comma or empty component."),
                                                        source: "shaderlab".to_string(),
                                                    });
                                                } else if parts.len() != 4 {
                                                    diagnostics.push(Diagnostic {
                                                        range: Range {
                                                            start: Position { line: line_idx, character: col_val },
                                                            end: Position { line: line_idx, character: col_val + val_str.len() },
                                                        },
                                                        severity: 1,
                                                        message: format!("{p_type} property '{name}' requires 4 components (R, G, B, A), but found {}.", parts.len()),
                                                        source: "shaderlab".to_string(),
                                                    });
                                                } else {
                                                    for (c_idx, p) in parts.iter().enumerate() {
                                                        if p.parse::<f64>().is_err() {
                                                            diagnostics.push(Diagnostic {
                                                                range: Range {
                                                                    start: Position { line: line_idx, character: col_val },
                                                                    end: Position { line: line_idx, character: col_val + val_str.len() },
                                                                },
                                                                severity: 1,
                                                                message: format!("Invalid numeric value '{p}' in component {} of {p_type} property '{name}'.", c_idx + 1),
                                                                source: "shaderlab".to_string(),
                                                            });
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                        } else if base_type == "Float" || base_type == "Int" || base_type == "Integer" || base_type == "Range" {
                                            if val_str.starts_with('(') {
                                                diagnostics.push(Diagnostic {
                                                    range: Range {
                                                        start: Position { line: line_idx, character: col_val },
                                                        end: Position { line: line_idx, character: col_val + val_str.len() },
                                                    },
                                                    severity: 1,
                                                    message: format!("Type mismatch: scalar property '{name}' of type {p_type} cannot take vector default value '{val_str}'. Expected a numeric scalar (e.g. 0.0)."),
                                                    source: "shaderlab".to_string(),
                                                });
                                            } else if val_str.starts_with('"') {
                                                diagnostics.push(Diagnostic {
                                                    range: Range {
                                                        start: Position { line: line_idx, character: col_val },
                                                        end: Position { line: line_idx, character: col_val + val_str.len() },
                                                    },
                                                    severity: 1,
                                                    message: format!("Type mismatch: scalar property '{name}' of type {p_type} cannot take texture default value '{val_str}'. Expected a numeric scalar (e.g. 0.0)."),
                                                    source: "shaderlab".to_string(),
                                                });
                                            } else if let Ok(num_val) = val_str.parse::<f64>() {
                                                if let Some((min_v, max_v)) = range_bounds {
                                                    if num_val < min_v || num_val > max_v {
                                                        diagnostics.push(Diagnostic {
                                                            range: Range {
                                                                start: Position { line: line_idx, character: col_val },
                                                                end: Position { line: line_idx, character: col_val + val_str.len() },
                                                            },
                                                            severity: 2, // Warning
                                                            message: format!("Default value '{val_str}' for Range property '{name}' is outside the range [{min_v}, {max_v}]. It will be clamped in the Inspector."),
                                                            source: "shaderlab".to_string(),
                                                        });
                                                    }
                                                }
                                            } else {
                                                diagnostics.push(Diagnostic {
                                                    range: Range {
                                                        start: Position { line: line_idx, character: col_val },
                                                        end: Position { line: line_idx, character: col_val + val_str.len() },
                                                    },
                                                    severity: 1,
                                                    message: format!("Invalid numeric default value '{val_str}' for {p_type} property '{name}'."),
                                                    source: "shaderlab".to_string(),
                                                });
                                            }
                                        } else if ["2D", "3D", "Cube", "2DArray", "Rect"].contains(&base_type) {
                                            if val_str.starts_with('(') {
                                                diagnostics.push(Diagnostic {
                                                    range: Range {
                                                        start: Position { line: line_idx, character: col_val },
                                                        end: Position { line: line_idx, character: col_val + val_str.len() },
                                                    },
                                                    severity: 1,
                                                    message: format!("Type mismatch: texture property '{name}' of type {p_type} cannot take vector default value '{val_str}'. Expected texture default like \"white\" {{}}."),
                                                    source: "shaderlab".to_string(),
                                                });
                                            } else if val_str.parse::<f64>().is_ok() {
                                                diagnostics.push(Diagnostic {
                                                    range: Range {
                                                        start: Position { line: line_idx, character: col_val },
                                                        end: Position { line: line_idx, character: col_val + val_str.len() },
                                                    },
                                                    severity: 1,
                                                    message: format!("Type mismatch: texture property '{name}' of type {p_type} cannot take numeric default value '{val_str}'. Expected texture default like \"white\" {{}}."),
                                                    source: "shaderlab".to_string(),
                                                });
                                            } else if !val_str.contains('{') || !val_str.contains('}') {
                                                diagnostics.push(Diagnostic {
                                                    range: Range {
                                                        start: Position { line: line_idx, character: col_val },
                                                        end: Position { line: line_idx, character: col_val + val_str.len() },
                                                    },
                                                    severity: 1,
                                                    message: format!("Invalid texture default value '{val_str}' for property '{name}'. Expected syntax: \"white\" {{}} or \"\" {{}}"),
                                                    source: "shaderlab".to_string(),
                                                });
                                            } else if let Some(first_q) = val_str.find('"') {
                                                if let Some(second_q) = val_str[first_q + 1..].find('"') {
                                                    let tex_name = &val_str[first_q + 1..first_q + 1 + second_q];
                                                    const VALID_TEX: &[&str] = &["white", "black", "gray", "bump", "red", "", "_Skybox"];
                                                    if !VALID_TEX.contains(&tex_name) {
                                                        diagnostics.push(Diagnostic {
                                                            range: Range {
                                                                start: Position { line: line_idx, character: col_val + first_q + 1 },
                                                                end: Position { line: line_idx, character: col_val + first_q + 1 + tex_name.len() },
                                                            },
                                                            severity: 2, // Warning
                                                            message: format!("Unknown built-in texture default '{tex_name}'. Standard Unity texture defaults are: \"white\", \"black\", \"gray\", \"bump\", \"red\", or \"\"."),
                                                            source: "shaderlab".to_string(),
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    let col_name = raw_line.find(name).unwrap_or(0);
                                    diagnostics.push(Diagnostic {
                                        range: Range {
                                            start: Position { line: line_idx, character: col_name },
                                            end: Position { line: line_idx, character: raw_line.len() },
                                        },
                                        severity: 1,
                                        message: format!("Missing '=' in property declaration for '{name}'. Expected: {name} (\"...\", {p_type}) = defaultValue"),
                                        source: "shaderlab".to_string(),
                                    });
                                }
                            }
                        }
                    }
                } else if !without_attr.is_empty() {
                    let col = raw_line.find(without_attr).unwrap_or(0);
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: line_idx, character: col },
                            end: Position { line: line_idx, character: col + without_attr.len() },
                        },
                        severity: 1, // Error
                        message: "Incomplete property declaration. Expected syntax: _PropertyName (\"DisplayName\", Type) = defaultValue".to_string(),
                        source: "shaderlab".to_string(),
                    });
                }
            }

            if prop_brace_depth > 0 && prop_brace_depth <= close_b {
                in_properties = false;
                prop_brace_depth = 0;
            } else {
                prop_brace_depth = prop_brace_depth.saturating_sub(close_b);
            }
        }
    }

    // Check CBuffer / HLSL declarations for properties
    let is_urp = content.contains("UniversalPipeline")
        || content.contains("com.unity.render-pipelines.universal")
        || content.contains("UniversalForward")
        || content.contains("Universal2D");

    let has_cbuffer = content.contains("CBUFFER_START") || content.contains("cbuffer ");

    // Extract cbuffer variable lines
    let mut in_cb = false;
    let mut cbuffer_vars = Vec::new();
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("CBUFFER_START") || t.starts_with("cbuffer ") {
            in_cb = true;
            continue;
        }
        if t.starts_with("CBUFFER_END") || (in_cb && t.starts_with("};")) {
            in_cb = false;
            continue;
        }
        if in_cb && t.ends_with(';') && !t.starts_with("//") {
            cbuffer_vars.push(t);
        }
    }

    if is_urp && !properties.is_empty() && !has_cbuffer {
        let first_prop = &properties[0];
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position { line: first_prop.2, character: first_prop.3 },
                end: Position { line: first_prop.2, character: first_prop.3 + first_prop.0.len() },
            },
            severity: 2, // Warning
            message: "URP shader with material properties should define 'CBUFFER_START(UnityPerMaterial) ... CBUFFER_END' for SRP Batcher compatibility.".to_string(),
            source: "unity-urp".to_string(),
        });
    } else if is_urp && has_cbuffer {
        for (name, p_type, line_idx, col) in &properties {
            let is_texture = ["2D", "3D", "Cube", "2DArray"].contains(&p_type.as_str());
            if is_texture {
                let tex_macro = format!("TEXTURE2D({name})");
                let sampler_old = format!("sampler2D {name};");
                let tex_old = format!("Texture2D {name};");
                let tex_cube_macro = format!("TEXTURECUBE({name})");
                let tex_cube_old = format!("TextureCube {name};");
                let tex_3d_macro = format!("TEXTURE3D({name})");
                let tex_array_macro = format!("TEXTURE2D_ARRAY({name})");
                if !content.contains(&tex_macro)
                    && !content.contains(&sampler_old)
                    && !content.contains(&tex_old)
                    && !content.contains(&tex_cube_macro)
                    && !content.contains(&tex_cube_old)
                    && !content.contains(&tex_3d_macro)
                    && !content.contains(&tex_array_macro)
                {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: *line_idx, character: *col },
                            end: Position { line: *line_idx, character: *col + name.len() },
                        },
                        severity: 2, // Warning
                        message: format!("Texture property '{name}' is defined in Properties but texture declaration is missing in HLSL (e.g. 'TEXTURE2D({name}); SAMPLER(sampler_{name});')."),
                        source: "unity-urp".to_string(),
                    });
                }
            } else {
                let found = cbuffer_vars.iter().any(|line| {
                    line.split_whitespace().any(|tok| tok.trim_matches(|c| c == ';' || c == ',') == name.as_str())
                });
                if !found {
                    let suggested_type = match p_type.as_str() {
                        "Float" => "float",
                        s if s.starts_with("Range") => "float",
                        "Int" => "int",
                        "Color" => "half4",
                        "Vector" => "float4",
                        _ => "float4",
                    };
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: *line_idx, character: *col },
                            end: Position { line: *line_idx, character: *col + name.len() },
                        },
                        severity: 2, // Warning
                        message: format!("Property '{name}' is defined in Properties but not declared in 'CBUFFER_START(UnityPerMaterial)'. Add '{suggested_type} {name};' to CBuffer for SRP Batcher compatibility."),
                        source: "unity-urp".to_string(),
                    });
                }
            }
        }
    }

    diagnostics
}

pub fn validate_shaderlab_tags(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut in_tags = false;

    for (line_idx, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with("//") {
            continue;
        }

        if trimmed.contains("Tags") && trimmed.contains('{') {
            in_tags = true;
        }

        if in_tags {
            // Check for unclosed quotes
            let quote_count = trimmed.chars().filter(|&c| c == '"').count();
            if quote_count % 2 != 0 {
                let col = raw_line.rfind('"').unwrap_or(0);
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx, character: col },
                        end: Position { line: line_idx, character: raw_line.len() },
                    },
                    severity: 1, // Error
                    message: "Unclosed quote string in Tags block".to_string(),
                    source: "shaderlab".to_string(),
                });
            }

            // Check for missing value after '=' e.g. "RenderType" = } or "RenderType" = \n
            if trimmed.contains('=') {
                let mut after_eq = trimmed;
                while let Some(eq_idx) = after_eq.find('=') {
                    let before = after_eq[..eq_idx].trim();
                    let rest = after_eq[eq_idx + 1..].trim();

                    if rest.is_empty() || rest.starts_with('}') {
                        let col = raw_line.rfind('=').unwrap_or(0);
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: line_idx, character: col },
                                end: Position { line: line_idx, character: col + 1 },
                            },
                            severity: 1,
                            message: "Missing tag value after '=' in Tags block".to_string(),
                            source: "shaderlab".to_string(),
                        });
                        break;
                    } else if let Some(stripped) = rest.strip_prefix('"') {
                        if let Some(val_end) = stripped.find('"') {
                            let tag_val = &stripped[..val_end];
                            let key = before.rsplit('"').nth(1).unwrap_or(before);

                            let col = raw_line.find(tag_val).unwrap_or(0);
                            for (known_key, valid_vals, _desc) in docs::SHADERLAB_TAG_KEYS_AND_VALUES {
                                if *known_key == key {
                                    let matches_exact = valid_vals.iter().any(|v| v.trim_matches('"') == tag_val);
                                    if !matches_exact {
                                        if let Some(suggestion) = valid_vals.iter().find(|v| v.trim_matches('"').eq_ignore_ascii_case(tag_val)) {
                                            let sug_clean = suggestion.trim_matches('"');
                                            diagnostics.push(Diagnostic {
                                                range: Range {
                                                    start: Position { line: line_idx, character: col },
                                                    end: Position { line: line_idx, character: col + tag_val.len() },
                                                },
                                                severity: 2, // Warning
                                                message: format!("Unknown {key} '{tag_val}'. Did you mean '{sug_clean}'? (Tag values are case-sensitive)"),
                                                source: "shaderlab".to_string(),
                                            });
                                        } else if key == "RenderType" && tag_val == "Opque" {
                                            diagnostics.push(Diagnostic {
                                                range: Range {
                                                    start: Position { line: line_idx, character: col },
                                                    end: Position { line: line_idx, character: col + tag_val.len() },
                                                },
                                                severity: 2,
                                                message: "Unknown RenderType 'Opque'. Did you mean 'Opaque'?".to_string(),
                                                source: "shaderlab".to_string(),
                                            });
                                        } else if key == "RenderPipeline" && (tag_val == "Universal" || tag_val.eq_ignore_ascii_case("URP")) {
                                            diagnostics.push(Diagnostic {
                                                range: Range {
                                                    start: Position { line: line_idx, character: col },
                                                    end: Position { line: line_idx, character: col + tag_val.len() },
                                                },
                                                severity: 2,
                                                message: "Unknown RenderPipeline. Did you mean 'UniversalPipeline'?".to_string(),
                                                source: "shaderlab".to_string(),
                                            });
                                        } else if (key == "IgnoreProjector" || key == "CanUseSpriteAtlas") && (tag_val == "true" || tag_val == "false") {
                                            let sug = if tag_val == "true" { "True" } else { "False" };
                                            diagnostics.push(Diagnostic {
                                                range: Range {
                                                    start: Position { line: line_idx, character: col },
                                                    end: Position { line: line_idx, character: col + tag_val.len() },
                                                },
                                                severity: 2,
                                                message: format!("Tag '{key}' requires capitalized '{sug}' in ShaderLab."),
                                                source: "shaderlab".to_string(),
                                            });
                                        }
                                    }
                                    break;
                                }
                            }
                            let next_start = val_end + 2;
                            if next_start < rest.len() {
                                after_eq = &rest[next_start..];
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }

            if trimmed.contains('}') {
                in_tags = false;
            }
        }
    }

    diagnostics
}

pub fn validate_shaderlab_render_states(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut in_hlsl = false;
    let mut in_properties = false;
    let mut prop_depth = 0;

    for (line_idx, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with("//") {
            continue;
        }

        if trimmed == "HLSLPROGRAM" || trimmed == "CGPROGRAM" || trimmed == "HLSLINCLUDE" || trimmed == "CGINCLUDE" {
            in_hlsl = true;
            continue;
        }
        if trimmed == "ENDHLSL" || trimmed == "ENDCG" {
            in_hlsl = false;
            continue;
        }
        if in_hlsl {
            continue;
        }

        if trimmed.starts_with("Properties") && trimmed.contains('{') {
            in_properties = true;
            prop_depth = 1;
            continue;
        }
        if in_properties {
            let opens = trimmed.chars().filter(|&c| c == '{').count();
            let closes = trimmed.chars().filter(|&c| c == '}').count();
            prop_depth += opens;
            if prop_depth <= closes {
                in_properties = false;
                prop_depth = 0;
            } else {
                prop_depth -= closes;
            }
            continue;
        }

        // 1. Cull
        if trimmed.starts_with("Cull") && (trimmed.len() == 4 || trimmed[4..].starts_with(' ') || trimmed[4..].starts_with('\t')) {
            let val = trimmed[4..].trim();
            let col = raw_line.find("Cull").unwrap_or(0);
            if val.is_empty() {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx, character: col },
                        end: Position { line: line_idx, character: raw_line.len() },
                    },
                    severity: 1,
                    message: "Missing Cull mode. Expected: Cull Off, Cull Front, or Cull Back.".to_string(),
                    source: "shaderlab".to_string(),
                });
            } else if !val.starts_with('[') && !["Off", "Front", "Back"].contains(&val) {
                let col_val = raw_line.find(val).unwrap_or(col + 5);
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx, character: col_val },
                        end: Position { line: line_idx, character: col_val + val.len() },
                    },
                    severity: 1,
                    message: format!("Unknown Cull mode '{val}'. Valid options: Off, Front, Back (or dynamic property '[_Cull]')."),
                    source: "shaderlab".to_string(),
                });
            }
        }

        // 2. ZWrite
        if trimmed.starts_with("ZWrite") && (trimmed.len() == 6 || trimmed[6..].starts_with(' ') || trimmed[6..].starts_with('\t')) {
            let val = trimmed[6..].trim();
            let col = raw_line.find("ZWrite").unwrap_or(0);
            if val.is_empty() {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx, character: col },
                        end: Position { line: line_idx, character: raw_line.len() },
                    },
                    severity: 1,
                    message: "Missing ZWrite value. Expected: ZWrite On or ZWrite Off.".to_string(),
                    source: "shaderlab".to_string(),
                });
            } else if !val.starts_with('[') && !["On", "Off"].contains(&val) {
                let col_val = raw_line.find(val).unwrap_or(col + 7);
                let hint = if val.eq_ignore_ascii_case("true") || val == "1" {
                    " Did you mean 'On'?"
                } else if val.eq_ignore_ascii_case("false") || val == "0" {
                    " Did you mean 'Off'?"
                } else {
                    ""
                };
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx, character: col_val },
                        end: Position { line: line_idx, character: col_val + val.len() },
                    },
                    severity: 1,
                    message: format!("Invalid ZWrite value '{val}'. Expected 'On' or 'Off'.{hint}"),
                    source: "shaderlab".to_string(),
                });
            }
        }

        // 3. ZTest
        if trimmed.starts_with("ZTest") && (trimmed.len() == 5 || trimmed[5..].starts_with(' ') || trimmed[5..].starts_with('\t')) {
            let val = trimmed[5..].trim();
            let col = raw_line.find("ZTest").unwrap_or(0);
            const VALID_ZTEST: &[&str] = &["Less", "Greater", "LEqual", "GEqual", "Equal", "NotEqual", "Always", "Off"];
            if val.is_empty() {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx, character: col },
                        end: Position { line: line_idx, character: raw_line.len() },
                    },
                    severity: 1,
                    message: "Missing ZTest mode. Expected: ZTest LEqual, Less, Greater, GEqual, Equal, NotEqual, Always, or Off.".to_string(),
                    source: "shaderlab".to_string(),
                });
            } else if !val.starts_with('[') && !VALID_ZTEST.contains(&val) {
                let col_val = raw_line.find(val).unwrap_or(col + 6);
                let sug = VALID_ZTEST.iter().find(|m| m.eq_ignore_ascii_case(val));
                let hint = if let Some(s) = sug {
                    format!(" Did you mean '{s}'?")
                } else {
                    "".to_string()
                };
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx, character: col_val },
                        end: Position { line: line_idx, character: col_val + val.len() },
                    },
                    severity: 1,
                    message: format!("Unknown ZTest comparison mode '{val}'. Valid options: LEqual, Less, Greater, GEqual, Equal, NotEqual, Always, Off.{hint}"),
                    source: "shaderlab".to_string(),
                });
            }
        }

        // 4. ColorMask
        if trimmed.starts_with("ColorMask") && (trimmed.len() == 9 || trimmed[9..].starts_with(' ') || trimmed[9..].starts_with('\t')) {
            let val = trimmed[9..].trim();
            let col = raw_line.find("ColorMask").unwrap_or(0);
            if val.is_empty() {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx, character: col },
                        end: Position { line: line_idx, character: raw_line.len() },
                    },
                    severity: 1,
                    message: "Missing ColorMask value. Expected: ColorMask RGBA, RGB, A, 0, or channel combination.".to_string(),
                    source: "shaderlab".to_string(),
                });
            } else if !val.starts_with('[') && val != "0" && !val.chars().all(|c| ['R', 'G', 'B', 'A'].contains(&c)) {
                let col_val = raw_line.find(val).unwrap_or(col + 10);
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx, character: col_val },
                        end: Position { line: line_idx, character: col_val + val.len() },
                    },
                    severity: 1,
                    message: format!("Invalid ColorMask '{val}'. Must be combinations of R, G, B, A, or 0 (e.g. RGBA, RGB, 0)."),
                    source: "shaderlab".to_string(),
                });
            }
        }
    }

    diagnostics
}

pub fn validate_shader(
    uri: &str,
    content: &str,
    dxc_path: &str,
    workspace_root: Option<&Path>,
) -> Vec<Diagnostic> {
    let context = detect_shader_context(uri, content);
    let mut diagnostics = Vec::new();
    let include_dirs = discover_include_paths(uri, workspace_root);

    match context {
        ShaderContext::UnityShaderLab => {
            diagnostics.extend(validate_shaderlab_properties_and_cbuffer(content));
            diagnostics.extend(validate_shaderlab_tags(content));
            diagnostics.extend(validate_shaderlab_render_states(content));

            // Extract shared HLSLINCLUDE / CGINCLUDE code across passes
            let mut include_lines = Vec::new();
            let mut in_include = false;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed == "HLSLINCLUDE" || trimmed == "CGINCLUDE" {
                    in_include = true;
                    continue;
                }
                if trimmed == "ENDHLSL" || trimmed == "ENDCG" {
                    in_include = false;
                    continue;
                }
                if in_include {
                    if should_stub_include(trimmed) {
                        include_lines.push(format!("// {}", line));
                    } else {
                        include_lines.push(line.to_string());
                    }
                }
            }
            let shared_include = include_lines.join("\n");

            let mut in_block = false;
            let mut block_start_line = 0;
            let mut block_lines = Vec::new();

            for (line_idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed == "HLSLPROGRAM" || trimmed == "CGPROGRAM" {
                    in_block = true;
                    block_start_line = line_idx + 1;
                    block_lines.clear();
                    continue;
                }
                if trimmed == "ENDHLSL" || trimmed == "ENDCG" {
                    if in_block {
                        in_block = false;
                        let mut block_code = String::from(UNITY_COMPAT_PREAMBLE.trim_start());
                        if !block_code.ends_with('\n') {
                            block_code.push('\n');
                        }
                        if !shared_include.is_empty() {
                            block_code.push_str(&shared_include);
                            block_code.push('\n');
                        }
                        let preamble_line_count = block_code.lines().count();
                        block_code.push_str(&block_lines.join("\n"));

                        let block_diags = run_dxc_on_text(&block_code, dxc_path, block_start_line, &include_dirs, preamble_line_count);
                        diagnostics.extend(block_diags);
                    }
                    continue;
                }

                if in_block {
                    if should_stub_include(trimmed) {
                        block_lines.push(format!("// {}", line));
                    } else {
                        block_lines.push(line.to_string());
                    }
                }
            }
        }
        ShaderContext::UnityHlsl => {
            let mut block_code = String::from(UNITY_COMPAT_PREAMBLE.trim_start());
            if !block_code.ends_with('\n') {
                block_code.push('\n');
            }
            let preamble_line_count = block_code.lines().count();
            for line in content.lines() {
                if should_stub_include(line) {
                    block_code.push_str("// ");
                }
                block_code.push_str(line);
                block_code.push('\n');
            }
            diagnostics = run_dxc_on_text(&block_code, dxc_path, 0, &include_dirs, preamble_line_count);
        }
        ShaderContext::UnrealEngine => {
            let preamble = if content.contains("FMaterialPixelParameters") {
                "float3 Luminance(float3 LinearColor) { return dot(LinearColor, float3(0.3, 0.59, 0.11)); }\nfloat3 RotateAboutAxis(float4 NormalizedRotationAxisAndAngle, float3 PivotPoint, float3 Position) {\n    float3 Axis = NormalizedRotationAxisAndAngle.xyz;\n    float Angle = NormalizedRotationAxisAndAngle.w;\n    return Position + sin(Angle) * cross(Axis, Position - PivotPoint);\n}\n"
            } else {
                UNREAL_COMPAT_PREAMBLE
            };
            let mut block_code = String::from(preamble.trim_start());
            if !block_code.ends_with('\n') {
                block_code.push('\n');
            }
            let preamble_line_count = block_code.lines().count();
            for line in content.lines() {
                if should_stub_include(line) {
                    block_code.push_str("// ");
                }
                block_code.push_str(line);
                block_code.push('\n');
            }
            diagnostics = run_dxc_on_text(&block_code, dxc_path, 0, &include_dirs, preamble_line_count);
        }
        ShaderContext::PureHlsl => {
            diagnostics = run_dxc_on_text(content, dxc_path, 0, &include_dirs, 0);
        }
    }

    diagnostics
}

static TEMP_FILE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn run_dxc_on_text(
    content: &str,
    dxc_path: &str,
    line_offset: usize,
    include_dirs: &[PathBuf],
    preamble_line_count: usize,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let temp_dir = env::temp_dir();
    let count = TEMP_FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temp_file = temp_dir.join(format!("hlsl_val_{}_{}.hlsl", std::process::id(), count));

    if fs::write(&temp_file, content).is_err() {
        return diagnostics;
    }

    let mut cmd = Command::new(dxc_path);
    cmd.arg("-T").arg("lib_6_3");
    cmd.arg("-HV").arg("2021");

    for inc in include_dirs {
        cmd.arg("-I").arg(inc);
    }

    cmd.arg(&temp_file);

    let output = cmd.output();
    let _ = fs::remove_file(&temp_file);

    let output = match output {
        Ok(out) => out,
        Err(_) => return diagnostics,
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stderr}\n{stdout}");

    for line in combined.lines() {
        let (severity, rest) = if let Some(idx) = line.find(": error:") {
            (1, line[idx + 8..].trim())
        } else if let Some(idx) = line.find(": warning:") {
            (2, line[idx + 10..].trim())
        } else {
            continue;
        };

        if rest.contains("file not found")
            && (rest.contains("Packages/") || rest.contains("UnityCG") || rest.contains("Engine") || rest.contains(".ush") || rest.contains(".cginc"))
        {
            continue;
        }

        if rest.starts_with("redefinition of") {
            let sym = rest.split('\'').nth(1).unwrap_or("");
            if [
                "_RendererColor", "_Flip", "_ClipRect",
                "UNITY_MATRIX_MVP", "UNITY_MATRIX_MV", "UNITY_MATRIX_V", "UNITY_MATRIX_P", "UNITY_MATRIX_VP",
                "unity_ObjectToWorld", "unity_WorldToObject", "unity_MatrixVP", "unity_MatrixV", "unity_MatrixInvV", "unity_MatrixP", "unity_MatrixInvP",
                "_Time", "_SinTime", "_CosTime", "unity_DeltaTime",
                "_ScreenParams", "_ScaledScreenParams", "_ProjectionParams", "_ZBufferParams", "unity_OrthoParams",
                "_WorldSpaceCameraPos", "_WorldSpaceLightPos0", "_LightColor0",
                "_MainLightPosition", "_MainLightColor", "_AdditionalLightsCount",
                "UnitySampler2D", "TransformObjectToHClip", "UnityObjectToClipPos", "UnityObjectToWorldNormal",
                "TransformObjectToWorldNormal", "TransformObjectToWorldDir", "TransformWorldToView",
                "ResolvedView", "View",
            ].contains(&sym) {
                continue;
            }
        }

        let prefix_part = line.split(": error:").next().or_else(|| line.split(": warning:").next()).unwrap_or("");
        let mut parts = prefix_part.rsplitn(3, ':');
        let col_str = parts.next().unwrap_or("");
        let line_str = parts.next().unwrap_or("");

        if let (Ok(parsed_line), Ok(parsed_col)) = (line_str.parse::<usize>(), col_str.parse::<usize>()) {
            if parsed_line <= preamble_line_count {
                // Ignore internal preamble errors if any
                continue;
            }

            let actual_line = (parsed_line - 1).saturating_sub(preamble_line_count) + line_offset;
            let actual_col = parsed_col.saturating_sub(1);

            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: actual_line, character: actual_col },
                    end: Position { line: actual_line, character: actual_col + 5 },
                },
                severity,
                message: rest.to_string(),
                source: "dxc".to_string(),
            });
        }
    }

    diagnostics
}

pub fn format_document(text: &str, tab_size: usize, insert_spaces: bool) -> Vec<Value> {
    let indent_unit = if insert_spaces {
        " ".repeat(tab_size)
    } else {
        "\t".to_string()
    };

    let mut formatted_lines = Vec::new();
    let mut indent_level: usize = 0;
    let mut blank_count = 0;

    for raw_line in text.lines() {
        let trimmed = raw_line.trim();

        if trimmed.is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                formatted_lines.push(String::new());
            }
            continue;
        }
        blank_count = 0;

        if trimmed.starts_with('#') {
            formatted_lines.push(trimmed.to_string());
            continue;
        }

        // Ignore lines that are comments so they don't alter indent levels
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            let line_indent = indent_unit.repeat(indent_level);
            formatted_lines.push(format!("{}{}", line_indent, trimmed));
            continue;
        }

        let mut leading_close = 0;
        for c in trimmed.chars() {
            if c == '}' {
                leading_close += 1;
            } else if !c.is_whitespace() {
                break;
            }
        }
        indent_level = indent_level.saturating_sub(leading_close);

        let line_indent = indent_unit.repeat(indent_level);
        formatted_lines.push(format!("{}{}", line_indent, trimmed));

        // Count braces in code only (ignoring trailing single-line comments)
        let code_part = trimmed.split("//").next().unwrap_or("").trim();
        let open_count = code_part.chars().filter(|&c| c == '{').count();
        let close_count = code_part.chars().filter(|&c| c == '}').count();
        let net_close_after = close_count.saturating_sub(leading_close);
        indent_level = indent_level.saturating_sub(net_close_after) + open_count;
    }

    let new_text = formatted_lines.join("\n") + "\n";
    let line_count = text.lines().count();
    let last_line_len = text.lines().last().map(|l| l.len()).unwrap_or(0);
    let end_line = line_count.saturating_sub(1);

    vec![json!({
        "range": {
            "start": { "line": 0, "character": 0 },
            "end": { "line": end_line, "character": last_line_len }
        },
        "newText": new_text
    })]
}

fn get_document_from_cache<'a>(doc_cache: &'a HashMap<String, String>, uri: &str) -> Option<&'a str> {
    if let Some(d) = doc_cache.get(uri) {
        return Some(d.as_str());
    }
    let uri_lower = uri.to_lowercase();
    for (k, v) in doc_cache {
        if k.to_lowercase() == uri_lower {
            return Some(v.as_str());
        }
    }
    None
}

fn handle_completion(
    msg: &Value,
    doc_cache: &HashMap<String, String>,
) -> Value {
    let params = match msg.get("params") {
        Some(p) => p,
        None => return json!([]),
    };

    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
    let line_idx = params["position"]["line"].as_u64().unwrap_or(0) as usize;
    let col_idx = params["position"]["character"].as_u64().unwrap_or(0) as usize;

    let owned_doc;
    let doc = match get_document_from_cache(doc_cache, uri) {
        Some(d) => d,
        None => {
            if let Some(p) = uri_to_path(uri) {
                if let Ok(content) = std::fs::read_to_string(p) {
                    owned_doc = content;
                    owned_doc.as_str()
                } else {
                    return json!([]);
                }
            } else {
                return json!([]);
            }
        }
    };

    if signature::is_in_comment_or_string(doc, line_idx, col_idx) {
        return json!([]);
    }

    let line = match doc.lines().nth(line_idx) {
        Some(l) => l,
        None => return json!([]),
    };

    let safe_col = signature::safe_floor_char_boundary(line, col_idx.min(line.len()));
    let prefix = &line[..safe_col];

    let context = detect_shader_context(uri, doc);

    // ------------------------------------------------------------------------
    // Context A: Unity ShaderLab (Outside HLSL blocks)
    // ------------------------------------------------------------------------
    if context == ShaderContext::UnityShaderLab && !is_inside_hlsl_block(doc, line_idx) {
        let mut sl_items = Vec::new();
        let in_props = is_inside_properties_block(doc, line_idx);

        if in_props {
            let line_before_cursor = prefix;

            // 1. If after '=', suggest default values based on property type
            if let Some(eq_pos) = line_before_cursor.rfind('=') {
                let before_eq = &line[..eq_pos];
                let mut default_items = Vec::new();

                if before_eq.contains("Color") {
                    let color_defaults = [
                        ("(1, 1, 1, 1)", "Opaque White (1, 1, 1, 1)"),
                        ("(0, 0, 0, 1)", "Opaque Black (0, 0, 0, 1)"),
                        ("(1, 0, 0, 1)", "Opaque Red (1, 0, 0, 1)"),
                        ("(0, 1, 0, 1)", "Opaque Green (0, 1, 0, 1)"),
                        ("(0, 0, 1, 1)", "Opaque Blue (0, 0, 1, 1)"),
                        ("(0, 0, 0, 0)", "Transparent Black (0, 0, 0, 0)"),
                        ("(1, 1, 1, 0)", "Transparent White (1, 1, 1, 0)"),
                    ];
                    for (idx, (val, desc)) in color_defaults.iter().enumerate() {
                        default_items.push(json!({
                            "label": *val,
                            "kind": 12,
                            "detail": *desc,
                            "insertText": *val,
                            "sortText": format!("00_{:02}_{}", idx, val),
                        }));
                    }
                } else if before_eq.contains("Vector") {
                    let vector_defaults = [
                        ("(0, 0, 0, 0)", "Zero Vector (0, 0, 0, 0)"),
                        ("(1, 1, 1, 1)", "One Vector (1, 1, 1, 1)"),
                        ("(0, 0, 1, 1)", "UV Tiling (1,1), Offset (0,0)"),
                    ];
                    for (idx, (val, desc)) in vector_defaults.iter().enumerate() {
                        default_items.push(json!({
                            "label": *val,
                            "kind": 12,
                            "detail": *desc,
                            "insertText": *val,
                            "sortText": format!("00_{:02}_{}", idx, val),
                        }));
                    }
                } else if before_eq.contains("2D") || before_eq.contains("Cube") || before_eq.contains("3D") || before_eq.contains("2DArray") {
                    let tex_defaults = [
                        ("\"white\" {}", "Default white 2D texture (RGB 1, 1, 1)"),
                        ("\"black\" {}", "Default black 2D texture (RGB 0, 0, 0)"),
                        ("\"gray\" {}", "Default neutral gray texture (RGB 0.5, 0.5, 0.5)"),
                        ("\"bump\" {}", "Default flat normal map (RGB 0.5, 0.5, 1.0)"),
                        ("\"normal\" {}", "Default normal map slot"),
                        ("\"red\" {}", "Default red texture (RGB 1, 0, 0)"),
                        ("\"\" {}", "Empty texture slot"),
                    ];
                    for (idx, (val, desc)) in tex_defaults.iter().enumerate() {
                        default_items.push(json!({
                            "label": *val,
                            "kind": 12,
                            "detail": *desc,
                            "insertText": *val,
                            "sortText": format!("00_{:02}_{}", idx, val),
                        }));
                    }
                } else if before_eq.contains("Float") || before_eq.contains("Integer") || before_eq.contains("Int") {
                    let float_defaults = [
                        ("0.0", "Zero (0.0)"),
                        ("1.0", "One (1.0)"),
                        ("0.5", "Half (0.5)"),
                        ("-1.0", "Negative One (-1.0)"),
                        ("0", "Integer Zero (0)"),
                        ("1", "Integer One (1)"),
                    ];
                    for (idx, (val, desc)) in float_defaults.iter().enumerate() {
                        default_items.push(json!({
                            "label": *val,
                            "kind": 12,
                            "detail": *desc,
                            "insertText": *val,
                            "sortText": format!("00_{:02}_{}", idx, val),
                        }));
                    }
                } else if before_eq.contains("Range") {
                    let mut min_val = "0.0";
                    let mut max_val = "1.0";
                    if let Some(r_open) = before_eq.find("Range(") {
                        if let Some(r_close) = before_eq[r_open..].find(')') {
                            let r_content = &before_eq[r_open + 6..r_open + r_close];
                            let r_parts: Vec<&str> = r_content.split(',').collect();
                            if r_parts.len() == 2 {
                                min_val = r_parts[0].trim();
                                max_val = r_parts[1].trim();
                            }
                        }
                    }
                    default_items.push(json!({
                        "label": min_val,
                        "kind": 12,
                        "detail": format!("Range minimum value ({min_val})"),
                        "insertText": min_val,
                        "sortText": "00_00_min",
                    }));
                    default_items.push(json!({
                        "label": max_val,
                        "kind": 12,
                        "detail": format!("Range maximum value ({max_val})"),
                        "insertText": max_val,
                        "sortText": "00_01_max",
                    }));
                    default_items.push(json!({
                        "label": "0.5",
                        "kind": 12,
                        "detail": "Range midpoint (0.5)",
                        "insertText": "0.5",
                        "sortText": "00_02_mid",
                    }));
                }

                if !default_items.is_empty() {
                    return json!(default_items);
                }
            }

            // 2. Property Attributes: [MainColor], [MainTexture], [HDR], [HideInInspector], etc.
            let trimmed_p = prefix.trim_start();
            let is_in_attr = trimmed_p.starts_with('[') || line_before_cursor.contains('[');
            if is_in_attr {
                let last_bracket = line_before_cursor.rfind('[');
                let unclosed_bracket = match last_bracket {
                    Some(idx) => !line_before_cursor[idx..].contains(']'),
                    None => false,
                };
                let line_after_cursor = &line[safe_col..];
                let has_closing_bracket_after = line_after_cursor.trim_start().starts_with(']');

                for (idx, (attr_name, snip, desc)) in docs::SHADERLAB_ATTRIBUTES.iter().enumerate() {
                    let label = format!("[{}]", attr_name);
                    let insert = if unclosed_bracket {
                        if has_closing_bracket_after {
                            snip.to_string()
                        } else {
                            format!("{}]", snip)
                        }
                    } else {
                        format!("[{}]", snip)
                    };
                    sl_items.push(json!({
                        "label": label,
                        "kind": 15,
                        "detail": *desc,
                        "insertText": insert,
                        "insertTextFormat": 2,
                        "sortText": format!("00_{:02}_{}", idx, attr_name),
                    }));
                }
                return json!(sl_items);
            }

            // 1.5. If inside `_PropName ("Display Name", |)` -> suggest property types with default snippets!
            if let Some(comma_idx) = line_before_cursor.rfind(',') {
                let before_comma = line_before_cursor[..comma_idx].trim();
                if before_comma.ends_with('"') && before_comma.contains('(') {
                    let mut type_items = Vec::new();
                    for (idx, (p_type, snip, desc)) in docs::SHADERLAB_TYPE_COMPLETIONS_AFTER_COMMA.iter().enumerate() {
                        type_items.push(json!({
                            "label": *p_type,
                            "kind": 7,
                            "detail": *desc,
                            "insertText": *snip,
                            "insertTextFormat": 2,
                            "sortText": format!("00_{:02}_{}", idx, p_type),
                        }));
                    }
                    return json!(type_items);
                }
            }

            // 2. Keyword-based property snippet completions (typing "float", "range", "color", "tex", "hdr", etc.)
            for (idx, (kw, snip, desc)) in docs::SHADERLAB_PROPERTY_KEYWORD_SNIPPETS.iter().enumerate() {
                sl_items.push(json!({
                    "label": *kw,
                    "kind": 15,
                    "detail": *desc,
                    "insertText": *snip,
                    "insertTextFormat": 2,
                    "sortText": format!("03_{:02}_{}", idx, kw),
                }));
            }

            // 3. Standard & Common Property Templates
            for (idx, (label, snip, desc)) in docs::SHADERLAB_COMMON_PROPERTIES.iter().enumerate() {
                sl_items.push(json!({
                    "label": *label,
                    "kind": 15,
                    "detail": *desc,
                    "insertText": *snip,
                    "sortText": format!("05_{:02}_{}", idx, label),
                }));
            }

            // 4. Built-in Property Types with snippets
            let already_has_ident = prefix.trim_start().starts_with('_');
            for (idx, (prop_type, full_snip, suffix_snip, desc)) in docs::SHADERLAB_PROPERTY_TYPES.iter().enumerate() {
                let snip = if already_has_ident { suffix_snip } else { full_snip };
                sl_items.push(json!({
                    "label": *prop_type,
                    "kind": 7,
                    "detail": *desc,
                    "insertText": *snip,
                    "insertTextFormat": 2,
                    "sortText": format!("10_{:02}_{}", idx, prop_type),
                }));
            }

            // 5. Attributes also available standalone
            for (idx, (attr_name, snip, desc)) in docs::SHADERLAB_ATTRIBUTES.iter().enumerate() {
                sl_items.push(json!({
                    "label": format!("[{}]", attr_name),
                    "kind": 15,
                    "detail": *desc,
                    "insertText": format!("[{}]", snip),
                    "insertTextFormat": 2,
                    "sortText": format!("15_{:02}_{}", idx, attr_name),
                }));
            }

            return json!(sl_items);
        }

        // Outside Properties (In SubShader / Pass)
        let trimmed_prefix = prefix.trim();

        // Helper for ZCull alias (common confusion with Cull and ZClip)
        if trimmed_prefix.eq_ignore_ascii_case("zcull") || trimmed_prefix.to_ascii_lowercase().starts_with("zcull") {
            sl_items.push(json!({
                "label": "Cull Back (ZCull -> Cull)",
                "kind": 12,
                "detail": "Unity ShaderLab uses 'Cull' for polygon culling and 'ZClip' for depth clipping",
                "insertText": "Cull Back",
                "sortText": "00_00_cull_back",
            }));
            sl_items.push(json!({
                "label": "Cull Front",
                "kind": 12,
                "detail": "Culls front-facing polygons (renders back faces)",
                "insertText": "Cull Front",
                "sortText": "00_01_cull_front",
            }));
            sl_items.push(json!({
                "label": "Cull Off",
                "kind": 12,
                "detail": "Disables polygon culling (double-sided)",
                "insertText": "Cull Off",
                "sortText": "00_02_cull_off",
            }));
            sl_items.push(json!({
                "label": "ZClip False",
                "kind": 12,
                "detail": "Disables depth clipping (clamps near/far depth planes)",
                "insertText": "ZClip False",
                "sortText": "00_03_zclip_false",
            }));
            return json!(sl_items);
        }

        // Specific command value completions when prefix starts with the command name
        let state_triggers = [
            ("BlendOp", "BlendOp"),
            ("Blend", "Blend"),
            ("Cull", "Cull"),
            ("ZWrite", "ZWrite"),
            ("ZTest", "ZTest"),
            ("ZClip", "ZClip"),
            ("ColorMask", "ColorMask"),
            ("Offset", "Offset"),
            ("Conservative", "Conservative"),
            ("AlphaToMask", "AlphaToMask"),
            ("Lighting", "Lighting"),
            ("Ref", "Ref"),
            ("ReadMask", "ReadMask"),
            ("WriteMask", "WriteMask"),
            ("Comp", "Comp"),
            ("UsePass", "UsePass"),
            ("GrabPass", "GrabPass"),
            ("Name", "Name"),
            ("LOD", "LOD"),
        ];

        for (trigger, state_category) in &state_triggers {
            if trimmed_prefix.starts_with(trigger) {
                let has_space = trimmed_prefix.contains(' ');
                let prefix_to_strip = format!("{} ", trigger);
                for (idx, (_, val, desc)) in docs::SHADERLAB_RENDER_STATES.iter().filter(|(s, _, _)| *s == *state_category).enumerate() {
                    let insert = if has_space {
                        val.strip_prefix(&prefix_to_strip).unwrap_or(val)
                    } else {
                        *val
                    };
                    sl_items.push(json!({
                        "label": *val,
                        "kind": 12,
                        "detail": *desc,
                        "insertText": insert,
                        "sortText": format!("00_{:02}_{}", idx, val),
                    }));
                }
                return json!(sl_items);
            }
        }

        // Pass operation values (Stencil Pass, Fail, ZFail)
        if trimmed_prefix.starts_with("Pass ") {
            for (idx, (_, val, desc)) in docs::SHADERLAB_RENDER_STATES.iter().filter(|(s, _, _)| *s == "Pass").enumerate() {
                let insert = val.strip_prefix("Pass ").unwrap_or(val);
                sl_items.push(json!({
                    "label": *val,
                    "kind": 12,
                    "detail": *desc,
                    "insertText": insert,
                    "sortText": format!("00_{:02}_{}", idx, val),
                }));
            }
            return json!(sl_items);
        }
        if trimmed_prefix.starts_with("Fail") {
            let has_space = trimmed_prefix.contains(' ');
            for (idx, (_, val, desc)) in docs::SHADERLAB_RENDER_STATES.iter().filter(|(s, _, _)| *s == "Fail").enumerate() {
                let insert = if has_space { val.strip_prefix("Fail ").unwrap_or(val) } else { *val };
                sl_items.push(json!({
                    "label": *val,
                    "kind": 12,
                    "detail": *desc,
                    "insertText": insert,
                    "sortText": format!("00_{:02}_{}", idx, val),
                }));
            }
            return json!(sl_items);
        }
        if trimmed_prefix.starts_with("ZFail") {
            let has_space = trimmed_prefix.contains(' ');
            for (idx, (_, val, desc)) in docs::SHADERLAB_RENDER_STATES.iter().filter(|(s, _, _)| *s == "ZFail").enumerate() {
                let insert = if has_space { val.strip_prefix("ZFail ").unwrap_or(val) } else { *val };
                sl_items.push(json!({
                    "label": *val,
                    "kind": 12,
                    "detail": *desc,
                    "insertText": insert,
                    "sortText": format!("00_{:02}_{}", idx, val),
                }));
            }
            return json!(sl_items);
        }

        if line.contains("Tags") || is_inside_tags_block(doc, line_idx) {
            let line_before_cursor = prefix;
            let line_after_cursor = &line[safe_col..];
            if let Some(eq_idx) = line_before_cursor.rfind('=') {
                let before_eq = line_before_cursor[..eq_idx].trim();
                let key = before_eq.rsplit('"').nth(1)
                    .or_else(|| before_eq.split_whitespace().last())
                    .unwrap_or(before_eq)
                    .trim_start_matches('{')
                    .trim();
                let after_eq = line_before_cursor[eq_idx + 1..].trim_start();
                let already_has_quote = after_eq.starts_with('"');
                let closing_quote_present = line_after_cursor.trim_start().starts_with('"');

                for (tag_key, values, desc) in docs::SHADERLAB_TAG_KEYS_AND_VALUES {
                    if *tag_key == key {
                        for (v_idx, val) in values.iter().enumerate() {
                            let clean_val = val.trim_matches('"');
                            let insert = if already_has_quote && closing_quote_present {
                                clean_val.to_string()
                            } else if already_has_quote {
                                format!("{}\"", clean_val)
                            } else {
                                (*val).to_string()
                            };
                            sl_items.push(json!({
                                "label": *val,
                                "kind": 12,
                                "detail": format!("{}: {}", tag_key, desc),
                                "insertText": insert,
                                "sortText": format!("00_{:02}_{}", v_idx, clean_val),
                            }));
                        }
                        return json!(sl_items);
                    }
                }
            }

            // Inside Tags block (before '='): suggest tag keys with value templates
            let quote_before_key = prefix.trim_end().ends_with('"');
            for (idx, (tag_key, values, desc)) in docs::SHADERLAB_TAG_KEYS_AND_VALUES.iter().enumerate() {
                let default_val = values.first().map(|v| v.trim_matches('"')).unwrap_or("Opaque");
                let snippet = if quote_before_key {
                    format!("{}\" = \"${{1:{}}}\"", tag_key, default_val)
                } else {
                    format!("\"{}\" = \"${{1:{}}}\"", tag_key, default_val)
                };
                sl_items.push(json!({
                    "label": *tag_key,
                    "kind": 10,
                    "detail": format!("Tag: {}", desc),
                    "insertText": snippet,
                    "insertTextFormat": 2,
                    "sortText": format!("01_{:02}_{}", idx, tag_key),
                }));
            }

            for (idx, (tag, desc)) in docs::SHADERLAB_TAGS.iter().enumerate() {
                sl_items.push(json!({
                    "label": *tag,
                    "kind": 10,
                    "detail": *desc,
                    "insertText": *tag,
                    "sortText": format!("05_{:02}_{}", idx, tag),
                }));
            }
            if !sl_items.is_empty() {
                return json!(sl_items);
            }
        }

        // Add all ShaderLab render states so fuzzy-filtering matches ZWrite, ZTest, ZClip, Cull, Blend, etc.
        let is_z_query = trimmed_prefix.eq_ignore_ascii_case("z");
        for (idx, (cat, val, desc)) in docs::SHADERLAB_RENDER_STATES.iter().enumerate() {
            let is_z_state = *cat == "ZWrite" || *cat == "ZTest" || *cat == "ZClip";
            let sort_prefix = if is_z_query && is_z_state { "00" } else { "08" };
            sl_items.push(json!({
                "label": *val,
                "kind": 12,
                "detail": *desc,
                "insertText": *val,
                "sortText": format!("{}_{:02}_{}", sort_prefix, idx, val),
            }));
        }

        for (idx, kw) in docs::BUILTIN_KEYWORDS.iter().filter(|kw| !["float", "float4", "cbuffer", "struct", "return"].contains(kw)).enumerate() {
            sl_items.push(json!({
                "label": *kw,
                "kind": 14,
                "insertText": *kw,
                "sortText": format!("15_{:02}_{}", idx, kw),
            }));
        }

        let is_2d = is_unity_2d_context(uri, doc);
        let sl_snippets = [
            ("shader", "Unity ShaderLab 3D Shader template", "Shader \"$1\"\n{\n    Properties\n    {\n        _MainTex (\"Texture\", 2D) = \"white\" {}\n    }\n    SubShader\n    {\n        Tags { \"RenderType\"=\"Opaque\" \"RenderPipeline\"=\"UniversalPipeline\" }\n        Pass\n        {\n            HLSLPROGRAM\n            #pragma vertex vert\n            #pragma fragment frag\n            $0\n            ENDHLSL\n        }\n    }\n}"),
            ("sprite", "Unity 2D Sprite Shader template", "Shader \"Sprites/${1:CustomSprite}\"\n{\n    Properties\n    {\n        [PerRendererData] _MainTex (\"Sprite Texture\", 2D) = \"white\" {}\n        _Color (\"Tint\", Color) = (1,1,1,1)\n        [MaterialToggle] PixelSnap (\"Pixel snap\", Float) = 0\n        [HideInInspector] _RendererColor (\"RendererColor\", Color) = (1,1,1,1)\n        [HideInInspector] _Flip (\"Flip\", Vector) = (1,1,1,1)\n    }\n    SubShader\n    {\n        Tags\n        {\n            \"Queue\"=\"Transparent\"\n            \"IgnoreProjector\"=\"True\"\n            \"RenderType\"=\"Transparent\"\n            \"PreviewType\"=\"Plane\"\n            \"CanUseSpriteAtlas\"=\"True\"\n        }\n        Cull Off\n        Lighting Off\n        ZWrite Off\n        Blend One OneMinusSrcAlpha\n        Pass\n        {\n            CGPROGRAM\n            #pragma vertex vert\n            #pragma fragment frag\n            #pragma multi_compile _ PIXELSNAP_ON\n            #include \"UnityCG.cginc\"\n\n            struct appdata_t\n            {\n                float4 vertex   : POSITION;\n                float4 color    : COLOR;\n                float2 texcoord : TEXCOORD0;\n            };\n\n            struct v2f\n            {\n                float4 vertex   : SV_POSITION;\n                fixed4 color    : COLOR;\n                float2 texcoord : TEXCOORD0;\n            };\n\n            sampler2D _MainTex;\n            fixed4 _Color;\n            fixed4 _RendererColor;\n            float4 _Flip;\n\n            v2f vert(appdata_t IN)\n            {\n                v2f OUT;\n                IN.vertex.xy *= _Flip.xy;\n                OUT.vertex = UnityObjectToClipPos(IN.vertex);\n                OUT.texcoord = IN.texcoord;\n                OUT.color = IN.color * _Color * _RendererColor;\n                #ifdef PIXELSNAP_ON\n                OUT.vertex = UnityPixelSnap(OUT.vertex);\n                #endif\n                return OUT;\n            }\n\n            fixed4 frag(v2f IN) : SV_Target\n            {\n                fixed4 c = tex2D(_MainTex, IN.texcoord) * IN.color;\n                c.rgb *= c.a;\n                return c;\n            }\n            ENDCG\n        }\n    }\n}"),
            ("ui", "Unity UI (Canvas 2D) Shader template", "Shader \"UI/${1:CustomUI}\"\n{\n    Properties\n    {\n        [PerRendererData] _MainTex (\"Sprite Texture\", 2D) = \"white\" {}\n        _Color (\"Tint\", Color) = (1,1,1,1)\n        _ClipRect (\"Clip Rect\", Vector) = (-32767, -32767, 32767, 32767)\n        _ColorMask (\"Color Mask\", Float) = 15\n    }\n    SubShader\n    {\n        Tags\n        {\n            \"Queue\"=\"Transparent\"\n            \"IgnoreProjector\"=\"True\"\n            \"RenderType\"=\"Transparent\"\n            \"PreviewType\"=\"Plane\"\n            \"CanUseSpriteAtlas\"=\"True\"\n        }\n        Cull Off\n        Lighting Off\n        ZWrite Off\n        Blend SrcAlpha OneMinusSrcAlpha\n        ColorMask [_ColorMask]\n        Pass\n        {\n            CGPROGRAM\n            #pragma vertex vert\n            #pragma fragment frag\n            #include \"UnityCG.cginc\"\n\n            struct appdata_t\n            {\n                float4 vertex   : POSITION;\n                float4 color    : COLOR;\n                float2 texcoord : TEXCOORD0;\n            };\n\n            struct v2f\n            {\n                float4 vertex   : SV_POSITION;\n                fixed4 color    : COLOR;\n                float2 texcoord : TEXCOORD0;\n                float4 worldPosition : TEXCOORD1;\n            };\n\n            sampler2D _MainTex;\n            fixed4 _Color;\n            float4 _ClipRect;\n\n            v2f vert(appdata_t v)\n            {\n                v2f OUT;\n                OUT.worldPosition = v.vertex;\n                OUT.vertex = UnityObjectToClipPos(OUT.worldPosition);\n                OUT.texcoord = v.texcoord;\n                OUT.color = v.color * _Color;\n                return OUT;\n            }\n\n            fixed4 frag(v2f IN) : SV_Target\n            {\n                half4 color = tex2D(_MainTex, IN.texcoord) * IN.color;\n                color.a *= UnityGet2DClipping(IN.worldPosition.xy, _ClipRect);\n                return color;\n            }\n            ENDCG\n        }\n    }\n}"),
            ("pass", "Unity ShaderLab Pass block", "Pass\n{\n    Name \"$1\"\n    HLSLPROGRAM\n    #pragma vertex vert\n    #pragma fragment frag\n    $0\n    ENDHLSL\n}"),
            ("stencil", "ShaderLab Stencil buffer block", "Stencil\n{\n    Ref ${1:1}\n    Comp ${2:Always}\n    Pass ${3:Replace}\n}"),
            ("usepass", "ShaderLab UsePass command", "UsePass \"${1:Shader/PASSNAME}\""),
            ("grabpass", "ShaderLab GrabPass command", "GrabPass { \"${1:_BackgroundTexture}\" }"),
            ("name", "Pass Name command", "Name \"${1:PassName}\""),
            ("lod", "SubShader LOD level", "LOD ${1:100}"),
            ("tags", "ShaderLab Tags block", "Tags { \"${1:RenderType}\" = \"${2:Opaque}\" }"),
            ("properties", "Properties block", "Properties\n{\n    $0\n}"),
            ("subshader", "SubShader block", "SubShader\n{\n    $0\n}"),
        ];
        for (idx, (label, detail, snip)) in sl_snippets.iter().enumerate() {
            let sort_prefix = if is_2d && (*label == "sprite" || *label == "ui") {
                "25"
            } else {
                "30"
            };
            sl_items.push(json!({
                "label": *label,
                "kind": 15,
                "detail": *detail,
                "insertText": *snip,
                "insertTextFormat": 2,
                "sortText": format!("{}_{:02}_{}", sort_prefix, idx, label),
            }));
        }

        return json!(sl_items);
    }

    // ------------------------------------------------------------------------
    // Context B: HLSL, Unity HLSL, or Unreal Engine
    // ------------------------------------------------------------------------

    // Member access: expr.field or expr.partial
    if let Some(dot_idx) = prefix.rfind('.') {
        let before_dot = prefix[..dot_idx].trim_end();
        let after_dot = prefix[dot_idx + 1..].trim_start();

        let is_valid_after = after_dot.chars().all(|c| c.is_alphanumeric() || c == '_');
        if is_valid_after {
            let mut expr_start = before_dot.len();
            for (i, c) in before_dot.char_indices().rev() {
                if c.is_alphanumeric() || c == '_' || c == '.' || c == ']' {
                    expr_start = i;
                } else {
                    break;
                }
            }
            let expr = &before_dot[expr_start..];
            if !expr.is_empty() {
                let clean_expr = if let Some(b_idx) = expr.find('[') {
                    expr[..b_idx].trim()
                } else {
                    expr
                };
                let parts: Vec<&str> = clean_expr.split('.').collect();
                let (_, _, user_structs) = signature::resolve_includes_and_scan_symbols(uri, doc, doc_cache);
                let structs = user_structs;

                if let Some(root_var) = parts.first() {
                    let mut current_type = signature::infer_variable_type(doc, root_var, line_idx);

                    for field in &parts[1..] {
                        if let Some(ref ct) = current_type {
                            if let Some(s_def) = structs.iter().find(|s| &s.name == ct) {
                                current_type = s_def.fields.iter().find(|f| &f.name == field).map(|f| f.field_type.clone());
                            } else {
                                current_type = None;
                                break;
                            }
                        }
                    }

                    if let Some(target_type) = current_type {
                        // Struct fields
                        if let Some(s_def) = structs.iter().find(|s| s.name == target_type) {
                            let field_items: Vec<Value> = s_def.fields.iter().enumerate().map(|(idx, f)| {
                                json!({
                                    "label": f.name,
                                    "kind": 5,
                                    "detail": format!("{} {}.{}", f.field_type, s_def.name, f.name),
                                    "insertText": f.name,
                                    "sortText": format!("00_{:02}_{}", idx, f.name),
                                })
                            }).collect();
                            return json!(field_items);
                        }

                        // Texture methods
                        if target_type.starts_with("Texture") {
                            let method_items: Vec<Value> = docs::TEXTURE_METHODS.iter().enumerate().map(|(idx, m)| {
                                json!({
                                    "label": m.name,
                                    "kind": 2,
                                    "detail": m.signature,
                                    "documentation": { "kind": "markdown", "value": m.description },
                                    "insertText": m.snippet,
                                    "insertTextFormat": 2,
                                    "command": {
                                        "title": "Trigger Parameter Hints",
                                        "command": "editor.action.triggerParameterHints"
                                    },
                                    "sortText": format!("00_{:02}_{}", idx, m.name),
                                })
                            }).collect();
                            return json!(method_items);
                        }

                        // Buffer methods
                        if target_type.contains("Buffer") {
                            let method_items: Vec<Value> = docs::BUFFER_METHODS.iter().enumerate().map(|(idx, m)| {
                                json!({
                                    "label": m.name,
                                    "kind": 2,
                                    "detail": m.signature,
                                    "documentation": { "kind": "markdown", "value": m.description },
                                    "insertText": m.snippet,
                                    "insertTextFormat": 2,
                                    "command": {
                                        "title": "Trigger Parameter Hints",
                                        "command": "editor.action.triggerParameterHints"
                                    },
                                    "sortText": format!("00_{:02}_{}", idx, m.name),
                                })
                            }).collect();
                            return json!(method_items);
                        }

                        // Matrix element access (_m00.._m33, _11.._44)
                        let is_matrix = target_type.contains('x') || target_type == "matrix";
                        if is_matrix {
                            let (rows, cols) = if target_type.contains("4x4") || target_type == "matrix" {
                                (4, 4)
                            } else if target_type.contains("3x3") {
                                (3, 3)
                            } else if target_type.contains("2x2") {
                                (2, 2)
                            } else if target_type.contains("4x3") {
                                (4, 3)
                            } else if target_type.contains("3x4") {
                                (3, 4)
                            } else {
                                (4, 4)
                            };

                            let mut matrix_items = Vec::new();
                            let mut idx = 0;
                            // 0-based: _m00, _m01, ...
                            for r in 0..rows {
                                for c in 0..cols {
                                    let label = format!("_m{r}{c}");
                                    matrix_items.push(json!({
                                        "label": label,
                                        "kind": 5, // Field
                                        "detail": format!("Matrix element [{r}][{c}] (0-based)"),
                                        "insertText": label,
                                        "sortText": format!("00_{:02}_{}", idx, label),
                                    }));
                                    idx += 1;
                                }
                            }
                            // 1-based: _11, _12, ...
                            for r in 1..=rows {
                                for c in 1..=cols {
                                    let label = format!("_{r}{c}");
                                    matrix_items.push(json!({
                                        "label": label,
                                        "kind": 5, // Field
                                        "detail": format!("Matrix element [{r}][{c}] (1-based)"),
                                        "insertText": label,
                                        "sortText": format!("01_{:02}_{}", idx, label),
                                    }));
                                    idx += 1;
                                }
                            }
                            return json!(matrix_items);
                        }

                        // Vector swizzles (Dimension-aware)
                        let is_vec = target_type.starts_with("float")
                            || target_type.starts_with("half")
                            || target_type.starts_with("int")
                            || target_type.starts_with("uint")
                            || target_type.starts_with("fixed")
                            || target_type.starts_with("bool");

                        if is_vec {
                            let dim = if target_type.ends_with('4') {
                                4
                            } else if target_type.ends_with('3') {
                                3
                            } else if target_type.ends_with('2') {
                                2
                            } else {
                                4
                            };

                            let swizzles: &[&str] = match dim {
                                2 => &[
                                    "x", "y", "r", "g",
                                    "xy", "yx", "xx", "yy",
                                    "rg", "gr", "rr", "gg",
                                ],
                                3 => &[
                                    "x", "y", "z", "r", "g", "b",
                                    "xy", "xz", "yz", "yx", "zx", "zy", "xx", "yy", "zz",
                                    "rg", "rb", "gb", "gr", "br", "bg", "rr", "gg", "bb",
                                    "xyz", "xzy", "yxz", "yzx", "zxy", "zyx",
                                    "xxx", "yyy", "zzz",
                                    "rgb", "rbg", "grb", "gbr", "brg", "bgr",
                                ],
                                _ => &[
                                    "x", "y", "z", "w", "r", "g", "b", "a",
                                    "xy", "xz", "xw", "yz", "yw", "zw",
                                    "yx", "zx", "wx", "zy", "wy", "wz",
                                    "xx", "yy", "zz", "ww",
                                    "rg", "rb", "ra", "gb", "ga", "ba",
                                    "gr", "br", "ar", "bg", "ag", "ab",
                                    "xyz", "xyw", "xzw", "yzw",
                                    "zyx", "wyx", "wzx", "wzy",
                                    "xxx", "yyy", "zzz", "www",
                                    "rgb", "rga", "gba", "bgr",
                                    "xyzw", "rgba", "bgra", "argb", "wzyx", "abgr",
                                ],
                            };

                            let items: Vec<Value> = swizzles.iter().enumerate().map(|(idx, sw)| {
                                json!({
                                    "label": *sw,
                                    "kind": 10,
                                    "detail": format!("Swizzle .{sw}"),
                                    "insertText": *sw,
                                    "sortText": format!("00_{:02}_{}", idx, sw),
                                })
                            }).collect();
                            return json!(items);
                        }
                    }

                    // Fallback for member access '.' when target_type could not be inferred:
                    // Provide swizzles, common struct fields (positionHCS, uv, etc.) and texture methods
                    let mut fallback_items = Vec::new();
                    let mut seen_fields = std::collections::HashSet::new();

                    // 1. Swizzles
                    const COMMON_SWIZZLES: &[&str] = &["x", "y", "z", "w", "xy", "zw", "xyz", "rgb", "rgba"];
                    for (idx, sw) in COMMON_SWIZZLES.iter().enumerate() {
                        if seen_fields.insert(sw.to_string()) {
                            fallback_items.push(json!({
                                "label": *sw,
                                "kind": 10,
                                "detail": format!("Swizzle .{sw}"),
                                "insertText": *sw,
                                "sortText": format!("00_{:02}_{}", idx, sw),
                            }));
                        }
                    }

                    // 2. Struct fields from known structs in this file/includes
                    for s in &structs {
                        for f in &s.fields {
                            if seen_fields.insert(f.name.clone()) {
                                fallback_items.push(json!({
                                    "label": f.name,
                                    "kind": 5,
                                    "detail": format!("{} (field of {})", f.field_type, s.name),
                                    "insertText": f.name,
                                    "sortText": format!("05_{}", f.name),
                                }));
                            }
                        }
                    }

                    // 3. Common texture methods
                    for (idx, m) in docs::TEXTURE_METHODS.iter().enumerate() {
                        if seen_fields.insert(m.name.to_string()) {
                            fallback_items.push(json!({
                                "label": m.name,
                                "kind": 2,
                                "detail": m.signature,
                                "documentation": { "kind": "markdown", "value": m.description },
                                "insertText": m.snippet,
                                "insertTextFormat": 2,
                                "command": {
                                    "title": "Trigger Parameter Hints",
                                    "command": "editor.action.triggerParameterHints"
                                },
                                "sortText": format!("10_{:02}_{}", idx, m.name),
                            }));
                        }
                    }

                    return json!(fallback_items);
                }
            }
        }
    }

    // Semantic completion when typing after ':'
    if let Some(colon_idx) = prefix.rfind(':') {
        let after_colon = prefix[colon_idx + 1..].trim();
        if !prefix.contains('?') && !prefix.contains(';') && !prefix.contains('{') && after_colon.chars().all(|c| c.is_alphanumeric() || c == '_') {
            let semantic_items: Vec<Value> = docs::BUILTIN_VARIABLES.iter().enumerate().map(|(idx, (sem, desc))| {
                json!({
                    "label": *sem,
                    "kind": 6,
                    "detail": "HLSL Semantic",
                    "documentation": { "kind": "markdown", "value": *desc },
                    "insertText": *sem,
                    "sortText": format!("00_{:02}_{}", idx, sem),
                })
            }).collect();
            return json!(semantic_items);
        }
    }

    let mut items = Vec::new();
    let mut seen_labels: std::collections::HashSet<String> = std::collections::HashSet::new();

    let (user_funcs, user_vars, user_structs) = signature::resolve_includes_and_scan_symbols(uri, doc, doc_cache);

    // 0. Parameters & Local Variables of Enclosing Function (Priority: HIGHEST!)
    if let Some(f) = signature::find_enclosing_function(&user_funcs, line_idx) {
        for (idx, p) in f.parsed_params.iter().enumerate() {
            if seen_labels.insert(p.name.clone()) {
                items.push(json!({
                    "label": p.name,
                    "kind": 6,
                    "detail": format!("{} {} (parameter)", p.param_type, p.name),
                    "documentation": format!("Parameter of function `{}`", f.name),
                    "insertText": p.name,
                    "sortText": format!("00_{:02}_{}", idx, p.name),
                }));
            }
        }

        for (idx, lv) in f.local_vars.iter().filter(|v| v.line <= line_idx).enumerate() {
            if seen_labels.insert(lv.name.clone()) {
                items.push(json!({
                    "label": lv.name,
                    "kind": 6,
                    "detail": format!("{} {} (local)", lv.var_type, lv.name),
                    "documentation": format!("Local variable declared in `{}` at line {}", f.name, lv.line + 1),
                    "insertText": lv.name,
                    "sortText": format!("01_{:02}_{}", idx, lv.name),
                }));
            }
        }
    }

    // 1. User-defined global variables & cbuffer members
    for (idx, v) in user_vars.iter().enumerate() {
        if seen_labels.insert(v.name.clone()) {
            let detail = if v.qualifier.is_empty() {
                format!("{} {}", v.var_type, v.name)
            } else {
                format!("{} {} ({})", v.var_type, v.name, v.qualifier)
            };
            items.push(json!({
                "label": v.name,
                "kind": 6,
                "detail": detail,
                "documentation": v.doc.as_deref().unwrap_or("Global variable"),
                "insertText": v.name,
                "sortText": format!("10_{:03}_{}", idx, v.name),
            }));
        }
    }

    // 2. User-defined functions
    for (idx, f) in user_funcs.iter().enumerate() {
        if seen_labels.insert(f.name.clone()) {
            items.push(json!({
                "label": f.name,
                "kind": 3,
                "detail": f.label,
                "documentation": f.doc.as_deref().unwrap_or("User function"),
                "insertText": format!("{}($1)", f.name),
                "insertTextFormat": 2,
                "command": {
                    "title": "Trigger Parameter Hints",
                    "command": "editor.action.triggerParameterHints"
                },
                "sortText": format!("15_{:03}_{}", idx, f.name),
            }));
        }
    }

    // 3. User structs (Types!)
    let structs = &user_structs;
    for (idx, s) in structs.iter().enumerate() {
        if seen_labels.insert(s.name.clone()) {
            let fields_doc = if s.fields.is_empty() {
                String::new()
            } else {
                let f_list = s.fields.iter().map(|f| format!("- `{} {}`", f.field_type, f.name)).collect::<Vec<_>>().join("\n");
                format!("\n\n### Fields:\n{}", f_list)
            };
            let doc_value = format!("```hlsl\nstruct {}\n```{}", s.name, fields_doc);

            items.push(json!({
                "label": s.name,
                "kind": 22, // Struct
                "detail": format!("struct {}", s.name),
                "documentation": {
                    "kind": "markdown",
                    "value": doc_value
                },
                "insertText": s.name,
                "sortText": format!("20_{:02}_{}", idx, s.name),
            }));
        }
    }

    // 3b. Material properties from ShaderLab Properties block (for HLSL & CBuffer completion)
    if context == ShaderContext::UnityShaderLab {
        let sl_props = signature::scan_shaderlab_properties(doc);
        for (idx, p) in sl_props.iter().enumerate() {
            if seen_labels.insert(p.name.clone()) {
                let hlsl_type = match p.prop_type.as_str() {
                    "Float" => "float",
                    "Int" => "int",
                    "Range" => "float",
                    s if s.starts_with("Range") => "float",
                    "Color" => "half4",
                    "Vector" => "float4",
                    "2D" => "Texture2D",
                    "3D" => "Texture3D",
                    "Cube" => "TextureCube",
                    _ => "float4",
                };
                items.push(json!({
                    "label": p.name,
                    "kind": 6, // Variable
                    "detail": format!("{} {} (from Properties)", hlsl_type, p.name),
                    "documentation": {
                        "kind": "markdown",
                        "value": format!("Material property `{}` (`{}`)\n\nShaderLab Property: `{}`", p.name, p.prop_type, p.display_name)
                    },
                    "insertText": p.name,
                    "sortText": format!("12_{:02}_{}", idx, p.name),
                }));
            }
        }
    }

    // 3c. Engine-filtered Built-in Variables & Macros (_Time, unity_ObjectToWorld, ResolvedView, etc.)
    for (idx, ev) in docs::ENGINE_VARIABLES.iter().enumerate() {
        let include_var = match context {
            ShaderContext::PureHlsl => false, // Pure HLSL is kept strictly free of engine pollution!
            ShaderContext::UnrealEngine => ev.engine == "unreal",
            ShaderContext::UnityShaderLab | ShaderContext::UnityHlsl => ev.engine == "unity",
        };

        if !include_var {
            continue;
        }

        if seen_labels.insert(ev.name.to_string()) {
            let kind = match ev.var_type {
                "function" => 3, // Function
                "macro" => 14,   // Keyword / Macro
                _ => 6,          // Variable
            };
            let (insert_text, insert_format) = if ev.var_type == "function" || ev.var_type == "macro" {
                (format!("{}($1)", ev.name), 2)
            } else {
                (ev.name.to_string(), 1)
            };

            let mut item = json!({
                "label": ev.name,
                "kind": kind,
                "detail": ev.detail,
                "documentation": {
                    "kind": "markdown",
                    "value": ev.description
                },
                "insertText": insert_text,
                "insertTextFormat": insert_format,
                "sortText": format!("15_{:02}_{}", idx, ev.name),
            });
            if ev.var_type == "function" || ev.var_type == "macro" {
                item["command"] = json!({
                    "title": "Trigger Parameter Hints",
                    "command": "editor.action.triggerParameterHints"
                });
            }
            items.push(item);
        }
    }

    // 4. Engine-filtered Built-in Functions
    for (idx, func) in docs::BUILTIN_FUNCTIONS.iter().enumerate() {
        let is_unity = func.description.contains("Unity");
        let is_unreal = func.description.contains("Unreal");

        let include_func = match context {
            ShaderContext::PureHlsl => !is_unity && !is_unreal,
            ShaderContext::UnrealEngine => !is_unity,
            ShaderContext::UnityShaderLab | ShaderContext::UnityHlsl => !is_unreal,
        };

        if !include_func {
            continue;
        }

        if seen_labels.insert(func.name.to_string()) {
            let primary_overload = func.overloads.first().map(|o| o.label).unwrap_or(func.name);
            items.push(json!({
                "label": func.name,
                "kind": 3,
                "detail": primary_overload,
                "documentation": {
                    "kind": "markdown",
                    "value": func.description
                },
                "insertText": format!("{}($1)", func.name),
                "insertTextFormat": 2,
                "command": {
                    "title": "Trigger Parameter Hints",
                    "command": "editor.action.triggerParameterHints"
                },
                "sortText": format!("30_{:03}_{}", idx, func.name),
            }));
        }
    }

    // 5. Built-in Types
    for (idx, t) in docs::BUILTIN_TYPES.iter().enumerate() {
        if seen_labels.insert((*t).to_string()) {
            items.push(json!({
                "label": *t,
                "kind": 7,
                "detail": "HLSL Type",
                "insertText": *t,
                "sortText": format!("35_{:03}_{}", idx, t),
            }));
        }
    }

    // 6. Built-in Keywords (HLSL only, exclude ShaderLab keywords)
    let hlsl_keywords = [
        "struct", "cbuffer", "tbuffer", "register", "static", "const", "inline",
        "return", "if", "else", "for", "while", "do", "switch", "case", "default",
        "break", "continue", "discard", "true", "false",
        "in", "out", "inout", "packoffset",
    ];
    for (idx, kw) in hlsl_keywords.iter().enumerate() {
        if seen_labels.insert((*kw).to_string()) {
            items.push(json!({
                "label": *kw,
                "kind": 14,
                "insertText": *kw,
                "sortText": format!("40_{:03}_{}", idx, kw),
            }));
        }
    }

    // 7. High-Productivity Snippets
    let snippets = [
        ("vert", "Vertex Shader function", "Varyings vert(Attributes input)\n{\n    Varyings output = (Varyings)0;\n    output.positionCS = TransformObjectToHClip(input.positionOS.xyz);\n    $0\n    return output;\n}"),
        ("frag", "Fragment/Pixel Shader function", "float4 frag(Varyings input) : SV_Target\n{\n    $0\n    return float4(1.0, 1.0, 1.0, 1.0);\n}"),
        ("kernel", "Compute Shader kernel", "[numthreads(${1:8}, ${2:8}, ${3:1})]\nvoid ${4:CSMain}(uint3 id : SV_DispatchThreadID)\n{\n    $0\n}"),
        ("struct", "Struct declaration", "struct $1\n{\n    $0\n};"),
        ("cbuffer", "Constant Buffer with register", "cbuffer $1 : register(b${2:0})\n{\n    $0\n};"),
        ("tex2d", "Texture2D and SamplerState pair", "Texture2D $1 : register(t${2:0});\nSamplerState sampler_$1 : register(s${2:0});"),
        ("for", "For loop", "for (int ${1:i} = 0; ${1:i} < ${2:count}; ++${1:i})\n{\n    $0\n}"),
        ("while", "While loop", "while ($1)\n{\n    $0\n}"),
        ("if", "If condition", "if ($1)\n{\n    $0\n}"),
        ("ifelse", "If-Else statement", "if ($1)\n{\n    $2\n}\nelse\n{\n    $0\n}"),
        ("switch", "Switch statement", "switch ($1)\n{\n    case $2:\n        break;\n    default:\n        break;\n}"),
    ];
    for (idx, (label, detail, snip)) in snippets.iter().enumerate() {
        items.push(json!({
            "label": *label,
            "kind": 15,
            "detail": *detail,
            "insertText": *snip,
            "insertTextFormat": 2,
            "sortText": format!("50_{:02}_{}", idx, label),
        }));
    }

    json!(items)
}

fn write_lsp_response(id: &Value, result: Value) {
    let resp = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    });
    send_lsp_payload(&resp);
}

fn send_lsp_payload(val: &Value) {
    let payload = serde_json::to_string(val).unwrap_or_default();
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = write!(handle, "Content-Length: {}\r\n\r\n{}", payload.len(), payload);
    let _ = handle.flush();
}

fn send_diagnostics(uri: &str, diags: &[Diagnostic]) {
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": diags
        }
    });
    send_lsp_payload(&notification);
}

fn main() {
    let dxc_path = find_dxc_path();

    let (tx, rx): (Sender<ValidationTask>, Receiver<ValidationTask>) = mpsc::channel();
    let mut doc_cache: HashMap<String, String> = HashMap::new();
    let workspace_root: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(
        env::var("WORKSPACE_ROOT").ok().map(PathBuf::from),
    ));

    let worker_dxc = dxc_path.clone();
    let worker_ws = Arc::clone(&workspace_root);
    thread::spawn(move || {
        let mut pending_task: Option<ValidationTask> = None;
        loop {
            while let Ok(task) = rx.try_recv() {
                pending_task = Some(task);
            }

            if let Some(task) = pending_task.take() {
                let mut interrupted = false;
                for _ in 0..12 {
                    thread::sleep(Duration::from_millis(10));
                    if let Ok(new_task) = rx.try_recv() {
                        pending_task = Some(new_task);
                        interrupted = true;
                        break;
                    }
                }

                if interrupted {
                    continue;
                }

                let ws_opt = worker_ws.lock().ok().and_then(|g| g.clone());
                let diags = validate_shader(&task.uri, &task.content, &worker_dxc, ws_opt.as_deref());
                send_diagnostics(&task.uri, &diags);
            } else {
                match rx.recv() {
                    Ok(task) => pending_task = Some(task),
                    Err(_) => break,
                }
            }
        }
    });

    let stdin = io::stdin();
    let mut reader = stdin.lock();

    loop {
        let mut line = String::new();
        let mut content_length: Option<usize> = None;

        loop {
            line.clear();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                return;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some(rest) = trimmed.strip_prefix("Content-Length:") {
                if let Ok(len) = rest.trim().parse::<usize>() {
                    content_length = Some(len);
                }
            }
        }

        let len = match content_length {
            Some(l) => l,
            None => continue,
        };

        let mut body = vec![0u8; len];
        if reader.read_exact(&mut body).is_err() {
            return;
        }

        let msg: Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = msg.get("id");

        match method {
            "initialize" => {
                if let Some(id) = id {
                    if let Some(params) = msg.get("params") {
                        let ws_path = params.get("rootUri")
                            .and_then(|u| u.as_str())
                            .and_then(uri_to_path)
                            .or_else(|| params.get("rootPath").and_then(|rp| rp.as_str()).map(PathBuf::from))
                            .or_else(|| {
                                params.get("workspaceFolders")
                                    .and_then(|wf| wf.as_array())
                                    .and_then(|arr| arr.first())
                                    .and_then(|f| f.get("uri"))
                                    .and_then(|u| u.as_str())
                                    .and_then(uri_to_path)
                            });
                        if let Some(ws) = ws_path {
                            if let Ok(mut g) = workspace_root.lock() {
                                *g = Some(ws);
                            }
                        }
                    }

                    let result = json!({
                        "capabilities": {
                            "textDocumentSync": 1,
                            "completionProvider": {
                                "triggerCharacters": [".", ">", ":", "\"", "/"],
                                "resolveProvider": false
                            },
                            "signatureHelpProvider": {
                                "triggerCharacters": ["(", ","],
                                "retriggerCharacters": [",", ")", " "]
                            },
                            "hoverProvider": true,
                            "definitionProvider": true,
                            "documentSymbolProvider": true,
                            "documentFormattingProvider": true
                        },
                        "serverInfo": {
                            "name": "hlsl_validator",
                            "version": "0.1.0"
                        }
                    });
                    write_lsp_response(id, result);
                }
            }
            "textDocument/didOpen" => {
                if let Some(params) = msg.get("params") {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                    let text = params["textDocument"]["text"].as_str().unwrap_or("").to_string();

                    doc_cache.insert(uri.clone(), text.clone());
                    let _ = tx.send(ValidationTask { uri, content: text });
                }
            }
            "textDocument/didChange" => {
                if let Some(params) = msg.get("params") {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                    if let Some(changes) = params["contentChanges"].as_array() {
                        if let Some(last_change) = changes.last() {
                            let text = last_change["text"].as_str().unwrap_or("").to_string();
                            doc_cache.insert(uri.clone(), text.clone());
                            let _ = tx.send(ValidationTask { uri, content: text });
                        }
                    }
                }
            }
            "textDocument/didClose" => {
                if let Some(params) = msg.get("params") {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    doc_cache.remove(uri);
                    send_diagnostics(uri, &[]);
                }
            }
            "textDocument/completion" => {
                if let Some(id) = id {
                    let res = handle_completion(&msg, &doc_cache);
                    write_lsp_response(id, res);
                }
            }
            "textDocument/signatureHelp" => {
                if let Some(id) = id {
                    let params = &msg["params"];
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    let line = params["position"]["line"].as_u64().unwrap_or(0) as usize;
                    let col = params["position"]["character"].as_u64().unwrap_or(0) as usize;

                    let owned_doc;
                    let doc = match get_document_from_cache(&doc_cache, uri) {
                        Some(d) => d,
                        None => {
                            owned_doc = uri_to_path(uri).and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
                            owned_doc.as_str()
                        }
                    };
                    let res = signature::get_signature_help(uri, doc, line, col, &doc_cache);
                    write_lsp_response(id, res);
                }
            }
            "textDocument/hover" => {
                if let Some(id) = id {
                    let params = &msg["params"];
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    let line = params["position"]["line"].as_u64().unwrap_or(0) as usize;
                    let col = params["position"]["character"].as_u64().unwrap_or(0) as usize;

                    let owned_doc;
                    let doc = match get_document_from_cache(&doc_cache, uri) {
                        Some(d) => d,
                        None => {
                            owned_doc = uri_to_path(uri).and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
                            owned_doc.as_str()
                        }
                    };
                    let res = signature::get_hover_info(uri, doc, line, col, &doc_cache);
                    write_lsp_response(id, res);
                }
            }
            "textDocument/definition" => {
                if let Some(id) = id {
                    let res = signature::handle_definition(&msg, &doc_cache);
                    write_lsp_response(id, res);
                }
            }
            "textDocument/documentSymbol" => {
                if let Some(id) = id {
                    let params = &msg["params"];
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    let owned_doc;
                    let doc = match get_document_from_cache(&doc_cache, uri) {
                        Some(d) => d,
                        None => {
                            owned_doc = uri_to_path(uri).and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
                            owned_doc.as_str()
                        }
                    };
                    let symbols = signature::get_document_symbols(doc);
                    write_lsp_response(id, symbols);
                }
            }
            "textDocument/formatting" => {
                if let Some(id) = id {
                    let params = &msg["params"];
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    let options = &params["options"];
                    let tab_size = options["tabSize"].as_u64().unwrap_or(4) as usize;
                    let insert_spaces = options["insertSpaces"].as_bool().unwrap_or(true);

                    let owned_doc;
                    let doc = match get_document_from_cache(&doc_cache, uri) {
                        Some(d) => d,
                        None => {
                            owned_doc = uri_to_path(uri).and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
                            owned_doc.as_str()
                        }
                    };
                    let edits = format_document(doc, tab_size, insert_spaces);
                    write_lsp_response(id, json!(edits));
                }
            }
            "shutdown" => {
                if let Some(id) = id {
                    write_lsp_response(id, json!(null));
                }
            }
            "exit" => {
                return;
            }
            _ => {
                if let Some(id) = id {
                    write_lsp_response(id, json!(null));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docs_builtin_functions() {
        let lerp = docs::find_builtin_function("lerp").expect("lerp should exist in docs");
        assert_eq!(lerp.name, "lerp");
        assert!(!lerp.overloads.is_empty());
        assert!(lerp.description.contains("linear interpolation"));

        let saturate = docs::find_builtin_function("saturate").expect("saturate should exist in docs");
        assert_eq!(saturate.name, "saturate");

        let mul = docs::find_builtin_function("mul").expect("mul should exist in docs");
        assert_eq!(mul.name, "mul");

        let unity_transform = docs::find_builtin_function("TransformObjectToHClip").expect("TransformObjectToHClip should exist");
        assert_eq!(unity_transform.name, "TransformObjectToHClip");

        let unreal_norm = docs::find_builtin_function("GetWorldNormal").expect("GetWorldNormal should exist");
        assert_eq!(unreal_norm.name, "GetWorldNormal");
    }

    #[test]
    fn test_signature_help_parsing() {
        let code = "float4 col = lerp(colorA, colorB, ";
        let (fn_name, param_idx) = signature::find_enclosing_call(code, 0, code.len()).expect("Should find call");
        assert_eq!(fn_name, "lerp");
        assert_eq!(param_idx, 2);

        let code_first_param = "float x = saturate(";
        let (fn_name_sat, param_idx_sat) = signature::find_enclosing_call(code_first_param, 0, code_first_param.len()).expect("Should find call");
        assert_eq!(fn_name_sat, "saturate");
        assert_eq!(param_idx_sat, 0);
    }

    #[test]
    fn test_range_signature_help() {
        let cache = HashMap::new();
        let code = "    _Gloss (\"Smoothness\", Range(0, ";
        let sig = signature::get_signature_help("file:///test.shader", code, 0, code.len(), &cache);
        assert!(!sig.is_null());
        assert_eq!(sig["signatures"][0]["label"], "Range(float min, float max)");
        assert_eq!(sig["activeParameter"], 1);
    }

    #[test]
    fn test_hover_info() {
        let cache = HashMap::new();
        let code = "float v = saturate(val);";
        let hover = signature::get_hover_info("file:///test.hlsl", code, 0, 12, &cache);
        assert!(!hover.is_null());
        let val_str = hover["contents"]["value"].as_str().unwrap();
        assert!(val_str.contains("saturate"));
    }

    #[test]
    fn test_user_symbol_scanning() {
        let code = r#"
struct Attributes {
    float3 positionOS : POSITION;
    float2 uv : TEXCOORD0;
};

float4 MyVertShader(Attributes input) : SV_Position {
    return float4(input.positionOS, 1.0);
}
"#;
        let funcs = signature::scan_user_functions(code, None, None);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "MyVertShader");
        assert_eq!(funcs[0].parameters.len(), 1);
        assert_eq!(funcs[0].parsed_params.len(), 1);
        assert_eq!(funcs[0].parsed_params[0].name, "input");
        assert_eq!(funcs[0].parsed_params[0].param_type, "Attributes");

        let structs = signature::scan_struct_definitions(code);
        assert!(structs.iter().any(|s| s.name == "Attributes" && s.fields.len() == 2));
    }

    #[test]
    fn test_function_parameter_completion_and_deduplication() {
        let code = r#"
struct VSInput {
    float3 position : POSITION;
    float4 color : COLOR;
};

struct VSOutput {
    float4 position : SV_Position;
    float4 color : COLOR;
};

VSOutput VSMain(VSInput input)
{
    VSOutput output;

    output.position = float4(inpu, 1.0);
    output.color = input.color;

    return output;
}
"#;
        let mut cache = HashMap::new();
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        // Cursor at line 17: "output.position = float4(inpu, 1.0);" right after 'inpu'
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 17, "character": 33 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result should be array");

        // 1. Parameter "input" MUST be present with top priority!
        let input_item = arr.iter().find(|i| i["label"] == "input");
        assert!(input_item.is_some(), "Function parameter 'input' MUST be in completions!");
        let input_obj = input_item.unwrap();
        assert_eq!(input_obj["kind"], 6);
        assert!(input_obj["sortText"].as_str().unwrap().starts_with("00_"));

        // 2. Local variable "output" MUST be present!
        let output_item = arr.iter().find(|i| i["label"] == "output");
        assert!(output_item.is_some(), "Local variable 'output' MUST be in completions!");
        assert_eq!(output_item.unwrap()["kind"], 6);
        assert!(output_item.unwrap()["sortText"].as_str().unwrap().starts_with("01_"));

        // 3. "VSInput" MUST NOT be duplicated!
        let vsinput_count = arr.iter().filter(|i| i["label"] == "VSInput").count();
        assert_eq!(vsinput_count, 1, "VSInput should appear exactly once, got {}", vsinput_count);
    }

    #[test]
    fn test_parameter_hover_and_definition() {
        let code = r#"
VSOutput VSMain(VSInput input)
{
    VSOutput output;
    output.color = input.color;
    return output;
}
"#;
        let mut cache = HashMap::new();
        cache.insert("file:///test.hlsl".to_string(), code.to_string());

        // Test Hover on "input" (line 4: "    output.color = input.color;")
        let hover_val = signature::get_hover_info("file:///test.hlsl", code, 4, 20, &cache);
        assert!(!hover_val.is_null(), "Hover on parameter 'input' should return markdown!");
        let hover_md = hover_val["contents"]["value"].as_str().unwrap();
        assert!(hover_md.contains("VSInput input"), "Hover should show 'VSInput input'");
        assert!(hover_md.contains("parameter of `VSMain`"), "Hover should mention 'parameter of VSMain'");

        // Test Go to Definition on "input"
        let def_req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 4, "character": 20 }
            }
        });
        let def_val = signature::handle_definition(&def_req, &cache);
        assert!(!def_val.is_null(), "Go to definition on 'input' should not be null!");
        assert_eq!(def_val["range"]["start"]["line"], 1, "Parameter 'input' is declared on line 1");

        // Test Hover on "output" (line 4: "    output.color = input.color;")
        let hover_out = signature::get_hover_info("file:///test.hlsl", code, 4, 6, &cache);
        assert!(!hover_out.is_null(), "Hover on local 'output' should return markdown!");
        let hover_out_md = hover_out["contents"]["value"].as_str().unwrap();
        assert!(hover_out_md.contains("VSOutput output"), "Hover should show 'VSOutput output'");
        assert!(hover_out_md.contains("local variable in `VSMain`"), "Hover should mention local variable");

        // Test Go to Definition on "output"
        let def_out_req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 4, "character": 6 }
            }
        });
        let def_out_val = signature::handle_definition(&def_out_req, &cache);
        assert!(!def_out_val.is_null(), "Go to definition on 'output' should not be null!");
        assert_eq!(def_out_val["range"]["start"]["line"], 3, "Local 'output' is declared on line 3");
    }

    #[test]
    fn test_validate_pure_hlsl_valid() {
        let dxc = find_dxc_path();
        let valid_shader = r#"
float4 MainVs(float3 pos : POSITION) : SV_Position {
    return float4(pos, 1.0);
}
"#;
        let diags = validate_shader("file:///test.hlsl", valid_shader, &dxc, None);
        assert!(diags.is_empty(), "Expected 0 diagnostics for valid pure HLSL, got: {:?}", diags);
    }

    #[test]
    fn test_validate_pure_hlsl_error() {
        let dxc = find_dxc_path();
        let invalid_shader = r#"
float4 MainVs(float3 pos : POSITION) : SV_Position {
    return float4(pos, undefined_variable);
}
"#;
        let diags = validate_shader("file:///test.hlsl", invalid_shader, &dxc, None);
        assert!(!diags.is_empty(), "Expected diagnostics for undefined variable in pure HLSL");
        assert!(diags.iter().any(|d| d.message.contains("undefined_variable") || d.severity == 1));
    }

    #[test]
    fn test_validate_unity_unlit_shader() {
        let dxc = find_dxc_path();
        let unity_shader = r#"
Shader "Unlit/TestUnlit"
{
    Properties
    {
        _MainTex ("Texture", 2D) = "white" {}
    }
    SubShader
    {
        Tags { "RenderType"="Opaque" }
        Pass
        {
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "UnityCG.cginc"

            struct appdata
            {
                float4 vertex : POSITION;
                float2 uv : TEXCOORD0;
            };

            struct v2f
            {
                float2 uv : TEXCOORD0;
                float4 vertex : SV_POSITION;
            };

            sampler2D _MainTex;
            float4 _MainTex_ST;

            v2f vert (appdata v)
            {
                v2f o;
                o.vertex = UnityObjectToClipPos(v.vertex);
                o.uv = TRANSFORM_TEX(v.uv, _MainTex);
                return o;
            }

            fixed4 frag (v2f i) : SV_Target
            {
                fixed4 col = tex2D(_MainTex, i.uv);
                return col;
            }
            ENDCG
        }
    }
}
"#;
        let diags = validate_shader("file:///test_unlit.shader", unity_shader, &dxc, None);
        assert!(diags.is_empty(), "Standard Unity Unlit shader MUST have 0 errors, got: {:?}", diags);
    }

    #[test]
    fn test_validate_unity_2d_sprite_shader() {
        let dxc = find_dxc_path();
        let sprite_shader = r#"
Shader "Sprites/Custom2DSprite"
{
    Properties
    {
        [PerRendererData] _MainTex ("Sprite Texture", 2D) = "white" {}
        _Color ("Tint", Color) = (1,1,1,1)
        [MaterialToggle] PixelSnap ("Pixel snap", Float) = 0
        [HideInInspector] _RendererColor ("RendererColor", Color) = (1,1,1,1)
        [HideInInspector] _Flip ("Flip", Vector) = (1,1,1,1)
    }
    SubShader
    {
        Tags
        {
            "Queue"="Transparent"
            "IgnoreProjector"="True"
            "RenderType"="Transparent"
            "PreviewType"="Plane"
            "CanUseSpriteAtlas"="True"
        }
        Cull Off
        Lighting Off
        ZWrite Off
        Blend One OneMinusSrcAlpha
        Pass
        {
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "UnityCG.cginc"

            struct appdata_t
            {
                float4 vertex   : POSITION;
                float4 color    : COLOR;
                float2 texcoord : TEXCOORD0;
            };

            struct v2f
            {
                float4 vertex   : SV_POSITION;
                fixed4 color    : COLOR;
                float2 texcoord : TEXCOORD0;
            };

            sampler2D _MainTex;
            fixed4 _Color;
            fixed4 _RendererColor;
            float4 _Flip;

            v2f vert(appdata_t IN)
            {
                v2f OUT;
                IN.vertex.xy *= _Flip.xy;
                OUT.vertex = UnityObjectToClipPos(IN.vertex);
                OUT.texcoord = IN.texcoord;
                OUT.color = IN.color * _Color * _RendererColor;
                OUT.vertex = UnityPixelSnap(OUT.vertex);
                return OUT;
            }

            fixed4 frag(v2f IN) : SV_Target
            {
                fixed4 c = tex2D(_MainTex, IN.texcoord) * IN.color;
                c.rgb *= c.a;
                return c;
            }
            ENDCG
        }
    }
}
"#;
        let diags = validate_shader("file:///Assets/Sprites/Custom2DSprite.shader", sprite_shader, &dxc, None);
        assert!(diags.is_empty(), "Unity 2D Sprite shader MUST validate with 0 errors, got: {:?}", diags);
    }

    #[test]
    fn test_validate_unity_2d_ui_shader() {
        let dxc = find_dxc_path();
        let ui_shader = r#"
Shader "UI/Custom2DUI"
{
    Properties
    {
        [PerRendererData] _MainTex ("Sprite Texture", 2D) = "white" {}
        _Color ("Tint", Color) = (1,1,1,1)
        _ClipRect ("Clip Rect", Vector) = (-32767, -32767, 32767, 32767)
    }
    SubShader
    {
        Tags
        {
            "Queue"="Transparent"
            "IgnoreProjector"="True"
            "RenderType"="Transparent"
            "PreviewType"="Plane"
            "CanUseSpriteAtlas"="True"
        }
        Cull Off
        Lighting Off
        ZWrite Off
        Blend SrcAlpha OneMinusSrcAlpha
        Pass
        {
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "UnityCG.cginc"

            struct appdata_t
            {
                float4 vertex   : POSITION;
                float4 color    : COLOR;
                float2 texcoord : TEXCOORD0;
            };

            struct v2f
            {
                float4 vertex   : SV_POSITION;
                fixed4 color    : COLOR;
                float2 texcoord : TEXCOORD0;
                float4 worldPosition : TEXCOORD1;
            };

            sampler2D _MainTex;
            fixed4 _Color;
            float4 _ClipRect;

            v2f vert(appdata_t v)
            {
                v2f OUT;
                OUT.worldPosition = v.vertex;
                OUT.vertex = UnityObjectToClipPos(OUT.worldPosition);
                OUT.texcoord = v.texcoord;
                OUT.color = v.color * _Color;
                return OUT;
            }

            fixed4 frag(v2f IN) : SV_Target
            {
                half4 color = tex2D(_MainTex, IN.texcoord) * IN.color;
                color.a *= UnityGet2DClipping(IN.worldPosition.xy, _ClipRect);
                return color;
            }
            ENDCG
        }
    }
}
"#;
        let diags = validate_shader("file:///Assets/UI/Custom2DUI.shader", ui_shader, &dxc, None);
        assert!(diags.is_empty(), "Unity 2D UI Canvas shader MUST validate with 0 errors, got: {:?}", diags);
    }

    #[test]
    fn test_validate_unreal_shader() {
        let dxc = find_dxc_path();
        let unreal_shader = r#"
float3 CustomUnrealLighting(float3 WorldPos, float3 WorldNormal, float3 LightDir)
{
    float NdotL = saturate(dot(WorldNormal, LightDir));
    float3 lum = Luminance(LightDir);
    return RotateAboutAxis(float4(WorldNormal, 0.5), WorldPos, LightDir) * (NdotL + lum);
}
"#;
        let diags = validate_shader("file:///test_unreal.usf", unreal_shader, &dxc, None);
        assert!(diags.is_empty(), "Unreal shader MUST have 0 errors, got: {:?}", diags);
    }

    #[test]
    fn test_validate_all_sample_files() {
        let dxc = find_dxc_path();

        // 1. Pure HLSL Sample
        if let Ok(pure_code) = std::fs::read_to_string("../samples/pure_sample.hlsl") {
            let context = detect_shader_context("file:///pure_sample.hlsl", &pure_code);
            assert_eq!(context, ShaderContext::PureHlsl);
            let diags = validate_shader("file:///pure_sample.hlsl", &pure_code, &dxc, None);
            assert!(diags.is_empty(), "Sample pure HLSL MUST have 0 errors, got: {:?}", diags);
        }

        // 2. Unity Unlit Sample
        if let Ok(unity_code) = std::fs::read_to_string("../samples/unity_unlit.shader") {
            let context = detect_shader_context("file:///unity_unlit.shader", &unity_code);
            assert_eq!(context, ShaderContext::UnityShaderLab);
            let diags = validate_shader("file:///unity_unlit.shader", &unity_code, &dxc, None);
            assert!(diags.is_empty(), "Sample Unity Unlit shader MUST have 0 errors, got: {:?}", diags);
        }

        // 3. Unity 2D Sprite Sample
        if let Ok(sprite_code) = std::fs::read_to_string("../samples/unity_sprite.shader") {
            let context = detect_shader_context("file:///unity_sprite.shader", &sprite_code);
            assert_eq!(context, ShaderContext::UnityShaderLab);
            assert!(is_unity_2d_context("file:///unity_sprite.shader", &sprite_code));
            let diags = validate_shader("file:///unity_sprite.shader", &sprite_code, &dxc, None);
            assert!(diags.is_empty(), "Sample Unity 2D Sprite shader MUST have 0 errors, got: {:?}", diags);
        }

        // 4. Unreal Engine USF Sample
        if let Ok(unreal_code) = std::fs::read_to_string("../samples/unreal_sample.usf") {
            let context = detect_shader_context("file:///unreal_sample.usf", &unreal_code);
            assert_eq!(context, ShaderContext::UnrealEngine);
            let diags = validate_shader("file:///unreal_sample.usf", &unreal_code, &dxc, None);
            assert!(diags.is_empty(), "Sample Unreal shader MUST have 0 errors, got: {:?}", diags);
        }
    }

    #[test]
    fn test_validate_unity_urp_shader() {
        let dxc = find_dxc_path();
        let urp_shader = r#"
Shader "Universal/TestUnlit"
{
    Properties
    {
        [MainTexture] _BaseMap("Texture", 2D) = "white" {}
        [MainColor] _BaseColor("Color", Color) = (1, 1, 1, 1)
    }
    SubShader
    {
        Tags { "RenderType"="Opaque" "RenderPipeline"="UniversalPipeline" }
        Pass
        {
            HLSLPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Core.hlsl"

            struct Attributes
            {
                float4 positionOS   : POSITION;
                float2 uv           : TEXCOORD0;
            };

            struct Varyings
            {
                float4 positionHCS  : SV_POSITION;
                float2 uv           : TEXCOORD0;
            };

            TEXTURE2D(_BaseMap);
            SAMPLER(sampler_BaseMap);

            CBUFFER_START(UnityPerMaterial)
                half4 _BaseColor;
                float4 _BaseMap_ST;
            CBUFFER_END

            Varyings vert(Attributes IN)
            {
                Varyings OUT;
                OUT.positionHCS = TransformObjectToHClip(IN.positionOS.xyz);
                OUT.uv = TRANSFORM_TEX(IN.uv, _BaseMap);
                return OUT;
            }

            half4 frag(Varyings IN) : SV_Target
            {
                half4 color = SAMPLE_TEXTURE2D(_BaseMap, sampler_BaseMap, IN.uv) * _BaseColor;
                return color;
            }
            ENDHLSL
        }
    }
}
"#;
        let diags = validate_shader("file:///test_urp.shader", urp_shader, &dxc, None);
        assert!(diags.is_empty(), "Valid URP shader MUST have 0 errors, got: {:?}", diags);

        let broken_urp = urp_shader.replace("Varyings OUT;", "Var");
        let broken_diags = validate_shader("file:///broken_urp.shader", &broken_urp, &dxc, None);
        assert!(!broken_diags.is_empty(), "Broken URP shader MUST report syntax error");
        assert!(broken_diags.iter().any(|d| d.message.contains("Var")), "Diagnostics must flag 'Var': {:?}", broken_diags);
    }

    #[test]
    fn test_context_filter_completion() {
        let mut cache = HashMap::new();
        // 1. Pure HLSL - should NOT have Unity TransformObjectToHClip or Unreal RotateAboutAxis
        cache.insert("file:///pure.hlsl".to_string(), "float4 frag() : SV_Target {\n    \n}".to_string());
        let req_pure = json!({
            "params": {
                "textDocument": { "uri": "file:///pure.hlsl" },
                "position": { "line": 1, "character": 4 }
            }
        });
        let res_pure = handle_completion(&req_pure, &cache);
        let arr_pure = res_pure.as_array().unwrap();
        assert!(arr_pure.iter().any(|i| i["label"] == "lerp"));
        assert!(!arr_pure.iter().any(|i| i["label"] == "TransformObjectToHClip"));
        assert!(!arr_pure.iter().any(|i| i["label"] == "RotateAboutAxis"));

        // 2. Unreal - should have Unreal functions but NOT Unity functions
        cache.insert("file:///unreal.usf".to_string(), "float3 frag() { ".to_string());
        let req_unreal = json!({
            "params": {
                "textDocument": { "uri": "file:///unreal.usf" },
                "position": { "line": 0, "character": 16 }
            }
        });
        let res_unreal = handle_completion(&req_unreal, &cache);
        let arr_unreal = res_unreal.as_array().unwrap();
        assert!(arr_unreal.iter().any(|i| i["label"] == "RotateAboutAxis"));
        assert!(!arr_unreal.iter().any(|i| i["label"] == "TransformObjectToHClip"));

        // 3. Unity HLSL - should have Unity functions but NOT Unreal functions
        cache.insert("file:///unity.cginc".to_string(), "float4 frag() { ".to_string());
        let req_unity = json!({
            "params": {
                "textDocument": { "uri": "file:///unity.cginc" },
                "position": { "line": 0, "character": 16 }
            }
        });
        let res_unity = handle_completion(&req_unity, &cache);
        let arr_unity = res_unity.as_array().unwrap();
        assert!(arr_unity.iter().any(|i| i["label"] == "TransformObjectToHClip"));
        assert!(!arr_unity.iter().any(|i| i["label"] == "RotateAboutAxis"));
    }

    #[test]
    fn test_shaderlab_properties_completion() {
        let mut cache = HashMap::new();
        let code = "Shader \"Test\" {\nProperties {\n    _\n}\n}";
        cache.insert("file:///test.shader".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.shader" },
                "position": { "line": 2, "character": 5 }
            }
        });
        let res = handle_completion(&req, &cache);
        let arr = res.as_array().unwrap();
        assert!(arr.iter().any(|i| i["label"] == "_MainTex"));
        assert!(arr.iter().any(|i| i["label"] == "_Color"));
        assert!(arr.iter().any(|i| i["label"] == "Range"));
    }

    #[test]
    fn test_member_access_completion() {
        let mut cache = HashMap::new();
        let code = r#"
struct Varyings {
    float4 position : SV_Position;
    float4 color : COLOR;
};

float4 frag(Varyings output) : SV_Target {
    output.po
}
"#;
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 7, "character": 13 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "position"));
        assert!(arr.iter().any(|item| item["label"] == "color"));
    }

    #[test]
    fn test_texture_method_completion() {
        let mut cache = HashMap::new();
        let code = r#"
Texture2D _MainTex;
SamplerState sampler_MainTex;

float4 frag() : SV_Target {
    _MainTex.
}
"#;
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 5, "character": 13 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "Sample"));
        assert!(arr.iter().any(|item| item["label"] == "SampleLevel"));
        assert!(arr.iter().any(|item| item["label"] == "Load"));
    }

    #[test]
    fn test_semantic_completion() {
        let mut cache = HashMap::new();
        let code = "float4 pos : SV_";
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 0, "character": 16 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "SV_Position"));
        assert!(arr.iter().any(|item| item["label"] == "SV_Target"));
    }

    #[test]
    fn test_document_symbols() {
        let code = r#"
Shader "Custom/MyShader"
{
    SubShader
    {
        Pass
        {
            struct Attributes {
                float3 pos : POSITION;
            };

            float4 vert(Attributes input) : SV_Position {
                return float4(input.pos, 1.0);
            }
        }
    }
}
"#;
        let symbols = signature::get_document_symbols(code);
        let arr = symbols.as_array().expect("Symbols should be array");
        assert!(arr.iter().any(|s| s["name"] == "Custom/MyShader"));
        assert!(arr.iter().any(|s| s["name"] == "SubShader"));
        assert!(arr.iter().any(|s| s["name"] == "Pass"));
        assert!(arr.iter().any(|s| s["name"] == "Attributes"));
        assert!(arr.iter().any(|s| s["name"] == "vert"));
    }

    #[test]
    fn test_format_document() {
        let messy = "Shader \"Test\" {\nSubShader {\nPass {\nHLSLPROGRAM\nfloat4 frag() : SV_Target {\nreturn 1;\n}\nENDHLSL\n}\n}\n}";
        let edits = format_document(messy, 4, true);
        assert!(!edits.is_empty());
        let new_text = edits[0]["newText"].as_str().unwrap();
        assert!(new_text.contains("    SubShader"));
        assert!(new_text.contains("        Pass"));
    }

    #[test]
    fn test_matrix_element_completion() {
        let mut cache = HashMap::new();
        let code = r#"
void TestMatrix()
{
    float4x4 mvp;
    float val = mvp.
}
"#;
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 4, "character": 20 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "_m00"), "Matrix should offer _m00");
        assert!(arr.iter().any(|item| item["label"] == "_m33"), "Matrix should offer _m33");
        assert!(arr.iter().any(|item| item["label"] == "_11"), "Matrix should offer _11");
        assert!(arr.iter().any(|item| item["label"] == "_44"), "Matrix should offer _44");
    }

    #[test]
    fn test_vector_swizzle_dimension() {
        let mut cache = HashMap::new();
        let code = r#"
void TestVec()
{
    float2 uv;
    float val = uv.
}
"#;
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 4, "character": 19 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "x"), "float2 should have x");
        assert!(arr.iter().any(|item| item["label"] == "y"), "float2 should have y");
        assert!(arr.iter().any(|item| item["label"] == "xy"), "float2 should have xy");
        assert!(!arr.iter().any(|item| item["label"] == "z"), "float2 MUST NOT have z");
        assert!(!arr.iter().any(|item| item["label"] == "w"), "float2 MUST NOT have w");
        assert!(!arr.iter().any(|item| item["label"] == "rgba"), "float2 MUST NOT have rgba");
    }

    #[test]
    fn test_extended_semantics() {
        let mut cache = HashMap::new();
        let code = "float4 col : SV_";
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 0, "character": 16 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "SV_Target7"), "Should have SV_Target7");
        assert!(arr.iter().any(|item| item["label"] == "SV_DepthGreaterEqual"), "Should have SV_DepthGreaterEqual");
        assert!(arr.iter().any(|item| item["label"] == "SV_IsFrontFace"), "Should have SV_IsFrontFace");
        assert!(arr.iter().any(|item| item["label"] == "SV_Barycentrics"), "Should have SV_Barycentrics");
        assert!(arr.iter().any(|item| item["label"] == "BINORMAL0"), "Should have BINORMAL0");
    }

    #[test]
    fn test_shader_snippets() {
        let mut cache = HashMap::new();
        let code = "\n";
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 0, "character": 0 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "kernel"), "Should have kernel snippet");
        assert!(arr.iter().any(|item| item["label"] == "tex2d"), "Should have tex2d snippet");
        assert!(arr.iter().any(|item| item["label"] == "cbuffer"), "Should have cbuffer snippet");
        assert!(arr.iter().any(|item| item["label"] == "for"), "Should have for snippet");
    }

    #[test]
    fn test_shaderlab_nested_structs_completion() {
        let code = r#"Shader "Test/URPUnlit"
{
    SubShader
    {
        Pass
        {
            HLSLPROGRAM
            struct Attributes
            {
                float4 positionOS : POSITION;
            };

            struct Varyings
            {
                float4 positionCS : SV_POSITION;
            };

            Vary
            ENDHLSL
        }
    }
}"#;
        let structs = signature::scan_struct_definitions(code);
        assert_eq!(structs.len(), 2, "Should discover both Attributes and Varyings");
        assert_eq!(structs[0].name, "Attributes");
        assert_eq!(structs[1].name, "Varyings");

        let mut cache = HashMap::new();
        cache.insert("file:///test.shader".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.shader" },
                "position": { "line": 17, "character": 16 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "Varyings"), "Completion should contain Varyings");
        assert!(arr.iter().any(|item| item["label"] == "Attributes"), "Completion should contain Attributes");
    }

    #[test]
    fn test_shaderlab_property_syntax_error_and_cbuffer_warning() {
        // Test malformed property without identifier
        let code_with_syntax_err = r#"Shader "Test/Error"
{
    Properties
    {
        ("Float", Float) = 0.0
    }
    SubShader
    {
        Pass
        {
            HLSLPROGRAM
            ENDHLSL
        }
    }
}"#;
        let diags = validate_shaderlab_properties_and_cbuffer(code_with_syntax_err);
        assert!(diags.iter().any(|d| d.severity == 1 && d.message.contains("Missing property name")),
            "Should report syntax error for property missing name");

        // Test missing CBuffer warning
        let code_missing_cbuffer = r#"Shader "Test/URP"
{
    Properties
    {
        _BaseColor("Base Color", Color) = (1, 1, 1, 1)
        _Speed("Speed", Float) = 1.0
    }
    SubShader
    {
        Tags { "RenderPipeline" = "UniversalPipeline" }
        Pass
        {
            HLSLPROGRAM
            // No CBUFFER_START(UnityPerMaterial)
            ENDHLSL
        }
    }
}"#;
        let diags2 = validate_shaderlab_properties_and_cbuffer(code_missing_cbuffer);
        assert!(diags2.iter().any(|d| d.severity == 2 && d.message.contains("UnityPerMaterial")),
            "Should warn about missing CBUFFER_START(UnityPerMaterial)");

        // Test property inside CBuffer -> no warning
        let code_with_cbuffer = r#"Shader "Test/URP"
{
    Properties
    {
        _BaseColor("Base Color", Color) = (1, 1, 1, 1)
    }
    SubShader
    {
        Tags { "RenderPipeline" = "UniversalPipeline" }
        Pass
        {
            HLSLPROGRAM
            CBUFFER_START(UnityPerMaterial)
                half4 _BaseColor;
            CBUFFER_END
            ENDHLSL
        }
    }
}"#;
        let diags3 = validate_shaderlab_properties_and_cbuffer(code_with_cbuffer);
        assert!(!diags3.iter().any(|d| d.message.contains("not declared in 'CBUFFER_START(UnityPerMaterial)'")),
            "Should not warn when property is declared in CBuffer");
    }

    #[test]
    fn test_auto_context_detection() {
        // Pure HLSL: standard HLSL without Unity or Unreal indicators
        let pure_hlsl = "Texture2D tex : register(t0);\nfloat4 main() : SV_Target { return float4(1, 0, 0, 1); }";
        assert_eq!(detect_shader_context("file:///C:/projects/game/render.hlsl", pure_hlsl), ShaderContext::PureHlsl);

        // Unity ShaderLab: .shader extension
        let sl_code = "Shader \"Custom/MyShader\" { SubShader { Pass {} } }";
        assert_eq!(detect_shader_context("file:///C:/projects/MyShader.shader", sl_code), ShaderContext::UnityShaderLab);

        // Unity HLSL: detected by _Time in content
        let unity_hlsl_by_var = "float4 main() : SV_Target { return _Time; }";
        assert_eq!(detect_shader_context("file:///C:/projects/render.hlsl", unity_hlsl_by_var), ShaderContext::UnityHlsl);

        // Unity HLSL: detected by URP transform function
        let unity_hlsl_by_fn = "float4 main(float3 pos : POSITION) : SV_Position { return TransformObjectToHClip(pos); }";
        assert_eq!(detect_shader_context("file:///C:/projects/vert.hlsl", unity_hlsl_by_fn), ShaderContext::UnityHlsl);

        // Unreal Engine: detected by .ush extension or ResolvedView
        let ue_code = "float4 main() : SV_Target { return ResolvedView.WorldCameraOrigin.xyzz; }";
        assert_eq!(detect_shader_context("file:///C:/UnrealProjects/MyGame/Shaders/Private/Test.usf", ue_code), ShaderContext::UnrealEngine);
        assert_eq!(detect_shader_context("file:///C:/somewhere/test.hlsl", ue_code), ShaderContext::UnrealEngine);
    }

    #[test]
    fn test_unity_builtins_completion_and_filtering() {
        let mut cache = HashMap::new();

        // 1. Unity context: _Time, unity_ObjectToWorld, etc. MUST be present
        let unity_code = "float4 main() : SV_Target\n{\n    _Ti\n    return float4(0, 0, 0, 0);\n}";
        cache.insert("file:///C:/UnityProject/Assets/Shaders/test.hlsl".to_string(), unity_code.to_string());
        let req_unity = json!({
            "params": {
                "textDocument": { "uri": "file:///C:/UnityProject/Assets/Shaders/test.hlsl" },
                "position": { "line": 2, "character": 7 }
            }
        });
        let res_unity = handle_completion(&req_unity, &cache);
        let items_unity = res_unity.as_array().expect("Must be array");
        assert!(items_unity.iter().any(|i| i["label"] == "_Time"), "Unity completion MUST include _Time");
        assert!(items_unity.iter().any(|i| i["label"] == "_SinTime"), "Unity completion MUST include _SinTime");
        assert!(items_unity.iter().any(|i| i["label"] == "unity_ObjectToWorld"), "Unity completion MUST include unity_ObjectToWorld");
        assert!(items_unity.iter().any(|i| i["label"] == "TransformObjectToHClip"), "Unity completion MUST include TransformObjectToHClip");

        // 2. Pure HLSL context: _Time, unity_ObjectToWorld MUST NOT be present
        let pure_code = "float4 main() : SV_Target\n{\n    \n    return float4(0, 0, 0, 0);\n}";
        cache.insert("file:///C:/PureHLSL/shader.hlsl".to_string(), pure_code.to_string());
        let req_pure = json!({
            "params": {
                "textDocument": { "uri": "file:///C:/PureHLSL/shader.hlsl" },
                "position": { "line": 2, "character": 4 }
            }
        });
        let res_pure = handle_completion(&req_pure, &cache);
        let items_pure = res_pure.as_array().expect("Must be array");
        assert!(!items_pure.iter().any(|i| i["label"] == "_Time"), "Pure HLSL completion MUST NOT include _Time");
        assert!(!items_pure.iter().any(|i| i["label"] == "unity_ObjectToWorld"), "Pure HLSL completion MUST NOT include unity_ObjectToWorld");
        assert!(!items_pure.iter().any(|i| i["label"] == "ResolvedView.WorldCameraOrigin"), "Pure HLSL completion MUST NOT include ResolvedView");
    }

    #[test]
    fn test_engine_variable_hover() {
        let mut cache = HashMap::new();
        let code = "float4 t = _Time;\nfloat4x4 m = unity_ObjectToWorld;\n";
        cache.insert("file:///test.hlsl".to_string(), code.to_string());

        let hover_time = signature::get_hover_info("file:///test.hlsl", code, 0, 13, &cache);
        let hover_time_str = hover_time["contents"]["value"].as_str().unwrap_or("");
        assert!(hover_time_str.contains("_Time"), "Hover should describe _Time");
        assert!(hover_time_str.contains("t / 20"), "Hover should explain _Time components");

        let hover_mat = signature::get_hover_info("file:///test.hlsl", code, 1, 16, &cache);
        let hover_mat_str = hover_mat["contents"]["value"].as_str().unwrap_or("");
        assert!(hover_mat_str.contains("unity_ObjectToWorld"), "Hover should describe unity_ObjectToWorld");
    }

    #[test]
    fn test_utf8_char_boundary_no_panic() {
        let mut cache = HashMap::new();
        // Line with Turkish characters and emojis
        let code = "// Türkçe açıklama: değişkenler 🚀 ve fonksiyonlar\nfloat4 renk = float4(1, 0, 0, 1);\n";
        cache.insert("file:///test_utf8.hlsl".to_string(), code.to_string());

        // Test hover across various column offsets on the UTF-8 line
        for col in 0..code.lines().next().unwrap().len() {
            let h = signature::get_hover_info("file:///test_utf8.hlsl", code, 0, col, &cache);
            let _ = h;
        }

        // Test definition across various column offsets
        for col in 0..code.lines().next().unwrap().len() {
            let def_req = json!({
                "params": {
                    "textDocument": { "uri": "file:///test_utf8.hlsl" },
                    "position": { "line": 0, "character": col }
                }
            });
            let d = signature::handle_definition(&def_req, &cache);
            let _ = d;
        }

        // Test completion across column offsets
        for col in 0..10 {
            let req = json!({
                "params": {
                    "textDocument": { "uri": "file:///test_utf8.hlsl" },
                    "position": { "line": 0, "character": col }
                }
            });
            let _ = handle_completion(&req, &cache);
        }
    }

    #[test]
    fn test_forward_declaration_struct_scanning() {
        let code = r#"
struct ForwardType;
struct TargetStruct {
    float3 pos;
    float2 uv;
};
"#;
        let structs = signature::scan_struct_definitions(code);
        assert!(structs.iter().any(|s| s.name == "TargetStruct"), "TargetStruct must be found even with preceding forward declaration");
        assert!(!structs.iter().any(|s| s.name == "ForwardType"), "Forward declaration must not be recorded as a complete struct");
    }

    #[test]
    fn test_local_variables_not_in_global_symbols() {
        let code = r#"
float4 g_GlobalColor;

float4 FragmentFunction(float4 inColor)
{
    float4 localColor = inColor * 2.0;
    return localColor;
}
"#;
        let vars = signature::scan_user_variables(code, None, None);
        assert!(vars.iter().any(|v| v.name == "g_GlobalColor"), "g_GlobalColor must be in global variables");
        assert!(!vars.iter().any(|v| v.name == "localColor"), "localColor must NOT be in global variables");
    }

    #[test]
    fn test_register_and_packoffset_variables() {
        let code = r#"
Texture2D _MainTex : register(t0);
SamplerState _Sampler : register(s0);
float4 _Color : packoffset(c0);
"#;
        let vars = signature::scan_user_variables(code, None, None);
        assert!(vars.iter().any(|v| v.name == "_MainTex"), "_MainTex with register binding must be detected");
        assert!(vars.iter().any(|v| v.name == "_Sampler"), "_Sampler with register binding must be detected");
        assert!(vars.iter().any(|v| v.name == "_Color"), "_Color with packoffset binding must be detected");
    }

    #[test]
    fn test_multi_attribute_and_comma_in_shaderlab_properties() {
        let code = r#"
Shader "Test/MultiAttr"
{
    Properties
    {
        [HideInInspector] [HDR] _Color ("Main, Secondary (RGB)", Color) = (1, 1, 1, 1)
        _Smoothness ("Smoothness (0 to 1)", Range(0, 1)) = 0.5
    }
    SubShader { Pass {} }
}
"#;
        let props = signature::scan_shaderlab_properties(code);
        assert_eq!(props.len(), 2);
        assert_eq!(props[0].name, "_Color");
        assert_eq!(props[0].display_name, "Main, Secondary (RGB)");
        assert_eq!(props[0].prop_type, "Color");
        assert_eq!(props[1].name, "_Smoothness");
        assert_eq!(props[1].prop_type, "Range(0, 1)");

        // Also test validation does not emit false syntax error
        let diags = validate_shaderlab_properties_and_cbuffer(code);
        assert!(!diags.iter().any(|d| d.message.contains("Invalid property name")), "Multi-attributes should not trigger invalid name error");
    }

    #[test]
    fn test_dxr_raytracing_docs() {
        assert!(docs::find_builtin_function("TraceRay").is_some(), "TraceRay must be in docs");
        assert!(docs::find_builtin_function("WorldRayOrigin").is_some(), "WorldRayOrigin must be in docs");
        assert!(docs::find_builtin_function("RayTCurrent").is_some(), "RayTCurrent must be in docs");
    }

    #[test]
    fn test_shaderlab_pass_render_states() {
        let code = "Shader \"Test/RenderStates\"\n{\n    SubShader\n    {\n        Z\n    }\n}";
        let mut doc_cache = HashMap::new();
        let uri = "file:///test.shader";
        doc_cache.insert(uri.to_string(), code.to_string());

        // Test typing "Z" in SubShader (line 4, character 9)
        let msg_z = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 4, "character": 9 }
            }
        });
        let completions_z = handle_completion(&msg_z, &doc_cache);
        let items_z = completions_z.as_array().expect("Should return completion array");
        assert!(items_z.iter().any(|item| item["label"].as_str().unwrap().starts_with("ZWrite")), "Should suggest ZWrite on Z");
        assert!(items_z.iter().any(|item| item["label"].as_str().unwrap().starts_with("ZTest")), "Should suggest ZTest on Z");
        assert!(items_z.iter().any(|item| item["label"].as_str().unwrap().starts_with("ZClip")), "Should suggest ZClip on Z");

        // Test typing "ZCull" alias
        let code_zcull = "Shader \"Test\" { SubShader { ZCull } }";
        doc_cache.insert(uri.to_string(), code_zcull.to_string());
        let msg_zcull = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 0, "character": 33 }
            }
        });
        let completions_zcull = handle_completion(&msg_zcull, &doc_cache);
        let items_zcull = completions_zcull.as_array().expect("Should return completion array");
        assert!(items_zcull.iter().any(|item| item["label"].as_str().unwrap().contains("Cull")), "ZCull should suggest Cull");

        // Test typing "BlendOp "
        let code_blendop = "Shader \"Test\" { SubShader { BlendOp  } }";
        doc_cache.insert(uri.to_string(), code_blendop.to_string());
        let msg_blendop = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 0, "character": 36 }
            }
        });
        let completions_blendop = handle_completion(&msg_blendop, &doc_cache);
        let items_blendop = completions_blendop.as_array().expect("Should return completion array");
        assert!(items_blendop.iter().any(|item| item["label"] == "BlendOp Add"), "Should suggest BlendOp Add");
        assert!(items_blendop.iter().any(|item| item["label"] == "BlendOp Min"), "Should suggest BlendOp Min");

        // Test typing "ZClip "
        let code_zclip = "Shader \"Test\" { SubShader { ZClip  } }";
        doc_cache.insert(uri.to_string(), code_zclip.to_string());
        let msg_zclip = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 0, "character": 34 }
            }
        });
        let completions_zclip = handle_completion(&msg_zclip, &doc_cache);
        let items_zclip = completions_zclip.as_array().expect("Should return completion array");
        assert!(items_zclip.iter().any(|item| item["label"] == "ZClip False"), "Should suggest ZClip False");
    }

    #[test]
    fn test_shaderlab_user_image_diagnostics() {
        let code = r#"Shader "Custom/Test"
{
    Properties
    {
        [MainColor] _BaseColor("Base Color", Color) = (1, 1, 1, 1)
        [MainTexture] _BaseMap("Base Map", 2D) = "white" {}
        _Freq ("freq", Float) = 
        _Amp ("Amp", Float) = 0.0
        _Speed ("Speed", Float) = 0.0
    }
    SubShader
    {
        Tags { "RenderType" = "Opaque" "RenderPipeline" = "UniversalPipeline" }
    }
}"#;
        let diags = validate_shaderlab_properties_and_cbuffer(code);
        let freq_err = diags.iter().find(|d| d.message.contains("Missing default value after '=' for Float property '_Freq'"));
        assert!(freq_err.is_some(), "Must report missing default value for _Freq: {:?}", diags);
    }

    #[test]
    fn test_shaderlab_malformed_vector_and_attribute_diagnostic() {
        let code = r#"Shader "Custom/Test2"
{
    Properties
    {
        [MainColor] _BaseColor("Base Color", Color) = (1, 1, 1,)
        [UnclosedBracket _BadProp("Bad", Float) = 1.0
    }
    SubShader { Pass {} }
}"#;
        let diags = validate_shaderlab_properties_and_cbuffer(code);
        assert!(diags.iter().any(|d| d.message.contains("Trailing comma or empty component")), "Must detect trailing comma in (1, 1, 1,): {:?}", diags);
        assert!(diags.iter().any(|d| d.message.contains("Unclosed attribute bracket")), "Must detect unclosed '[': {:?}", diags);
    }

    #[test]
    fn test_shaderlab_tag_validation_and_completion() {
        let code_typo = r#"Shader "Test" { SubShader { Tags { "RenderType" = "Opque" } Pass {} } }"#;
        let tag_diags = validate_shaderlab_tags(code_typo);
        assert!(tag_diags.iter().any(|d| d.message.contains("Unknown RenderType 'Opque'. Did you mean 'Opaque'?")), "Must warn on Opque typo: {:?}", tag_diags);

        // Test completion inside Tags after "RenderType" = 
        let mut doc_cache = HashMap::new();
        let uri = "file:///test.shader";
        let doc = "Shader \"Test\" { SubShader { Tags { \"RenderType\" =  } } }";
        doc_cache.insert(uri.to_string(), doc.to_string());
        let msg = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 0, "character": 50 }
            }
        });
        let res = handle_completion(&msg, &doc_cache);
        let items = res.as_array().expect("Should return completions");
        assert!(items.iter().any(|item| item["label"] == "\"Opaque\""), "Should suggest \"Opaque\" for RenderType");
        assert!(items.iter().any(|item| item["label"] == "\"Transparent\""), "Should suggest \"Transparent\" for RenderType");
    }

    #[test]
    fn test_shaderlab_property_default_value_and_attribute_completion() {
        let mut doc_cache = HashMap::new();
        let uri = "file:///test.shader";

        // 1. Completion after "=" for Color
        let doc_color = "Shader \"T\" {\n    Properties {\n        _BaseColor (\"Color\", Color) = \n    }\n}";
        doc_cache.insert(uri.to_string(), doc_color.to_string());
        let msg_color = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 2, "character": 39 }
            }
        });
        let res_color = handle_completion(&msg_color, &doc_cache);
        let items_color = res_color.as_array().expect("Should return color completions");
        assert!(items_color.iter().any(|item| item["label"] == "(1, 1, 1, 1)"), "Should suggest (1, 1, 1, 1) for Color");

        // 2. Completion after "=" for 2D Texture
        let doc_tex = "Shader \"T\" {\n    Properties {\n        _BaseMap (\"Map\", 2D) = \n    }\n}";
        doc_cache.insert(uri.to_string(), doc_tex.to_string());
        let msg_tex = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 2, "character": 32 }
            }
        });
        let res_tex = handle_completion(&msg_tex, &doc_cache);
        let items_tex = res_tex.as_array().expect("Should return texture completions");
        assert!(items_tex.iter().any(|item| item["label"] == "\"white\" {}"), "Should suggest \"white\" {{}} for 2D");
        assert!(items_tex.iter().any(|item| item["label"] == "\"bump\" {}"), "Should suggest \"bump\" {{}} for 2D");

        // 3. Completion for attributes typing "["
        let doc_attr = "Shader \"T\" {\n    Properties {\n        [\n    }\n}";
        doc_cache.insert(uri.to_string(), doc_attr.to_string());
        let msg_attr = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 2, "character": 9 }
            }
        });
        let res_attr = handle_completion(&msg_attr, &doc_cache);
        let items_attr = res_attr.as_array().expect("Should return attribute completions");
        assert!(items_attr.iter().any(|item| item["label"] == "[MainColor]"), "Should suggest [MainColor]");
        assert!(items_attr.iter().any(|item| item["label"] == "[MainTexture]"), "Should suggest [MainTexture]");
        assert!(items_attr.iter().any(|item| item["label"] == "[HDR]"), "Should suggest [HDR]");
    }

    #[test]
    fn test_shaderlab_extended_properties_validation_and_completion() {
        let code = r#"Shader "Test/Extended"
{
    Properties
    {
        [Normal] _Color ("Color", Color) = (1, 1, 1, 1)
        _Dup ("Dup 1", Float) = 0.0
        _Dup ("Dup 2", Float) = 1.0
        _InvalidType ("Bad", Texture) = "white" {}
        _InvertedRange ("Range", Range(2.0, 0.0)) = 1.0
        _OutOfRange ("Range", Range(0.0, 1.0)) = 5.0
        _ScalarWithVector ("Float", Float) = (1, 1, 1, 1)
        _Semi ("WithSemi", Float) = 0.0;
    }
    SubShader { Pass {} }
}"#;
        let diags = validate_shaderlab_properties_and_cbuffer(code);

        // 1. Incompatible attribute: Normal on Color
        assert!(diags.iter().any(|d| d.message.contains("[Normal] attribute can only be applied to 2D texture")),
            "Must detect [Normal] on Color: {:?}", diags);

        // 2. Duplicate property name
        assert!(diags.iter().any(|d| d.message.contains("Duplicate property name '_Dup'")),
            "Must detect duplicate property: {:?}", diags);

        // 3. Unknown property type with suggestion
        assert!(diags.iter().any(|d| d.message.contains("Unknown property type 'Texture'") && d.message.contains("Did you mean '2D'?")),
            "Must detect unknown type Texture with suggestion: {:?}", diags);

        // 4. Inverted Range bounds
        assert!(diags.iter().any(|d| d.message.contains("Invalid Range limits") && d.message.contains("min (2) is greater than max (0)")),
            "Must detect inverted Range bounds: {:?}", diags);

        // 5. Default value out of Range
        assert!(diags.iter().any(|d| d.message.contains("Default value '5.0' for Range property '_OutOfRange' is outside the range [0, 1]")),
            "Must detect out-of-range value: {:?}", diags);

        // 6. Type mismatch: Float taking vector literal
        assert!(diags.iter().any(|d| d.message.contains("Type mismatch: scalar property '_ScalarWithVector' of type Float cannot take vector default value")),
            "Must detect scalar with vector default: {:?}", diags);

        // 7. Trailing semicolon warning
        assert!(diags.iter().any(|d| d.message.contains("ShaderLab property declarations do not use trailing semicolons ';'. Remove ';'")),
            "Must warn on trailing semicolon: {:?}", diags);

        // Test completion after comma: `_Test ("Display Name", `
        let mut doc_cache = HashMap::new();
        let uri = "file:///test_comma.shader";
        let doc_comma = "Shader \"T\" {\n    Properties {\n        _Test (\"Display Name\", \n    }\n}";
        doc_cache.insert(uri.to_string(), doc_comma.to_string());
        let msg_comma = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 2, "character": 32 }
            }
        });
        let res_comma = handle_completion(&msg_comma, &doc_cache);
        let items_comma = res_comma.as_array().expect("Must return items after comma");
        assert!(items_comma.iter().any(|i| i["label"] == "Color"), "Must suggest Color after comma");
        assert!(items_comma.iter().any(|i| i["label"] == "2D"), "Must suggest 2D after comma");
        assert!(items_comma.iter().any(|i| i["label"] == "Range"), "Must suggest Range after comma");
        assert!(items_comma.iter().any(|i| i["label"] == "Float"), "Must suggest Float after comma");

        // Test keyword snippet completion inside Properties:
        let doc_empty = "Shader \"T\" {\n    Properties {\n        \n    }\n}";
        doc_cache.insert(uri.to_string(), doc_empty.to_string());
        let msg_empty = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 2, "character": 8 }
            }
        });
        let res_empty = handle_completion(&msg_empty, &doc_cache);
        let items_empty = res_empty.as_array().expect("Must return items on empty line");
        assert!(items_empty.iter().any(|i| i["label"] == "float"), "Must suggest 'float' snippet");
        assert!(items_empty.iter().any(|i| i["label"] == "range"), "Must suggest 'range' snippet");
        assert!(items_empty.iter().any(|i| i["label"] == "color"), "Must suggest 'color' snippet");
        assert!(items_empty.iter().any(|i| i["label"] == "normal"), "Must suggest 'normal' snippet");
        assert!(items_empty.iter().any(|i| i["label"] == "toggle"), "Must suggest 'toggle' snippet");
    }

    #[test]
    fn test_shaderlab_render_states_and_generalized_tags() {
        let bad_shader = r#"
Shader "Custom/BadStates" {
    Properties {
        _Color ("Color", Color) = (1, 1, 1, 1)
    }
    SubShader {
        Tags { "RenderType" = "opaque" "RenderPipeline" = "Universal" "IgnoreProjector" = "true" }
        Cull Backface
        ZWrite True
        ZTest Alwayss
        ColorMask XYZ
        Pass {
            HLSLPROGRAM
            float4 frag() : SV_Target { return 0; }
            ENDHLSL
        }
    }
}
"#;
        let diags = validate_shader("file:///bad.shader", bad_shader, "dxc", None);

        // 1. Tag casing warning: "opaque" -> "Opaque"
        assert!(diags.iter().any(|d| d.message.contains("Unknown RenderType 'opaque'") && d.message.contains("Opaque")),
            "Must suggest Opaque: {:?}", diags);

        // 2. Tag pipeline suggestion: "Universal" -> "UniversalPipeline"
        assert!(diags.iter().any(|d| d.message.contains("UniversalPipeline")),
            "Must suggest UniversalPipeline: {:?}", diags);

        // 3. Tag boolean capitalization: "true" -> "True"
        assert!(diags.iter().any(|d| d.message.contains("Unknown IgnoreProjector 'true'") && d.message.contains("Did you mean 'True'?")),
            "Must warn on lowercase true: {:?}", diags);

        // 4. Render state Cull
        assert!(diags.iter().any(|d| d.message.contains("Unknown Cull mode 'Backface'")),
            "Must catch invalid Cull mode: {:?}", diags);

        // 5. Render state ZWrite
        assert!(diags.iter().any(|d| d.message.contains("Invalid ZWrite value 'True'")),
            "Must catch ZWrite True: {:?}", diags);

        // 6. Render state ZTest
        assert!(diags.iter().any(|d| d.message.contains("Unknown ZTest comparison mode 'Alwayss'")),
            "Must catch ZTest Alwayss: {:?}", diags);

        // 7. Render state ColorMask
        assert!(diags.iter().any(|d| d.message.contains("Invalid ColorMask 'XYZ'")),
            "Must catch invalid ColorMask: {:?}", diags);
    }

    #[test]
    fn test_compute_shader_attributes_and_multiline_params() {
        let code = r#"
[numthreads(64, 1, 1)]
void CSMain(
    uint3 id : SV_DispatchThreadID,
    uint groupIndex : SV_GroupIndex
) {
    // Body
}
"#;
        let funcs = signature::scan_user_functions(code, None, None);
        assert_eq!(funcs.len(), 1, "Must parse function with [numthreads] attribute");
        assert_eq!(funcs[0].name, "CSMain");
        assert_eq!(funcs[0].parsed_params.len(), 2);
        assert_eq!(funcs[0].parsed_params[0].name, "id");
        assert_eq!(funcs[0].parsed_params[0].line, 3);
        assert_eq!(funcs[0].parsed_params[1].name, "groupIndex");
        assert_eq!(funcs[0].parsed_params[1].line, 4);
    }

    #[test]
    fn test_dot_access_fallback_and_ternary_prevention() {
        let mut doc_cache = HashMap::new();
        let uri = "file:///test_dot.hlsl";
        let doc = r#"
struct SurfaceData {
    float4 albedo;
    float3 normal;
};

float4 TestFunc(float x) {
    SurfaceData surf;
    unknownVar.
    float val = x > 0.0 ? 1.0 : 
    return surf.albedo;
}
"#;
        doc_cache.insert(uri.to_string(), doc.to_string());

        // 1. unknownVar. completion should return fallback swizzles and known struct fields
        let msg_dot = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 8, "character": 15 } // line 8: after unknownVar.
            }
        });
        let res_dot = handle_completion(&msg_dot, &doc_cache);
        let items_dot = res_dot.as_array().expect("Must return fallback items on unknown dot access");
        assert!(items_dot.iter().any(|i| i["label"] == "x"), "Must suggest swizzle x");
        assert!(items_dot.iter().any(|i| i["label"] == "albedo"), "Must suggest known struct field 'albedo'");
        assert!(items_dot.iter().any(|i| i["label"] == "normal"), "Must suggest known struct field 'normal'");
        assert!(!items_dot.iter().any(|i| i["label"] == "cbuffer"), "Must NOT suggest cbuffer on dot access");

        // 2. Ternary operator branch 'val = x > 0.0 ? 1.0 : ' should NOT suggest SV_Target
        let msg_ternary = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 9, "character": 32 } // line 9: after ':'
            }
        });
        let res_ternary = handle_completion(&msg_ternary, &doc_cache);
        let items_ternary = res_ternary.as_array().expect("Must return completions");
        assert!(!items_ternary.iter().any(|i| i["label"] == "SV_Target"), "Ternary ':' must NOT trigger SV_Target");
    }

    #[test]
    fn test_hover_local_precedence_over_intrinsics() {
        let mut doc_cache = HashMap::new();
        let uri = "file:///test_hover.hlsl";
        let doc = r#"
float4 Calculate(float distance, float saturate) {
    return float4(distance, saturate, 0, 1);
}
"#;
        doc_cache.insert(uri.to_string(), doc.to_string());

        // Hover on parameter 'distance' (line 1, col 25) or inside body (line 2, col 19)
        let hover_res = signature::get_hover_info(uri, doc, 2, 19, &doc_cache);
        let val_str = hover_res["contents"]["value"].as_str().unwrap_or("");
        assert!(val_str.contains("parameter of `Calculate`"), "Hover must show parameter doc: {}", val_str);
        assert!(!val_str.contains("Microsoft HLSL Intrinsic"), "Hover must NOT show intrinsic doc for local parameter");
    }

    #[test]
    fn test_user_signature_help() {
        let mut doc_cache = HashMap::new();
        let uri = "file:///test_sig.shader";
        let doc = r#"Shader "Custom/Test"
{
    SubShader
    {
        Pass
        {
            HLSLPROGRAM
            int test(int a){
                return a;
            }

            void frag() {
                int b = test()
            }
            ENDHLSL
        }
    }
}
"#;
        doc_cache.insert(uri.to_string(), doc.to_string());

        let line_12 = doc.lines().nth(12).unwrap();
        let open_col = line_12.find('(').unwrap() + 1; // between ( and )
        let after_col = line_12.find(')').unwrap() + 1; // after )
        
        let sig_inside = signature::get_signature_help(uri, doc, 12, open_col, &doc_cache);
        let sig_after = signature::get_signature_help(uri, doc, 12, after_col, &doc_cache);
        assert!(!sig_inside.is_null(), "Signature help must work inside ()!");
        assert!(!sig_after.is_null(), "Signature help must work right after ()!");

        assert_eq!(sig_inside["signatures"][0]["label"], "int test(int a)");
        assert_eq!(sig_after["signatures"][0]["label"], "int test(int a)");
    }

    #[test]
    fn test_completion_items_have_parameter_hints_command() {
        let mut doc_cache = HashMap::new();
        let uri = "file:///test_cmd.hlsl";
        let doc = r#"
int my_custom_func(int x, int y) { return x + y; }
void main() {
    
}
"#;
        doc_cache.insert(uri.to_string(), doc.to_string());

        let msg = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 3, "character": 4 }
            }
        });
        let res = handle_completion(&msg, &doc_cache);
        let items = res.as_array().expect("Completion items array");

        // 1. User function item has command
        let user_fn = items.iter().find(|i| i["label"] == "my_custom_func").expect("my_custom_func must be present");
        assert_eq!(user_fn["command"]["command"], "editor.action.triggerParameterHints");

        // 2. Builtin function item has command
        let clamp_fn = items.iter().find(|i| i["label"] == "clamp").expect("clamp must be present");
        assert_eq!(clamp_fn["command"]["command"], "editor.action.triggerParameterHints");
    }

    #[test]
    fn test_included_signature_help_and_hybrid_collision() {
        let mut doc_cache = HashMap::new();
        let main_uri = "file:///workspace/Shaders/Main.shader";
        let inc_uri = "file:///workspace/Shaders/UnityCG.cginc";

        let inc_doc = r#"
// Transforms 2D UV by texture scale and offset
float2 TRANSFORM_TEX(float2 uv, float4 st) {
    return uv * st.xy + st.zw;
}

float CustomHelper(float a, float b, float c) {
    return (a + b) * c;
}
"#;
        let main_doc = r#"Shader "Custom/Main"
{
    SubShader
    {
        Pass
        {
            HLSLPROGRAM
            #include "UnityCG.cginc"

            void frag() {
                float2 uv = TRANSFORM_TEX(
                float val = CustomHelper(
            }
            ENDHLSL
        }
    }
}
"#;
        doc_cache.insert(inc_uri.to_string(), inc_doc.to_string());
        doc_cache.insert(main_uri.to_string(), main_doc.to_string());

        // 1. Signature help for CustomHelper (line 11, col 41)
        let sig_custom = signature::get_signature_help(main_uri, main_doc, 11, 41, &doc_cache);
        assert!(!sig_custom.is_null(), "CustomHelper signature help must resolve from include!");
        let label_custom = sig_custom["signatures"][0]["label"].as_str().unwrap_or("");
        assert!(label_custom.contains("CustomHelper(float a, float b, float c)"));
        let doc_custom = sig_custom["signatures"][0]["documentation"]["value"].as_str().unwrap_or("");
        assert!(doc_custom.contains("*(Defined in `UnityCG.cginc`)*"), "Must show source provenance");

        // 2. Signature help for TRANSFORM_TEX (collision with docs.rs):
        // Must use header's actual signature AND hybrid enriched markdown doc!
        let sig_trans = signature::get_signature_help(main_uri, main_doc, 10, 42, &doc_cache);
        assert!(!sig_trans.is_null(), "TRANSFORM_TEX signature help must resolve from include!");
        let label_trans = sig_trans["signatures"][0]["label"].as_str().unwrap_or("");
        assert!(label_trans.contains("TRANSFORM_TEX(float2 uv, float4 st)"));
        let doc_trans = sig_trans["signatures"][0]["documentation"]["value"].as_str().unwrap_or("");
        assert!(doc_trans.contains("*(Defined in `UnityCG.cginc`)*"), "Must show source provenance");
        assert!(doc_trans.contains("Transforms 2D UV"), "Must show enriched documentation");
    }
}


