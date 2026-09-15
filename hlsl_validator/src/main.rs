// hlsl_validator - main.rs
// High-performance HLSL and Unity ShaderLab Language Server powered by Microsoft DXC

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
    pub severity: u8, // 1 = Error, 2 = Warning, 3 = Info, 4 = Hint
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

/// Resolves the candidate binary path for `dxc`.
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

/// Discovers include directories for DXC by inspecting the file path, parent folders, and workspace root.
pub fn discover_include_paths(uri: &str, workspace_root: Option<&Path>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(file_path) = uri_to_path(uri) {
        if let Some(parent) = file_path.parent() {
            paths.push(parent.to_path_buf());

            // Walk up to discover project markers
            let mut current = parent;
            while let Some(up) = current.parent() {
                // Unity project structure: Assets & ProjectSettings
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
                // Unreal project structure: Source or *.uproject
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

/// Checks if target_line is inside a Unity HLSLPROGRAM/CGPROGRAM block.
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

/// Runs Microsoft DXC and extracts diagnostics.
pub fn validate_shader(
    uri: &str,
    content: &str,
    dxc_path: &str,
    workspace_root: Option<&Path>,
) -> Vec<Diagnostic> {
    let is_shaderlab = uri.ends_with(".shader") || uri.ends_with(".cginc");
    let mut diagnostics = Vec::new();
    let include_dirs = discover_include_paths(uri, workspace_root);

    if is_shaderlab {
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
                    let block_hlsl = block_lines.join("\n");
                    let block_diags = run_dxc_on_text(&block_hlsl, dxc_path, block_start_line, &include_dirs);
                    diagnostics.extend(block_diags);
                }
                continue;
            }

            if in_block {
                block_lines.push(line);
            }
        }
    } else {
        diagnostics = run_dxc_on_text(content, dxc_path, 0, &include_dirs);
    }

    diagnostics
}

static TEMP_FILE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Writes content to a temporary file and runs DXC with `-T lib_6_3 -HV 2021` and `-I` search paths.
pub fn run_dxc_on_text(
    content: &str,
    dxc_path: &str,
    line_offset: usize,
    include_dirs: &[PathBuf],
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
                let actual_line = parsed_line.saturating_sub(1) + line_offset;
                let actual_col = parsed_col.saturating_sub(1);

                let (severity, message) = if rest.contains("error:") {
                    let raw_msg = rest.split("error:").nth(1).unwrap_or(rest).trim();
                    if raw_msg.contains("file not found")
                        && (raw_msg.contains("Packages/") || raw_msg.contains("UnityCG") || raw_msg.contains("Engine/"))
                    {
                        (2, format!("Include file not found: {raw_msg}. (Check project root or include paths in settings)"))
                    } else {
                        (1, raw_msg.to_string())
                    }
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

/// Formats HLSL and ShaderLab source code with proper brace indentation and space normalization.
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

    let is_shaderlab = uri.ends_with(".shader") || uri.ends_with(".cginc");
    let in_hlsl = !is_shaderlab || is_inside_hlsl_block(doc, line_idx);

    // ------------------------------------------------------------------------
    // ShaderLab Contextual Completions (outside HLSL blocks)
    // ------------------------------------------------------------------------
    if !in_hlsl {
        let trimmed_prefix = prefix.trim();
        let mut sl_items = Vec::new();

        // 1. Render State completions
        if trimmed_prefix.starts_with("Blend") {
            for (idx, (_, val, desc)) in docs::SHADERLAB_RENDER_STATES.iter().filter(|(s, _, _)| *s == "Blend").enumerate() {
                let insert = val.strip_prefix("Blend ").unwrap_or(val);
                sl_items.push(json!({
                    "label": *val,
                    "kind": 12, // Value
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

        // 2. Tags completion
        if prefix.contains("Tags") || prefix.contains('"') {
            for (idx, (tag, desc)) in docs::SHADERLAB_TAGS.iter().enumerate() {
                sl_items.push(json!({
                    "label": *tag,
                    "kind": 10, // Property
                    "detail": *desc,
                    "insertText": *tag,
                    "sortText": format!("05_{:02}_{}", idx, tag),
                }));
            }
        }

        // 3. ShaderLab Property types
        for (idx, (prop_type, snip, desc)) in docs::SHADERLAB_PROPERTY_TYPES.iter().enumerate() {
            sl_items.push(json!({
                "label": *prop_type,
                "kind": 7, // Type
                "detail": *desc,
                "insertText": *snip,
                "sortText": format!("10_{:02}_{}", idx, prop_type),
            }));
        }

        // 4. ShaderLab Keywords
        for (idx, kw) in docs::BUILTIN_KEYWORDS.iter().enumerate() {
            sl_items.push(json!({
                "label": *kw,
                "kind": 14, // Keyword
                "insertText": *kw,
                "sortText": format!("20_{:02}_{}", idx, kw),
            }));
        }

        // 5. ShaderLab Snippets
        let sl_snippets = [
            ("shader", "Unity ShaderLab Shader template", "Shader \"$1\"\n{\n    Properties\n    {\n        _MainTex (\"Texture\", 2D) = \"white\" {}\n    }\n    SubShader\n    {\n        Tags { \"RenderType\"=\"Opaque\" \"RenderPipeline\"=\"UniversalPipeline\" }\n        Pass\n        {\n            HLSLPROGRAM\n            #pragma vertex vert\n            #pragma fragment frag\n            $0\n            ENDHLSL\n        }\n    }\n}"),
            ("pass", "Unity ShaderLab Pass block", "Pass\n{\n    Name \"$1\"\n    HLSLPROGRAM\n    #pragma vertex vert\n    #pragma fragment frag\n    $0\n    ENDHLSL\n}"),
            ("properties", "Properties block", "Properties\n{\n    $0\n}"),
            ("subshader", "SubShader block", "SubShader\n{\n    $0\n}"),
        ];
        for (idx, (label, detail, snip)) in sl_snippets.iter().enumerate() {
            sl_items.push(json!({
                "label": *label,
                "kind": 15, // Snippet
                "detail": *detail,
                "insertText": *snip,
                "insertTextFormat": 2,
                "sortText": format!("30_{:02}_{}", idx, label),
            }));
        }

        return json!(sl_items);
    }

    // ------------------------------------------------------------------------
    // Pure HLSL & HLSLPROGRAM Context
    // ------------------------------------------------------------------------

    // Member access completions: expr.field or expr.partial (e.g. "output.po" or "output.")
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
                        // 1. Struct or cbuffer member fields
                        if let Some(s_def) = structs.iter().find(|s| s.name == target_type) {
                            let field_items: Vec<Value> = s_def.fields.iter().enumerate().map(|(idx, f)| {
                                json!({
                                    "label": f.name,
                                    "kind": 5, // Field
                                    "detail": format!("{} {}.{}", f.field_type, s_def.name, f.name),
                                    "insertText": f.name,
                                    "sortText": format!("00_{:02}_{}", idx, f.name),
                                })
                            }).collect();
                            return json!(field_items);
                        }

                        // 2. Texture object methods (.Sample, .SampleLevel, .Load, etc.)
                        if target_type.starts_with("Texture") {
                            let method_items: Vec<Value> = docs::TEXTURE_METHODS.iter().enumerate().map(|(idx, m)| {
                                json!({
                                    "label": m.name,
                                    "kind": 2, // Method
                                    "detail": m.signature,
                                    "documentation": {
                                        "kind": "markdown",
                                        "value": m.description
                                    },
                                    "insertText": m.snippet,
                                    "insertTextFormat": 2,
                                    "sortText": format!("00_{:02}_{}", idx, m.name),
                                })
                            }).collect();
                            return json!(method_items);
                        }

                        // 3. Buffer object methods (.Load, .GetDimensions)
                        if target_type.contains("Buffer") {
                            let method_items: Vec<Value> = docs::BUFFER_METHODS.iter().enumerate().map(|(idx, m)| {
                                json!({
                                    "label": m.name,
                                    "kind": 2, // Method
                                    "detail": m.signature,
                                    "documentation": {
                                        "kind": "markdown",
                                        "value": m.description
                                    },
                                    "insertText": m.snippet,
                                    "insertTextFormat": 2,
                                    "sortText": format!("00_{:02}_{}", idx, m.name),
                                })
                            }).collect();
                            return json!(method_items);
                        }

                        // 4. Vector swizzles
                        let is_vec = target_type.starts_with("float")
                            || target_type.starts_with("half")
                            || target_type.starts_with("int")
                            || target_type.starts_with("uint");

                        if is_vec {
                            let swizzles = [
                                "x", "y", "z", "w",
                                "xy", "xyz", "xyzw",
                                "r", "g", "b", "a",
                                "rgb", "rgba",
                            ];
                            let items: Vec<Value> = swizzles.iter().enumerate().map(|(idx, sw)| {
                                json!({
                                    "label": *sw,
                                    "kind": 10, // Property
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

    // Semantic completion when typing after ':' (e.g. "float4 pos : SV_")
    if let Some(colon_idx) = prefix.rfind(':') {
        let after_colon = prefix[colon_idx + 1..].trim();
        if !prefix.contains(';') && after_colon.chars().all(|c| c.is_alphanumeric() || c == '_') {
            let semantic_items: Vec<Value> = docs::BUILTIN_VARIABLES.iter().enumerate().map(|(idx, (sem, desc))| {
                json!({
                    "label": *sem,
                    "kind": 6, // Variable
                    "detail": "HLSL Semantic",
                    "documentation": {
                        "kind": "markdown",
                        "value": *desc
                    },
                    "insertText": *sem,
                    "sortText": format!("00_{:02}_{}", idx, sem),
                })
            }).collect();
            return json!(semantic_items);
        }
    }

    let mut items = Vec::new();

    // 1. User-defined symbols and included files
    let (user_funcs, user_vars) = signature::resolve_includes_and_scan_symbols(uri, doc, doc_cache);
    for (idx, v) in user_vars.iter().enumerate() {
        items.push(json!({
            "label": v.name,
            "kind": 6, // Variable
            "detail": format!("{} {}", v.var_type, v.name),
            "documentation": v.doc.as_deref().unwrap_or("User variable"),
            "insertText": v.name,
            "sortText": format!("10_{:03}_{}", idx, v.name),
        }));
    }

    for (idx, f) in user_funcs.iter().enumerate() {
        items.push(json!({
            "label": f.name,
            "kind": 3, // Function
            "detail": f.label,
            "documentation": f.doc.as_deref().unwrap_or("User function"),
            "insertText": format!("{}($1)", f.name),
            "insertTextFormat": 2,
            "sortText": format!("15_{:03}_{}", idx, f.name),
        }));
    }

    // 2. User-defined structs
    let structs = signature::scan_struct_definitions(doc);
    for (idx, s) in structs.iter().enumerate() {
        items.push(json!({
            "label": s.name,
            "kind": 7, // Struct / Class
            "detail": format!("struct {}", s.name),
            "insertText": s.name,
            "sortText": format!("20_{:02}_{}", idx, s.name),
        }));
    }

    // 3. Built-in HLSL Intrinsics (Microsoft reference)
    for (idx, func) in docs::BUILTIN_FUNCTIONS.iter().enumerate() {
        let primary_overload = func.overloads.first().map(|o| o.label).unwrap_or(func.name);
        items.push(json!({
            "label": func.name,
            "kind": 3, // Function
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

    // 4. Built-in Types
    for (idx, t) in docs::BUILTIN_TYPES.iter().enumerate() {
        items.push(json!({
            "label": *t,
            "kind": 7, // Class / Type
            "detail": "HLSL Type",
            "insertText": *t,
            "sortText": format!("35_{:03}_{}", idx, t),
        }));
    }

    // 5. Built-in Keywords
    for (idx, kw) in docs::BUILTIN_KEYWORDS.iter().enumerate() {
        items.push(json!({
            "label": *kw,
            "kind": 14, // Keyword
            "insertText": *kw,
            "sortText": format!("40_{:03}_{}", idx, kw),
        }));
    }

    // 6. Production Code Snippets
    let snippets = [
        ("vert", "Vertex Shader function", "Varyings vert(Attributes input)\n{\n    Varyings output = (Varyings)0;\n    output.positionCS = TransformObjectToHClip(input.positionOS.xyz);\n    $0\n    return output;\n}"),
        ("frag", "Fragment/Pixel Shader function", "float4 frag(Varyings input) : SV_Target\n{\n    $0\n    return float4(1.0, 1.0, 1.0, 1.0);\n}"),
        ("struct", "Struct declaration", "struct $1\n{\n    $0\n};"),
        ("cbuffer", "Constant Buffer declaration", "cbuffer $1\n{\n    $0\n};"),
    ];

    for (idx, (label, detail, snip)) in snippets.iter().enumerate() {
        items.push(json!({
            "label": *label,
            "kind": 15, // Snippet
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
    let doc_cache: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let workspace_root: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(
        env::var("WORKSPACE_ROOT").ok().map(PathBuf::from),
    ));

    // Worker Thread: 120ms debounce queue
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

    // LSP JSON-RPC Dispatch Loop
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    loop {
        let mut line = String::new();
        let mut content_length: Option<usize> = None;

        loop {
            line.clear();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                return; // EOF
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
                                "triggerCharacters": [".", "(", ">", ":", "\"", " "],
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

                    if let Ok(mut cache) = doc_cache.lock() {
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
                            if let Ok(mut cache) = doc_cache.lock() {
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
                    if let Ok(mut cache) = doc_cache.lock() {
                        cache.remove(uri);
                    }
                    send_diagnostics(uri, &[]);
                }
            }
            "textDocument/completion" => {
                if let Some(id) = id {
                    let cache = doc_cache.lock().map(|c| c.clone()).unwrap_or_default();
                    let res = handle_completion(&msg, &cache);
                    write_lsp_response(id, res);
                }
            }
            "textDocument/signatureHelp" => {
                if let Some(id) = id {
                    let params = &msg["params"];
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    let line = params["position"]["line"].as_u64().unwrap_or(0) as usize;
                    let col = params["position"]["character"].as_u64().unwrap_or(0) as usize;

                    let cache = doc_cache.lock().map(|c| c.clone()).unwrap_or_default();
                    let doc = cache.get(uri).cloned().unwrap_or_default();
                    let res = signature::get_signature_help(uri, &doc, line, col, &cache);
                    write_lsp_response(id, res);
                }
            }
            "textDocument/hover" => {
                if let Some(id) = id {
                    let params = &msg["params"];
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    let line = params["position"]["line"].as_u64().unwrap_or(0) as usize;
                    let col = params["position"]["character"].as_u64().unwrap_or(0) as usize;

                    let cache = doc_cache.lock().map(|c| c.clone()).unwrap_or_default();
                    let doc = cache.get(uri).cloned().unwrap_or_default();
                    let res = signature::get_hover_info(uri, &doc, line, col, &cache);
                    write_lsp_response(id, res);
                }
            }
            "textDocument/definition" => {
                if let Some(id) = id {
                    let cache = doc_cache.lock().map(|c| c.clone()).unwrap_or_default();
                    let res = signature::handle_definition(&msg, &cache);
                    write_lsp_response(id, res);
                }
            }
            "textDocument/documentSymbol" => {
                if let Some(id) = id {
                    let params = &msg["params"];
                    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                    let cache = doc_cache.lock().map(|c| c.clone()).unwrap_or_default();
                    let doc = cache.get(uri).cloned().unwrap_or_default();
                    let symbols = signature::get_document_symbols(&doc);
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

                    let cache = doc_cache.lock().map(|c| c.clone()).unwrap_or_default();
                    let doc = cache.get(uri).cloned().unwrap_or_default();
                    let edits = format_document(&doc, tab_size, insert_spaces);
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
// Object attributes
struct Attributes {
    float3 positionOS : POSITION;
    float2 uv : TEXCOORD0;
};

// Vertex shader
float4 MyVertShader(Attributes input) : SV_Position {
    return float4(input.positionOS, 1.0);
}
"#;
        let funcs = signature::scan_user_functions(code, None, None);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "MyVertShader");
        assert_eq!(funcs[0].parameters.len(), 1);

        let vars = signature::scan_user_variables(code, None, None);
        assert!(vars.iter().any(|v| v.name == "Attributes" && v.var_type == "struct"));
    }

    #[test]
    fn test_validate_shader_dxc_valid() {
        let dxc = find_dxc_path();
        let valid_shader = r#"
float4 MainVs(float3 pos : POSITION) : SV_Position {
    return float4(pos, 1.0);
}
"#;
        let diags = validate_shader("file:///test.hlsl", valid_shader, &dxc, None);
        assert!(diags.is_empty(), "Expected 0 diagnostics for valid shader, got: {:?}", diags);
    }

    #[test]
    fn test_validate_shader_dxc_error() {
        let dxc = find_dxc_path();
        let invalid_shader = r#"
float4 MainVs(float3 pos : POSITION) : SV_Position {
    return float4(pos, undefined_variable);
}
"#;
        let diags = validate_shader("file:///test.hlsl", invalid_shader, &dxc, None);
        assert!(!diags.is_empty(), "Expected diagnostics for undefined variable error");
        assert!(diags.iter().any(|d| d.message.contains("undefined_variable") || d.severity == 1));
    }

    #[test]
    fn test_shaderlab_block_extraction() {
        let dxc = find_dxc_path();
        let shaderlab = r#"
Shader "Custom/TestShader"
{
    SubShader
    {
        Pass
        {
            HLSLPROGRAM
            float4 MainVs(float3 pos : POSITION) : SV_Position {
                return float4(pos, 1.0);
            }
            ENDHLSL
        }
    }
}
"#;
        let diags = validate_shader("file:///test.shader", shaderlab, &dxc, None);
        assert!(diags.is_empty(), "Expected 0 diagnostics for valid ShaderLab, got: {:?}", diags);
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
    fn test_discover_include_paths() {
        let uri = "file:///C:/MyProject/Assets/Shaders/Lit.shader";
        let paths = discover_include_paths(uri, None);
        assert!(!paths.is_empty());
        assert_eq!(paths[0], PathBuf::from("C:/MyProject/Assets/Shaders"));
    }

    #[test]
    fn test_shaderlab_contextual_completion() {
        let mut cache = HashMap::new();
        let shaderlab = r#"
Shader "Test"
{
    SubShader
    {
        Pass
        {
            Blend 
        }
    }
}
"#;
        cache.insert("file:///test.shader".to_string(), shaderlab.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.shader" },
                "position": { "line": 7, "character": 18 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"].as_str().unwrap().contains("SrcAlpha")));
    }
}
