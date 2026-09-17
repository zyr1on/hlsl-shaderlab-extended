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
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

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

    // 3. Unity HLSL / CG / URP / HDRP / 2D / Compute: includes, macros, or built-in variables
    let in_unity_dir = uri.ends_with(".cginc")
        || uri.ends_with(".compute")
        || uri.contains("com.unity.render-pipelines")
        || uri.contains("/Assets/")
        || uri.contains("\\Assets\\")
        || uri.contains("/Packages/")
        || uri.contains("\\Packages\\");

    if in_unity_dir
        || content.contains("HLSLPROGRAM")
        || content.contains("CGPROGRAM")
        || content.contains("#pragma kernel")
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
        if idx > target_line {
            break;
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
            if idx > target_line {
                break;
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
        if idx > target_line {
            break;
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

pub fn normalize_uri(uri: &str) -> String {
    if let Some(p) = uri_to_path(uri) {
        #[cfg(windows)]
        {
            format!("file:///{}", p.to_string_lossy().replace('\\', "/").to_lowercase())
        }
        #[cfg(not(windows))]
        {
            format!("file://{}", p.to_string_lossy())
        }
    } else {
        uri.to_lowercase()
    }
}

#[allow(dead_code)]
#[cfg(target_arch = "aarch64")]
const TARGET_DXC_ARCH: &str = "arm64";
#[allow(dead_code)]
#[cfg(target_arch = "x86")]
const TARGET_DXC_ARCH: &str = "x86";
#[allow(dead_code)]
#[cfg(not(any(target_arch = "aarch64", target_arch = "x86")))]
const TARGET_DXC_ARCH: &str = "x64";

#[cfg(windows)]
const DXC_BINARY_NAME: &str = "dxc.exe";
#[cfg(not(windows))]
const DXC_BINARY_NAME: &str = "dxc";

/// Returns platform-specific Zed extension work directories (Windows, Linux/Ubuntu, macOS).
fn get_zed_extension_work_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(windows)]
    if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
        dirs.push(
            PathBuf::from(local_app_data)
                .join("Zed")
                .join("extensions")
                .join("work")
                .join("hlsl-shaderlab-extended"),
        );
    }

    #[cfg(target_os = "macos")]
    if let Ok(home) = env::var("HOME") {
        dirs.push(
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Zed")
                .join("extensions")
                .join("work")
                .join("hlsl-shaderlab-extended"),
        );
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // 1. $XDG_DATA_HOME/zed/extensions/work/hlsl-shaderlab-extended
        if let Ok(xdg) = env::var("XDG_DATA_HOME") {
            dirs.push(
                PathBuf::from(xdg)
                    .join("zed")
                    .join("extensions")
                    .join("work")
                    .join("hlsl-shaderlab-extended"),
            );
        }
        if let Ok(home) = env::var("HOME") {
            // 2. Standard Ubuntu / Linux XDG fallback: ~/.local/share/zed/extensions/work/hlsl-shaderlab-extended
            dirs.push(
                PathBuf::from(&home)
                    .join(".local")
                    .join("share")
                    .join("zed")
                    .join("extensions")
                    .join("work")
                    .join("hlsl-shaderlab-extended"),
            );
            // 3. Flatpak Zed: ~/.var/app/dev.zed.Zed/data/zed/extensions/work/hlsl-shaderlab-extended
            dirs.push(
                PathBuf::from(&home)
                    .join(".var")
                    .join("app")
                    .join("dev.zed.Zed")
                    .join("data")
                    .join("zed")
                    .join("extensions")
                    .join("work")
                    .join("hlsl-shaderlab-extended"),
            );
        }
    }

    dirs
}

/// Attempts to resolve a given candidate path (absolute or relative) to an existing DXC executable.
pub fn try_resolve_dxc(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let p = Path::new(trimmed);
    if p.is_file() {
        return Some(p.to_string_lossy().to_string());
    }

    // If relative, resolve against current_exe parent / grandparent (Zed extension work dir)
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(p);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().to_string());
            }
            if let Some(grandparent) = parent.parent() {
                let candidate = grandparent.join(p);
                if candidate.is_file() {
                    return Some(candidate.to_string_lossy().to_string());
                }
            }
        }
    }

    // Also check relative to current working directory
    if let Ok(cwd) = env::current_dir() {
        let candidate = cwd.join(p);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }

    // Check relative to Zed extension work directories (Windows, Linux/Ubuntu, macOS)
    for zed_work in get_zed_extension_work_dirs() {
        let candidate = zed_work.join(p);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }

    None
}

