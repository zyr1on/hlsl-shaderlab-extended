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
    if uri.ends_with(".shader") || content.contains("Shader \"") {
        ShaderContext::UnityShaderLab
    } else if uri.ends_with(".usf") || uri.ends_with(".ush") || content.contains("/Engine/") {
        ShaderContext::UnrealEngine
    } else if uri.ends_with(".cginc")
        || content.contains("HLSLPROGRAM")
        || content.contains("CGPROGRAM")
        || content.contains("UnityCG")
        || uri.contains("Assets")
        || uri.contains("Packages")
    {
        ShaderContext::UnityHlsl
    } else {
        ShaderContext::PureHlsl
    }
}

pub fn is_inside_properties_block(doc: &str, target_line: usize) -> bool {
    let mut in_props = false;
    let mut brace_depth = 0;
    for (idx, line) in doc.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("Properties") {
            in_props = true;
        }
        let open_b = line.chars().filter(|&c| c == '{').count();
        let close_b = line.chars().filter(|&c| c == '}').count();
        brace_depth += open_b;
        if in_props && brace_depth > 0 && close_b >= brace_depth {
            in_props = false;
        }
        brace_depth = brace_depth.saturating_sub(close_b);
        if idx == target_line && in_props {
            return true;
        }
    }
    false
}

pub static SHADERLAB_PROPERTY_ATTRIBUTES: &[(&str, &str)] = &[
    ("[HDR]", "Marks a Color or Texture property as High Dynamic Range"),
    ("[HideInInspector]", "Hides property from the default Material Inspector"),
    ("[Toggle]", "Displays a boolean checkbox toggle in the Material Inspector"),
    ("[Normal]", "Validates that assigned texture is marked as Normal Map"),
    ("[NoScaleOffset]", "Hides the Tiling and Offset fields for texture property"),
    ("[IntRange]", "Restricts slider steps to integer values only"),
];

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
struct UnitySampler2D { Texture2D t; SamplerState s; };
#define sampler2D UnitySampler2D
#define tex2D(tex, uv) (tex.t.Sample(tex.s, uv))
#define tex2Dlod(tex, uv) (tex.t.SampleLevel(tex.s, (uv).xy, (uv).w))
#define TRANSFORM_TEX(tex,name) ((tex.xy) * name##_ST.xy + name##_ST.zw)
float4x4 UNITY_MATRIX_MVP;
float4x4 unity_ObjectToWorld;
float4x4 unity_WorldToObject;
inline float4 UnityObjectToClipPos(float3 pos) { return mul(UNITY_MATRIX_MVP, float4(pos, 1.0)); }
inline float4 UnityObjectToClipPos(float4 pos) { return mul(UNITY_MATRIX_MVP, pos); }
inline float3 UnityObjectToWorldNormal(float3 norm) { return mul((float3x3)unity_WorldToObject, norm); }
inline float3 UnityObjectToWorldDir(float3 dir) { return mul((float3x3)unity_ObjectToWorld, dir); }
#define TEXTURE2D(name) Texture2D name
#define SAMPLER(name) SamplerState name
#define SAMPLE_TEXTURE2D(name, samplerName, coord2) name.Sample(samplerName, coord2)
inline float4 TransformObjectToHClip(float3 pos) { return mul(UNITY_MATRIX_MVP, float4(pos, 1.0)); }
inline float4 TransformObjectToHClip(float4 pos) { return mul(UNITY_MATRIX_MVP, pos); }
#endif
"#;

