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

pub fn safe_floor_char_boundary(s: &str, mut index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    while index > 0 && !s.is_char_boundary(index) {
        index -= 1;
    }
    index
}

pub fn extract_word_at_pos(line: &str, col_idx: usize) -> &str {
    if line.is_empty() {
        return "";
    }
    let byte_pos = if col_idx >= line.len() {
        line.len()
    } else if line.is_char_boundary(col_idx) {
        col_idx
    } else {
        safe_floor_char_boundary(line, col_idx)
    };

    let mut word_start = byte_pos;
    for (i, c) in line[..byte_pos].char_indices().rev() {
        if c.is_alphanumeric() || c == '_' {
            word_start = i;
        } else {
            break;
        }
    }

    let mut word_end = byte_pos;
    for (i, c) in line[byte_pos..].char_indices() {
        if c.is_alphanumeric() || c == '_' {
            word_end = byte_pos + i + c.len_utf8();
        } else {
            break;
        }
    }

    &line[word_start..word_end]
}

pub fn find_identifier_in_line(line: &str, ident: &str) -> Option<usize> {
    for (idx, _) in line.match_indices(ident) {
        let before_ok = if idx == 0 {
            true
        } else {
            let prev = line[..idx].chars().next_back().unwrap_or(' ');
            !prev.is_alphanumeric() && prev != '_'
        };
        let after_idx = idx + ident.len();
        let after_ok = if after_idx >= line.len() {
            true
        } else {
            let next = line[after_idx..].chars().next().unwrap_or(' ');
            !next.is_alphanumeric() && next != '_'
        };
        if before_ok && after_ok {
            return Some(idx);
        }
    }
    None
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam {
    pub name: String,
    pub param_type: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalVar {
    pub name: String,
    pub var_type: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
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
            if line_idx == target_line {
                return in_line_comment || in_block_comment || in_string;
            }
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

    // If the token immediately preceding fn_name is a type identifier (e.g. "int test(",
    // "void myFunc(", "float doMath("), then this is a FUNCTION DECLARATION, NOT a function call!
    let before_fn = trimmed[..ident_start].trim_end();
    if !before_fn.is_empty() {
        let mut prev_token_start = before_fn.len();
        for (i, c) in before_fn.char_indices().rev() {
            if c.is_alphanumeric() || c == '_' {
                prev_token_start = i;
            } else {
                break;
            }
        }
        let prev_token = &before_fn[prev_token_start..];
        if !prev_token.is_empty() && !["return", "else", "do"].contains(&prev_token) {
            let between_tokens = &before_fn[prev_token_start + prev_token.len()..];
            if between_tokens.trim().is_empty() {
                return None;
            }
        }
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
                        for check_line in start_line..=line_idx {
                            if let Some(l) = text.lines().nth(check_line) {
                                if let Some(col) = find_identifier_in_line(l, &fp.name) {
                                    fp.line = check_line;
                                    fp.col = col;
                                    break;
                                }
                            }
                        }
                        parsed_params.push(fp);
                    }
                }

                let doc = if pending_doc.is_empty() { None } else { Some(pending_doc.join(" ")) };
                results.push(FunctionSignature {
                    name: fn_name,
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

        // Support function-like macros: #define MACRO_NAME(a, b) ...
        if !in_func_body && line.starts_with("#define") {
            if let Some(rest) = line.strip_prefix("#define") {
                let trimmed = rest.trim();
                if let Some(open_paren) = trimmed.find('(') {
                    let name_part = trimmed[..open_paren].trim();
                    if is_valid_identifier(name_part) && !INVALID_NAMES.contains(&name_part) {
                        if let Some(close_paren) = trimmed.find(')') {
                            let params_raw = &trimmed[open_paren + 1..close_paren];
                            let params: Vec<String> = params_raw
                                .split(',')
                                .map(|p| p.trim().to_string())
                                .filter(|p| !p.is_empty())
                                .collect();
                            let header = format!("{}({})", name_part, params.join(", "));
                            let mut parsed_params = Vec::new();
                            for p in &params {
                                parsed_params.push(FunctionParam {
                                    name: p.clone(),
                                    param_type: "var".to_string(),
                                    line: line_idx,
                                    col: 0,
                                });
                            }
                            let doc = if pending_doc.is_empty() {
                                None
                            } else {
                                Some(pending_doc.join(" "))
                            };
                            results.push(FunctionSignature {
                                name: name_part.to_string(),
                                label: header,
                                parameters: params,
                                parsed_params,
                                local_vars: Vec::new(),
                                body_start_line: line_idx,
                                body_end_line: line_idx,
                                doc,
                                source: source_name.map(|s| s.to_string()),
                                line: line_idx,
                                col: raw_line.find(name_part).unwrap_or(0),
                                file_uri: file_uri.map(|s| s.to_string()),
                            });
                            pending_doc.clear();
                        }
                    }
                }
            }
            continue;
        }

        if !in_func_body && !line.starts_with("return") && !line.starts_with('#') {
            let mut decl_line = line;
            while decl_line.starts_with('[') {
                if let Some(close_b) = decl_line.find(']') {
                    decl_line = decl_line[close_b + 1..].trim();
                } else {
                    break;
                }
            }

            if let Some(open_paren) = decl_line.find('(') {
                let before = decl_line[..open_paren].trim();
                let mut it = before.split_whitespace().rev();
                if let (Some(fn_name), Some(return_type)) = (it.next(), it.next()) {
                    if is_valid_identifier(fn_name)
                        && !INVALID_NAMES.contains(&fn_name)
                        && fn_name != "register"
                        && fn_name != "packoffset"
                        && is_valid_identifier(return_type)
                        && !INVALID_TYPES.contains(&return_type)
                    {
                        let col_idx = raw_line.find(fn_name).unwrap_or(0);
                        if let Some(close_idx) = decl_line.find(')') {
                            let header = &decl_line[..=close_idx];
                            let params_part = &decl_line[open_paren + 1..close_idx];
                            let params: Vec<String> = params_part
                                .split(',')
                                .map(|p| p.trim().to_string())
                                .filter(|p| !p.is_empty() && p != "void")
                                .collect();

                            let mut parsed_params = Vec::new();
                            for p in &params {
                                if let Some(mut fp) = parse_parameter_decl(p, line_idx, open_paren + 1) {
                                    fp.col = find_identifier_in_line(raw_line, &fp.name).unwrap_or(open_paren + 1);
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

                            let rest = &decl_line[close_idx..];
                            let open_b = rest.chars().filter(|&c| c == '{').count();
                            let close_b = rest.chars().filter(|&c| c == '}').count();
                            if open_b > close_b {
                                in_func_body = true;
                                func_brace_depth = open_b - close_b;
                            } else if open_b == 0 && !decl_line.ends_with(';') {
                                expecting_body = true;
                            }
                        } else {
                            // Multi-line header detected
                            pending_header = Some((
                                fn_name.to_string(),
                                return_type.to_string(),
                                line_idx,
                                col_idx,
                                decl_line.to_string(),
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
    scan_user_variables_with_funcs(text, source_name, file_uri, None)
}

pub fn scan_user_variables_with_funcs(
    text: &str,
    source_name: Option<&str>,
    file_uri: Option<&str>,
    existing_funcs: Option<&[FunctionSignature]>,
) -> Vec<VariableSymbol> {
    let fallback_funcs;
    let funcs: &[FunctionSignature] = match existing_funcs {
        Some(f) => f,
        None => {
            fallback_funcs = scan_user_functions(text, None, None);
            &fallback_funcs
        }
    };
    let mut results = Vec::new();
    let mut pending_doc = Vec::new();
    let mut block_depth: usize = 0;
    let mut current_block: Option<(String, bool)> = None; // (name, is_cbuffer)
    let mut func_idx = 0;

    for (line_idx, raw_line) in text.lines().enumerate() {
        while func_idx < funcs.len() && line_idx > funcs[func_idx].body_end_line {
            func_idx += 1;
        }
        if func_idx < funcs.len() && line_idx >= funcs[func_idx].body_start_line && line_idx <= funcs[func_idx].body_end_line {
            continue;
        }

        let line = raw_line.trim();

        if line.starts_with("//") {
            let doc_line = line.trim_start_matches('/').trim();
            if !doc_line.is_empty() {
                pending_doc.push(doc_line.to_string());
            }
            continue;
        }

        if line.is_empty() {
            if current_block.is_none() {
                pending_doc.clear();
            }
            continue;
        }

        // cbuffer Name, CBUFFER_START(Name), or struct Name
        if (line.starts_with("cbuffer ") || line.starts_with("struct ")) && current_block.is_none() {
            let mut it = line.split_whitespace();
            let kw = it.next().unwrap_or("");
            if let Some(name_raw) = it.next() {
                let name = name_raw.split(['{', ':']).next().unwrap_or("").trim();
                if is_valid_identifier(name) {
                    current_block = Some((name.to_string(), kw == "cbuffer"));
                    block_depth = 0;
                }
            }
        } else if (line.starts_with("CBUFFER_START(") || line.starts_with("CBUFFER_START ")) && current_block.is_none() {
            let inner = line.trim_start_matches("CBUFFER_START").trim();
            let name = inner.trim_matches(|c| c == '(' || c == ')' || c == ';' || c == '{').trim();
            if is_valid_identifier(name) {
                current_block = Some((name.to_string(), true));
                block_depth = 0;
            }
        } else if line.starts_with("CBUFFER_END") {
            current_block = None;
            block_depth = 0;
        }

        // Texture/Sampler macros: TEXTURE2D(_BaseMap); SAMPLER(sampler_BaseMap);
        if (line.starts_with("TEXTURE2D") || line.starts_with("TEXTURE3D") || line.starts_with("TEXTURECUBE") || line.starts_with("SAMPLER")) && line.ends_with(';') {
            let open = line.find('(');
            let close = line.find(')');
            if let (Some(o), Some(c)) = (open, close) {
                if c > o + 1 {
                    let var_name = line[o + 1..c].trim();
                    let macro_type = line[..o].trim();
                    if is_valid_identifier(var_name) {
                        let col = raw_line.find(var_name).unwrap_or(0);
                        results.push(VariableSymbol {
                            name: var_name.to_string(),
                            var_type: macro_type.to_string(),
                            qualifier: "uniform".to_string(),
                            doc: if pending_doc.is_empty() { None } else { Some(pending_doc.join(" ")) },
                            line: line_idx,
                            col,
                            source: source_name.map(|s| s.to_string()),
                            file_uri: file_uri.map(String::from),
                        });
                    }
                }
            }
        }

        let in_cbuffer = current_block.as_ref().map(|(_, is_cb)| *is_cb).unwrap_or(false);
        let in_struct = current_block.as_ref().map(|(_, is_cb)| !*is_cb).unwrap_or(false);

        let code_part = line.split("//").next().unwrap_or("").trim();

        // Variable declaration ending with semicolon
        if code_part.ends_with(';') && !code_part.starts_with('#') && !code_part.starts_with("return") && !in_struct {
            let clean = code_part.trim_end_matches(';').trim();
            let decl = clean.split('=').next().unwrap_or("").trim();
            let before_colon = decl.split(':').next().unwrap_or("").trim();
            let tokens: Vec<&str> = before_colon.split_whitespace().collect();
            if tokens.len() >= 2 {
                let var_name_raw = tokens.last().copied().unwrap_or("");
                let var_name = var_name_raw.split('[').next().unwrap_or("").trim();
                let var_type = tokens[tokens.len() - 2].split('<').next().unwrap_or("").trim();

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

        let open_b = code_part.chars().filter(|&c| c == '{').count();
        let close_b = code_part.chars().filter(|&c| c == '}').count();
        if current_block.is_some() {
            block_depth += open_b;
            if block_depth > 0 && block_depth <= close_b {
                current_block = None;
                block_depth = 0;
            } else {
                block_depth = block_depth.saturating_sub(close_b);
            }
        }

        if line.ends_with(';') || line.ends_with('}') {
            pending_doc.clear();
        }
    }

    results
}

#[derive(Clone)]
struct CachedIncludeSymbols {
    mtime: std::time::SystemTime,
    functions: Vec<FunctionSignature>,
    variables: Vec<VariableSymbol>,
    structs: Vec<StructDef>,
    include_targets: Vec<String>,
}

static ON_DISK_SYMBOL_CACHE: std::sync::OnceLock<std::sync::Mutex<HashMap<std::path::PathBuf, CachedIncludeSymbols>>> =
    std::sync::OnceLock::new();

fn extract_include_targets(text: &str) -> Vec<String> {
    let mut targets = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("#include") {
            let target = rest.trim().trim_matches(|c| c == '"' || c == '<' || c == '>');
            if !target.is_empty() {
                targets.push(target.to_string());
            }
        }
    }
    targets
}

///// Resolves #include directives recursively and collects user functions, symbols & structs.
pub fn resolve_includes_and_scan_symbols(
    uri: &str,
    doc_content: &str,
    doc_cache: &HashMap<String, String>,
) -> (Vec<FunctionSignature>, Vec<VariableSymbol>, Vec<StructDef>) {
    let mut all_functions = Vec::new();
    let mut all_variables = Vec::new();
    let mut all_structs = Vec::new();
    let mut visited = std::collections::HashSet::new();
    visited.insert(uri.to_string());

    // Scan the current active document first
    let doc_funcs = scan_user_functions(doc_content, None, Some(uri));
    let doc_vars = scan_user_variables_with_funcs(doc_content, None, Some(uri), Some(&doc_funcs));
    let doc_structs = scan_struct_definitions_with_uri(doc_content, Some(uri));
    let doc_includes = extract_include_targets(doc_content);

    all_functions.extend(doc_funcs);
    all_variables.extend(doc_vars);
    all_structs.extend(doc_structs);

    let mut queue = vec![(uri.to_string(), doc_includes, 0usize)];

    while let Some((curr_uri, inc_targets, depth)) = queue.pop() {
        if depth >= 8 {
            continue;
        }

        let curr_path = uri_to_path(&curr_uri);
        let curr_dir = curr_path.as_ref().and_then(|p| p.parent()).map(|p| p.to_path_buf());

        for include_target in &inc_targets {
            let mut found_candidate: Option<(std::path::PathBuf, String)> = None;

            // 1. Check relative to current file in doc_cache first, then on disk
            if let Some(dir) = &curr_dir {
                let candidate = dir.join(include_target);
                let cand_uri = path_to_uri(&candidate);
                if crate::get_document_from_cache(doc_cache, &cand_uri).is_some() {
                    found_candidate = Some((candidate, cand_uri));
                } else if candidate.is_file() {
                    found_candidate = Some((candidate.clone(), cand_uri));
                }
            }

            // 2. Check doc_cache by filename/suffix match (for unsaved buffers or virtual URIs)
            if found_candidate.is_none() {
                let target_suffix = format!("/{}", include_target.replace('\\', "/"));
                for k in doc_cache.keys() {
                    if k.ends_with(&target_suffix) || k.ends_with(include_target.as_str()) {
                        let path = uri_to_path(k).unwrap_or_else(|| std::path::PathBuf::from(include_target));
                        found_candidate = Some((path, k.clone()));
                        break;
                    }
                }
            }

            // 3. Check search paths on disk
            if found_candidate.is_none() {
                let search_paths = crate::discover_include_paths(&curr_uri, None);
                for sp in &search_paths {
                    let direct = sp.join(include_target);
                    if direct.is_file() {
                        let uri = path_to_uri(&direct);
                        found_candidate = Some((direct, uri));
                        break;
                    }

                    // Handle Unity Packages/com.unity... mapped to Library/PackageCache
                    let lower_target = include_target.to_lowercase();
                    if lower_target.starts_with("packages/") {
                        let pkg_sub = &include_target[9..];
                        let cache_dir = if sp.ends_with("PackageCache") {
                            Some(sp.clone())
                        } else if sp.join("Library").join("PackageCache").is_dir() {
                            Some(sp.join("Library").join("PackageCache"))
                        } else {
                            None
                        };

                        if let Some(pcd) = cache_dir {
                            if let Some((pkg_name, inner)) = pkg_sub.split_once('/') {
                                if let Ok(entries) = std::fs::read_dir(&pcd) {
                                    for entry in entries.flatten() {
                                        let file_name = entry.file_name().to_string_lossy().to_string();
                                        if file_name.starts_with(pkg_name) && file_name.contains('@') {
                                            let cand = entry.path().join(inner);
                                            if cand.is_file() {
                                                let uri = path_to_uri(&cand);
                                                found_candidate = Some((cand, uri));
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if found_candidate.is_some() {
                        break;
                    }
                }
            }

            if let Some((candidate, inc_uri)) = found_candidate {
                if visited.insert(inc_uri.clone()) {
                    let inc_name = candidate.file_name().and_then(|n| n.to_str()).unwrap_or(include_target);

                    // If open in editor, use live doc_cache content
                    if let Some(inc_content) = crate::get_document_from_cache(doc_cache, &inc_uri) {
                        let funcs = scan_user_functions(inc_content, Some(inc_name), Some(&inc_uri));
                        let vars = scan_user_variables_with_funcs(inc_content, Some(inc_name), Some(&inc_uri), Some(&funcs));
                        let structs = scan_struct_definitions_with_uri(inc_content, Some(&inc_uri));
                        let child_includes = extract_include_targets(inc_content);

                        all_functions.extend(funcs);
                        all_variables.extend(vars);
                        all_structs.extend(structs);
                        queue.push((inc_uri, child_includes, depth + 1));
                    } else if let Ok(m) = std::fs::metadata(&candidate) {
                        let mtime = m.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                        let cache_lock = ON_DISK_SYMBOL_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));

                        let cached_entry = if let Ok(guard) = cache_lock.lock() {
                            guard.get(&candidate).filter(|entry| entry.mtime == mtime).cloned()
                        } else {
                            None
                        };

                        if let Some(entry) = cached_entry {
                            all_functions.extend(entry.functions);
                            all_variables.extend(entry.variables);
                            all_structs.extend(entry.structs);
                            queue.push((inc_uri, entry.include_targets, depth + 1));
                        } else if let Ok(inc_content) = std::fs::read_to_string(&candidate) {
                            let funcs = scan_user_functions(&inc_content, Some(inc_name), Some(&inc_uri));
                            let vars = scan_user_variables_with_funcs(&inc_content, Some(inc_name), Some(&inc_uri), Some(&funcs));
                            let structs = scan_struct_definitions_with_uri(&inc_content, Some(&inc_uri));
                            let child_includes = extract_include_targets(&inc_content);

                            let new_entry = CachedIncludeSymbols {
                                mtime,
                                functions: funcs.clone(),
                                variables: vars.clone(),
                                structs: structs.clone(),
                                include_targets: child_includes.clone(),
                            };

                            if let Ok(mut guard) = cache_lock.lock() {
                                if guard.len() > 500 {
                                    guard.clear();
                                }
                                guard.insert(candidate, new_entry);
                            }

                            all_functions.extend(funcs);
                            all_variables.extend(vars);
                            all_structs.extend(structs);
                            queue.push((inc_uri, child_includes, depth + 1));
                        }
                    }
                }
            }
        }
    }

    (all_functions, all_variables, all_structs)
}

#[inline]
pub fn select_best_overload(param_counts: &[usize], active_param: usize) -> (usize, usize) {
    let mut best_matching_sig = None;
    let mut best_diff = usize::MAX;
    let mut max_params = 0;
    let mut max_params_sig = 0;

    for (idx, &p_count) in param_counts.iter().enumerate() {
        if p_count > max_params {
            max_params = p_count;
            max_params_sig = idx;
        }
        if active_param < p_count {
            let diff = p_count - active_param;
            if diff < best_diff {
                best_diff = diff;
                best_matching_sig = Some(idx);
            }
        }
    }

    let active_sig = best_matching_sig.unwrap_or(max_params_sig);
    let sig_param_len = param_counts.get(active_sig).copied().unwrap_or(0);
    let active_p = if sig_param_len > 0 {
        active_param.min(sig_param_len.saturating_sub(1))
    } else {
        0
    };
    (active_sig, active_p)
}

/// Handles textDocument/signatureHelp requests.
pub fn get_signature_help(
    uri: &str,
    doc_content: &str,
    line_idx: usize,
    col_idx: usize,
    doc_cache: &HashMap<String, String>,
) -> Value {
    let call_info = find_enclosing_call(doc_content, line_idx, col_idx)
        .or_else(|| find_enclosing_call(doc_content, line_idx, col_idx + 1))
        .or_else(|| {
            if col_idx > 0 {
                find_enclosing_call(doc_content, line_idx, col_idx - 1)
            } else {
                None
            }
        });

    let (fn_name, active_param) = match call_info {
        Some((name, param)) => (name, param),
        None => return json!(null),
    };

    // 0. Unity ShaderLab Built-in Property Types (Range)
    if fn_name == "Range" {
        let active_p = active_param.min(1);
        return json!({
            "signatures": [{
                "label": "Range(float min, float max)",
                "documentation": {
                    "kind": "markdown",
                    "value": "### `Range(min, max)`\n*Unity ShaderLab Property Type*\n\nCreates a floating-point property bounded by an interactive slider between `min` and `max` in the Unity Material Inspector."
                },
                "parameters": [
                    { "label": "float min" },
                    { "label": "float max" }
                ]
            }],
            "activeSignature": 0,
            "activeParameter": active_p
        });
    }

    let (user_funcs, _, _) = resolve_includes_and_scan_symbols(uri, doc_content, doc_cache);
    let matched_funcs: Vec<&FunctionSignature> = user_funcs
        .iter()
        .filter(|f| f.name == fn_name)
        .collect();

    if !matched_funcs.is_empty() {
        let param_counts: Vec<usize> = matched_funcs.iter().map(|f| f.parameters.len()).collect();
        let (active_sig, active_p) = select_best_overload(&param_counts, active_param);

        let builtin_doc = docs::find_builtin_function(&fn_name);
        let signatures: Vec<Value> = matched_funcs
            .iter()
            .map(|f| {
                let params: Vec<Value> = f.parameters.iter().map(|p| json!({ "label": p })).collect();
                let doc_val = match (&f.doc, &f.source) {
                    (Some(d), Some(src)) => format!("**Source:** `{src}`\n\n{d}"),
                    (Some(d), None) => d.clone(),
                    (None, Some(src)) => {
                        if let Some(bi) = builtin_doc {
                            format!("*(Defined in `{src}`)*\n\n{}", bi.description)
                        } else {
                            format!("**Source:** `{src}`")
                        }
                    }
                    (None, None) => {
                        if let Some(bi) = builtin_doc {
                            bi.description.to_string()
                        } else {
                            format!("User-defined function `{}`", f.name)
                        }
                    }
                };
                json!({
                    "label": f.label,
                    "documentation": {
                        "kind": "markdown",
                        "value": doc_val
                    },
                    "parameters": params
                })
            })
            .collect();

        return json!({
            "signatures": signatures,
            "activeSignature": active_sig,
            "activeParameter": active_p
        });
    }

    // 2. Built-in HLSL Intrinsics & Engine Helpers (Fallback if not found in code or includes)
    if let Some(builtin) = docs::find_builtin_function(&fn_name) {
        let param_counts: Vec<usize> = builtin.overloads.iter().map(|ol| ol.params.len()).collect();
        let (active_sig, active_p) = select_best_overload(&param_counts, active_param);

        let signatures: Vec<Value> = builtin
            .overloads
            .iter()
            .map(|ol| {
                let params: Vec<Value> = ol.params.iter().map(|p| json!({ "label": *p })).collect();
                json!({
                    "label": ol.label,
                    "documentation": {
                        "kind": "markdown",
                        "value": builtin.description
                    },
                    "parameters": params
                })
            })
            .collect();

        return json!({
            "signatures": signatures,
            "activeSignature": active_sig,
            "activeParameter": active_p
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

    let word = extract_word_at_pos(line, col_idx);
    if word.is_empty() {
        return json!(null);
    }

    let (user_funcs, user_vars, user_structs) = resolve_includes_and_scan_symbols(uri, doc_content, doc_cache);

    // 1. Local scope inside enclosing function takes highest precedence!
    // (Prevents parameters/locals named 'distance' or 'saturate' from being shadowed by intrinsics)
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

    // 2. Built-in HLSL Intrinsics (Microsoft reference)
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

    // 3. Built-in Semantics
    if let Some((_, desc)) = docs::BUILTIN_VARIABLES.iter().find(|(name, _)| *name == word) {
        return json!({
            "contents": {
                "kind": "markdown",
                "value": format!("### `{word}`\n*HLSL Semantic*\n\n{desc}")
            }
        });
    }

    // 4. Engine Built-in Variables & Functions (_Time, unity_ObjectToWorld, TransformObjectToHClip, etc.)
    if let Some(ev) = docs::find_engine_variable(word) {
        return json!({
            "contents": {
                "kind": "markdown",
                "value": format!("```hlsl\n{}\n```\n\n{}", ev.detail, ev.description)
            }
        });
    }

    // 5. User-defined functions
    if let Some(func) = user_funcs.iter().find(|f| f.name == word) {
        let doc_part = func.doc.as_deref().map(|d| format!("\n\n{d}")).unwrap_or_default();
        return json!({
            "contents": {
                "kind": "markdown",
                "value": format!("```hlsl\n{}\n```{doc_part}", func.label)
            }
        });
    }

    // 6. User-defined variables & CBuffer members
    if let Some(var) = user_vars.iter().find(|v| v.name == word) {
        let doc_part = var.doc.as_deref().map(|d| format!("\n\n{d}")).unwrap_or_default();
        return json!({
            "contents": {
                "kind": "markdown",
                "value": format!("```hlsl\n{} {}\n```{doc_part}", var.var_type, var.name).trim().to_string()
            }
        });
    }

    // 7. Struct Definitions (including structs declared in #included files)
    if let Some(s) = user_structs.iter().find(|s| s.name == word) {
        let mut fields_str = String::new();
        for f in &s.fields {
            fields_str.push_str(&format!("    {} {};\n", f.field_type, f.name));
        }
        return json!({
            "contents": {
                "kind": "markdown",
                "value": format!("```hlsl\nstruct {}\n{{\n{}}};\n```", s.name, fields_str)
            }
        });
    }

    // 8. ShaderLab Properties hover
    let props = scan_shaderlab_properties(doc_content);
    if let Some(p) = props.iter().find(|p| p.name == word) {
        let def_str = if p.default_val.is_empty() {
            "".to_string()
        } else {
            format!(" = {}", p.default_val)
        };
        return json!({
            "contents": {
                "kind": "markdown",
                "value": format!("### `{}` ({})\n*ShaderLab Property*\n\nDisplay Name: **\"{}\"**\nDefault: `{}`", p.name, p.prop_type, p.display_name, def_str.trim_start_matches(" = "))
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

    let owned_doc;
    let doc = match crate::get_document_from_cache(doc_cache, uri) {
        Some(d) => d,
        None => {
            if let Some(p) = uri_to_path(uri) {
                if let Ok(content) = std::fs::read_to_string(p) {
                    owned_doc = content;
                    owned_doc.as_str()
                } else {
                    return Value::Null;
                }
            } else {
                return Value::Null;
            }
        }
    };

    let line = match doc.lines().nth(line_idx) {
        Some(l) => l,
        None => return Value::Null,
    };

    // Check if clicking on #include "file.hlsl"
    if let Some(rest) = line.trim().strip_prefix("#include") {
        let inc_name = rest.trim().trim_matches(|c| c == '"' || c == '<' || c == '>');
        if !inc_name.is_empty() {
            if let Some(main_path) = uri_to_path(uri) {
                if let Some(parent) = main_path.parent() {
                    let candidate = parent.join(inc_name);
                    let cand_uri = path_to_uri(&candidate);
                    if doc_cache.contains_key(&cand_uri) || candidate.is_file() {
                        return json!({
                            "uri": cand_uri,
                            "range": {
                                "start": { "line": 0, "character": 0 },
                                "end": { "line": 0, "character": 0 }
                            }
                        });
                    }
                }
            }

            // Check doc_cache by suffix
            let target_suffix = format!("/{}", inc_name.replace('\\', "/"));
            for k in doc_cache.keys() {
                if k.ends_with(&target_suffix) || k.ends_with(inc_name) {
                    return json!({
                        "uri": k,
                        "range": {
                            "start": { "line": 0, "character": 0 },
                            "end": { "line": 0, "character": 0 }
                        }
                    });
                }
            }
        }
    }

    let word = extract_word_at_pos(line, col_idx);
    if word.is_empty() {
        return Value::Null;
    }

    let (user_funcs, user_vars, user_structs) = resolve_includes_and_scan_symbols(uri, doc, doc_cache);

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

    if let Some(s) = user_structs.iter().find(|s| s.name == word) {
        let target_uri = s.file_uri.as_deref().unwrap_or(uri);
        return json!({
            "uri": target_uri,
            "range": {
                "start": { "line": s.line, "character": s.col },
                "end": { "line": s.line, "character": s.col + s.name.len() }
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
    pub line: usize,
    pub col: usize,
    pub file_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShaderLabProperty {
    pub name: String,
    pub display_name: String,
    pub prop_type: String,
    pub default_val: String,
    pub line: usize,
    pub col: usize,
}

pub fn scan_shaderlab_properties(text: &str) -> Vec<ShaderLabProperty> {
    let mut properties = Vec::new();
    let mut in_properties = false;
    let mut brace_depth = 0;

    for (line_idx, raw_line) in text.lines().enumerate() {
        let trimmed = raw_line.trim();

        if trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("Properties") && !in_properties {
            in_properties = true;
            brace_depth = 0;
        }

        let open_b = trimmed.chars().filter(|&c| c == '{').count();
        let close_b = trimmed.chars().filter(|&c| c == '}').count();

        if in_properties {
            brace_depth += open_b;

            if brace_depth > 0 && !trimmed.starts_with("Properties") && !trimmed.starts_with('{') && !trimmed.starts_with('}') {
                let mut without_attr = trimmed;
                while without_attr.starts_with('[') {
                    if let Some(end_bracket) = without_attr.find(']') {
                        without_attr = without_attr[end_bracket + 1..].trim();
                    } else {
                        break;
                    }
                }

                if let Some(paren_idx) = without_attr.find('(') {
                    let name = without_attr[..paren_idx].trim();
                    if is_valid_identifier(name) {
                        let col = raw_line.find(name).unwrap_or(0);
                        let after_paren = &without_attr[paren_idx + 1..];

                        // Extract display name between quotes
                        if let Some(first_quote) = after_paren.find('"') {
                            if let Some(second_quote) = after_paren[first_quote + 1..].find('"') {
                                let display_end = first_quote + 1 + second_quote;
                                let display = &after_paren[first_quote + 1..display_end];
                                let after_display = &after_paren[display_end + 1..];

                                if let Some(comma_idx) = after_display.find(',') {
                                    let rest = after_display[comma_idx + 1..].trim();
                                    // Parse property type, correctly balancing parentheses for Range(min, max)
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
                                    let p_type = rest[..type_end].trim();
                                    let after_type = if type_end < rest.len() { rest[type_end + 1..].trim() } else { "" };
                                    let def_val = after_type.strip_prefix('=').map(|s| s.trim()).unwrap_or("");

                                    properties.push(ShaderLabProperty {
                                        name: name.to_string(),
                                        display_name: display.to_string(),
                                        prop_type: p_type.to_string(),
                                        default_val: def_val.to_string(),
                                        line: line_idx,
                                        col,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            if brace_depth > 0 && brace_depth <= close_b {
                in_properties = false;
                brace_depth = 0;
            } else {
                brace_depth = brace_depth.saturating_sub(close_b);
            }
        }
    }

    properties
}

pub fn scan_struct_definitions_with_uri(text: &str, file_uri: Option<&str>) -> Vec<StructDef> {
    let mut structs = Vec::new();
    let mut current_struct: Option<(String, Vec<StructField>, usize, usize)> = None;
    let mut struct_brace_depth = 0;

    for (line_idx, raw_line) in text.lines().enumerate() {
        let trimmed = raw_line.trim();
        let code_part = trimmed.split("//").next().unwrap_or("").trim();

        if code_part.is_empty() {
            continue;
        }

        // struct Name or cbuffer Name
        if (code_part.starts_with("struct ") || code_part.starts_with("cbuffer ")) && current_struct.is_none() {
            if code_part.ends_with(';') && !code_part.contains('{') {
                continue;
            }
            let mut parts = code_part.split_whitespace();
            parts.next(); // "struct" or "cbuffer"
            if let Some(name_raw) = parts.next() {
                let name = name_raw.split(['{', ':', ';']).next().unwrap_or("").trim();
                if is_valid_identifier(name) {
                    let col = raw_line.find(name).unwrap_or(0);
                    current_struct = Some((name.to_string(), Vec::new(), line_idx, col));
                    struct_brace_depth = 0;
                }
            }
        }

        let open_b = code_part.chars().filter(|&c| c == '{').count();
        let close_b = code_part.chars().filter(|&c| c == '}').count();

        if current_struct.is_some() {
            struct_brace_depth += open_b;

            if struct_brace_depth > 0 && code_part.ends_with(';') {
                // e.g. "float4 position : SV_Position;" or "float4 color : COLOR;"
                let decl = code_part.trim_end_matches(';').trim();
                let before_colon = decl.split(':').next().unwrap_or("").trim();
                let tokens: Vec<&str> = before_colon.split_whitespace().collect();
                if tokens.len() >= 2 {
                    let field_name = tokens[tokens.len() - 1].split('[').next().unwrap_or("").trim();
                    let field_type = tokens[tokens.len() - 2].split('<').next().unwrap_or("").trim();
                    if is_valid_identifier(field_name) {
                        if let Some((_, ref mut fields, _, _)) = current_struct {
                            fields.push(StructField {
                                name: field_name.to_string(),
                                field_type: field_type.to_string(),
                            });
                        }
                    }
                }
            }

            if struct_brace_depth > 0 && struct_brace_depth <= close_b {
                if let Some((name, fields, line, col)) = current_struct.take() {
                    structs.push(StructDef {
                        name,
                        fields,
                        line,
                        col,
                        file_uri: file_uri.map(|s| s.to_string()),
                    });
                }
                struct_brace_depth = 0;
            } else {
                struct_brace_depth = struct_brace_depth.saturating_sub(close_b);
            }
        }
    }

    if let Some((name, fields, line, col)) = current_struct {
        structs.push(StructDef {
            name,
            fields,
            line,
            col,
            file_uri: file_uri.map(|s| s.to_string()),
        });
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
                        let p_name = tokens[tokens.len() - 1].trim();
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
            let end_line = (line_idx + 1).min(lines.len().saturating_sub(1));
            symbols.push(json!({
                "name": "SubShader",
                "detail": "ShaderLab SubShader",
                "kind": 5, // Class
                "range": {
                    "start": { "line": line_idx, "character": 0 },
                    "end": { "line": end_line, "character": lines.get(end_line).map(|l| l.len()).unwrap_or(0) }
                },
                "selectionRange": {
                    "start": { "line": line_idx, "character": 0 },
                    "end": { "line": line_idx, "character": line.len() }
                }
            }));
        } else if trimmed.starts_with("Pass") {
            let end_line = (line_idx + 1).min(lines.len().saturating_sub(1));
            symbols.push(json!({
                "name": "Pass",
                "detail": "ShaderLab Pass",
                "kind": 6, // Method
                "range": {
                    "start": { "line": line_idx, "character": 0 },
                    "end": { "line": end_line, "character": lines.get(end_line).map(|l| l.len()).unwrap_or(0) }
                },
                "selectionRange": {
                    "start": { "line": line_idx, "character": 0 },
                    "end": { "line": line_idx, "character": line.len() }
                }
            }));
        }
    }

    // 1b. Scan ShaderLab Properties
    for p in scan_shaderlab_properties(text) {
        symbols.push(json!({
            "name": p.name,
            "detail": format!("{} ({})", p.prop_type, p.display_name),
            "kind": 7, // Property
            "range": {
                "start": { "line": p.line, "character": p.col },
                "end": { "line": p.line, "character": p.col + p.name.len() }
            },
            "selectionRange": {
                "start": { "line": p.line, "character": p.col },
                "end": { "line": p.line, "character": p.col + p.name.len() }
            }
        }));
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
                                let f_name = tokens[tokens.len() - 1].split('[').next().unwrap_or("").trim();
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