/// Scans a directory for any dxc* folder and looks for the appropriate architecture binary.
fn scan_dir_for_dxc(dir: &Path) -> Option<String> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.to_ascii_lowercase().starts_with("dxc") {
                    #[cfg(windows)]
                    {
                        let primary = path.join("bin").join(TARGET_DXC_ARCH).join(DXC_BINARY_NAME);
                        if primary.is_file() {
                            return Some(primary.to_string_lossy().to_string());
                        }
                        if TARGET_DXC_ARCH == "arm64" {
                            let x64_fb = path.join("bin").join("x64").join(DXC_BINARY_NAME);
                            if x64_fb.is_file() {
                                return Some(x64_fb.to_string_lossy().to_string());
                            }
                        }
                        let bin_direct = path.join("bin").join(DXC_BINARY_NAME);
                        if bin_direct.is_file() {
                            return Some(bin_direct.to_string_lossy().to_string());
                        }
                        let root_direct = path.join(DXC_BINARY_NAME);
                        if root_direct.is_file() {
                            return Some(root_direct.to_string_lossy().to_string());
                        }
                    }
                    #[cfg(not(windows))]
                    {
                        let arch_direct = path.join("bin").join(TARGET_DXC_ARCH).join(DXC_BINARY_NAME);
                        if arch_direct.is_file() {
                            return Some(arch_direct.to_string_lossy().to_string());
                        }
                        let bin_direct = path.join("bin").join(DXC_BINARY_NAME);
                        if bin_direct.is_file() {
                            return Some(bin_direct.to_string_lossy().to_string());
                        }
                        let root_direct = path.join(DXC_BINARY_NAME);
                        if root_direct.is_file() {
                            return Some(root_direct.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Locates the DXC binary with the following priority:
/// 1. `DXC_PATH` environment variable (if valid)
/// 2. System `PATH`
/// 3. Neighbor directories of `current_exe()` (Zed extension work directory)
/// 4. `%LOCALAPPDATA%\Zed\extensions\work\hlsl-shaderlab-extended`
/// 5. Windows 10/11 Kits (DirectXShaderCompiler from Windows SDK)
/// 6. `%VULKAN_SDK%\Bin\dxc.exe`
/// 7. Fallback to `"dxc.exe"` / `"dxc"`
pub fn find_dxc_path() -> String {
    // 1. Explicit DXC_PATH env var
    if let Ok(p) = env::var("DXC_PATH") {
        if let Some(resolved) = try_resolve_dxc(&p) {
            return resolved;
        }
    }

    // 2. System PATH check
    if let Ok(path_var) = env::var("PATH") {
        for dir in env::split_paths(&path_var) {
            let candidate = dir.join(DXC_BINARY_NAME);
            if candidate.is_file() {
                return candidate.to_string_lossy().to_string();
            }
        }
    }

    // 3. Scan neighbor directories of current_exe() (e.g. extension work dir where dxc-* is unpacked)
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            if let Some(found) = scan_dir_for_dxc(parent) {
                return found;
            }
            if let Some(grandparent) = parent.parent() {
                if let Some(found) = scan_dir_for_dxc(grandparent) {
                    return found;
                }
            }
        }
    }

    // 4. Check Zed extension work directories (Windows, Linux/Ubuntu, macOS)
    for zed_work in get_zed_extension_work_dirs() {
        if zed_work.is_dir() {
            if let Some(found) = scan_dir_for_dxc(&zed_work) {
                return found;
            }
        }
    }

    // 5. Check Windows Kits (Windows 10/11 SDK)
    #[cfg(windows)]
    {
        let kits_bin = Path::new(r"C:\Program Files (x86)\Windows Kits\10\bin");
        if kits_bin.is_dir() {
            if let Ok(entries) = fs::read_dir(kits_bin) {
                let mut versions: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.is_dir())
                    .collect();
                versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

                for vdir in versions {
                    let arch_candidate = vdir.join(TARGET_DXC_ARCH).join(DXC_BINARY_NAME);
                    if arch_candidate.is_file() {
                        return arch_candidate.to_string_lossy().to_string();
                    }
                    let x64_candidate = vdir.join("x64").join(DXC_BINARY_NAME);
                    if x64_candidate.is_file() {
                        return x64_candidate.to_string_lossy().to_string();
                    }
                }
            }
        }
    }

    // 6. Check Vulkan SDK
    if let Ok(vulkan_sdk) = env::var("VULKAN_SDK") {
        let candidate = PathBuf::from(&vulkan_sdk).join("Bin").join(DXC_BINARY_NAME);
        if candidate.is_file() {
            return candidate.to_string_lossy().to_string();
        }
        #[cfg(not(windows))]
        {
            let candidate_unix = PathBuf::from(&vulkan_sdk).join("bin").join(DXC_BINARY_NAME);
            if candidate_unix.is_file() {
                return candidate_unix.to_string_lossy().to_string();
            }
        }
    }

    // 7. Fallback
    DXC_BINARY_NAME.to_string()
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
            // 1. Check for consecutive or empty quotes: `""`
            let mut search_from = 0;
            while let Some(pos) = raw_line[search_from..].find("\"\"") {
                let col = search_from + pos;
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx, character: col },
                        end: Position { line: line_idx, character: col + 2 },
                    },
                    severity: 1, // Error
                    message: "Syntax error: empty or consecutive quotes '\"\"' in Tags block".to_string(),
                    source: "shaderlab".to_string(),
                });
                search_from = col + 2;
            }

            // 2. Check for unclosed quotes
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

            // 3. Parse and validate tag entries on this line
            let mut line_content = raw_line;
            if let Some(brace_idx) = line_content.find('{') {
                if line_content[..brace_idx].contains("Tags") {
                    line_content = &line_content[brace_idx + 1..];
                }
            }
            if let Some(close_idx) = line_content.rfind('}') {
                line_content = &line_content[..close_idx];
            }

            if line_content.contains('=') {
                let mut after_eq = line_content;
                while let Some(eq_idx) = after_eq.find('=') {
                    let before = after_eq[..eq_idx].trim();
                    let rest = after_eq[eq_idx + 1..].trim();

                    if before.contains('=') {
                        let col = raw_line.find('=').unwrap_or(0);
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: line_idx, character: col },
                                end: Position { line: line_idx, character: col + 1 },
                            },
                            severity: 1, // Error
                            message: "Malformed tag entry: unexpected '=' in tag declaration".to_string(),
                            source: "shaderlab".to_string(),
                        });
                    }

                    if rest.is_empty() || rest.starts_with('}') {
                        let col = raw_line.rfind('=').unwrap_or(0);
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: line_idx, character: col },
                                end: Position { line: line_idx, character: col + 1 },
                            },
                            severity: 1, // Error
                            message: "Missing tag value after '=' in Tags block".to_string(),
                            source: "shaderlab".to_string(),
                        });
                        break;
                    } else if let Some(stripped) = rest.strip_prefix('"') {
                        if let Some(val_end) = stripped.find('"') {
                            let tag_val = &stripped[..val_end];
                            let key = before.rsplit('"').nth(1).unwrap_or(before);
                            let clean_key = key.trim_matches(|c: char| c == '"' || c == '{' || c == '}' || c.is_whitespace());

                            if clean_key.is_empty() {
                                let col = raw_line.find('=').unwrap_or(0);
                                diagnostics.push(Diagnostic {
                                    range: Range {
                                        start: Position { line: line_idx, character: col.saturating_sub(2) },
                                        end: Position { line: line_idx, character: col },
                                    },
                                    severity: 1, // Error
                                    message: "Tag key cannot be empty in Tags block".to_string(),
                                    source: "shaderlab".to_string(),
                                });
                            } else {
                                let is_known_key = docs::SHADERLAB_TAG_KEYS_AND_VALUES.iter().any(|(k, _, _)| k.eq_ignore_ascii_case(clean_key));
                                let key_col = raw_line.find(clean_key).unwrap_or(0);

                                if !is_known_key {
                                    let mut sug_msg = String::new();
                                    if let Some((best_key, _, _)) = docs::SHADERLAB_TAG_KEYS_AND_VALUES.iter().find(|(k, _, _)| {
                                        k.to_lowercase().starts_with(&clean_key.to_lowercase())
                                            || clean_key.to_lowercase().starts_with(&k.to_lowercase())
                                    }) {
                                        sug_msg = format!(" Did you mean '{best_key}'?");
                                    }
                                    diagnostics.push(Diagnostic {
                                        range: Range {
                                            start: Position { line: line_idx, character: key_col },
                                            end: Position { line: line_idx, character: key_col + clean_key.len() },
                                        },
                                        severity: 2, // Warning
                                        message: format!("Unknown Tag key '{clean_key}'.{sug_msg}"),
                                        source: "shaderlab".to_string(),
                                    });
                                } else {
                                    let col = raw_line.find(tag_val).unwrap_or(0);
                                    for (known_key, valid_vals, _desc) in docs::SHADERLAB_TAG_KEYS_AND_VALUES {
                                        if known_key.eq_ignore_ascii_case(clean_key) {
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
                                                        message: format!("Unknown {clean_key} '{tag_val}'. Did you mean '{sug_clean}'? (Tag values are case-sensitive)"),
                                                        source: "shaderlab".to_string(),
                                                    });
                                                } else if (clean_key == "IgnoreProjector" || clean_key == "CanUseSpriteAtlas" || clean_key == "ForceNoShadowCasting") && (tag_val == "true" || tag_val == "false") {
                                                    let sug = if tag_val == "true" { "True" } else { "False" };
                                                    diagnostics.push(Diagnostic {
                                                        range: Range {
                                                            start: Position { line: line_idx, character: col },
                                                            end: Position { line: line_idx, character: col + tag_val.len() },
                                                        },
                                                        severity: 2,
                                                        message: format!("Tag '{clean_key}' requires capitalized '{sug}' in ShaderLab."),
                                                        source: "shaderlab".to_string(),
                                                    });
                                                } else {
                                                    diagnostics.push(Diagnostic {
                                                        range: Range {
                                                            start: Position { line: line_idx, character: col },
                                                            end: Position { line: line_idx, character: col + tag_val.len() },
                                                        },
                                                        severity: 2,
                                                        message: format!("Unknown {clean_key} value '{tag_val}'."),
                                                        source: "shaderlab".to_string(),
                                                    });
                                                }
                                            }
                                            break;
                                        }
                                    }
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

///// Checks for missing relative #include files that are not found in doc_cache or on disk
pub fn validate_missing_includes(
    content: &str,
    uri: &str,
    doc_cache: &HashMap<String, String>,
    workspace_root: Option<&Path>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let current_dir = uri_to_path(uri).and_then(|p| p.parent().map(|d| d.to_path_buf()));

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with("#include") {
            continue;
        }
        let rest = trimmed.strip_prefix("#include").unwrap_or("").trim();
        if rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2 {
            let inc_path_str = &rest[1..rest.len() - 1];
            if inc_path_str.starts_with("Packages/")
                || inc_path_str.starts_with("/Engine/")
                || inc_path_str.starts_with("Engine/")
                || inc_path_str.contains("UnityCG.cginc")
                || inc_path_str.contains("Lighting.cginc")
                || inc_path_str.contains("AutoLight.cginc")
                || inc_path_str.contains("TerrainEngine.cginc")
                || inc_path_str.contains("Core.hlsl")
                || inc_path_str.contains("Common.ush")
            {
                continue;
            }

            let in_cache = doc_cache.keys().any(|k| {
                let norm = normalize_uri(k);
                if let Some(prefix) = norm.strip_suffix(inc_path_str) {
                    if prefix.ends_with('/') || prefix.ends_with('\\') {
                        return true;
                    }
                }
                k.ends_with(inc_path_str)
            });

            if in_cache {
                continue;
            }

            let file_exists = current_dir.as_ref().map(|d| d.join(inc_path_str).exists()).unwrap_or(false)
                || workspace_root.map(|ws| ws.join(inc_path_str).exists()).unwrap_or(false);

            if !file_exists {
                let col = line.find(inc_path_str).unwrap_or(0);
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx, character: col },
                        end: Position { line: line_idx, character: col + inc_path_str.len() },
                    },
                    severity: 1,
                    message: format!("Cannot find include file '{inc_path_str}'."),
                    source: "hlsl".to_string(),
                });
            }
        }
    }
    diagnostics
}

