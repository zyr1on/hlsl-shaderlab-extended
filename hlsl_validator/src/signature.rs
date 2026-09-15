// hlsl_validator - signature.rs
// Signature Help, Hover, and Symbol Scanner for HLSL & Unity ShaderLab

use crate::docs;
use crate::{path_to_uri, uri_to_path};
use serde_json::{json, Value};
use std::collections::HashMap;

const INVALID_TYPES: &[&str] = &[
    "return", "else", "case", "default", "discard", "break", "continue",
    "goto", "packoffset", "register",
];

const INVALID_NAMES: &[&str] = &[
    "if", "for", "while", "switch", "return", "struct", "cbuffer", "tbuffer",
];

const CONTROL_KEYWORDS: &[&str] = &["if", "for", "while", "switch", "catch", "return"];

#[allow(dead_code)]
pub const STORAGE_QUALIFIERS: &[&str] = &[
    "static", "const", "inline", "in", "out", "inout", "uniform",
    "column_major", "row_major", "precise", "groupshared",
];

#[allow(dead_code)]
pub const KNOWN_BASE_TYPES: &[&str] = &[
    "float", "float2", "float3", "float4",
    "half", "half2", "half3", "half4",
    "int", "int2", "int3", "int4",
    "uint", "uint2", "uint3", "uint4",
    "bool", "bool2", "bool3", "bool4",
    "double",
    "float4x4", "float3x3", "float2x2",
    "half4x4", "half3x3", "matrix",
    "Texture2D", "Texture2DArray", "Texture3D", "TextureCube",
    "SamplerState", "SamplerComparisonState",
    "sampler2D", "samplerCUBE",
    "cbuffer", "tbuffer",
    "StructuredBuffer", "RWStructuredBuffer",
    "ByteAddressBuffer", "RWByteAddressBuffer",
];

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VariableSymbol {
    pub name: String,
    pub var_type: String,
    pub qualifier: String,
    pub doc: Option<String>,
    pub source: Option<String>,
    pub line: usize,
    pub col: usize,
    pub file_uri: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam {
    pub name: String,
    pub param_type: String,
    pub line: usize,
    pub col: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct LocalVar {
    pub name: String,
    pub var_type: String,
    pub line: usize,
    pub col: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub return_type: String,
    pub label: String,
    pub parameters: Vec<String>,
    pub parsed_params: Vec<FunctionParam>,
    pub local_vars: Vec<LocalVar>,
    pub body_start_line: usize,
    pub body_end_line: usize,
    pub doc: Option<String>,
    pub source: Option<String>,
    pub line: usize,
    pub col: usize,
    pub file_uri: Option<String>,
}

/// Determines if a given cursor position (line, col) is inside a comment or string literal.
pub fn is_in_comment_or_string(text: &str, target_line: usize, target_col: usize) -> bool {
    let mut line_idx = 0;
    let mut col_idx = 0;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut escaped = false;

    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if line_idx == target_line && col_idx >= target_col {
            return in_line_comment || in_block_comment || in_string;
        }

        if c == '\n' {
            in_line_comment = false;
            line_idx += 1;
            col_idx = 0;
            if line_idx > target_line {
                return false;
            }
            continue;
        }

        col_idx += 1;

        if in_line_comment {
            continue;
        }

        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                col_idx += 1;
                in_block_comment = false;
            }
            continue;
        }

        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        match c {
            '"' => in_string = true,
            '/' => {
                if chars.peek() == Some(&'/') {
                    chars.next();
                    col_idx += 1;
                    in_line_comment = true;
                } else if chars.peek() == Some(&'*') {
                    chars.next();
                    col_idx += 1;
                    in_block_comment = true;
                }
            }
            _ => {}
        }
    }

    if line_idx == target_line {
        in_line_comment || in_block_comment || in_string
    } else {
        false
    }
}

