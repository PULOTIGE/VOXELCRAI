// Pattern Lighting System for Unity 6
// Emissive Material with Pattern Animation

Shader "Pattern Lighting/Emissive"
{
    Properties
    {
        [Header(Base)]
        _BaseColor ("Base Color", Color) = (0, 0, 0, 1)
        _BaseMap ("Base Map", 2D) = "black" {}
        
        [Header(Emission)]
        [HDR] _EmissionColor ("Emission Color", Color) = (1, 0.5, 0, 1)
        _EmissionMap ("Emission Mask", 2D) = "white" {}
        
        [Header(Pattern)]
        _PatternType ("Pattern Type", Range(0, 11)) = 1
        _PatternSpeed ("Speed", Range(0.1, 10)) = 1
        _PatternMinIntensity ("Min Intensity", Range(0, 1)) = 0.1
        _PatternMaxIntensity ("Max Intensity", Range(0, 20)) = 5
        
        [Header(Color Shift)]
        [Toggle] _UseColorShift ("Enable Color Shift", Float) = 0
        [HDR] _ColorA ("Color A", Color) = (1, 0, 0, 1)
        [HDR] _ColorB ("Color B", Color) = (0, 0, 1, 1)
        _ColorShiftSpeed ("Color Shift Speed", Range(0.1, 5)) = 1
        
        [Header(Pulse Glow)]
        [Toggle] _UsePulseGlow ("Enable Pulse Glow", Float) = 0
        _GlowRadius ("Glow Radius", Range(0, 0.5)) = 0.1
        _GlowSoftness ("Glow Softness", Range(0, 1)) = 0.5
        
        [Header(Render)]
        [Enum(UnityEngine.Rendering.BlendMode)] _SrcBlend ("Src Blend", Float) = 1
        [Enum(UnityEngine.Rendering.BlendMode)] _DstBlend ("Dst Blend", Float) = 0
        [Enum(Off, 0, On, 1)] _ZWrite ("Z Write", Float) = 1
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
            Name "Emissive"
            Tags { "LightMode" = "UniversalForward" }
            
            Blend [_SrcBlend] [_DstBlend]
            ZWrite [_ZWrite]
            
            HLSLPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            #pragma multi_compile_fog
            
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Core.hlsl"
            
            struct Attributes
            {
                float4 positionOS : POSITION;
                float2 uv : TEXCOORD0;
            };
            
            struct Varyings
            {
                float4 positionCS : SV_POSITION;
                float2 uv : TEXCOORD0;
                float3 positionWS : TEXCOORD1;
                float fogFactor : TEXCOORD2;
            };
            
            TEXTURE2D(_BaseMap);
            SAMPLER(sampler_BaseMap);
            TEXTURE2D(_EmissionMap);
            SAMPLER(sampler_EmissionMap);
            
            CBUFFER_START(UnityPerMaterial)
                float4 _BaseColor;
                float4 _BaseMap_ST;
                float4 _EmissionColor;
                float _PatternType;
                float _PatternSpeed;
                float _PatternMinIntensity;
                float _PatternMaxIntensity;
                float _UseColorShift;
                float4 _ColorA;
                float4 _ColorB;
                float _ColorShiftSpeed;
                float _UsePulseGlow;
                float _GlowRadius;
                float _GlowSoftness;
            CBUFFER_END
            
            float EvaluatePattern(float patternType, float time, float3 worldPos)
            {
                float value = 1.0;
                
                if (patternType < 0.5) value = 1.0;
                else if (patternType < 1.5) value = 0.5 + 0.5 * sin(time * 6.28318);
                else if (patternType < 2.5) value = 0.7 + 0.3 * sin(time * 20.0) * sin(time * 7.3);
                else if (patternType < 3.5) value = sin(time * 10.0) > 0.0 ? 1.0 : 0.0;
                else if (patternType < 4.5) value = 0.8 + 0.2 * sin(time * 12.0) * sin(time * 5.7) * sin(time * 3.1);
                else if (patternType < 5.5)
                {
                    float startup = saturate(fmod(time, 5.0) / 2.0);
                    float buzz = 0.05 * sin(time * 120.0);
                    value = startup + buzz * startup;
                }
                else if (patternType < 6.5) value = pow(max(0.0, sin(time * 0.5)), 20.0);
                else if (patternType < 7.5) value = 0.7 + 0.3 * sin(time * 8.0) * sin(time * 4.3) * sin(time * 2.1);
                else if (patternType < 8.5) value = sin(time * 4.0) > 0.0 ? 1.0 : 0.2;
                else if (patternType < 9.5)
                {
                    float caustic = sin(worldPos.x * 0.01 + time) * sin(worldPos.z * 0.01 + time * 0.7);
                    value = 0.7 + 0.3 * caustic;
                }
                else if (patternType < 10.5)
                {
                    float beat = pow(sin(time * 2.5), 12.0);
                    float beat2 = pow(sin(time * 2.5 + 0.3), 12.0) * 0.5;
                    value = max(beat, beat2);
                }
                else value = 0.3 + 0.7 * (sin(time * 0.5) * 0.5 + 0.5);
                
                return saturate(value);
            }
            
            Varyings vert(Attributes input)
            {
                Varyings output;
                
                VertexPositionInputs posInputs = GetVertexPositionInputs(input.positionOS.xyz);
                output.positionCS = posInputs.positionCS;
                output.positionWS = posInputs.positionWS;
                output.uv = TRANSFORM_TEX(input.uv, _BaseMap);
                output.fogFactor = ComputeFogFactor(output.positionCS.z);
                
                return output;
            }
            
            half4 frag(Varyings input) : SV_Target
            {
                // Base color
                half4 base = SAMPLE_TEXTURE2D(_BaseMap, sampler_BaseMap, input.uv) * _BaseColor;
                half emissionMask = SAMPLE_TEXTURE2D(_EmissionMap, sampler_EmissionMap, input.uv).r;
                
                // Calculate pattern
                float time = _Time.y * _PatternSpeed;
                float patternValue = EvaluatePattern(_PatternType, time, input.positionWS);
                patternValue = lerp(_PatternMinIntensity, _PatternMaxIntensity, patternValue);
                
                // Emission color
                half3 emissionColor = _EmissionColor.rgb;
                
                // Color shift
                if (_UseColorShift > 0.5)
                {
                    float colorT = sin(time * _ColorShiftSpeed) * 0.5 + 0.5;
                    emissionColor = lerp(_ColorA.rgb, _ColorB.rgb, colorT);
                }
                
                // Calculate emission
                half3 emission = emissionColor * emissionMask * patternValue;
                
                // Pulse glow effect (adds slight bloom-like effect)
                if (_UsePulseGlow > 0.5)
                {
                    float glowPulse = patternValue;
                    emission += emission * _GlowRadius * glowPulse;
                }
                
                // Final color
                half3 color = base.rgb + emission;
                
                // Apply fog
                color = MixFog(color, input.fogFactor);
                
                return half4(color, base.a);
            }
            ENDHLSL
        }
    }
    
    FallBack "Universal Render Pipeline/Unlit"
}
