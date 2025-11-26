// Screen-Space Reflections Shader for Xonotic
// Part of Pattern Lighting Mod
// Version 1.0.0

// ============================================================================
// VERTEX SHADER (Fullscreen quad)
// ============================================================================
#ifdef VERTEX_SHADER

attribute vec3 Attrib_Position;
attribute vec2 Attrib_TexCoord0;

varying vec2 v_TexCoord;

void main() {
    gl_Position = vec4(Attrib_Position.xy, 0.0, 1.0);
    v_TexCoord = Attrib_TexCoord0;
}

#endif

// ============================================================================
// FRAGMENT SHADER
// ============================================================================
#ifdef FRAGMENT_SHADER

varying vec2 v_TexCoord;

// Input textures
uniform sampler2D Texture_Color;     // Scene color
uniform sampler2D Texture_Depth;     // Scene depth
uniform sampler2D Texture_Normal;    // G-Buffer normals
uniform sampler2D Texture_Gloss;     // Roughness/Metallic

// Matrices
uniform mat4 ProjectionMatrix;
uniform mat4 InverseProjectionMatrix;
uniform mat4 ViewMatrix;
uniform mat4 InverseViewMatrix;

// SSR Parameters
uniform vec4 SSR_Config;  // x=max_distance, y=stride, z=thickness, w=enabled
uniform vec4 SSR_Quality; // x=max_steps, y=binary_steps, z=jitter, w=fade

// Screen size
uniform vec2 ScreenSize;

const float PI = 3.14159265359;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

// Reconstruct view-space position from depth
vec3 getViewPosition(vec2 uv, float depth) {
    vec4 clipPos = vec4(uv * 2.0 - 1.0, depth * 2.0 - 1.0, 1.0);
    vec4 viewPos = InverseProjectionMatrix * clipPos;
    return viewPos.xyz / viewPos.w;
}

// Project view-space position to screen UV
vec2 projectToScreen(vec3 viewPos) {
    vec4 clipPos = ProjectionMatrix * vec4(viewPos, 1.0);
    clipPos.xyz /= clipPos.w;
    return clipPos.xy * 0.5 + 0.5;
}

// Get linear depth
float getLinearDepth(float depth) {
    float near = 0.1;
    float far = 10000.0;
    return (2.0 * near) / (far + near - depth * (far - near));
}

// Hash function for jittering
float hash(vec2 p) {
    return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453);
}

// ============================================================================
// SSR RAY MARCHING
// ============================================================================

vec4 traceSSR(vec3 viewPos, vec3 viewDir, vec3 normal, float roughness) {
    // Reflect view direction around normal
    vec3 reflectDir = reflect(viewDir, normal);
    
    // Ray march parameters
    float maxDist = SSR_Config.x;
    float stride = SSR_Config.y;
    float thickness = SSR_Config.z;
    int maxSteps = int(SSR_Quality.x);
    int binarySteps = int(SSR_Quality.y);
    float jitter = SSR_Quality.z;
    
    // Add jitter to reduce banding
    float startOffset = hash(v_TexCoord * ScreenSize) * stride * jitter;
    
    // Start and end positions
    vec3 startPos = viewPos + reflectDir * startOffset;
    vec3 endPos = viewPos + reflectDir * maxDist;
    
    // Project to screen space
    vec2 startUV = projectToScreen(startPos);
    vec2 endUV = projectToScreen(endPos);
    
    // Delta per step
    vec2 deltaUV = (endUV - startUV) / float(maxSteps);
    float deltaZ = (endPos.z - startPos.z) / float(maxSteps);
    
    // Ray march
    vec2 currentUV = startUV;
    float currentZ = startPos.z;
    
    for (int i = 0; i < maxSteps; i++) {
        // Advance ray
        currentUV += deltaUV;
        currentZ += deltaZ;
        
        // Check bounds
        if (currentUV.x < 0.0 || currentUV.x > 1.0 || 
            currentUV.y < 0.0 || currentUV.y > 1.0) {
            break;
        }
        
        // Sample scene depth
        float sceneDepth = texture2D(Texture_Depth, currentUV).r;
        vec3 sceneViewPos = getViewPosition(currentUV, sceneDepth);
        
        // Check intersection
        float depthDiff = currentZ - sceneViewPos.z;
        
        if (depthDiff > 0.0 && depthDiff < thickness) {
            // Binary search refinement
            vec2 hitUV = currentUV;
            float hitZ = currentZ;
            
            vec2 searchDelta = deltaUV * 0.5;
            float searchZDelta = deltaZ * 0.5;
            
            for (int j = 0; j < binarySteps; j++) {
                float testDepth = texture2D(Texture_Depth, hitUV).r;
                vec3 testViewPos = getViewPosition(hitUV, testDepth);
                float testDiff = hitZ - testViewPos.z;
                
                if (testDiff > 0.0) {
                    hitUV -= searchDelta;
                    hitZ -= searchZDelta;
                } else {
                    hitUV += searchDelta;
                    hitZ += searchZDelta;
                }
                
                searchDelta *= 0.5;
                searchZDelta *= 0.5;
            }
            
            // Calculate confidence/fade
            float edgeFade = 1.0;
            float distFromEdge = min(
                min(hitUV.x, 1.0 - hitUV.x),
                min(hitUV.y, 1.0 - hitUV.y)
            );
            edgeFade = smoothstep(0.0, 0.1, distFromEdge);
            
            float distanceFade = 1.0 - float(i) / float(maxSteps);
            
            float fade = edgeFade * distanceFade * SSR_Quality.w;
            
            // Blur based on roughness (simulate)
            vec3 hitColor = texture2D(Texture_Color, hitUV).rgb;
            
            return vec4(hitColor, fade * (1.0 - roughness * 0.5));
        }
    }
    
    return vec4(0.0);
}

// ============================================================================
// MAIN
// ============================================================================

void main() {
    if (SSR_Config.w < 0.5) {
        // SSR disabled, pass through
        gl_FragColor = texture2D(Texture_Color, v_TexCoord);
        return;
    }
    
    // Get scene data
    vec4 sceneColor = texture2D(Texture_Color, v_TexCoord);
    float depth = texture2D(Texture_Depth, v_TexCoord).r;
    vec3 normal = texture2D(Texture_Normal, v_TexCoord).rgb * 2.0 - 1.0;
    vec4 glossData = texture2D(Texture_Gloss, v_TexCoord);
    
    float roughness = glossData.r;
    float metallic = glossData.g;
    
    // Skip SSR for very rough surfaces or sky
    if (roughness > 0.8 || depth > 0.9999) {
        gl_FragColor = sceneColor;
        return;
    }
    
    // Reconstruct view-space position and direction
    vec3 viewPos = getViewPosition(v_TexCoord, depth);
    vec3 viewDir = normalize(viewPos);
    
    // Transform normal to view space
    mat3 normalMatrix = mat3(ViewMatrix);
    vec3 viewNormal = normalize(normalMatrix * normal);
    
    // Trace SSR
    vec4 ssrResult = traceSSR(viewPos, viewDir, viewNormal, roughness);
    
    // Fresnel for reflection intensity
    float NdotV = max(dot(viewNormal, -viewDir), 0.0);
    float fresnel = pow(1.0 - NdotV, 5.0);
    fresnel = mix(0.04, 1.0, fresnel);
    fresnel *= mix(0.5, 1.0, metallic);
    
    // Blend reflection with scene
    vec3 finalColor = mix(sceneColor.rgb, ssrResult.rgb, ssrResult.a * fresnel);
    
    gl_FragColor = vec4(finalColor, sceneColor.a);
}

#endif