/// Identifies the function call surrounding the cursor and the active parameter index.
pub fn find_enclosing_call(text: &str, line_idx: usize, col_idx: usize) -> Option<(String, usize)> {
    let mut current_line = 0;
    let mut offset = text.len();
    let mut line_start = 0;

    for (idx, b) in text.bytes().enumerate() {
        if current_line == line_idx {
            line_start = idx;
            break;
        }
        if b == b'\n' {
            current_line += 1;
        }
    }

    if current_line == line_idx {
        let line_slice = &text[line_start..];
        let raw_line_len = line_slice.find('\n').unwrap_or(line_slice.len());
        let line_len = line_slice[..raw_line_len].trim_end_matches('\r').len();
        offset = line_start + col_idx.min(line_len);
    }

    let bytes = text.as_bytes();
    let mut depth = 0;
    let mut bracket_depth = 0;
    let mut brace_depth = 0;
    let mut open_paren_idx = None;

    let mut idx = offset;
    while idx > 0 {
        idx -= 1;
        let b = bytes[idx];

        if b == b'"' {
            while idx > 0 {
                idx -= 1;
                if bytes[idx] == b'"' && (idx == 0 || bytes[idx - 1] != b'\\') {
                    break;
                }
            }
            continue;
        }

        match b {
            b')' => depth += 1,
            b']' => bracket_depth += 1,
            b'}' => brace_depth += 1,
            b'(' => {
                if depth > 0 {
                    depth -= 1;
                } else if bracket_depth == 0 && brace_depth == 0 {
                    open_paren_idx = Some(idx);
                    break;
                }
            }
            b'[' => {
                if bracket_depth > 0 {
                    bracket_depth -= 1;
                }
            }
            b'{' => {
                if brace_depth > 0 {
                    brace_depth -= 1;
                } else {
                    break;
                }
            }
            b';' if depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                break;
            }
            _ => {}
        }
    }

    let open_idx = open_paren_idx?;
    if offset <= open_idx {
        return None;
    }

    let before_paren = &text[..open_idx];
    let trimmed = before_paren.trim_end();
    let mut ident_start = trimmed.len();
    for (i, c) in trimmed.char_indices().rev() {
        if c.is_alphanumeric() || c == '_' {
            ident_start = i;
        } else {
            break;
        }
    }

    let fn_name = trimmed[ident_start..].trim();
    if fn_name.is_empty() || CONTROL_KEYWORDS.contains(&fn_name) {
        return None;
    }

    let mut param_index = 0;
    let mut p_depth: usize = 0;
    let mut p_bracket_depth: usize = 0;
    let mut p_brace_depth: usize = 0;
    let mut p_in_str = false;

    let call_args = &bytes[open_idx + 1..offset];
    let mut i = 0;
    while i < call_args.len() {
        let b = call_args[i];
        if p_in_str {
            if b == b'\\' {
                i += 1;
            } else if b == b'"' {
                p_in_str = false;
            }
            i += 1;
            continue;
        }

        match b {
            b'"' => p_in_str = true,
            b'(' => p_depth += 1,
            b')' => p_depth = p_depth.saturating_sub(1),
            b'[' => p_bracket_depth += 1,
            b']' => p_bracket_depth = p_bracket_depth.saturating_sub(1),
            b'{' => p_brace_depth += 1,
            b'}' => p_brace_depth = p_brace_depth.saturating_sub(1),
            b',' if p_depth == 0 && p_bracket_depth == 0 && p_brace_depth == 0 => {
                param_index += 1;
            }
            _ => {}
        }
        i += 1;
    }

    Some((fn_name.to_string(), param_index))
}

pub fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) if first.is_alphabetic() || first == '_' => {
            chars.all(|c| c.is_alphanumeric() || c == '_')
        }
        _ => false,
    }
}

pub fn parse_parameter_decl(raw: &str, line_idx: usize, base_col: usize) -> Option<FunctionParam> {
    let clean = raw.trim();
    if clean.is_empty() || clean == "void" {
        return None;
    }
    let without_def = clean.split('=').next()?.trim();
    let without_sem = without_def.split(':').next()?.trim();

    let tokens: Vec<&str> = without_sem.split_whitespace().collect();
    const QUALIFIERS: &[&str] = &[
        "in", "out", "inout", "uniform", "const", "linear", "centroid",
        "nointerpolation", "noperspective", "sample", "precise", "point",
        "row_major", "column_major", "globallycoherent"
    ];
    let filtered: Vec<&str> = tokens.into_iter()
        .filter(|t| !QUALIFIERS.contains(t))
        .collect();

    if filtered.len() >= 2 {
        let name_raw = filtered.last()?;
        let name = name_raw.trim_matches(|c: char| c == '[' || c == ']');
        let p_type = filtered[..filtered.len() - 1].join(" ");
        if is_valid_identifier(name) && !INVALID_NAMES.contains(&name) {
            let col = raw.find(name).map(|c| base_col + c).unwrap_or(base_col);
            return Some(FunctionParam {
                name: name.to_string(),
                param_type: p_type,
                line: line_idx,
                col,
            });
        }
    } else if filtered.len() == 1 {
        let name = filtered[0];
        if is_valid_identifier(name) && !INVALID_NAMES.contains(&name) {
            let col = raw.find(name).map(|c| base_col + c).unwrap_or(base_col);
            return Some(FunctionParam {
                name: name.to_string(),
                param_type: "var".to_string(),
                line: line_idx,
                col,
            });
        }
    }
    None
}

pub fn parse_local_var_decl(line: &str, line_idx: usize) -> Option<LocalVar> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
        return None;
    }
    const CONTROL_KWS: &[&str] = &[
        "return", "break", "continue", "discard", "if", "else", "while", "do", "switch", "case", "default"
    ];
    let first_word = trimmed.split_whitespace().next().unwrap_or("");
    if CONTROL_KWS.contains(&first_word) {
        return None;
    }

    if !trimmed.ends_with(';') && !trimmed.contains('=') {
        return None;
    }

    let clean = trimmed.trim_end_matches(';').trim();
    let decl_part = clean.split('=').next()?.trim();
    let tokens: Vec<&str> = decl_part.split_whitespace().collect();
    if tokens.len() >= 2 {
        let name_raw = tokens.last()?;
        let name = name_raw.split([':', '[']).next()?.trim();
        let type_part = tokens[tokens.len() - 2].split([':', '<']).next()?.trim();

        if is_valid_identifier(name)
            && !INVALID_NAMES.contains(&name)
            && is_valid_identifier(type_part)
            && !INVALID_TYPES.contains(&type_part)
            && !type_part.contains('.')
            && !name.contains('.')
        {
            let col = line.find(name).unwrap_or(0);
            return Some(LocalVar {
                name: name.to_string(),
                var_type: type_part.to_string(),
                line: line_idx,
                col,
            });
        }
    }
    None
}