const UNREAL_COMPAT_PREAMBLE: &str = r#"
#ifndef __UNREAL_BUILTIN_STUBS__
#define __UNREAL_BUILTIN_STUBS__
struct FMaterialPixelParameters {
    float3 WorldPosition;
    float3 WorldPosition_CamRelative;
    float3 WorldNormal;
    float4 ScreenPosition;
    float2 TexCoords[4];
};
struct FPixelMaterialInputs {
    float3 EmissiveColor;
    float3 BaseColor;
};
float3 Luminance(float3 LinearColor) {
    return dot(LinearColor, float3(0.3, 0.59, 0.11));
}
float3 RotateAboutAxis(float4 NormalizedRotationAxisAndAngle, float3 PivotPoint, float3 Position) {
    float3 Axis = NormalizedRotationAxisAndAngle.xyz;
    float Angle = NormalizedRotationAxisAndAngle.w;
    return Position + sin(Angle) * cross(Axis, Position - PivotPoint);
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

pub fn discover_include_paths(uri: &str, workspace_root: Option<&Path>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(file_path) = uri_to_path(uri) {
        if let Some(parent) = file_path.parent() {
            paths.push(parent.to_path_buf());

            let mut current = parent;
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
                        let block_hlsl = format!("{}\n{}", UNITY_COMPAT_PREAMBLE, block_lines.join("\n"));
                        let preamble_line_count = UNITY_COMPAT_PREAMBLE.lines().count();
                        let block_diags = run_dxc_on_text(&block_hlsl, dxc_path, block_start_line, &include_dirs, preamble_line_count);
                        diagnostics.extend(block_diags);
                    }
                    continue;
                }

                if in_block {
                    block_lines.push(line);
                }
            }
        }
        ShaderContext::UnityHlsl => {
            let wrapped = format!("{}\n{}", UNITY_COMPAT_PREAMBLE, content);
            let preamble_line_count = UNITY_COMPAT_PREAMBLE.lines().count();
            diagnostics = run_dxc_on_text(&wrapped, dxc_path, 0, &include_dirs, preamble_line_count);
        }
        ShaderContext::UnrealEngine => {
            let preamble = if content.contains("FMaterialPixelParameters") {
                "float3 Luminance(float3 LinearColor) { return dot(LinearColor, float3(0.3, 0.59, 0.11)); }\nfloat3 RotateAboutAxis(float4 NormalizedRotationAxisAndAngle, float3 PivotPoint, float3 Position) {\n    float3 Axis = NormalizedRotationAxisAndAngle.xyz;\n    float Angle = NormalizedRotationAxisAndAngle.w;\n    return Position + sin(Angle) * cross(Axis, Position - PivotPoint);\n}\n"
            } else {
                UNREAL_COMPAT_PREAMBLE
            };
            let wrapped = format!("{}\n{}", preamble, content);
            let preamble_line_count = preamble.lines().count();
            diagnostics = run_dxc_on_text(&wrapped, dxc_path, 0, &include_dirs, preamble_line_count);
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
        let parts: Vec<&str> = line.splitn(5, ':').collect();
        if parts.len() >= 5 {
            let (line_str, col_str, rest) = if parts[0].len() == 1 {
                if parts.len() >= 5 {
                    (parts[2].trim(), parts[3].trim(), parts[4].trim())
                } else {
                    continue;
                }
            } else {
                let rest_str = if parts.len() >= 4 { parts[3].trim() } else { "" };
                (parts[1].trim(), parts[2].trim(), rest_str)
            };

            if let (Ok(parsed_line), Ok(parsed_col)) = (line_str.parse::<usize>(), col_str.parse::<usize>()) {
                if parsed_line <= preamble_line_count {
                    // Ignore internal preamble errors if any
                    continue;
                }

                let actual_line = parsed_line.saturating_sub(preamble_line_count).saturating_sub(1) + line_offset;
                let actual_col = parsed_col.saturating_sub(1);

                let (severity, message) = if rest.contains("error:") {
                    let raw_msg = rest.split("error:").nth(1).unwrap_or(rest).trim();
                    if raw_msg.contains("file not found")
                        && (raw_msg.contains("Packages/") || raw_msg.contains("UnityCG") || raw_msg.contains("Engine/"))
                    {
                        // Stubs provided, don't flag missing engine files as errors
                        continue;
                    }
                    (1, raw_msg.to_string())
                } else if rest.contains("warning:") {
                    let msg = rest.split("warning:").nth(1).unwrap_or(rest).trim();
                    (2, msg.to_string())
                } else {
                    continue;
                };

                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: actual_line, character: actual_col },
                        end: Position { line: actual_line, character: actual_col + 5 },
                    },
                    severity,
                    message,
                    source: "dxc".to_string(),
                });
            }
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

        let open_count = trimmed.chars().filter(|&c| c == '{').count();
        let close_count = trimmed.chars().filter(|&c| c == '}').count();
        let net_close_after = close_count.saturating_sub(leading_close);
        indent_level = indent_level.saturating_sub(net_close_after) + open_count;
    }

    let new_text = formatted_lines.join("\n") + "\n";
    let line_count = text.lines().count();
    let last_line_len = text.lines().last().map(|l| l.len()).unwrap_or(0);

    vec![json!({
        "range": {
            "start": { "line": 0, "character": 0 },
            "end": { "line": line_count, "character": last_line_len }
        },
        "newText": new_text
    })]
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

    let doc = match doc_cache.get(uri) {
        Some(d) => d,
        None => return json!([]),
    };

    if signature::is_in_comment_or_string(doc, line_idx, col_idx) {
        return json!([]);
    }

    let line = match doc.lines().nth(line_idx) {
        Some(l) => l,
        None => return json!([]),
    };

    let safe_col = col_idx.min(line.len());
    let prefix = &line[..safe_col];

    let context = detect_shader_context(uri, doc);

    // ------------------------------------------------------------------------
    // Context A: Unity ShaderLab (Outside HLSL blocks)
    // ------------------------------------------------------------------------
    if context == ShaderContext::UnityShaderLab && !is_inside_hlsl_block(doc, line_idx) {
        let mut sl_items = Vec::new();
        let in_props = is_inside_properties_block(doc, line_idx);

        if in_props {
            // 1. Property Attributes: [HDR], [HideInInspector], [Toggle]
            if prefix.trim_start().starts_with('[') {
                for (idx, (attr, desc)) in SHADERLAB_PROPERTY_ATTRIBUTES.iter().enumerate() {
                    sl_items.push(json!({
                        "label": *attr,
                        "kind": 14,
                        "detail": *desc,
                        "insertText": *attr,
                        "sortText": format!("00_{:02}_{}", idx, attr),
                    }));
                }
                return json!(sl_items);
            }

            // 2. Standard Property Templates
            let prop_templates = [
                ("_MainTex", "_MainTex (\"Texture\", 2D) = \"white\" {}", "Albedo 2D Texture slot"),
                ("_Color", "_Color (\"Color\", Color) = (1, 1, 1, 1)", "Main RGBA Color property"),
                ("_Glossiness", "_Glossiness (\"Smoothness\", Range(0, 1)) = 0.5", "Smoothness slider property"),
                ("_Metallic", "_Metallic (\"Metallic\", Range(0, 1)) = 0.0", "Metallic slider property"),
                ("_BumpMap", "_BumpMap (\"Normal Map\", 2D) = \"bump\" {}", "Tangent normal map slot"),
                ("_EmissionColor", "_EmissionColor (\"Emission\", Color) = (0, 0, 0, 1)", "HDR emission color"),
                ("_Vector", "_Vector (\"Vector\", Vector) = (0, 0, 0, 0)", "4D Vector property"),
            ];
            for (idx, (label, snip, desc)) in prop_templates.iter().enumerate() {
                sl_items.push(json!({
                    "label": *label,
                    "kind": 15,
                    "detail": *desc,
                    "insertText": *snip,
                    "sortText": format!("05_{:02}_{}", idx, label),
                }));
            }

            // 3. Property Types: 2D, Color, Float, Range, etc.
            for (idx, (prop_type, snip, desc)) in docs::SHADERLAB_PROPERTY_TYPES.iter().enumerate() {
                sl_items.push(json!({
                    "label": *prop_type,
                    "kind": 7,
                    "detail": *desc,
                    "insertText": *snip,
                    "sortText": format!("10_{:02}_{}", idx, prop_type),
                }));
            }

            return json!(sl_items);
        }

        // Outside Properties (In SubShader / Pass)
        let trimmed_prefix = prefix.trim();

        if trimmed_prefix.starts_with("Blend") {
            for (idx, (_, val, desc)) in docs::SHADERLAB_RENDER_STATES.iter().filter(|(s, _, _)| *s == "Blend").enumerate() {
                let insert = val.strip_prefix("Blend ").unwrap_or(val);
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

        if trimmed_prefix.starts_with("Cull") {
            for (idx, (_, val, desc)) in docs::SHADERLAB_RENDER_STATES.iter().filter(|(s, _, _)| *s == "Cull").enumerate() {
                let insert = val.strip_prefix("Cull ").unwrap_or(val);
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

        if trimmed_prefix.starts_with("ZWrite") {
            for (idx, (_, val, desc)) in docs::SHADERLAB_RENDER_STATES.iter().filter(|(s, _, _)| *s == "ZWrite").enumerate() {
                let insert = val.strip_prefix("ZWrite ").unwrap_or(val);
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

        if trimmed_prefix.starts_with("ZTest") {
            for (idx, (_, val, desc)) in docs::SHADERLAB_RENDER_STATES.iter().filter(|(s, _, _)| *s == "ZTest").enumerate() {
                let insert = val.strip_prefix("ZTest ").unwrap_or(val);
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

        if prefix.contains("Tags") || prefix.contains('"') {
            for (idx, (tag, desc)) in docs::SHADERLAB_TAGS.iter().enumerate() {
                sl_items.push(json!({
                    "label": *tag,
                    "kind": 10,
                    "detail": *desc,
                    "insertText": *tag,
                    "sortText": format!("05_{:02}_{}", idx, tag),
                }));
            }
        }

        for (idx, kw) in docs::BUILTIN_KEYWORDS.iter().filter(|kw| !["float", "float4", "cbuffer", "struct", "return"].contains(kw)).enumerate() {
            sl_items.push(json!({
                "label": *kw,
                "kind": 14,
                "insertText": *kw,
                "sortText": format!("20_{:02}_{}", idx, kw),
            }));
        }

        let sl_snippets = [
            ("shader", "Unity ShaderLab Shader template", "Shader \"$1\"\n{\n    Properties\n    {\n        _MainTex (\"Texture\", 2D) = \"white\" {}\n    }\n    SubShader\n    {\n        Tags { \"RenderType\"=\"Opaque\" \"RenderPipeline\"=\"UniversalPipeline\" }\n        Pass\n        {\n            HLSLPROGRAM\n            #pragma vertex vert\n            #pragma fragment frag\n            $0\n            ENDHLSL\n        }\n    }\n}"),
            ("pass", "Unity ShaderLab Pass block", "Pass\n{\n    Name \"$1\"\n    HLSLPROGRAM\n    #pragma vertex vert\n    #pragma fragment frag\n    $0\n    ENDHLSL\n}"),
            ("properties", "Properties block", "Properties\n{\n    $0\n}"),
            ("subshader", "SubShader block", "SubShader\n{\n    $0\n}"),
        ];
        for (idx, (label, detail, snip)) in sl_snippets.iter().enumerate() {
            sl_items.push(json!({
                "label": *label,
                "kind": 15,
                "detail": *detail,
                "insertText": *snip,
                "insertTextFormat": 2,
                "sortText": format!("30_{:02}_{}", idx, label),
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
                if c.is_alphanumeric() || c == '_' || c == '.' {
                    expr_start = i;
                } else {
                    break;
                }
            }
            let expr = &before_dot[expr_start..];
            if !expr.is_empty() {
                let parts: Vec<&str> = expr.split('.').collect();
                let structs = signature::scan_struct_definitions(doc);

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
                }
            }
        }
    }

    // Semantic completion when typing after ':'
    if let Some(colon_idx) = prefix.rfind(':') {
        let after_colon = prefix[colon_idx + 1..].trim();
        if !prefix.contains(';') && !prefix.contains('{') && after_colon.chars().all(|c| c.is_alphanumeric() || c == '_') {
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

    let (user_funcs, user_vars) = signature::resolve_includes_and_scan_symbols(uri, doc, doc_cache);

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
                "sortText": format!("15_{:03}_{}", idx, f.name),
            }));
        }
    }

    // 3. User structs (Types!)
    let structs = signature::scan_struct_definitions(doc);
    for (idx, s) in structs.iter().enumerate() {
        if seen_labels.insert(s.name.clone()) {
            items.push(json!({
                "label": s.name,
                "kind": 7,
                "detail": format!("struct {}", s.name),
                "insertText": s.name,
                "sortText": format!("20_{:02}_{}", idx, s.name),
            }));
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
                                "triggerCharacters": [".", "(", ">", ":", "\"", " ", "["],
                                "resolveProvider": false
                            },
                            "signatureHelpProvider": {
                                "triggerCharacters": ["(", ","]
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

                    let doc = doc_cache.get(uri).map(|s| s.as_str()).unwrap_or("");
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

                    let doc = doc_cache.get(uri).map(|s| s.as_str()).unwrap_or("");
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
                    let doc = doc_cache.get(uri).map(|s| s.as_str()).unwrap_or("");
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

                    let doc = doc_cache.get(uri).map(|s| s.as_str()).unwrap_or("");
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

        // 3. Unreal Engine USF Sample
        if let Ok(unreal_code) = std::fs::read_to_string("../samples/unreal_sample.usf") {
            let context = detect_shader_context("file:///unreal_sample.usf", &unreal_code);
            assert_eq!(context, ShaderContext::UnrealEngine);
            let diags = validate_shader("file:///unreal_sample.usf", &unreal_code, &dxc, None);
            assert!(diags.is_empty(), "Sample Unreal shader MUST have 0 errors, got: {:?}", diags);
        }
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
}