/// Validates Unity #pragma vertex, #pragma fragment, etc. entry points against user-defined and included functions.
pub fn validate_shader_pragmas(content: &str, uri: &str, doc_cache: &HashMap<String, String>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let (all_funcs, _, _) = signature::resolve_includes_and_scan_symbols(uri, content, doc_cache);

    const STAGE_PRAGMAS: &[&str] = &[
        "vertex", "fragment", "geometry", "hull", "domain", "compute", "kernel", "raytracing"
    ];

    const KNOWN_UNITY_ENTRY_POINTS: &[&str] = &[
        "LitPassVertex", "LitPassFragment",
        "DepthOnlyVertex", "DepthNormalsVertex",
        "ShadowCasterVertex", "Universal2DVertex", "Universal2DFragment",
        "UnlitPassVertex", "UnlitPassFragment",
        "MetaPassVertex", "MetaPassFragment",
        "SpriteVert", "SpriteFrag",
    ];

    let has_unresolved_packages = content.lines().any(|l| {
        let t = l.trim();
        t.starts_with("#include") && (t.contains("Packages/") || t.contains("UnityCG") || t.contains("Core.hlsl"))
    });

    for (line_idx, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        if !trimmed.starts_with("#pragma ") && !trimmed.starts_with("#pragma\t") {
            continue;
        }

        let after_pragma = trimmed.strip_prefix("#pragma").unwrap_or("").trim_start();
        let mut tokens = after_pragma.split_whitespace();
        let stage = match tokens.next() {
            Some(s) => s,
            None => continue,
        };

        if !STAGE_PRAGMAS.contains(&stage) {
            continue;
        }

        let entry_name = match tokens.next() {
            Some(e) => e,
            None => {
                let col = raw_line.find(stage).unwrap_or(0);
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: line_idx, character: col },
                        end: Position { line: line_idx, character: raw_line.len() },
                    },
                    severity: 1,
                    message: format!("Missing entry point function name for '#pragma {stage}'."),
                    source: "shaderlab".to_string(),
                });
                continue;
            }
        };

        // If found in user functions or included functions, all good!
        if all_funcs.iter().any(|f| f.name == entry_name) {
            continue;
        }

        // If it's a known built-in Unity URP/HDRP entry point and we have unresolved packages, allow it
        if has_unresolved_packages && KNOWN_UNITY_ENTRY_POINTS.contains(&entry_name) {
            continue;
        }

        // Entry point not found! Find closest matching function
        let col = raw_line.find(entry_name).unwrap_or(raw_line.find(stage).unwrap_or(0));
        let mut suggestion = String::new();

        // 1. Check if there's a function with similar name
        let similar = all_funcs.iter().find(|f| {
            f.name.eq_ignore_ascii_case(entry_name)
                || (f.name.starts_with(entry_name) && f.name.len() <= entry_name.len() + 3)
                || (entry_name.starts_with(&f.name) && entry_name.len() <= f.name.len() + 3)
        });

        if let Some(sim) = similar {
            suggestion = format!(" Did you mean '{}'?", sim.name);
        } else {
            // 2. Look for likely candidate based on stage
            let candidate = match stage {
                "vertex" => all_funcs.iter().find(|f| f.name.contains("vert") || f.name.contains("Vert") || f.label.contains("SV_POSITION") || f.label.contains("positionCS")),
                "fragment" => all_funcs.iter().find(|f| f.name.contains("frag") || f.name.contains("Frag") || f.label.contains("SV_Target")),
                _ => None,
            };
            if let Some(c) = candidate {
                suggestion = format!(" Did you mean '{}'?", c.name);
            }
        }

        diagnostics.push(Diagnostic {
            range: Range {
                start: Position { line: line_idx, character: col },
                end: Position { line: line_idx, character: col + entry_name.len() },
            },
            severity: 1,
            message: format!("Entry point function '{entry_name}' declared in '#pragma {stage} {entry_name}' was not found in this shader or included headers.{suggestion}"),
            source: "shaderlab".to_string(),
        });
    }

    diagnostics
}

pub fn validate_shader(
    uri: &str,
    content: &str,
    dxc_path: &str,
    workspace_root: Option<&Path>,
    doc_cache: &HashMap<String, String>,
) -> Vec<Diagnostic> {
    let context = detect_shader_context(uri, content);
    let mut diagnostics = Vec::new();
    let include_dirs = discover_include_paths(uri, workspace_root);

    diagnostics.extend(validate_missing_includes(content, uri, doc_cache, workspace_root));

    match context {
        ShaderContext::UnityShaderLab => {
            diagnostics.extend(validate_shaderlab_properties_and_cbuffer(content));
            diagnostics.extend(validate_shaderlab_tags(content));
            diagnostics.extend(validate_shaderlab_render_states(content));
            diagnostics.extend(validate_shader_pragmas(content, uri, doc_cache));

            // Extract shared HLSLINCLUDE / CGINCLUDE code across passes
            let mut shared_include = String::new();
            let mut in_include = false;
            let mut shared_include_start = 0;
            let mut shared_include_count = 0;
            for (line_idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed == "HLSLINCLUDE" || trimmed == "CGINCLUDE" {
                    in_include = true;
                    shared_include_start = line_idx + 1;
                    continue;
                }
                if trimmed == "ENDHLSL" || trimmed == "ENDCG" {
                    in_include = false;
                    continue;
                }
                if in_include {
                    shared_include_count += 1;
                    if should_stub_include(trimmed) {
                        shared_include.push_str("// ");
                    }
                    shared_include.push_str(line);
                    shared_include.push('\n');
                }
            }

            let mut in_block = false;
            let mut block_start_line = 0;
            let mut block_code = String::new();
            let mut stubs_count = 0;

            for (line_idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed == "HLSLPROGRAM" || trimmed == "CGPROGRAM" {
                    in_block = true;
                    block_start_line = line_idx + 1;
                    block_code.clear();
                    block_code.push_str(UNITY_COMPAT_PREAMBLE.trim_start());
                    if !block_code.ends_with('\n') {
                        block_code.push('\n');
                    }
                    stubs_count = block_code.lines().count();
                    if !shared_include.is_empty() {
                        block_code.push_str(&shared_include);
                    }
                    continue;
                }
                if trimmed == "ENDHLSL" || trimmed == "ENDCG" {
                    if in_block {
                        in_block = false;
                        let shared_info = if shared_include_count > 0 {
                            Some((shared_include_count, shared_include_start))
                        } else {
                            None
                        };

                        let block_diags = run_dxc_on_text(&block_code, dxc_path, block_start_line, &include_dirs, stubs_count, shared_info);
                        diagnostics.extend(block_diags);
                    }
                    continue;
                }

                if in_block {
                    if should_stub_include(trimmed) {
                        block_code.push_str("// ");
                    }
                    block_code.push_str(line);
                    block_code.push('\n');
                }
            }
        }
        ShaderContext::UnityHlsl => {
            diagnostics.extend(validate_shader_pragmas(content, uri, doc_cache));
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
            diagnostics.extend(run_dxc_on_text(&block_code, dxc_path, 0, &include_dirs, preamble_line_count, None));
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
            diagnostics = run_dxc_on_text(&block_code, dxc_path, 0, &include_dirs, preamble_line_count, None);
        }
        ShaderContext::PureHlsl => {
            diagnostics = run_dxc_on_text(content, dxc_path, 0, &include_dirs, 0, None);
        }
    }

    diagnostics
}

static TEMP_FILE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