pub fn find_enclosing_function(
    funcs: &[FunctionSignature],
    line_idx: usize,
) -> Option<&FunctionSignature> {
    funcs.iter().find(|f| line_idx >= f.line && line_idx <= f.body_end_line)
}

/// Parses function signatures, their parameters, and local variables from HLSL and ShaderLab source text.
pub fn scan_user_functions(
    text: &str,
    source_name: Option<&str>,
    file_uri: Option<&str>,
) -> Vec<FunctionSignature> {
    let mut results = Vec::new();
    let mut pending_doc = Vec::new();
    let mut in_func_body = false;
    let mut expecting_body = false;
    let mut func_brace_depth: usize = 0;
    let mut pending_header: Option<(String, String, usize, usize, String)> = None;

    for (line_idx, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();

        if line.starts_with("//") {
            let doc_line = line.trim_start_matches('/').trim();
            if !in_func_body && !doc_line.is_empty() {
                pending_doc.push(doc_line.to_string());
            }
            continue;
        }

        if line.is_empty() {
            if !in_func_body && !expecting_body && pending_header.is_none() {
                pending_doc.clear();
            }
            continue;
        }

        // Multi-line function header continuation
        if let Some((fn_name, ret_type, start_line, col_idx, mut acc)) = pending_header.take() {
            acc.push(' ');
            acc.push_str(line);
            if let Some(close_idx) = acc.find(')') {
                let open_paren = acc.find('(').unwrap_or(0);
                let header = &acc[..=close_idx];
                let params_part = &acc[open_paren + 1..close_idx];
                let params: Vec<String> = params_part
                    .split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty() && p != "void")
                    .collect();

                let mut parsed_params = Vec::new();
                for p in &params {
                    if let Some(mut fp) = parse_parameter_decl(p, start_line, 0) {
                        fp.col = text.lines().nth(fp.line).and_then(|l| l.find(&fp.name)).unwrap_or(0);
                        parsed_params.push(fp);
                    }
                }

                let doc = if pending_doc.is_empty() { None } else { Some(pending_doc.join(" ")) };
                results.push(FunctionSignature {
                    name: fn_name,
                    return_type: ret_type,
                    label: header.to_string(),
                    parameters: params,
                    parsed_params,
                    local_vars: Vec::new(),
                    body_start_line: start_line,
                    body_end_line: start_line,
                    doc,
                    source: source_name.map(|s| s.to_string()),
                    line: start_line,
                    col: col_idx,
                    file_uri: file_uri.map(|s| s.to_string()),
                });
                pending_doc.clear();

                let rest = &acc[close_idx..];
                let open_b = rest.chars().filter(|&c| c == '{').count();
                let close_b = rest.chars().filter(|&c| c == '}').count();
                if open_b > close_b {
                    in_func_body = true;
                    func_brace_depth = open_b - close_b;
                    if let Some(f) = results.last_mut() {
                        f.body_start_line = line_idx;
                    }
                } else if open_b == 0 && !acc.ends_with(';') {
                    expecting_body = true;
                }
            } else {
                pending_header = Some((fn_name, ret_type, start_line, col_idx, acc));
            }
            continue;
        }

        if expecting_body {
            let open_b = line.chars().filter(|&c| c == '{').count();
            let close_b = line.chars().filter(|&c| c == '}').count();
            if open_b > 0 {
                expecting_body = false;
                if let Some(f) = results.last_mut() {
                    f.body_start_line = line_idx;
                }
                if open_b > close_b {
                    in_func_body = true;
                    func_brace_depth = open_b - close_b;
                }
                continue;
            } else if line.ends_with(';') {
                expecting_body = false;
            }
        }

        if !in_func_body && !line.starts_with("return") && !line.starts_with('#') {
            if let Some(open_paren) = line.find('(') {
                let before = line[..open_paren].trim();
                let mut it = before.split_whitespace().rev();
                if let (Some(fn_name), Some(return_type)) = (it.next(), it.next()) {
                    if is_valid_identifier(fn_name)
                        && !INVALID_NAMES.contains(&fn_name)
                        && !INVALID_TYPES.contains(&return_type)
                    {
                        let col_idx = raw_line.find(fn_name).unwrap_or(0);
                        if let Some(close_idx) = line.find(')') {
                            let header = &line[..=close_idx];
                            let params_part = &line[open_paren + 1..close_idx];
                            let params: Vec<String> = params_part
                                .split(',')
                                .map(|p| p.trim().to_string())
                                .filter(|p| !p.is_empty() && p != "void")
                                .collect();

                            let mut parsed_params = Vec::new();
                            for p in &params {
                                if let Some(mut fp) = parse_parameter_decl(p, line_idx, open_paren + 1) {
                                    fp.col = raw_line.find(&fp.name).unwrap_or(open_paren + 1);
                                    parsed_params.push(fp);
                                }
                            }

                            let doc = if pending_doc.is_empty() {
                                None
                            } else {
                                Some(pending_doc.join(" "))
                            };

                            results.push(FunctionSignature {
                                name: fn_name.to_string(),
                                return_type: return_type.to_string(),
                                label: header.to_string(),
                                parameters: params,
                                parsed_params,
                                local_vars: Vec::new(),
                                body_start_line: line_idx,
                                body_end_line: line_idx,
                                doc,
                                source: source_name.map(|s| s.to_string()),
                                line: line_idx,
                                col: col_idx,
                                file_uri: file_uri.map(|s| s.to_string()),
                            });
                            pending_doc.clear();

                            let rest = &line[close_idx..];
                            let open_b = rest.chars().filter(|&c| c == '{').count();
                            let close_b = rest.chars().filter(|&c| c == '}').count();
                            if open_b > close_b {
                                in_func_body = true;
                                func_brace_depth = open_b - close_b;
                            } else if open_b == 0 && !line.ends_with(';') {
                                expecting_body = true;
                            }
                        } else {
                            // Multi-line header detected
                            pending_header = Some((
                                fn_name.to_string(),
                                return_type.to_string(),
                                line_idx,
                                col_idx,
                                line.to_string(),
                            ));
                        }
                    }
                }
            }
        } else if in_func_body {
            if let Some(lv) = parse_local_var_decl(raw_line, line_idx) {
                if let Some(f) = results.last_mut() {
                    f.local_vars.push(lv);
                }
            }

            let open_b = line.chars().filter(|&c| c == '{').count();
            let close_b = line.chars().filter(|&c| c == '}').count();
            func_brace_depth += open_b;
            if func_brace_depth <= close_b {
                in_func_body = false;
                func_brace_depth = 0;
                if let Some(f) = results.last_mut() {
                    f.body_end_line = line_idx;
                }
            } else {
                func_brace_depth -= close_b;
            }
        }
    }

    results
}

