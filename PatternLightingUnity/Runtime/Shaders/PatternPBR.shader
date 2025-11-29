// Pattern Lighting System for Unity 6
// PBR Material with Pattern Support

Shader "Pattern Lighting/PBR"
{
    Properties
    {
        [Header(Base)]
        _BaseColor ("Base Color", Color) = (1, 1, 1, 1)
        _BaseMap ("Albedo", 2D) = "white" {}
        
        [Header(PBR)]
        _Metallic ("Metallic", Range(0, 1)) = 0
        _MetallicMap ("Metallic Map", 2D) = "white" {}
        _Smoothness ("Smoothness", Range(0, 1)) = 0.5
        _SmoothnessMap ("Smoothness Map", 2D) = "white" {}
        
        [Header(Normal)]
        [Normal] _BumpMap ("Normal Map", 2D) = "bump" {}
        _BumpScale ("Normal Scale", Range(0, 2)) = 1
        
        [Header(Occlusion)]
        _OcclusionMap ("Occlusion Map", 2D) = "white" {}
        _OcclusionStrength ("Occlusion Strength", Range(0, 1)) = 1
        
        [Header(Emission)]
        [HDR] _EmissionColor ("Emission Color", Color) = (0, 0, 0, 0)
        _EmissionMap ("Emission Map", 2D) = "white" {}
        
        [Header(Pattern Emission)]
        [Toggle] _UsePatternEmission ("Use Pattern Emission", Float) = 0
        _PatternType ("Pattern Type", Range(0, 11)) = 1
        _PatternSpeed ("Pattern Speed", Range(0.1, 10)) = 1
        _PatternMinEmission ("Min Emission", Range(0, 1)) = 0
        _PatternMaxEmission ("Max Emission", Range(0, 10)) = 1
        
        [Header(Detail)]
        _DetailAlbedoMap ("Detail Albedo", 2D) = "white" {}
        _DetailNormalMap ("Detail Normal", 2D) = "bump" {}
        _DetailScale ("Detail Scale", Range(0.1, 10)) = 1
        
        [Header(Render)]
        [Enum(UnityEngine.Rendering.CullMode)] _Cull ("Cull", Float) = 2
        [Toggle] _AlphaClip ("Alpha Clip", Float) = 0
        _Cutoff ("Alpha Cutoff", Range(0, 1)) = 0.5
    }
    
    SubShader
    {
        Tags 
        { 
            "RenderType" = "Opaque" 
            "Queue" = "Geometry"
            "RenderPipeline" = "UniversalPipeline"
        }
        
        Pass
        {
            Name "ForwardLit"
            Tags { "LightMode" = "UniversalForward" }
            
            Cull [_Cull]
            
            HLSLPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            
            #pragma multi_compile _ _MAIN_LIGHT_SHADOWS
            #pragma multi_compile _ _MAIN_LIGHT_SHADOWS_CASCADE
            #pragma multi_compile _ _ADDITIONAL_LIGHTS_VERTEX _ADDITIONAL_LIGHTS
            #pragma multi_compile _ _ADDITIONAL_LIGHT_SHADOWS
            #pragma multi_compile _ _SHADOWS_SOFT
            #pragma multi_compile_fog
            
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Core.hlsl"
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Lighting.hlsl"
            
            struct Attributes
            {
                float4 positionOS : POSITION;
                float2 uv : TEXCOORD0;
                float3 normalOS : NORMAL;
                float4 tangentOS : TANGENT;
            };
            
            struct Varyings
            {
                float4 positionCS : SV_POSITION;
                float2 uv : TEXCOORD0;
                float3 positionWS : TEXCOORD1;
                float3 normalWS : TEXCOORD2;
                float3 tangentWS : TEXCOORD3;
                float3 bitangentWS : TEXCOORD4;
                float fogFactor : TEXCOORD5;
            };
            
            TEXTURE2D(_BaseMap);
            SAMPLER(sampler_BaseMap);
            TEXTURE2D(_MetallicMap);
            SAMPLER(sampler_MetallicMap);
            TEXTURE2D(_SmoothnessMap);
            SAMPLER(sampler_SmoothnessMap);
            TEXTURE2D(_BumpMap);
            SAMPLER(sampler_BumpMap);
            TEXTURE2D(_OcclusionMap);
            SAMPLER(sampler_OcclusionMap);
            TEXTURE2D(_EmissionMap);
            SAMPLER(sampler_EmissionMap);
            
            CBUFFER_START(UnityPerMaterial)
                float4 _BaseColor;
                float4 _BaseMap_ST;
                float _Metallic;
                float _Smoothness;
                float _BumpScale;
                float _OcclusionStrength;
                float4 _EmissionColor;
                float _UsePatternEmission;
                float _PatternType;
                float _PatternSpeed;
                float _PatternMinEmission;
                float _PatternMaxEmission;
                float _DetailScale;
                float _Cutoff;
            CBUFFER_END
            
            // Pattern evaluation function
            float EvaluatePattern(float patternType, float time, float3 worldPos)
            {
                float value = 1.0;
                
                if (patternType < 0.5) // Steady
                    value = 1.0;
                else if (patternType < 1.5) // Pulse
                    value = 0.5 + 0.5 * sin(time * 6.28318);
                else if (patternType < 2.5) // Flicker
                    value = 0.7 + 0.3 * sin(time * 20.0) * sin(time * 7.3);
                else if (patternType < 3.5) // Strobe
                    value = sin(time * 10.0) > 0.0 ? 1.0 : 0.0;
                else if (patternType < 4.5) // Candle
                    value = 0.8 + 0.2 * sin(time * 12.0) * sin(time * 5.7) * sin(time * 3.1);
                else if (patternType < 5.5) // Fluorescent
                {
                    float startup = saturate(fmod(time, 5.0) / 2.0);
                    float buzz = 0.05 * sin(time * 120.0);
                    value = startup + buzz * startup;
                }
                else if (patternType < 6.5) // Lightning
                    value = pow(max(0.0, sin(time * 0.5)), 20.0);
                else if (patternType < 7.5) // Fire
                    value = 0.7 + 0.3 * sin(time * 8.0) * sin(time * 4.3) * sin(time * 2.1);
                else if (patternType < 8.5) // Alarm
                    value = sin(time * 4.0) > 0.0 ? 1.0 : 0.2;
                else if (patternType < 9.5) // Underwater
                {
                    float caustic = sin(worldPos.x * 0.01 + time) * sin(worldPos.z * 0.01 + time * 0.7);
                    value = 0.7 + 0.3 * caustic;
                }
                else if (patternType < 10.5) // Heartbeat
                {
                    float beat = pow(sin(time * 2.5), 12.0);
                    float beat2 = pow(sin(time * 2.5 + 0.3), 12.0) * 0.5;
                    value = max(beat, beat2);
                }
                else // Breathing
                    value = 0.3 + 0.7 * (sin(time * 0.5) * 0.5 + 0.5);
                
                return saturate(value);
            }
            
            Varyings vert(Attributes input)
            {
                Varyings output;
                
                VertexPositionInputs posInputs = GetVertexPositionInputs(input.positionOS.xyz);
                VertexNormalInputs normalInputs = GetVertexNormalInputs(input.normalOS, input.tangentOS);
                
                output.positionCS = posInputs.positionCS;
                output.positionWS = posInputs.positionWS;
                output.normalWS = normalInputs.normalWS;
                output.tangentWS = normalInputs.tangentWS;
                output.bitangentWS = normalInputs.bitangentWS;
                output.uv = TRANSFORM_TEX(input.uv, _BaseMap);
                output.fogFactor = ComputeFogFactor(output.positionCS.z);
                
                return output;
            }
            
            half4 frag(Varyings input) : SV_Target
            {
                // Sample textures
                half4 albedo = SAMPLE_TEXTURE2D(_BaseMap, sampler_BaseMap, input.uv) * _BaseColor;
                half metallic = SAMPLE_TEXTURE2D(_MetallicMap, sampler_MetallicMap, input.uv).r * _Metallic;
                half smoothness = SAMPLE_TEXTURE2D(_SmoothnessMap, sampler_SmoothnessMap, input.uv).r * _Smoothness;
                half occlusion = lerp(1, SAMPLE_TEXTURE2D(_OcclusionMap, sampler_OcclusionMap, input.uv).r, _OcclusionStrength);
                
                // Normal mapping
                half3 normalTS = UnpackNormalScale(SAMPLE_TEXTURE2D(_BumpMap, sampler_BumpMap, input.uv), _BumpScale);
                float3x3 TBN = float3x3(input.tangentWS, input.bitangentWS, input.normalWS);
                half3 normalWS = normalize(mul(normalTS, TBN));
                
                // Emission
                half3 emission = SAMPLE_TEXTURE2D(_EmissionMap, sampler_EmissionMap, input.uv).rgb * _EmissionColor.rgb;
                
                // Pattern emission
                if (_UsePatternEmission > 0.5)
                {
                    float time = _Time.y * _PatternSpeed;
                    float patternValue = EvaluatePattern(_PatternType, time, input.positionWS);
                    patternValue = lerp(_PatternMinEmission, _PatternMaxEmission, patternValue);
                    emission *= patternValue;
                }
                
                // Setup surface data
                InputData inputData;
                inputData.positionWS = input.positionWS;
                inputData.normalWS = normalWS;
                inputData.viewDirectionWS = GetWorldSpaceNormalizeViewDir(input.positionWS);
                inputData.shadowCoord = TransformWorldToShadowCoord(input.positionWS);
                inputData.fogCoord = input.fogFactor;
                inputData.vertexLighting = half3(0, 0, 0);
                inputData.bakedGI = SampleSH(normalWS);
                inputData.normalizedScreenSpaceUV = GetNormalizedScreenSpaceUV(input.positionCS);
                inputData.shadowMask = half4(1, 1, 1, 1);
                
                SurfaceData surfaceData;
                surfaceData.albedo = albedo.rgb;
                surfaceData.metallic = metallic;
                surfaceData.specular = half3(0, 0, 0);
                surfaceData.smoothness = smoothness;
                surfaceData.normalTS = normalTS;
                surfaceData.emission = emission;
                surfaceData.occlusion = occlusion;
                surfaceData.alpha = albedo.a;
                surfaceData.clearCoatMask = 0;
                surfaceData.clearCoatSmoothness = 0;
                
                // Calculate lighting
                half4 color = UniversalFragmentPBR(inputData, surfaceData);
                
                // Apply fog
                color.rgb = MixFog(color.rgb, input.fogFactor);
                
                return color;
            }
            ENDHLSL
        }
        
        // Shadow caster pass
        Pass
        {
            Name "ShadowCaster"
            Tags { "LightMode" = "ShadowCaster" }
            
            ZWrite On
            ZTest LEqual
            ColorMask 0
            Cull [_Cull]
            
            HLSLPROGRAM
            #pragma vertex ShadowPassVertex
            #pragma fragment ShadowPassFragment
            
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Core.hlsl"
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Lighting.hlsl"
            #include "Packages/com.unity.render-pipelines.universal/Shaders/ShadowCasterPass.hlsl"
            ENDHLSL
        }
        
        // Depth pass
        Pass
        {
            Name "DepthOnly"
            Tags { "LightMode" = "DepthOnly" }
            
            ZWrite On
            ColorMask 0
            Cull [_Cull]
            
            HLSLPROGRAM
            #pragma vertex DepthOnlyVertex
            #pragma fragment DepthOnlyFragment
            
            #include "Packages/com.unity.render-pipelines.universal/Shaders/DepthOnlyPass.hlsl"
            ENDHLSL
        }
    }
    
    FallBack "Universal Render Pipeline/Lit"
    CustomEditor "PatternLighting.Editor.PatternPBRShaderGUI"
}
