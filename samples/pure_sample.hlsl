// Pure HLSL Shader Sample
// Context: PureHlsl (No Unity, No Unreal, No ShaderLab pollution)

cbuffer CameraBuffer : register(b0)
{
    float4x4 ViewProjection;
    float3 CameraPosition;
    float Time;
};

cbuffer LightBuffer : register(b1)
{
    float3 LightDirection;
    float4 LightColor;
};

struct VSInput
{
    float3 position : POSITION;
    float3 normal   : NORMAL;
    float2 uv       : TEXCOORD0;
};

struct PSInput
{
    float4 position : SV_Position;
    float3 normal   : NORMAL;
    float2 uv       : TEXCOORD0;
};

PSInput VSMain(VSInput input)
{
    PSInput output;
    output.position = mul(ViewProjection, float4(input.position, 1.0));
    output.normal = input.normal;
    output.uv = input.uv;
    return output;
}

Texture2D DiffuseTexture : register(t0);
SamplerState SamplerLinear : register(s0);

float4 PSMain(PSInput input) : SV_Target
{
    float4 texColor = DiffuseTexture.Sample(SamplerLinear, input.uv);
    float NdotL = saturate(dot(input.normal, -LightDirection));
    float3 finalRgb = texColor.rgb * LightColor.rgb * (NdotL + 0.1);
    return float4(finalRgb, texColor.a);
}