/// Parses global user variables and cbuffer members from HLSL source text.
pub fn scan_user_variables(
    text: &str,
    source_name: Option<&str>,
    file_uri: Option<&str>,
) -> Vec<VariableSymbol> {
    let mut results = Vec::new();
    let mut pending_doc = Vec::new();
    let mut brace_level: usize = 0;
    let mut current_block: Option<(String, bool)> = None; // (name, is_cbuffer)

    for (line_idx, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();

        if line.starts_with("//") {
            let doc_line = line.trim_start_matches('/').trim();
            if !doc_line.is_empty() {
                pending_doc.push(doc_line.to_string());
            }
            continue;
        }

        if line.is_empty() {
            if brace_level == 0 {
                pending_doc.clear();
            }
            continue;
        }

        // cbuffer Name or struct Name
        if line.starts_with("cbuffer ") || line.starts_with("struct ") {
            let mut it = line.split_whitespace();
            let kw = it.next().unwrap_or("");
            if let Some(name_raw) = it.next() {
                let name = name_raw.split(['{', ':']).next().unwrap_or("").trim();
                if is_valid_identifier(name) {
                    current_block = Some((name.to_string(), kw == "cbuffer"));
                }
            }
        }

        // Variable declaration ending with semicolon
        if line.ends_with(';') && !line.starts_with('#') && !line.starts_with("return") {
            let in_cbuffer = current_block.as_ref().map(|(_, is_cb)| *is_cb).unwrap_or(false);
            // Only add variables declared at file root (brace_level == 0) or inside cbuffer
            if brace_level == 0 || in_cbuffer {
                let clean = line.trim_end_matches(';').trim();
                let decl = clean.split('=').next().unwrap_or("").trim();
                let tokens: Vec<&str> = decl.split_whitespace().collect();
                if tokens.len() >= 2 {
                    let var_name_raw = tokens.last().copied().unwrap_or("");
                    let var_name = var_name_raw.split([':', '[']).next().unwrap_or("").trim();
                    let var_type = tokens[tokens.len() - 2].split([':', '<']).next().unwrap_or("").trim();

                    if is_valid_identifier(var_name) && !INVALID_NAMES.contains(&var_name) {
                        let col = raw_line.find(var_name).unwrap_or(0);
                        let doc = if pending_doc.is_empty() { None } else { Some(pending_doc.join(" ")) };
                        let qualifier = if in_cbuffer {
                            current_block.as_ref().map(|(cb, _)| format!("cbuffer {cb}")).unwrap_or_default()
                        } else {
                            String::new()
                        };
                        results.push(VariableSymbol {
                            name: var_name.to_string(),
                            var_type: var_type.to_string(),
                            qualifier,
                            doc,
                            source: source_name.map(|s| s.to_string()),
                            line: line_idx,
                            col,
                            file_uri: file_uri.map(|s| s.to_string()),
                        });
                    }
                }
            }
        }

        for b in line.bytes() {
            if b == b'{' {
                brace_level += 1;
            } else if b == b'}' {
                brace_level = brace_level.saturating_sub(1);
                if brace_level == 0 {
                    current_block = None;
                }
            }
        }

        if line.ends_with(';') || line.ends_with('}') {
            pending_doc.clear();
        }
    }

    results
}

