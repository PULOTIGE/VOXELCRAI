// Post-Processing Shader for Xonotic Pattern Lighting Mod
// Includes: Bloom, Color Grading, Vignette, Chromatic Aberration
// Version 1.0.0

// ============================================================================
// VERTEX SHADER
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

uniform sampler2D Texture_Color;      // Main scene
uniform sampler2D Texture_Bloom;      // Bloom texture
uniform sampler2D Texture_Dirt;       // Lens dirt texture

// Post-process parameters
uniform vec4 PP_Bloom;          // x=intensity, y=threshold, z=dirt_intensity, w=enabled
uniform vec4 PP_ColorGrade;     // x=exposure, y=contrast, z=saturation, w=temperature
uniform vec4 PP_Vignette;       // x=intensity, y=smoothness, z=roundness, w=enabled
uniform vec4 PP_ChromaAberr;    // x=intensity, y=samples, z=radial, w=enabled
uniform vec4 PP_FilmGrain;      // x=intensity, y=speed, z=size, w=enabled
uniform float Time;

// ============================================================================
// COLOR FUNCTIONS
// ============================================================================

// Convert RGB to HSV
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0/3.0, 2.0/3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

// Convert HSV to RGB
vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0/3.0, 1.0/3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

// Color temperature adjustment
vec3 adjustTemperature(vec3 color, float temp) {
    // Warm positive, cool negative
    vec3 warm = vec3(1.0, 0.9, 0.8);
    vec3 cool = vec3(0.8, 0.9, 1.0);
    vec3 tint = mix(cool, warm, temp * 0.5 + 0.5);
    return color * tint;
}

// ============================================================================
// BLOOM
// ============================================================================

vec3 applyBloom(vec3 color, vec2 uv) {
    if (PP_Bloom.w < 0.5) return color;
    
    vec3 bloom = texture2D(Texture_Bloom, uv).rgb;
    
    // Lens dirt effect
    vec3 dirt = texture2D(Texture_Dirt, uv).rgb;
    float dirtMask = dot(bloom, vec3(0.299, 0.587, 0.114));
    vec3 dirtContrib = dirt * dirtMask * PP_Bloom.z;
    
    // Add bloom
    color += bloom * PP_Bloom.x;
    color += dirtContrib;
    
    return color;
}

// ============================================================================
// COLOR GRADING
// ============================================================================

vec3 applyColorGrading(vec3 color) {
    // Exposure
    color *= pow(2.0, PP_ColorGrade.x);
    
    // Contrast
    color = (color - 0.5) * PP_ColorGrade.y + 0.5;
    
    // Saturation
    float luminance = dot(color, vec3(0.299, 0.587, 0.114));
    color = mix(vec3(luminance), color, PP_ColorGrade.z);
    
    // Temperature
    color = adjustTemperature(color, PP_ColorGrade.w);
    
    return max(color, 0.0);
}

// ============================================================================
// VIGNETTE
// ============================================================================

vec3 applyVignette(vec3 color, vec2 uv) {
    if (PP_Vignette.w < 0.5) return color;
    
    vec2 center = uv - 0.5;
    
    // Adjust for aspect ratio and roundness
    center.x *= PP_Vignette.z;
    
    float dist = length(center);
    float vignette = smoothstep(0.5, 0.5 - PP_Vignette.y, dist * PP_Vignette.x);
    
    return color * vignette;
}

// ============================================================================
// CHROMATIC ABERRATION
// ============================================================================

vec3 applyChromaticAberration(vec2 uv) {
    if (PP_ChromaAberr.w < 0.5) {
        return texture2D(Texture_Color, uv).rgb;
    }
    
    vec2 center = uv - 0.5;
    float dist = length(center);
    
    // Radial or uniform
    vec2 dir = PP_ChromaAberr.z > 0.5 ? normalize(center) : vec2(1.0, 0.0);
    
    float intensity = PP_ChromaAberr.x * dist;
    
    // Sample RGB at different offsets
    vec2 rOffset = dir * intensity * 1.0;
    vec2 gOffset = dir * intensity * 0.5;
    vec2 bOffset = dir * intensity * 0.0;
    
    float r = texture2D(Texture_Color, uv + rOffset).r;
    float g = texture2D(Texture_Color, uv + gOffset).g;
    float b = texture2D(Texture_Color, uv + bOffset).b;
    
    return vec3(r, g, b);
}

// ============================================================================
// FILM GRAIN
// ============================================================================

float filmGrainNoise(vec2 uv) {
    return fract(sin(dot(uv, vec2(12.9898, 78.233))) * 43758.5453);
}

vec3 applyFilmGrain(vec3 color, vec2 uv) {
    if (PP_FilmGrain.w < 0.5) return color;
    
    vec2 noiseUV = uv * PP_FilmGrain.z + Time * PP_FilmGrain.y;
    float noise = filmGrainNoise(noiseUV);
    noise = (noise - 0.5) * PP_FilmGrain.x;
    
    // Apply grain more to midtones
    float luminance = dot(color, vec3(0.299, 0.587, 0.114));
    float grainAmount = 1.0 - abs(luminance - 0.5) * 2.0;
    
    return color + noise * grainAmount;
}

// ============================================================================
// TONEMAPPING
// ============================================================================

vec3 ACESFilm(vec3 x) {
    float a = 2.51;
    float b = 0.03;
    float c = 2.43;
    float d = 0.59;
    float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

// ============================================================================
// MAIN
// ============================================================================

void main() {
    vec2 uv = v_TexCoord;
    
    // Get base color (with chromatic aberration if enabled)
    vec3 color = applyChromaticAberration(uv);
    
    // Apply bloom
    color = applyBloom(color, uv);
    
    // Tonemapping
    color = ACESFilm(color);
    
    // Color grading
    color = applyColorGrading(color);
    
    // Vignette
    color = applyVignette(color, uv);
    
    // Film grain
    color = applyFilmGrain(color, uv);
    
    // Gamma correction
    color = pow(color, vec3(1.0 / 2.2));
    
    gl_FragColor = vec4(color, 1.0);
}

#endif
