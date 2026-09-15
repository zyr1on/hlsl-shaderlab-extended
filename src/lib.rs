// src/lib.rs
// Zed Extension for HLSL & Unity ShaderLab powered by Microsoft DXC

use std::fs;
use zed::settings::LspSettings;
use zed_extension_api::{self as zed, LanguageServerId, Result};

struct HlslShaderlabExtension {
    cached_dxc: Option<String>,
    cached_hlsl_validator: Option<String>,
}

impl HlslShaderlabExtension {
    fn find_dxc(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        // 1. User explicit config in Zed settings
        if let Ok(settings) = LspSettings::for_worktree(language_server_id.as_ref(), worktree) {
            if let Some(opts) = settings.initialization_options {
                if let Some(path) = opts.get("dxc_path").and_then(|v| v.as_str()) {
                    let trimmed = path.trim();
                    if !trimmed.is_empty() && fs::metadata(trimmed).is_ok_and(|s| s.is_file()) {
                        return Ok(trimmed.to_string());
                    }
                }
            }
        }

        // 2. Check cached path
        if let Some(path) = &self.cached_dxc {
            if fs::metadata(path).is_ok_and(|s| s.is_file()) {
                return Ok(path.clone());
            }
        }

        // 3. Check system PATH
        if let Some(path) = worktree.which("dxc") {
            self.cached_dxc = Some(path.clone());
            return Ok(path);
        }

        // 4. Common fallback locations
        let (platform, _) = zed::current_platform();
        if matches!(platform, zed::Os::Windows) {
            let fallbacks = [
                "C:\\msys64\\ucrt64\\bin\\dxc.exe",
                "C:\\Program Files (x86)\\Windows Kits\\10\\bin\\x64\\dxc.exe",
            ];
            for fb in fallbacks {
                if fs::metadata(fb).is_ok_and(|s| s.is_file()) {
                    self.cached_dxc = Some(fb.to_string());
                    return Ok(fb.to_string());
                }
            }
        }

        let exe = if matches!(platform, zed::Os::Windows) { ".exe" } else { "" };
        Ok(format!("dxc{exe}"))
    }

    fn find_hlsl_validator(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        // 1. User config in settings
        if let Ok(settings) = LspSettings::for_worktree(language_server_id.as_ref(), worktree) {
            if let Some(binary) = settings.binary {
                if let Some(p) = binary.path {
                    let trimmed = p.trim();
                    if !trimmed.is_empty() {
                        return Ok(trimmed.to_string());
                    }
                }
            }
        }

        // 2. Check cached path
        if let Some(path) = &self.cached_hlsl_validator {
            if fs::metadata(path).is_ok_and(|s| s.is_file()) {
                return Ok(path.clone());
            }
        }

        // 3. Check system PATH
        if let Some(path) = worktree.which("hlsl_validator") {
            self.cached_hlsl_validator = Some(path.clone());
            return Ok(path);
        }

        // 4. Check user cargo bin directory (~/.cargo/bin/hlsl_validator.exe)
        let (platform, _) = zed::current_platform();
        let exe = if matches!(platform, zed::Os::Windows) { ".exe" } else { "" };
        if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            let cargo_bin = format!("{home}/.cargo/bin/hlsl_validator{exe}");
            if fs::metadata(&cargo_bin).is_ok_and(|s| s.is_file()) {
                self.cached_hlsl_validator = Some(cargo_bin.clone());
                return Ok(cargo_bin);
            }
        }

        // 5. Check local project target release directory
        let local_release = format!("D:/zed-hlsl-shaderlab/hlsl_validator/target/release/hlsl_validator{exe}");
        if fs::metadata(&local_release).is_ok_and(|s| s.is_file()) {
            self.cached_hlsl_validator = Some(local_release.clone());
            return Ok(local_release);
        }

        Ok(format!("hlsl_validator{exe}"))
    }
}

impl zed::Extension for HlslShaderlabExtension {
    fn new() -> Self {
        Self {
            cached_dxc: None,
            cached_hlsl_validator: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        match language_server_id.as_ref() {
            "hlsl_validator" => {
                let validator = self.find_hlsl_validator(language_server_id, worktree)?;
                let mut env = Vec::new();
                if let Ok(dxc) = self.find_dxc(language_server_id, worktree) {
                    env.push(("DXC_PATH".to_string(), dxc));
                }
                let root_path = worktree.root_path();
                if !root_path.is_empty() {
                    env.push(("WORKSPACE_ROOT".to_string(), root_path));
                }

                Ok(zed::Command {
                    command: validator,
                    args: vec![],
                    env,
                })
            }
            unknown => Err(format!("Unknown language server: {unknown}")),
        }
    }
}

zed::register_extension!(HlslShaderlabExtension);