/// Resolves #include directives recursively and collects user functions & symbols.
pub fn resolve_includes_and_scan_symbols(
    uri: &str,
    doc_content: &str,
    doc_cache: &HashMap<String, String>,
) -> (Vec<FunctionSignature>, Vec<VariableSymbol>) {
    let mut all_functions = Vec::new();
    let mut all_variables = Vec::new();

    // Scan the current active document first
    all_functions.extend(scan_user_functions(doc_content, None, Some(uri)));
    all_variables.extend(scan_user_variables(doc_content, None, Some(uri)));

    // Parse #include directives
    let main_path = uri_to_path(uri);
    let main_dir = main_path.as_ref().and_then(|p| p.parent()).map(|p| p.to_path_buf());

    for line in doc_content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("#include") {
            let include_target = rest.trim().trim_matches(|c| c == '"' || c == '<' || c == '>');
            if include_target.is_empty() {
                continue;
            }

            // Try to resolve path
            if let Some(dir) = &main_dir {
                let candidate = dir.join(include_target);
                if candidate.is_file() {
                    let inc_uri = path_to_uri(&candidate);
                    let content = doc_cache
                        .get(&inc_uri)
                        .cloned()
                        .or_else(|| std::fs::read_to_string(&candidate).ok());

                    if let Some(inc_content) = content {
                        let inc_name = candidate.file_name().and_then(|n| n.to_str()).unwrap_or(include_target);
                        all_functions.extend(scan_user_functions(&inc_content, Some(inc_name), Some(&inc_uri)));
                        all_variables.extend(scan_user_variables(&inc_content, Some(inc_name), Some(&inc_uri)));
                    }
                }
            }
        }
    }

    (all_functions, all_variables)
}

/// Handles textDocument/signatureHelp requests.
pub fn get_signature_help(
    uri: &str,
    doc_content: &str,
    line_idx: usize,
    col_idx: usize,
    doc_cache: &HashMap<String, String>,
) -> Value {
    let (fn_name, active_param) = match find_enclosing_call(doc_content, line_idx, col_idx) {
        Some(res) => res,
        None => return json!(null),
    };

    // 0. ShaderLab Built-in Property Signatures (e.g. Range(min, max))
    if fn_name == "Range" {
        return json!({
            "signatures": [{
                "label": "Range(float min, float max)",
                "parameters": [
                    { "label": "float min" },
                    { "label": "float max" }
                ],
                "documentation": {
                    "kind": "markdown",
                    "value": "### `Range(min, max)`\n*Unity ShaderLab Property Type*\n\nCreates a floating-point property bounded by an interactive slider between `min` and `max` in the Unity Material Inspector."
                }
            }],
            "activeSignature": 0,
            "activeParameter": active_param
        });
    }

    // 1. Built-in HLSL Intrinsics (Microsoft reference)
    if let Some(builtin) = docs::find_builtin_function(&fn_name) {
        let signatures: Vec<Value> = builtin
            .overloads
            .iter()
            .map(|ol| {
                let params: Vec<Value> = ol.params.iter().map(|p| json!({ "label": *p })).collect();
                json!({
                    "label": ol.label,
                    "parameters": params,
                    "documentation": {
                        "kind": "markdown",
                        "value": builtin.description
                    }
                })
            })
            .collect();

        return json!({
            "signatures": signatures,
            "activeSignature": 0,
            "activeParameter": active_param
        });
    }

    // 2. User-defined functions
    let (user_funcs, _) = resolve_includes_and_scan_symbols(uri, doc_content, doc_cache);
    let matches: Vec<&FunctionSignature> = user_funcs.iter().filter(|f| f.name == fn_name).collect();

    if !matches.is_empty() {
        let signatures: Vec<Value> = matches
            .iter()
            .map(|f| {
                let params: Vec<Value> = f.parameters.iter().map(|p| json!({ "label": p })).collect();
                let doc_val = f.doc.as_deref().unwrap_or("");
                json!({
                    "label": f.label,
                    "parameters": params,
                    "documentation": {
                        "kind": "markdown",
                        "value": doc_val
                    }
                })
            })
            .collect();

        return json!({
            "signatures": signatures,
            "activeSignature": 0,
            "activeParameter": active_param
        });
    }

    json!(null)
}

