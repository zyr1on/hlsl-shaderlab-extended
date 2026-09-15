; ShaderLab highlights.scm for Zed
; High-performance syntax highlighting for Unity ShaderLab and embedded HLSL
; Based on tree-sitter-hlsl (inherits tree-sitter-cpp)

; =============================================================================
; 1. Base Fallback (Lowest Precedence)
; =============================================================================
(identifier) @variable

; =============================================================================
; 2. Standard Grammar Keywords, Types, Booleans, Literals & Comments
; =============================================================================
[
  "break"
  "case"
  "const"
  "continue"
  "default"
  "do"
  "else"
  "enum"
  "extern"
  "for"
  "if"
  "inline"
  "return"
  "sizeof"
  "static"
  "struct"
  "switch"
  "typedef"
  "union"
  "volatile"
  "while"
  "#define"
  "#elif"
  "#else"
  "#endif"
  "#if"
  "#ifdef"
  "#ifndef"
  "#include"
  "discard"
  "cbuffer"
  "register"
  "in"
  "out"
  "inout"
  "uniform"
  "groupshared"
  "precise"
  "row_major"
  "column_major"
] @keyword

(preproc_directive) @keyword

[
  (true)
  (false)
] @boolean

[
  (type_identifier)
  (primitive_type)
  (sized_type_specifier)
] @type

(semantics) @attribute
(number_literal) @number
(string_literal) @string
(comment) @comment

; =============================================================================
; 3. Structural Constructs: Structs, Functions, Parameters, Fields
; =============================================================================
(struct_specifier
  name: (type_identifier) @type)

(cbuffer_specifier
  name: (type_identifier) @type)

; Function return type & parameters
(function_definition
  type: (type_identifier) @type)

(parameter_declaration
  type: (type_identifier) @type)

(parameter_declaration
  declarator: (identifier) @variable.parameter)

; Function declarations and call expressions
(function_declarator
  declarator: (identifier) @function)

(call_expression
  function: (identifier) @function)

(call_expression
  function: (field_expression
    field: (field_identifier) @function))

(field_identifier) @property

; =============================================================================
; 4. High-Precedence Specific Overrides (Types, Macros, Engine Built-ins)
; =============================================================================

; A. HLSL Primitives, Vectors, Matrices, Textures, Samplers & ShaderLab Types
((identifier) @type
  (#match? @type "^(float[1-4]?|half[1-4]?|int[1-4]?|uint[1-4]?|bool[1-4]?|double|float[2-4]x[2-4]|half[2-4]x[2-4]|matrix|Texture[1-3]D.*|TextureCube.*|SamplerState.*|SamplerComparisonState|sampler2D|samplerCUBE|StructuredBuffer|RWStructuredBuffer|ByteAddressBuffer|RWByteAddressBuffer|cbuffer|tbuffer|Color|Float|Int|Integer|Range|Cube|Vector|Rect|2DArray|CubeArray|UnityPer.*|Varyings|Attributes|AppData.*|SurfaceOutput.*)$"))

; Match 2D and 3D property types in ShaderLab (which tree-sitter parses as number literals)
((number_literal) @type
  (#match? @type "^(2D|3D)$"))

; B. Engine Macros & Keywords (CBUFFER_START, CBUFFER_END, packoffset)
((identifier) @keyword
  (#match? @keyword "^(CBUFFER_START|CBUFFER_END|packoffset)$"))

; C. Preprocessor Directives & Pragma Keywords (#pragma vertex, fragment, target, multi_compile, etc.)
((identifier) @keyword
  (#match? @keyword "^(pragma|define|include|ifdef|ifndef|endif|undef|vertex|fragment|geometry|hull|domain|compute|kernel|surface|multi_compile.*|shader_feature.*|target|only_renderers|exclude_renderers|require|enable_d3d11_debug_symbols|disable_fastmath|skip_variants)$"))

; D. Unity Built-in Functions, Texture Sampling Macros & Shader Helpers
((identifier) @function
  (#match? @function "^(SAMPLE_TEXTURE2D.*|TEXTURE2D.*|TEXTURECUBE.*|TEXTURE3D.*|SAMPLER.*|Transform.*|GetVertexPositionInputs|GetVertexNormalInputs|GetWorldSpaceViewDir|GetWorldSpaceNormalizeViewDir|GetWorldSpacePosition|GetWorldNormal|GetMainLight|GetAdditionalLight.*|UniversalFragment.*|SampleShadowmap|SampleSH|SampleSceneColor|SampleSceneDepth|LinearEyeDepth|Linear01Depth|SafeNormalize|AlphaDiscard|UNITY_.*|SHADOW_CASTER_FRAGMENT)$"))

; E. ShaderLab Block & Command Keywords
((identifier) @keyword
  (#match? @keyword "^(Shader|Properties|SubShader|Pass|Tags|Fallback|CustomEditor|ZWrite|ZTest|ZClip|Cull|Blend|BlendOp|ColorMask|Offset|Conservative|AlphaToMask|Stencil|Ref|ReadMask|WriteMask|Comp|UsePass|GrabPass|LOD|Name|HLSLPROGRAM|ENDHLSL|CGPROGRAM|ENDCG|HLSLINCLUDE)$"))

; F. ShaderLab Attributes
((identifier) @attribute
  (#match? @attribute "^(MainColor|MainTexture|HDR|HideInInspector|NoScaleOffset|Normal|SingleLineTexture|PerRendererData|MaterialToggle|Toggle|KeywordEnum|Enum|Space|Header|IntRange|Tooltip)$"))

; G. ShaderLab Render State Constants & Values
((identifier) @constant
  (#match? @constant "^(On|Off|True|False|LEqual|Always|Equal|Less|Greater|GEqual|NotEqual|Back|Front|SrcAlpha|OneMinusSrcAlpha|One|Zero|DstColor|SrcColor|OneMinusDstColor|OneMinusSrcColor|Add|Sub|RevSub|Min|Max|Keep|Replace|IncrSat|DecrSat|Invert|IncrWrap|DecrWrap)$"))

; =============================================================================
; 5. Punctuation Delimiters & Brackets
; =============================================================================
[
  ";"
  ","
] @punctuation.delimiter

[
  "{"
  "}"
  "["
  "]"
  "("
  ")"
] @punctuation.bracket

; =============================================================================
; 6. Operators
; =============================================================================
[
  "="
  "+"
  "-"
  "*"
  "/"
  "%"
  "=="
  "!="
  "<"
  "<="
  ">"
  ">="
  "&&"
  "||"
  "!"
  "&"
  "|"
  "^"
  "~"
  "<<"
  ">>"
  "+="
  "-="
  "*="
  "/="
  "."
  ":"
] @operator
