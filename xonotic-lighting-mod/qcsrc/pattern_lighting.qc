// Pattern Lighting System for Xonotic
// QuakeC implementation
// Version 1.0.0

// ============================================================================
// CONSTANTS
// ============================================================================

// Light pattern IDs
float PATTERN_STEADY     = 0;
float PATTERN_PULSE      = 1;
float PATTERN_FLICKER    = 2;
float PATTERN_STROBE     = 3;
float PATTERN_CANDLE     = 4;
float PATTERN_FLUORESCENT = 5;
float PATTERN_LIGHTNING  = 6;
float PATTERN_FIRE       = 7;
float PATTERN_ALARM      = 8;
float PATTERN_UNDERWATER = 9;

// Emissive pattern IDs
float EMIT_STEADY    = 0;
float EMIT_PULSE     = 1;
float EMIT_HEARTBEAT = 2;
float EMIT_SCANNER   = 3;
float EMIT_WAVE      = 4;
float EMIT_FLICKER   = 5;
float EMIT_BUILDUP   = 6;
float EMIT_PLASMA    = 7;
float EMIT_ALARM     = 8;
float EMIT_SPARKS    = 9;

// ============================================================================
// CVARS (Console Variables)
// ============================================================================

// Main controls
float autocvar_r_pattern_lighting = 1;           // Enable pattern lighting
float autocvar_r_pattern_intensity = 1.0;        // Global intensity multiplier
float autocvar_r_pattern_speed = 1.0;            // Global speed multiplier

// Direct lighting
vector autocvar_r_pattern_direct_color = '1 1 0.9';
float autocvar_r_pattern_direct_intensity = 1.0;

// Indirect lighting  
vector autocvar_r_pattern_indirect_color = '0.4 0.5 0.7';
float autocvar_r_pattern_indirect_intensity = 0.3;

// Ambient
vector autocvar_r_pattern_ambient_color = '0.1 0.1 0.15';
float autocvar_r_pattern_ao_strength = 0.8;

// Reflections
float autocvar_r_ssr_enable = 1;
float autocvar_r_ssr_maxdist = 100;
float autocvar_r_ssr_stride = 1;
float autocvar_r_ssr_thickness = 0.5;
float autocvar_r_ssr_maxsteps = 64;
float autocvar_r_ssr_fade = 0.9;

// Post-processing
float autocvar_r_pp_bloom = 1;
float autocvar_r_pp_bloom_intensity = 0.3;
float autocvar_r_pp_bloom_threshold = 0.8;
float autocvar_r_pp_exposure = 0;
float autocvar_r_pp_contrast = 1;
float autocvar_r_pp_saturation = 1;
float autocvar_r_pp_temperature = 0;
float autocvar_r_pp_vignette = 1;
float autocvar_r_pp_vignette_intensity = 0.3;
float autocvar_r_pp_chromatic = 0;
float autocvar_r_pp_chromatic_intensity = 0.01;
float autocvar_r_pp_filmgrain = 0;
float autocvar_r_pp_filmgrain_intensity = 0.05;

// ============================================================================
// PATTERN LIGHT ENTITY
// ============================================================================

.float pattern_id;
.float pattern_speed;
.float pattern_phase;
.vector pattern_color;
.float pattern_intensity;
.float pattern_radius;

// Spawn function for pattern lights
void spawnfunc_light_pattern() {
    if (!self.pattern_id)
        self.pattern_id = PATTERN_STEADY;
    if (!self.pattern_speed)
        self.pattern_speed = 1.0;
    if (!self.pattern_color)
        self.pattern_color = '1 1 1';
    if (!self.pattern_intensity)
        self.pattern_intensity = 1.0;
    if (!self.pattern_radius)
        self.pattern_radius = 300;
    if (!self.pattern_phase)
        self.pattern_phase = random() * 6.28; // Random phase offset
        
    // Set up light
    self.light_lev = self.pattern_intensity * 300;
    setorigin(self, self.origin);
    
    // Register for pattern updates
    self.think = pattern_light_think;
    self.nextthink = time + 0.05;
}

// Pattern calculation
float PatternValue(float pattern_id, float t) {
    if (pattern_id == PATTERN_STEADY)
        return 1.0;
    else if (pattern_id == PATTERN_PULSE)
        return 0.5 + 0.5 * sin(t * 2);
    else if (pattern_id == PATTERN_FLICKER)
        return 0.7 + 0.3 * sin(t * 20) * sin(t * 7.3);
    else if (pattern_id == PATTERN_STROBE)
        return (sin(t * 10) > 0) ? 1 : 0;
    else if (pattern_id == PATTERN_CANDLE)
        return 0.8 + 0.2 * sin(t * 12) * sin(t * 5.7) * sin(t * 3.1);
    else if (pattern_id == PATTERN_FLUORESCENT) {
        float startup = bound(0, (t % 5) / 2, 1);
        float buzz = 0.05 * sin(t * 120);
        return startup + buzz * startup;
    }
    else if (pattern_id == PATTERN_LIGHTNING)
        return pow(max(0, sin(t * 0.5)), 20);
    else if (pattern_id == PATTERN_FIRE)
        return 0.7 + 0.3 * sin(t * 8) * sin(t * 4.3) * sin(t * 2.1);
    else if (pattern_id == PATTERN_ALARM)
        return (sin(t * 4) > 0) ? 1 : 0.2;
    
    return 1.0;
}