/// Handles textDocument/hover requests.
pub fn get_hover_info(
    uri: &str,
    doc_content: &str,
    line_idx: usize,
    col_idx: usize,
    doc_cache: &HashMap<String, String>,
) -> Value {
    let line = match doc_content.lines().nth(line_idx) {
        Some(l) => l,
        None => return json!(null),
    };

    let safe_col = col_idx.min(line.len());
    let mut word_start = safe_col;
    for (i, c) in line[..safe_col].char_indices().rev() {
        if c.is_alphanumeric() || c == '_' {
            word_start = i;
        } else {
            break;
        }
    }

    let mut word_end = safe_col;
    for (i, c) in line[safe_col..].char_indices() {
        if c.is_alphanumeric() || c == '_' {
            word_end = safe_col + i + c.len_utf8();
        } else {
            break;
        }
    }

    let word = &line[word_start..word_end];
    if word.is_empty() {
        return json!(null);
    }

    // 1. Built-in HLSL Intrinsics (Microsoft reference)
    if let Some(builtin) = docs::find_builtin_function(word) {
        let mut overloads_str = String::new();
        for ol in builtin.overloads {
            overloads_str.push_str(ol.label);
            overloads_str.push('\n');
        }

        let markdown = format!(
            "```hlsl\n{}\n```\n\n{}",
            overloads_str.trim_end(),
            builtin.description
        );

        return json!({
            "contents": {
                "kind": "markdown",
                "value": markdown
            }
        });
    }

    // 2. Built-in Semantics
    if let Some((_, desc)) = docs::BUILTIN_VARIABLES.iter().find(|(name, _)| *name == word) {
        return json!({
            "contents": {
                "kind": "markdown",
                "value": format!("### `{word}`\n*HLSL Semantic*\n\n{desc}")
            }
        });
    }

    // 3. User-defined functions & variables in current scope (Parameters & Locals first)
    let (user_funcs, user_vars) = resolve_includes_and_scan_symbols(uri, doc_content, doc_cache);

    if let Some(f) = find_enclosing_function(&user_funcs, line_idx) {
        if let Some(p) = f.parsed_params.iter().find(|p| p.name == word) {
            return json!({
                "contents": {
                    "kind": "markdown",
                    "value": format!("```hlsl\n{} {}\n```\n*(parameter of `{}`)*", p.param_type, p.name, f.name)
                }
            });
        }
        if let Some(v) = f.local_vars.iter().find(|v| v.name == word) {
            return json!({
                "contents": {
                    "kind": "markdown",
                    "value": format!("```hlsl\n{} {}\n```\n*(local variable in `{}`)*", v.var_type, v.name, f.name)
                }
            });
        }
    }

    if let Some(func) = user_funcs.iter().find(|f| f.name == word) {
        let doc_part = func.doc.as_deref().map(|d| format!("\n\n{d}")).unwrap_or_default();
        return json!({
            "contents": {
                "kind": "markdown",
                "value": format!("```hlsl\n{}\n```{doc_part}", func.label)
            }
        });
    }

    if let Some(var) = user_vars.iter().find(|v| v.name == word) {
        let doc_part = var.doc.as_deref().map(|d| format!("\n\n{d}")).unwrap_or_default();
        return json!({
            "contents": {
                "kind": "markdown",
                "value": format!("```hlsl\n{} {}\n```{doc_part}", var.var_type, var.name).trim().to_string()
            }
        });
    }

    json!(null)
}

/// Handles textDocument/definition requests.
pub fn handle_definition(msg: &Value, doc_cache: &HashMap<String, String>) -> Value {
    let params = match msg.get("params") {
        Some(p) => p,
        None => return Value::Null,
    };

    let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
    let line_idx = params["position"]["line"].as_u64().unwrap_or(0) as usize;
    let col_idx = params["position"]["character"].as_u64().unwrap_or(0) as usize;

    let doc = match doc_cache.get(uri) {
        Some(d) => d,
        None => return Value::Null,
    };

    let line = match doc.lines().nth(line_idx) {
        Some(l) => l,
        None => return Value::Null,
    };

    // Check if clicking on #include "file.hlsl"
    if let Some(rest) = line.trim().strip_prefix("#include") {
        let inc_name = rest.trim().trim_matches(|c| c == '"' || c == '<' || c == '>');
        if let Some(main_path) = uri_to_path(uri) {
            if let Some(parent) = main_path.parent() {
                let candidate = parent.join(inc_name);
                if candidate.is_file() {
                    return json!({
                        "uri": path_to_uri(&candidate),
                        "range": {
                            "start": { "line": 0, "character": 0 },
                            "end": { "line": 0, "character": 0 }
                        }
                    });
                }
            }
        }
    }

    let max_col = col_idx.min(line.len());
    let mut word_start = max_col;
    for (i, c) in line[..max_col].char_indices().rev() {
        if c.is_alphanumeric() || c == '_' {
            word_start = i;
        } else {
            break;
        }
    }

    let mut word_end = max_col;
    for (i, c) in line[max_col..].char_indices() {
        if c.is_alphanumeric() || c == '_' {
            word_end = max_col + i + c.len_utf8();
        } else {
            break;
        }
    }

    let word = &line[word_start..word_end];
    if word.is_empty() {
        return Value::Null;
    }

    let (user_funcs, user_vars) = resolve_includes_and_scan_symbols(uri, doc, doc_cache);

    // Check parameters & local variables in current function scope first
    if let Some(f) = find_enclosing_function(&user_funcs, line_idx) {
        if let Some(p) = f.parsed_params.iter().find(|p| p.name == word) {
            let target_uri = f.file_uri.as_deref().unwrap_or(uri);
            return json!({
                "uri": target_uri,
                "range": {
                    "start": { "line": p.line, "character": p.col },
                    "end": { "line": p.line, "character": p.col + p.name.len() }
                }
            });
        }
        if let Some(v) = f.local_vars.iter().find(|v| v.name == word) {
            let target_uri = f.file_uri.as_deref().unwrap_or(uri);
            return json!({
                "uri": target_uri,
                "range": {
                    "start": { "line": v.line, "character": v.col },
                    "end": { "line": v.line, "character": v.col + v.name.len() }
                }
            });
        }
    }

    if let Some(f) = user_funcs.iter().find(|f| f.name == word) {
        let target_uri = f.file_uri.as_deref().unwrap_or(uri);
        return json!({
            "uri": target_uri,
            "range": {
                "start": { "line": f.line, "character": f.col },
                "end": { "line": f.line, "character": f.col + f.name.len() }
            }
        });
    }

    if let Some(v) = user_vars.iter().find(|v| v.name == word) {
        let target_uri = v.file_uri.as_deref().unwrap_or(uri);
        return json!({
            "uri": target_uri,
            "range": {
                "start": { "line": v.line, "character": v.col },
                "end": { "line": v.line, "character": v.col + v.name.len() }
            }
        });
    }

    Value::Null
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub field_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
}

