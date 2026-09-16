// src/lib.rs
//
// HLSL & ShaderLab Extended for Zed Editor
// =====================================================================
// LSP: hlsl_validator
// Backend compiler: Microsoft DirectXShaderCompiler (DXC)
// =====================================================================

use std::fs;
use std::path::{Path, PathBuf};
use zed::settings::LspSettings;
use zed_extension_api::{self as zed, LanguageServerId, Result, serde_json};

/// Extracts the first non-empty, trimmed string matching any key in `keys` from a JSON value.
fn extract_path_from_json(val: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for &key in keys {
        if let Some(v) = val.get(key)
            && let Some(s) = v.as_str()
        {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Resolves a user-configured path if non-empty, verifying file existence or resolving via PATH.
fn resolve_configured_path(configured: Option<String>, worktree: &zed::Worktree) -> Option<String> {
    let path = configured?;
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }
    if fs::metadata(trimmed).is_ok_and(|s| s.is_file()) {
        return Some(trimmed.to_string());
    }
    if let Some(resolved) = worktree.which(trimmed) {
        return Some(resolved);
    }
    Some(trimmed.to_string())
}

/// Recursively searches for a file matching `target_name` in a directory.
fn find_file_recursive(dir: &Path, target_name: &str) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(name) = p.file_name().and_then(|n| n.to_str())
                    && name.eq_ignore_ascii_case(target_name)
                {
                    return Some(p);
                }
            } else if p.is_dir()
                && let Some(found) = find_file_recursive(&p, target_name)
            {
                return Some(found);
            }
        }
    }
    None
}

/// Helper to locate DXC executable inside extracted release folder.
fn locate_dxc(version_dir: &str, is_windows: bool, arch: zed::Architecture) -> Option<String> {
    let dir_path = Path::new(version_dir);
    if is_windows {
        let arch_folder = match arch {
            zed::Architecture::Aarch64 => "arm64",
            zed::Architecture::X86 => "x86",
            _ => "x64",
        };
        let preferred = format!("{version_dir}/bin/{arch_folder}/dxc.exe");
        if fs::metadata(&preferred).is_ok_and(|s| s.is_file()) {
            return Some(preferred);
        }
        find_file_recursive(dir_path, "dxc.exe").map(|p| p.to_string_lossy().to_string())
    } else {
        let preferred = format!("{version_dir}/bin/dxc");
        if fs::metadata(&preferred).is_ok_and(|s| s.is_file()) {
            return Some(preferred);
        }
        find_file_recursive(dir_path, "dxc").map(|p| p.to_string_lossy().to_string())
    }
}

/// Helper to locate hlsl_validator executable inside extracted release folder.
fn locate_validator(version_dir: &str, binary_name: &str) -> Option<String> {
    let root = format!("{version_dir}/{binary_name}");
    if fs::metadata(&root).is_ok_and(|s| s.is_file()) {
        return Some(root);
    }
    let bin = format!("{version_dir}/bin/{binary_name}");
    if fs::metadata(&bin).is_ok_and(|s| s.is_file()) {
        return Some(bin);
    }
    find_file_recursive(Path::new(version_dir), binary_name).map(|p| p.to_string_lossy().to_string())
}

struct HlslShaderlabExtension {
    cached_dxc: Option<String>,
    cached_hlsl_validator: Option<String>,
}

impl HlslShaderlabExtension {
    /// Locates or downloads Microsoft DirectXShaderCompiler (DXC).
    fn find_dxc(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        // 0) User explicit configuration in Zed settings.json
        if let Ok(settings) = LspSettings::for_worktree(language_server_id.as_ref(), worktree) {
            let keys = ["dxc_path", "dxcPath", "dxc", "path"];
            let configured = settings
                .initialization_options
                .as_ref()
                .and_then(|opts| extract_path_from_json(opts, &keys))
                .or_else(|| {
                    settings
                        .settings
                        .as_ref()
                        .and_then(|s| extract_path_from_json(s, &keys))
                });

            if let Some(path) = resolve_configured_path(configured, worktree) {
                return Ok(path);
            }
        }

        // 1) Check system PATH
        if let Some(path) = worktree.which("dxc").or_else(|| worktree.which("dxc.exe")) {
            self.cached_dxc = Some(path.clone());
            return Ok(path);
        }

        // 2) Check cached binary
        if let Some(path) = &self.cached_dxc
            && fs::metadata(path).is_ok_and(|s| s.is_file())
        {
            return Ok(path.clone());
        }

        let (platform, arch) = zed::current_platform();

        // 3) macOS check - Microsoft DXC does not provide official macOS releases
        if matches!(platform, zed::Os::Mac) {
            return Err("Microsoft DXC does not officially distribute macOS binaries. Please install DXC locally or configure 'dxc_path' in settings.json.".to_string());
        }

        // 4) Download official release from microsoft/DirectXShaderCompiler GitHub Releases
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "microsoft/DirectXShaderCompiler",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let is_windows = matches!(platform, zed::Os::Windows);
        let asset = release
            .assets
            .iter()
            .find(|a| {
                let name = &a.name;
                if name.contains("pdb") {
                    return false;
                }
                if is_windows {
                    name.starts_with("dxc_") && name.ends_with(".zip")
                } else {
                    name.starts_with("linux_dxc_")
                        && (name.ends_with(".tar.gz") || name.ends_with(".tgz"))
                }
            })
            .ok_or_else(|| {
                format!(
                    "Compatible DXC release asset not found for {platform:?}-{arch:?} in release {}",
                    release.version
                )
            })?;

        let version_dir = format!("dxc-{}", release.version);

        let binary_path = if let Some(path) = locate_dxc(&version_dir, is_windows, arch) {
            path
        } else {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            let file_type = if is_windows {
                zed::DownloadedFileType::Zip
            } else {
                zed::DownloadedFileType::GzipTar
            };

            zed::download_file(&asset.download_url, &version_dir, file_type)
                .map_err(|e| format!("Failed to download DXC from microsoft/DirectXShaderCompiler: {e}"))?;

            locate_dxc(&version_dir, is_windows, arch)
                .ok_or_else(|| format!("DXC executable not found in '{version_dir}'"))?
        };

        let _ = zed::make_file_executable(&binary_path);

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );

