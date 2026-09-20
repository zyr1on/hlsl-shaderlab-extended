# HLSL & ShaderLab Extended (for Zed & VS Code)

High-performance, zero-bloat language extension and LSP for **HLSL**, **Unity ShaderLab**, and **Unreal Engine Shaders** in the [Zed code editor](https://zed.dev) and [Visual Studio Code](https://code.visualstudio.com/).

Powered by a lightweight Rust LSP server (`hlsl_validator`) and the **Microsoft DirectX Shader Compiler (`dxc`)**.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Zed Extension API](https://img.shields.io/badge/Zed%20Extension%20API-v0.7.0-blue)](https://crates.io/crates/zed_extension_api)
[![Release](https://img.shields.io/github/v/release/zyr1on/hlsl-shaderlab-extended?color=green)](https://github.com/zyr1on/hlsl-shaderlab-extended/releases)

---

> [!TIP]
> ### 📦 Quick Install: Zed Editor
> 1. Download **`zed-hlsl_shaderlab-general-release.zip`** from [Latest Releases](https://github.com/zyr1on/hlsl-shaderlab-extended/releases) and extract it anywhere on your computer.
> 2. Open Zed and open the Extensions panel (`Ctrl+Shift+X` on Windows/Linux, `Cmd+Shift+X` on macOS).
> 3. Click **"Install Dev Extension"** at the top right and select the extracted folder.
> 
> *Done! The extension will load immediately without requiring Rust or Cargo.*

> [!TIP]
> ### 📦 Quick Install: Visual Studio Code
> 1. Download **`vscode-hlsl_shaderlab-general-release.vsix`** from [Latest Releases](https://github.com/zyr1on/hlsl-shaderlab-extended/releases).
> 2. In VS Code, open Extensions (`Ctrl+Shift+X` / `Cmd+Shift+X`), click the **`...`** (Views and More Actions) menu at the top of the Extensions panel, and select **"Install from VSIX..."**.
> 3. Select the downloaded `vscode-hlsl_shaderlab-general-release.vsix` file.
> 
> *Done! VS Code will automatically download the language server binary in the background upon opening an HLSL or ShaderLab file.*

> [!TIP]
> ### Automatic Installation (Zero Setup)
> **Everything is automatic!** The extension automatically handles its dependencies:
> 1. **`hlsl_validator`**: Automatically downloaded from GitHub Releases on first launch if not found in `PATH`.
> 2. **Microsoft DXC**: Automatically downloaded from Microsoft's official releases on Windows and Linux if not found in `PATH`.

---

## Features

- **Compiler Diagnostics (Microsoft DXC):** Real-time syntax and type checking against DirectX Shader Model 6.3+ (HLSL 2021) with 120ms debounced execution.
- **Unity ShaderLab Support:** Automatic extraction and validation of embedded `HLSLPROGRAM`/`CGPROGRAM` blocks with precise line mapping.
- **Smart Autocomplete:**
  - Struct fields and nested member access (`output.`)
  - Vector swizzling components (`.xy`, `.xyz`, `.xyzw`, `.rgba`)
  - Texture and buffer methods (`.Sample()`, `.Load()`, `.GetDimensions()`)
  - System-value semantics (`SV_Position`, `SV_Target`, `TEXCOORD`)
  - ShaderLab properties, render states (`Cull`, `ZWrite`, `Blend`), and tags
- **Intrinsics & Engine Documentation:** 198+ HLSL intrinsics, Unity (URP/HDRP/Built-in) helpers, and Unreal Engine shader functions with full signature help and hover docs.
- **Document Symbols & Breadcrumbs:** Instant outline for functions, structs, cbuffers, and SubShader passes.
- **Code Formatting:** Clean indentation, bracket alignment, and preprocessor formatting via `format_on_save`.
- **Go to Definition:** Jump directly to user functions, structs, variables, or `#include` files with `F12`.

---

## Supported Languages & Extensions

| Language | File Extensions |
| :--- | :--- |
| **HLSL** | `.hlsl`, `.hlsli`, `.fx`, `.usf`, `.ush`, `.compute` |
| **ShaderLab** | `.shader`, `.cginc` |

---

## Platform Support

| Platform | Diagnostics (DXC) | Autocomplete, Hover, Symbols, Formatting |
| :--- | :--- | :--- |
| **Windows (x86_64 / ARM64)** | Supported (Auto-downloaded or system `PATH`) | Supported |
| **Linux (x86_64)** | Supported (Auto-downloaded or system `PATH`) | Supported |
| **macOS (Apple Silicon / Intel)** | **Not Supported** | Supported |

### Why is DXC Diagnostics Not Supported on macOS?
Microsoft DirectXShaderCompiler (`dxc`) relies on DirectX and Windows/Linux LLVM backends. Microsoft does not distribute official precompiled DXC binaries for macOS. All non-compiler language features (autocomplete, hover, document outline, signature help, formatting, syntax highlighting) work fully across all platforms including macOS.

---

## Setup & Configuration

### Zed Settings (`settings.json`)

To configure HLSL & ShaderLab settings in Zed, open your `settings.json` (`Ctrl+,` or `Cmd+,` -> **Open Settings**):

```json
{
  "languages": {
    "HLSL": {
      "tab_size": 4,                        // Indentation space width (e.g., 2, 4, 8)
      "format_on_save": "on",               // Auto-format HLSL shader on save ("off" | "on")
      "formatter": {
        "language_server": {
          "name": "hlsl_validator"          // Route formatting to hlsl_validator LSP
        }
      }
    },
    "ShaderLab": {
      "tab_size": 4,                        // Indentation space width (e.g., 2, 4, 8)
      "format_on_save": "on",               // Auto-format ShaderLab file on save ("off" | "on")
      "formatter": {
        "language_server": {
          "name": "hlsl_validator"          // Route formatting to hlsl_validator LSP
        }
      }
    }
  },
  "lsp": {
    "hlsl_validator": {
      "binary": {
        "path": ""                          // Custom path to hlsl_validator binary (empty for auto-download / PATH)
      },
      "initialization_options": {
        "dxc_path": "",                     // Custom path to Microsoft DXC dxc.exe / dxc (empty for auto-download / PATH)
        "hlsl_validator_path": ""           // Alternative custom path to hlsl_validator
      }
    }
  }
}
```

### Visual Studio Code Settings (`settings.json`)

To configure HLSL & ShaderLab settings in VS Code, open your User or Workspace `settings.json` (`Ctrl+Shift+P` -> `Preferences: Open User Settings (JSON)`):

```json
{
  "[hlsl]": {
    "editor.tabSize": 4,                    // Indentation space width (e.g., 2, 4, 8)
    "editor.formatOnSave": true             // Auto-format HLSL shader document on save (true | false)
  },
  "[shaderlab]": {
    "editor.tabSize": 4,                    // Indentation space width (e.g., 2, 4, 8)
    "editor.formatOnSave": true             // Auto-format ShaderLab document on save (true | false)
  },
  "hlslExtended.validatorPath": "",          // Custom executable path to hlsl_validator (empty for auto-download / PATH)
  "hlslExtended.dxcPath": "",                // Custom executable path to Microsoft DXC dxc.exe / dxc (empty for auto-download / PATH)
  "hlslExtended.trace.server": "off"         // Trace LSP communication in Output panel ("off" | "messages" | "verbose")
}
```

---

## Performance & Architecture

- **LSP Binary Size:** ~985 KB native binary.
- **WASM Extension:** Compiled for `wasm32-wasip2` (Zed Extension API 0.7.0+).
- **Memory Footprint:** < 2 MB RAM usage.
- **Idle CPU:** 0.0% CPU usage with non-polling event loop.
- **Speed:** DXC runs with `-O0` to bypass codegen optimization passes for instant diagnostic feedback.

---

## 🛠️ Building from Source

If you want to build and hack on HLSL & ShaderLab Extended locally, you can easily compile all components from source:

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (latest stable via `rustup`)
- [Node.js](https://nodejs.org/) (v18+ with npm, for the VS Code extension)
- WebAssembly Target for Zed:
  ```bash
  rustup target add wasm32-wasip2
  ```

### 1. Build the Language Server (`hlsl_validator`)
To compile the standalone Rust LSP engine:
```bash
cargo build --release --manifest-path hlsl_validator/Cargo.toml
```
The compiled binary will be located at:
- **Windows:** `target/release/hlsl_validator.exe`
- **Linux / macOS:** `target/release/hlsl_validator`

### 2. Build the Zed Extension (`.wasm`)
To compile the WebAssembly extension for Zed Editor:
```bash
cargo build --release --target wasm32-wasip2 --manifest-path editors/zed/Cargo.toml
```
The compiled `.wasm` binary will be at:
`target/wasm32-wasip2/release/zed_hlsl_shaderlab.wasm`

### 3. Build the VS Code Extension (`.vsix`)
To compile and package the VS Code extension:
```bash
cd editors/vscode
npm install
npm run compile
npx @vscode/vsce package --no-dependencies
```
This produces `vscode-hlsl_shaderlab-general-release.vsix` (under 500 KB, zero bundled binaries).

### 4. Running Tests
To run all unit and integration tests across the workspace:
```bash
cargo test --workspace
```

---

## Author & License

- **Author:** Semih Özdemir ([@zyr1on](https://github.com/zyr1on)) - `semihozdmirr@gmail.com`
- **License:** [MIT License](LICENSE)

