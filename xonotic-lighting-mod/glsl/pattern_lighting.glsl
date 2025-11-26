// Pattern-Based PBR Lighting System for Xonotic/DarkPlaces
// Based on Adaptive Entity Engine lighting patterns
// Version 1.0.0

// ============================================================================
// VERTEX SHADER
// ============================================================================
#ifdef VERTEX_SHADER

attribute vec3 Attrib_Position;
attribute vec3 Attrib_Normal;
attribute vec4 Attrib_TexCoord0;
attribute vec4 Attrib_TexCoord1; // Lightmap coords
attribute vec4 Attrib_TexCoord2; // Deluxemap coords
attribute vec4 Attrib_Color;
attribute vec4 Attrib_TexCoord4; // Tangent
attribute vec3 Attrib_TexCoord5; // Bitangent

uniform mat4 ModelViewProjectionMatrix;
uniform mat4 ModelViewMatrix;
uniform mat4 ModelMatrix;
uniform mat3 NormalMatrix;

varying vec3 v_Position;
varying vec3 v_WorldPosition;
varying vec3 v_Normal;
varying vec3 v_Tangent;
varying vec3 v_Bitangent;
varying vec2 v_TexCoord;
varying vec2 v_LightmapCoord;
varying vec4 v_Color;
varying vec3 v_ViewDir;

uniform vec3 EyePosition;

void main() {
    gl_Position = ModelViewProjectionMatrix * vec4(Attrib_Position, 1.0);
    
    v_Position = (ModelViewMatrix * vec4(Attrib_Position, 1.0)).xyz;
    v_WorldPosition = (ModelMatrix * vec4(Attrib_Position, 1.0)).xyz;
    
    v_Normal = normalize(NormalMatrix * Attrib_Normal);
    v_Tangent = normalize(NormalMatrix * Attrib_TexCoord4.xyz);
    v_Bitangent = normalize(NormalMatrix * Attrib_TexCoord5);
    
    v_TexCoord = Attrib_TexCoord0.xy;
    v_LightmapCoord = Attrib_TexCoord1.xy;
    v_Color = Attrib_Color;
    
    v_ViewDir = normalize(EyePosition - v_WorldPosition);
}

#endif

// ============================================================================
// FRAGMENT SHADER
// ============================================================================
#ifdef FRAGMENT_SHADER

// Varying inputs
varying vec3 v_Position;
varying vec3 v_WorldPosition;
varying vec3 v_Normal;
varying vec3 v_Tangent;
varying vec3 v_Bitangent;
varying vec2 v_TexCoord;
varying vec2 v_LightmapCoord;
varying vec4 v_Color;
varying vec3 v_ViewDir;

// Texture samplers
uniform sampler2D Texture_Color;        // Albedo/Diffuse
uniform sampler2D Texture_Normal;       // Normal map
uniform sampler2D Texture_Gloss;        // Roughness/Metallic (R=Rough, G=Metal, B=AO)
uniform sampler2D Texture_Lightmap;     // Baked lightmap
uniform sampler2D Texture_Deluxemap;    // Lightmap direction
uniform samplerCube Texture_Cube;       // Environment cubemap

// Pattern lighting uniforms
uniform vec4 PatternLighting_Direct;    // xyz = color, w = intensity
uniform vec4 PatternLighting_Indirect;  // xyz = color, w = intensity  
uniform vec4 PatternLighting_Ambient;   // xyz = color, w = AO strength
uniform vec4 PatternLighting_Config;    // x = time, y = pattern_id, z = speed, w = enabled

// Light uniforms (from engine)
uniform vec3 LightColor;
uniform vec3 LightDir;
uniform vec3 LightPosition;
uniform float LightRadius;

// Material properties
uniform float MaterialRoughness;
uniform float MaterialMetallic;
uniform float MaterialReflectivity;

// Constants
const float PI = 3.14159265359;
const float EPSILON = 0.0001;

// ============================================================================
// PBR FUNCTIONS
// ============================================================================

// Normal Distribution Function (GGX/Trowbridge-Reitz)
float DistributionGGX(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;
    
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    
    return a2 / max(denom, EPSILON);
}

// Geometry Function (Smith's Schlick-GGX)
float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = roughness + 1.0;
    float k = (r * r) / 8.0;
    return NdotV / (NdotV * (1.0 - k) + k);
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx1 = GeometrySchlickGGX(NdotV, roughness);
    float ggx2 = GeometrySchlickGGX(NdotL, roughness);
    return ggx1 * ggx2;
}

// Fresnel (Schlick approximation)
vec3 FresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