struct TempFileGuard(PathBuf);
impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn run_dxc_on_text(
    content: &str,
    dxc_path: &str,
    line_offset: usize,
    include_dirs: &[PathBuf],
    stubs_line_count: usize,
    shared_include: Option<(usize, usize)>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let temp_dir = env::temp_dir();
    let count = TEMP_FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temp_file = temp_dir.join(format!("hlsl_val_{}_{}.hlsl", std::process::id(), count));

    if fs::write(&temp_file, content).is_err() {
        return diagnostics;
    }
    let _guard = TempFileGuard(temp_file.clone());

    let mut cmd = Command::new(dxc_path);
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    cmd.arg("-T").arg("lib_6_3");
    cmd.arg("-HV").arg("2021");
    cmd.arg("-O0");
    cmd.arg("-Wno-misplaced-attributes");
    cmd.arg("-Wno-unknown-pragmas");

    for inc in include_dirs {
        cmd.arg("-I").arg(inc);
    }

    cmd.arg(&temp_file);

    let mut child = match cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[hlsl_validator] Failed to spawn DXC at '{}': {e}", dxc_path);
            return diagnostics;
        }
    };

    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();

    let stdout_handle = thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut p) = stdout_pipe.take() {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    });
    let stderr_handle = thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut p) = stderr_pipe.take() {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    });

    let start = Instant::now();
    let mut exited = false;
    while start.elapsed() < Duration::from_secs(4) {
        match child.try_wait() {
            Ok(Some(_)) => {
                exited = true;
                break;
            }
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(_) => break,
        }
    }

    if !exited {
        let _ = child.kill();
        let _ = child.wait();
        return diagnostics;
    }

    let stdout_bytes = stdout_handle.join().unwrap_or_default();
    let stderr_bytes = stderr_handle.join().unwrap_or_default();
    let stderr = String::from_utf8_lossy(&stderr_bytes);
    let stdout = String::from_utf8_lossy(&stdout_bytes);

    for line in stderr.lines().chain(stdout.lines()) {
        let (severity, rest, tag) = if let Some(idx) = line.find(": fatal error:") {
            (1, line[idx + 14..].trim(), ": fatal error:")
        } else if let Some(idx) = line.find(": error:") {
            (1, line[idx + 8..].trim(), ": error:")
        } else if let Some(idx) = line.find(": warning:") {
            (2, line[idx + 10..].trim(), ": warning:")
        } else {
            continue;
        };

        if rest.contains("file not found")
            && (rest.contains("Packages/")
                || rest.contains("UnityCG.cginc")
                || rest.contains("Lighting.cginc")
                || rest.contains("AutoLight.cginc")
                || rest.contains("TerrainEngine.cginc")
                || rest.contains("Engine")
                || rest.contains(".ush"))
        {
            continue;
        }

        if rest.contains("attribute 'numthreads' ignored")
            || rest.contains("unknown pragma ignored")
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

        let prefix_part = line.split(tag).next().unwrap_or("");
        let mut parts = prefix_part.rsplitn(3, ':');
        let col_str = parts.next().unwrap_or("");
        let line_str = parts.next().unwrap_or("");
        let file_part = parts.next().unwrap_or("");

        if let (Ok(parsed_line), Ok(parsed_col)) = (line_str.parse::<usize>(), col_str.parse::<usize>()) {
            if parsed_line <= stubs_line_count {
                // Ignore internal preamble stubs errors
                continue;
            }

            let (shared_count, shared_start) = shared_include.unwrap_or((0, 0));
            let total_preamble = stubs_line_count + shared_count;

            let actual_line = if file_part.contains("hlsl_val_") {
                if shared_count > 0 && parsed_line <= total_preamble {
                    (parsed_line - 1).saturating_sub(stubs_line_count) + shared_start
                } else {
                    (parsed_line - 1).saturating_sub(total_preamble) + line_offset
                }
            } else {
                let inc_name = Path::new(file_part).file_name().and_then(|n| n.to_str()).unwrap_or("");
                content.lines().enumerate()
                    .find(|(_, l)| l.contains("#include") && !inc_name.is_empty() && l.contains(inc_name))
                    .map(|(idx, _)| idx)
                    .unwrap_or(line_offset)
            };
            let actual_col = parsed_col.saturating_sub(1);
            let span_len = if let Some(missing_file) = rest.strip_prefix('\'').and_then(|s| s.split('\'').next()) {
                missing_file.len() + 2
            } else {
                5
            };

            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: actual_line, character: actual_col },
                    end: Position { line: actual_line, character: actual_col + span_len },
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

    let mut new_text = String::with_capacity(text.len());
    let mut indent_level: usize = 0;
    let mut blank_count = 0;
    let mut line_count: usize = 0;
    let mut last_line_len = 0;

    for raw_line in text.lines() {
        line_count += 1;
        last_line_len = raw_line.len();
        let trimmed = raw_line.trim();

        if trimmed.is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                new_text.push('\n');
            }
            continue;
        }
        blank_count = 0;

        if trimmed.starts_with('#') {
            new_text.push_str(trimmed);
            new_text.push('\n');
            continue;
        }

        // Ignore lines that are comments so they don't alter indent levels
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            for _ in 0..indent_level {
                new_text.push_str(&indent_unit);
            }
            new_text.push_str(trimmed);
            new_text.push('\n');
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

        for _ in 0..indent_level {
            new_text.push_str(&indent_unit);
        }
        new_text.push_str(trimmed);
        new_text.push('\n');

        // Count braces in code only (ignoring trailing single-line comments)
        let code_part = trimmed.split("//").next().unwrap_or("").trim();
        let open_count = code_part.chars().filter(|&c| c == '{').count();
        let close_count = code_part.chars().filter(|&c| c == '}').count();
        let net_close_after = close_count.saturating_sub(leading_close);
        indent_level = indent_level.saturating_sub(net_close_after) + open_count;
    }

    if !new_text.is_empty() && !new_text.ends_with('\n') {
        new_text.push('\n');
    }
    let end_line = line_count.saturating_sub(1);

    vec![json!({
        "range": {
            "start": { "line": 0, "character": 0 },
            "end": { "line": end_line, "character": last_line_len }
        },
        "newText": new_text
    })]
}

pub fn get_document_from_cache<'a>(doc_cache: &'a HashMap<String, String>, uri: &str) -> Option<&'a str> {
    if let Some(d) = doc_cache.get(uri) {
        return Some(d.as_str());
    }
    let norm = normalize_uri(uri);
    for (k, v) in doc_cache {
        if normalize_uri(k) == norm {
            return Some(v.as_str());
        }
    }
    None
}

pub fn extract_word_prefix(prefix: &str) -> (usize, &str) {
    let mut word_start = prefix.len();
    for (i, c) in prefix.char_indices().rev() {
        if c.is_alphanumeric() || c == '_' {
            word_start = i;
        } else {
            break;
        }
    }
    (word_start, &prefix[word_start..])
}

pub fn check_following_paren(line: &str, safe_col: usize) -> (usize, bool) {
    let mut word_end = safe_col;
    for (i, c) in line[safe_col..].char_indices() {
        if c.is_alphanumeric() || c == '_' {
            word_end = safe_col + i + c.len_utf8();
        } else {
            break;
        }
    }
    let has_paren = line[word_end..].trim_start().starts_with('(');
    (word_end, has_paren)
}

#[inline]
pub fn starts_with_ignore_ascii_case(s: &str, prefix: &str) -> bool {
    if prefix.len() > s.len() {
        return false;
    }
    s.as_bytes()[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())
}

