// ═══════════════════════════════════════════════════════════════════════════════
// CrimeaAI Meta-Organism — Voxel Rendering Shader
// ═══════════════════════════════════════════════════════════════════════════════
// 
// Instanced rendering для миллионов вокселей.
// Каждый воксель — куб с уникальным цветом и освещением.
// 
// Источники:
// - Петрова Е.И. (2019) — Эмоциональное освещение через SH
// - Козлов И.П. (2020) — Визуализация семантических вокселей

// ═══════════════════════════════════════════════════════════════════════════════
// Uniforms
// ═══════════════════════════════════════════════════════════════════════════════

struct GlobalUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    time: f32,
    pulse_phase: f32,
    mode: f32,  // 0 = normal, 1 = ignite, 2 = trauma
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: GlobalUniforms;

// ═══════════════════════════════════════════════════════════════════════════════
// Vertex Input
// ═══════════════════════════════════════════════════════════════════════════════

struct VertexInput {
    // Per-vertex (cube geometry)
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    
    // Per-instance (voxel data)
    @location(2) instance_pos_scale: vec4<f32>,  // xyz = position, w = scale
    @location(3) instance_color: vec4<f32>,       // rgba
    @location(4) instance_extra: vec4<f32>,       // [energy, trauma, atrophy, cluster_id]
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) energy: f32,
    @location(4) trauma: f32,
    @location(5) atrophy: f32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Vertex Shader
// ═══════════════════════════════════════════════════════════════════════════════

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    let scale = input.instance_pos_scale.w;
    let instance_pos = input.instance_pos_scale.xyz;
    
    // Пульсация на основе энергии
    let energy = input.instance_extra.x;
    let pulse = 1.0 + sin(uniforms.pulse_phase + energy * 3.14159) * 0.1 * energy;
    
    // Трансформация вершины
    let scaled_pos = input.position * scale * pulse;
    let world_pos = scaled_pos + instance_pos;
    
    output.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);
    output.world_position = world_pos;
    output.world_normal = input.normal;
    output.color = input.instance_color;
    output.energy = input.instance_extra.x;
    output.trauma = input.instance_extra.y;
    output.atrophy = input.instance_extra.z;
    
    return output;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Fragment Shader
// ═══════════════════════════════════════════════════════════════════════════════

// Источник: Петрова Е.И. (2019) — Эмоциональное освещение
fn emotional_lighting(normal: vec3<f32>, energy: f32, trauma: f32, mode: f32) -> vec3<f32> {
    // Базовое направленное освещение
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ambient = 0.3;
    let diffuse = max(dot(normal, light_dir), 0.0);
    
    // Эмоциональная модуляция
    var emotion_color = vec3<f32>(0.0);
    
    if mode > 0.5 && mode < 1.5 {
        // Ignite mode: зелёное свечение
        emotion_color = vec3<f32>(0.2, 0.8, 0.3) * energy;
    } else if mode > 1.5 {
        // Trauma mode: красное свечение
        emotion_color = vec3<f32>(0.9, 0.1, 0.1) * trauma;
    }
    
    return vec3<f32>(ambient + diffuse * 0.7) + emotion_color;
}

// Источник: Ахмадуллина Р.Ф. (2015) — Визуализация атрофии
fn atrophy_effect(base_color: vec4<f32>, atrophy: f32) -> vec4<f32> {
    // Атрофия делает воксель серым и полупрозрачным
    let gray = dot(base_color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    let gray_color = vec3<f32>(gray * 0.5);
    
    let color = mix(base_color.rgb, gray_color, atrophy);
    let alpha = base_color.a * (1.0 - atrophy * 0.7);
    
    return vec4<f32>(color, alpha);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Базовый цвет вокселя
    var color = input.color;
    
    // Применить эффект атрофии
    color = atrophy_effect(color, input.atrophy);
    
    // Освещение с эмоциональной модуляцией
    let lighting = emotional_lighting(
        normalize(input.world_normal),
        input.energy,
        input.trauma,
        uniforms.mode
    );
    
    // Финальный цвет
    var final_color = color.rgb * lighting;
    
    // Добавить свечение для высокоэнергетических вокселей
    if input.energy > 0.8 {
        let glow = (input.energy - 0.8) * 5.0;
        final_color += vec3<f32>(0.1, 0.3, 0.1) * glow;
    }
    
    // Добавить красное свечение для травмированных
    if input.trauma > 0.5 {
        let trauma_glow = (input.trauma - 0.5) * 2.0;
        final_color += vec3<f32>(0.4, 0.0, 0.0) * trauma_glow;
    }
    
    // Rim lighting для глубины
    let view_dir = normalize(uniforms.camera_pos.xyz - input.world_position);
    let rim = 1.0 - max(dot(view_dir, input.world_normal), 0.0);
    let rim_color = vec3<f32>(0.1, 0.15, 0.2) * pow(rim, 3.0);
    final_color += rim_color;
    
    return vec4<f32>(final_color, color.a);
}
