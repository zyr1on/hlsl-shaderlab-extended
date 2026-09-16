use super::*;

fn is_dxc_runnable(dxc: &str) -> bool {
    std::process::Command::new(dxc)
        .arg("-help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
    fn test_docs_builtin_functions() {
        let lerp = docs::find_builtin_function("lerp").expect("lerp should exist in docs");
        assert_eq!(lerp.name, "lerp");
        assert!(!lerp.overloads.is_empty());
        assert!(lerp.description.contains("linear interpolation"));

        let saturate = docs::find_builtin_function("saturate").expect("saturate should exist in docs");
        assert_eq!(saturate.name, "saturate");

        let mul = docs::find_builtin_function("mul").expect("mul should exist in docs");
        assert_eq!(mul.name, "mul");

        let unity_transform = docs::find_builtin_function("TransformObjectToHClip").expect("TransformObjectToHClip should exist");
        assert_eq!(unity_transform.name, "TransformObjectToHClip");

        let unreal_norm = docs::find_builtin_function("GetWorldNormal").expect("GetWorldNormal should exist");
        assert_eq!(unreal_norm.name, "GetWorldNormal");
    }

    #[test]
    fn test_signature_help_parsing() {
        let code = "float4 col = lerp(colorA, colorB, ";
        let (fn_name, param_idx) = signature::find_enclosing_call(code, 0, code.len()).expect("Should find call");
        assert_eq!(fn_name, "lerp");
        assert_eq!(param_idx, 2);

        let code_first_param = "float x = saturate(";
        let (fn_name_sat, param_idx_sat) = signature::find_enclosing_call(code_first_param, 0, code_first_param.len()).expect("Should find call");
        assert_eq!(fn_name_sat, "saturate");
        assert_eq!(param_idx_sat, 0);
    }

    #[test]
    fn test_range_signature_help() {
        let cache = HashMap::new();
        let code = "    _Gloss (\"Smoothness\", Range(0, ";
        let sig = signature::get_signature_help("file:///test.shader", code, 0, code.len(), &cache);
        assert!(!sig.is_null());
        assert_eq!(sig["signatures"][0]["label"], "Range(float min, float max)");
        assert_eq!(sig["activeParameter"], 1);
    }

    #[test]
    fn test_hover_info() {
        let cache = HashMap::new();
        let code = "float v = saturate(val);";
        let hover = signature::get_hover_info("file:///test.hlsl", code, 0, 12, &cache);
        assert!(!hover.is_null());
        let val_str = hover["contents"]["value"].as_str().unwrap();
        assert!(val_str.contains("saturate"));
    }

    #[test]
    fn test_user_symbol_scanning() {
        let code = r#"
struct Attributes {
    float3 positionOS : POSITION;
    float2 uv : TEXCOORD0;
};

float4 MyVertShader(Attributes input) : SV_Position {
    return float4(input.positionOS, 1.0);
}
"#;
        let funcs = signature::scan_user_functions(code, None, None);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "MyVertShader");
        assert_eq!(funcs[0].parameters.len(), 1);
        assert_eq!(funcs[0].parsed_params.len(), 1);
        assert_eq!(funcs[0].parsed_params[0].name, "input");
        assert_eq!(funcs[0].parsed_params[0].param_type, "Attributes");

        let structs = signature::scan_struct_definitions(code);
        assert!(structs.iter().any(|s| s.name == "Attributes" && s.fields.len() == 2));
    }

    #[test]
    fn test_function_parameter_completion_and_deduplication() {
        let code = r#"
struct VSInput {
    float3 position : POSITION;
    float4 color : COLOR;
};

struct VSOutput {
    float4 position : SV_Position;
    float4 color : COLOR;
};

VSOutput VSMain(VSInput input)
{
    VSOutput output;

    output.position = float4(inpu, 1.0);
    output.color = input.color;

    return output;
}
"#;
        let mut cache = HashMap::new();
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        // Cursor at line 17: "output.position = float4(inpu, 1.0);" right after 'inpu'
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 17, "character": 33 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result should be array");

        // 1. Parameter "input" MUST be present with top priority!
        let input_item = arr.iter().find(|i| i["label"] == "input");
        assert!(input_item.is_some(), "Function parameter 'input' MUST be in completions!");
        let input_obj = input_item.unwrap();
        assert_eq!(input_obj["kind"], 6);
        assert!(input_obj["sortText"].as_str().unwrap().starts_with("00_"));

        // 2. Local variable "output" MUST be present!
        let output_item = arr.iter().find(|i| i["label"] == "output");
        assert!(output_item.is_some(), "Local variable 'output' MUST be in completions!");
        assert_eq!(output_item.unwrap()["kind"], 6);
        assert!(output_item.unwrap()["sortText"].as_str().unwrap().starts_with("01_"));

        // 3. "VSInput" MUST NOT be duplicated!
        let vsinput_count = arr.iter().filter(|i| i["label"] == "VSInput").count();
        assert_eq!(vsinput_count, 1, "VSInput should appear exactly once, got {}", vsinput_count);
    }

    #[test]
    fn test_parameter_hover_and_definition() {
        let code = r#"
VSOutput VSMain(VSInput input)
{
    VSOutput output;
    output.color = input.color;
    return output;
}
"#;
        let mut cache = HashMap::new();
        cache.insert("file:///test.hlsl".to_string(), code.to_string());

        // Test Hover on "input" (line 4: "    output.color = input.color;")
        let hover_val = signature::get_hover_info("file:///test.hlsl", code, 4, 20, &cache);
        assert!(!hover_val.is_null(), "Hover on parameter 'input' should return markdown!");
        let hover_md = hover_val["contents"]["value"].as_str().unwrap();
        assert!(hover_md.contains("VSInput input"), "Hover should show 'VSInput input'");
        assert!(hover_md.contains("parameter of `VSMain`"), "Hover should mention 'parameter of VSMain'");

        // Test Go to Definition on "input"
        let def_req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 4, "character": 20 }
            }
        });
        let def_val = signature::handle_definition(&def_req, &cache);
        assert!(!def_val.is_null(), "Go to definition on 'input' should not be null!");
        assert_eq!(def_val["range"]["start"]["line"], 1, "Parameter 'input' is declared on line 1");

        // Test Hover on "output" (line 4: "    output.color = input.color;")
        let hover_out = signature::get_hover_info("file:///test.hlsl", code, 4, 6, &cache);
        assert!(!hover_out.is_null(), "Hover on local 'output' should return markdown!");
        let hover_out_md = hover_out["contents"]["value"].as_str().unwrap();
        assert!(hover_out_md.contains("VSOutput output"), "Hover should show 'VSOutput output'");
        assert!(hover_out_md.contains("local variable in `VSMain`"), "Hover should mention local variable");

        // Test Go to Definition on "output"
        let def_out_req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 4, "character": 6 }
            }
        });
        let def_out_val = signature::handle_definition(&def_out_req, &cache);
        assert!(!def_out_val.is_null(), "Go to definition on 'output' should not be null!");
        assert_eq!(def_out_val["range"]["start"]["line"], 3, "Local 'output' is declared on line 3");
    }

    #[test]
    fn test_validate_pure_hlsl_valid() {
        let dxc = find_dxc_path();
        if !is_dxc_runnable(&dxc) {
            eprintln!("Skipping fn test_validate_pure_hlsl_valid(): DXC compiler not available on system");
            return;
        }
        let valid_shader = r#"
float4 MainVs(float3 pos : POSITION) : SV_Position {
    return float4(pos, 1.0);
}
"#;
        let diags = validate_shader("file:///test.hlsl", valid_shader, &dxc, None);
        assert!(diags.is_empty(), "Expected 0 diagnostics for valid pure HLSL, got: {:?}", diags);
    }

    #[test]
    fn test_validate_pure_hlsl_error() {
        let dxc = find_dxc_path();
        if !is_dxc_runnable(&dxc) {
            eprintln!("Skipping fn test_validate_pure_hlsl_error(): DXC compiler not available on system");
            return;
        }
        let invalid_shader = r#"
float4 MainVs(float3 pos : POSITION) : SV_Position {
    return float4(pos, undefined_variable);
}
"#;
        let diags = validate_shader("file:///test.hlsl", invalid_shader, &dxc, None);
        assert!(!diags.is_empty(), "Expected diagnostics for undefined variable in pure HLSL");
        assert!(diags.iter().any(|d| d.message.contains("undefined_variable") || d.severity == 1));
    }

    #[test]
    fn test_validate_unity_unlit_shader() {
        let dxc = find_dxc_path();
        if !is_dxc_runnable(&dxc) {
            eprintln!("Skipping fn test_validate_unity_unlit_shader(): DXC compiler not available on system");
            return;
        }
        let unity_shader = r#"
Shader "Unlit/TestUnlit"
{
    Properties
    {
        _MainTex ("Texture", 2D) = "white" {}
    }
    SubShader
    {
        Tags { "RenderType"="Opaque" }
        Pass
        {
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "UnityCG.cginc"

            struct appdata
            {
                float4 vertex : POSITION;
                float2 uv : TEXCOORD0;
            };

            struct v2f
            {
                float2 uv : TEXCOORD0;
                float4 vertex : SV_POSITION;
            };

            sampler2D _MainTex;
            float4 _MainTex_ST;

            v2f vert (appdata v)
            {
                v2f o;
                o.vertex = UnityObjectToClipPos(v.vertex);
                o.uv = TRANSFORM_TEX(v.uv, _MainTex);
                return o;
            }

            fixed4 frag (v2f i) : SV_Target
            {
                fixed4 col = tex2D(_MainTex, i.uv);
                return col;
            }
            ENDCG
        }
    }
}
"#;
        let diags = validate_shader("file:///test_unlit.shader", unity_shader, &dxc, None);
        assert!(diags.is_empty(), "Standard Unity Unlit shader MUST have 0 errors, got: {:?}", diags);
    }

    #[test]
    fn test_validate_unity_2d_sprite_shader() {
        let dxc = find_dxc_path();
        if !is_dxc_runnable(&dxc) {
            eprintln!("Skipping fn test_validate_unity_2d_sprite_shader(): DXC compiler not available on system");
            return;
        }
        let sprite_shader = r#"
Shader "Sprites/Custom2DSprite"
{
    Properties
    {
        [PerRendererData] _MainTex ("Sprite Texture", 2D) = "white" {}
        _Color ("Tint", Color) = (1,1,1,1)
        [MaterialToggle] PixelSnap ("Pixel snap", Float) = 0
        [HideInInspector] _RendererColor ("RendererColor", Color) = (1,1,1,1)
        [HideInInspector] _Flip ("Flip", Vector) = (1,1,1,1)
    }
    SubShader
    {
        Tags
        {
            "Queue"="Transparent"
            "IgnoreProjector"="True"
            "RenderType"="Transparent"
            "PreviewType"="Plane"
            "CanUseSpriteAtlas"="True"
        }
        Cull Off
        Lighting Off
        ZWrite Off
        Blend One OneMinusSrcAlpha
        Pass
        {
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "UnityCG.cginc"

            struct appdata_t
            {
                float4 vertex   : POSITION;
                float4 color    : COLOR;
                float2 texcoord : TEXCOORD0;
            };

            struct v2f
            {
                float4 vertex   : SV_POSITION;
                fixed4 color    : COLOR;
                float2 texcoord : TEXCOORD0;
            };

            sampler2D _MainTex;
            fixed4 _Color;
            fixed4 _RendererColor;
            float4 _Flip;

            v2f vert(appdata_t IN)
            {
                v2f OUT;
                IN.vertex.xy *= _Flip.xy;
                OUT.vertex = UnityObjectToClipPos(IN.vertex);
                OUT.texcoord = IN.texcoord;
                OUT.color = IN.color * _Color * _RendererColor;
                OUT.vertex = UnityPixelSnap(OUT.vertex);
                return OUT;
            }

            fixed4 frag(v2f IN) : SV_Target
            {
                fixed4 c = tex2D(_MainTex, IN.texcoord) * IN.color;
                c.rgb *= c.a;
                return c;
            }
            ENDCG
        }
    }
}
"#;
        let diags = validate_shader("file:///Assets/Sprites/Custom2DSprite.shader", sprite_shader, &dxc, None);
        assert!(diags.is_empty(), "Unity 2D Sprite shader MUST validate with 0 errors, got: {:?}", diags);
    }

    #[test]
    fn test_validate_unity_2d_ui_shader() {
        let dxc = find_dxc_path();
        if !is_dxc_runnable(&dxc) {
            eprintln!("Skipping fn test_validate_unity_2d_ui_shader(): DXC compiler not available on system");
            return;
        }
        let ui_shader = r#"
Shader "UI/Custom2DUI"
{
    Properties
    {
        [PerRendererData] _MainTex ("Sprite Texture", 2D) = "white" {}
        _Color ("Tint", Color) = (1,1,1,1)
        _ClipRect ("Clip Rect", Vector) = (-32767, -32767, 32767, 32767)
    }
    SubShader
    {
        Tags
        {
            "Queue"="Transparent"
            "IgnoreProjector"="True"
            "RenderType"="Transparent"
            "PreviewType"="Plane"
            "CanUseSpriteAtlas"="True"
        }
        Cull Off
        Lighting Off
        ZWrite Off
        Blend SrcAlpha OneMinusSrcAlpha
        Pass
        {
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "UnityCG.cginc"

            struct appdata_t
            {
                float4 vertex   : POSITION;
                float4 color    : COLOR;
                float2 texcoord : TEXCOORD0;
            };

            struct v2f
            {
                float4 vertex   : SV_POSITION;
                fixed4 color    : COLOR;
                float2 texcoord : TEXCOORD0;
                float4 worldPosition : TEXCOORD1;
            };

            sampler2D _MainTex;
            fixed4 _Color;
            float4 _ClipRect;

            v2f vert(appdata_t v)
            {
                v2f OUT;
                OUT.worldPosition = v.vertex;
                OUT.vertex = UnityObjectToClipPos(OUT.worldPosition);
                OUT.texcoord = v.texcoord;
                OUT.color = v.color * _Color;
                return OUT;
            }

            fixed4 frag(v2f IN) : SV_Target
            {
                half4 color = tex2D(_MainTex, IN.texcoord) * IN.color;
                color.a *= UnityGet2DClipping(IN.worldPosition.xy, _ClipRect);
                return color;
            }
            ENDCG
        }
    }
}
"#;
        let diags = validate_shader("file:///Assets/UI/Custom2DUI.shader", ui_shader, &dxc, None);
        assert!(diags.is_empty(), "Unity 2D UI Canvas shader MUST validate with 0 errors, got: {:?}", diags);
    }

    #[test]
    fn test_validate_unreal_shader() {
        let dxc = find_dxc_path();
        if !is_dxc_runnable(&dxc) {
            eprintln!("Skipping fn test_validate_unreal_shader(): DXC compiler not available on system");
            return;
        }
        let unreal_shader = r#"
float3 CustomUnrealLighting(float3 WorldPos, float3 WorldNormal, float3 LightDir)
{
    float NdotL = saturate(dot(WorldNormal, LightDir));
    float3 lum = Luminance(LightDir);
    return RotateAboutAxis(float4(WorldNormal, 0.5), WorldPos, LightDir) * (NdotL + lum);
}
"#;
        let diags = validate_shader("file:///test_unreal.usf", unreal_shader, &dxc, None);
        assert!(diags.is_empty(), "Unreal shader MUST have 0 errors, got: {:?}", diags);
    }

    #[test]
    fn test_validate_all_sample_files() {
        let dxc = find_dxc_path();
        if !is_dxc_runnable(&dxc) {
            eprintln!("Skipping fn test_validate_all_sample_files(): DXC compiler not available on system");
            return;
        }

        // 1. Pure HLSL Sample
        if let Ok(pure_code) = std::fs::read_to_string("../samples/pure_sample.hlsl") {
            let context = detect_shader_context("file:///pure_sample.hlsl", &pure_code);
            assert_eq!(context, ShaderContext::PureHlsl);
            let diags = validate_shader("file:///pure_sample.hlsl", &pure_code, &dxc, None);
            assert!(diags.is_empty(), "Sample pure HLSL MUST have 0 errors, got: {:?}", diags);
        }

        // 2. Unity Unlit Sample
        if let Ok(unity_code) = std::fs::read_to_string("../samples/unity_unlit.shader") {
            let context = detect_shader_context("file:///unity_unlit.shader", &unity_code);
            assert_eq!(context, ShaderContext::UnityShaderLab);
            let diags = validate_shader("file:///unity_unlit.shader", &unity_code, &dxc, None);
            assert!(diags.is_empty(), "Sample Unity Unlit shader MUST have 0 errors, got: {:?}", diags);
        }

        // 3. Unity 2D Sprite Sample
        if let Ok(sprite_code) = std::fs::read_to_string("../samples/unity_sprite.shader") {
            let context = detect_shader_context("file:///unity_sprite.shader", &sprite_code);
            assert_eq!(context, ShaderContext::UnityShaderLab);
            assert!(is_unity_2d_context("file:///unity_sprite.shader", &sprite_code));
            let diags = validate_shader("file:///unity_sprite.shader", &sprite_code, &dxc, None);
            assert!(diags.is_empty(), "Sample Unity 2D Sprite shader MUST have 0 errors, got: {:?}", diags);
        }

        // 4. Unreal Engine USF Sample
        if let Ok(unreal_code) = std::fs::read_to_string("../samples/unreal_sample.usf") {
            let context = detect_shader_context("file:///unreal_sample.usf", &unreal_code);
            assert_eq!(context, ShaderContext::UnrealEngine);
            let diags = validate_shader("file:///unreal_sample.usf", &unreal_code, &dxc, None);
            assert!(diags.is_empty(), "Sample Unreal shader MUST have 0 errors, got: {:?}", diags);
        }
    }

    #[test]
    fn test_validate_unity_urp_shader() {
        let dxc = find_dxc_path();
        if !is_dxc_runnable(&dxc) {
            eprintln!("Skipping fn test_validate_unity_urp_shader(): DXC compiler not available on system");
            return;
        }
        let urp_shader = r#"
Shader "Universal/TestUnlit"
{
    Properties
    {
        [MainTexture] _BaseMap("Texture", 2D) = "white" {}
        [MainColor] _BaseColor("Color", Color) = (1, 1, 1, 1)
    }
    SubShader
    {
        Tags { "RenderType"="Opaque" "RenderPipeline"="UniversalPipeline" }
        Pass
        {
            HLSLPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Core.hlsl"

            struct Attributes
            {
                float4 positionOS   : POSITION;
                float2 uv           : TEXCOORD0;
            };

            struct Varyings
            {
                float4 positionHCS  : SV_POSITION;
                float2 uv           : TEXCOORD0;
            };

            TEXTURE2D(_BaseMap);
            SAMPLER(sampler_BaseMap);

            CBUFFER_START(UnityPerMaterial)
                half4 _BaseColor;
                float4 _BaseMap_ST;
            CBUFFER_END

            Varyings vert(Attributes IN)
            {
                Varyings OUT;
                OUT.positionHCS = TransformObjectToHClip(IN.positionOS.xyz);
                OUT.uv = TRANSFORM_TEX(IN.uv, _BaseMap);
                return OUT;
            }

            half4 frag(Varyings IN) : SV_Target
            {
                half4 color = SAMPLE_TEXTURE2D(_BaseMap, sampler_BaseMap, IN.uv) * _BaseColor;
                return color;
            }
            ENDHLSL
        }
    }
}
"#;
        let diags = validate_shader("file:///test_urp.shader", urp_shader, &dxc, None);
        assert!(diags.is_empty(), "Valid URP shader MUST have 0 errors, got: {:?}", diags);

        let broken_urp = urp_shader.replace("Varyings OUT;", "Var");
        let broken_diags = validate_shader("file:///broken_urp.shader", &broken_urp, &dxc, None);
        assert!(!broken_diags.is_empty(), "Broken URP shader MUST report syntax error");
        assert!(broken_diags.iter().any(|d| d.message.contains("Var")), "Diagnostics must flag 'Var': {:?}", broken_diags);
    }

    #[test]
    fn test_context_filter_completion() {
        let mut cache = HashMap::new();
        // 1. Pure HLSL - should NOT have Unity TransformObjectToHClip or Unreal RotateAboutAxis
        cache.insert("file:///pure.hlsl".to_string(), "float4 frag() : SV_Target {\n    \n}".to_string());
        let req_pure = json!({
            "params": {
                "textDocument": { "uri": "file:///pure.hlsl" },
                "position": { "line": 1, "character": 4 }
            }
        });
        let res_pure = handle_completion(&req_pure, &cache);
        let arr_pure = res_pure.as_array().unwrap();
        assert!(arr_pure.iter().any(|i| i["label"] == "lerp"));
        assert!(!arr_pure.iter().any(|i| i["label"] == "TransformObjectToHClip"));
        assert!(!arr_pure.iter().any(|i| i["label"] == "RotateAboutAxis"));

        // 2. Unreal - should have Unreal functions but NOT Unity functions
        cache.insert("file:///unreal.usf".to_string(), "float3 frag() { ".to_string());
        let req_unreal = json!({
            "params": {
                "textDocument": { "uri": "file:///unreal.usf" },
                "position": { "line": 0, "character": 16 }
            }
        });
        let res_unreal = handle_completion(&req_unreal, &cache);
        let arr_unreal = res_unreal.as_array().unwrap();
        assert!(arr_unreal.iter().any(|i| i["label"] == "RotateAboutAxis"));
        assert!(!arr_unreal.iter().any(|i| i["label"] == "TransformObjectToHClip"));

        // 3. Unity HLSL - should have Unity functions but NOT Unreal functions
        cache.insert("file:///unity.cginc".to_string(), "float4 frag() { ".to_string());
        let req_unity = json!({
            "params": {
                "textDocument": { "uri": "file:///unity.cginc" },
                "position": { "line": 0, "character": 16 }
            }
        });
        let res_unity = handle_completion(&req_unity, &cache);
        let arr_unity = res_unity.as_array().unwrap();
        assert!(arr_unity.iter().any(|i| i["label"] == "TransformObjectToHClip"));
        assert!(!arr_unity.iter().any(|i| i["label"] == "RotateAboutAxis"));
    }

    #[test]
    fn test_shaderlab_properties_completion() {
        let mut cache = HashMap::new();
        let code = "Shader \"Test\" {\nProperties {\n    _\n}\n}";
        cache.insert("file:///test.shader".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.shader" },
                "position": { "line": 2, "character": 5 }
            }
        });
        let res = handle_completion(&req, &cache);
        let arr = res.as_array().unwrap();
        assert!(arr.iter().any(|i| i["label"] == "_MainTex"));
        assert!(arr.iter().any(|i| i["label"] == "_Color"));
        assert!(arr.iter().any(|i| i["label"] == "Range"));
    }

    #[test]
    fn test_member_access_completion() {
        let mut cache = HashMap::new();
        let code = r#"
struct Varyings {
    float4 position : SV_Position;
    float4 color : COLOR;
};

float4 frag(Varyings output) : SV_Target {
    output.po
}
"#;
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 7, "character": 13 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "position"));
        assert!(arr.iter().any(|item| item["label"] == "color"));
    }

    #[test]
    fn test_texture_method_completion() {
        let mut cache = HashMap::new();
        let code = r#"
Texture2D _MainTex;
SamplerState sampler_MainTex;

float4 frag() : SV_Target {
    _MainTex.
}
"#;
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 5, "character": 13 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "Sample"));
        assert!(arr.iter().any(|item| item["label"] == "SampleLevel"));
        assert!(arr.iter().any(|item| item["label"] == "Load"));
    }

    #[test]
    fn test_semantic_completion() {
        let mut cache = HashMap::new();
        let code = "float4 pos : SV_";
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 0, "character": 16 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "SV_Position"));
        assert!(arr.iter().any(|item| item["label"] == "SV_Target"));
    }

    #[test]
    fn test_document_symbols() {
        let code = r#"
Shader "Custom/MyShader"
{
    SubShader
    {
        Pass
        {
            struct Attributes {
                float3 pos : POSITION;
            };

            float4 vert(Attributes input) : SV_Position {
                return float4(input.pos, 1.0);
            }
        }
    }
}
"#;
        let symbols = signature::get_document_symbols(code);
        let arr = symbols.as_array().expect("Symbols should be array");
        assert!(arr.iter().any(|s| s["name"] == "Custom/MyShader"));
        assert!(arr.iter().any(|s| s["name"] == "SubShader"));
        assert!(arr.iter().any(|s| s["name"] == "Pass"));
        assert!(arr.iter().any(|s| s["name"] == "Attributes"));
        assert!(arr.iter().any(|s| s["name"] == "vert"));
    }

    #[test]
    fn test_format_document() {
        let messy = "Shader \"Test\" {\nSubShader {\nPass {\nHLSLPROGRAM\nfloat4 frag() : SV_Target {\nreturn 1;\n}\nENDHLSL\n}\n}\n}";
        let edits = format_document(messy, 4, true);
        assert!(!edits.is_empty());
        let new_text = edits[0]["newText"].as_str().unwrap();
        assert!(new_text.contains("    SubShader"));
        assert!(new_text.contains("        Pass"));
    }

    #[test]
    fn test_matrix_element_completion() {
        let mut cache = HashMap::new();
        let code = r#"
void TestMatrix()
{
    float4x4 mvp;
    float val = mvp.
}
"#;
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 4, "character": 20 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "_m00"), "Matrix should offer _m00");
        assert!(arr.iter().any(|item| item["label"] == "_m33"), "Matrix should offer _m33");
        assert!(arr.iter().any(|item| item["label"] == "_11"), "Matrix should offer _11");
        assert!(arr.iter().any(|item| item["label"] == "_44"), "Matrix should offer _44");
    }

    #[test]
    fn test_vector_swizzle_dimension() {
        let mut cache = HashMap::new();
        let code = r#"
void TestVec()
{
    float2 uv;
    float val = uv.
}
"#;
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 4, "character": 19 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "x"), "float2 should have x");
        assert!(arr.iter().any(|item| item["label"] == "y"), "float2 should have y");
        assert!(arr.iter().any(|item| item["label"] == "xy"), "float2 should have xy");
        assert!(!arr.iter().any(|item| item["label"] == "z"), "float2 MUST NOT have z");
        assert!(!arr.iter().any(|item| item["label"] == "w"), "float2 MUST NOT have w");
        assert!(!arr.iter().any(|item| item["label"] == "rgba"), "float2 MUST NOT have rgba");
    }

    #[test]
    fn test_extended_semantics() {
        let mut cache = HashMap::new();
        let code = "float4 col : SV_";
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 0, "character": 16 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "SV_Target7"), "Should have SV_Target7");
        assert!(arr.iter().any(|item| item["label"] == "SV_DepthGreaterEqual"), "Should have SV_DepthGreaterEqual");
        assert!(arr.iter().any(|item| item["label"] == "SV_IsFrontFace"), "Should have SV_IsFrontFace");
        assert!(arr.iter().any(|item| item["label"] == "SV_Barycentrics"), "Should have SV_Barycentrics");
        assert!(arr.iter().any(|item| item["label"] == "BINORMAL0"), "Should have BINORMAL0");
    }

    #[test]
    fn test_shader_snippets() {
        let mut cache = HashMap::new();
        let code = "\n";
        cache.insert("file:///test.hlsl".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.hlsl" },
                "position": { "line": 0, "character": 0 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "kernel"), "Should have kernel snippet");
        assert!(arr.iter().any(|item| item["label"] == "tex2d"), "Should have tex2d snippet");
        assert!(arr.iter().any(|item| item["label"] == "cbuffer"), "Should have cbuffer snippet");
        assert!(arr.iter().any(|item| item["label"] == "for"), "Should have for snippet");
    }

    #[test]
    fn test_shaderlab_nested_structs_completion() {
        let code = r#"Shader "Test/URPUnlit"
{
    SubShader
    {
        Pass
        {
            HLSLPROGRAM
            struct Attributes
            {
                float4 positionOS : POSITION;
            };

            struct Varyings
            {
                float4 positionCS : SV_POSITION;
            };

            Vary
            ENDHLSL
        }
    }
}"#;
        let structs = signature::scan_struct_definitions(code);
        assert_eq!(structs.len(), 2, "Should discover both Attributes and Varyings");
        assert_eq!(structs[0].name, "Attributes");
        assert_eq!(structs[1].name, "Varyings");

        let mut cache = HashMap::new();
        cache.insert("file:///test.shader".to_string(), code.to_string());
        let req = json!({
            "params": {
                "textDocument": { "uri": "file:///test.shader" },
                "position": { "line": 17, "character": 16 }
            }
        });
        let result = handle_completion(&req, &cache);
        let arr = result.as_array().expect("Result must be array");
        assert!(arr.iter().any(|item| item["label"] == "Varyings"), "Completion should contain Varyings");
        assert!(arr.iter().any(|item| item["label"] == "Attributes"), "Completion should contain Attributes");
    }

    #[test]
    fn test_shaderlab_property_syntax_error_and_cbuffer_warning() {
        // Test malformed property without identifier
        let code_with_syntax_err = r#"Shader "Test/Error"
{
    Properties
    {
        ("Float", Float) = 0.0
    }
    SubShader
    {
        Pass
        {
            HLSLPROGRAM
            ENDHLSL
        }
    }
}"#;
        let diags = validate_shaderlab_properties_and_cbuffer(code_with_syntax_err);
        assert!(diags.iter().any(|d| d.severity == 1 && d.message.contains("Missing property name")),
            "Should report syntax error for property missing name");

        // Test missing CBuffer warning
        let code_missing_cbuffer = r#"Shader "Test/URP"
{
    Properties
    {
        _BaseColor("Base Color", Color) = (1, 1, 1, 1)
        _Speed("Speed", Float) = 1.0
    }
    SubShader
    {
        Tags { "RenderPipeline" = "UniversalPipeline" }
        Pass
        {
            HLSLPROGRAM
            // No CBUFFER_START(UnityPerMaterial)
            ENDHLSL
        }
    }
}"#;
        let diags2 = validate_shaderlab_properties_and_cbuffer(code_missing_cbuffer);
        assert!(diags2.iter().any(|d| d.severity == 2 && d.message.contains("UnityPerMaterial")),
            "Should warn about missing CBUFFER_START(UnityPerMaterial)");

        // Test property inside CBuffer -> no warning
        let code_with_cbuffer = r#"Shader "Test/URP"
{
    Properties
    {
        _BaseColor("Base Color", Color) = (1, 1, 1, 1)
    }
    SubShader
    {
        Tags { "RenderPipeline" = "UniversalPipeline" }
        Pass
        {
            HLSLPROGRAM
            CBUFFER_START(UnityPerMaterial)
                half4 _BaseColor;
            CBUFFER_END
            ENDHLSL
        }
    }
}"#;
        let diags3 = validate_shaderlab_properties_and_cbuffer(code_with_cbuffer);
        assert!(!diags3.iter().any(|d| d.message.contains("not declared in 'CBUFFER_START(UnityPerMaterial)'")),
            "Should not warn when property is declared in CBuffer");
    }

    #[test]
    fn test_auto_context_detection() {
        // Pure HLSL: standard HLSL without Unity or Unreal indicators
        let pure_hlsl = "Texture2D tex : register(t0);\nfloat4 main() : SV_Target { return float4(1, 0, 0, 1); }";
        assert_eq!(detect_shader_context("file:///C:/projects/game/render.hlsl", pure_hlsl), ShaderContext::PureHlsl);

        // Unity ShaderLab: .shader extension
        let sl_code = "Shader \"Custom/MyShader\" { SubShader { Pass {} } }";
        assert_eq!(detect_shader_context("file:///C:/projects/MyShader.shader", sl_code), ShaderContext::UnityShaderLab);

        // Unity HLSL: detected by _Time in content
        let unity_hlsl_by_var = "float4 main() : SV_Target { return _Time; }";
        assert_eq!(detect_shader_context("file:///C:/projects/render.hlsl", unity_hlsl_by_var), ShaderContext::UnityHlsl);

        // Unity HLSL: detected by URP transform function
        let unity_hlsl_by_fn = "float4 main(float3 pos : POSITION) : SV_Position { return TransformObjectToHClip(pos); }";
        assert_eq!(detect_shader_context("file:///C:/projects/vert.hlsl", unity_hlsl_by_fn), ShaderContext::UnityHlsl);

        // Unreal Engine: detected by .ush extension or ResolvedView
        let ue_code = "float4 main() : SV_Target { return ResolvedView.WorldCameraOrigin.xyzz; }";
        assert_eq!(detect_shader_context("file:///C:/UnrealProjects/MyGame/Shaders/Private/Test.usf", ue_code), ShaderContext::UnrealEngine);
        assert_eq!(detect_shader_context("file:///C:/somewhere/test.hlsl", ue_code), ShaderContext::UnrealEngine);
    }

    #[test]
    fn test_unity_builtins_completion_and_filtering() {
        let mut cache = HashMap::new();

        // 1. Unity context: _Time, unity_ObjectToWorld, etc. MUST be present
        let unity_code = "float4 main() : SV_Target\n{\n    _Ti\n    return float4(0, 0, 0, 0);\n}";
        cache.insert("file:///C:/UnityProject/Assets/Shaders/test.hlsl".to_string(), unity_code.to_string());
        let req_unity = json!({
            "params": {
                "textDocument": { "uri": "file:///C:/UnityProject/Assets/Shaders/test.hlsl" },
                "position": { "line": 2, "character": 7 }
            }
        });
        let res_unity = handle_completion(&req_unity, &cache);
        let items_unity = res_unity.as_array().expect("Must be array");
        assert!(items_unity.iter().any(|i| i["label"] == "_Time"), "Unity completion MUST include _Time");
        assert!(items_unity.iter().any(|i| i["label"] == "_SinTime"), "Unity completion MUST include _SinTime");
        assert!(items_unity.iter().any(|i| i["label"] == "unity_ObjectToWorld"), "Unity completion MUST include unity_ObjectToWorld");
        assert!(items_unity.iter().any(|i| i["label"] == "TransformObjectToHClip"), "Unity completion MUST include TransformObjectToHClip");

        // 2. Pure HLSL context: _Time, unity_ObjectToWorld MUST NOT be present
        let pure_code = "float4 main() : SV_Target\n{\n    \n    return float4(0, 0, 0, 0);\n}";
        cache.insert("file:///C:/PureHLSL/shader.hlsl".to_string(), pure_code.to_string());
        let req_pure = json!({
            "params": {
                "textDocument": { "uri": "file:///C:/PureHLSL/shader.hlsl" },
                "position": { "line": 2, "character": 4 }
            }
        });
        let res_pure = handle_completion(&req_pure, &cache);
        let items_pure = res_pure.as_array().expect("Must be array");
        assert!(!items_pure.iter().any(|i| i["label"] == "_Time"), "Pure HLSL completion MUST NOT include _Time");
        assert!(!items_pure.iter().any(|i| i["label"] == "unity_ObjectToWorld"), "Pure HLSL completion MUST NOT include unity_ObjectToWorld");
        assert!(!items_pure.iter().any(|i| i["label"] == "ResolvedView.WorldCameraOrigin"), "Pure HLSL completion MUST NOT include ResolvedView");
    }

    #[test]
    fn test_engine_variable_hover() {
        let mut cache = HashMap::new();
        let code = "float4 t = _Time;\nfloat4x4 m = unity_ObjectToWorld;\n";
        cache.insert("file:///test.hlsl".to_string(), code.to_string());

        let hover_time = signature::get_hover_info("file:///test.hlsl", code, 0, 13, &cache);
        let hover_time_str = hover_time["contents"]["value"].as_str().unwrap_or("");
        assert!(hover_time_str.contains("_Time"), "Hover should describe _Time");
        assert!(hover_time_str.contains("t / 20"), "Hover should explain _Time components");

        let hover_mat = signature::get_hover_info("file:///test.hlsl", code, 1, 16, &cache);
        let hover_mat_str = hover_mat["contents"]["value"].as_str().unwrap_or("");
        assert!(hover_mat_str.contains("unity_ObjectToWorld"), "Hover should describe unity_ObjectToWorld");
    }

    #[test]
    fn test_utf8_char_boundary_no_panic() {
        let mut cache = HashMap::new();
        // Line with Turkish characters and emojis
        let code = "// Türkçe açıklama: değişkenler 🚀 ve fonksiyonlar\nfloat4 renk = float4(1, 0, 0, 1);\n";
        cache.insert("file:///test_utf8.hlsl".to_string(), code.to_string());

        // Test hover across various column offsets on the UTF-8 line
        for col in 0..code.lines().next().unwrap().len() {
            let h = signature::get_hover_info("file:///test_utf8.hlsl", code, 0, col, &cache);
            let _ = h;
        }

        // Test definition across various column offsets
        for col in 0..code.lines().next().unwrap().len() {
            let def_req = json!({
                "params": {
                    "textDocument": { "uri": "file:///test_utf8.hlsl" },
                    "position": { "line": 0, "character": col }
                }
            });
            let d = signature::handle_definition(&def_req, &cache);
            let _ = d;
        }

        // Test completion across column offsets
        for col in 0..10 {
            let req = json!({
                "params": {
                    "textDocument": { "uri": "file:///test_utf8.hlsl" },
                    "position": { "line": 0, "character": col }
                }
            });
            let _ = handle_completion(&req, &cache);
        }
    }

    #[test]
    fn test_forward_declaration_struct_scanning() {
        let code = r#"
struct ForwardType;
struct TargetStruct {
    float3 pos;
    float2 uv;
};
"#;
        let structs = signature::scan_struct_definitions(code);
        assert!(structs.iter().any(|s| s.name == "TargetStruct"), "TargetStruct must be found even with preceding forward declaration");
        assert!(!structs.iter().any(|s| s.name == "ForwardType"), "Forward declaration must not be recorded as a complete struct");
    }

    #[test]
    fn test_local_variables_not_in_global_symbols() {
        let code = r#"
float4 g_GlobalColor;

float4 FragmentFunction(float4 inColor)
{
    float4 localColor = inColor * 2.0;
    return localColor;
}
"#;
        let vars = signature::scan_user_variables(code, None, None);
        assert!(vars.iter().any(|v| v.name == "g_GlobalColor"), "g_GlobalColor must be in global variables");
        assert!(!vars.iter().any(|v| v.name == "localColor"), "localColor must NOT be in global variables");
    }

    #[test]
    fn test_register_and_packoffset_variables() {
        let code = r#"
Texture2D _MainTex : register(t0);
SamplerState _Sampler : register(s0);
float4 _Color : packoffset(c0);
"#;
        let vars = signature::scan_user_variables(code, None, None);
        assert!(vars.iter().any(|v| v.name == "_MainTex"), "_MainTex with register binding must be detected");
        assert!(vars.iter().any(|v| v.name == "_Sampler"), "_Sampler with register binding must be detected");
        assert!(vars.iter().any(|v| v.name == "_Color"), "_Color with packoffset binding must be detected");
    }

    #[test]
    fn test_multi_attribute_and_comma_in_shaderlab_properties() {
        let code = r#"
Shader "Test/MultiAttr"
{
    Properties
    {
        [HideInInspector] [HDR] _Color ("Main, Secondary (RGB)", Color) = (1, 1, 1, 1)
        _Smoothness ("Smoothness (0 to 1)", Range(0, 1)) = 0.5
    }
    SubShader { Pass {} }
}
"#;
        let props = signature::scan_shaderlab_properties(code);
        assert_eq!(props.len(), 2);
        assert_eq!(props[0].name, "_Color");
        assert_eq!(props[0].display_name, "Main, Secondary (RGB)");
        assert_eq!(props[0].prop_type, "Color");
        assert_eq!(props[1].name, "_Smoothness");
        assert_eq!(props[1].prop_type, "Range(0, 1)");

        // Also test validation does not emit false syntax error
        let diags = validate_shaderlab_properties_and_cbuffer(code);
        assert!(!diags.iter().any(|d| d.message.contains("Invalid property name")), "Multi-attributes should not trigger invalid name error");
    }

    #[test]
    fn test_dxr_raytracing_docs() {
        assert!(docs::find_builtin_function("TraceRay").is_some(), "TraceRay must be in docs");
        assert!(docs::find_builtin_function("WorldRayOrigin").is_some(), "WorldRayOrigin must be in docs");
        assert!(docs::find_builtin_function("RayTCurrent").is_some(), "RayTCurrent must be in docs");
    }

    #[test]
    fn test_shaderlab_pass_render_states() {
        let code = "Shader \"Test/RenderStates\"\n{\n    SubShader\n    {\n        Z\n    }\n}";
        let mut doc_cache = HashMap::new();
        let uri = "file:///test.shader";
        doc_cache.insert(uri.to_string(), code.to_string());

        // Test typing "Z" in SubShader (line 4, character 9)
        let msg_z = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 4, "character": 9 }
            }
        });
        let completions_z = handle_completion(&msg_z, &doc_cache);
        let items_z = completions_z.as_array().expect("Should return completion array");
        assert!(items_z.iter().any(|item| item["label"].as_str().unwrap().starts_with("ZWrite")), "Should suggest ZWrite on Z");
        assert!(items_z.iter().any(|item| item["label"].as_str().unwrap().starts_with("ZTest")), "Should suggest ZTest on Z");
        assert!(items_z.iter().any(|item| item["label"].as_str().unwrap().starts_with("ZClip")), "Should suggest ZClip on Z");

        // Test typing "ZCull" alias
        let code_zcull = "Shader \"Test\" { SubShader { ZCull } }";
        doc_cache.insert(uri.to_string(), code_zcull.to_string());
        let msg_zcull = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 0, "character": 33 }
            }
        });
        let completions_zcull = handle_completion(&msg_zcull, &doc_cache);
        let items_zcull = completions_zcull.as_array().expect("Should return completion array");
        assert!(items_zcull.iter().any(|item| item["label"].as_str().unwrap().contains("Cull")), "ZCull should suggest Cull");

        // Test typing "BlendOp "
        let code_blendop = "Shader \"Test\" { SubShader { BlendOp  } }";
        doc_cache.insert(uri.to_string(), code_blendop.to_string());
        let msg_blendop = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 0, "character": 36 }
            }
        });
        let completions_blendop = handle_completion(&msg_blendop, &doc_cache);
        let items_blendop = completions_blendop.as_array().expect("Should return completion array");
        assert!(items_blendop.iter().any(|item| item["label"] == "BlendOp Add"), "Should suggest BlendOp Add");
        assert!(items_blendop.iter().any(|item| item["label"] == "BlendOp Min"), "Should suggest BlendOp Min");

        // Test typing "ZClip "
        let code_zclip = "Shader \"Test\" { SubShader { ZClip  } }";
        doc_cache.insert(uri.to_string(), code_zclip.to_string());
        let msg_zclip = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 0, "character": 34 }
            }
        });
        let completions_zclip = handle_completion(&msg_zclip, &doc_cache);
        let items_zclip = completions_zclip.as_array().expect("Should return completion array");
        assert!(items_zclip.iter().any(|item| item["label"] == "ZClip False"), "Should suggest ZClip False");
    }

    #[test]
    fn test_shaderlab_user_image_diagnostics() {
        let code = r#"Shader "Custom/Test"
{
    Properties
    {
        [MainColor] _BaseColor("Base Color", Color) = (1, 1, 1, 1)
        [MainTexture] _BaseMap("Base Map", 2D) = "white" {}
        _Freq ("freq", Float) = 
        _Amp ("Amp", Float) = 0.0
        _Speed ("Speed", Float) = 0.0
    }
    SubShader
    {
        Tags { "RenderType" = "Opaque" "RenderPipeline" = "UniversalPipeline" }
    }
}"#;
        let diags = validate_shaderlab_properties_and_cbuffer(code);
        let freq_err = diags.iter().find(|d| d.message.contains("Missing default value after '=' for Float property '_Freq'"));
        assert!(freq_err.is_some(), "Must report missing default value for _Freq: {:?}", diags);
    }

    #[test]
    fn test_shaderlab_malformed_vector_and_attribute_diagnostic() {
        let code = r#"Shader "Custom/Test2"
{
    Properties
    {
        [MainColor] _BaseColor("Base Color", Color) = (1, 1, 1,)
        [UnclosedBracket _BadProp("Bad", Float) = 1.0
    }
    SubShader { Pass {} }
}"#;
        let diags = validate_shaderlab_properties_and_cbuffer(code);
        assert!(diags.iter().any(|d| d.message.contains("Trailing comma or empty component")), "Must detect trailing comma in (1, 1, 1,): {:?}", diags);
        assert!(diags.iter().any(|d| d.message.contains("Unclosed attribute bracket")), "Must detect unclosed '[': {:?}", diags);
    }

    #[test]
    fn test_shaderlab_tag_validation_and_completion() {
        let code_typo = r#"Shader "Test" { SubShader { Tags { "RenderType" = "Opque" } Pass {} } }"#;
        let tag_diags = validate_shaderlab_tags(code_typo);
        assert!(tag_diags.iter().any(|d| d.message.contains("Unknown RenderType 'Opque'. Did you mean 'Opaque'?")), "Must warn on Opque typo: {:?}", tag_diags);

        // Test completion inside Tags after "RenderType" = 
        let mut doc_cache = HashMap::new();
        let uri = "file:///test.shader";
        let doc = "Shader \"Test\" { SubShader { Tags { \"RenderType\" =  } } }";
        doc_cache.insert(uri.to_string(), doc.to_string());
        let msg = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 0, "character": 50 }
            }
        });
        let res = handle_completion(&msg, &doc_cache);
        let items = res.as_array().expect("Should return completions");
        assert!(items.iter().any(|item| item["label"] == "\"Opaque\""), "Should suggest \"Opaque\" for RenderType");
        assert!(items.iter().any(|item| item["label"] == "\"Transparent\""), "Should suggest \"Transparent\" for RenderType");
    }

    #[test]
    fn test_shaderlab_property_default_value_and_attribute_completion() {
        let mut doc_cache = HashMap::new();
        let uri = "file:///test.shader";

        // 1. Completion after "=" for Color
        let doc_color = "Shader \"T\" {\n    Properties {\n        _BaseColor (\"Color\", Color) = \n    }\n}";
        doc_cache.insert(uri.to_string(), doc_color.to_string());
        let msg_color = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 2, "character": 39 }
            }
        });
        let res_color = handle_completion(&msg_color, &doc_cache);
        let items_color = res_color.as_array().expect("Should return color completions");
        assert!(items_color.iter().any(|item| item["label"] == "(1, 1, 1, 1)"), "Should suggest (1, 1, 1, 1) for Color");

        // 2. Completion after "=" for 2D Texture
        let doc_tex = "Shader \"T\" {\n    Properties {\n        _BaseMap (\"Map\", 2D) = \n    }\n}";
        doc_cache.insert(uri.to_string(), doc_tex.to_string());
        let msg_tex = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 2, "character": 32 }
            }
        });
        let res_tex = handle_completion(&msg_tex, &doc_cache);
        let items_tex = res_tex.as_array().expect("Should return texture completions");
        assert!(items_tex.iter().any(|item| item["label"] == "\"white\" {}"), "Should suggest \"white\" {{}} for 2D");
        assert!(items_tex.iter().any(|item| item["label"] == "\"bump\" {}"), "Should suggest \"bump\" {{}} for 2D");

        // 3. Completion for attributes typing "["
        let doc_attr = "Shader \"T\" {\n    Properties {\n        [\n    }\n}";
        doc_cache.insert(uri.to_string(), doc_attr.to_string());
        let msg_attr = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 2, "character": 9 }
            }
        });
        let res_attr = handle_completion(&msg_attr, &doc_cache);
        let items_attr = res_attr.as_array().expect("Should return attribute completions");
        assert!(items_attr.iter().any(|item| item["label"] == "[MainColor]"), "Should suggest [MainColor]");
        assert!(items_attr.iter().any(|item| item["label"] == "[MainTexture]"), "Should suggest [MainTexture]");
        assert!(items_attr.iter().any(|item| item["label"] == "[HDR]"), "Should suggest [HDR]");
    }

    #[test]
    fn test_shaderlab_extended_properties_validation_and_completion() {
        let code = r#"Shader "Test/Extended"
{
    Properties
    {
        [Normal] _Color ("Color", Color) = (1, 1, 1, 1)
        _Dup ("Dup 1", Float) = 0.0
        _Dup ("Dup 2", Float) = 1.0
        _InvalidType ("Bad", Texture) = "white" {}
        _InvertedRange ("Range", Range(2.0, 0.0)) = 1.0
        _OutOfRange ("Range", Range(0.0, 1.0)) = 5.0
        _ScalarWithVector ("Float", Float) = (1, 1, 1, 1)
        _Semi ("WithSemi", Float) = 0.0;
    }
    SubShader { Pass {} }
}"#;
        let diags = validate_shaderlab_properties_and_cbuffer(code);

        // 1. Incompatible attribute: Normal on Color
        assert!(diags.iter().any(|d| d.message.contains("[Normal] attribute can only be applied to 2D texture")),
            "Must detect [Normal] on Color: {:?}", diags);

        // 2. Duplicate property name
        assert!(diags.iter().any(|d| d.message.contains("Duplicate property name '_Dup'")),
            "Must detect duplicate property: {:?}", diags);

        // 3. Unknown property type with suggestion
        assert!(diags.iter().any(|d| d.message.contains("Unknown property type 'Texture'") && d.message.contains("Did you mean '2D'?")),
            "Must detect unknown type Texture with suggestion: {:?}", diags);

        // 4. Inverted Range bounds
        assert!(diags.iter().any(|d| d.message.contains("Invalid Range limits") && d.message.contains("min (2) is greater than max (0)")),
            "Must detect inverted Range bounds: {:?}", diags);

        // 5. Default value out of Range
        assert!(diags.iter().any(|d| d.message.contains("Default value '5.0' for Range property '_OutOfRange' is outside the range [0, 1]")),
            "Must detect out-of-range value: {:?}", diags);

        // 6. Type mismatch: Float taking vector literal
        assert!(diags.iter().any(|d| d.message.contains("Type mismatch: scalar property '_ScalarWithVector' of type Float cannot take vector default value")),
            "Must detect scalar with vector default: {:?}", diags);

        // 7. Trailing semicolon warning
        assert!(diags.iter().any(|d| d.message.contains("ShaderLab property declarations do not use trailing semicolons ';'. Remove ';'")),
            "Must warn on trailing semicolon: {:?}", diags);

        // Test completion after comma: `_Test ("Display Name", `
        let mut doc_cache = HashMap::new();
        let uri = "file:///test_comma.shader";
        let doc_comma = "Shader \"T\" {\n    Properties {\n        _Test (\"Display Name\", \n    }\n}";
        doc_cache.insert(uri.to_string(), doc_comma.to_string());
        let msg_comma = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 2, "character": 32 }
            }
        });
        let res_comma = handle_completion(&msg_comma, &doc_cache);
        let items_comma = res_comma.as_array().expect("Must return items after comma");
        assert!(items_comma.iter().any(|i| i["label"] == "Color"), "Must suggest Color after comma");
        assert!(items_comma.iter().any(|i| i["label"] == "2D"), "Must suggest 2D after comma");
        assert!(items_comma.iter().any(|i| i["label"] == "Range"), "Must suggest Range after comma");
        assert!(items_comma.iter().any(|i| i["label"] == "Float"), "Must suggest Float after comma");

        // Test keyword snippet completion inside Properties:
        let doc_empty = "Shader \"T\" {\n    Properties {\n        \n    }\n}";
        doc_cache.insert(uri.to_string(), doc_empty.to_string());
        let msg_empty = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 2, "character": 8 }
            }
        });
        let res_empty = handle_completion(&msg_empty, &doc_cache);
        let items_empty = res_empty.as_array().expect("Must return items on empty line");
        assert!(items_empty.iter().any(|i| i["label"] == "float"), "Must suggest 'float' snippet");
        assert!(items_empty.iter().any(|i| i["label"] == "range"), "Must suggest 'range' snippet");
        assert!(items_empty.iter().any(|i| i["label"] == "color"), "Must suggest 'color' snippet");
        assert!(items_empty.iter().any(|i| i["label"] == "normal"), "Must suggest 'normal' snippet");
        assert!(items_empty.iter().any(|i| i["label"] == "toggle"), "Must suggest 'toggle' snippet");
    }

    #[test]
    fn test_shaderlab_render_states_and_generalized_tags() {
        let bad_shader = r#"
Shader "Custom/BadStates" {
    Properties {
        _Color ("Color", Color) = (1, 1, 1, 1)
    }
    SubShader {
        Tags { "RenderType" = "opaque" "RenderPipeline" = "Universal" "IgnoreProjector" = "true" }
        Cull Backface
        ZWrite True
        ZTest Alwayss
        ColorMask XYZ
        Pass {
            HLSLPROGRAM
            float4 frag() : SV_Target { return 0; }
            ENDHLSL
        }
    }
}
"#;
        let diags = validate_shader("file:///bad.shader", bad_shader, "dxc", None);

        // 1. Tag casing warning: "opaque" -> "Opaque"
        assert!(diags.iter().any(|d| d.message.contains("Unknown RenderType 'opaque'") && d.message.contains("Opaque")),
            "Must suggest Opaque: {:?}", diags);

        // 2. Tag pipeline suggestion: "Universal" -> "UniversalPipeline"
        assert!(diags.iter().any(|d| d.message.contains("UniversalPipeline")),
            "Must suggest UniversalPipeline: {:?}", diags);

        // 3. Tag boolean capitalization: "true" -> "True"
        assert!(diags.iter().any(|d| d.message.contains("Unknown IgnoreProjector 'true'") && d.message.contains("Did you mean 'True'?")),
            "Must warn on lowercase true: {:?}", diags);

        // 4. Render state Cull
        assert!(diags.iter().any(|d| d.message.contains("Unknown Cull mode 'Backface'")),
            "Must catch invalid Cull mode: {:?}", diags);

        // 5. Render state ZWrite
        assert!(diags.iter().any(|d| d.message.contains("Invalid ZWrite value 'True'")),
            "Must catch ZWrite True: {:?}", diags);

        // 6. Render state ZTest
        assert!(diags.iter().any(|d| d.message.contains("Unknown ZTest comparison mode 'Alwayss'")),
            "Must catch ZTest Alwayss: {:?}", diags);

        // 7. Render state ColorMask
        assert!(diags.iter().any(|d| d.message.contains("Invalid ColorMask 'XYZ'")),
            "Must catch invalid ColorMask: {:?}", diags);
    }

    #[test]
    fn test_compute_shader_attributes_and_multiline_params() {
        let code = r#"
[numthreads(64, 1, 1)]
void CSMain(
    uint3 id : SV_DispatchThreadID,
    uint groupIndex : SV_GroupIndex
) {
    // Body
}
"#;
        let funcs = signature::scan_user_functions(code, None, None);
        assert_eq!(funcs.len(), 1, "Must parse function with [numthreads] attribute");
        assert_eq!(funcs[0].name, "CSMain");
        assert_eq!(funcs[0].parsed_params.len(), 2);
        assert_eq!(funcs[0].parsed_params[0].name, "id");
        assert_eq!(funcs[0].parsed_params[0].line, 3);
        assert_eq!(funcs[0].parsed_params[1].name, "groupIndex");
        assert_eq!(funcs[0].parsed_params[1].line, 4);
    }

    #[test]
    fn test_dot_access_fallback_and_ternary_prevention() {
        let mut doc_cache = HashMap::new();
        let uri = "file:///test_dot.hlsl";
        let doc = r#"
struct SurfaceData {
    float4 albedo;
    float3 normal;
};

float4 TestFunc(float x) {
    SurfaceData surf;
    unknownVar.
    float val = x > 0.0 ? 1.0 : 
    return surf.albedo;
}
"#;
        doc_cache.insert(uri.to_string(), doc.to_string());

        // 1. unknownVar. completion should return fallback swizzles and known struct fields
        let msg_dot = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 8, "character": 15 } // line 8: after unknownVar.
            }
        });
        let res_dot = handle_completion(&msg_dot, &doc_cache);
        let items_dot = res_dot.as_array().expect("Must return fallback items on unknown dot access");
        assert!(items_dot.iter().any(|i| i["label"] == "x"), "Must suggest swizzle x");
        assert!(items_dot.iter().any(|i| i["label"] == "albedo"), "Must suggest known struct field 'albedo'");
        assert!(items_dot.iter().any(|i| i["label"] == "normal"), "Must suggest known struct field 'normal'");
        assert!(!items_dot.iter().any(|i| i["label"] == "cbuffer"), "Must NOT suggest cbuffer on dot access");

        // 2. Ternary operator branch 'val = x > 0.0 ? 1.0 : ' should NOT suggest SV_Target
        let msg_ternary = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 9, "character": 32 } // line 9: after ':'
            }
        });
        let res_ternary = handle_completion(&msg_ternary, &doc_cache);
        let items_ternary = res_ternary.as_array().expect("Must return completions");
        assert!(!items_ternary.iter().any(|i| i["label"] == "SV_Target"), "Ternary ':' must NOT trigger SV_Target");
    }

    #[test]
    fn test_hover_local_precedence_over_intrinsics() {
        let mut doc_cache = HashMap::new();
        let uri = "file:///test_hover.hlsl";
        let doc = r#"
float4 Calculate(float distance, float saturate) {
    return float4(distance, saturate, 0, 1);
}
"#;
        doc_cache.insert(uri.to_string(), doc.to_string());

        // Hover on parameter 'distance' (line 1, col 25) or inside body (line 2, col 19)
        let hover_res = signature::get_hover_info(uri, doc, 2, 19, &doc_cache);
        let val_str = hover_res["contents"]["value"].as_str().unwrap_or("");
        assert!(val_str.contains("parameter of `Calculate`"), "Hover must show parameter doc: {}", val_str);
        assert!(!val_str.contains("Microsoft HLSL Intrinsic"), "Hover must NOT show intrinsic doc for local parameter");
    }

    #[test]
    fn test_user_signature_help() {
        let mut doc_cache = HashMap::new();
        let uri = "file:///test_sig.shader";
        let doc = r#"Shader "Custom/Test"
{
    SubShader
    {
        Pass
        {
            HLSLPROGRAM
            int test(int a){
                return a;
            }

            void frag() {
                int b = test()
            }
            ENDHLSL
        }
    }
}
"#;
        doc_cache.insert(uri.to_string(), doc.to_string());

        let line_12 = doc.lines().nth(12).unwrap();
        let open_col = line_12.find('(').unwrap() + 1; // between ( and )
        let after_col = line_12.find(')').unwrap() + 1; // after )
        
        let sig_inside = signature::get_signature_help(uri, doc, 12, open_col, &doc_cache);
        let sig_after = signature::get_signature_help(uri, doc, 12, after_col, &doc_cache);
        assert!(!sig_inside.is_null(), "Signature help must work inside ()!");
        assert!(!sig_after.is_null(), "Signature help must work right after ()!");

        assert_eq!(sig_inside["signatures"][0]["label"], "int test(int a)");
        assert_eq!(sig_after["signatures"][0]["label"], "int test(int a)");
    }

    #[test]
    fn test_completion_items_have_parameter_hints_command() {
        let mut doc_cache = HashMap::new();
        let uri = "file:///test_cmd.hlsl";
        let doc = r#"
int my_custom_func(int x, int y) { return x + y; }
void main() {
    
}
"#;
        doc_cache.insert(uri.to_string(), doc.to_string());

        let msg = json!({
            "params": {
                "textDocument": { "uri": uri },
                "position": { "line": 3, "character": 4 }
            }
        });
        let res = handle_completion(&msg, &doc_cache);
        let items = res.as_array().expect("Completion items array");

        // 1. User function item has command
        let user_fn = items.iter().find(|i| i["label"] == "my_custom_func").expect("my_custom_func must be present");
        assert_eq!(user_fn["command"]["command"], "editor.action.triggerParameterHints");

        // 2. Builtin function item has command
        let clamp_fn = items.iter().find(|i| i["label"] == "clamp").expect("clamp must be present");
        assert_eq!(clamp_fn["command"]["command"], "editor.action.triggerParameterHints");
    }

    #[test]
    fn test_included_signature_help_and_hybrid_collision() {
        let mut doc_cache = HashMap::new();
        let main_uri = "file:///workspace/Shaders/Main.shader";
        let inc_uri = "file:///workspace/Shaders/UnityCG.cginc";

        let inc_doc = r#"
// Transforms 2D UV by texture scale and offset
float2 TRANSFORM_TEX(float2 uv, float4 st) {
    return uv * st.xy + st.zw;
}

float CustomHelper(float a, float b, float c) {
    return (a + b) * c;
}
"#;
        let main_doc = r#"Shader "Custom/Main"
{
    SubShader
    {
        Pass
        {
            HLSLPROGRAM
            #include "UnityCG.cginc"

            void frag() {
                float2 uv = TRANSFORM_TEX(
                float val = CustomHelper(
            }
            ENDHLSL
        }
    }
}
"#;
        doc_cache.insert(inc_uri.to_string(), inc_doc.to_string());
        doc_cache.insert(main_uri.to_string(), main_doc.to_string());

        // 1. Signature help for CustomHelper (line 11, col 41)
        let sig_custom = signature::get_signature_help(main_uri, main_doc, 11, 41, &doc_cache);
        assert!(!sig_custom.is_null(), "CustomHelper signature help must resolve from include!");
        let label_custom = sig_custom["signatures"][0]["label"].as_str().unwrap_or("");
        assert!(label_custom.contains("CustomHelper(float a, float b, float c)"));
        let doc_custom = sig_custom["signatures"][0]["documentation"]["value"].as_str().unwrap_or("");
        assert!(doc_custom.contains("*(Defined in `UnityCG.cginc`)*"), "Must show source provenance");

        // 2. Signature help for TRANSFORM_TEX (collision with docs.rs):
        // Must use header's actual signature AND hybrid enriched markdown doc!
        let sig_trans = signature::get_signature_help(main_uri, main_doc, 10, 42, &doc_cache);
        assert!(!sig_trans.is_null(), "TRANSFORM_TEX signature help must resolve from include!");
        let label_trans = sig_trans["signatures"][0]["label"].as_str().unwrap_or("");
        assert!(label_trans.contains("TRANSFORM_TEX(float2 uv, float4 st)"));
        let doc_trans = sig_trans["signatures"][0]["documentation"]["value"].as_str().unwrap_or("");
        assert!(doc_trans.contains("*(Defined in `UnityCG.cginc`)*"), "Must show source provenance");
        assert!(doc_trans.contains("Transforms 2D UV"), "Must show enriched documentation");
    }
