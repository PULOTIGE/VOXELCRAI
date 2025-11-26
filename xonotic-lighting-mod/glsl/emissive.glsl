// Emissive Materials Shader for Xonotic
// Supports pattern-based emission animations
// Version 1.0.0

// ============================================================================
// VERTEX SHADER
// ============================================================================
#ifdef VERTEX_SHADER

attribute vec3 Attrib_Position;
attribute vec3 Attrib_Normal;
attribute vec4 Attrib_TexCoord0;
attribute vec4 Attrib_Color;

uniform mat4 ModelViewProjectionMatrix;
uniform mat4 ModelMatrix;
uniform mat3 NormalMatrix;

varying vec3 v_WorldPosition;
varying vec3 v_Normal;
varying vec2 v_TexCoord;
varying vec4 v_Color;

void main() {
    gl_Position = ModelViewProjectionMatrix * vec4(Attrib_Position, 1.0);
    v_WorldPosition = (ModelMatrix * vec4(Attrib_Position, 1.0)).xyz;
    v_Normal = normalize(NormalMatrix * Attrib_Normal);
    v_TexCoord = Attrib_TexCoord0.xy;
    v_Color = Attrib_Color;
}

#endif

// ============================================================================
// FRAGMENT SHADER
// ============================================================================
#ifdef FRAGMENT_SHADER

varying vec3 v_WorldPosition;
varying vec3 v_Normal;
varying vec2 v_TexCoord;
varying vec4 v_Color;

uniform sampler2D Texture_Color;      // Base color
uniform sampler2D Texture_Emissive;   // Emissive map

// Emissive parameters
uniform vec4 Emissive_Color;      // xyz=color, w=intensity
uniform vec4 Emissive_Pattern;    // x=pattern_id, y=speed, z=time, w=enabled

// Pattern functions
float getEmissivePattern(float time, float patternId) {
    float t = time * Emissive_Pattern.y;
    
    if (patternId < 1.0) {
        // Steady glow
        return 1.0;
    } else if (patternId < 2.0) {
        // Pulse
        return 0.5 + 0.5 * sin(t * 3.0);
    } else if (patternId < 3.0) {
        // Heartbeat
        float beat = pow(sin(t * 2.5), 12.0);
        float beat2 = pow(sin(t * 2.5 + 0.3), 12.0) * 0.5;
        return beat + beat2;
    } else if (patternId < 4.0) {
        // Scanner
        float scan = fract(t * 0.5);
        float pos = v_TexCoord.y;
        return smoothstep(scan - 0.1, scan, pos) * smoothstep(scan + 0.1, scan, pos);
    } else if (patternId < 5.0) {
        // Wave
        float wave = sin(v_TexCoord.x * 10.0 + t * 3.0) * 0.5 + 0.5;
        return wave;
    } else if (patternId < 6.0) {
        // Flicker
        return 0.8 + 0.2 * (sin(t * 40.0) * sin(t * 17.0));
    } else if (patternId < 7.0) {
        // Energy buildup
        float buildup = smoothstep(0.0, 4.0, mod(t, 5.0));
        float release = 1.0 - smoothstep(4.0, 4.2, mod(t, 5.0));
        return buildup * release;
    } else if (patternId < 8.0) {
        // Plasma
        float plasma = sin(v_TexCoord.x * 8.0 + t) * sin(v_TexCoord.y * 8.0 + t * 1.3);
        plasma += sin(length(v_TexCoord - 0.5) * 12.0 - t * 2.0);
        return plasma * 0.25 + 0.75;
    } else if (patternId < 9.0) {
        // Alarm pulse
        return step(0.5, sin(t * 8.0)) * (sin(t * 0.5) * 0.3 + 0.7);
    } else {
        // Random sparks
        float spark = fract(sin(dot(v_TexCoord + t * 0.1, vec2(12.9898, 78.233))) * 43758.5453);
        return step(0.95, spark);
    }
}

void main() {
    // Sample textures
    vec4 baseColor = texture2D(Texture_Color, v_TexCoord) * v_Color;
    vec4 emissiveMap = texture2D(Texture_Emissive, v_TexCoord);
    
    // Calculate emissive contribution
    vec3 emissive = vec3(0.0);
    
    if (Emissive_Pattern.w > 0.5) {
        float pattern = getEmissivePattern(Emissive_Pattern.z, Emissive_Pattern.x);
        emissive = Emissive_Color.rgb * Emissive_Color.w * emissiveMap.rgb * pattern;
    } else {
        emissive = Emissive_Color.rgb * Emissive_Color.w * emissiveMap.rgb;
    }
    
    // Combine base color with emission
    vec3 finalColor = baseColor.rgb + emissive;
    
    // Add glow halo effect at edges (fresnel-like)
    vec3 viewDir = normalize(-v_WorldPosition);
    float rim = 1.0 - max(dot(v_Normal, viewDir), 0.0);
    rim = pow(rim, 3.0);
    finalColor += emissive * rim * 0.5;
    
    gl_FragColor = vec4(finalColor, baseColor.a);
}

#endif
