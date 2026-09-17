# HLSL & ShaderLab Extended for Zed

High-performance, zero-bloat language extension and LSP for **HLSL**, **Unity ShaderLab**, and **Unreal Engine Shaders** in the [Zed code editor](https://zed.dev).

Powered by a lightweight Rust LSP server (`hlsl_validator`) and the **Microsoft DirectX Shader Compiler (`dxc`)**.

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

### Automatic Installation
The extension automatically handles its dependencies:
1. **`hlsl_validator`**: Automatically downloaded from GitHub Releases on first launch if not found in `PATH`.
2. **Microsoft DXC**: Automatically downloaded from Microsoft's official releases on Windows and Linux if not found in `PATH`.

### Custom Configuration (`settings.json`)
You can specify custom binary paths or formatting preferences in your Zed settings:

```json
{
  "languages": {
    "HLSL": {
      "format_on_save": "on"
    },
    "ShaderLab": {
      "format_on_save": "on"
    }
  },
  "lsp": {
    "hlsl_validator": {
      "initialization_options": {
        "dxc_path": "C:\\Program Files (x86)\\Windows Kits\\10\\bin\\x64\\dxc.exe"
      }
    }
  }
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

## License

MIT License. Created by Semih Ozdemir.