vec3 FresnelSchlickRoughness(float cosTheta, vec3 F0, float roughness) {
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

// ============================================================================
// PATTERN LIGHTING FUNCTIONS
// ============================================================================

// Generate procedural light pattern
float getPattern(float time, float patternId) {
    float t = time * PatternLighting_Config.z;
    
    if (patternId < 1.0) {
        // Pattern 0: Steady
        return 1.0;
    } else if (patternId < 2.0) {
        // Pattern 1: Pulse
        return 0.5 + 0.5 * sin(t * 2.0);
    } else if (patternId < 3.0) {
        // Pattern 2: Flicker
        return 0.7 + 0.3 * sin(t * 20.0) * sin(t * 7.3);
    } else if (patternId < 4.0) {
        // Pattern 3: Strobe
        return step(0.5, fract(t));
    } else if (patternId < 5.0) {
        // Pattern 4: Candle
        float flicker = sin(t * 12.0) * sin(t * 5.7) * sin(t * 3.1);
        return 0.8 + 0.2 * flicker;
    } else if (patternId < 6.0) {
        // Pattern 5: Fluorescent startup
        float startup = smoothstep(0.0, 2.0, mod(t, 5.0));
        float buzz = 0.05 * sin(t * 120.0);
        return startup + buzz * startup;
    } else if (patternId < 7.0) {
        // Pattern 6: Lightning
        float flash = pow(max(0.0, sin(t * 0.5)), 20.0);
        return flash;
    } else if (patternId < 8.0) {
        // Pattern 7: Fire
        float fire = 0.7 + 0.3 * sin(t * 8.0) * sin(t * 4.3) * sin(t * 2.1);
        return fire;
    } else if (patternId < 9.0) {
        // Pattern 8: Alarm
        return step(0.5, sin(t * 4.0));
    } else {
        // Pattern 9: Underwater caustics
        vec2 uv = v_WorldPosition.xz * 0.1;
        float caustic = sin(uv.x * 10.0 + t) * sin(uv.y * 10.0 + t * 0.7);
        return 0.7 + 0.3 * caustic;
    }
}

// Calculate pattern-modulated lighting color
vec3 getPatternLightColor(float time) {
    float pattern = getPattern(time, PatternLighting_Config.y);
    vec3 directColor = PatternLighting_Direct.rgb * PatternLighting_Direct.w;
    return directColor * pattern;
}

// Spherical Harmonics for ambient lighting
vec3 evaluateSH(vec3 normal) {
    // Simple SH approximation using hemisphere
    vec3 skyColor = PatternLighting_Indirect.rgb * PatternLighting_Indirect.w;
    vec3 groundColor = PatternLighting_Ambient.rgb * 0.3;
    
    float hemisphere = normal.y * 0.5 + 0.5;
    return mix(groundColor, skyColor, hemisphere);
}

// ============================================================================
// REFLECTION FUNCTIONS
// ============================================================================

// Screen-space reflection approximation
vec3 getReflection(vec3 V, vec3 N, float roughness) {
    vec3 R = reflect(-V, N);
    
    // Sample environment cubemap with roughness-based mip level
    float mipLevel = roughness * 8.0;
    vec3 envColor = textureCube(Texture_Cube, R, mipLevel).rgb;
    
    return envColor;
}

// Planar reflection for floors
vec3 getPlanarReflection(vec3 worldPos, vec3 N, vec3 V, float roughness) {
    // Check if surface is roughly horizontal (floor)
    if (abs(N.y) > 0.9) {
        vec3 R = reflect(-V, N);
        
        // Blur based on roughness
        float blur = roughness * 0.5;
        vec3 reflection = textureCube(Texture_Cube, R).rgb;
        
        // Fresnel-based fade
        float fresnel = pow(1.0 - max(dot(N, V), 0.0), 3.0);
        return reflection * fresnel * (1.0 - roughness);
    }
    return vec3(0.0);
}

// ============================================================================
// MAIN FRAGMENT SHADER
// ============================================================================

void main() {
    // Sample textures
    vec4 albedoSample = texture2D(Texture_Color, v_TexCoord);
    vec3 albedo = albedoSample.rgb * v_Color.rgb;
    float alpha = albedoSample.a * v_Color.a;
    
    // Normal mapping
    vec3 normalMap = texture2D(Texture_Normal, v_TexCoord).rgb * 2.0 - 1.0;
    mat3 TBN = mat3(v_Tangent, v_Bitangent, v_Normal);
    vec3 N = normalize(TBN * normalMap);
    
    // PBR properties
    vec4 glossSample = texture2D(Texture_Gloss, v_TexCoord);
    float roughness = max(glossSample.r, 0.04);
    float metallic = glossSample.g;
    float ao = glossSample.b;
    
    // Apply material overrides if set
    if (MaterialRoughness > 0.0) roughness = MaterialRoughness;
    if (MaterialMetallic > 0.0) metallic = MaterialMetallic;
    
    // View direction
    vec3 V = normalize(v_ViewDir);
    
    // Base reflectance
    vec3 F0 = mix(vec3(0.04), albedo, metallic);
    
    // ========================================================================
    // DIRECT LIGHTING (Pattern-modulated)
    // ========================================================================
    
    vec3 directLighting = vec3(0.0);
    
    if (PatternLighting_Config.w > 0.5) {
        // Pattern lighting enabled
        float time = PatternLighting_Config.x;
        vec3 patternColor = getPatternLightColor(time);
        
        vec3 L = normalize(LightDir);
        vec3 H = normalize(V + L);
        
        float NdotL = max(dot(N, L), 0.0);
        float NdotV = max(dot(N, V), 0.0);
        
        // Cook-Torrance BRDF
        float D = DistributionGGX(N, H, roughness);
        float G = GeometrySmith(N, V, L, roughness);
        vec3 F = FresnelSchlick(max(dot(H, V), 0.0), F0);
        
        vec3 numerator = D * G * F;
        float denominator = 4.0 * NdotV * NdotL + EPSILON;
        vec3 specular = numerator / denominator;
        
        vec3 kS = F;
        vec3 kD = (1.0 - kS) * (1.0 - metallic);
        
        directLighting = (kD * albedo / PI + specular) * patternColor * NdotL;
    } else {
        // Standard DarkPlaces lighting
        vec3 L = normalize(LightDir);
        float NdotL = max(dot(N, L), 0.0);
        directLighting = albedo * LightColor * NdotL;
    }
    
    // ========================================================================
    // LIGHTMAP CONTRIBUTION
    // ========================================================================
    
    vec3 lightmapColor = texture2D(Texture_Lightmap, v_LightmapCoord).rgb;
    vec3 lightmapDir = texture2D(Texture_Deluxemap, v_LightmapCoord).rgb * 2.0 - 1.0;
    
    float lightmapNdotL = max(dot(N, normalize(lightmapDir)), 0.0);
    vec3 lightmapContrib = lightmapColor * lightmapNdotL;
    
    // ========================================================================
    // INDIRECT LIGHTING (Ambient + SH)
    // ========================================================================
    
    vec3 ambient;
    if (PatternLighting_Config.w > 0.5) {
        // Pattern-based ambient with SH
        vec3 shLight = evaluateSH(N);
        vec3 F_ambient = FresnelSchlickRoughness(max(dot(N, V), 0.0), F0, roughness);
        vec3 kD_ambient = (1.0 - F_ambient) * (1.0 - metallic);
        ambient = kD_ambient * albedo * shLight * ao;
    } else {
        // Standard ambient
        ambient = albedo * PatternLighting_Ambient.rgb * ao;
    }
    
    // ========================================================================
    // REFLECTIONS
    // ========================================================================
    
    vec3 reflection = vec3(0.0);
    if (MaterialReflectivity > 0.0 || metallic > 0.1) {
        vec3 envReflection = getReflection(V, N, roughness);
        vec3 planarReflection = getPlanarReflection(v_WorldPosition, N, V, roughness);
        
        vec3 F_reflection = FresnelSchlickRoughness(max(dot(N, V), 0.0), F0, roughness);
        reflection = (envReflection + planarReflection) * F_reflection;
        reflection *= mix(MaterialReflectivity, 1.0, metallic);
    }
    
    // ========================================================================
    // COMBINE LIGHTING
    // ========================================================================
    
    vec3 finalColor = vec3(0.0);
    
    // Add all lighting contributions
    finalColor += directLighting;
    finalColor += lightmapContrib * albedo * 0.5;
    finalColor += ambient;
    finalColor += reflection;
    
    // Apply AO from pattern system
    float patternAO = PatternLighting_Ambient.w;
    finalColor *= mix(1.0, ao, patternAO);
    
    // ========================================================================
    // TONEMAPPING AND OUTPUT
    // ========================================================================
    
    // ACES Filmic tonemapping
    vec3 x = finalColor;
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    finalColor = clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
    
    // Gamma correction
    finalColor = pow(finalColor, vec3(1.0 / 2.2));
    
    gl_FragColor = vec4(finalColor, alpha);
}

#endif
