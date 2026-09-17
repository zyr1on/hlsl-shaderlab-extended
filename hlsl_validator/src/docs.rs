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
    BuiltinFunction {
        name: "WaveActiveBallot",
        description: "### `WaveActiveBallot`\n*HLSL Wave Intrinsic (SM 6.0+)*\n\nReturns a 4-component unsigned integer bitmask representing evaluation of `expr` for all active lanes in the wave.",
        overloads: &[
            BuiltinOverload { label: "uint4 WaveActiveBallot(bool expr)", params: &["bool expr"] },
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
        description: "### `dst`\n*Microsoft HLSL Intrinsic*\n\nCalculates a distance/attenuation vector `(1.0, src0.y * src1.y, src0.z, src1.w)` from distance terms.\n\n**Parameters:**\n* `src0`: First source vector containing `(1.0, d, d^2, _)`. \n* `src1`: Second source vector containing `(1.0, 1/d, 1/d^2, _)`. ",
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
            BuiltinOverload { label: "void InterlockedAdd(inout int dest, int value)", params: &["inout int dest", "int value"] },
            BuiltinOverload { label: "void InterlockedAdd(inout uint dest, uint value)", params: &["inout uint dest", "uint value"] },
            BuiltinOverload { label: "void InterlockedAdd(inout int dest, int value, out int original_value)", params: &["inout int dest", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedAdd(inout uint dest, uint value, out uint original_value)", params: &["inout uint dest", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedMin",
        description: "### `InterlockedMin`\n*Microsoft HLSL Atomic Intrinsic*\n\nPerforms an atomic minimum comparison and update.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedMin(inout int dest, int value)", params: &["inout int dest", "int value"] },
            BuiltinOverload { label: "void InterlockedMin(inout uint dest, uint value)", params: &["inout uint dest", "uint value"] },
            BuiltinOverload { label: "void InterlockedMin(inout int dest, int value, out int original_value)", params: &["inout int dest", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedMin(inout uint dest, uint value, out uint original_value)", params: &["inout uint dest", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedMax",
        description: "### `InterlockedMax`\n*Microsoft HLSL Atomic Intrinsic*\n\nPerforms an atomic maximum comparison and update.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedMax(inout int dest, int value)", params: &["inout int dest", "int value"] },
            BuiltinOverload { label: "void InterlockedMax(inout uint dest, uint value)", params: &["inout uint dest", "uint value"] },
            BuiltinOverload { label: "void InterlockedMax(inout int dest, int value, out int original_value)", params: &["inout int dest", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedMax(inout uint dest, uint value, out uint original_value)", params: &["inout uint dest", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedAnd",
        description: "### `InterlockedAnd`\n*Microsoft HLSL Atomic Intrinsic*\n\nPerforms an atomic bitwise AND operation.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedAnd(inout int dest, int value)", params: &["inout int dest", "int value"] },
            BuiltinOverload { label: "void InterlockedAnd(inout uint dest, uint value)", params: &["inout uint dest", "uint value"] },
            BuiltinOverload { label: "void InterlockedAnd(inout int dest, int value, out int original_value)", params: &["inout int dest", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedAnd(inout uint dest, uint value, out uint original_value)", params: &["inout uint dest", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedOr",
        description: "### `InterlockedOr`\n*Microsoft HLSL Atomic Intrinsic*\n\nPerforms an atomic bitwise OR operation.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedOr(inout int dest, int value)", params: &["inout int dest", "int value"] },
            BuiltinOverload { label: "void InterlockedOr(inout uint dest, uint value)", params: &["inout uint dest", "uint value"] },
            BuiltinOverload { label: "void InterlockedOr(inout int dest, int value, out int original_value)", params: &["inout int dest", "int value", "out int original_value"] },
            BuiltinOverload { label: "void InterlockedOr(inout uint dest, uint value, out uint original_value)", params: &["inout uint dest", "uint value", "out uint original_value"] },
        ],
    },
    BuiltinFunction {
        name: "InterlockedXor",
        description: "### `InterlockedXor`\n*Microsoft HLSL Atomic Intrinsic*\n\nPerforms an atomic bitwise XOR operation.",
        overloads: &[
            BuiltinOverload { label: "void InterlockedXor(inout int dest, int value)", params: &["inout int dest", "int value"] },
            BuiltinOverload { label: "void InterlockedXor(inout uint dest, uint value)", params: &["inout uint dest", "uint value"] },
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
    // DirectX Raytracing (DXR) Intrinsics (Shader Model 6.3+)
    // ------------------------------------------------------------------------
    BuiltinFunction {
        name: "TraceRay",
        description: "### `TraceRay`\n*DirectX Raytracing (DXR) Intrinsic (SM 6.3+)*\n\nInitiates a ray traversal and intersection test through an acceleration structure.",
        overloads: &[
            BuiltinOverload { label: "void TraceRay(RaytracingAccelerationStructure AccelerationStructure, uint RayFlags, uint InstanceInclusionMask, uint RayContributionToHitGroupIndex, uint MultiplierForGeometryContributionToHitGroupIndex, uint MissShaderIndex, RayDesc Ray, inout payload_t Payload)", params: &["RaytracingAccelerationStructure AccelerationStructure", "uint RayFlags", "uint InstanceInclusionMask", "uint RayContributionToHitGroupIndex", "uint MultiplierForGeometryContributionToHitGroupIndex", "uint MissShaderIndex", "RayDesc Ray", "inout payload_t Payload"] },
        ],
    },
    BuiltinFunction {
        name: "WorldRayOrigin",
        description: "### `WorldRayOrigin`\n*DirectX Raytracing (DXR) Intrinsic (SM 6.3+)*\n\nReturns the origin of the current ray in world space coordinates (`float3`).",
        overloads: &[
            BuiltinOverload { label: "float3 WorldRayOrigin()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "WorldRayDirection",
        description: "### `WorldRayDirection`\n*DirectX Raytracing (DXR) Intrinsic (SM 6.3+)*\n\nReturns the direction vector of the current ray in world space coordinates (`float3`).",
        overloads: &[
            BuiltinOverload { label: "float3 WorldRayDirection()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "RayTCurrent",
        description: "### `RayTCurrent`\n*DirectX Raytracing (DXR) Intrinsic (SM 6.3+)*\n\nReturns the current parametric distance `t` along the ray for the closest hit found so far (`float`).",
        overloads: &[
            BuiltinOverload { label: "float RayTCurrent()", params: &[] },
        ],
    },
    BuiltinFunction {
        name: "RayTMin",
        description: "### `RayTMin`\n*DirectX Raytracing (DXR) Intrinsic (SM 6.3+)*\n\nReturns the minimum parametric distance `t_min` along the ray (`float`).",
        overloads: &[
            BuiltinOverload { label: "float RayTMin()", params: &[] },
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
        description: "### `FastSRGBToLinear`\n*Unity Core Library / Color.hlsl*\n\nApproximates sRGB to linear conversion using quadratic gamma 2.0 approximation ($c^2$).",
        overloads: &[
            BuiltinOverload { label: "half3 FastSRGBToLinear(half3 c)", params: &["half3 c"] },
        ],
    },
    BuiltinFunction {
        name: "FastLinearToSRGB",
        description: "### `FastLinearToSRGB`\n*Unity Core Library / Color.hlsl*\n\nApproximates linear to sRGB conversion using square-root gamma 2.0 approximation ($\\sqrt{c}$).",
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
    // Standard Textures & Samplers
    "Texture1D", "Texture1DArray",
    "Texture2D", "Texture2DArray", "Texture2DMS", "Texture2DMSArray",
    "Texture3D", "TextureCube", "TextureCubeArray",
    "SamplerState", "SamplerComparisonState",
    "sampler2D", "samplerCUBE",
    // Compute Shader & UAV Writable Textures
    "RWTexture1D", "RWTexture1DArray",
    "RWTexture2D", "RWTexture2DArray", "RWTexture3D",
    // Buffers & Streams
    "Buffer", "RWBuffer",
    "StructuredBuffer", "RWStructuredBuffer",
    "AppendStructuredBuffer", "ConsumeStructuredBuffer",
    "ByteAddressBuffer", "RWByteAddressBuffer",
    "tbuffer",
    "RasterizerOrderedTexture2D", "RasterizerOrderedStructuredBuffer", "RasterizerOrderedByteAddressBuffer",
    "PointStream", "LineStream", "TriangleStream",
    "InputPatch", "OutputPatch",
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

    // HLSL System-Value Semantics (Tessellation - Hull / Domain)
    ("SV_TessFactor", "System-Value: Patch edge tessellation factors (`float[2]`, `float[3]`, or `float[4]`)."),
    ("SV_InsideTessFactor", "System-Value: Patch interior tessellation factors (`float` or `float[2]`)."),
    ("SV_DomainLocation", "System-Value: Barycentric/domain coordinates for domain shader (`float2` or `float3`)."),
    ("SV_OutputControlPointID", "System-Value: Control point index processed by hull shader (`uint`)."),

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
    // HLSL Keywords & Qualifiers
    "groupshared", "cbuffer", "tbuffer", "struct", "register", "packoffset",
    "static", "const", "inline", "extern", "volatile", "precise",
    "return", "if", "else", "for", "while", "do", "switch", "case", "default",
    "break", "continue", "discard", "true", "false",
    "in", "out", "inout", "row_major", "column_major",
    "numthreads",
    // ShaderLab Keywords
    "Shader", "Properties", "SubShader", "Pass", "Tags",
    "Blend", "BlendOp", "ZWrite", "ZTest", "ZClip", "Cull", "ColorMask", "Offset",
    "Conservative", "AlphaToMask", "UsePass", "GrabPass",
    "Stencil", "Ref", "ReadMask", "WriteMask", "Comp", "Fail", "ZFail",
    "CompFront", "PassFront", "FailFront", "ZFailFront",
    "CompBack", "PassBack", "FailBack", "ZFailBack",
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
    MethodInfo {
        name: "Gather",
        signature: "float4 Gather(SamplerState s, float2 uv)",
        snippet: "Gather($1, $2)",
        description: "### `Texture.Gather`\n*HLSL Texture Method*\n\nGets the four texel values that would be used in a bi-linear filtering operation.",
    },
    MethodInfo {
        name: "GatherRed",
        signature: "float4 GatherRed(SamplerState s, float2 uv)",
        snippet: "GatherRed($1, $2)",
        description: "### `Texture.GatherRed`\n*HLSL Texture Method*\n\nGets the red component of the four texel values used in bi-linear filtering.",
    },
    MethodInfo {
        name: "GatherGreen",
        signature: "float4 GatherGreen(SamplerState s, float2 uv)",
        snippet: "GatherGreen($1, $2)",
        description: "### `Texture.GatherGreen`\n*HLSL Texture Method*\n\nGets the green component of the four texel values used in bi-linear filtering.",
    },
    MethodInfo {
        name: "GatherBlue",
        signature: "float4 GatherBlue(SamplerState s, float2 uv)",
        snippet: "GatherBlue($1, $2)",
        description: "### `Texture.GatherBlue`\n*HLSL Texture Method*\n\nGets the blue component of the four texel values used in bi-linear filtering.",
    },
    MethodInfo {
        name: "GatherAlpha",
        signature: "float4 GatherAlpha(SamplerState s, float2 uv)",
        snippet: "GatherAlpha($1, $2)",
        description: "### `Texture.GatherAlpha`\n*HLSL Texture Method*\n\nGets the alpha component of the four texel values used in bi-linear filtering.",
    },
    MethodInfo {
        name: "GatherCmp",
        signature: "float4 GatherCmp(SamplerComparisonState s, float2 uv, float compare_value)",
        snippet: "GatherCmp($1, $2, $3)",
        description: "### `Texture.GatherCmp`\n*HLSL Texture Method*\n\nSamples four texels, compares each against a reference value, and returns comparison results in a float4.",
    },
    MethodInfo {
        name: "CalculateLevelOfDetail",
        signature: "float CalculateLevelOfDetail(SamplerState s, float2 uv)",
        snippet: "CalculateLevelOfDetail($1, $2)",
        description: "### `Texture.CalculateLevelOfDetail`\n*HLSL Texture Method*\n\nCalculates the mipmap level of detail from UV gradients.",
    },
];

pub static RWTEXTURE_METHODS: &[MethodInfo] = &[
    MethodInfo {
        name: "GetDimensions",
        signature: "void GetDimensions(out uint width, out uint height)",
        snippet: "GetDimensions($1, $2)",
        description: "### `RWTexture.GetDimensions`\n*HLSL Unordered Access View Method*\n\nRetrieves the dimensions (width and height in texels) of this writable texture UAV.",
    },
    MethodInfo {
        name: "Load",
        signature: "float4 Load(int2 location)",
        snippet: "Load($1)",
        description: "### `RWTexture.Load`\n*HLSL Unordered Access View Method*\n\nReads raw texel data from this writable UAV at the specified integer coordinate without sampling.",
    },
];

pub static BUFFER_METHODS: &[MethodInfo] = &[
    MethodInfo {
        name: "GetDimensions",
        signature: "void GetDimensions(out uint numStructs, out uint stride)",
        snippet: "GetDimensions($1, $2)",
        description: "### `Buffer.GetDimensions`\n*HLSL Buffer Method*\n\nRetrieves the number of elements and byte stride of the buffer.",
    },
    MethodInfo {
        name: "Load",
        signature: "T Load(int location)",
        snippet: "Load($1)",
        description: "### `StructuredBuffer.Load`\n*HLSL Buffer Method*\n\nReads an element from the buffer at the specified index.",
    },
    MethodInfo {
        name: "Append",
        signature: "void Append(T value)",
        snippet: "Append($1)",
        description: "### `AppendStructuredBuffer.Append`\n*HLSL Buffer Method*\n\nAppends a new structure element to the end of the buffer and increments its internal counter.",
    },
    MethodInfo {
        name: "Consume",
        signature: "T Consume()",
        snippet: "Consume()",
        description: "### `ConsumeStructuredBuffer.Consume`\n*HLSL Buffer Method*\n\nConsumes and returns an element from the buffer and decrements its internal counter.",
    },
    MethodInfo {
        name: "IncrementCounter",
        signature: "uint IncrementCounter()",
        snippet: "IncrementCounter()",
        description: "### `RWStructuredBuffer.IncrementCounter`\n*HLSL Buffer Method*\n\nIncrements the hidden atomic counter associated with the UAV and returns the previous value.",
    },
    MethodInfo {
        name: "DecrementCounter",
        signature: "uint DecrementCounter()",
        snippet: "DecrementCounter()",
        description: "### `RWStructuredBuffer.DecrementCounter`\n*HLSL Buffer Method*\n\nDecrements the hidden atomic counter associated with the UAV and returns the new value.",
    },
    MethodInfo {
        name: "Load2",
        signature: "uint2 Load2(uint byteOffset)",
        snippet: "Load2($1)",
        description: "### `ByteAddressBuffer.Load2`\n*HLSL ByteAddressBuffer Method*\n\nLoads two 32-bit unsigned integers (64 bits) starting from the byte offset.",
    },
    MethodInfo {
        name: "Load3",
        signature: "uint3 Load3(uint byteOffset)",
        snippet: "Load3($1)",
        description: "### `ByteAddressBuffer.Load3`\n*HLSL ByteAddressBuffer Method*\n\nLoads three 32-bit unsigned integers (96 bits) starting from the byte offset.",
    },
    MethodInfo {
        name: "Load4",
        signature: "uint4 Load4(uint byteOffset)",
        snippet: "Load4($1)",
        description: "### `ByteAddressBuffer.Load4`\n*HLSL ByteAddressBuffer Method*\n\nLoads four 32-bit unsigned integers (128 bits) starting from the byte offset.",
    },
    MethodInfo {
        name: "Store",
        signature: "void Store(uint byteOffset, uint value)",
        snippet: "Store($1, $2)",
        description: "### `RWByteAddressBuffer.Store`\n*HLSL ByteAddressBuffer Method*\n\nStores a 32-bit unsigned integer into the raw buffer at the specified byte offset.",
    },
    MethodInfo {
        name: "Store2",
        signature: "void Store2(uint byteOffset, uint2 value)",
        snippet: "Store2($1, $2)",
        description: "### `RWByteAddressBuffer.Store2`\n*HLSL ByteAddressBuffer Method*\n\nStores two 32-bit unsigned integers into the raw buffer at the specified byte offset.",
    },
    MethodInfo {
        name: "Store3",
        signature: "void Store3(uint byteOffset, uint3 value)",
        snippet: "Store3($1, $2)",
        description: "### `RWByteAddressBuffer.Store3`\n*HLSL ByteAddressBuffer Method*\n\nStores three 32-bit unsigned integers into the raw buffer at the specified byte offset.",
    },
    MethodInfo {
        name: "Store4",
        signature: "void Store4(uint byteOffset, uint4 value)",
        snippet: "Store4($1, $2)",
        description: "### `RWByteAddressBuffer.Store4`\n*HLSL ByteAddressBuffer Method*\n\nStores four 32-bit unsigned integers into the raw buffer at the specified byte offset.",
    },
    MethodInfo {
        name: "InterlockedAdd",
        signature: "void InterlockedAdd(uint dest, uint value, out uint original_value)",
        snippet: "InterlockedAdd($1, $2, $3)",
        description: "### `RWByteAddressBuffer.InterlockedAdd`\n*HLSL Atomic Method*\n\nPerforms an atomic addition on the specified byte offset location.",
    },
    MethodInfo {
        name: "InterlockedAnd",
        signature: "void InterlockedAnd(uint dest, uint value, out uint original_value)",
        snippet: "InterlockedAnd($1, $2, $3)",
        description: "### `RWByteAddressBuffer.InterlockedAnd`\n*HLSL Atomic Method*\n\nPerforms an atomic bitwise AND on the specified byte offset location.",
    },
    MethodInfo {
        name: "InterlockedCompareExchange",
        signature: "void InterlockedCompareExchange(uint dest, uint compare_value, uint value, out uint original_value)",
        snippet: "InterlockedCompareExchange($1, $2, $3, $4)",
        description: "### `RWByteAddressBuffer.InterlockedCompareExchange`\n*HLSL Atomic Method*\n\nCompares the destination value with a reference value and conditionally stores a new value atomically.",
    },
    MethodInfo {
        name: "InterlockedCompareStore",
        signature: "void InterlockedCompareStore(uint dest, uint compare_value, uint value)",
        snippet: "InterlockedCompareStore($1, $2, $3)",
        description: "### `RWByteAddressBuffer.InterlockedCompareStore`\n*HLSL Atomic Method*\n\nCompares the destination value with a reference value and conditionally stores a new value atomically without returning the original.",
    },
    MethodInfo {
        name: "InterlockedExchange",
        signature: "void InterlockedExchange(uint dest, uint value, out uint original_value)",
        snippet: "InterlockedExchange($1, $2, $3)",
        description: "### `RWByteAddressBuffer.InterlockedExchange`\n*HLSL Atomic Method*\n\nAtomically assigns a value to the destination and returns the original value.",
    },
    MethodInfo {
        name: "InterlockedMax",
        signature: "void InterlockedMax(uint dest, uint value, out uint original_value)",
        snippet: "InterlockedMax($1, $2, $3)",
        description: "### `RWByteAddressBuffer.InterlockedMax`\n*HLSL Atomic Method*\n\nPerforms an atomic maximum comparison and stores the larger value.",
    },
    MethodInfo {
        name: "InterlockedMin",
        signature: "void InterlockedMin(uint dest, uint value, out uint original_value)",
        snippet: "InterlockedMin($1, $2, $3)",
        description: "### `RWByteAddressBuffer.InterlockedMin`\n*HLSL Atomic Method*\n\nPerforms an atomic minimum comparison and stores the smaller value.",
    },
    MethodInfo {
        name: "InterlockedOr",
        signature: "void InterlockedOr(uint dest, uint value, out uint original_value)",
        snippet: "InterlockedOr($1, $2, $3)",
        description: "### `RWByteAddressBuffer.InterlockedOr`\n*HLSL Atomic Method*\n\nPerforms an atomic bitwise OR on the specified byte offset location.",
    },
    MethodInfo {
        name: "InterlockedXor",
        signature: "void InterlockedXor(uint dest, uint value, out uint original_value)",
        snippet: "InterlockedXor($1, $2, $3)",
        description: "### `RWByteAddressBuffer.InterlockedXor`\n*HLSL Atomic Method*\n\nPerforms an atomic bitwise XOR on the specified byte offset location.",
    },
];

pub fn find_builtin_function(name: &str) -> Option<&'static BuiltinFunction> {
    BUILTIN_FUNCTIONS.iter().find(|f| f.name == name)
}

pub static SHADERLAB_PROPERTY_TYPES: &[(&str, &str, &str, &str)] = &[
    ("Float", "_${1:Float} (\"${2:Float}\", Float) = ${3:0.0}", " (\"${1:Float}\", Float) = ${2:0.0}", "Floating-point scalar property"),
    ("Int", "_${1:Int} (\"${2:Int}\", Int) = ${3:0}", " (\"${1:Int}\", Int) = ${2:0}", "Integer scalar property"),
    ("Range", "_${1:Range} (\"${2:Range}\", Range(${3:0}, ${4:1})) = ${5:0.5}", " (\"${1:Range}\", Range(${2:0}, ${3:1})) = ${4:0.5}", "Bounded slider floating-point property"),
    ("Color", "_${1:Color} (\"${2:Color}\", Color) = (${3:1, 1, 1, 1})", " (\"${1:Color}\", Color) = (${2:1, 1, 1, 1})", "RGBA color property with color picker"),
    ("Vector", "_${1:Vector} (\"${2:Vector}\", Vector) = (${3:0, 0, 0, 0})", " (\"${1:Vector}\", Vector) = (${2:0, 0, 0, 0})", "4D floating-point vector property"),
    ("2D", "_${1:MainTex} (\"${2:Texture}\", 2D) = \"${3:white}\" {}", " (\"${1:Texture}\", 2D) = \"${2:white}\" {}", "2D texture slot with default tint"),
    ("3D", "_${1:Volume} (\"${2:Volume}\", 3D) = \"\" {}", " (\"${1:Volume}\", 3D) = \"\" {}", "3D volumetric texture slot"),
    ("Cube", "_${1:Cubemap} (\"${2:Cubemap}\", Cube) = \"\" {}", " (\"${1:Cubemap}\", Cube) = \"\" {}", "Cubemap reflection texture slot"),
    ("2DArray", "_${1:TexArray} (\"${2:TextureArray}\", 2DArray) = \"\" {}", " (\"${1:TextureArray}\", 2DArray) = \"\" {}", "2D texture array slot"),
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
    ("ZTest", "ZTest Off", "Disables depth testing completely (equivalent to ZTest Always)"),
    // ZClip
    ("ZClip", "ZClip True", "Enables depth clipping (default). Fragments outside near/far clipping planes are clipped"),
    ("ZClip", "ZClip False", "Disables depth clipping (depth clamping). Fragments outside near/far planes are clamped instead of clipped (useful for shadow maps and portals)"),
    // Blend
    ("Blend", "Blend Off", "Disables alpha blending (opaque rendering)"),
    ("Blend", "Blend SrcAlpha OneMinusSrcAlpha", "Standard traditional alpha blending: (Src * A) + (Dst * (1 - A))"),
    ("Blend", "Blend One One", "Additive blending (useful for particle effects, fire, lights)"),
    ("Blend", "Blend OneMinusDstColor One", "Soft additive blending"),
    ("Blend", "Blend DstColor Zero", "Multiplicative blending (shadows, darkening)"),
    ("Blend", "Blend DstColor SrcColor", "2x multiplicative blending: Result = 2 * (Src * Dst)"),
    ("Blend", "Blend SrcColor OneMinusSrcColor", "Color-weighted blending"),
    ("Blend", "Blend One OneMinusSrcAlpha", "Premultiplied alpha blending"),
    ("Blend", "Blend SrcAlpha OneMinusSrcAlpha, One OneMinusSrcAlpha", "Separate blend: standard alpha blend for color, premultiplied alpha blend for alpha"),
    ("Blend", "Blend One One, One Zero", "Separate blend: additive color, preserves destination alpha"),
    ("Blend", "Blend 0 SrcAlpha OneMinusSrcAlpha", "Alpha blending specifically for render target 0"),
    ("Blend", "Blend 1 One One", "Additive blending specifically for render target 1"),
    // BlendOp
    ("BlendOp", "BlendOp Add", "Adds source and destination results together (default): Result = Src + Dst"),
    ("BlendOp", "BlendOp Sub", "Subtracts destination from source: Result = Src - Dst"),
    ("BlendOp", "BlendOp RevSub", "Subtracts source from destination: Result = Dst - Src"),
    ("BlendOp", "BlendOp Min", "Takes the smaller value between source and destination: Result = min(Src, Dst)"),
    ("BlendOp", "BlendOp Max", "Takes the larger value between source and destination: Result = max(Src, Dst)"),
    ("BlendOp", "BlendOp LogicalClear", "Logical blend operation: Result = 0 (DirectX 11.1+ / Vulkan)"),
    ("BlendOp", "BlendOp LogicalSet", "Logical blend operation: Result = ~0"),
    ("BlendOp", "BlendOp LogicalCopy", "Logical blend operation: Result = Src"),
    ("BlendOp", "BlendOp LogicalCopyInverted", "Logical blend operation: Result = ~Src"),
    ("BlendOp", "BlendOp LogicalNoop", "Logical blend operation: Result = Dst"),
    ("BlendOp", "BlendOp LogicalInvert", "Logical blend operation: Result = ~Dst"),
    ("BlendOp", "BlendOp LogicalAnd", "Logical blend operation: Result = Src & Dst"),
    ("BlendOp", "BlendOp LogicalNand", "Logical blend operation: Result = ~(Src & Dst)"),
    ("BlendOp", "BlendOp LogicalOr", "Logical blend operation: Result = Src | Dst"),
    ("BlendOp", "BlendOp LogicalNor", "Logical blend operation: Result = ~(Src | Dst)"),
    ("BlendOp", "BlendOp LogicalXor", "Logical blend operation: Result = Src ^ Dst"),
    ("BlendOp", "BlendOp LogicalEquiv", "Logical blend operation: Result = ~(Src ^ Dst)"),
    ("BlendOp", "BlendOp LogicalAndReverse", "Logical blend operation: Result = Src & ~Dst"),
    ("BlendOp", "BlendOp LogicalAndInverted", "Logical blend operation: Result = ~Src & Dst"),
    ("BlendOp", "BlendOp LogicalOrReverse", "Logical blend operation: Result = Src | ~Dst"),
    ("BlendOp", "BlendOp LogicalOrInverted", "Logical blend operation: Result = ~Src & Dst"),
    ("BlendOp", "BlendOp Add, Add", "Separate blend operation for color and alpha channels"),
    ("BlendOp", "BlendOp Min, Max", "Separate blend operation: Min for color, Max for alpha"),
    // ColorMask
    ("ColorMask", "ColorMask RGBA", "Writes to Red, Green, Blue, and Alpha channels (default)"),
    ("ColorMask", "ColorMask RGB", "Writes to Red, Green, and Blue channels, leaving Alpha untouched"),
    ("ColorMask", "ColorMask A", "Writes only to the Alpha channel"),
    ("ColorMask", "ColorMask 0", "Disables color output entirely (useful for depth-only or stencil-only passes)"),
    ("ColorMask", "ColorMask R", "Writes only to the Red channel"),
    ("ColorMask", "ColorMask G", "Writes only to the Green channel"),
    ("ColorMask", "ColorMask B", "Writes only to the Blue channel"),
    ("ColorMask", "ColorMask RGBA 0", "Sets color mask to RGBA for render target 0"),
    ("ColorMask", "ColorMask 0 1", "Disables color output on render target 1"),
    // Offset
    ("Offset", "Offset -1, -1", "Pulls polygon closer to camera (negative depth offset). Prevents Z-fighting on decals and wireframes"),
    ("Offset", "Offset 1, 1", "Pushes polygon further away from camera (positive depth offset)"),
    ("Offset", "Offset 0, 0", "Default depth offset (no bias)"),
    // Conservative
    ("Conservative", "Conservative True", "Enables conservative rasterization (pixel rendered if any polygon part touches it). Useful for shadow mapping and voxelization"),
    ("Conservative", "Conservative False", "Disables conservative rasterization (standard center sampling)"),
    // AlphaToMask
    ("AlphaToMask", "AlphaToMask On", "Enables Alpha-to-Coverage (MSAA alpha testing). Smooth alpha cutout on foliage and fences"),
    ("AlphaToMask", "AlphaToMask Off", "Disables Alpha-to-Coverage (default)"),
    // Stencil Commands
    ("Ref", "Ref 1", "Stencil reference value to compare against and/or write (0-255)"),
    ("ReadMask", "ReadMask 255", "Stencil 8-bit read mask (0-255)"),
    ("WriteMask", "WriteMask 255", "Stencil 8-bit write mask (0-255)"),
    ("Comp", "Comp Always", "Stencil test comparison: Always pass"),
    ("Comp", "Comp Equal", "Stencil test comparison: Pass if stencil buffer equals reference value"),
    ("Comp", "Comp NotEqual", "Stencil test comparison: Pass if stencil buffer does not equal reference value"),
    ("Comp", "Comp Less", "Stencil test comparison: Pass if stencil buffer is less than reference value"),
    ("Comp", "Comp Greater", "Stencil test comparison: Pass if stencil buffer is greater than reference value"),
    ("Comp", "Comp LEqual", "Stencil test comparison: Pass if stencil buffer is less than or equal to reference value"),
    ("Comp", "Comp GEqual", "Stencil test comparison: Pass if stencil buffer is greater than or equal to reference value"),
    ("Comp", "Comp Never", "Stencil test comparison: Never pass"),
    ("Pass", "Pass Keep", "Stencil operation when stencil and depth tests pass: Keep current stencil value"),
    ("Pass", "Pass Replace", "Stencil operation when stencil and depth tests pass: Replace with reference value"),
    ("Pass", "Pass Zero", "Stencil operation when stencil and depth tests pass: Set stencil to 0"),
    ("Pass", "Pass IncrSat", "Stencil operation when stencil and depth tests pass: Increment value, clamp at 255"),
    ("Pass", "Pass DecrSat", "Stencil operation when stencil and depth tests pass: Decrement value, clamp at 0"),
    ("Pass", "Pass Invert", "Stencil operation when stencil and depth tests pass: Bitwise invert stencil bits"),
    ("Pass", "Pass IncrWrap", "Stencil operation when stencil and depth tests pass: Increment value, wrap 255 to 0"),
    ("Pass", "Pass DecrWrap", "Stencil operation when stencil and depth tests pass: Decrement value, wrap 0 to 255"),
    ("Fail", "Fail Keep", "Stencil operation when stencil test fails: Keep current stencil value"),
    ("Fail", "Fail Replace", "Stencil operation when stencil test fails: Replace with reference value"),
    ("Fail", "Fail Zero", "Stencil operation when stencil test fails: Set stencil to 0"),
    ("ZFail", "ZFail Keep", "Stencil operation when stencil passes but depth test fails: Keep current stencil value"),
    ("ZFail", "ZFail Replace", "Stencil operation when stencil passes but depth test fails: Replace with reference value"),
    ("ZFail", "ZFail Zero", "Stencil operation when stencil passes but depth test fails: Set stencil to 0"),
    // Pass Commands
    ("UsePass", "UsePass \"Shader/PASSNAME\"", "Inserts a named pass from another shader into this SubShader"),
    ("GrabPass", "GrabPass { \"_BackgroundTexture\" }", "Captures current frame buffer into a named texture before executing this pass"),
    ("GrabPass", "GrabPass { }", "Captures current frame buffer into default _GrabTexture"),
    ("Name", "Name \"PassName\"", "Assigns a name to this pass (used by UsePass and frame debugger)"),
    ("LOD", "LOD 100", "Sets level of detail limit for SubShader (compared against Shader.globalMaximumLOD)"),
    ("LOD", "LOD 200", "Sets level of detail limit for SubShader"),
    // Lighting
    ("Lighting", "Lighting Off", "Disables fixed-function lighting (standard for 2D sprites, UI, and unlit shaders)"),
    ("Lighting", "Lighting On", "Enables fixed-function lighting"),
];


pub static SHADERLAB_ATTRIBUTES: &[(&str, &str, &str)] = &[
    ("MainColor", "MainColor", "Marks property as primary diffuse/base color in Material Inspector and URP/HDRP"),
    ("MainTexture", "MainTexture", "Marks property as primary diffuse/albedo texture in Material Inspector and URP/HDRP"),
    ("HDR", "HDR", "Enables high dynamic range (HDR) color picker with wide intensity range and exposure control"),
    ("HideInInspector", "HideInInspector", "Hides this property from the Unity Material Inspector"),
    ("NoScaleOffset", "NoScaleOffset", "Hides texture tiling and offset controls in the Material Inspector"),
    ("Normal", "Normal", "Validates that the assigned texture is marked as a Normal Map"),
    ("SingleLineTexture", "SingleLineTexture", "Displays texture property in a compact single-line field"),
    ("PerRendererData", "PerRendererData", "Fetches texture/property from SpriteRenderer or MaterialPropertyBlock"),
    ("MaterialToggle", "MaterialToggle", "Renders float as a checkbox toggle and defines a shader keyword (e.g. PROPERTY_ON)"),
    ("Toggle", "Toggle", "Renders float property as a checkbox toggle"),
    ("Toggle", "Toggle(${1:KEYWORD})", "Checkbox toggle setting a specific shader keyword when enabled"),
    ("KeywordEnum", "KeywordEnum(${1:ChoiceA, ChoiceB, ChoiceC})", "Displays a dropdown popup setting mutually exclusive shader keywords"),
    ("Enum", "Enum(${1:UnityEngine.Rendering.CullMode})", "Displays an enum dropdown from a C# Enum or custom values"),
    ("Space", "Space", "Adds vertical spacing in the Material Inspector"),
    ("Header", "Header(\"${1:Section Title}\")", "Adds a bold header section title in the Material Inspector"),
    ("IntRange", "IntRange", "Restricts Range slider values to whole integers"),
    ("Tooltip", "Tooltip(\"${1:Description}\")", "Adds a hover tooltip in the Material Inspector"),
];

pub static SHADERLAB_TAG_KEYS_AND_VALUES: &[(&str, &[&str], &str)] = &[
    ("RenderType", &["\"Opaque\"", "\"Transparent\"", "\"TransparentCutout\"", "\"Background\"", "\"Overlay\"", "\"TreeOpaque\"", "\"TreeTransparentCutout\"", "\"TreeSoftSurround\"", "\"Water\"", "\"Grass\""], "Classifies shader for replacement shaders and depth/shadow passes"),
    ("RenderPipeline", &["\"UniversalPipeline\"", "\"HighDefinitionPipeline\""], "Restricts SubShader to a specific Scriptable Render Pipeline"),
    ("Queue", &["\"Geometry\"", "\"Geometry+1\"", "\"AlphaTest\"", "\"Transparent\"", "\"Transparent+1\"", "\"Overlay\"", "\"Background\""], "Determines render sorting order queue"),
    ("LightMode", &["\"UniversalForward\"", "\"UniversalForwardOnly\"", "\"UniversalGBuffer\"", "\"Universal2D\"", "\"NormalsRendering\"", "\"ShadowCaster\"", "\"DepthOnly\"", "\"DepthNormals\"", "\"Meta\"", "\"SRPDefaultUnlit\"", "\"ForwardBase\"", "\"ForwardAdd\"", "\"Deferred\"", "\"Vertex\"", "\"VertexLM\"", "\"MotionVectors\"", "\"Always\""], "Specifies pass role in lighting and pipeline execution"),
    ("IgnoreProjector", &["\"True\"", "\"False\""], "Whether 3D projectors affect this material (True for 2D sprites/UI)"),
    ("PreviewType", &["\"Plane\"", "\"Skybox\""], "Shape displayed in the material preview Inspector (Plane for 2D/UI)"),
    ("CanUseSpriteAtlas", &["\"True\"", "\"False\""], "Enables UV coordinate packing for 2D sprite atlas rendering"),
    ("DisableBatching", &["\"True\"", "\"False\"", "\"LODFading\""], "Disables draw call dynamic batching if vertex shader modifies positions"),
    ("ForceNoShadowCasting", &["\"True\"", "\"False\""], "Prevents objects using this shader from casting shadows"),
    ("UniversalMaterialType", &["\"Lit\"", "\"Unlit\""], "URP material classification"),
    ("PassFlags", &["\"OnlyDirectional\""], "Pass execution flags"),
    ("RequireOptions", &["\"SoftVegetation\""], "Renders pass only when specified graphics options are enabled"),
    ("SortingOrder", &["\"FrontToBack\"", "\"BackToFront\""], "Determines pass sorting order for draw calls"),
    ("PerformanceChecks", &["\"False\""], "Disables shader compiler performance warnings for this pass"),
    ("ShaderModel", &["\"2.0\"", "\"3.0\"", "\"4.5\"", "\"5.0\""], "Target HLSL shader model requirement"),
];

pub static SHADERLAB_COMMON_PROPERTIES: &[(&str, &str, &str)] = &[
    ("[MainColor] _BaseColor", "[MainColor] _BaseColor (\"Base Color\", Color) = (1, 1, 1, 1)", "Standard URP/HDRP primary diffuse color"),
    ("[MainTexture] _BaseMap", "[MainTexture] _BaseMap (\"Base Map\", 2D) = \"white\" {}", "Standard URP/HDRP primary albedo texture"),
    ("_Color", "_Color (\"Color\", Color) = (1, 1, 1, 1)", "Built-in / Legacy standard color property"),
    ("_MainTex", "_MainTex (\"Texture\", 2D) = \"white\" {}", "Built-in / Legacy standard texture property"),
    ("_BumpMap", "[Normal] _BumpMap (\"Normal Map\", 2D) = \"bump\" {}", "Tangent-space normal map texture"),
    ("_Metallic", "_Metallic (\"Metallic\", Range(0.0, 1.0)) = 0.0", "Metallic surface parameter (0 = dielectric, 1 = metal)"),
    ("_Smoothness", "_Smoothness (\"Smoothness\", Range(0.0, 1.0)) = 0.5", "Surface smoothness / microfacet roughness parameter"),
    ("_EmissionColor", "[HDR] _EmissionColor (\"Emission Color\", Color) = (0, 0, 0, 1)", "HDR emissive glow color"),
    ("_EmissionMap", "_EmissionMap (\"Emission Map\", 2D) = \"black\" {}", "Emissive light mask texture"),
    ("_Cutoff", "_Cutoff (\"Alpha Cutoff\", Range(0.0, 1.0)) = 0.5", "Alpha testing / clipping threshold"),
    ("_BumpScale", "_BumpScale (\"Normal Scale\", Float) = 1.0", "Normal map perturbation intensity"),
    ("_OcclusionMap", "_OcclusionMap (\"Occlusion Map\", 2D) = \"white\" {}", "Ambient occlusion texture"),
    ("_OcclusionStrength", "_OcclusionStrength (\"Occlusion Strength\", Range(0.0, 1.0)) = 1.0", "Ambient occlusion shadow intensity"),
    ("_Glossiness", "_Glossiness (\"Smoothness\", Range(0.0, 1.0)) = 0.5", "Legacy / Standard shader smoothness parameter"),
    ("_SpecColor", "_SpecColor (\"Specular\", Color) = (0.2, 0.2, 0.2, 1.0)", "Specular reflection highlight color"),
    ("_DetailAlbedoMap", "_DetailAlbedoMap (\"Detail Albedo x2\", 2D) = \"gray\" {}", "Secondary detail albedo texture multiplied by 2"),
    ("_DetailNormalMap", "[Normal] _DetailNormalMap (\"Normal Map\", 2D) = \"bump\" {}", "Secondary high-frequency detail normal map"),
    ("_DetailNormalMapScale", "_DetailNormalMapScale (\"Scale\", Float) = 1.0", "Detail normal map perturbation scale"),
    ("_Surface", "_Surface (\"Surface Type\", Float) = 0.0", "URP surface type (0.0 = Opaque, 1.0 = Transparent)"),
    ("_Blend", "_Blend (\"Blend Mode\", Float) = 0.0", "URP blend mode (0 = Alpha, 1 = Premultiply, 2 = Additive, 3 = Multiply)"),
    ("_Cull", "_Cull (\"Culling\", Float) = 2.0", "Polygon culling mode (0 = Off, 1 = Front, 2 = Back)"),
    ("_ZWrite", "_ZWrite (\"ZWrite\", Float) = 1.0", "Depth write control (0.0 = Off, 1.0 = On)"),
    ("_QueueOffset", "_QueueOffset (\"Queue Offset\", Float) = 0.0", "Render queue priority offset"),
    ("_ReceiveShadows", "_ReceiveShadows (\"Receive Shadows\", Float) = 1.0", "Toggles receiving real-time shadows (0.0 = Off, 1.0 = On)"),
];

pub static SHADERLAB_PROPERTY_KEYWORD_SNIPPETS: &[(&str, &str, &str)] = &[
    ("float", "_${1:Float} (\"${2:Float}\", Float) = ${3:0.0}", "Snippet: Float scalar property"),
    ("range", "_${1:Range} (\"${2:Range}\", Range(${3:0.0}, ${4:1.0})) = ${5:0.5}", "Snippet: Bounded Range slider property"),
    ("color", "_${1:Color} (\"${2:Color}\", Color) = (${3:1, 1, 1, 1})", "Snippet: RGBA Color property"),
    ("hdr", "[HDR] _${1:EmissionColor} (\"${2:Emission Color}\", Color) = (${3:0, 0, 0, 1})", "Snippet: High Dynamic Range (HDR) Color property"),
    ("vector", "_${1:Vector} (\"${2:Vector}\", Vector) = (${3:0, 0, 0, 0})", "Snippet: 4D Vector property"),
    ("2d", "_${1:Texture} (\"${2:Texture}\", 2D) = \"${3:white}\" {}", "Snippet: 2D Texture property"),
    ("tex", "_${1:Texture} (\"${2:Texture}\", 2D) = \"${3:white}\" {}", "Snippet: 2D Texture property"),
    ("texture", "_${1:Texture} (\"${2:Texture}\", 2D) = \"${3:white}\" {}", "Snippet: 2D Texture property"),
    ("normal", "[Normal] _${1:BumpMap} (\"${2:Normal Map}\", 2D) = \"bump\" {}", "Snippet: Normal Map 2D texture property"),
    ("cube", "_${1:Cubemap} (\"${2:Cubemap}\", Cube) = \"\" {}", "Snippet: Reflection Cubemap property"),
    ("int", "_${1:Int} (\"${2:Int}\", Int) = ${3:0}", "Snippet: Integer scalar property"),
    ("toggle", "[Toggle] _${1:Feature} (\"${2:Enable Feature}\", Float) = ${3:0}", "Snippet: Checkbox toggle property"),
    ("keywordenum", "[KeywordEnum(${1:ChoiceA, ChoiceB})] _${2:Mode} (\"${3:Mode}\", Float) = 0", "Snippet: KeywordEnum dropdown property"),
    ("space", "[Space]", "Snippet: Vertical inspector spacing"),
    ("header", "[Header(\"${1:Section Title}\")]", "Snippet: Bold inspector section header"),
];

pub static SHADERLAB_TYPE_COMPLETIONS_AFTER_COMMA: &[(&str, &str, &str)] = &[
    ("Color", "Color) = (1, 1, 1, 1)", "Color property with RGBA default (1, 1, 1, 1)"),
    ("Float", "Float) = 0.0", "Floating-point scalar property with default 0.0"),
    ("Int", "Int) = 0", "Integer scalar property with default 0"),
    ("Range", "Range(${1:0.0}, ${2:1.0})) = ${3:0.5}", "Bounded Range slider property"),
    ("2D", "2D) = \"${1:white}\" {}", "2D texture property with default 'white' tint"),
    ("Vector", "Vector) = (${1:0}, ${2:0}, ${3:0}, ${4:0})", "4D vector property with default (0, 0, 0, 0)"),
    ("Cube", "Cube) = \"\" {}", "Cubemap reflection texture slot"),
    ("3D", "3D) = \"\" {}", "3D volumetric texture slot"),
    ("2DArray", "2DArray) = \"\" {}", "2D texture array slot"),
    ("Rect", "Rect) = \"white\" {}", "Rect texture slot"),
];

pub struct EngineVariable {
    pub name: &'static str,
    pub var_type: &'static str,
    pub engine: &'static str, // "unity", "unreal"
    pub detail: &'static str,
    pub description: &'static str,
}

pub fn find_engine_variable(name: &str) -> Option<&'static EngineVariable> {
    ENGINE_VARIABLES.iter().find(|v| v.name == name)
}

pub static ENGINE_VARIABLES: &[EngineVariable] = &[
    // ========================================================================
    // Unity: Time & DeltaTime Variables
    // ========================================================================
    EngineVariable {
        name: "_Time",
        var_type: "float4",
        engine: "unity",
        detail: "float4 _Time (t/20, t, t*2, t*3)",
        description: "### `_Time`\n*Unity Built-in Shader Variable*\n\nTime since level load in seconds:\n- `x`: `t / 20`\n- `y`: `t` (current time in seconds)\n- `z`: `t * 2`\n- `w`: `t * 3`",
    },
    EngineVariable {
        name: "_SinTime",
        var_type: "float4",
        engine: "unity",
        detail: "float4 _SinTime (sin(t/8), sin(t/4), sin(t/2), sin(t))",
        description: "### `_SinTime`\n*Unity Built-in Shader Variable*\n\nSine of time since level load:\n- `x`: `sin(t / 8)`\n- `y`: `sin(t / 4)`\n- `z`: `sin(t / 2)`\n- `w`: `sin(t)`",
    },
    EngineVariable {
        name: "_CosTime",
        var_type: "float4",
        engine: "unity",
        detail: "float4 _CosTime (cos(t/8), cos(t/4), cos(t/2), cos(t))",
        description: "### `_CosTime`\n*Unity Built-in Shader Variable*\n\nCosine of time since level load:\n- `x`: `cos(t / 8)`\n- `y`: `cos(t / 4)`\n- `z`: `cos(t / 2)`\n- `w`: `cos(t)`",
    },
    EngineVariable {
        name: "unity_DeltaTime",
        var_type: "float4",
        engine: "unity",
        detail: "float4 unity_DeltaTime (dt, 1/dt, smoothDt, 1/smoothDt)",
        description: "### `unity_DeltaTime`\n*Unity Built-in Shader Variable*\n\nDelta time between rendering frames in seconds:\n- `x`: `dt` (frame delta time)\n- `y`: `1.0 / dt`\n- `z`: `smoothDeltaTime`\n- `w`: `1.0 / smoothDeltaTime`",
    },

    // ========================================================================
    // Unity: Camera, Screen & Projection Parameters
    // ========================================================================
    EngineVariable {
        name: "_WorldSpaceCameraPos",
        var_type: "float3",
        engine: "unity",
        detail: "float3 _WorldSpaceCameraPos",
        description: "### `_WorldSpaceCameraPos`\n*Unity Built-in Shader Variable*\n\nCamera position in World Space coordinates (`float3`).",
    },
    EngineVariable {
        name: "_ProjectionParams",
        var_type: "float4",
        engine: "unity",
        detail: "float4 _ProjectionParams (sign, near, far, 1/far)",
        description: "### `_ProjectionParams`\n*Unity Built-in Shader Variable*\n\nCamera projection parameters:\n- `x`: `1.0` (or `-1.0` if flipping projection for render-textures)\n- `y`: Camera near plane\n- `z`: Camera far plane\n- `w`: `1.0 / far`",
    },
    EngineVariable {
        name: "_ScreenParams",
        var_type: "float4",
        engine: "unity",
        detail: "float4 _ScreenParams (width, height, 1+1/width, 1+1/height)",
        description: "### `_ScreenParams`\n*Unity Built-in Shader Variable*\n\nCurrent render target resolution in pixels:\n- `x`: Width in pixels\n- `y`: Height in pixels\n- `z`: `1.0 + 1.0 / width`\n- `w`: `1.0 + 1.0 / height`",
    },
    EngineVariable {
        name: "_ScaledScreenParams",
        var_type: "float4",
        engine: "unity",
        detail: "float4 _ScaledScreenParams (URP)",
        description: "### `_ScaledScreenParams`\n*Unity URP Shader Variable*\n\nRender scale-adjusted screen resolution in pixels for Dynamic Resolution Scaling.",
    },
    EngineVariable {
        name: "_ZBufferParams",
        var_type: "float4",
        engine: "unity",
        detail: "float4 _ZBufferParams",
        description: "### `_ZBufferParams`\n*Unity Built-in Shader Variable*\n\nParameters to linearize non-linear Z buffer depth values to view space distance.",
    },
    EngineVariable {
        name: "unity_OrthoParams",
        var_type: "float4",
        engine: "unity",
        detail: "float4 unity_OrthoParams (width, height, unused, isOrtho)",
        description: "### `unity_OrthoParams`\n*Unity Built-in Shader Variable*\n\nOrthographic projection parameters:\n- `x`: Orthographic width\n- `y`: Orthographic height\n- `w`: `1.0` if orthographic camera, `0.0` if perspective",
    },
    EngineVariable {
        name: "unity_CameraWorldClipPlanes",
        var_type: "float4[6]",
        engine: "unity",
        detail: "float4 unity_CameraWorldClipPlanes[6]",
        description: "### `unity_CameraWorldClipPlanes`\n*Unity Built-in Shader Variable*\n\nWorld space frustum clipping planes (Left, Right, Bottom, Top, Near, Far).",
    },

    // ========================================================================
    // Unity: Transformation Matrices (Object, World, View, Projection)
    // ========================================================================
    EngineVariable {
        name: "unity_ObjectToWorld",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 unity_ObjectToWorld",
        description: "### `unity_ObjectToWorld`\n*Unity Transformation Matrix*\n\nCurrent model matrix. Transforms coordinates from Object Space to World Space.",
    },
    EngineVariable {
        name: "unity_WorldToObject",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 unity_WorldToObject",
        description: "### `unity_WorldToObject`\n*Unity Transformation Matrix*\n\nInverse model matrix. Transforms coordinates from World Space to Object Space.",
    },
    EngineVariable {
        name: "unity_MatrixVP",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 unity_MatrixVP",
        description: "### `unity_MatrixVP`\n*Unity Transformation Matrix*\n\nView-Projection matrix. Transforms coordinates from World Space to Homogeneous Clip Space.",
    },
    EngineVariable {
        name: "unity_MatrixV",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 unity_MatrixV",
        description: "### `unity_MatrixV`\n*Unity Transformation Matrix*\n\nView matrix. Transforms coordinates from World Space to Camera (View) Space.",
    },
    EngineVariable {
        name: "unity_MatrixInvV",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 unity_MatrixInvV",
        description: "### `unity_MatrixInvV`\n*Unity Transformation Matrix*\n\nInverse View matrix. Transforms coordinates from Camera (View) Space to World Space.",
    },
    EngineVariable {
        name: "unity_MatrixP",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 unity_MatrixP",
        description: "### `unity_MatrixP`\n*Unity Transformation Matrix*\n\nProjection matrix. Transforms coordinates from Camera (View) Space to Clip Space.",
    },
    EngineVariable {
        name: "unity_MatrixInvP",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 unity_MatrixInvP",
        description: "### `unity_MatrixInvP`\n*Unity Transformation Matrix*\n\nInverse Projection matrix. Transforms coordinates from Clip Space to Camera (View) Space.",
    },
    EngineVariable {
        name: "UNITY_MATRIX_MVP",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 / macro UNITY_MATRIX_MVP",
        description: "### `UNITY_MATRIX_MVP`\n*Unity Transformation Matrix*\n\nModel-View-Projection matrix. Transforms coordinates directly from Object Space to Clip Space.",
    },
    EngineVariable {
        name: "UNITY_MATRIX_MV",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 UNITY_MATRIX_MV",
        description: "### `UNITY_MATRIX_MV`\n*Unity Transformation Matrix*\n\nModel-View matrix. Transforms coordinates from Object Space to Eye/View Space.",
    },
    EngineVariable {
        name: "UNITY_MATRIX_M",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 UNITY_MATRIX_M",
        description: "### `UNITY_MATRIX_M`\n*Unity Transformation Matrix*\n\nAlias for `unity_ObjectToWorld`.",
    },
    EngineVariable {
        name: "UNITY_MATRIX_V",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 UNITY_MATRIX_V",
        description: "### `UNITY_MATRIX_V`\n*Unity Transformation Matrix*\n\nAlias for `unity_MatrixV`.",
    },
    EngineVariable {
        name: "UNITY_MATRIX_P",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 UNITY_MATRIX_P",
        description: "### `UNITY_MATRIX_P`\n*Unity Transformation Matrix*\n\nAlias for `unity_MatrixP`.",
    },
    EngineVariable {
        name: "UNITY_MATRIX_VP",
        var_type: "float4x4",
        engine: "unity",
        detail: "float4x4 UNITY_MATRIX_VP",
        description: "### `UNITY_MATRIX_VP`\n*Unity Transformation Matrix*\n\nAlias for `unity_MatrixVP`.",
    },

    // ========================================================================
    // Unity: Lighting Variables
    // ========================================================================
    EngineVariable {
        name: "_WorldSpaceLightPos0",
        var_type: "float4",
        engine: "unity",
        detail: "float4 _WorldSpaceLightPos0",
        description: "### `_WorldSpaceLightPos0`\n*Unity Lighting Variable*\n\nPosition or direction of the primary light:\n- `w == 0.0`: Directional light (`xyz` is direction toward light source)\n- `w == 1.0`: Point or Spot light (`xyz` is position in world space)",
    },
    EngineVariable {
        name: "_LightColor0",
        var_type: "half4",
        engine: "unity",
        detail: "half4 _LightColor0",
        description: "### `_LightColor0`\n*Unity Lighting Variable*\n\nColor and intensity of the main light in the current pass.",
    },
    EngineVariable {
        name: "_MainLightPosition",
        var_type: "float4",
        engine: "unity",
        detail: "float4 _MainLightPosition (URP)",
        description: "### `_MainLightPosition`\n*Unity URP Lighting Variable*\n\nWorld space direction or position of the URP Main Directional Light.",
    },
    EngineVariable {
        name: "_MainLightColor",
        var_type: "half4",
        engine: "unity",
        detail: "half4 _MainLightColor (URP)",
        description: "### `_MainLightColor`\n*Unity URP Lighting Variable*\n\nColor and intensity of the URP Main Directional Light.",
    },
    EngineVariable {
        name: "_AdditionalLightsCount",
        var_type: "half4",
        engine: "unity",
        detail: "half4 _AdditionalLightsCount (URP)",
        description: "### `_AdditionalLightsCount`\n*Unity URP Lighting Variable*\n\n`x` channel specifies the count of non-directional additional lights affecting the current object.",
    },

    // ========================================================================
    // Unity: Screen Textures (URP & Built-in)
    // ========================================================================
    EngineVariable {
        name: "_CameraDepthTexture",
        var_type: "Texture2D",
        engine: "unity",
        detail: "Texture2D _CameraDepthTexture",
        description: "### `_CameraDepthTexture`\n*Unity Screen Texture*\n\nScreen-space depth buffer texture rendered by the camera.",
    },
    EngineVariable {
        name: "_CameraNormalsTexture",
        var_type: "Texture2D",
        engine: "unity",
        detail: "Texture2D _CameraNormalsTexture (URP)",
        description: "### `_CameraNormalsTexture`\n*Unity URP Screen Texture*\n\nScreen-space normals texture rendered by the camera during the DepthNormals pass.",
    },
    EngineVariable {
        name: "_CameraOpaqueTexture",
        var_type: "Texture2D",
        engine: "unity",
        detail: "Texture2D _CameraOpaqueTexture (URP)",
        description: "### `_CameraOpaqueTexture`\n*Unity URP Screen Texture*\n\nScreen-space color texture copy captured after rendering all opaque geometry.",
    },

    // ========================================================================
    // Unity: Transform & Helper Functions / Macros
    // ========================================================================
    EngineVariable {
        name: "TRANSFORM_TEX",
        var_type: "macro",
        engine: "unity",
        detail: "TRANSFORM_TEX(tex, name)",
        description: "### `TRANSFORM_TEX(tex, name)`\n*Unity UV Macro*\n\nTransforms UV coordinates using Material Inspector Tiling & Offset (`_ST` vector):\n`((tex.xy) * name##_ST.xy + name##_ST.zw)`",
    },
    EngineVariable {
        name: "UnityObjectToClipPos",
        var_type: "function",
        engine: "unity",
        detail: "float4 UnityObjectToClipPos(float3 pos)",
        description: "### `UnityObjectToClipPos`\n*Unity Built-in Transform Function*\n\nTransforms coordinates from Object Space to Homogeneous Clip Space.",
    },
    EngineVariable {
        name: "TransformObjectToHClip",
        var_type: "function",
        engine: "unity",
        detail: "float4 TransformObjectToHClip(float3 positionOS)",
        description: "### `TransformObjectToHClip`\n*Unity URP Transform Function*\n\nTransforms vertex position from Object Space to Homogeneous Clip Space.",
    },
    EngineVariable {
        name: "TransformObjectToWorld",
        var_type: "function",
        engine: "unity",
        detail: "float3 TransformObjectToWorld(float3 positionOS)",
        description: "### `TransformObjectToWorld`\n*Unity URP Transform Function*\n\nTransforms coordinates from Object Space to World Space.",
    },
    EngineVariable {
        name: "TransformWorldToObject",
        var_type: "function",
        engine: "unity",
        detail: "float3 TransformWorldToObject(float3 positionWS)",
        description: "### `TransformWorldToObject`\n*Unity URP Transform Function*\n\nTransforms coordinates from World Space to Object Space.",
    },
    EngineVariable {
        name: "TransformWorldToHClip",
        var_type: "function",
        engine: "unity",
        detail: "float4 TransformWorldToHClip(float3 positionWS)",
        description: "### `TransformWorldToHClip`\n*Unity URP Transform Function*\n\nTransforms coordinates from World Space to Homogeneous Clip Space.",
    },
    EngineVariable {
        name: "TransformObjectToWorldNormal",
        var_type: "function",
        engine: "unity",
        detail: "float3 TransformObjectToWorldNormal(float3 normalOS)",
        description: "### `TransformObjectToWorldNormal`\n*Unity URP Normal Transform Function*\n\nTransforms a normal vector from Object Space to World Space with proper inverse transpose scaling.",
    },
    EngineVariable {
        name: "TransformObjectToWorldDir",
        var_type: "function",
        engine: "unity",
        detail: "float3 TransformObjectToWorldDir(float3 dirOS)",
        description: "### `TransformObjectToWorldDir`\n*Unity URP Direction Transform Function*\n\nTransforms a direction vector from Object Space to World Space without translation.",
    },
    EngineVariable {
        name: "GetVertexPositionInputs",
        var_type: "function",
        engine: "unity",
        detail: "VertexPositionInputs GetVertexPositionInputs(float3 positionOS)",
        description: "### `GetVertexPositionInputs`\n*Unity URP Helper Function*\n\nCalculates vertex positions simultaneously in Object, World, View, Clip, and NDC spaces.",
    },
    EngineVariable {
        name: "GetVertexNormalInputs",
        var_type: "function",
        engine: "unity",
        detail: "VertexNormalInputs GetVertexNormalInputs(float3 normalOS, float4 tangentOS)",
        description: "### `GetVertexNormalInputs`\n*Unity URP Helper Function*\n\nCalculates world space normal, tangent, and bitangent vectors.",
    },
    EngineVariable {
        name: "GetMainLight",
        var_type: "function",
        engine: "unity",
        detail: "Light GetMainLight()",
        description: "### `GetMainLight`\n*Unity URP Lighting Function*\n\nReturns the `Light` struct for the main directional light in the scene.",
    },
    EngineVariable {
        name: "GetAdditionalLight",
        var_type: "function",
        engine: "unity",
        detail: "Light GetAdditionalLight(uint i, float3 positionWS)",
        description: "### `GetAdditionalLight`\n*Unity URP Lighting Function*\n\nReturns the `Light` struct for the additional light at index `i` affecting `positionWS`.",
    },
    EngineVariable {
        name: "SAMPLE_TEXTURE2D",
        var_type: "macro",
        engine: "unity",
        detail: "SAMPLE_TEXTURE2D(textureName, samplerName, coord2)",
        description: "### `SAMPLE_TEXTURE2D`\n*Unity URP Texture Macro*\n\nSamples a 2D Texture using the specified SamplerState.",
    },
    EngineVariable {
        name: "SAMPLE_TEXTURE2D_LOD",
        var_type: "macro",
        engine: "unity",
        detail: "SAMPLE_TEXTURE2D_LOD(textureName, samplerName, coord2, lod)",
        description: "### `SAMPLE_TEXTURE2D_LOD`\n*Unity URP Texture Macro*\n\nSamples a 2D Texture at an explicit LOD mipmap level.",
    },
    EngineVariable {
        name: "TEXTURE2D",
        var_type: "macro",
        engine: "unity",
        detail: "TEXTURE2D(textureName)",
        description: "### `TEXTURE2D`\n*Unity URP Texture Macro*\n\nCross-platform declaration macro for a 2D texture.",
    },
    EngineVariable {
        name: "SAMPLER",
        var_type: "macro",
        engine: "unity",
        detail: "SAMPLER(samplerName)",
        description: "### `SAMPLER`\n*Unity URP Sampler Macro*\n\nCross-platform declaration macro for a SamplerState.",
    },
    EngineVariable {
        name: "UnityPixelSnap",
        var_type: "function",
        engine: "unity",
        detail: "float4 UnityPixelSnap(float4 pos)",
        description: "### `UnityPixelSnap`\n*Unity 2D Sprite Function*\n\nSnaps vertex position to pixel boundaries for crisp pixel-perfect 2D sprites.",
    },
    EngineVariable {
        name: "UnityGet2DClipping",
        var_type: "function",
        engine: "unity",
        detail: "float UnityGet2DClipping(float2 position, float4 clipRect)",
        description: "### `UnityGet2DClipping`\n*Unity 2D UI Canvas Function*\n\nEvaluates 2D UI rectangular clipping against `clipRect`.",
    },

    // ========================================================================
    // Unreal Engine: View Uniforms & Material Functions
    // ========================================================================
    EngineVariable {
        name: "ResolvedView.WorldCameraOrigin",
        var_type: "float3",
        engine: "unreal",
        detail: "float3 ResolvedView.WorldCameraOrigin",
        description: "### `ResolvedView.WorldCameraOrigin`\n*Unreal Engine View Uniform*\n\nCamera position in World Space coordinates (`float3`).",
    },
    EngineVariable {
        name: "ResolvedView.GameTime",
        var_type: "float",
        engine: "unreal",
        detail: "float ResolvedView.GameTime",
        description: "### `ResolvedView.GameTime`\n*Unreal Engine View Uniform*\n\nCurrent game time in seconds.",
    },
    EngineVariable {
        name: "ResolvedView.RealTime",
        var_type: "float",
        engine: "unreal",
        detail: "float ResolvedView.RealTime",
        description: "### `ResolvedView.RealTime`\n*Unreal Engine View Uniform*\n\nReal wall-clock time in seconds.",
    },
    EngineVariable {
        name: "ResolvedView.DeltaTime",
        var_type: "float",
        engine: "unreal",
        detail: "float ResolvedView.DeltaTime",
        description: "### `ResolvedView.DeltaTime`\n*Unreal Engine View Uniform*\n\nDelta time between rendering frames in seconds.",
    },
    EngineVariable {
        name: "ResolvedView.ViewSizeAndInvSize",
        var_type: "float4",
        engine: "unreal",
        detail: "float4 ResolvedView.ViewSizeAndInvSize (w, h, 1/w, 1/h)",
        description: "### `ResolvedView.ViewSizeAndInvSize`\n*Unreal Engine View Uniform*\n\nViewport dimensions: `xy` is pixel width and height, `zw` is inverse width and height.",
    },
    EngineVariable {
        name: "ResolvedView.WorldToClip",
        var_type: "float4x4",
        engine: "unreal",
        detail: "float4x4 ResolvedView.WorldToClip",
        description: "### `ResolvedView.WorldToClip`\n*Unreal Engine View Uniform*\n\nTransforms coordinates from World Space to Clip Space.",
    },
    EngineVariable {
        name: "ResolvedView.ClipToWorld",
        var_type: "float4x4",
        engine: "unreal",
        detail: "float4x4 ResolvedView.ClipToWorld",
        description: "### `ResolvedView.ClipToWorld`\n*Unreal Engine View Uniform*\n\nTransforms coordinates from Clip Space to World Space.",
    },
    EngineVariable {
        name: "GetWorldPosition",
        var_type: "function",
        engine: "unreal",
        detail: "float3 GetWorldPosition(FMaterialVertexParameters Parameters)",
        description: "### `GetWorldPosition`\n*Unreal Engine Material Function*\n\nReturns the world position of the current vertex or pixel.",
    },
    EngineVariable {
        name: "CalcPixelDepth",
        var_type: "function",
        engine: "unreal",
        detail: "float CalcPixelDepth(FMaterialPixelParameters Parameters)",
        description: "### `CalcPixelDepth`\n*Unreal Engine Material Function*\n\nCalculates camera-to-pixel depth in Unreal Units.",
    },

    // ========================================================================
    // Unity: URP / HDRP Constant Buffer Blocks (SRP Batcher)
    // ========================================================================
    EngineVariable {
        name: "UnityPerMaterial",
        var_type: "cbuffer",
        engine: "unity",
        detail: "CBUFFER_START(UnityPerMaterial)",
        description: "### `UnityPerMaterial`\n*Unity URP/HDRP Constant Buffer*\n\nContains all material properties exposed in the Inspector. Required by the SRP Batcher to persist material data across draw calls and achieve maximum performance.",
    },
    EngineVariable {
        name: "UnityPerDraw",
        var_type: "cbuffer",
        engine: "unity",
        detail: "CBUFFER_START(UnityPerDraw)",
        description: "### `UnityPerDraw`\n*Unity Built-in Constant Buffer*\n\nContains per-object transformation matrices (`unity_ObjectToWorld`, `unity_WorldToObject`) and GPU instancing properties.",
    },
    EngineVariable {
        name: "UnityPerCamera",
        var_type: "cbuffer",
        engine: "unity",
        detail: "CBUFFER_START(UnityPerCamera)",
        description: "### `UnityPerCamera`\n*Unity Built-in Constant Buffer*\n\nContains camera-specific projection, view matrices, and camera world position parameters.",
    },
    EngineVariable {
        name: "UnityPerFrame",
        var_type: "cbuffer",
        engine: "unity",
        detail: "CBUFFER_START(UnityPerFrame)",
        description: "### `UnityPerFrame`\n*Unity Built-in Constant Buffer*\n\nContains global per-frame time parameters (`_Time`, `_SinTime`, `_CosTime`, `unity_DeltaTime`).",
    },
    EngineVariable {
        name: "UnityPerPass",
        var_type: "cbuffer",
        engine: "unity",
        detail: "CBUFFER_START(UnityPerPass)",
        description: "### `UnityPerPass`\n*Unity Built-in Constant Buffer*\n\nContains per-pass parameters such as shadow cascade parameters, light count, and ambient lighting.",
    },
    EngineVariable {
        name: "UnityPerDrawRare",
        var_type: "cbuffer",
        engine: "unity",
        detail: "CBUFFER_START(UnityPerDrawRare)",
        description: "### `UnityPerDrawRare`\n*Unity Built-in Constant Buffer*\n\nContains rarely updated draw call state parameters.",
    },
    EngineVariable {
        name: "CBUFFER_START",
        var_type: "macro",
        engine: "unity",
        detail: "CBUFFER_START(name)",
        description: "### `CBUFFER_START(name)`\n*Unity URP/HDRP Macro*\n\nBegins a constant buffer block declaration. For SRP Batcher compatibility, expose material properties within `CBUFFER_START(UnityPerMaterial) ... CBUFFER_END`.",
    },
    EngineVariable {
        name: "CBUFFER_END",
        var_type: "macro",
        engine: "unity",
        detail: "CBUFFER_END",
        description: "### `CBUFFER_END`\n*Unity URP/HDRP Macro*\n\nCloses a constant buffer block declaration started with `CBUFFER_START(name)`.",
    },
    EngineVariable {
        name: "TEXTURE2D",
        var_type: "macro",
        engine: "unity",
        detail: "TEXTURE2D(textureName)",
        description: "### `TEXTURE2D(textureName)`\n*Unity URP/HDRP Texture Macro*\n\nDeclares a 2D texture object separate from its sampler (e.g. `TEXTURE2D(_BaseMap);`).",
    },
    EngineVariable {
        name: "SAMPLER",
        var_type: "macro",
        engine: "unity",
        detail: "SAMPLER(samplerName)",
        description: "### `SAMPLER(samplerName)`\n*Unity URP/HDRP Sampler Macro*\n\nDeclares a texture sampler state (e.g. `SAMPLER(sampler_BaseMap);`).",
    },
    EngineVariable {
        name: "SAMPLE_TEXTURE2D",
        var_type: "macro",
        engine: "unity",
        detail: "SAMPLE_TEXTURE2D(textureName, samplerName, coord2)",
        description: "### `SAMPLE_TEXTURE2D(texture, sampler, uv)`\n*Unity URP/HDRP Texture Sampling Macro*\n\nSamples a 2D texture using the specified sampler state and UV coordinates.",
    },
];
