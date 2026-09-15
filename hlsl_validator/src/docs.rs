// hlsl_validator - docs.rs
// Comprehensive Microsoft HLSL Intrinsics & Unity ShaderLab built-in database

pub struct BuiltinOverload {
    pub label: &'static str,
    pub params: &'static [&'static str],
}

pub struct BuiltinFunction {
    pub name: &'static str,
    pub description: &'static str,
    pub overloads: &'static [BuiltinOverload],
}

pub static BUILTIN_FUNCTIONS: &[BuiltinFunction] = &[
    // ------------------------------------------------------------------------
    // Vector Constructors
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "float4",
        description: "### `float4`\n*HLSL Vector Constructor*\n\nConstructs a 4-component floating-point vector from scalar or vector components.",
        overloads: &[
            BuiltinOverload { label: "float4(float x, float y, float z, float w)", params: &["float x", "float y", "float z", "float w"] },
            BuiltinOverload { label: "float4(float3 xyz, float w)", params: &["float3 xyz", "float w"] },
            BuiltinOverload { label: "float4(float x, float3 yzw)", params: &["float x", "float3 yzw"] },
            BuiltinOverload { label: "float4(float2 xy, float2 zw)", params: &["float2 xy", "float2 zw"] },
            BuiltinOverload { label: "float4(float2 xy, float z, float w)", params: &["float2 xy", "float z", "float w"] },
            BuiltinOverload { label: "float4(float s)", params: &["float s"] },
        ],
    },
    BuiltinFunction {
        name: "float3",
        description: "### `float3`\n*HLSL Vector Constructor*\n\nConstructs a 3-component floating-point vector from scalar or vector components.",
        overloads: &[
            BuiltinOverload { label: "float3(float x, float y, float z)", params: &["float x", "float y", "float z"] },
            BuiltinOverload { label: "float3(float2 xy, float z)", params: &["float2 xy", "float z"] },
            BuiltinOverload { label: "float3(float x, float2 yz)", params: &["float x", "float2 yz"] },
            BuiltinOverload { label: "float3(float s)", params: &["float s"] },
        ],
    },
    BuiltinFunction {
        name: "float2",
        description: "### `float2`\n*HLSL Vector Constructor*\n\nConstructs a 2-component floating-point vector.",
        overloads: &[
            BuiltinOverload { label: "float2(float x, float y)", params: &["float x", "float y"] },
            BuiltinOverload { label: "float2(float s)", params: &["float s"] },
        ],
    },
    BuiltinFunction {
        name: "half4",
        description: "### `half4`\n*HLSL Half-Precision Vector Constructor*\n\nConstructs a 4-component half-precision vector.",
        overloads: &[
            BuiltinOverload { label: "half4(half x, half y, half z, half w)", params: &["half x", "half y", "half z", "half w"] },
            BuiltinOverload { label: "half4(half3 xyz, half w)", params: &["half3 xyz", "half w"] },
            BuiltinOverload { label: "half4(half2 xy, half2 zw)", params: &["half2 xy", "half2 zw"] },
            BuiltinOverload { label: "half4(half s)", params: &["half s"] },
        ],
    },
    BuiltinFunction {
        name: "half3",
        description: "### `half3`\n*HLSL Half-Precision Vector Constructor*\n\nConstructs a 3-component half-precision vector.",
        overloads: &[
            BuiltinOverload { label: "half3(half x, half y, half z)", params: &["half x", "half y", "half z"] },
            BuiltinOverload { label: "half3(half2 xy, half z)", params: &["half2 xy", "half z"] },
            BuiltinOverload { label: "half3(half s)", params: &["half s"] },
        ],
    },
    BuiltinFunction {
        name: "half2",
        description: "### `half2`\n*HLSL Half-Precision Vector Constructor*\n\nConstructs a 2-component half-precision vector.",
        overloads: &[
            BuiltinOverload { label: "half2(half x, half y)", params: &["half x", "half y"] },
            BuiltinOverload { label: "half2(half s)", params: &["half s"] },
        ],
    },
    // ------------------------------------------------------------------------
    // Interpolation & Clamping (Common HLSL Intrinsics)
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "lerp",
        description: "### `lerp`\n*Microsoft HLSL Intrinsic*\n\nPerforms a linear interpolation between `x` and `y` using factor `s`.\n\n$$\\text{lerp}(x, y, s) = x + s \\cdot (y - x)$$\n\n**Parameters:**\n* `x`: Start value (scalar or vector).\n* `y`: End value (scalar or vector).\n* `s`: Interpolation weight factor, typically in $[0.0, 1.0]$.",
        overloads: &[
            BuiltinOverload { label: "float lerp(float x, float y, float s)", params: &["float x", "float y", "float s"] },
            BuiltinOverload { label: "float2 lerp(float2 x, float2 y, float2 s)", params: &["float2 x", "float2 y", "float2 s"] },
            BuiltinOverload { label: "float2 lerp(float2 x, float2 y, float s)", params: &["float2 x", "float2 y", "float s"] },
            BuiltinOverload { label: "float3 lerp(float3 x, float3 y, float3 s)", params: &["float3 x", "float3 y", "float3 s"] },
            BuiltinOverload { label: "float3 lerp(float3 x, float3 y, float s)", params: &["float3 x", "float3 y", "float s"] },
            BuiltinOverload { label: "float4 lerp(float4 x, float4 y, float4 s)", params: &["float4 x", "float4 y", "float4 s"] },
            BuiltinOverload { label: "float4 lerp(float4 x, float4 y, float s)", params: &["float4 x", "float4 y", "float s"] },
            BuiltinOverload { label: "half lerp(half x, half y, half s)", params: &["half x", "half y", "half s"] },
            BuiltinOverload { label: "half3 lerp(half3 x, half3 y, half s)", params: &["half3 x", "half3 y", "half s"] },
            BuiltinOverload { label: "half4 lerp(half4 x, half4 y, half s)", params: &["half4 x", "half4 y", "half s"] },
        ],
    },
    BuiltinFunction {
        name: "saturate",
        description: "### `saturate`\n*Microsoft HLSL Intrinsic*\n\nClamps the specified value to the range $[0.0, 1.0]$. Extremely fast on GPU hardware.\n\n$$\\text{saturate}(x) = \\min(\\max(x, 0.0), 1.0)$$\n\n**Parameters:**\n* `x`: Input value to clamp.",
        overloads: &[
            BuiltinOverload { label: "float saturate(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 saturate(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 saturate(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 saturate(float4 x)", params: &["float4 x"] },
            BuiltinOverload { label: "half saturate(half x)", params: &["half x"] },
            BuiltinOverload { label: "half3 saturate(half3 x)", params: &["half3 x"] },
            BuiltinOverload { label: "half4 saturate(half4 x)", params: &["half4 x"] },
        ],
    },
    BuiltinFunction {
        name: "clamp",
        description: "### `clamp`\n*Microsoft HLSL Intrinsic*\n\nClamps value `x` to the specified `[min, max]` range.\n\n$$\\text{clamp}(x, min, max) = \\min(\\max(x, min), max)$$\n\n**Parameters:**\n* `x`: Input value to clamp.\n* `min`: Minimum allowed value.\n* `max`: Maximum allowed value.",
        overloads: &[
            BuiltinOverload { label: "float clamp(float x, float min, float max)", params: &["float x", "float min", "float max"] },
            BuiltinOverload { label: "float2 clamp(float2 x, float2 min, float2 max)", params: &["float2 x", "float2 min", "float2 max"] },
            BuiltinOverload { label: "float3 clamp(float3 x, float3 min, float3 max)", params: &["float3 x", "float3 min", "float3 max"] },
            BuiltinOverload { label: "float4 clamp(float4 x, float4 min, float4 max)", params: &["float4 x", "float4 min", "float4 max"] },
        ],
    },
    BuiltinFunction {
        name: "step",
        description: "### `step`\n*Microsoft HLSL Intrinsic*\n\nCompares two values, returning 0 if `x < y`, or 1 otherwise.\n\n**Parameters:**\n* `y`: The threshold/edge value.\n* `x`: Value to compare against `y`.",
        overloads: &[
            BuiltinOverload { label: "float step(float y, float x)", params: &["float y", "float x"] },
            BuiltinOverload { label: "float2 step(float2 y, float2 x)", params: &["float2 y", "float2 x"] },
            BuiltinOverload { label: "float3 step(float3 y, float3 x)", params: &["float3 y", "float3 x"] },
            BuiltinOverload { label: "float4 step(float4 y, float4 x)", params: &["float4 y", "float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "smoothstep",
        description: "### `smoothstep`\n*Microsoft HLSL Intrinsic*\n\nReturns a smooth Hermite interpolation between 0 and 1, if `x` is in the range `[min, max]`.\n\n**Parameters:**\n* `min`: Lower edge.\n* `max`: Upper edge.\n* `x`: Input value.",
        overloads: &[
            BuiltinOverload { label: "float smoothstep(float min, float max, float x)", params: &["float min", "float max", "float x"] },
            BuiltinOverload { label: "float2 smoothstep(float2 min, float2 max, float2 x)", params: &["float2 min", "float2 max", "float2 x"] },
            BuiltinOverload { label: "float3 smoothstep(float3 min, float3 max, float3 x)", params: &["float3 min", "float3 max", "float3 x"] },
            BuiltinOverload { label: "float4 smoothstep(float4 min, float4 max, float4 x)", params: &["float4 min", "float4 max", "float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "frac",
        description: "### `frac`\n*Microsoft HLSL Intrinsic*\n\nReturns the fractional (decimal) part of `x`.\n\n$$\\text{frac}(x) = x - \\lfloor x \\rfloor$$\n\n**Parameters:**\n* `x`: Floating point input.",
        overloads: &[
            BuiltinOverload { label: "float frac(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 frac(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 frac(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 frac(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "fmod",
        description: "### `fmod`\n*Microsoft HLSL Intrinsic*\n\nReturns the floating-point remainder of `x / y`.\n\n**Parameters:**\n* `x`: Numerator.\n* `y`: Denominator.",
        overloads: &[
            BuiltinOverload { label: "float fmod(float x, float y)", params: &["float x", "float y"] },
            BuiltinOverload { label: "float2 fmod(float2 x, float2 y)", params: &["float2 x", "float2 y"] },
            BuiltinOverload { label: "float3 fmod(float3 x, float3 y)", params: &["float3 x", "float3 y"] },
            BuiltinOverload { label: "float4 fmod(float4 x, float4 y)", params: &["float4 x", "float4 y"] },
        ],
    },
    BuiltinFunction {
        name: "abs",
        description: "### `abs`\n*Microsoft HLSL Intrinsic*\n\nReturns the absolute value of `x`.",
        overloads: &[
            BuiltinOverload { label: "float abs(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 abs(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 abs(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 abs(float4 x)", params: &["float4 x"] },
            BuiltinOverload { label: "int abs(int x)", params: &["int x"] },
        ],
    },
    BuiltinFunction {
        name: "sign",
        description: "### `sign`\n*Microsoft HLSL Intrinsic*\n\nReturns -1 if `x < 0`, 0 if `x == 0`, and 1 if `x > 0`.",
        overloads: &[
            BuiltinOverload { label: "float sign(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 sign(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 sign(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 sign(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "floor",
        description: "### `floor`\n*Microsoft HLSL Intrinsic*\n\nReturns the largest integer that is less than or equal to `x`.",
        overloads: &[
            BuiltinOverload { label: "float floor(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 floor(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 floor(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 floor(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "ceil",
        description: "### `ceil`\n*Microsoft HLSL Intrinsic*\n\nReturns the smallest integer that is greater than or equal to `x`.",
        overloads: &[
            BuiltinOverload { label: "float ceil(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 ceil(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 ceil(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 ceil(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "round",
        description: "### `round`\n*Microsoft HLSL Intrinsic*\n\nRounds `x` to the nearest integer.",
        overloads: &[
            BuiltinOverload { label: "float round(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 round(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 round(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 round(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "min",
        description: "### `min`\n*Microsoft HLSL Intrinsic*\n\nReturns the lesser of `x` and `y`.",
        overloads: &[
            BuiltinOverload { label: "float min(float x, float y)", params: &["float x", "float y"] },
            BuiltinOverload { label: "float2 min(float2 x, float2 y)", params: &["float2 x", "float2 y"] },
            BuiltinOverload { label: "float3 min(float3 x, float3 y)", params: &["float3 x", "float3 y"] },
            BuiltinOverload { label: "float4 min(float4 x, float4 y)", params: &["float4 x", "float4 y"] },
        ],
    },
    BuiltinFunction {
        name: "max",
        description: "### `max`\n*Microsoft HLSL Intrinsic*\n\nReturns the greater of `x` and `y`.",
        overloads: &[
            BuiltinOverload { label: "float max(float x, float y)", params: &["float x", "float y"] },
            BuiltinOverload { label: "float2 max(float2 x, float2 y)", params: &["float2 x", "float2 y"] },
            BuiltinOverload { label: "float3 max(float3 x, float3 y)", params: &["float3 x", "float3 y"] },
            BuiltinOverload { label: "float4 max(float4 x, float4 y)", params: &["float4 x", "float4 y"] },
        ],
    },

    // ------------------------------------------------------------------------
    // Geometric / Vector Functions
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "dot",
        description: "### `dot`\n*Microsoft HLSL Intrinsic*\n\nComputes the dot product of two vectors.\n\n$$\\text{dot}(x, y) = \\sum x_i \\cdot y_i$$\n\n**Parameters:**\n* `x`: First vector.\n* `y`: Second vector.",
        overloads: &[
            BuiltinOverload { label: "float dot(float x, float y)", params: &["float x", "float y"] },
            BuiltinOverload { label: "float dot(float2 x, float2 y)", params: &["float2 x", "float2 y"] },
            BuiltinOverload { label: "float dot(float3 x, float3 y)", params: &["float3 x", "float3 y"] },
            BuiltinOverload { label: "float dot(float4 x, float4 y)", params: &["float4 x", "float4 y"] },
            BuiltinOverload { label: "half dot(half3 x, half3 y)", params: &["half3 x", "half3 y"] },
            BuiltinOverload { label: "half dot(half4 x, half4 y)", params: &["half4 x", "half4 y"] },
        ],
    },
    BuiltinFunction {
        name: "cross",
        description: "### `cross`\n*Microsoft HLSL Intrinsic*\n\nComputes the cross product of two 3-component vectors.\n\n$$\\text{cross}(x, y) = (x_y y_z - x_z y_y, \\, x_z y_x - x_x y_z, \\, x_x y_y - x_y y_x)$$\n\n**Parameters:**\n* `x`: First 3D vector.\n* `y`: Second 3D vector.",
        overloads: &[
            BuiltinOverload { label: "float3 cross(float3 x, float3 y)", params: &["float3 x", "float3 y"] },
            BuiltinOverload { label: "half3 cross(half3 x, half3 y)", params: &["half3 x", "half3 y"] },
        ],
    },
    BuiltinFunction {
        name: "normalize",
        description: "### `normalize`\n*Microsoft HLSL Intrinsic*\n\nNormalizes the specified vector so that its length equals 1.\n\n$$\\text{normalize}(v) = \\frac{v}{\\text{length}(v)}$$\n\n**Parameters:**\n* `v`: Vector to normalize.",
        overloads: &[
            BuiltinOverload { label: "float normalize(float v)", params: &["float v"] },
            BuiltinOverload { label: "float2 normalize(float2 v)", params: &["float2 v"] },
            BuiltinOverload { label: "float3 normalize(float3 v)", params: &["float3 v"] },
            BuiltinOverload { label: "float4 normalize(float4 v)", params: &["float4 v"] },
            BuiltinOverload { label: "half3 normalize(half3 v)", params: &["half3 v"] },
        ],
    },
    BuiltinFunction {
        name: "length",
        description: "### `length`\n*Microsoft HLSL Intrinsic*\n\nReturns the Euclidean length (magnitude) of a vector.\n\n$$\\text{length}(v) = \\sqrt{\\text{dot}(v, v)}$$\n\n**Parameters:**\n* `v`: Input vector.",
        overloads: &[
            BuiltinOverload { label: "float length(float v)", params: &["float v"] },
            BuiltinOverload { label: "float length(float2 v)", params: &["float2 v"] },
            BuiltinOverload { label: "float length(float3 v)", params: &["float3 v"] },
            BuiltinOverload { label: "float length(float4 v)", params: &["float4 v"] },
        ],
    },
    BuiltinFunction {
        name: "distance",
        description: "### `distance`\n*Microsoft HLSL Intrinsic*\n\nReturns the Euclidean distance between two points.\n\n$$\\text{distance}(x, y) = \\text{length}(x - y)$$\n\n**Parameters:**\n* `x`: First point.\n* `y`: Second point.",
        overloads: &[
            BuiltinOverload { label: "float distance(float x, float y)", params: &["float x", "float y"] },
            BuiltinOverload { label: "float distance(float2 x, float2 y)", params: &["float2 x", "float2 y"] },
            BuiltinOverload { label: "float distance(float3 x, float3 y)", params: &["float3 x", "float3 y"] },
            BuiltinOverload { label: "float distance(float4 x, float4 y)", params: &["float4 x", "float4 y"] },
        ],
    },
    BuiltinFunction {
        name: "reflect",
        description: "### `reflect`\n*Microsoft HLSL Intrinsic*\n\nReturns a reflection vector using an incident ray `i` and surface normal `n`.\n\n$$v = i - 2 \\cdot \\text{dot}(n, i) \\cdot n$$\n\n**Parameters:**\n* `i`: Incident ray vector.\n* `n`: Surface normal vector.",
        overloads: &[
            BuiltinOverload { label: "float3 reflect(float3 i, float3 n)", params: &["float3 i", "float3 n"] },
            BuiltinOverload { label: "float4 reflect(float4 i, float4 n)", params: &["float4 i", "float4 n"] },
        ],
    },
    BuiltinFunction {
        name: "refract",
        description: "### `refract`\n*Microsoft HLSL Intrinsic*\n\nCalculates a refraction vector using Snell's law.\n\n**Parameters:**\n* `i`: Incident vector.\n* `n`: Normal vector.\n* `eta`: Ratio of indices of refraction.",
        overloads: &[
            BuiltinOverload { label: "float3 refract(float3 i, float3 n, float eta)", params: &["float3 i", "float3 n", "float eta"] },
        ],
    },
    BuiltinFunction {
        name: "mul",
        description: "### `mul`\n*Microsoft HLSL Intrinsic*\n\nMultiplies `x` and `y` using matrix multiplication rules.\nSupports vector-matrix, matrix-vector, and matrix-matrix products.\n\n**Parameters:**\n* `x`: Left operand (vector or matrix).\n* `y`: Right operand (vector or matrix).",
        overloads: &[
            BuiltinOverload { label: "float4 mul(float4 v, float4x4 m)", params: &["float4 v", "float4x4 m"] },
            BuiltinOverload { label: "float4 mul(float4x4 m, float4 v)", params: &["float4x4 m", "float4 v"] },
            BuiltinOverload { label: "float3 mul(float3 v, float3x3 m)", params: &["float3 v", "float3x3 m"] },
            BuiltinOverload { label: "float3 mul(float3x3 m, float3 v)", params: &["float3x3 m", "float3 v"] },
            BuiltinOverload { label: "float4x4 mul(float4x4 a, float4x4 b)", params: &["float4x4 a", "float4x4 b"] },
        ],
    },
    BuiltinFunction {
        name: "transpose",
        description: "### `transpose`\n*Microsoft HLSL Intrinsic*\n\nReturns the transpose of matrix `m`.",
        overloads: &[
            BuiltinOverload { label: "float4x4 transpose(float4x4 m)", params: &["float4x4 m"] },
            BuiltinOverload { label: "float3x3 transpose(float3x3 m)", params: &["float3x3 m"] },
        ],
    },
    BuiltinFunction {
        name: "determinant",
        description: "### `determinant`\n*Microsoft HLSL Intrinsic*\n\nComputes the determinant of a square matrix.",
        overloads: &[
            BuiltinOverload { label: "float determinant(float4x4 m)", params: &["float4x4 m"] },
            BuiltinOverload { label: "float determinant(float3x3 m)", params: &["float3x3 m"] },
        ],
    },

    // ------------------------------------------------------------------------
    // Trigonometric & Exponential
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "sin",
        description: "### `sin`\n*Microsoft HLSL Intrinsic*\n\nComputes the sine of `x` (in radians).",
        overloads: &[
            BuiltinOverload { label: "float sin(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 sin(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 sin(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 sin(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "cos",
        description: "### `cos`\n*Microsoft HLSL Intrinsic*\n\nComputes the cosine of `x` (in radians).",
        overloads: &[
            BuiltinOverload { label: "float cos(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 cos(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 cos(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 cos(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "tan",
        description: "### `tan`\n*Microsoft HLSL Intrinsic*\n\nComputes the tangent of `x` (in radians).",
        overloads: &[
            BuiltinOverload { label: "float tan(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 tan(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 tan(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 tan(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "atan2",
        description: "### `atan2`\n*Microsoft HLSL Intrinsic*\n\nComputes the 2-argument arctangent of `y / x`.",
        overloads: &[
            BuiltinOverload { label: "float atan2(float y, float x)", params: &["float y", "float x"] },
            BuiltinOverload { label: "float2 atan2(float2 y, float2 x)", params: &["float2 y", "float2 x"] },
            BuiltinOverload { label: "float3 atan2(float3 y, float3 x)", params: &["float3 y", "float3 x"] },
            BuiltinOverload { label: "float4 atan2(float4 y, float4 x)", params: &["float4 y", "float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "sincos",
        description: "### `sincos`\n*Microsoft HLSL Intrinsic*\n\nComputes both sine and cosine simultaneously.\n\n**Parameters:**\n* `x`: Angle in radians.\n* `s`: Output sine variable (`out`).\n* `c`: Output cosine variable (`out`).",
        overloads: &[
            BuiltinOverload { label: "void sincos(float x, out float s, out float c)", params: &["float x", "out float s", "out float c"] },
            BuiltinOverload { label: "void sincos(float2 x, out float2 s, out float2 c)", params: &["float2 x", "out float2 s", "out float2 c"] },
            BuiltinOverload { label: "void sincos(float3 x, out float3 s, out float3 c)", params: &["float3 x", "out float3 s", "out float3 c"] },
            BuiltinOverload { label: "void sincos(float4 x, out float4 s, out float4 c)", params: &["float4 x", "out float4 s", "out float4 c"] },
        ],
    },
    BuiltinFunction {
        name: "pow",
        description: "### `pow`\n*Microsoft HLSL Intrinsic*\n\nComputes $x^y$.\n\n**Parameters:**\n* `x`: Base.\n* `y`: Exponent.",
        overloads: &[
            BuiltinOverload { label: "float pow(float x, float y)", params: &["float x", "float y"] },
            BuiltinOverload { label: "float2 pow(float2 x, float2 y)", params: &["float2 x", "float2 y"] },
            BuiltinOverload { label: "float3 pow(float3 x, float3 y)", params: &["float3 x", "float3 y"] },
            BuiltinOverload { label: "float4 pow(float4 x, float4 y)", params: &["float4 x", "float4 y"] },
        ],
    },
    BuiltinFunction {
        name: "sqrt",
        description: "### `sqrt`\n*Microsoft HLSL Intrinsic*\n\nComputes square root of `x`.",
        overloads: &[
            BuiltinOverload { label: "float sqrt(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 sqrt(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 sqrt(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 sqrt(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "rsqrt",
        description: "### `rsqrt`\n*Microsoft HLSL Intrinsic*\n\nComputes fast reciprocal square root $1 / \\sqrt{x}$.",
        overloads: &[
            BuiltinOverload { label: "float rsqrt(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 rsqrt(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 rsqrt(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 rsqrt(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "exp",
        description: "### `exp`\n*Microsoft HLSL Intrinsic*\n\nComputes base-$e$ exponential $e^x$.",
        overloads: &[
            BuiltinOverload { label: "float exp(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 exp(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 exp(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 exp(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "exp2",
        description: "### `exp2`\n*Microsoft HLSL Intrinsic*\n\nComputes base-2 exponential $2^x$.",
        overloads: &[
            BuiltinOverload { label: "float exp2(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 exp2(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 exp2(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 exp2(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "log",
        description: "### `log`\n*Microsoft HLSL Intrinsic*\n\nComputes natural logarithm $\\ln(x)$.",
        overloads: &[
            BuiltinOverload { label: "float log(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 log(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 log(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 log(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "log2",
        description: "### `log2`\n*Microsoft HLSL Intrinsic*\n\nComputes base-2 logarithm $\\log_2(x)$.",
        overloads: &[
            BuiltinOverload { label: "float log2(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 log2(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 log2(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 log2(float4 x)", params: &["float4 x"] },
        ],
    },

    // ------------------------------------------------------------------------
    // Screen-space Derivatives
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "ddx",
        description: "### `ddx`\n*Microsoft HLSL Intrinsic*\n\nComputes the partial derivative of `x` with respect to screen-space x-coordinate.",
        overloads: &[
            BuiltinOverload { label: "float ddx(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 ddx(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 ddx(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 ddx(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "ddy",
        description: "### `ddy`\n*Microsoft HLSL Intrinsic*\n\nComputes the partial derivative of `x` with respect to screen-space y-coordinate.",
        overloads: &[
            BuiltinOverload { label: "float ddy(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 ddy(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 ddy(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 ddy(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "fwidth",
        description: "### `fwidth`\n*Microsoft HLSL Intrinsic*\n\nReturns the sum of the absolute values of derivatives: `abs(ddx(x)) + abs(ddy(x))`.",
        overloads: &[
            BuiltinOverload { label: "float fwidth(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 fwidth(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 fwidth(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 fwidth(float4 x)", params: &["float4 x"] },
        ],
    },

    // ------------------------------------------------------------------------
    // Flow / Discard / Logic
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "clip",
        description: "### `clip`\n*Microsoft HLSL Intrinsic*\n\nDiscards the current pixel if the specified value is less than zero.\nEquivalent to `if (any(x < 0)) discard;`\n\n**Parameters:**\n* `x`: Cutoff scalar or vector.",
        overloads: &[
            BuiltinOverload { label: "void clip(float x)", params: &["float x"] },
            BuiltinOverload { label: "void clip(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "void clip(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "void clip(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "all",
        description: "### `all`\n*Microsoft HLSL Intrinsic*\n\nReturns `true` if all components of `x` are non-zero.",
        overloads: &[
            BuiltinOverload { label: "bool all(bool x)", params: &["bool x"] },
            BuiltinOverload { label: "bool all(bool2 x)", params: &["bool2 x"] },
            BuiltinOverload { label: "bool all(bool3 x)", params: &["bool3 x"] },
            BuiltinOverload { label: "bool all(bool4 x)", params: &["bool4 x"] },
        ],
    },
    BuiltinFunction {
        name: "any",
        description: "### `any`\n*Microsoft HLSL Intrinsic*\n\nReturns `true` if any component of `x` is non-zero.",
        overloads: &[
            BuiltinOverload { label: "bool any(bool x)", params: &["bool x"] },
            BuiltinOverload { label: "bool any(bool2 x)", params: &["bool2 x"] },
            BuiltinOverload { label: "bool any(bool3 x)", params: &["bool3 x"] },
            BuiltinOverload { label: "bool any(bool4 x)", params: &["bool4 x"] },
        ],
    },

    // ------------------------------------------------------------------------
    // Texture Sampling (Cg / Legacy & Modern HLSL)
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "tex2D",
        description: "### `tex2D`\n*HLSL / Cg Texture Sampling*\n\nSamples a 2D texture using UV coordinates.\n\n**Parameters:**\n* `s`: Sampler2D object.\n* `uv`: 2D texture coordinates.",
        overloads: &[
            BuiltinOverload { label: "float4 tex2D(sampler2D s, float2 uv)", params: &["sampler2D s", "float2 uv"] },
            BuiltinOverload { label: "float4 tex2D(sampler2D s, float2 uv, float2 dx, float2 dy)", params: &["sampler2D s", "float2 uv", "float2 dx", "float2 dy"] },
        ],
    },
    BuiltinFunction {
        name: "tex2Dlod",
        description: "### `tex2Dlod`\n*HLSL Texture Sampling*\n\nSamples a 2D texture with a specific mipmap LOD level.\n\n**Parameters:**\n* `s`: Sampler2D object.\n* `uv_lod`: `float4(uv.x, uv.y, 0, lod)`.",
        overloads: &[
            BuiltinOverload { label: "float4 tex2Dlod(sampler2D s, float4 uv_lod)", params: &["sampler2D s", "float4 uv_lod"] },
        ],
    },
    BuiltinFunction {
        name: "texCUBE",
        description: "### `texCUBE`\n*HLSL Texture Sampling*\n\nSamples a cubemap texture using a 3D direction vector.",
        overloads: &[
            BuiltinOverload { label: "float4 texCUBE(samplerCUBE s, float3 dir)", params: &["samplerCUBE s", "float3 dir"] },
        ],
    },

    // ------------------------------------------------------------------------
    // Unity Engine Built-in Helper Functions & Macros
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "TRANSFORM_TEX",
        description: "### `TRANSFORM_TEX(uv, name)`\n*Unity Built-in Macro*\n\nTransforms UV coordinates using the tiling and offset parameters (`name_ST`) of a texture.\n\n$$\\text{uv}' = \\text{uv} \\cdot \\text{name\\_ST.xy} + \\text{name\\_ST.zw}$$\n\n**Parameters:**\n* `uv`: Input 2D texture coordinates.\n* `name`: Texture identifier (requires matching uniform `float4 name_ST`).",
        overloads: &[
            BuiltinOverload { label: "float2 TRANSFORM_TEX(float2 uv, TextureName name)", params: &["float2 uv", "TextureName name"] },
        ],
    },
    BuiltinFunction {
        name: "TransformObjectToHClip",
        description: "### `TransformObjectToHClip`\n*Unity URP / HDRP*\n\nTransforms an object-space vertex position directly to Homogeneous Clip Space (CS).\n\n**Parameters:**\n* `positionOS`: Object-space position (`float3`).",
        overloads: &[
            BuiltinOverload { label: "float4 TransformObjectToHClip(float3 positionOS)", params: &["float3 positionOS"] },
        ],
    },
    BuiltinFunction {
        name: "TransformObjectToWorld",
        description: "### `TransformObjectToWorld`\n*Unity URP / HDRP*\n\nTransforms a position from Object Space (OS) to World Space (WS).\n\n**Parameters:**\n* `positionOS`: Object-space position (`float3`).",
        overloads: &[
            BuiltinOverload { label: "float3 TransformObjectToWorld(float3 positionOS)", params: &["float3 positionOS"] },
        ],
    },
    BuiltinFunction {
        name: "TransformWorldToObject",
        description: "### `TransformWorldToObject`\n*Unity URP / HDRP*\n\nTransforms a position from World Space (WS) to Object Space (OS).\n\n**Parameters:**\n* `positionWS`: World-space position (`float3`).",
        overloads: &[
            BuiltinOverload { label: "float3 TransformWorldToObject(float3 positionWS)", params: &["float3 positionWS"] },
        ],
    },
    BuiltinFunction {
        name: "TransformWorldToHClip",
        description: "### `TransformWorldToHClip`\n*Unity URP / HDRP*\n\nTransforms a position from World Space (WS) to Homogeneous Clip Space (CS).",
        overloads: &[
            BuiltinOverload { label: "float4 TransformWorldToHClip(float3 positionWS)", params: &["float3 positionWS"] },
        ],
    },
    BuiltinFunction {
        name: "TransformObjectToWorldNormal",
        description: "### `TransformObjectToWorldNormal`\n*Unity URP / HDRP*\n\nTransforms a normal vector from Object Space to normalized World Space.",
        overloads: &[
            BuiltinOverload { label: "float3 TransformObjectToWorldNormal(float3 normalOS)", params: &["float3 normalOS"] },
        ],
    },
    BuiltinFunction {
        name: "SAMPLE_TEXTURE2D",
        description: "### `SAMPLE_TEXTURE2D(textureName, samplerName, coord2)`\n*Unity Modern Macro*\n\nSamples a Texture2D using the designated SamplerState.",
        overloads: &[
            BuiltinOverload { label: "float4 SAMPLE_TEXTURE2D(Texture2D tex, SamplerState smp, float2 uv)", params: &["Texture2D tex", "SamplerState smp", "float2 uv"] },
        ],
    },
    BuiltinFunction {
        name: "UnpackNormal",
        description: "### `UnpackNormal`\n*Unity Built-in*\n\nUnpacks a normal map vector sampled from a standard or DXT5nm texture into a tangent-space normal vector.",
        overloads: &[
            BuiltinOverload { label: "float3 UnpackNormal(float4 packedNormal)", params: &["float4 packedNormal"] },
        ],
    },
    BuiltinFunction {
        name: "UnityObjectToClipPos",
        description: "### `UnityObjectToClipPos`\n*Unity Built-in RP (Legacy)*\n\nTransforms a position from Object Space directly to Homogeneous Clip Space.\n\n$$\\text{UnityObjectToClipPos}(pos) = \\text{mul}(\\text{UNITY\\_MATRIX\\_MVP}, \\text{float4}(pos, 1.0))$$",
        overloads: &[
            BuiltinOverload { label: "float4 UnityObjectToClipPos(float3 pos)", params: &["float3 pos"] },
            BuiltinOverload { label: "float4 UnityObjectToClipPos(float4 pos)", params: &["float4 pos"] },
        ],
    },
    BuiltinFunction {
        name: "UnityPixelSnap",
        description: "### `UnityPixelSnap`\n*Unity 2D Sprite Function*\n\nSnaps vertex position to screen pixel grid for pixel-perfect 2D rendering.\n\n**Parameters:**\n* `pos`: Clip-space position (`float4`).",
        overloads: &[
            BuiltinOverload { label: "float4 UnityPixelSnap(float4 pos)", params: &["float4 pos"] },
        ],
    },
    BuiltinFunction {
        name: "UnityGet2DClipping",
        description: "### `UnityGet2DClipping`\n*Unity 2D UI Function*\n\nCalculates rectangular clip mask factor for 2D UI Canvas elements.\n\n**Parameters:**\n* `position`: Screen/World-space position (`float2`).\n* `clipRect`: Rectangular clip boundaries min/max (`float4`).",
        overloads: &[
            BuiltinOverload { label: "float UnityGet2DClipping(float2 position, float4 clipRect)", params: &["float2 position", "float4 clipRect"] },
        ],
    },
    BuiltinFunction {
        name: "UnityWorldToClipPos",
        description: "### `UnityWorldToClipPos`\n*Unity Built-in RP*\n\nTransforms a position from World Space to Homogeneous Clip Space.\n\n**Parameters:**\n* `pos`: World-space position (`float3`).",
        overloads: &[
            BuiltinOverload { label: "float4 UnityWorldToClipPos(float3 pos)", params: &["float3 pos"] },
        ],
    },
    BuiltinFunction {
        name: "UnityObjectToViewPos",
        description: "### `UnityObjectToViewPos`\n*Unity Built-in RP*\n\nTransforms a position from Object Space into View/Eye Space.\n\n**Parameters:**\n* `pos`: Object-space position (`float3`).",
        overloads: &[
            BuiltinOverload { label: "float3 UnityObjectToViewPos(float3 pos)", params: &["float3 pos"] },
        ],
    },

    // ------------------------------------------------------------------------
    // Extended MSDN HLSL Intrinsics (Math, Float, Bitwise)
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "rcp",
        description: "### `rcp`\n*Microsoft HLSL Intrinsic*\n\nCalculates a fast per-component reciprocal $1 / x$.\n\n**Parameters:**\n* `x`: Floating point input.",
        overloads: &[
            BuiltinOverload { label: "float rcp(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 rcp(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 rcp(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 rcp(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "fma",
        description: "### `fma`\n*Microsoft HLSL Intrinsic*\n\nPerforms a single-precision fused multiply-accumulate: $a \\cdot b + c$.\n\n**Parameters:**\n* `a`: Multiplicand.\n* `b`: Multiplier.\n* `c`: Addend.",
        overloads: &[
            BuiltinOverload { label: "float fma(float a, float b, float c)", params: &["float a", "float b", "float c"] },
            BuiltinOverload { label: "float2 fma(float2 a, float2 b, float2 c)", params: &["float2 a", "float2 b", "float2 c"] },
            BuiltinOverload { label: "float3 fma(float3 a, float3 b, float3 c)", params: &["float3 a", "float3 b", "float3 c"] },
            BuiltinOverload { label: "float4 fma(float4 a, float4 b, float4 c)", params: &["float4 a", "float4 b", "float4 c"] },
        ],
    },
    BuiltinFunction {
        name: "mad",
        description: "### `mad`\n*Microsoft HLSL Intrinsic*\n\nPerforms multiply-add: $m \\cdot a + b$.\n\n**Parameters:**\n* `m`: Multiplicand.\n* `a`: Multiplier.\n* `b`: Addend.",
        overloads: &[
            BuiltinOverload { label: "float mad(float m, float a, float b)", params: &["float m", "float a", "float b"] },
            BuiltinOverload { label: "float2 mad(float2 m, float2 a, float2 b)", params: &["float2 m", "float2 a", "float2 b"] },
            BuiltinOverload { label: "float3 mad(float3 m, float3 a, float3 b)", params: &["float3 m", "float3 a", "float3 b"] },
            BuiltinOverload { label: "float4 mad(float4 m, float4 a, float4 b)", params: &["float4 m", "float4 a", "float4 b"] },
        ],
    },
    BuiltinFunction {
        name: "modf",
        description: "### `modf`\n*Microsoft HLSL Intrinsic*\n\nSplits value `x` into fractional and integer parts.\n\n**Parameters:**\n* `x`: Input value.\n* `ip`: Output variable receiving the integer part.",
        overloads: &[
            BuiltinOverload { label: "float modf(float x, out float ip)", params: &["float x", "out float ip"] },
            BuiltinOverload { label: "float2 modf(float2 x, out float2 ip)", params: &["float2 x", "out float2 ip"] },
            BuiltinOverload { label: "float3 modf(float3 x, out float3 ip)", params: &["float3 x", "out float3 ip"] },
            BuiltinOverload { label: "float4 modf(float4 x, out float4 ip)", params: &["float4 x", "out float4 ip"] },
        ],
    },
    BuiltinFunction {
        name: "degrees",
        description: "### `degrees`\n*Microsoft HLSL Intrinsic*\n\nConverts radians to degrees.\n\n$$\\text{degrees}(x) = x \\cdot \\frac{180}{\\pi}$$\n\n**Parameters:**\n* `x`: Angle in radians.",
        overloads: &[
            BuiltinOverload { label: "float degrees(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 degrees(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 degrees(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 degrees(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "radians",
        description: "### `radians`\n*Microsoft HLSL Intrinsic*\n\nConverts degrees to radians.\n\n$$\\text{radians}(x) = x \\cdot \\frac{\\pi}{180}$$\n\n**Parameters:**\n* `x`: Angle in degrees.",
        overloads: &[
            BuiltinOverload { label: "float radians(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 radians(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 radians(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 radians(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "asin",
        description: "### `asin`\n*Microsoft HLSL Intrinsic*\n\nComputes the arcsine of `x`.",
        overloads: &[
            BuiltinOverload { label: "float asin(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 asin(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 asin(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 asin(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "acos",
        description: "### `acos`\n*Microsoft HLSL Intrinsic*\n\nComputes the arccosine of `x`.",
        overloads: &[
            BuiltinOverload { label: "float acos(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 acos(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 acos(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 acos(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "atan",
        description: "### `atan`\n*Microsoft HLSL Intrinsic*\n\nComputes the arctangent of `x`.",
        overloads: &[
            BuiltinOverload { label: "float atan(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 atan(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 atan(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 atan(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "sinh",
        description: "### `sinh`\n*Microsoft HLSL Intrinsic*\n\nComputes the hyperbolic sine of `x`.",
        overloads: &[
            BuiltinOverload { label: "float sinh(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 sinh(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 sinh(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 sinh(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "cosh",
        description: "### `cosh`\n*Microsoft HLSL Intrinsic*\n\nComputes the hyperbolic cosine of `x`.",
        overloads: &[
            BuiltinOverload { label: "float cosh(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 cosh(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 cosh(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 cosh(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "tanh",
        description: "### `tanh`\n*Microsoft HLSL Intrinsic*\n\nComputes the hyperbolic tangent of `x`.",
        overloads: &[
            BuiltinOverload { label: "float tanh(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 tanh(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 tanh(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 tanh(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "trunc",
        description: "### `trunc`\n*Microsoft HLSL Intrinsic*\n\nTruncates the floating-point value to integer, dropping fractional digits.",
        overloads: &[
            BuiltinOverload { label: "float trunc(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 trunc(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 trunc(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 trunc(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "isnan",
        description: "### `isnan`\n*Microsoft HLSL Intrinsic*\n\nReturns `true` if `x` is NaN (Not a Number).",
        overloads: &[
            BuiltinOverload { label: "bool isnan(float x)", params: &["float x"] },
            BuiltinOverload { label: "bool2 isnan(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "bool3 isnan(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "bool4 isnan(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "isinf",
        description: "### `isinf`\n*Microsoft HLSL Intrinsic*\n\nReturns `true` if `x` is infinite (+/- INF).",
        overloads: &[
            BuiltinOverload { label: "bool isinf(float x)", params: &["float x"] },
            BuiltinOverload { label: "bool2 isinf(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "bool3 isinf(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "bool4 isinf(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "isfinite",
        description: "### `isfinite`\n*Microsoft HLSL Intrinsic*\n\nReturns `true` if `x` is a finite number (not NaN or INF).",
        overloads: &[
            BuiltinOverload { label: "bool isfinite(float x)", params: &["float x"] },
            BuiltinOverload { label: "bool2 isfinite(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "bool3 isfinite(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "bool4 isfinite(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "ldexp",
        description: "### `ldexp`\n*Microsoft HLSL Intrinsic*\n\nComputes $x \\cdot 2^{\\text{exp}}$.",
        overloads: &[
            BuiltinOverload { label: "float ldexp(float x, float exp)", params: &["float x", "float exp"] },
            BuiltinOverload { label: "float2 ldexp(float2 x, float2 exp)", params: &["float2 x", "float2 exp"] },
            BuiltinOverload { label: "float3 ldexp(float3 x, float3 exp)", params: &["float3 x", "float3 exp"] },
            BuiltinOverload { label: "float4 ldexp(float4 x, float4 exp)", params: &["float4 x", "float4 exp"] },
        ],
    },
    BuiltinFunction {
        name: "asfloat",
        description: "### `asfloat`\n*Microsoft HLSL Intrinsic*\n\nReinterprets the bit pattern of an integer/uint as a float.",
        overloads: &[
            BuiltinOverload { label: "float asfloat(int x)", params: &["int x"] },
            BuiltinOverload { label: "float asfloat(uint x)", params: &["uint x"] },
            BuiltinOverload { label: "float4 asfloat(uint4 x)", params: &["uint4 x"] },
        ],
    },
    BuiltinFunction {
        name: "asint",
        description: "### `asint`\n*Microsoft HLSL Intrinsic*\n\nReinterprets the bit pattern of a float/uint as a signed integer.",
        overloads: &[
            BuiltinOverload { label: "int asint(float x)", params: &["float x"] },
            BuiltinOverload { label: "int asint(uint x)", params: &["uint x"] },
            BuiltinOverload { label: "int4 asint(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "asuint",
        description: "### `asuint`\n*Microsoft HLSL Intrinsic*\n\nReinterprets the bit pattern of a float/int as an unsigned integer.",
        overloads: &[
            BuiltinOverload { label: "uint asuint(float x)", params: &["float x"] },
            BuiltinOverload { label: "uint asuint(int x)", params: &["int x"] },
            BuiltinOverload { label: "uint4 asuint(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "countbits",
        description: "### `countbits`\n*Microsoft HLSL Intrinsic*\n\nCounts the number of set bits (1s) in an integer.",
        overloads: &[
            BuiltinOverload { label: "uint countbits(uint value)", params: &["uint value"] },
            BuiltinOverload { label: "uint2 countbits(uint2 value)", params: &["uint2 value"] },
            BuiltinOverload { label: "uint3 countbits(uint3 value)", params: &["uint3 value"] },
            BuiltinOverload { label: "uint4 countbits(uint4 value)", params: &["uint4 value"] },
        ],
    },
    BuiltinFunction {
        name: "firstbitlow",
        description: "### `firstbitlow`\n*Microsoft HLSL Intrinsic*\n\nReturns the index of the first set bit (1) starting from bit 0 (least significant bit).",
        overloads: &[
            BuiltinOverload { label: "uint firstbitlow(uint value)", params: &["uint value"] },
            BuiltinOverload { label: "uint4 firstbitlow(uint4 value)", params: &["uint4 value"] },
        ],
    },
    BuiltinFunction {
        name: "firstbithigh",
        description: "### `firstbithigh`\n*Microsoft HLSL Intrinsic*\n\nReturns the index of the first set bit (1) starting from bit 31 (most significant bit).",
        overloads: &[
            BuiltinOverload { label: "uint firstbithigh(uint value)", params: &["uint value"] },
            BuiltinOverload { label: "uint4 firstbithigh(uint4 value)", params: &["uint4 value"] },
        ],
    },
    BuiltinFunction {
        name: "reversebits",
        description: "### `reversebits`\n*Microsoft HLSL Intrinsic*\n\nReverses the bit order of the specified 32-bit integer.",
        overloads: &[
            BuiltinOverload { label: "uint reversebits(uint value)", params: &["uint value"] },
            BuiltinOverload { label: "uint4 reversebits(uint4 value)", params: &["uint4 value"] },
        ],
    },
    BuiltinFunction {
        name: "f32tof16",
        description: "### `f32tof16`\n*Microsoft HLSL Intrinsic*\n\nConverts a 32-bit float to a 16-bit half precision float encoded as a uint.",
        overloads: &[
            BuiltinOverload { label: "uint f32tof16(float value)", params: &["float value"] },
            BuiltinOverload { label: "uint4 f32tof16(float4 value)", params: &["float4 value"] },
        ],
    },
    BuiltinFunction {
        name: "f16tof32",
        description: "### `f16tof32`\n*Microsoft HLSL Intrinsic*\n\nConverts a 16-bit half precision float encoded in uint to a 32-bit float.",
        overloads: &[
            BuiltinOverload { label: "float f16tof32(uint value)", params: &["uint value"] },
            BuiltinOverload { label: "float4 f16tof32(uint4 value)", params: &["uint4 value"] },
        ],
    },

    // ------------------------------------------------------------------------
    // Wave Intrinsics (Shader Model 6.0+)
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "WaveIsFirstLane",
        description: "### `WaveIsFirstLane`\n*HLSL Wave Intrinsic (SM 6.0+)*\n\nReturns `true` only for the active lane with the smallest index in the current wave.",
        overloads: &[
            BuiltinOverload { label: "bool WaveIsFirstLane()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "WaveGetLaneCount",
        description: "### `WaveGetLaneCount`\n*HLSL Wave Intrinsic (SM 6.0+)*\n\nReturns the number of lanes in the current wave.",
        overloads: &[
            BuiltinOverload { label: "uint WaveGetLaneCount()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "WaveGetLaneIndex",
        description: "### `WaveGetLaneIndex`\n*HLSL Wave Intrinsic (SM 6.0+)*\n\nReturns the lane index of the executing thread within the current wave.",
        overloads: &[
            BuiltinOverload { label: "uint WaveGetLaneIndex()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "WaveActiveAnyTrue",
        description: "### `WaveActiveAnyTrue`\n*HLSL Wave Intrinsic (SM 6.0+)*\n\nReturns `true` if `expr` evaluates to true for any active lane in the current wave.",
        overloads: &[
            BuiltinOverload { label: "bool WaveActiveAnyTrue(bool expr)", params: &["bool expr"] },
        ],
    },
    BuiltinFunction {
        name: "WaveActiveAllTrue",
        description: "### `WaveActiveAllTrue`\n*HLSL Wave Intrinsic (SM 6.0+)*\n\nReturns `true` if `expr` evaluates to true for all active lanes in the current wave.",
        overloads: &[
            BuiltinOverload { label: "bool WaveActiveAllTrue(bool expr)", params: &["bool expr"] },
        ],
    },
    BuiltinFunction {
        name: "WaveActiveSum",
        description: "### `WaveActiveSum`\n*HLSL Wave Intrinsic (SM 6.0+)*\n\nSums up value across all active lanes in the wave.",
        overloads: &[
            BuiltinOverload { label: "float WaveActiveSum(float expr)", params: &["float expr"] },
            BuiltinOverload { label: "float4 WaveActiveSum(float4 expr)", params: &["float4 expr"] },
            BuiltinOverload { label: "uint WaveActiveSum(uint expr)", params: &["uint expr"] },
        ],
    },

    // ------------------------------------------------------------------------
    // Unity Engine Extended Helpers (Lighting, Fog, Sampling)
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "GetMainLight",
        description: "### `GetMainLight`\n*Unity URP / HDRP*\n\nRetrieves lighting information (direction, color, shadow attenuation) for the primary directional light.",
        overloads: &[
            BuiltinOverload { label: "Light GetMainLight()", params: &[] },
            BuiltinOverload { label: "Light GetMainLight(float4 shadowCoord)", params: &["float4 shadowCoord"] },
        ],
    },
    BuiltinFunction {
        name: "GetAdditionalLight",
        description: "### `GetAdditionalLight`\n*Unity URP*\n\nRetrieves lighting data for an additional light (point, spot).",
        overloads: &[
            BuiltinOverload { label: "Light GetAdditionalLight(uint i, float3 positionWS)", params: &["uint i", "float3 positionWS"] },
        ],
    },
    BuiltinFunction {
        name: "GetAdditionalLightsCount",
        description: "### `GetAdditionalLightsCount`\n*Unity URP*\n\nReturns the number of additional per-pixel lights affecting the object.",
        overloads: &[
            BuiltinOverload { label: "uint GetAdditionalLightsCount()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "ComputeFogFactor",
        description: "### `ComputeFogFactor`\n*Unity URP*\n\nCalculates the fog blend factor based on clip space depth.",
        overloads: &[
            BuiltinOverload { label: "float ComputeFogFactor(float z)", params: &["float z"] },
        ],
    },
    BuiltinFunction {
        name: "MixFog",
        description: "### `MixFog`\n*Unity URP*\n\nMixes fog color into the final fragment color using calculated fog factor.",
        overloads: &[
            BuiltinOverload { label: "float3 MixFog(float3 fragColor, float fogFactor)", params: &["float3 fragColor", "float fogFactor"] },
        ],
    },
    BuiltinFunction {
        name: "TransformWorldToViewNormal",
        description: "### `TransformWorldToViewNormal`\n*Unity URP*\n\nTransforms a normal vector from World Space to View Space.",
        overloads: &[
            BuiltinOverload { label: "float3 TransformWorldToViewNormal(float3 normalWS, bool doNormalize = true)", params: &["float3 normalWS", "bool doNormalize"] },
        ],
    },
    BuiltinFunction {
        name: "UnpackNormalScale",
        description: "### `UnpackNormalScale`\n*Unity Built-in*\n\nUnpacks a tangent space normal and scales its intensity with `scale`.",
        overloads: &[
            BuiltinOverload { label: "float3 UnpackNormalScale(float4 packedNormal, float bumpScale)", params: &["float4 packedNormal", "float bumpScale"] },
        ],
    },
    BuiltinFunction {
        name: "UnityObjectToWorldNormal",
        description: "### `UnityObjectToWorldNormal`\n*Unity Built-in RP*\n\nTransforms an object-space normal to normalized world space.",
        overloads: &[
            BuiltinOverload { label: "float3 UnityObjectToWorldNormal(float3 norm)", params: &["float3 norm"] },
        ],
    },
    BuiltinFunction {
        name: "UnityObjectToWorldDir",
        description: "### `UnityObjectToWorldDir`\n*Unity Built-in RP*\n\nTransforms an object-space direction vector to normalized world space.",
        overloads: &[
            BuiltinOverload { label: "float3 UnityObjectToWorldDir(float3 dir)", params: &["float3 dir"] },
        ],
    },

    // ------------------------------------------------------------------------
    // Unreal Engine Extended Helpers (.ush / Custom Material Expression)
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "RotateAboutAxis",
        description: "### `RotateAboutAxis`\n*Unreal Engine Shader Function*\n\nRotates vector `Position` around `NormalizedRotationAxisAndAngle.xyz` by angle `w` (in radians).",
        overloads: &[
            BuiltinOverload { label: "float3 RotateAboutAxis(float4 NormalizedRotationAxisAndAngle, float3 PositionOnAxis, float3 Position)", params: &["float4 NormalizedRotationAxisAndAngle", "float3 PositionOnAxis", "float3 Position"] },
        ],
    },
    BuiltinFunction {
        name: "GetWorldPosition",
        description: "### `GetWorldPosition`\n*Unreal Engine Material Expression*\n\nReturns the absolute world space position of the current vertex/pixel.",
        overloads: &[
            BuiltinOverload { label: "float3 GetWorldPosition(FMaterialVertexParameters Parameters)", params: &["FMaterialVertexParameters Parameters"] },
            BuiltinOverload { label: "float3 GetWorldPosition(FMaterialPixelParameters Parameters)", params: &["FMaterialPixelParameters Parameters"] },
        ],
    },
    BuiltinFunction {
        name: "LinearToSrgb",
        description: "### `LinearToSrgb`\n*Unreal Engine / Color Transform*\n\nConverts color from Linear space to gamma sRGB space.",
        overloads: &[
            BuiltinOverload { label: "float3 LinearToSrgb(float3 Color)", params: &["float3 Color"] },
        ],
    },
    BuiltinFunction {
        name: "SrgbToLinear",
        description: "### `SrgbToLinear`\n*Unreal Engine / Color Transform*\n\nConverts color from gamma sRGB space to Linear space.",
        overloads: &[
            BuiltinOverload { label: "float3 SrgbToLinear(float3 Color)", params: &["float3 Color"] },
        ],
    },
    BuiltinFunction {
        name: "Luminance",
        description: "### `Luminance`\n*Unreal Engine / Common Utility*\n\nCalculates perceptual photometric luminance (grayscale intensity) of an RGB color.",
        overloads: &[
            BuiltinOverload { label: "float Luminance(float3 LinearColor)", params: &["float3 LinearColor"] },
        ],
    },
    // ------------------------------------------------------------------------
    // Additional Microsoft HLSL Intrinsics (DirectX 9 - 12 / SM 6.x)
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "faceforward",
        description: "### `faceforward`\n*Microsoft HLSL Intrinsic*\n\nFlips a surface normal `n` to face away from the incident vector `i` relative to geometric normal `ng`.\n\n$$\\text{faceforward}(n, i, ng) = -n \\cdot \\text{sign}(\\text{dot}(i, ng))$$\n\n**Parameters:**\n* `n`: Vector to orient.\n* `i`: Incident vector.\n* `ng`: Reference geometric normal vector.",
        overloads: &[
            BuiltinOverload { label: "float faceforward(float n, float i, float ng)", params: &["float n", "float i", "float ng"] },
            BuiltinOverload { label: "float2 faceforward(float2 n, float2 i, float2 ng)", params: &["float2 n", "float2 i", "float2 ng"] },
            BuiltinOverload { label: "float3 faceforward(float3 n, float3 i, float3 ng)", params: &["float3 n", "float3 i", "float3 ng"] },
            BuiltinOverload { label: "float4 faceforward(float4 n, float4 i, float4 ng)", params: &["float4 n", "float4 i", "float4 ng"] },
        ],
    },
    BuiltinFunction {
        name: "lit",
        description: "### `lit`\n*Microsoft HLSL Intrinsic*\n\nComputes lighting coefficients for ambient, diffuse, and specular illumination.\n\nReturns a `float4(ambient, diffuse, specular, 1.0)`.\n\n**Parameters:**\n* `n_dot_l`: Dot product between surface normal and light direction.\n* `n_dot_h`: Dot product between surface normal and half-angle vector.\n* `m`: Specular exponent.",
        overloads: &[
            BuiltinOverload { label: "float4 lit(float n_dot_l, float n_dot_h, float m)", params: &["float n_dot_l", "float n_dot_h", "float m"] },
        ],
    },
    BuiltinFunction {
        name: "dst",
        description: "### `dst`\n*Microsoft HLSL Intrinsic*\n\nCalculates a distance vector `(1.0, d, d^2, 1/d)` between two vectors.\n\n**Parameters:**\n* `src0`: First source vector.\n* `src1`: Second source vector.",
        overloads: &[
            BuiltinOverload { label: "float4 dst(float4 src0, float4 src1)", params: &["float4 src0", "float4 src1"] },
        ],
    },
    BuiltinFunction {
        name: "msad4",
        description: "### `msad4`\n*Microsoft HLSL Intrinsic*\n\nCompares a 4-byte reference value and an 8-byte source value, accumulating the sum of absolute differences.\n\n**Parameters:**\n* `reference`: 32-bit integer containing 4 reference bytes.\n* `source`: `uint2` containing 8 source bytes.\n* `accum`: `uint4` accumulator vector.",
        overloads: &[
            BuiltinOverload { label: "uint4 msad4(uint reference, uint2 source, uint4 accum)", params: &["uint reference", "uint2 source", "uint4 accum"] },
        ],
    },
    BuiltinFunction {
        name: "ubfe",
        description: "### `ubfe`\n*Microsoft HLSL Intrinsic*\n\nExtracts a bitfield from an unsigned integer without sign extension.\n\n**Parameters:**\n* `width`: Bitfield width in bits.\n* `offset`: Bitfield start offset in bits.\n* `value`: Integer value to extract from.",
        overloads: &[
            BuiltinOverload { label: "uint ubfe(uint width, uint offset, uint value)", params: &["uint width", "uint offset", "uint value"] },
            BuiltinOverload { label: "uint2 ubfe(uint width, uint offset, uint2 value)", params: &["uint width", "uint offset", "uint2 value"] },
            BuiltinOverload { label: "uint3 ubfe(uint width, uint offset, uint3 value)", params: &["uint width", "uint offset", "uint3 value"] },
            BuiltinOverload { label: "uint4 ubfe(uint width, uint offset, uint4 value)", params: &["uint width", "uint offset", "uint4 value"] },
        ],
    },
    BuiltinFunction {
        name: "ibfe",
        description: "### `ibfe`\n*Microsoft HLSL Intrinsic*\n\nExtracts a bitfield from a signed integer with sign extension.\n\n**Parameters:**\n* `width`: Bitfield width in bits.\n* `offset`: Bitfield start offset in bits.\n* `value`: Integer value to extract from.",
        overloads: &[
            BuiltinOverload { label: "int ibfe(uint width, uint offset, int value)", params: &["uint width", "uint offset", "int value"] },
            BuiltinOverload { label: "int2 ibfe(uint width, uint offset, int2 value)", params: &["uint width", "uint offset", "int2 value"] },
            BuiltinOverload { label: "int3 ibfe(uint width, uint offset, int3 value)", params: &["uint width", "uint offset", "int3 value"] },
            BuiltinOverload { label: "int4 ibfe(uint width, uint offset, int4 value)", params: &["uint width", "uint offset", "int4 value"] },
        ],
    },
    BuiltinFunction {
        name: "ddx_fine",
        description: "### `ddx_fine`\n*Microsoft HLSL Intrinsic*\n\nComputes a high-precision per-pixel screen-space partial derivative in the X direction.",
        overloads: &[
            BuiltinOverload { label: "float ddx_fine(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 ddx_fine(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 ddx_fine(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 ddx_fine(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "ddy_fine",
        description: "### `ddy_fine`\n*Microsoft HLSL Intrinsic*\n\nComputes a high-precision per-pixel screen-space partial derivative in the Y direction.",
        overloads: &[
            BuiltinOverload { label: "float ddy_fine(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 ddy_fine(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 ddy_fine(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 ddy_fine(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "ddx_coarse",
        description: "### `ddx_coarse`\n*Microsoft HLSL Intrinsic*\n\nComputes a coarse (per-quad) screen-space partial derivative in the X direction.",
        overloads: &[
            BuiltinOverload { label: "float ddx_coarse(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 ddx_coarse(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 ddx_coarse(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 ddx_coarse(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "ddy_coarse",
        description: "### `ddy_coarse`\n*Microsoft HLSL Intrinsic*\n\nComputes a coarse (per-quad) screen-space partial derivative in the Y direction.",
        overloads: &[
            BuiltinOverload { label: "float ddy_coarse(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 ddy_coarse(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 ddy_coarse(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 ddy_coarse(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "fwidth_fine",
        description: "### `fwidth_fine`\n*Microsoft HLSL Intrinsic*\n\nReturns the fine absolute sum of screen-space derivatives: `abs(ddx_fine(x)) + abs(ddy_fine(x))`.",
        overloads: &[
            BuiltinOverload { label: "float fwidth_fine(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 fwidth_fine(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 fwidth_fine(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 fwidth_fine(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "fwidth_coarse",
        description: "### `fwidth_coarse`\n*Microsoft HLSL Intrinsic*\n\nReturns the coarse absolute sum of screen-space derivatives: `abs(ddx_coarse(x)) + abs(ddy_coarse(x))`.",
        overloads: &[
            BuiltinOverload { label: "float fwidth_coarse(float x)", params: &["float x"] },
            BuiltinOverload { label: "float2 fwidth_coarse(float2 x)", params: &["float2 x"] },
            BuiltinOverload { label: "float3 fwidth_coarse(float3 x)", params: &["float3 x"] },
            BuiltinOverload { label: "float4 fwidth_coarse(float4 x)", params: &["float4 x"] },
        ],
    },
    BuiltinFunction {
        name: "WaveActiveMin",
        description: "### `WaveActiveMin`\n*Microsoft HLSL Wave Intrinsic (SM 6.0+)*\n\nComputes the minimum value of `expr` across all active lanes in the wave.",
        overloads: &[
            BuiltinOverload { label: "float WaveActiveMin(float expr)", params: &["float expr"] },
            BuiltinOverload { label: "int WaveActiveMin(int expr)", params: &["int expr"] },
            BuiltinOverload { label: "uint WaveActiveMin(uint expr)", params: &["uint expr"] },
        ],
    },
    BuiltinFunction {
        name: "WaveActiveMax",
        description: "### `WaveActiveMax`\n*Microsoft HLSL Wave Intrinsic (SM 6.0+)*\n\nComputes the maximum value of `expr` across all active lanes in the wave.",
        overloads: &[
            BuiltinOverload { label: "float WaveActiveMax(float expr)", params: &["float expr"] },
            BuiltinOverload { label: "int WaveActiveMax(int expr)", params: &["int expr"] },
            BuiltinOverload { label: "uint WaveActiveMax(uint expr)", params: &["uint expr"] },
        ],
    },
    BuiltinFunction {
        name: "WaveActiveProduct",
        description: "### `WaveActiveProduct`\n*Microsoft HLSL Wave Intrinsic (SM 6.0+)*\n\nComputes the product of `expr` across all active lanes in the current wave.",
        overloads: &[
            BuiltinOverload { label: "float WaveActiveProduct(float expr)", params: &["float expr"] },
            BuiltinOverload { label: "uint WaveActiveProduct(uint expr)", params: &["uint expr"] },
        ],
    },
    BuiltinFunction {
        name: "WaveActiveBitAnd",
        description: "### `WaveActiveBitAnd`\n*Microsoft HLSL Wave Intrinsic (SM 6.0+)*\n\nComputes a bitwise AND reduction of `expr` across all active lanes in the wave.",
        overloads: &[
            BuiltinOverload { label: "uint WaveActiveBitAnd(uint expr)", params: &["uint expr"] },
            BuiltinOverload { label: "uint4 WaveActiveBitAnd(uint4 expr)", params: &["uint4 expr"] },
        ],
    },
    BuiltinFunction {
        name: "WaveActiveBitOr",
        description: "### `WaveActiveBitOr`\n*Microsoft HLSL Wave Intrinsic (SM 6.0+)*\n\nComputes a bitwise OR reduction of `expr` across all active lanes in the wave.",
        overloads: &[
            BuiltinOverload { label: "uint WaveActiveBitOr(uint expr)", params: &["uint expr"] },
            BuiltinOverload { label: "uint4 WaveActiveBitOr(uint4 expr)", params: &["uint4 expr"] },
        ],
    },
    BuiltinFunction {
        name: "WaveActiveBitXor",
        description: "### `WaveActiveBitXor`\n*Microsoft HLSL Wave Intrinsic (SM 6.0+)*\n\nComputes a bitwise XOR reduction of `expr` across all active lanes in the wave.",
        overloads: &[
            BuiltinOverload { label: "uint WaveActiveBitXor(uint expr)", params: &["uint expr"] },
            BuiltinOverload { label: "uint4 WaveActiveBitXor(uint4 expr)", params: &["uint4 expr"] },
        ],
    },
    BuiltinFunction {
        name: "WaveActiveCountBits",
        description: "### `WaveActiveCountBits`\n*Microsoft HLSL Wave Intrinsic (SM 6.0+)*\n\nCounts the total number of active lanes for which `bBit` evaluates to `true`.",
        overloads: &[
            BuiltinOverload { label: "uint WaveActiveCountBits(bool bBit)", params: &["bool bBit"] },
        ],
    },
    BuiltinFunction {
        name: "WaveActivePrefixSum",
        description: "### `WaveActivePrefixSum`\n*Microsoft HLSL Wave Intrinsic (SM 6.0+)*\n\nComputes the exclusive prefix sum of `value` across lanes with lower indices than the current lane.",
        overloads: &[
            BuiltinOverload { label: "float WaveActivePrefixSum(float value)", params: &["float value"] },
            BuiltinOverload { label: "uint WaveActivePrefixSum(uint value)", params: &["uint value"] },
        ],
    },
    BuiltinFunction {
        name: "WaveActivePrefixProduct",
        description: "### `WaveActivePrefixProduct`\n*Microsoft HLSL Wave Intrinsic (SM 6.0+)*\n\nComputes the exclusive prefix multiplication of `value` across lower-indexed lanes.",
        overloads: &[
            BuiltinOverload { label: "float WaveActivePrefixProduct(float value)", params: &["float value"] },
            BuiltinOverload { label: "uint WaveActivePrefixProduct(uint value)", params: &["uint value"] },
        ],
    },
    BuiltinFunction {
        name: "WaveReadLaneFirst",
        description: "### `WaveReadLaneFirst`\n*Microsoft HLSL Wave Intrinsic (SM 6.0+)*\n\nBroadcasts the value of `expr` evaluated on the first active lane in the wave to all lanes.",
        overloads: &[
            BuiltinOverload { label: "float WaveReadLaneFirst(float expr)", params: &["float expr"] },
            BuiltinOverload { label: "uint WaveReadLaneFirst(uint expr)", params: &["uint expr"] },
            BuiltinOverload { label: "float4 WaveReadLaneFirst(float4 expr)", params: &["float4 expr"] },
        ],
    },
    BuiltinFunction {
        name: "WaveReadLaneAt",
        description: "### `WaveReadLaneAt`\n*Microsoft HLSL Wave Intrinsic (SM 6.0+)*\n\nReads `expr` from a specific lane index `laneIndex` within the wave.",
        overloads: &[
            BuiltinOverload { label: "float WaveReadLaneAt(float expr, uint laneIndex)", params: &["float expr", "uint laneIndex"] },
            BuiltinOverload { label: "uint WaveReadLaneAt(uint expr, uint laneIndex)", params: &["uint expr", "uint laneIndex"] },
            BuiltinOverload { label: "float4 WaveReadLaneAt(float4 expr, uint laneIndex)", params: &["float4 expr", "uint laneIndex"] },
        ],
    },
    BuiltinFunction {
        name: "QuadReadAcrossX",
        description: "### `QuadReadAcrossX`\n*Microsoft HLSL Quad Intrinsic (SM 6.0+)*\n\nReturns the value of `localValue` from the neighboring lane horizontally across the quad.",
        overloads: &[
            BuiltinOverload { label: "float QuadReadAcrossX(float localValue)", params: &["float localValue"] },
            BuiltinOverload { label: "float4 QuadReadAcrossX(float4 localValue)", params: &["float4 localValue"] },
        ],
    },
    BuiltinFunction {
        name: "QuadReadAcrossY",
        description: "### `QuadReadAcrossY`\n*Microsoft HLSL Quad Intrinsic (SM 6.0+)*\n\nReturns the value of `localValue` from the neighboring lane vertically across the quad.",
        overloads: &[
            BuiltinOverload { label: "float QuadReadAcrossY(float localValue)", params: &["float localValue"] },
            BuiltinOverload { label: "float4 QuadReadAcrossY(float4 localValue)", params: &["float4 localValue"] },
        ],
    },
    BuiltinFunction {
        name: "QuadReadAcrossDiagonal",
        description: "### `QuadReadAcrossDiagonal`\n*Microsoft HLSL Quad Intrinsic (SM 6.0+)*\n\nReturns the value of `localValue` from the diagonally opposite lane in the quad.",
        overloads: &[
            BuiltinOverload { label: "float QuadReadAcrossDiagonal(float localValue)", params: &["float localValue"] },
            BuiltinOverload { label: "float4 QuadReadAcrossDiagonal(float4 localValue)", params: &["float4 localValue"] },
        ],
    },
    BuiltinFunction {
        name: "QuadReadLaneAt",
        description: "### `QuadReadLaneAt`\n*Microsoft HLSL Quad Intrinsic (SM 6.0+)*\n\nReturns the value of `sourceValue` from a specific lane index (0 to 3) within the quad.",
        overloads: &[
            BuiltinOverload { label: "float QuadReadLaneAt(float sourceValue, uint quadLaneID)", params: &["float sourceValue", "uint quadLaneID"] },
            BuiltinOverload { label: "float4 QuadReadLaneAt(float4 sourceValue, uint quadLaneID)", params: &["float4 sourceValue", "uint quadLaneID"] },
        ],
    },
    BuiltinFunction {
        name: "GroupMemoryBarrier",
        description: "### `GroupMemoryBarrier`\n*Microsoft HLSL Barrier Intrinsic*\n\nBlocks execution of all threads in a thread group until all group shared memory accesses are completed.",
        overloads: &[
            BuiltinOverload { label: "void GroupMemoryBarrier()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "GroupMemoryBarrierWithGroupSync",
        description: "### `GroupMemoryBarrierWithGroupSync`\n*Microsoft HLSL Barrier Intrinsic*\n\nExecutes a group memory barrier and synchronizes execution of all threads in the compute group.",
        overloads: &[
            BuiltinOverload { label: "void GroupMemoryBarrierWithGroupSync()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "DeviceMemoryBarrier",
        description: "### `DeviceMemoryBarrier`\n*Microsoft HLSL Barrier Intrinsic*\n\nBlocks execution until all device memory accesses (UAVs / textures) are completed.",
        overloads: &[
            BuiltinOverload { label: "void DeviceMemoryBarrier()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "DeviceMemoryBarrierWithGroupSync",
        description: "### `DeviceMemoryBarrierWithGroupSync`\n*Microsoft HLSL Barrier Intrinsic*\n\nExecutes a device memory barrier and synchronizes all threads in the compute group.",
        overloads: &[
            BuiltinOverload { label: "void DeviceMemoryBarrierWithGroupSync()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "AllMemoryBarrier",
        description: "### `AllMemoryBarrier`\n*Microsoft HLSL Barrier Intrinsic*\n\nBlocks execution until both group shared memory and device memory accesses are completed.",
        overloads: &[
            BuiltinOverload { label: "void AllMemoryBarrier()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "AllMemoryBarrierWithGroupSync",
        description: "### `AllMemoryBarrierWithGroupSync`\n*Microsoft HLSL Barrier Intrinsic*\n\nExecutes a full memory barrier and synchronizes all threads in the compute group.",
        overloads: &[
            BuiltinOverload { label: "void AllMemoryBarrierWithGroupSync()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedAdd",
        description: "### `InterlockedAdd`\n*Microsoft HLSL Atomic Intrinsic*\n\nPerforms an atomic addition of `value` to destination `dest`.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedAdd(inout int dest, int value, out int original_value)", params: &["inout int dest", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedAdd(inout uint dest, uint value, out uint original_value)", params: &["inout uint dest", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedMin",
        description: "### `InterlockedMin`\n*Microsoft HLSL Atomic Intrinsic*\n\nPerforms an atomic minimum comparison and update.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedMin(inout int dest, int value, out int original_value)", params: &["inout int dest", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedMin(inout uint dest, uint value, out uint original_value)", params: &["inout uint dest", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedMax",
        description: "### `InterlockedMax`\n*Microsoft HLSL Atomic Intrinsic*\n\nPerforms an atomic maximum comparison and update.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedMax(inout int dest, int value, out int original_value)", params: &["inout int dest", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedMax(inout uint dest, uint value, out uint original_value)", params: &["inout uint dest", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedAnd",
        description: "### `InterlockedAnd`\n*Microsoft HLSL Atomic Intrinsic*\n\nPerforms an atomic bitwise AND operation.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedAnd(inout int dest, int value, out int original_value)", params: &["inout int dest", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedAnd(inout uint dest, uint value, out uint original_value)", params: &["inout uint dest", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedOr",
        description: "### `InterlockedOr`\n*Microsoft HLSL Atomic Intrinsic*\n\nPerforms an atomic bitwise OR operation.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedOr(inout int dest, int value, out int original_value)", params: &["inout int dest", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedOr(inout uint dest, uint value, out uint original_value)", params: &["inout uint dest", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedXor",
        description: "### `InterlockedXor`\n*Microsoft HLSL Atomic Intrinsic*\n\nPerforms an atomic bitwise XOR operation.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedXor(inout int dest, int value, out int original_value)", params: &["inout int dest", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedXor(inout uint dest, uint value, out uint original_value)", params: &["inout uint dest", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedExchange",
        description: "### `InterlockedExchange`\n*Microsoft HLSL Atomic Intrinsic*\n\nAtomically assigns `value` to `dest` and returns the old value.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedExchange(inout int dest, int value, out int original_value)", params: &["inout int dest", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedExchange(inout uint dest, uint value, out uint original_value)", params: &["inout uint dest", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedCompareExchange",
        description: "### `InterlockedCompareExchange`\n*Microsoft HLSL Atomic Intrinsic*\n\nAtomically compares `dest` to `compare_value`. If they are equal, assigns `value` to `dest`.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedCompareExchange(inout int dest, int compare_value, int value, out int original_value)", params: &["inout int dest", "int compare_value", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedCompareExchange(inout uint dest, uint compare_value, uint value, out uint original_value)", params: &["inout uint dest", "uint compare_value", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "CheckAccessFullyMapped",
        description: "### `CheckAccessFullyMapped`\n*Microsoft HLSL Intrinsic (Tiled Resources)*\n\nChecks whether all bytes accessed in a tiled resource were mapped in physical memory.",
        overloads: &[
            BuiltinOverload { label: "bool CheckAccessFullyMapped(uint status)", params: &["uint status"] },
        ],
    },

    // ------------------------------------------------------------------------
    // Additional Unity URP & Built-in Shader Library Helpers
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "TransformWorldToView",
        description: "### `TransformWorldToView`\n*Unity URP / ShaderVariablesFunctions.hlsl*\n\nTransforms a 3D position from World Space to View (Camera) Space.",
        overloads: &[
            BuiltinOverload { label: "float3 TransformWorldToView(float3 positionWS)", params: &["float3 positionWS"] },
        ],
    },
    BuiltinFunction {
        name: "TransformViewToHClip",
        description: "### `TransformViewToHClip`\n*Unity URP / ShaderVariablesFunctions.hlsl*\n\nTransforms a 3D position from View (Camera) Space to Homogeneous Clip Space.",
        overloads: &[
            BuiltinOverload { label: "float4 TransformViewToHClip(float3 positionVS)", params: &["float3 positionVS"] },
        ],
    },
    BuiltinFunction {
        name: "TransformHClipToView",
        description: "### `TransformHClipToView`\n*Unity URP / ShaderVariablesFunctions.hlsl*\n\nTransforms a position from Homogeneous Clip Space back to View Space.",
        overloads: &[
            BuiltinOverload { label: "float3 TransformHClipToView(float4 positionCS)", params: &["float4 positionCS"] },
        ],
    },
    BuiltinFunction {
        name: "TransformObjectToWorldDir",
        description: "### `TransformObjectToWorldDir`\n*Unity URP / ShaderVariablesFunctions.hlsl*\n\nTransforms a direction vector from Object Space to World Space (ignoring translation).",
        overloads: &[
            BuiltinOverload { label: "float3 TransformObjectToWorldDir(float3 dirOS, bool doNormalize = true)", params: &["float3 dirOS", "bool doNormalize = true"] },
        ],
    },
    BuiltinFunction {
        name: "TransformWorldToObjectDir",
        description: "### `TransformWorldToObjectDir`\n*Unity URP / ShaderVariablesFunctions.hlsl*\n\nTransforms a direction vector from World Space to Object Space (ignoring translation).",
        overloads: &[
            BuiltinOverload { label: "float3 TransformWorldToObjectDir(float3 dirWS, bool doNormalize = true)", params: &["float3 dirWS", "bool doNormalize = true"] },
        ],
    },
    BuiltinFunction {
        name: "TransformViewToWorld",
        description: "### `TransformViewToWorld`\n*Unity URP / ShaderVariablesFunctions.hlsl*\n\nTransforms a position from View (Camera) Space to World Space.",
        overloads: &[
            BuiltinOverload { label: "float3 TransformViewToWorld(float3 positionVS)", params: &["float3 positionVS"] },
        ],
    },
    BuiltinFunction {
        name: "TransformWorldToObjectNormal",
        description: "### `TransformWorldToObjectNormal`\n*Unity URP / ShaderVariablesFunctions.hlsl*\n\nTransforms a normal vector from World Space to Object Space using inverse transpose.",
        overloads: &[
            BuiltinOverload { label: "float3 TransformWorldToObjectNormal(float3 normalWS, bool doNormalize = true)", params: &["float3 normalWS", "bool doNormalize = true"] },
        ],
    },
    BuiltinFunction {
        name: "TransformWorldToTangent",
        description: "### `TransformWorldToTangent`\n*Unity URP / ShaderVariablesFunctions.hlsl*\n\nTransforms a world space vector into Tangent Space.",
        overloads: &[
            BuiltinOverload { label: "float3 TransformWorldToTangent(float3 dirWS, float3 normalWS, float4 tangentWS)", params: &["float3 dirWS", "float3 normalWS", "float4 tangentWS"] },
        ],
    },
    BuiltinFunction {
        name: "TransformTangentToWorld",
        description: "### `TransformTangentToWorld`\n*Unity URP / ShaderVariablesFunctions.hlsl*\n\nTransforms a tangent space vector into World Space.",
        overloads: &[
            BuiltinOverload { label: "float3 TransformTangentToWorld(float3 dirTS, float3 normalWS, float4 tangentWS)", params: &["float3 dirTS", "float3 normalWS", "float4 tangentWS"] },
        ],
    },
    BuiltinFunction {
        name: "ComputeScreenPos",
        description: "### `ComputeScreenPos`\n*Unity URP & Built-in / Common.hlsl*\n\nCalculates normalized device screen coordinates suitable for projected texture sampling.",
        overloads: &[
            BuiltinOverload { label: "float4 ComputeScreenPos(float4 positionCS)", params: &["float4 positionCS"] },
        ],
    },
    BuiltinFunction {
        name: "GetVertexPositionInputs",
        description: "### `GetVertexPositionInputs`\n*Unity URP / Core.hlsl*\n\nHelper structure providing `positionWS`, `positionVS`, `positionCS`, and `positionNDC` from object-space vertex position.",
        overloads: &[
            BuiltinOverload { label: "VertexPositionInputs GetVertexPositionInputs(float3 positionOS)", params: &["float3 positionOS"] },
        ],
    },
    BuiltinFunction {
        name: "GetVertexNormalInputs",
        description: "### `GetVertexNormalInputs`\n*Unity URP / Core.hlsl*\n\nHelper structure returning `normalWS`, `tangentWS`, and `bitangentWS`.",
        overloads: &[
            BuiltinOverload { label: "VertexNormalInputs GetVertexNormalInputs(float3 normalOS)", params: &["float3 normalOS"] },
            BuiltinOverload { label: "VertexNormalInputs GetVertexNormalInputs(float3 normalOS, float4 tangentOS)", params: &["float3 normalOS", "float4 tangentOS"] },
        ],
    },
    BuiltinFunction {
        name: "LinearEyeDepth",
        description: "### `LinearEyeDepth`\n*Unity Shader Library*\n\nConverts non-linear hardware Z buffer depth into linear eye-space units (distance in world meters from near plane).",
        overloads: &[
            BuiltinOverload { label: "float LinearEyeDepth(float depth, float4 zBufferParam)", params: &["float depth", "float4 zBufferParam"] },
        ],
    },
    BuiltinFunction {
        name: "Linear01Depth",
        description: "### `Linear01Depth`\n*Unity Shader Library*\n\nConverts non-linear hardware Z buffer depth into linear [0, 1] range.",
        overloads: &[
            BuiltinOverload { label: "float Linear01Depth(float depth, float4 zBufferParam)", params: &["float depth", "float4 zBufferParam"] },
        ],
    },
    BuiltinFunction {
        name: "SampleSceneDepth",
        description: "### `SampleSceneDepth`\n*Unity URP / DeclareDepthTexture.hlsl*\n\nSamples the screen depth buffer texture (`_CameraDepthTexture`).",
        overloads: &[
            BuiltinOverload { label: "float SampleSceneDepth(float2 uv)", params: &["float2 uv"] },
        ],
    },
    BuiltinFunction {
        name: "SampleSceneColor",
        description: "### `SampleSceneColor`\n*Unity URP / DeclareOpaqueTexture.hlsl*\n\nSamples the camera opaque color texture (`_CameraOpaqueTexture`).",
        overloads: &[
            BuiltinOverload { label: "float3 SampleSceneColor(float2 uv)", params: &["float2 uv"] },
        ],
    },
    BuiltinFunction {
        name: "SAMPLE_TEXTURE2D_LOD",
        description: "### `SAMPLE_TEXTURE2D_LOD`\n*Unity URP Macro / Common.hlsl*\n\nSamples a 2D texture at an explicit level-of-detail (mip level).",
        overloads: &[
            BuiltinOverload { label: "float4 SAMPLE_TEXTURE2D_LOD(Texture2D textureName, SamplerState samplerName, float2 coord2, float lod)", params: &["Texture2D textureName", "SamplerState samplerName", "float2 coord2", "float lod"] },
        ],
    },
    BuiltinFunction {
        name: "SAMPLE_TEXTURE2D_BIAS",
        description: "### `SAMPLE_TEXTURE2D_BIAS`\n*Unity URP Macro / Common.hlsl*\n\nSamples a 2D texture with a specified mip level bias offset.",
        overloads: &[
            BuiltinOverload { label: "float4 SAMPLE_TEXTURE2D_BIAS(Texture2D textureName, SamplerState samplerName, float2 coord2, float bias)", params: &["Texture2D textureName", "SamplerState samplerName", "float2 coord2", "float bias"] },
        ],
    },
    BuiltinFunction {
        name: "SAMPLE_TEXTURE2D_ARRAY",
        description: "### `SAMPLE_TEXTURE2D_ARRAY`\n*Unity URP Macro / Common.hlsl*\n\nSamples a slice from a Texture2DArray.",
        overloads: &[
            BuiltinOverload { label: "float4 SAMPLE_TEXTURE2D_ARRAY(Texture2DArray textureName, SamplerState samplerName, float2 coord2, float index)", params: &["Texture2DArray textureName", "SamplerState samplerName", "float2 coord2", "float index"] },
        ],
    },
    BuiltinFunction {
        name: "SAMPLE_TEXTURECUBE",
        description: "### `SAMPLE_TEXTURECUBE`\n*Unity URP Macro / Common.hlsl*\n\nSamples a cubemap reflection texture using a 3D direction vector.",
        overloads: &[
            BuiltinOverload { label: "float4 SAMPLE_TEXTURECUBE(TextureCube textureName, SamplerState samplerName, float3 coord3)", params: &["TextureCube textureName", "SamplerState samplerName", "float3 coord3"] },
        ],
    },
    BuiltinFunction {
        name: "SAMPLE_TEXTURECUBE_LOD",
        description: "### `SAMPLE_TEXTURECUBE_LOD`\n*Unity URP Macro / Common.hlsl*\n\nSamples a cubemap texture at an explicit mip level (commonly used for roughness-based specular IBL reflections).",
        overloads: &[
            BuiltinOverload { label: "float4 SAMPLE_TEXTURECUBE_LOD(TextureCube textureName, SamplerState samplerName, float3 coord3, float lod)", params: &["TextureCube textureName", "SamplerState samplerName", "float3 coord3", "float lod"] },
        ],
    },
    BuiltinFunction {
        name: "SAMPLE_DEPTH_TEXTURE",
        description: "### `SAMPLE_DEPTH_TEXTURE`\n*Unity URP Macro / Common.hlsl*\n\nSamples a single float depth value from a depth texture.",
        overloads: &[
            BuiltinOverload { label: "float SAMPLE_DEPTH_TEXTURE(Texture2D textureName, SamplerState samplerName, float2 coord2)", params: &["Texture2D textureName", "SamplerState samplerName", "float2 coord2"] },
        ],
    },
    BuiltinFunction {
        name: "LightingPhysicallyBased",
        description: "### `LightingPhysicallyBased`\n*Unity URP / Lighting.hlsl*\n\nCalculates physically based BRDF direct lighting from a Light source on a surface.",
        overloads: &[
            BuiltinOverload { label: "half3 LightingPhysicallyBased(BRDFData brdfData, Light light, half3 normalWS, half3 viewDirectionWS)", params: &["BRDFData brdfData", "Light light", "half3 normalWS", "half3 viewDirectionWS"] },
        ],
    },
    BuiltinFunction {
        name: "LightingSpecular",
        description: "### `LightingSpecular`\n*Unity URP / Lighting.hlsl*\n\nComputes Blinn-Phong specular highlight illumination.",
        overloads: &[
            BuiltinOverload { label: "half3 LightingSpecular(half3 lightColor, half3 lightDir, half3 normalWS, half3 viewDirectionWS, half4 specular, half smoothness)", params: &["half3 lightColor", "half3 lightDir", "half3 normalWS", "half3 viewDirectionWS", "half4 specular", "half smoothness"] },
        ],
    },
    BuiltinFunction {
        name: "UniversalFragmentPBR",
        description: "### `UniversalFragmentPBR`\n*Unity URP / UniversalFragmentPBR.hlsl*\n\nStandard complete Universal Render Pipeline fragment PBR shading evaluation.",
        overloads: &[
            BuiltinOverload { label: "half4 UniversalFragmentPBR(inout InputData inputData, SurfaceData surfaceData)", params: &["inout InputData inputData", "SurfaceData surfaceData"] },
        ],
    },
    BuiltinFunction {
        name: "UniversalFragmentBlinnPhong",
        description: "### `UniversalFragmentBlinnPhong`\n*Unity URP / UniversalFragmentBlinnPhong.hlsl*\n\nStandard SimpleLit Blinn-Phong fragment evaluation.",
        overloads: &[
            BuiltinOverload { label: "half4 UniversalFragmentBlinnPhong(inout InputData inputData, SurfaceData surfaceData)", params: &["inout InputData inputData", "SurfaceData surfaceData"] },
        ],
    },
    BuiltinFunction {
        name: "SRGBToLinear",
        description: "### `SRGBToLinear`\n*Unity Core Library / Color.hlsl*\n\nConverts color values from sRGB gamma space to linear RGB space.",
        overloads: &[
            BuiltinOverload { label: "half3 SRGBToLinear(half3 c)", params: &["half3 c"] },
            BuiltinOverload { label: "half4 SRGBToLinear(half4 c)", params: &["half4 c"] },
        ],
    },
    BuiltinFunction {
        name: "LinearToSRGB",
        description: "### `LinearToSRGB`\n*Unity Core Library / Color.hlsl*\n\nConverts linear color values to standard sRGB gamma space.",
        overloads: &[
            BuiltinOverload { label: "half3 LinearToSRGB(half3 c)", params: &["half3 c"] },
            BuiltinOverload { label: "half4 LinearToSRGB(half4 c)", params: &["half4 c"] },
        ],
    },
    BuiltinFunction {
        name: "FastSRGBToLinear",
        description: "### `FastSRGBToLinear`\n*Unity Core Library / Color.hlsl*\n\nApproximates sRGB to linear conversion using gamma 2.2 approximation ($c^{2.2}$).",
        overloads: &[
            BuiltinOverload { label: "half3 FastSRGBToLinear(half3 c)", params: &["half3 c"] },
        ],
    },
    BuiltinFunction {
        name: "FastLinearToSRGB",
        description: "### `FastLinearToSRGB`\n*Unity Core Library / Color.hlsl*\n\nApproximates linear to sRGB conversion using $c^{1/2.2}$.",
        overloads: &[
            BuiltinOverload { label: "half3 FastLinearToSRGB(half3 c)", params: &["half3 c"] },
        ],
    },
    BuiltinFunction {
        name: "SafeNormalize",
        description: "### `SafeNormalize`\n*Unity & Unreal Math Helper*\n\nNormalizes vector safely, preventing division-by-zero NaN artifacts if vector magnitude is near 0.",
        overloads: &[
            BuiltinOverload { label: "float3 SafeNormalize(float3 inVec)", params: &["float3 inVec"] },
            BuiltinOverload { label: "float2 SafeNormalize(float2 inVec)", params: &["float2 inVec"] },
        ],
    },
    BuiltinFunction {
        name: "SafePositivePow",
        description: "### `SafePositivePow`\n*Unity Core Library / Common.hlsl*\n\nComputes $\\max(\\text{base}, 0)^{\\text{power}}$ safely avoiding negative base undefined behaviors.",
        overloads: &[
            BuiltinOverload { label: "float SafePositivePow(float base, float power)", params: &["float base", "float power"] },
        ],
    },
    BuiltinFunction {
        name: "UnityWorldToClipPos",
        description: "### `UnityWorldToClipPos`\n*Unity Built-in / UnityCG.cginc*\n\nTransforms a world space position directly into homogeneous clip space.",
        overloads: &[
            BuiltinOverload { label: "float4 UnityWorldToClipPos(float3 pos)", params: &["float3 pos"] },
        ],
    },
    BuiltinFunction {
        name: "UnityWorldToViewPos",
        description: "### `UnityWorldToViewPos`\n*Unity Built-in / UnityCG.cginc*\n\nTransforms a world space position into view (camera) space.",
        overloads: &[
            BuiltinOverload { label: "float3 UnityWorldToViewPos(float3 pos)", params: &["float3 pos"] },
        ],
    },
    BuiltinFunction {
        name: "UnityViewToClipPos",
        description: "### `UnityViewToClipPos`\n*Unity Built-in / UnityCG.cginc*\n\nTransforms a view space position into clip space using projection matrix.",
        overloads: &[
            BuiltinOverload { label: "float4 UnityViewToClipPos(float3 pos)", params: &["float3 pos"] },
        ],
    },

    // ------------------------------------------------------------------------
    // Additional Unreal Engine Material & Shading Helpers
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "GetWorldNormal",
        description: "### `GetWorldNormal`\n*Unreal Engine Material Template*\n\nReturns the interpolated world space normal vector of the pixel.",
        overloads: &[
            BuiltinOverload { label: "float3 GetWorldNormal(FMaterialPixelParameters Parameters)", params: &["FMaterialPixelParameters Parameters"] },
        ],
    },
    BuiltinFunction {
        name: "GetMaterialEmissive",
        description: "### `GetMaterialEmissive`\n*Unreal Engine Material Template*\n\nEvaluates the emissive color output from the pixel material graph.",
        overloads: &[
            BuiltinOverload { label: "float3 GetMaterialEmissive(FPixelMaterialInputs PixelMaterialInputs)", params: &["FPixelMaterialInputs PixelMaterialInputs"] },
        ],
    },
    BuiltinFunction {
        name: "GetMaterialBaseColor",
        description: "### `GetMaterialBaseColor`\n*Unreal Engine Material Template*\n\nEvaluates the diffuse albedo base color output from the pixel material inputs.",
        overloads: &[
            BuiltinOverload { label: "float3 GetMaterialBaseColor(FPixelMaterialInputs PixelMaterialInputs)", params: &["FPixelMaterialInputs PixelMaterialInputs"] },
        ],
    },
    BuiltinFunction {
        name: "AntialiasedTextureMask",
        description: "### `AntialiasedTextureMask`\n*Unreal Engine Material Function*\n\nProduces an anti-aliased procedural alpha cutout mask from a texture channel.",
        overloads: &[
            BuiltinOverload { label: "float AntialiasedTextureMask(Texture2D Tex, SamplerState Sampler, float2 UV, float Threshold, float Width)", params: &["Texture2D Tex", "SamplerState Sampler", "float2 UV", "float Threshold", "float Width"] },
        ],
    },
    BuiltinFunction {
        name: "VectorToRadialValue",
        description: "### `VectorToRadialValue`\n*Unreal Engine Material Function*\n\nConverts a 2D UV direction offset into a normalized angle $[0, 1]$.",
        overloads: &[
            BuiltinOverload { label: "float VectorToRadialValue(float2 Vector)", params: &["float2 Vector"] },
        ],
    },
    BuiltinFunction {
        name: "UnitVectorToOctahedron",
        description: "### `UnitVectorToOctahedron`\n*Unreal Engine Octahedral Normal Encoding*\n\nEncodes a 3D unit normal vector into a 2D octahedral UV coordinate.",
        overloads: &[
            BuiltinOverload { label: "float2 UnitVectorToOctahedron(float3 N)", params: &["float3 N"] },
        ],
    },
    BuiltinFunction {
        name: "OctahedronToUnitVector",
        description: "### `OctahedronToUnitVector`\n*Unreal Engine Octahedral Normal Decoding*\n\nDecodes a 2D octahedral coordinate back into a normalized 3D unit vector.",
        overloads: &[
            BuiltinOverload { label: "float3 OctahedronToUnitVector(float2 Oct)", params: &["float2 Oct"] },
        ],
    },
    BuiltinFunction {
        name: "RGBToHSV",
        description: "### `RGBToHSV`\n*Unreal Engine / Color Utility*\n\nConverts RGB color tuple to Hue, Saturation, Value (HSV) color space.",
        overloads: &[
            BuiltinOverload { label: "float3 RGBToHSV(float3 RGB)", params: &["float3 RGB"] },
        ],
    },
    BuiltinFunction {
        name: "HSVToRGB",
        description: "### `HSVToRGB`\n*Unreal Engine / Color Utility*\n\nConverts Hue, Saturation, Value (HSV) color back to RGB space.",
        overloads: &[
            BuiltinOverload { label: "float3 HSVToRGB(float3 HSV)", params: &["float3 HSV"] },
        ],
    },
    BuiltinFunction {
        name: "DeriveFilterWidth",
        description: "### `DeriveFilterWidth`\n*Unreal Engine Utility*\n\nCalculates anisotropic filter footprint width from UV screen space derivatives.",
        overloads: &[
            BuiltinOverload { label: "float2 DeriveFilterWidth(float2 UV)", params: &["float2 UV"] },
        ],
    },
    BuiltinFunction {
        name: "DepthBiasedAlpha",
        description: "### `DepthBiasedAlpha`\n*Unreal Engine Material Function*\n\nCalculates soft particle depth fading alpha factor against scene geometry.",
        overloads: &[
            BuiltinOverload { label: "float DepthBiasedAlpha(FMaterialPixelParameters Parameters, float InAlpha, float InBias, float InFactor)", params: &["FMaterialPixelParameters Parameters", "float InAlpha", "float InBias", "float InFactor"] },
        ],
    },
];

pub static BUILTIN_TYPES: &[&str] = &[
    // Scalars & Vectors
    "float", "float2", "float3", "float4",
    "half", "half2", "half3", "half4",
    "int", "int2", "int3", "int4",
    "uint", "uint2", "uint3", "uint4",
    "bool", "bool2", "bool3", "bool4",
    "double",
    // Matrices
    "float4x4", "float3x3", "float2x2",
    "half4x4", "half3x3", "matrix",
    // Textures & Samplers
    "Texture2D", "Texture2DArray", "Texture3D", "TextureCube",
    "SamplerState", "SamplerComparisonState",
    "sampler2D", "samplerCUBE",
    // Buffers
    "cbuffer", "tbuffer",
    "StructuredBuffer", "RWStructuredBuffer",
    "ByteAddressBuffer", "RWByteAddressBuffer",
];

pub static BUILTIN_VARIABLES: &[(&str, &str)] = &[
    // HLSL System-Value Semantics (Pixel / Output)
    ("SV_Target", "System-Value: Render target output 0 color (`float4`)."),
    ("SV_Target0", "System-Value: Render target output 0 color (`float4`)."),
    ("SV_Target1", "System-Value: Render target output 1 color (`float4`)."),
    ("SV_Target2", "System-Value: Render target output 2 color (`float4`)."),
    ("SV_Target3", "System-Value: Render target output 3 color (`float4`)."),
    ("SV_Target4", "System-Value: Render target output 4 color (`float4`)."),
    ("SV_Target5", "System-Value: Render target output 5 color (`float4`)."),
    ("SV_Target6", "System-Value: Render target output 6 color (`float4`)."),
    ("SV_Target7", "System-Value: Render target output 7 color (`float4`)."),
    ("SV_Depth", "System-Value: Output pixel depth (`float`)."),
    ("SV_DepthGreaterEqual", "System-Value: Output depth must be >= rasterized depth (`float`)."),
    ("SV_DepthLessEqual", "System-Value: Output depth must be <= rasterized depth (`float`)."),
    ("SV_Coverage", "System-Value: Input/output MSAA sample coverage mask (`uint`)."),
    ("SV_InnerCoverage", "System-Value: MSAA inner-conservative coverage mask (`uint`)."),
    ("SV_IsFrontFace", "System-Value: Specifies whether primitive is front-facing (`bool`)."),
    ("SV_SampleIndex", "System-Value: MSAA sample index (`uint`)."),
    ("SV_ShadingRate", "System-Value: Variable Rate Shading (VRS) mask (`uint`)."),
    ("SV_Barycentrics", "System-Value: Pixel barycentric coordinates (`float3`)."),
    ("SV_RenderTargetArrayIndex", "System-Value: Target slice in a render-target array (`uint`)."),
    ("SV_ViewportArrayIndex", "System-Value: Viewport index for geometry rendering (`uint`)."),

    // HLSL System-Value Semantics (Vertex / Geometry)
    ("SV_Position", "System-Value: Homogeneous clip-space vertex position (`float4`)."),
    ("SV_VertexID", "System-Value: Per-vertex identifier generated by GPU (`uint`)."),
    ("SV_InstanceID", "System-Value: Per-instance identifier generated by GPU (`uint`)."),
    ("SV_PrimitiveID", "System-Value: Primitive index in geometry/pixel shader (`uint`)."),
    ("SV_GSInstanceID", "System-Value: Geometry shader instance identifier (`uint`)."),
    ("SV_ClipDistance", "System-Value: Hardware user clipping distance plane (`float`)."),
    ("SV_ClipDistance0", "System-Value: Hardware user clipping distance plane 0 (`float`)."),
    ("SV_ClipDistance1", "System-Value: Hardware user clipping distance plane 1 (`float`)."),
    ("SV_ClipDistance2", "System-Value: Hardware user clipping distance plane 2 (`float`)."),
    ("SV_ClipDistance3", "System-Value: Hardware user clipping distance plane 3 (`float`)."),
    ("SV_CullDistance", "System-Value: Hardware user culling distance plane (`float`)."),
    ("SV_CullDistance0", "System-Value: Hardware user culling distance plane 0 (`float`)."),
    ("SV_CullDistance1", "System-Value: Hardware user culling distance plane 1 (`float`)."),
    ("SV_CullDistance2", "System-Value: Hardware user culling distance plane 2 (`float`)."),
    ("SV_CullDistance3", "System-Value: Hardware user culling distance plane 3 (`float`)."),

    // HLSL System-Value Semantics (Compute / Mesh Shaders)
    ("SV_DispatchThreadID", "System-Value: Global compute thread index across all groups (`uint3`)."),
    ("SV_GroupID", "System-Value: Compute thread group index (`uint3`)."),
    ("SV_GroupIndex", "System-Value: Flattened 1D thread index within current group (`uint`)."),
    ("SV_GroupThreadID", "System-Value: 3D thread index within current compute group (`uint3`)."),
    ("SV_DispatchGrid", "System-Value: Amplification shader dispatch grid dimension (`uint3`)."),

    // Classic Vertex / Interpolator Semantics
    ("POSITION", "Vertex attribute semantic: Object-space vertex position."),
    ("POSITION0", "Vertex attribute semantic: Object-space vertex position 0."),
    ("POSITION1", "Vertex attribute semantic: Object-space vertex position 1."),
    ("NORMAL", "Vertex attribute semantic: Surface normal vector."),
    ("NORMAL0", "Vertex attribute semantic: Surface normal vector 0."),
    ("NORMAL1", "Vertex attribute semantic: Surface normal vector 1."),
    ("TANGENT", "Vertex attribute semantic: Surface tangent vector."),
    ("TANGENT0", "Vertex attribute semantic: Surface tangent vector 0."),
    ("TANGENT1", "Vertex attribute semantic: Surface tangent vector 1."),
    ("BINORMAL", "Vertex attribute semantic: Surface binormal/bitangent vector."),
    ("BINORMAL0", "Vertex attribute semantic: Surface binormal/bitangent vector 0."),
    ("BINORMAL1", "Vertex attribute semantic: Surface binormal/bitangent vector 1."),
    ("TEXCOORD0", "Vertex attribute / interpolator semantic: UV coordinate channel 0."),
    ("TEXCOORD1", "Vertex attribute / interpolator semantic: UV coordinate channel 1."),
    ("TEXCOORD2", "Vertex attribute / interpolator semantic: UV coordinate channel 2."),
    ("TEXCOORD3", "Vertex attribute / interpolator semantic: UV coordinate channel 3."),
    ("TEXCOORD4", "Vertex attribute / interpolator semantic: UV coordinate channel 4."),
    ("TEXCOORD5", "Vertex attribute / interpolator semantic: UV coordinate channel 5."),
    ("TEXCOORD6", "Vertex attribute / interpolator semantic: UV coordinate channel 6."),
    ("TEXCOORD7", "Vertex attribute / interpolator semantic: UV coordinate channel 7."),
    ("COLOR", "Vertex attribute / interpolator semantic: Primary color."),
    ("COLOR0", "Vertex attribute / interpolator semantic: Per-vertex primary color channel 0."),
    ("COLOR1", "Vertex attribute / interpolator semantic: Per-vertex secondary color channel 1."),
    ("BLENDWEIGHT", "Vertex attribute semantic: Bone blend weight."),
    ("BLENDWEIGHT0", "Vertex attribute semantic: Bone blend weight 0."),
    ("BLENDWEIGHT1", "Vertex attribute semantic: Bone blend weight 1."),
    ("BLENDINDICES", "Vertex attribute semantic: Bone blend indices."),
    ("BLENDINDICES0", "Vertex attribute semantic: Bone blend indices 0."),
    ("BLENDINDICES1", "Vertex attribute semantic: Bone blend indices 1."),
    ("PSIZE", "Vertex attribute semantic: Point size for point sprites."),
];

pub static BUILTIN_KEYWORDS: &[&str] = &[
    // HLSL Keywords
    "struct", "cbuffer", "register", "static", "const", "inline",
    "return", "if", "else", "for", "while", "do", "switch", "case", "default",
    "break", "continue", "discard", "true", "false",
    "in", "out", "inout", "packoffset",
    // ShaderLab Keywords
    "Shader", "Properties", "SubShader", "Pass", "Tags",
    "Blend", "BlendOp", "ZWrite", "ZTest", "Cull", "ColorMask", "Offset",
    "Stencil", "Ref", "Comp", "Fail", "ZFail",
    "HLSLPROGRAM", "ENDHLSL", "CGPROGRAM", "ENDCG",
    "HLSLINCLUDE", "CGINCLUDE",
    "LOD", "Name", "Fallback", "CustomEditor",
];

pub struct MethodInfo {
    pub name: &'static str,
    pub signature: &'static str,
    pub snippet: &'static str,
    pub description: &'static str,
}

pub static TEXTURE_METHODS: &[MethodInfo] = &[
    MethodInfo {
        name: "Sample",
        signature: "float4 Sample(SamplerState s, float2 uv)",
        snippet: "Sample($1, $2)",
        description: "### `Texture.Sample`\n*HLSL Texture Method*\n\nSamples a texture using specified SamplerState and coordinates.",
    },
    MethodInfo {
        name: "SampleLevel",
        signature: "float4 SampleLevel(SamplerState s, float2 uv, float lod)",
        snippet: "SampleLevel($1, $2, $3)",
        description: "### `Texture.SampleLevel`\n*HLSL Texture Method*\n\nSamples a texture with an explicit mipmap level of detail (LOD).",
    },
    MethodInfo {
        name: "SampleBias",
        signature: "float4 SampleBias(SamplerState s, float2 uv, float bias)",
        snippet: "SampleBias($1, $2, $3)",
        description: "### `Texture.SampleBias`\n*HLSL Texture Method*\n\nSamples a texture after applying a bias to the calculated mipmap level.",
    },
    MethodInfo {
        name: "SampleGrad",
        signature: "float4 SampleGrad(SamplerState s, float2 uv, float2 ddx, float2 ddy)",
        snippet: "SampleGrad($1, $2, $3, $4)",
        description: "### `Texture.SampleGrad`\n*HLSL Texture Method*\n\nSamples a texture using explicit screen-space gradients (derivatives).",
    },
    MethodInfo {
        name: "SampleCmp",
        signature: "float SampleCmp(SamplerComparisonState s, float2 uv, float compare_value)",
        snippet: "SampleCmp($1, $2, $3)",
        description: "### `Texture.SampleCmp`\n*HLSL Texture Method*\n\nSamples a shadow/depth texture and compares against a reference value.",
    },
    MethodInfo {
        name: "SampleCmpLevelZero",
        signature: "float SampleCmpLevelZero(SamplerComparisonState s, float2 uv, float compare_value)",
        snippet: "SampleCmpLevelZero($1, $2, $3)",
        description: "### `Texture.SampleCmpLevelZero`\n*HLSL Texture Method*\n\nSamples a shadow/depth texture at mip level 0 and compares against a reference value.",
    },
    MethodInfo {
        name: "Load",
        signature: "float4 Load(int3 location)",
        snippet: "Load($1)",
        description: "### `Texture.Load`\n*HLSL Texture Method*\n\nReads raw texel data without any filtering or sampler state.",
    },
    MethodInfo {
        name: "GetDimensions",
        signature: "void GetDimensions(out uint width, out uint height)",
        snippet: "GetDimensions($1, $2)",
        description: "### `Texture.GetDimensions`\n*HLSL Texture Method*\n\nRetrieves texture dimensions (width and height in texels).",
    },
];

pub static BUFFER_METHODS: &[MethodInfo] = &[
    MethodInfo {
        name: "Load",
        signature: "T Load(int location)",
        snippet: "Load($1)",
        description: "### `StructuredBuffer.Load`\n*HLSL Buffer Method*\n\nReads an element from the buffer at the specified index.",
    },
    MethodInfo {
        name: "GetDimensions",
        signature: "void GetDimensions(out uint numStructs, out uint stride)",
        snippet: "GetDimensions($1, $2)",
        description: "### `StructuredBuffer.GetDimensions`\n*HLSL Buffer Method*\n\nRetrieves the number of elements and byte stride of the buffer.",
    },
];

pub fn find_builtin_function(name: &str) -> Option<&'static BuiltinFunction> {
    BUILTIN_FUNCTIONS.iter().find(|f| f.name == name)
}

pub static SHADERLAB_PROPERTY_TYPES: &[(&str, &str, &str)] = &[
    ("Color", "(\"Color\", Color) = (1, 1, 1, 1)", "RGBA color property with color picker"),
    ("Vector", "(\"Vector\", Vector) = (0, 0, 0, 0)", "4D floating-point vector property"),
    ("Float", "(\"Float\", Float) = 0.0", "Floating-point scalar property"),
    ("Int", "(\"Int\", Int) = 0", "Integer scalar property"),
    ("Range", "(\"Range\", Range(0, 1)) = 0.5", "Bounded slider floating-point property"),
    ("2D", "(\"Texture\", 2D) = \"white\" {}", "2D texture slot with default tint"),
    ("3D", "(\"Volume\", 3D) = \"\" {}", "3D volumetric texture slot"),
    ("Cube", "(\"Cubemap\", Cube) = \"\" {}", "Cubemap reflection texture slot"),
    ("2DArray", "(\"TextureArray\", 2DArray) = \"\" {}", "2D texture array slot"),
];

pub static SHADERLAB_RENDER_STATES: &[(&str, &str, &str)] = &[
    // Cull
    ("Cull", "Cull Back", "Culls back-facing polygons (default)"),
    ("Cull", "Cull Front", "Culls front-facing polygons (renders back faces)"),
    ("Cull", "Cull Off", "Disables culling (renders double-sided geometry)"),
    // ZWrite
    ("ZWrite", "ZWrite On", "Enables writing to the depth buffer (default for opaque)"),
    ("ZWrite", "ZWrite Off", "Disables writing to the depth buffer (standard for transparent)"),
    // ZTest
    ("ZTest", "ZTest LEqual", "Passes if fragment depth is less than or equal to current depth (default)"),
    ("ZTest", "ZTest Always", "Always passes depth test (renders on top of everything)"),
    ("ZTest", "ZTest Equal", "Passes only if fragment depth equals current depth"),
    ("ZTest", "ZTest Less", "Passes if fragment depth is strictly less than current depth"),
    ("ZTest", "ZTest Greater", "Passes if fragment depth is greater than current depth"),
    ("ZTest", "ZTest GEqual", "Passes if fragment depth is greater than or equal to current depth"),
    ("ZTest", "ZTest NotEqual", "Passes if fragment depth does not equal current depth"),
    // Blend
    ("Blend", "Blend Off", "Disables alpha blending (opaque rendering)"),
    ("Blend", "Blend SrcAlpha OneMinusSrcAlpha", "Standard traditional alpha blending: (Src * A) + (Dst * (1 - A))"),
    ("Blend", "Blend One One", "Additive blending (useful for particle effects, fire, lights)"),
    ("Blend", "Blend OneMinusDstColor One", "Soft additive blending"),
    ("Blend", "Blend DstColor Zero", "Multiplicative blending (shadows, darkening)"),
    ("Blend", "Blend SrcColor OneMinusSrcColor", "Color-weighted blending"),
    ("Blend", "Blend One OneMinusSrcAlpha", "Premultiplied alpha blending"),
    // ColorMask
    ("ColorMask", "ColorMask RGBA", "Writes to Red, Green, Blue, and Alpha channels (default)"),
    ("ColorMask", "ColorMask RGB", "Writes to Red, Green, and Blue channels, leaving Alpha untouched"),
    ("ColorMask", "ColorMask A", "Writes only to the Alpha channel"),
    ("ColorMask", "ColorMask 0", "Disables color output entirely (useful for depth-only or stencil-only passes)"),
    // Lighting
    ("Lighting", "Lighting Off", "Disables fixed-function lighting (standard for 2D sprites, UI, and unlit shaders)"),
    ("Lighting", "Lighting On", "Enables fixed-function lighting"),
];

pub static SHADERLAB_TAGS: &[(&str, &str)] = &[
    ("\"RenderType\"=\"Opaque\"", "Classifies shader as standard opaque surface for replacement shaders / depth passes"),
    ("\"RenderType\"=\"Transparent\"", "Classifies shader as transparent for sorting"),
    ("\"RenderType\"=\"TransparentCutout\"", "Classifies shader as alpha-tested cutout surface"),
    ("\"Queue\"=\"Geometry\"", "Renders in Geometry queue (2000, standard opaque)"),
    ("\"Queue\"=\"AlphaTest\"", "Renders in AlphaTest queue (2450, cutout opaque)"),
    ("\"Queue\"=\"Transparent\"", "Renders in Transparent queue (3000, back-to-front sorted)"),
    ("\"Queue\"=\"Overlay\"", "Renders in Overlay queue (4000, HUD / lens flares)"),
    // 2D Sprite & UI Tags
    ("\"CanUseSpriteAtlas\"=\"True\"", "Enables sprite packing and atlas coordinate UV remapping for 2D SpriteRenderer"),
    ("\"CanUseSpriteAtlas\"=\"False\"", "Disables sprite atlas packing"),
    ("\"PreviewType\"=\"Plane\"", "Renders flat 2D plane in Inspector material preview (standard for 2D Sprites and UI)"),
    ("\"PreviewType\"=\"Skybox\"", "Renders preview as a skybox sphere"),
    ("\"IgnoreProjector\"=\"True\"", "Ignores 3D projectors (standard for transparent 2D sprites, UI, and particles)"),
    // Render Pipeline & LightModes (3D & 2D)
    ("\"RenderPipeline\"=\"UniversalPipeline\"", "Restricts SubShader to Universal Render Pipeline (URP)"),
    ("\"RenderPipeline\"=\"HighDefinitionPipeline\"", "Restricts SubShader to High Definition Render Pipeline (HDRP)"),
    ("\"LightMode\"=\"UniversalForward\"", "URP forward main shading pass (3D)"),
    ("\"LightMode\"=\"UniversalGBuffer\"", "URP deferred GBuffer pass (3D)"),
    ("\"LightMode\"=\"Universal2D\"", "URP 2D Light pass: evaluates 2D Point, Freeform, Sprite, and Global lights (2D)"),
    ("\"LightMode\"=\"NormalsRendering\"", "URP 2D Normal Map rendering pass for dynamic 2D lighting (2D)"),
    ("\"LightMode\"=\"ShadowCaster\"", "Pass responsible for casting shadows into shadow maps"),
    ("\"LightMode\"=\"DepthOnly\"", "Pass rendering scene depth into _CameraDepthTexture"),
    ("\"LightMode\"=\"DepthNormals\"", "Pass rendering screen-space normals and depth"),
    ("\"LightMode\"=\"Meta\"", "Pass evaluated by Unity lightmapper for GI light baking"),
];