fn make_completion_item(
    label: &str,
    kind: u64,
    detail: &str,
    doc: &str,
    insert: (&str, u64),
    range: &Value,
    sort_text: &str,
) -> Value {
    let (insert_text, insert_format) = insert;
    let mut item = json!({
        "label": label,
        "kind": kind,
        "detail": detail,
        "insertText": insert_text,
        "insertTextFormat": insert_format,
        "textEdit": {
            "range": range,
            "newText": insert_text
        },
        "sortText": sort_text
    });
    if !doc.is_empty() {
        item["documentation"] = json!({
            "kind": "markdown",
            "value": doc
        });
    }
    item
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

    let line = match doc.lines().nth(line_idx) {
        Some(l) => l,
        None => return json!([]),
    };

    let safe_col = signature::safe_floor_char_boundary(line, col_idx.min(line.len()));
    let prefix = &line[..safe_col];

    let is_tag = line.contains("Tags") || is_inside_tags_block(doc, line_idx);
    let is_include = prefix.trim_start().starts_with("#include");

    if signature::is_in_comment_or_string(doc, line_idx, col_idx) && !is_tag && !is_include {
        return json!([]);
    }

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
                let after_eq = &line_before_cursor[eq_idx + 1..];
                let quotes_after_eq = after_eq.chars().filter(|&c| c == '"').count();
                // If there are fewer than 2 quotes after '=', cursor is still inside the tag value
                if quotes_after_eq < 2 {
                    let before_eq = line_before_cursor[..eq_idx].trim();
                    let key = before_eq.rsplit('"').nth(1)
                        .or_else(|| before_eq.split_whitespace().last())
                        .unwrap_or(before_eq)
                        .trim_start_matches('{')
                        .trim();
                    let clean_key = key.trim_matches(|c: char| c == '"' || c == '{' || c == '}' || c.is_whitespace());
                    // Only suggest values if the key is valid and before_eq does not have consecutive quotes syntax error
                    if !clean_key.is_empty() && !before_eq.contains("\"\"") {
                        let after_eq_trimmed = after_eq.trim_start();
                        let already_has_quote = after_eq_trimmed.starts_with('"');
                        let closing_quote_present = line_after_cursor.trim_start().starts_with('"');

                        for (tag_key, values, desc) in docs::SHADERLAB_TAG_KEYS_AND_VALUES {
                            if tag_key.eq_ignore_ascii_case(clean_key) {
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
                                        "filterText": clean_val,
                                        "sortText": format!("00_{:02}_{}", v_idx, clean_val),
                                    }));
                                }
                                return json!(sl_items);
                            }
                        }
                    }
                }
            }

            // Inside Tags block (before '=' or after a completed tag):
            let clean_prefix_token = prefix.split_whitespace().last().unwrap_or("");
            let starts_with_quote = clean_prefix_token.starts_with('"') || prefix.trim_end().ends_with('"');
            let has_closing_quote = line_after_cursor.trim_start().starts_with('"');

            // 1. FIRST: Tag Keys alone (Highest priority, sortText: "00_...")
            for (idx, (tag_key, _values, desc)) in docs::SHADERLAB_TAG_KEYS_AND_VALUES.iter().enumerate() {
                let (insert_text, filter_text) = if starts_with_quote && has_closing_quote {
                    ((*tag_key).to_string(), format!("\"{}\"", tag_key))
                } else if starts_with_quote {
                    (format!("{}\"", tag_key), format!("\"{}\"", tag_key))
                } else {
                    (format!("\"{}\"", tag_key), (*tag_key).to_string())
                };
                sl_items.push(json!({
                    "label": format!("\"{}\"", tag_key),
                    "kind": 10,
                    "detail": format!("Tag: {}", desc),
                    "insertText": insert_text,
                    "filterText": filter_text,
                    "sortText": format!("00_{:02}_{}", idx, tag_key),
                }));
            }

            // 2. SECOND: Complete Tag Template with default value (Lower priority, sortText: "01_...")
            // Only suggest full snippet if there is NOT already an '=' after cursor on this line!
            let line_after_has_eq = line_after_cursor.contains('=');
            if !line_after_has_eq {
                for (idx, (tag_key, values, desc)) in docs::SHADERLAB_TAG_KEYS_AND_VALUES.iter().enumerate() {
                    let default_val = values.first().map(|v| v.trim_matches('"')).unwrap_or("Opaque");
                    let (snippet, filter_text) = if starts_with_quote {
                        (format!("{}\" = \"${{1:{}}}\"", tag_key, default_val), format!("\"{}\"", tag_key))
                    } else {
                        (format!("\"{}\" = \"${{1:{}}}\"", tag_key, default_val), (*tag_key).to_string())
                    };
                    sl_items.push(json!({
                        "label": format!("\"{}\" = \"{}\"", tag_key, default_val),
                        "kind": 15,
                        "detail": format!("Snippet: {}", desc),
                        "insertText": snippet,
                        "insertTextFormat": 2,
                        "filterText": filter_text,
                        "sortText": format!("01_{:02}_{}", idx, tag_key),
                    }));
                }
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

                    let member_range = json!({
                        "start": { "line": line_idx, "character": dot_idx + 1 },
                        "end": { "line": line_idx, "character": safe_col }
                    });

                    if let Some(target_type) = current_type {
                        // Struct fields
                        if let Some(s_def) = structs.iter().find(|s| s.name == target_type) {
                            let field_items: Vec<Value> = s_def.fields.iter().enumerate().map(|(idx, f)| {
                                let sort_text = format!("00_{:02}_{}", idx, f.name);
                                make_completion_item(
                                    &f.name,
                                    5,
                                    &format!("{} {}.{}", f.field_type, s_def.name, f.name),
                                    "",
                                    (&f.name, 1),
                                    &member_range,
                                    &sort_text,
                                )
                            }).collect();
                            return json!(field_items);
                        }

                        // RWTexture methods
                        if target_type.starts_with("RWTexture") {
                            let method_items: Vec<Value> = docs::RWTEXTURE_METHODS.iter().enumerate().map(|(idx, m)| {
                                let sort_text = format!("00_{:02}_{}", idx, m.name);
                                let snip = format!("{}$0", m.snippet);
                                make_completion_item(
                                    m.name,
                                    2,
                                    m.signature,
                                    m.description,
                                    (&snip, 2),
                                    &member_range,
                                    &sort_text,
                                )
                            }).collect();
                            return json!(method_items);
                        }

                        // Texture methods
                        if target_type.starts_with("Texture") {
                            let method_items: Vec<Value> = docs::TEXTURE_METHODS.iter().enumerate().map(|(idx, m)| {
                                let sort_text = format!("00_{:02}_{}", idx, m.name);
                                let snip = format!("{}$0", m.snippet);
                                make_completion_item(
                                    m.name,
                                    2,
                                    m.signature,
                                    m.description,
                                    (&snip, 2),
                                    &member_range,
                                    &sort_text,
                                )
                            }).collect();
                            return json!(method_items);
                        }

                        // Buffer methods
                        if target_type.contains("Buffer") {
                            let method_items: Vec<Value> = docs::BUFFER_METHODS.iter().enumerate().map(|(idx, m)| {
                                let sort_text = format!("00_{:02}_{}", idx, m.name);
                                let snip = format!("{}$0", m.snippet);
                                make_completion_item(
                                    m.name,
                                    2,
                                    m.signature,
                                    m.description,
                                    (&snip, 2),
                                    &member_range,
                                    &sort_text,
                                )
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
                                    let sort_text = format!("00_{:02}_{}", idx, label);
                                    matrix_items.push(make_completion_item(
                                        &label,
                                        5, // Field
                                        &format!("Matrix element [{r}][{c}] (0-based)"),
                                        "",
                                        (&label, 1),
                                        &member_range,
                                        &sort_text,
                                    ));
                                    idx += 1;
                                }
                            }
                            // 1-based: _11, _12, ...
                            for r in 1..=rows {
                                for c in 1..=cols {
                                    let label = format!("_{r}{c}");
                                    let sort_text = format!("01_{:02}_{}", idx, label);
                                    matrix_items.push(make_completion_item(
                                        &label,
                                        5, // Field
                                        &format!("Matrix element [{r}][{c}] (1-based)"),
                                        "",
                                        (&label, 1),
                                        &member_range,
                                        &sort_text,
                                    ));
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
                                let sort_text = format!("00_{:02}_{}", idx, sw);
                                make_completion_item(
                                    sw,
                                    10,
                                    &format!("Swizzle .{sw}"),
                                    "",
                                    (sw, 1),
                                    &member_range,
                                    &sort_text,
                                )
                            }).collect();
                            return json!(items);
                        }
                    }

                    // Fallback for member access '.' when target_type could not be inferred:
                    // Provide swizzles, common struct fields (positionHCS, uv, etc.) and texture methods
                    let mut fallback_items = Vec::new();
                    let mut seen_fields: std::collections::HashSet<&str> = std::collections::HashSet::new();

                    // 1. Swizzles
                    const COMMON_SWIZZLES: &[&str] = &["x", "y", "z", "w", "xy", "zw", "xyz", "rgb", "rgba"];
                    for (idx, sw) in COMMON_SWIZZLES.iter().enumerate() {
                        if seen_fields.insert(*sw) {
                            let sort_text = format!("00_{:02}_{}", idx, sw);
                            fallback_items.push(make_completion_item(
                                sw,
                                10,
                                &format!("Swizzle .{sw}"),
                                "",
                                (sw, 1),
                                &member_range,
                                &sort_text,
                            ));
                        }
                    }

                    // 2. Struct fields from known structs in this file/includes
                    for s in &structs {
                        for f in &s.fields {
                            if seen_fields.insert(&f.name) {
                                let sort_text = format!("05_{}", f.name);
                                fallback_items.push(make_completion_item(
                                    &f.name,
                                    5,
                                    &format!("{} (field of {})", f.field_type, s.name),
                                    "",
                                    (&f.name, 1),
                                    &member_range,
                                    &sort_text,
                                ));
                            }
                        }
                    }

                    // 3. Common texture & buffer methods
                    for (idx, m) in docs::TEXTURE_METHODS.iter().enumerate() {
                        if seen_fields.insert(m.name) {
                            let sort_text = format!("10_{:02}_{}", idx, m.name);
                            let snip = format!("{}$0", m.snippet);
                            fallback_items.push(make_completion_item(
                                m.name,
                                2,
                                m.signature,
                                m.description,
                                (&snip, 2),
                                &member_range,
                                &sort_text,
                            ));
                        }
                    }

                    for (idx, m) in docs::BUFFER_METHODS.iter().enumerate() {
                        if seen_fields.insert(m.name) {
                            let sort_text = format!("11_{:02}_{}", idx, m.name);
                            let snip = format!("{}$0", m.snippet);
                            fallback_items.push(make_completion_item(
                                m.name,
                                2,
                                m.signature,
                                m.description,
                                (&snip, 2),
                                &member_range,
                                &sort_text,
                            ));
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
            let sem_range = json!({
                "start": { "line": line_idx, "character": colon_idx + 1 },
                "end": { "line": line_idx, "character": safe_col }
            });
            let semantic_items: Vec<Value> = docs::BUILTIN_VARIABLES.iter().enumerate().map(|(idx, (sem, desc))| {
                let sort_text = format!("00_{:02}_{}", idx, sem);
                make_completion_item(
                    sem,
                    6,
                    "HLSL Semantic",
                    desc,
                    (sem, 1),
                    &sem_range,
                    &sort_text,
                )
            }).collect();
            return json!(semantic_items);
        }
    }

    let (word_start, word) = extract_word_prefix(prefix);
    let (word_end, following_has_paren) = check_following_paren(line, safe_col);

    let replace_range = json!({
        "start": { "line": line_idx, "character": word_start },
        "end": { "line": line_idx, "character": word_end }
    });

    let mut items = Vec::new();
    let (user_funcs, user_vars, user_structs) = signature::resolve_includes_and_scan_symbols(uri, doc, doc_cache);
    let sl_props = if context == ShaderContext::UnityShaderLab {
        signature::scan_shaderlab_properties(doc)
    } else {
        Vec::new()
    };
    let mut seen_labels: std::collections::HashSet<&str> = std::collections::HashSet::new();

    // 0. Parameters & Local Variables of Enclosing Function (Priority: HIGHEST!)
    if let Some(f) = signature::find_enclosing_function(&user_funcs, line_idx) {
        for (idx, p) in f.parsed_params.iter().enumerate() {
            if (word.is_empty() || starts_with_ignore_ascii_case(&p.name, word))
                && seen_labels.insert(&p.name)
            {
                let sort_text = format!("00_{:02}_{}", idx, p.name);
                items.push(make_completion_item(
                    &p.name,
                    6,
                    &format!("{} {} (parameter)", p.param_type, p.name),
                    &format!("Parameter of function `{}`", f.name),
                    (&p.name, 1),
                    &replace_range,
                    &sort_text,
                ));
            }
        }

        for (idx, lv) in f.local_vars.iter().filter(|v| v.line <= line_idx).enumerate() {
            if (word.is_empty() || starts_with_ignore_ascii_case(&lv.name, word))
                && seen_labels.insert(&lv.name)
            {
                let sort_text = format!("01_{:02}_{}", idx, lv.name);
                items.push(make_completion_item(
                    &lv.name,
                    6,
                    &format!("{} {} (local)", lv.var_type, lv.name),
                    &format!("Local variable declared in `{}` at line {}", f.name, lv.line + 1),
                    (&lv.name, 1),
                    &replace_range,
                    &sort_text,
                ));
            }
        }
    }

    // 1. User-defined global variables & cbuffer members
    for (idx, v) in user_vars.iter().enumerate() {
        if (word.is_empty() || starts_with_ignore_ascii_case(&v.name, word))
            && seen_labels.insert(&v.name)
        {
            let detail = if v.qualifier.is_empty() {
                format!("{} {}", v.var_type, v.name)
            } else {
                format!("{} {} ({})", v.var_type, v.name, v.qualifier)
            };
            let kind = match v.qualifier.as_str() {
                "struct" => 22,
                "const" | "#define" => 21,
                _ => 6,
            };
            let sort_text = format!("10_{:03}_{}", idx, v.name);
            items.push(make_completion_item(
                &v.name,
                kind,
                &detail,
                v.doc.as_deref().unwrap_or("Global variable"),
                (&v.name, 1),
                &replace_range,
                &sort_text,
            ));
        }
    }

    // 2. User-defined functions
    for (idx, f) in user_funcs.iter().enumerate() {
        if (word.is_empty() || starts_with_ignore_ascii_case(&f.name, word))
            && seen_labels.insert(&f.name)
        {
            let (insert_text, insert_format) = if following_has_paren {
                (f.name.clone(), 1)
            } else if f.parameters.is_empty() {
                (format!("{}()$0", f.name), 2)
            } else {
                (format!("{}($1)$0", f.name), 2)
            };
            let sort_text = format!("15_{:03}_{}", idx, f.name);
            items.push(make_completion_item(
                &f.name,
                3,
                &f.label,
                f.doc.as_deref().unwrap_or("User function"),
                (&insert_text, insert_format),
                &replace_range,
                &sort_text,
            ));
        }
    }

    // 3. User structs (Types!)
    let structs = &user_structs;
    for (idx, s) in structs.iter().enumerate() {
        if (word.is_empty() || starts_with_ignore_ascii_case(&s.name, word))
            && seen_labels.insert(&s.name)
        {
            let fields_doc = if s.fields.is_empty() {
                String::new()
            } else {
                let f_list = s.fields.iter().map(|f| format!("- `{} {}`", f.field_type, f.name)).collect::<Vec<_>>().join("\n");
                format!("\n\n### Fields:\n{}", f_list)
            };
            let doc_value = format!("```hlsl\nstruct {}\n```{}", s.name, fields_doc);
            let sort_text = format!("20_{:02}_{}", idx, s.name);
            items.push(make_completion_item(
                &s.name,
                22,
                &format!("struct {}", s.name),
                &doc_value,
                (&s.name, 1),
                &replace_range,
                &sort_text,
            ));
        }
    }

    // 3b. Material properties from ShaderLab Properties block (for HLSL & CBuffer completion)
    if context == ShaderContext::UnityShaderLab {
        for (idx, p) in sl_props.iter().enumerate() {
            if (word.is_empty() || starts_with_ignore_ascii_case(&p.name, word))
                && seen_labels.insert(&p.name)
            {
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
                let sort_text = format!("12_{:02}_{}", idx, p.name);
                let doc_val = format!("Material property `{}` (`{}`)\n\nShaderLab Property: `{}`", p.name, p.prop_type, p.display_name);
                items.push(make_completion_item(
                    &p.name,
                    6,
                    &format!("{} {} (from Properties)", hlsl_type, p.name),
                    &doc_val,
                    (&p.name, 1),
                    &replace_range,
                    &sort_text,
                ));
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

        if (word.is_empty() || starts_with_ignore_ascii_case(ev.name, word))
            && seen_labels.insert(ev.name)
        {
            let kind = match ev.var_type {
                "function" => 3, // Function
                "macro" => 14,   // Keyword / Macro
                _ => 6,          // Variable
            };
            let has_params = (ev.var_type == "function" || ev.var_type == "macro") && ev.detail.contains('(');
            let (insert_text, insert_format) = if has_params {
                if following_has_paren {
                    (ev.name.to_string(), 1)
                } else {
                    (format!("{}($1)$0", ev.name), 2)
                }
            } else {
                (ev.name.to_string(), 1)
            };
            let sort_text = format!("15_{:02}_{}", idx, ev.name);
            items.push(make_completion_item(
                ev.name,
                kind,
                ev.detail,
                ev.description,
                (&insert_text, insert_format),
                &replace_range,
                &sort_text,
            ));
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

        if (word.is_empty() || starts_with_ignore_ascii_case(func.name, word))
            && seen_labels.insert(func.name)
        {
            let primary_overload = func.overloads.first().map(|o| o.label).unwrap_or(func.name);
            let (insert_text, insert_format) = if following_has_paren {
                (func.name.to_string(), 1)
            } else {
                let has_params = func.overloads.iter().any(|o| !o.params.is_empty());
                if has_params {
                    (format!("{}($1)$0", func.name), 2)
                } else {
                    (format!("{}()$0", func.name), 2)
                }
            };
            let sort_text = format!("30_{:03}_{}", idx, func.name);
            items.push(make_completion_item(
                func.name,
                3,
                primary_overload,
                func.description,
                (&insert_text, insert_format),
                &replace_range,
                &sort_text,
            ));
        }
    }

    // 5. Built-in Types
    for (idx, t) in docs::BUILTIN_TYPES.iter().enumerate() {
        if (word.is_empty() || starts_with_ignore_ascii_case(t, word))
            && seen_labels.insert(*t)
        {
            let sort_text = format!("35_{:03}_{}", idx, t);
            items.push(make_completion_item(
                t,
                25,
                "HLSL Type",
                "",
                (t, 1),
                &replace_range,
                &sort_text,
            ));
        }
    }

    // 6. Built-in Keywords (HLSL only, exclude ShaderLab keywords)
    let hlsl_keywords = [
        "groupshared", "cbuffer", "tbuffer", "struct", "register", "packoffset",
        "static", "const", "inline", "extern", "volatile", "precise",
        "return", "if", "else", "for", "while", "do", "switch", "case", "default",
        "break", "continue", "discard", "true", "false",
        "in", "out", "inout", "row_major", "column_major", "numthreads",
    ];
    for (idx, kw) in hlsl_keywords.iter().enumerate() {
        if (word.is_empty() || starts_with_ignore_ascii_case(kw, word))
            && seen_labels.insert(*kw)
        {
            let sort_text = format!("40_{:03}_{}", idx, kw);
            items.push(make_completion_item(
                kw,
                14,
                "",
                "",
                (kw, 1),
                &replace_range,
                &sort_text,
            ));
        }
    }

    // 7. High-Productivity Snippets
    let snippets = [
        ("vert", "Vertex Shader function", "Varyings vert(Attributes input)\n{\n    Varyings output = (Varyings)0;\n    output.positionCS = TransformObjectToHClip(input.positionOS.xyz);\n    $0\n    return output;\n}"),
        ("frag", "Fragment/Pixel Shader function", "float4 frag(Varyings input) : SV_Target\n{\n    $0\n    return float4(1.0, 1.0, 1.0, 1.0);\n}"),
        ("kernel", "Compute Shader kernel", "[numthreads(${1:8}, ${2:8}, ${3:1})]\nvoid ${4:CSMain}(uint3 id : SV_DispatchThreadID)\n{\n    $0\n}"),
        ("struct", "Struct declaration", "struct $1\n{\n    $0\n};"),
        ("cbuffer", "Constant Buffer with register", "cbuffer $1 : register(b${2:0})\n{\n    $0\n};"),
        ("cbuffer_unity_per_material", "CBUFFER_START(UnityPerMaterial) block", "CBUFFER_START(UnityPerMaterial)\n    $0\nCBUFFER_END"),
        ("cbuffer_unity_per_draw", "CBUFFER_START(UnityPerDraw) block", "CBUFFER_START(UnityPerDraw)\n    $0\nCBUFFER_END"),
        ("cbuffer_unity_per_camera", "CBUFFER_START(UnityPerCamera) block", "CBUFFER_START(UnityPerCamera)\n    $0\nCBUFFER_END"),
        ("tex2d", "Texture2D and SamplerState pair", "Texture2D $1 : register(t${2:0});\nSamplerState sampler_$1 : register(s${2:0});"),
        ("for", "For loop", "for (int ${1:i} = 0; ${1:i} < ${2:count}; ++${1:i})\n{\n    $0\n}"),
        ("while", "While loop", "while ($1)\n{\n    $0\n}"),
        ("if", "If condition", "if ($1)\n{\n    $0\n}"),
        ("ifelse", "If-Else statement", "if ($1)\n{\n    $2\n}\nelse\n{\n    $0\n}"),
        ("switch", "Switch statement", "switch ($1)\n{\n    case $2:\n        break;\n    default:\n        break;\n}"),
    ];
    for (idx, (label, detail, snip)) in snippets.iter().enumerate() {
        if (word.is_empty() || starts_with_ignore_ascii_case(label, word))
            && seen_labels.insert(*label)
        {
            let sort_text = format!("50_{:02}_{}", idx, label);
            items.push(make_completion_item(
                label,
                15,
                detail,
                "",
                (snip, 2),
                &replace_range,
                &sort_text,
            ));
        }
    }

    json!(items)
}

fn send_lsp_message<W: Write>(writer: &mut W, val: &Value) -> io::Result<()> {
    let payload = serde_json::to_string(val)?;
    write!(writer, "Content-Length: {}\r\n\r\n{}", payload.len(), payload)?;
    writer.flush()
}

fn write_lsp_response(stdout: &Mutex<io::Stdout>, id: &Value, result: Value) {
    let resp = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    });
    if let Ok(mut lock) = stdout.lock() {
        let _ = send_lsp_message(&mut *lock, &resp);
    }
}

fn send_diagnostics(stdout: &Mutex<io::Stdout>, uri: &str, diags: &[Diagnostic]) {
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": diags
        }
    });
    if let Ok(mut lock) = stdout.lock() {
        let _ = send_lsp_message(&mut *lock, &notification);
    }
}