        self.cached_dxc = Some(binary_path.clone());
        Ok(binary_path)
    }

    /// Locates or downloads hlsl_validator LSP binary.
    fn find_hlsl_validator(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        let (platform, arch) = zed::current_platform();
        let exe = if matches!(platform, zed::Os::Windows) { ".exe" } else { "" };
        let binary_name = format!("hlsl_validator{exe}");

        // 0) User explicit configuration in Zed settings.json
        if let Ok(settings) = LspSettings::for_worktree(language_server_id.as_ref(), worktree) {
            let keys = ["hlsl_validator_path", "validator_path", "path"];
            let configured = settings
                .initialization_options
                .as_ref()
                .and_then(|opts| extract_path_from_json(opts, &keys))
                .or_else(|| {
                    settings
                        .settings
                        .as_ref()
                        .and_then(|s| extract_path_from_json(s, &keys))
                })
                .or_else(|| {
                    settings
                        .binary
                        .and_then(|b| b.path)
                        .filter(|p| !p.trim().is_empty())
                });

            if let Some(path) = resolve_configured_path(configured, worktree) {
                return Ok(path);
            }
        }

        // 1) Check system PATH
        if let Some(path) = worktree.which(&binary_name).or_else(|| worktree.which("hlsl_validator")) {
            self.cached_hlsl_validator = Some(path.clone());
            return Ok(path);
        }

        // 2) Check cached path
        if let Some(path) = &self.cached_hlsl_validator
            && fs::metadata(path).is_ok_and(|s| s.is_file())
        {
            return Ok(path.clone());
        }

        // 3) Check user cargo bin directory (~/.cargo/bin/hlsl_validator)
        if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            let cargo_bin = format!("{home}/.cargo/bin/{binary_name}");
            if fs::metadata(&cargo_bin).is_ok_and(|s| s.is_file()) {
                self.cached_hlsl_validator = Some(cargo_bin.clone());
                return Ok(cargo_bin);
            }
        }

        // 4) Check local workspace build target if developing on the extension repository
        let root = worktree.root_path();
        if !root.is_empty() {
            let local_release = format!("{root}/hlsl_validator/target/release/{binary_name}");
            if fs::metadata(&local_release).is_ok_and(|s| s.is_file()) {
                self.cached_hlsl_validator = Some(local_release.clone());
                return Ok(local_release);
            }
        }

        // 5) Download pre-built binary from zyr1on/zed-hlsl_shaderlab-extended GitHub Releases
        let is_windows = matches!(platform, zed::Os::Windows);
        let ext = if is_windows { "zip" } else { "tar.gz" };
        let file_type = if is_windows {
            zed::DownloadedFileType::Zip
        } else {
            zed::DownloadedFileType::GzipTar
        };

        let asset_name = format!(
            "hlsl_validator-{arch}-{os}.{ext}",
            arch = match arch {
                zed::Architecture::Aarch64 => "aarch64",
                zed::Architecture::X86 => "x86",
                zed::Architecture::X8664 => "x86_64",
            },
            os = match platform {
                zed::Os::Mac => "macos",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "windows",
            }
        );

        if let Ok(release) = zed::latest_github_release(
            "zyr1on/zed-hlsl_shaderlab-extended",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        ) && let Some(asset) = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .or_else(|| {
                if matches!((platform, arch), (zed::Os::Windows, zed::Architecture::Aarch64)) {
                    release.assets.iter().find(|a| a.name == "hlsl_validator-x86_64-windows.zip")
                } else {
                    None
                }
            })
        {
            let version_dir = format!("hlsl_validator-{}", release.version);

            let binary_path = if let Some(path) = locate_validator(&version_dir, &binary_name) {
                path
            } else {
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::Downloading,
                );
                zed::download_file(&asset.download_url, &version_dir, file_type)
                    .map_err(|e| format!("Failed to download hlsl_validator from zyr1on/zed-hlsl_shaderlab-extended: {e}"))?;

                locate_validator(&version_dir, &binary_name)
                    .ok_or_else(|| format!("hlsl_validator binary not found in '{version_dir}'"))?
            };

                let _ = zed::make_file_executable(&binary_path);
                zed::set_language_server_installation_status(
                    language_server_id,
                    &zed::LanguageServerInstallationStatus::None,
                );
                self.cached_hlsl_validator = Some(binary_path.clone());
                return Ok(binary_path);
            }

        Err("hlsl_validator binary not found. Please install hlsl_validator or ensure it is accessible in PATH.".to_string())
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
