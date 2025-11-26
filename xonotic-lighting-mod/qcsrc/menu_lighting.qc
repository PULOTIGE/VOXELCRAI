// Pattern Lighting Menu for Xonotic
// In-game settings interface
// Version 1.0.0

// ============================================================================
// MENU ITEMS
// ============================================================================

void XonoticPatternLightingSettings_draw(entity me) {
    // Draw menu background
    drawfill('0 0 0', '1 1 0', '0 0 0', 0.7, DRAWFLAG_NORMAL);
    
    float y = 0.1;
    float lineheight = 0.035;
    
    // Title
    draw_CenterText(0.5, y, "Pattern Lighting Settings", '0.04 0.04 0', '1 0.8 0.2', 1, DRAWFLAG_NORMAL);
    y += lineheight * 2;
    
    // Separator
    draw_CenterText(0.5, y, "━━━ Main Settings ━━━", '0.025 0.025 0', '0.5 0.5 0.5', 1, DRAWFLAG_NORMAL);
    y += lineheight * 1.5;
    
    // Enable toggle
    draw_Text('0.2 y 0', "Enable Pattern Lighting:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Toggle('0.6 y 0', cvar("r_pattern_lighting"), "r_pattern_lighting");
    y += lineheight;
    
    // Intensity
    draw_Text('0.2 y 0', "Intensity:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Slider('0.5 y 0', 0.3, cvar("r_pattern_intensity"), 0, 2, 0.1, "r_pattern_intensity");
    y += lineheight;
    
    // Speed
    draw_Text('0.2 y 0', "Pattern Speed:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Slider('0.5 y 0', 0.3, cvar("r_pattern_speed"), 0.1, 3, 0.1, "r_pattern_speed");
    y += lineheight * 1.5;
    
    // Separator
    draw_CenterText(0.5, y, "━━━ Direct Light ━━━", '0.025 0.025 0', '0.5 0.5 0.5', 1, DRAWFLAG_NORMAL);
    y += lineheight * 1.5;
    
    // Direct color
    draw_Text('0.2 y 0', "Direct Color R:", '0.025 0.025 0', '1 0.5 0.5', 1, DRAWFLAG_NORMAL);
    draw_Slider('0.5 y 0', 0.3, cvar("r_pattern_direct_color_x"), 0, 1, 0.05, "r_pattern_direct_color_x");
    y += lineheight;
    
    draw_Text('0.2 y 0', "Direct Color G:", '0.025 0.025 0', '0.5 1 0.5', 1, DRAWFLAG_NORMAL);
    draw_Slider('0.5 y 0', 0.3, cvar("r_pattern_direct_color_y"), 0, 1, 0.05, "r_pattern_direct_color_y");
    y += lineheight;
    
    draw_Text('0.2 y 0', "Direct Color B:", '0.025 0.025 0', '0.5 0.5 1', 1, DRAWFLAG_NORMAL);
    draw_Slider('0.5 y 0', 0.3, cvar("r_pattern_direct_color_z"), 0, 1, 0.05, "r_pattern_direct_color_z");
    y += lineheight;
    
    draw_Text('0.2 y 0', "Direct Intensity:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Slider('0.5 y 0', 0.3, cvar("r_pattern_direct_intensity"), 0, 2, 0.1, "r_pattern_direct_intensity");
    y += lineheight * 1.5;
    
    // Separator
    draw_CenterText(0.5, y, "━━━ Reflections (SSR) ━━━", '0.025 0.025 0', '0.5 0.5 0.5', 1, DRAWFLAG_NORMAL);
    y += lineheight * 1.5;
    
    // SSR toggle
    draw_Text('0.2 y 0', "Enable SSR:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Toggle('0.6 y 0', cvar("r_ssr_enable"), "r_ssr_enable");
    y += lineheight;
    
    // SSR quality
    draw_Text('0.2 y 0', "SSR Max Distance:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Slider('0.5 y 0', 0.3, cvar("r_ssr_maxdist"), 10, 500, 10, "r_ssr_maxdist");
    y += lineheight;
    
    draw_Text('0.2 y 0', "SSR Steps:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Slider('0.5 y 0', 0.3, cvar("r_ssr_maxsteps"), 16, 128, 8, "r_ssr_maxsteps");
    y += lineheight * 1.5;
    
    // Separator
    draw_CenterText(0.5, y, "━━━ Post-Processing ━━━", '0.025 0.025 0', '0.5 0.5 0.5', 1, DRAWFLAG_NORMAL);
    y += lineheight * 1.5;
    
    // Bloom
    draw_Text('0.2 y 0', "Enable Bloom:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Toggle('0.6 y 0', cvar("r_pp_bloom"), "r_pp_bloom");
    y += lineheight;
    
    draw_Text('0.2 y 0', "Bloom Intensity:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Slider('0.5 y 0', 0.3, cvar("r_pp_bloom_intensity"), 0, 1, 0.05, "r_pp_bloom_intensity");
    y += lineheight;
    
    // Exposure
    draw_Text('0.2 y 0', "Exposure:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Slider('0.5 y 0', 0.3, cvar("r_pp_exposure"), -2, 2, 0.1, "r_pp_exposure");
    y += lineheight;
    
    // Contrast
    draw_Text('0.2 y 0', "Contrast:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Slider('0.5 y 0', 0.3, cvar("r_pp_contrast"), 0.5, 2, 0.1, "r_pp_contrast");
    y += lineheight;
    
    // Saturation
    draw_Text('0.2 y 0', "Saturation:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Slider('0.5 y 0', 0.3, cvar("r_pp_saturation"), 0, 2, 0.1, "r_pp_saturation");
    y += lineheight;
    
    // Vignette
    draw_Text('0.2 y 0', "Enable Vignette:", '0.025 0.025 0', '1 1 1', 1, DRAWFLAG_NORMAL);
    draw_Toggle('0.6 y 0', cvar("r_pp_vignette"), "r_pp_vignette");
    y += lineheight * 2;
    
    // Close button
    if (draw_Button('0.4 y 0', '0.2 0.04 0', "Close", '1 1 1')) {
        me.hidden = TRUE;
    }
    
    // Preset buttons
    y += lineheight;
    if (draw_Button('0.15 y 0', '0.2 0.04 0', "Preset: Cinematic", '0.8 0.6 0.2')) {
        ApplyPreset_Cinematic();
    }
    if (draw_Button('0.4 y 0', '0.2 0.04 0', "Preset: Performance", '0.2 0.8 0.3')) {
        ApplyPreset_Performance();
    }
    if (draw_Button('0.65 y 0', '0.2 0.04 0', "Preset: Ultra", '0.3 0.3 1')) {
        ApplyPreset_Ultra();
    }
}

// ============================================================================
// PRESETS
// ============================================================================

void ApplyPreset_Cinematic() {
    cvar_set("r_pattern_lighting", "1");
    cvar_set("r_pattern_intensity", "1.2");
    cvar_set("r_ssr_enable", "1");
    cvar_set("r_ssr_maxsteps", "64");
    cvar_set("r_pp_bloom", "1");
    cvar_set("r_pp_bloom_intensity", "0.4");
    cvar_set("r_pp_contrast", "1.1");
    cvar_set("r_pp_saturation", "1.1");
    cvar_set("r_pp_vignette", "1");
    cvar_set("r_pp_vignette_intensity", "0.4");
    cvar_set("r_pp_chromatic", "1");
    cvar_set("r_pp_chromatic_intensity", "0.005");
    cvar_set("r_pp_filmgrain", "1");
    cvar_set("r_pp_filmgrain_intensity", "0.03");
    print("^2Cinematic preset applied\n");
}

void ApplyPreset_Performance() {
    cvar_set("r_pattern_lighting", "1");
    cvar_set("r_pattern_intensity", "1.0");
    cvar_set("r_ssr_enable", "0");
    cvar_set("r_pp_bloom", "0");
    cvar_set("r_pp_vignette", "0");
    cvar_set("r_pp_chromatic", "0");
    cvar_set("r_pp_filmgrain", "0");
    print("^2Performance preset applied\n");
}

void ApplyPreset_Ultra() {
    cvar_set("r_pattern_lighting", "1");
    cvar_set("r_pattern_intensity", "1.0");
    cvar_set("r_ssr_enable", "1");
    cvar_set("r_ssr_maxdist", "200");
    cvar_set("r_ssr_maxsteps", "128");
    cvar_set("r_pp_bloom", "1");
    cvar_set("r_pp_bloom_intensity", "0.3");
    cvar_set("r_pp_contrast", "1.05");
    cvar_set("r_pp_saturation", "1.05");
    cvar_set("r_pp_vignette", "1");
    cvar_set("r_pp_vignette_intensity", "0.25");
    print("^2Ultra preset applied\n");
}

// ============================================================================
// KEYBIND
// ============================================================================

void Cmd_PatternLightingMenu() {
    // Toggle menu visibility
    // This would be bound to a key
    entity menu = spawn();
    menu.classname = "patternlighting_menu";
    // Show menu logic would go here
    print("^3Pattern Lighting Settings opened\n");
    print("^7This menu requires Xonotic menu system integration\n");
}