/// Parses all structs and cbuffers and their member fields from HLSL source code.
pub fn scan_struct_definitions(text: &str) -> Vec<StructDef> {
    let mut structs = Vec::new();
    let mut current_struct: Option<(String, Vec<StructField>)> = None;
    let mut brace_depth = 0;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") {
            continue;
        }

        // struct Name or cbuffer Name
        if (trimmed.starts_with("struct ") || trimmed.starts_with("cbuffer ")) && current_struct.is_none() {
            let mut parts = trimmed.split_whitespace();
            parts.next(); // "struct" or "cbuffer"
            if let Some(name_raw) = parts.next() {
                let name = name_raw.split(['{', ':', ';']).next().unwrap_or("").trim();
                if is_valid_identifier(name) {
                    current_struct = Some((name.to_string(), Vec::new()));
                }
            }
        }

        let open_b = trimmed.chars().filter(|&c| c == '{').count();
        let close_b = trimmed.chars().filter(|&c| c == '}').count();

        if current_struct.is_some() {
            if brace_depth > 0 && trimmed.ends_with(';') {
                // e.g. "float4 position : SV_Position;" or "float4 color : COLOR;"
                let decl = trimmed.trim_end_matches(';').trim();
                let before_colon = decl.split(':').next().unwrap_or("").trim();
                let tokens: Vec<&str> = before_colon.split_whitespace().collect();
                if tokens.len() >= 2 {
                    let field_name = tokens.last().unwrap().split('[').next().unwrap_or("").trim();
                    let field_type = tokens[tokens.len() - 2].split('<').next().unwrap_or("").trim();
                    if is_valid_identifier(field_name) {
                        if let Some((_, ref mut fields)) = current_struct {
                            fields.push(StructField {
                                name: field_name.to_string(),
                                field_type: field_type.to_string(),
                            });
                        }
                    }
                }
            }

            if close_b > 0 && brace_depth + open_b <= close_b {
                if let Some((name, fields)) = current_struct.take() {
                    structs.push(StructDef { name, fields });
                }
            }
        }

        brace_depth = (brace_depth + open_b).saturating_sub(close_b);
    }

    if let Some((name, fields)) = current_struct {
        structs.push(StructDef { name, fields });
    }

    structs
}

