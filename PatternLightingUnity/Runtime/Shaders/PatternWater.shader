// Pattern Lighting System for Unity 6
// Advanced Water Shader

Shader "Pattern Lighting/Water"
{
    Properties
    {
        [Header(Colors)]
        _ShallowColor ("Shallow Color", Color) = (0.2, 0.6, 0.8, 0.8)
        _DeepColor ("Deep Color", Color) = (0.05, 0.2, 0.4, 1)
        
        [Header(Waves)]
        _WaveHeight ("Wave Height", Range(0, 2)) = 0.5
        _WaveSpeed ("Wave Speed", Range(0.1, 10)) = 1
        _WaveScale ("Wave Scale", Range(0.1, 50)) = 10
        _WaveNormalMap ("Wave Normal Map", 2D) = "bump" {}
        
        [Header(Appearance)]
        _Transparency ("Transparency", Range(0, 1)) = 0.7
        _RefractionStrength ("Refraction Strength", Range(0, 2)) = 0.5
        _DepthFade ("Depth Fade", Range(0.1, 10)) = 2
        
        [Header(Reflections)]
        _ReflectionTex ("Reflection Texture", 2D) = "white" {}
        _ReflectionIntensity ("Reflection Intensity", Range(0, 1)) = 0.8
        _FresnelPower ("Fresnel Power", Range(1, 10)) = 5
        
        [Header(Foam)]
        _FoamTex ("Foam Texture", 2D) = "white" {}
        _FoamAmount ("Foam Amount", Range(0, 2)) = 0.5
        _FoamColor ("Foam Color", Color) = (1, 1, 1, 1)
        _FoamScale ("Foam Scale", Range(0.1, 10)) = 1
        
        [Header(Caustics)]
        _CausticTex ("Caustic Texture", 2D) = "white" {}
        _CausticIntensity ("Caustic Intensity", Range(0, 2)) = 0.5
        _CausticSpeed ("Caustic Speed", Range(0.1, 5)) = 1
        _CausticScale ("Caustic Scale", Range(0.1, 10)) = 1
        
        [Header(Underwater)]
        _UnderwaterColor ("Underwater Color", Color) = (0.1, 0.3, 0.5, 1)
        _UnderwaterDensity ("Underwater Density", Range(0, 1)) = 0.3
    }
    
    SubShader
    {
        Tags 
        { 
            "RenderType" = "Transparent" 
            "Queue" = "Transparent"
            "RenderPipeline" = "UniversalPipeline"
        }
        
        Pass
        {
            Name "WaterForward"
            Tags { "LightMode" = "UniversalForward" }
            
            Blend SrcAlpha OneMinusSrcAlpha
            ZWrite Off
            Cull Back
            
            HLSLPROGRAM
            #pragma vertex vert
            #pragma fragment frag
            
            #pragma multi_compile_fog
            #pragma multi_compile _ _MAIN_LIGHT_SHADOWS
            #pragma multi_compile _ _MAIN_LIGHT_SHADOWS_CASCADE
            
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Core.hlsl"
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Lighting.hlsl"
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/DeclareDepthTexture.hlsl"
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/DeclareOpaqueTexture.hlsl"
            
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
                float4 screenPos : TEXCOORD5;
                float fogFactor : TEXCOORD6;
            };
            
            TEXTURE2D(_WaveNormalMap);
            SAMPLER(sampler_WaveNormalMap);
            TEXTURE2D(_ReflectionTex);
            SAMPLER(sampler_ReflectionTex);
            TEXTURE2D(_FoamTex);
            SAMPLER(sampler_FoamTex);
            TEXTURE2D(_CausticTex);
            SAMPLER(sampler_CausticTex);
            
            CBUFFER_START(UnityPerMaterial)
                float4 _ShallowColor;
                float4 _DeepColor;
                float _WaveHeight;
                float _WaveSpeed;
                float _WaveScale;
                float _Transparency;
                float _RefractionStrength;
                float _DepthFade;
                float _ReflectionIntensity;
                float _FresnelPower;
                float _FoamAmount;
                float4 _FoamColor;
                float _FoamScale;
                float _CausticIntensity;
                float _CausticSpeed;
                float _CausticScale;
                float4 _UnderwaterColor;
                float _UnderwaterDensity;
            CBUFFER_END
            
            // Gerstner wave function
            float3 GerstnerWave(float2 position, float2 direction, float steepness, float wavelength, float time)
            {
                float k = 2 * PI / wavelength;
                float c = sqrt(9.8 / k);
                float2 d = normalize(direction);
                float f = k * (dot(d, position) - c * time);
                float a = steepness / k;
                
                return float3(
                    d.x * (a * cos(f)),
                    a * sin(f),
                    d.y * (a * cos(f))
                );
            }
            
            Varyings vert(Attributes input)
            {
                Varyings output;
                
                // Calculate waves
                float3 posOS = input.positionOS.xyz;
                float time = _Time.y * _WaveSpeed;
                
                // Multiple wave layers
                float3 wave1 = GerstnerWave(posOS.xz, float2(1, 0), 0.25, _WaveScale, time);
                float3 wave2 = GerstnerWave(posOS.xz, float2(0, 1), 0.15, _WaveScale * 0.7, time * 1.2);
                float3 wave3 = GerstnerWave(posOS.xz, float2(1, 1), 0.1, _WaveScale * 1.3, time * 0.8);
                
                float3 waveOffset = (wave1 + wave2 + wave3) * _WaveHeight;
                posOS += waveOffset;
                
                // Transform
                VertexPositionInputs posInputs = GetVertexPositionInputs(posOS);
                VertexNormalInputs normalInputs = GetVertexNormalInputs(input.normalOS, input.tangentOS);
                
                output.positionCS = posInputs.positionCS;
                output.positionWS = posInputs.positionWS;
                output.normalWS = normalInputs.normalWS;
                output.tangentWS = normalInputs.tangentWS;
                output.bitangentWS = normalInputs.bitangentWS;
                output.uv = input.uv;
                output.screenPos = ComputeScreenPos(output.positionCS);
                output.fogFactor = ComputeFogFactor(output.positionCS.z);
                
                return output;
            }
            
            half4 frag(Varyings input) : SV_Target
            {
                // Screen UV
                float2 screenUV = input.screenPos.xy / input.screenPos.w;
                
                // Sample depth
                float depth = SampleSceneDepth(screenUV);
                float3 positionWS = ComputeWorldSpacePosition(screenUV, depth, UNITY_MATRIX_I_VP);
                float waterDepth = input.positionWS.y - positionWS.y;
                float depthFade = saturate(waterDepth / _DepthFade);
                
                // Normal mapping
                float time = _Time.y * _WaveSpeed;
                float2 uv1 = input.uv + float2(time * 0.05, time * 0.03);
                float2 uv2 = input.uv * 1.5 + float2(-time * 0.04, time * 0.02);
                
                float3 normal1 = UnpackNormal(SAMPLE_TEXTURE2D(_WaveNormalMap, sampler_WaveNormalMap, uv1));
                float3 normal2 = UnpackNormal(SAMPLE_TEXTURE2D(_WaveNormalMap, sampler_WaveNormalMap, uv2));
                float3 normalTS = normalize(normal1 + normal2);
                
                // Transform normal to world space
                float3x3 TBN = float3x3(input.tangentWS, input.bitangentWS, input.normalWS);
                float3 normalWS = normalize(mul(normalTS, TBN));
                
                // View direction
                float3 viewDir = normalize(GetWorldSpaceViewDir(input.positionWS));
                
                // Fresnel
                float fresnel = pow(1 - saturate(dot(normalWS, viewDir)), _FresnelPower);
                
                // Water color based on depth
                float4 waterColor = lerp(_ShallowColor, _DeepColor, depthFade);
                
                // Refraction
                float2 refractionOffset = normalTS.xy * _RefractionStrength * 0.1;
                float2 refractedUV = screenUV + refractionOffset * (1 - depthFade);
                float3 refractedColor = SampleSceneColor(refractedUV);
                
                // Reflection
                float4 reflectionColor = SAMPLE_TEXTURE2D(_ReflectionTex, sampler_ReflectionTex, screenUV + normalTS.xy * 0.05);
                
                // Foam
                float2 foamUV = input.uv * _FoamScale + float2(time * 0.1, 0);
                float foam = SAMPLE_TEXTURE2D(_FoamTex, sampler_FoamTex, foamUV).r;
                foam *= saturate(1 - depthFade * 2) * _FoamAmount;
                
                // Caustics
                float2 causticUV1 = input.positionWS.xz * _CausticScale * 0.1 + float2(time * _CausticSpeed * 0.1, 0);
                float2 causticUV2 = input.positionWS.xz * _CausticScale * 0.1 * 1.3 + float2(0, -time * _CausticSpeed * 0.1);
                float caustic1 = SAMPLE_TEXTURE2D(_CausticTex, sampler_CausticTex, causticUV1).r;
                float caustic2 = SAMPLE_TEXTURE2D(_CausticTex, sampler_CausticTex, causticUV2).r;
                float caustics = min(caustic1, caustic2) * _CausticIntensity * depthFade;
                
                // Lighting
                Light mainLight = GetMainLight();
                float3 halfDir = normalize(viewDir + mainLight.direction);
                float NdotL = saturate(dot(normalWS, mainLight.direction));
                float NdotH = saturate(dot(normalWS, halfDir));
                float specular = pow(NdotH, 128) * _ReflectionIntensity;
                
                // Combine
                float3 color = lerp(refractedColor, waterColor.rgb, waterColor.a * depthFade);
                color = lerp(color, reflectionColor.rgb, fresnel * _ReflectionIntensity);
                color += _FoamColor.rgb * foam;
                color += caustics * mainLight.color;
                color += specular * mainLight.color;
                color *= NdotL * 0.5 + 0.5; // Soft lighting
                
                // Apply fog
                color = MixFog(color, input.fogFactor);
                
                float alpha = saturate(_Transparency + fresnel * 0.3 + foam);
                
                return half4(color, alpha);
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
            
            HLSLPROGRAM
            #pragma vertex ShadowPassVertex
            #pragma fragment ShadowPassFragment
            
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Core.hlsl"
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/Lighting.hlsl"
            #include "Packages/com.unity.render-pipelines.universal/ShaderLibrary/ShadowCasterPass.hlsl"
            ENDHLSL
        }
    }
    
    FallBack "Universal Render Pipeline/Lit"
}
