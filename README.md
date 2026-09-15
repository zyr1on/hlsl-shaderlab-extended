# HLSL & ShaderLab Extended for Zed

High-performance, zero-bloat language extension and LSP for **HLSL**, **Unity ShaderLab**, and **Unreal Engine Shaders** in the [Zed code editor](https://zed.dev).

Powered by **Microsoft DirectX Shader Compiler (`dxc`)** and a standalone, ultra-lightweight Rust LSP server (`hlsl_validator`).

---

## Features

### 🚀 Ultra-Lightweight & Blazing Fast
- **Zero-Bloat Architecture:** Complete LSP server compiles down to **~700 KB** native binary with **< 2 MB RAM** usage and **0% idle CPU**.
- **No Zombies, No Leaks:** Safe process spawning and strict temporary file lifecycle management.
- **120ms Debounced Diagnostics:** Real-time compiler feedback without UI freezes or resource hogs while typing.

### 🔍 Microsoft DXC Diagnostics
- **Pure HLSL (`.hlsl`, `.hlsli`, `.fx`, `.usf`, `.ush`):** Full syntax and semantic type checking compiled against modern DirectX Shader Model 6.3+ via Microsoft DXC.
- **Unity ShaderLab (`.shader`, `.cginc`):** Automatically extracts and compiles `HLSLPROGRAM...ENDHLSL` and `CGPROGRAM...ENDCG` blocks with exact line-number translation.
- **Automatic Project Include Resolution (`-I`):**
  - **Unity Projects:** Automatically detects `Assets/`, `Packages/`, and `Library/PackageCache/`.
  - **Unreal Projects:** Automatically detects `Shaders/` and project roots.
  - **Local Folders:** Automatically includes the shader's parent directory.

### 💡 Smart Autocomplete & Swizzling
- **Struct Member Access:** Typing `output.` or `output.po` inspects the variable's type and proposes its struct fields (`position`, `color`, etc.).
- **Vector Swizzling:** Typing `output.position.` proposes vector swizzle components (`.xy`, `.xyz`, `.xyzw`, `.rgba`).
- **Texture & Buffer Methods:** Automatic completion for `.Sample()`, `.SampleLevel()`, `.SampleBias()`, `.Load()`, `.GetDimensions()`.
- **System-Value Semantics:** Typing `:` proposes HLSL semantics (`SV_Position`, `SV_Target`, `POSITION`, `TEXCOORD0`, etc.).
- **ShaderLab Contextual Autocomplete:**
  - Property types: `Color`, `Vector`, `Float`, `Int`, `Range(0, 1)`, `2D`, `Cube`, `3D`.
  - Render states: `Cull` (`Back`, `Front`, `Off`), `ZWrite` (`On`, `Off`), `ZTest` (`LEqual`, `Always`, `Equal`, etc.), `Blend` (`SrcAlpha OneMinusSrcAlpha`, `One One`, `Off`).
  - Unity Tags: `"RenderType"="Opaque"`, `"Queue"="Geometry"`, `"RenderPipeline"="UniversalPipeline"`, `"LightMode"="UniversalForward"`.

### 📚 198+ Built-in Functions & Signature Help
- Exhaustive documentation and signatures for:
  - **Microsoft HLSL Intrinsics:** Math, trigonometry, wave intrinsics (`WaveActiveMin`, `WaveReadLaneFirst`), derivatives (`ddx_fine`, `fwidth_coarse`), bitwise operations (`ubfe`, `ibfe`, `msad4`), barriers, and `Interlocked` atomics.
  - **Unity URP / HDRP / Built-in:** `TransformObjectToHClip`, `TRANSFORM_TEX`, `ComputeScreenPos`, `SAMPLE_TEXTURE2D_LOD`, `LightingPhysicallyBased`, `UniversalFragmentPBR`, `LinearEyeDepth`.
  - **Unreal Engine:** `GetWorldPosition`, `GetWorldNormal`, `RotateAboutAxis`, `AntialiasedTextureMask`, `UnitVectorToOctahedron`, `Luminance`, `RGBToHSV`.
- Active parameter highlighting as you type inside function parentheses: `lerp(a, |)`.

### 🗺️ Document Symbols (Outline & Breadcrumbs)
- Full support for `textDocument/documentSymbol`.
- Structs, constant buffers (`cbuffer`), functions, and ShaderLab `SubShader`/`Pass` blocks appear directly in Zed's **Outline** panel and editor **Breadcrumbs**.

### 🎨 Fast Document Formatting
- Format shaders on demand (`Shift+Alt+F`) or on save (`format_on_save`).
- Aligns braces `{ }`, indents nested blocks cleanly, preserves preprocessor directives (`#pragma`, `#include`) at column 0, and trims trailing whitespace.

### 🔗 Go to Definition & Hover Docs
- Press `F12` on any user function, struct, or variable to jump directly to its declaration.
- Press `F12` on `#include "MyLibrary.hlsl"` lines to jump straight to the included file.
- Hover over any intrinsic, type, keyword, or user variable for rich markdown documentation.

---

## Supported Languages & File Extensions

| Language | File Extensions |
| :--- | :--- |
| **HLSL** | `.hlsl`, `.hlsli`, `.fx`, `.usf`, `.ush` |
| **ShaderLab** | `.shader`, `.cginc` |

---

## Installation & Setup

### 1. Requirements
Ensure Microsoft DXC is installed:
- **Windows:** Installed with the Windows SDK or download from [microsoft/DirectXShaderCompiler Releases](https://github.com/microsoft/DirectXShaderCompiler/releases).
- Ensure `dxc.exe` is in your `PATH` or specify its path in Zed settings.

### 2. Install Extension in Zed
1. Clone or copy this repository to your local drive (e.g. `D:\zed-hlsl-shaderlab`).
2. Build the validator binary:
   ```bash
   cd hlsl_validator
   cargo build --release
   cp target/release/hlsl_validator ~/.cargo/bin/hlsl_validator
   ```
3. In Zed:
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS).
   - Select **Extensions: Install Dev Extension**.
   - Pick the directory `D:\zed-hlsl-shaderlab`.

### 3. Optional Zed Settings (`settings.json`)
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

## License
MIT License. Created by Semih Ozdemir.
