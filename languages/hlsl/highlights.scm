; HLSL highlights.scm for Zed
; Based on tree-sitter-hlsl (inherits tree-sitter-cpp)

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
  "in"
  "out"
  "inout"
  "uniform"
  "groupshared"
  "precise"
  "row_major"
  "column_major"
] @keyword

[
  (true)
  (false)
] @boolean

[
  (type_identifier)
  (primitive_type)
  (sized_type_specifier)
] @type

((identifier) @type
  (#match? @type "^(float[1-4]?|half[1-4]?|int[1-4]?|uint[1-4]?|bool[1-4]?|double|float[2-4]x[2-4]|half[2-4]x[2-4]|matrix|Texture[1-3]D.*|TextureCube.*|SamplerState.*|SamplerComparisonState|sampler2D|samplerCUBE|StructuredBuffer|RWStructuredBuffer|ByteAddressBuffer|RWByteAddressBuffer|cbuffer|tbuffer)$"))

(semantics) @attribute

(function_declarator
  declarator: (identifier) @function)

(call_expression
  function: (identifier) @function)

(call_expression
  function: (field_expression
    field: (field_identifier) @function))

(field_identifier) @property

(identifier) @variable

(number_literal) @number

(string_literal) @string

(comment) @comment

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
  ";"
  ","
  ":"
] @operator