// Think function for pattern lights
void pattern_light_think() {
    if (!autocvar_r_pattern_lighting) {
        self.light_lev = self.pattern_intensity * 300;
        self.nextthink = time + 0.1;
        return;
    }
    
    float t = time * self.pattern_speed * autocvar_r_pattern_speed + self.pattern_phase;
    float value = PatternValue(self.pattern_id, t);
    
    self.light_lev = self.pattern_intensity * value * autocvar_r_pattern_intensity * 300;
    
    // Update color
    // Note: DarkPlaces uses colormod for dynamic light colors
    self.colormod = self.pattern_color * value;
    
    self.nextthink = time + 0.02; // 50 FPS updates
}

// ============================================================================
// EMISSIVE SURFACE ENTITY
// ============================================================================

.float emit_pattern;
.float emit_speed;
.vector emit_color;
.float emit_intensity;

void spawnfunc_func_emissive() {
    if (!self.emit_pattern)
        self.emit_pattern = EMIT_STEADY;
    if (!self.emit_speed)
        self.emit_speed = 1.0;
    if (!self.emit_color)
        self.emit_color = '1 0.5 0';
    if (!self.emit_intensity)
        self.emit_intensity = 2.0;
    
    // Set up rendering
    self.effects |= EF_FULLBRIGHT;
    
    self.think = emissive_think;
    self.nextthink = time + 0.05;
}

void emissive_think() {
    float t = time * self.emit_speed;
    
    // Send to shader
    // Note: This would need engine support or custom rendering
    
    self.nextthink = time + 0.02;
}

// ============================================================================
// REFLECTION PROBE ENTITY
// ============================================================================

.float probe_resolution;
.float probe_range;
.float probe_update_rate;

void spawnfunc_env_reflection_probe() {
    if (!self.probe_resolution)
        self.probe_resolution = 256;
    if (!self.probe_range)
        self.probe_range = 1000;
    if (!self.probe_update_rate)
        self.probe_update_rate = 1; // Seconds between updates
    
    // Register probe
    // Note: Would need engine-level cubemap capture support
    
    self.think = reflection_probe_think;
    self.nextthink = time + self.probe_update_rate;
}

void reflection_probe_think() {
    // Capture cubemap if needed
    // This would require DarkPlaces extension
    
    self.nextthink = time + self.probe_update_rate;
}

// ============================================================================
// SHADER PARAMETER UPDATES
// ============================================================================

void UpdatePatternLightingShaderParams() {
    // Update shader uniforms
    // These would be passed to GLSL shaders
    
    string cmd;
    
    // Direct lighting
    cmd = strcat("set PatternLighting_Direct \"",
        ftos(autocvar_r_pattern_direct_color_x), " ",
        ftos(autocvar_r_pattern_direct_color_y), " ",
        ftos(autocvar_r_pattern_direct_color_z), " ",
        ftos(autocvar_r_pattern_direct_intensity), "\"");
    localcmd(cmd);
    
    // Indirect lighting
    cmd = strcat("set PatternLighting_Indirect \"",
        ftos(autocvar_r_pattern_indirect_color_x), " ",
        ftos(autocvar_r_pattern_indirect_color_y), " ",
        ftos(autocvar_r_pattern_indirect_color_z), " ",
        ftos(autocvar_r_pattern_indirect_intensity), "\"");
    localcmd(cmd);
    
    // Ambient
    cmd = strcat("set PatternLighting_Ambient \"",
        ftos(autocvar_r_pattern_ambient_color_x), " ",
        ftos(autocvar_r_pattern_ambient_color_y), " ",
        ftos(autocvar_r_pattern_ambient_color_z), " ",
        ftos(autocvar_r_pattern_ao_strength), "\"");
    localcmd(cmd);
    
    // Config (time, pattern, speed, enabled)
    cmd = strcat("set PatternLighting_Config \"",
        ftos(time), " 0 ",
        ftos(autocvar_r_pattern_speed), " ",
        ftos(autocvar_r_pattern_lighting), "\"");
    localcmd(cmd);
    
    // SSR config
    cmd = strcat("set SSR_Config \"",
        ftos(autocvar_r_ssr_maxdist), " ",
        ftos(autocvar_r_ssr_stride), " ",
        ftos(autocvar_r_ssr_thickness), " ",
        ftos(autocvar_r_ssr_enable), "\"");
    localcmd(cmd);
    
    cmd = strcat("set SSR_Quality \"",
        ftos(autocvar_r_ssr_maxsteps), " 8 0.5 ",
        ftos(autocvar_r_ssr_fade), "\"");
    localcmd(cmd);
    
    // Post-process bloom
    cmd = strcat("set PP_Bloom \"",
        ftos(autocvar_r_pp_bloom_intensity), " ",
        ftos(autocvar_r_pp_bloom_threshold), " 0.5 ",
        ftos(autocvar_r_pp_bloom), "\"");
    localcmd(cmd);
    
    // Color grading
    cmd = strcat("set PP_ColorGrade \"",
        ftos(autocvar_r_pp_exposure), " ",
        ftos(autocvar_r_pp_contrast), " ",
        ftos(autocvar_r_pp_saturation), " ",
        ftos(autocvar_r_pp_temperature), "\"");
    localcmd(cmd);
    
    // Vignette
    cmd = strcat("set PP_Vignette \"",
        ftos(autocvar_r_pp_vignette_intensity), " 0.5 1.0 ",
        ftos(autocvar_r_pp_vignette), "\"");
    localcmd(cmd);
}

// ============================================================================
// INITIALIZATION
// ============================================================================

void PatternLighting_Init() {
    print("^2Pattern Lighting System initialized\n");
    print("^3Based on Adaptive Entity Engine\n");
    print("^7Use r_pattern_* cvars to configure\n");
    
    // Initial shader update
    UpdatePatternLightingShaderParams();
}

// Frame update
void PatternLighting_Frame() {
    // Update shader time parameter every frame
    UpdatePatternLightingShaderParams();
}