/// Infers the type of a variable at or before `cursor_line`.
pub fn infer_variable_type(text: &str, var_name: &str, cursor_line: usize) -> Option<String> {
    let funcs = scan_user_functions(text, None, None);
    if let Some(f) = find_enclosing_function(&funcs, cursor_line) {
        if let Some(p) = f.parsed_params.iter().find(|p| p.name == var_name) {
            return Some(p.param_type.clone());
        }
        if let Some(v) = f.local_vars.iter().filter(|v| v.line <= cursor_line).find(|v| v.name == var_name) {
            return Some(v.var_type.clone());
        }
    }

    let vars = scan_user_variables(text, None, None);
    if let Some(v) = vars.iter().find(|v| v.name == var_name) {
        return Some(v.var_type.clone());
    }

    let lines: Vec<&str> = text.lines().collect();
    let max_line = cursor_line.min(lines.len());

    // Scan backwards from cursor_line
    for i in (0..max_line).rev() {
        let line = lines[i].trim();
        if line.starts_with("//") {
            continue;
        }

        // Check local declaration: "VSOutput output;" or "VSOutput output = ..."
        if line.contains(var_name) && (line.ends_with(';') || line.contains('=')) {
            let decl = line.trim_end_matches(';').split('=').next().unwrap_or("").trim();
            let tokens: Vec<&str> = decl.split_whitespace().collect();
            for (idx, &tok) in tokens.iter().enumerate() {
                let clean_tok = tok.split([':', '[']).next().unwrap_or("").trim();
                if clean_tok == var_name && idx > 0 {
                    let type_tok = tokens[idx - 1].split([':', '<']).next().unwrap_or("").trim();
                    if is_valid_identifier(type_tok) {
                        return Some(type_tok.to_string());
                    }
                }
            }
        }

        // Check function signature parameter: "VSMain(VSInput input)"
        if line.contains(var_name) && line.contains('(') {
            if let Some(open) = line.find('(') {
                let params_str = line[open + 1..].split(')').next().unwrap_or("");
                for param in params_str.split(',') {
                    let param_clean = param.split(':').next().unwrap_or("").trim();
                    let tokens: Vec<&str> = param_clean.split_whitespace().collect();
                    if tokens.len() >= 2 {
                        let p_name = tokens.last().unwrap().trim();
                        let p_type = tokens[tokens.len() - 2].trim();
                        if p_name == var_name && is_valid_identifier(p_type) {
                            return Some(p_type.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Generates hierarchical document symbols for Zed's Outline panel and Breadcrumbs.
pub fn get_document_symbols(text: &str) -> Value {
    let mut symbols = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    // 1. Scan ShaderLab blocks if present
    for (line_idx, &line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("Shader \"") {
            let name = trimmed.trim_start_matches("Shader \"").split('"').next().unwrap_or("Shader");
            symbols.push(json!({
                "name": name,
                "detail": "ShaderLab Shader",
                "kind": 4, // Package
                "range": {
                    "start": { "line": line_idx, "character": 0 },
                    "end": { "line": lines.len().saturating_sub(1), "character": 0 }
                },
                "selectionRange": {
                    "start": { "line": line_idx, "character": 0 },
                    "end": { "line": line_idx, "character": line.len() }
                }
            }));
        } else if trimmed.starts_with("SubShader") {
            symbols.push(json!({
                "name": "SubShader",
                "detail": "ShaderLab SubShader",
                "kind": 5, // Class
                "range": {
                    "start": { "line": line_idx, "character": 0 },
                    "end": { "line": line_idx + 1, "character": 0 }
                },
                "selectionRange": {
                    "start": { "line": line_idx, "character": 0 },
                    "end": { "line": line_idx, "character": line.len() }
                }
            }));
        } else if trimmed.starts_with("Pass") {
            symbols.push(json!({
                "name": "Pass",
                "detail": "ShaderLab Pass",
                "kind": 6, // Method
                "range": {
                    "start": { "line": line_idx, "character": 0 },
                    "end": { "line": line_idx + 1, "character": 0 }
                },
                "selectionRange": {
                    "start": { "line": line_idx, "character": 0 },
                    "end": { "line": line_idx, "character": line.len() }
                }
            }));
        }
    }

    // 2. Scan structs & cbuffers
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if (trimmed.starts_with("struct ") || trimmed.starts_with("cbuffer ")) && !trimmed.ends_with(';') {
            let is_struct = trimmed.starts_with("struct ");
            let kind = if is_struct { 23 } else { 5 }; // Struct : Class
            let detail = if is_struct { "struct" } else { "cbuffer" };
            let mut parts = trimmed.split_whitespace();
            parts.next();
            if let Some(name_raw) = parts.next() {
                let name = name_raw.split(['{', ':', ';']).next().unwrap_or("").trim();
                if is_valid_identifier(name) {
                    let start_line = i;
                    let mut brace_depth = 0;
                    let mut end_line = i;
                    let mut children = Vec::new();

                    for (j, line_item) in lines.iter().enumerate().skip(i) {
                        let cur = line_item.trim();
                        let open_b = cur.chars().filter(|&c| c == '{').count();
                        let close_b = cur.chars().filter(|&c| c == '}').count();
                        brace_depth += open_b;

                        if brace_depth > 0 && cur.ends_with(';') {
                            let decl = cur.trim_end_matches(';').trim();
                            let before_colon = decl.split(':').next().unwrap_or("").trim();
                            let tokens: Vec<&str> = before_colon.split_whitespace().collect();
                            if tokens.len() >= 2 {
                                let f_name = tokens.last().unwrap().split('[').next().unwrap_or("").trim();
                                let f_type = tokens[tokens.len() - 2].split('<').next().unwrap_or("").trim();
                                if is_valid_identifier(f_name) {
                                    children.push(json!({
                                        "name": f_name,
                                        "detail": f_type,
                                        "kind": 8, // Field
                                        "range": {
                                            "start": { "line": j, "character": 0 },
                                            "end": { "line": j, "character": line_item.len() }
                                        },
                                        "selectionRange": {
                                            "start": { "line": j, "character": 0 },
                                            "end": { "line": j, "character": line_item.len() }
                                        }
                                    }));
                                }
                            }
                        }

                        if brace_depth > 0 && brace_depth <= close_b {
                            end_line = j;
                            break;
                        }
                        brace_depth = brace_depth.saturating_sub(close_b);
                    }

                    symbols.push(json!({
                        "name": name,
                        "detail": detail,
                        "kind": kind,
                        "range": {
                            "start": { "line": start_line, "character": 0 },
                            "end": { "line": end_line, "character": lines[end_line].len() }
                        },
                        "selectionRange": {
                            "start": { "line": start_line, "character": 0 },
                            "end": { "line": start_line, "character": lines[start_line].len() }
                        },
                        "children": children
                    }));
                    i = end_line + 1;
                    continue;
                }
            }
        }
        i += 1;
    }

    // 3. Scan user functions (using pre-calculated body_end_line for O(1) performance)
    let funcs = scan_user_functions(text, None, None);
    for f in funcs {
        let start_line = f.line;
        let end_line = f.body_end_line.max(start_line);
        let end_line_len = lines.get(end_line).map(|l| l.len()).unwrap_or(0);

        symbols.push(json!({
            "name": f.name,
            "detail": f.label,
            "kind": 12, // Function
            "range": {
                "start": { "line": start_line, "character": 0 },
                "end": { "line": end_line, "character": end_line_len }
            },
            "selectionRange": {
                "start": { "line": start_line, "character": f.col },
                "end": { "line": start_line, "character": f.col + f.name.len() }
            }
        }));
    }

    json!(symbols)
}