fn get_doc_or_read<'a>(cache: &'a HashMap<String, String>, uri: &str, owned: &'a mut String) -> &'a str {
    if let Some(d) = get_document_from_cache(cache, uri) {
        d
    } else {
        *owned = uri_to_path(uri).and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default();
        owned.as_str()
    }
}

fn main() {
    let dxc_path_shared = Arc::new(RwLock::new(find_dxc_path()));

    let (tx, rx): (Sender<ValidationTask>, Receiver<ValidationTask>) = mpsc::channel();
    let doc_cache: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    let workspace_root: Arc<RwLock<Option<PathBuf>>> = Arc::new(RwLock::new(
        env::var("WORKSPACE_ROOT").ok().map(PathBuf::from),
    ));

    let stdout_shared = Arc::new(Mutex::new(io::stdout()));
    let stdout_worker = Arc::clone(&stdout_shared);

    let worker_dxc = Arc::clone(&dxc_path_shared);
    let worker_ws = Arc::clone(&workspace_root);
    let worker_cache = Arc::clone(&doc_cache);
    thread::spawn(move || {
        let mut pending_tasks: HashMap<String, (ValidationTask, Instant)> = HashMap::new();
        loop {
            while let Ok(task) = rx.try_recv() {
                pending_tasks.insert(task.uri.clone(), (task, Instant::now()));
            }

            let ready_uris: Vec<String> = pending_tasks
                .iter()
                .filter(|(_, (_, time))| time.elapsed() >= Duration::from_millis(120))
                .map(|(uri, _)| uri.clone())
                .collect();

            if ready_uris.is_empty() {
                if pending_tasks.is_empty() {
                    match rx.recv() {
                        Ok(task) => {
                            pending_tasks.insert(task.uri.clone(), (task, Instant::now()));
                        }
                        Err(_) => break,
                    }
                } else {
                    thread::sleep(Duration::from_millis(20));
                }
                continue;
            }

            for uri in ready_uris {
                if let Some((task, _)) = pending_tasks.remove(&uri) {
                    let ws_opt = worker_ws.read().ok().and_then(|g| g.clone());
                    let current_dxc = worker_dxc.read().map(|g| g.clone()).unwrap_or_else(|_| "dxc.exe".to_string());
                    let diags = {
                        let cache_guard = worker_cache.read().unwrap();
                        validate_shader(&task.uri, &task.content, &current_dxc, ws_opt.as_deref(), &cache_guard)
                    };
                    send_diagnostics(&stdout_worker, &task.uri, &diags);
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
                            if let Ok(mut g) = workspace_root.write() {
                                *g = Some(ws);
                            }
                        }

                        if let Some(opts) = params.get("initializationOptions") {
                            let keys = ["dxc_path", "dxcPath", "dxc"];
                            for key in keys {
                                if let Some(p) = opts.get(key).and_then(|v| v.as_str()) {
                                    if let Some(resolved) = try_resolve_dxc(p) {
                                        if let Ok(mut g) = dxc_path_shared.write() {
                                            *g = resolved;
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    let result = json!({
                        "capabilities": {
                            "textDocumentSync": 1,
                            "completionProvider": {
                                "triggerCharacters": ["."]
                            },
                            "signatureHelpProvider": {
                                "triggerCharacters": ["(", ","],
                                "retriggerCharacters": [","]
                            },
                            "hoverProvider": true,
                            "definitionProvider": true,
                            "documentSymbolProvider": true,
                            "documentFormattingProvider": true
                        }
                    });
                    write_lsp_response(&stdout_shared, id, result);
                }
            }
            "workspace/didChangeConfiguration" => {
                if let Some(params) = msg.get("params") {
                    if let Some(settings) = params.get("settings") {
                        let keys = ["dxc_path", "dxcPath", "dxc"];
                        for key in keys {
                            if let Some(p) = settings.get(key).and_then(|v| v.as_str()) {
                                if let Some(resolved) = try_resolve_dxc(p) {
                                    if let Ok(mut g) = dxc_path_shared.write() {
                                        *g = resolved;
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            "textDocument/didOpen" => {
                if let Some(params) = msg.get("params") {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                    let text = params["textDocument"]["text"].as_str().unwrap_or("").to_string();

                    if let Ok(mut cache) = doc_cache.write() {
                        cache.insert(uri.clone(), text.clone());
                    }
                    let _ = tx.send(ValidationTask { uri, content: text });
                }
            }
            "textDocument/didChange" => {
                if let Some(params) = msg.get("params") {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                    if let Some(changes) = params["contentChanges"].as_array() {
                        if let Some(last_change) = changes.last() {
                            let text = last_change["text"].as_str().unwrap_or("").to_string();
                            if let Ok(mut cache) = doc_cache.write() {
                                cache.insert(uri.clone(), text.clone());
                            }
                            let _ = tx.send(ValidationTask { uri, content: text });
                        }
                    }
                }
            }
            "textDocument/didClose" => {
                if let Some(params) = msg.get("params") {
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    if let Ok(mut cache) = doc_cache.write() {
                        cache.remove(uri);
                    }
                    send_diagnostics(&stdout_shared, uri, &[]);
                }
            }
            "textDocument/completion" => {
                if let Some(id) = id {
                    let cache_guard = doc_cache.read().unwrap();
                    let res = handle_completion(&msg, &cache_guard);
                    write_lsp_response(&stdout_shared, id, res);
                }
            }
            "textDocument/signatureHelp" => {
                if let Some(id) = id {
                    let params = &msg["params"];
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    let line = params["position"]["line"].as_u64().unwrap_or(0) as usize;
                    let col = params["position"]["character"].as_u64().unwrap_or(0) as usize;

                    let cache_guard = doc_cache.read().unwrap();
                    let mut owned = String::new();
                    let doc = get_doc_or_read(&cache_guard, uri, &mut owned);
                    let res = signature::get_signature_help(uri, doc, line, col, &cache_guard);
                    write_lsp_response(&stdout_shared, id, res);
                }
            }
            "textDocument/hover" => {
                if let Some(id) = id {
                    let params = &msg["params"];
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    let line = params["position"]["line"].as_u64().unwrap_or(0) as usize;
                    let col = params["position"]["character"].as_u64().unwrap_or(0) as usize;

                    let cache_guard = doc_cache.read().unwrap();
                    let mut owned = String::new();
                    let doc = get_doc_or_read(&cache_guard, uri, &mut owned);
                    let res = signature::get_hover_info(uri, doc, line, col, &cache_guard);
                    write_lsp_response(&stdout_shared, id, res);
                }
            }
            "textDocument/definition" => {
                if let Some(id) = id {
                    let cache_guard = doc_cache.read().unwrap();
                    let res = signature::handle_definition(&msg, &cache_guard);
                    write_lsp_response(&stdout_shared, id, res);
                }
            }
            "textDocument/documentSymbol" => {
                if let Some(id) = id {
                    let params = &msg["params"];
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    let cache_guard = doc_cache.read().unwrap();
                    let mut owned = String::new();
                    let doc = get_doc_or_read(&cache_guard, uri, &mut owned);
                    let symbols = signature::get_document_symbols(doc);
                    write_lsp_response(&stdout_shared, id, symbols);
                }
            }
            "textDocument/formatting" => {
                if let Some(id) = id {
                    let params = &msg["params"];
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    let options = &params["options"];
                    let tab_size = options["tabSize"].as_u64().unwrap_or(4) as usize;
                    let insert_spaces = options["insertSpaces"].as_bool().unwrap_or(true);

                    let cache_guard = doc_cache.read().unwrap();
                    let mut owned = String::new();
                    let doc = get_doc_or_read(&cache_guard, uri, &mut owned);
                    let edits = format_document(doc, tab_size, insert_spaces);
                    write_lsp_response(&stdout_shared, id, json!(edits));
                }
            }
            "shutdown" => {
                if let Some(id) = id {
                    write_lsp_response(&stdout_shared, id, json!(null));
                }
            }
            "exit" => {
                return;
            }
            _ => {
                if let Some(id) = id {
                    write_lsp_response(&stdout_shared, id, json!(null));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dxc_architecture_constants() {
        assert!(!TARGET_DXC_ARCH.is_empty());
        #[cfg(target_arch = "x86_64")]
        assert_eq!(TARGET_DXC_ARCH, "x64");
        #[cfg(target_arch = "aarch64")]
        assert_eq!(TARGET_DXC_ARCH, "arm64");
        #[cfg(target_arch = "x86")]
        assert_eq!(TARGET_DXC_ARCH, "x86");
    }

    #[test]
    fn test_try_resolve_dxc_empty_and_missing() {
        assert_eq!(try_resolve_dxc(""), None);
        assert_eq!(try_resolve_dxc("   "), None);
        assert_eq!(try_resolve_dxc("definitely_non_existent_binary_xyz123.exe"), None);
    }

    #[test]
    fn test_try_resolve_dxc_existing_file() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let resolved = try_resolve_dxc(manifest.to_str().unwrap());
        assert!(resolved.is_some());
        assert!(Path::new(&resolved.unwrap()).is_file());
    }

    #[test]
    fn test_find_dxc_path_not_empty() {
        let path = find_dxc_path();
        assert!(!path.trim().is_empty());
    }

    #[test]
    fn test_scan_dir_for_dxc_mock_work_dir() {
        let temp = env::temp_dir().join(format!("test_zed_work_{}", std::process::id()));
        let dxc_dir = temp.join("dxc-v1.9.2607").join("bin").join(TARGET_DXC_ARCH);
        fs::create_dir_all(&dxc_dir).unwrap();
        let dummy_dxc = dxc_dir.join(DXC_BINARY_NAME);
        fs::write(&dummy_dxc, b"dummy binary").unwrap();

        let found = scan_dir_for_dxc(&temp);
        assert!(found.is_some());
        assert_eq!(PathBuf::from(found.unwrap()), dummy_dxc);

        let _ = fs::remove_dir_all(&temp);

        // Also test bin/dxc without arch subfolder (standard Linux release layout)
        let temp2 = env::temp_dir().join(format!("test_zed_work_direct_{}", std::process::id()));
        let dxc_dir2 = temp2.join("dxc-v1.9.2607").join("bin");
        fs::create_dir_all(&dxc_dir2).unwrap();
        let dummy_dxc2 = dxc_dir2.join(DXC_BINARY_NAME);
        fs::write(&dummy_dxc2, b"dummy binary 2").unwrap();

        let found2 = scan_dir_for_dxc(&temp2);
        assert!(found2.is_some());
        assert_eq!(PathBuf::from(found2.unwrap()), dummy_dxc2);

        let _ = fs::remove_dir_all(&temp2);
    }
}
