# Changelog

All notable changes to the **HLSL & ShaderLab Extended** extension will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.1] - 2026-09-17

### Added
- **Full Compute Shader & Buffer Support (Pure HLSL & Unity)**:
  - Registered `.compute` file extension in Zed and added automatic context detection for Unity compute shaders and `#pragma kernel` declarations.
  - Added `-Wno-misplaced-attributes` and `-Wno-unknown-pragmas` to DXC compiler flags and diagnostic filters, enabling clean real-time validation of `[numthreads(x, y, z)]` without misplaced attribute warnings.
  - Expanded built-in types: `RWStructuredBuffer`, `StructuredBuffer`, `AppendStructuredBuffer`, `ConsumeStructuredBuffer`, `ByteAddressBuffer`, `RWByteAddressBuffer`, `RWTexture1D`, `RWTexture2D`, `RWTexture3D`, `RWTexture2DArray`, `Texture1D..3D`.
  - Added full member method completions (`.` operator) for buffers: `Append`, `Consume`, `GetDimensions`, `IncrementCounter`, `DecrementCounter`, `Load`, `Load2..4`, `Store`, `Store2..4`, and full atomic intrinsics (`InterlockedAdd`, `InterlockedCompareExchange`, etc.).
  - Added member method completions for writable textures (`RWTexture*`): `GetDimensions`, `Load`.
  - Added `groupshared` keyword and qualifiers to autocompletion and hover documentation.
  - Complete signature help and hover documentation for barrier intrinsics (`GroupMemoryBarrierWithGroupSync`, `GroupMemoryBarrier`, `DeviceMemoryBarrier`, `AllMemoryBarrier`).
  - Full autocompletion and hover documentation for compute semantics (`SV_DispatchThreadID`, `SV_GroupID`, `SV_GroupThreadID`, `SV_GroupIndex`).
- **1:1 Signature Help Parity with GLSL Validator**:
  - Implemented parameter-count-aware signature selection (`select_best_overload`) providing exact overload matching when typing user-defined or built-in functions.
  - Multi-line parameter scanning support for functions spread across several lines.
  - Unified overload sorting and deterministic active parameter highlighting.
- **Tree-sitter Syntax Enhancements**:
  - Added language queries (`brackets.scm`, `indents.scm`, `outline.scm`) for both HLSL and ShaderLab to support native bracket matching, auto-indentation, and breadcrumbs in Zed.
- **Process & Thread Safety Protections**:
  - Added Windows process flag `CREATE_NO_WINDOW` (`0x0800_0000`) for silent Microsoft DXC background compilation.
  - Implemented 4-second hard timeout for DXC child execution with guaranteed `.kill()` and `.wait()` cleanup, eliminating hanging threads and zombie processes.
  - Added RAII `TempFileGuard` to guarantee deletion of temporary `.hlsl` files in `%TEMP%` under any exit condition.

### Changed & Optimized
- **Zero-Copy Document Cache Architecture**:
  - Migrated `doc_cache` from `Arc<Mutex<HashMap<...>>>` with full-cache cloning on keystroke to `Arc<RwLock<HashMap<String, String>>>`.
  - Replaced mass document cloning across LSP handlers (`completion`, `signatureHelp`, `hover`, `definition`, `documentSymbol`) with zero-copy read locks.
- **Debounced Multi-File Validation Queue**:
  - Upgraded pending validation storage from single-task slot to URI-keyed `HashMap<String, (ValidationTask, Instant)>`, preventing task starvation or dropped diagnostics when switching rapidly between files.
- **Performance & Short-Circuit Optimizations**:
  - Added early loop breaks (`if idx > target_line { break; }`) to `is_inside_properties_block` and `is_inside_tags_block`, turning O(N) full-document scans into instant O(target_line) lookups.
  - Refined `is_in_comment_or_string` boundary tracking to accurately preserve line-end comment state.
- **Codebase Cleanliness & Polish**:
  - Resolved all Clippy auto-deref warnings (`cargo clippy --all-targets` passes with 0 warnings).
  - Removed temporary test harnesses, debug prints, and dead code.
  - Removed deprecated `samples/` directory from repository.
  - Updated Zed extension target to `wasm32-wasip2` (Extension API 0.7.0).

---

## [0.1.0] - 2026-09-15

### Added
- Initial release of HLSL & ShaderLab Extended for Zed.
- Real-time syntax and type checking powered by Microsoft DirectXShaderCompiler (`dxc`).
- Embedded `HLSLPROGRAM`/`CGPROGRAM` extraction and line mapping for Unity ShaderLab.
- Autocompletion for struct fields, swizzling, semantics, built-in functions, and keywords.
- Hover documentation and document symbol outlines.
- Go to Definition for user functions, structs, variables, and `#include` files.
