// VoxelCraft - Water Shader with Reflections and Patterns

struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
}

struct LightUniform {
    sun_direction: vec3<f32>,
    _padding1: f32,
    ambient: f32,
    time: f32,
    _padding2: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> light: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) ao: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) view_dir: vec3<f32>,
}

// Gerstner wave function
fn gerstner_wave(pos: vec2<f32>, time: f32, wavelength: f32, amplitude: f32, direction: vec2<f32>) -> vec3<f32> {
    let k = 2.0 * 3.14159 / wavelength;
    let c = sqrt(9.81 / k);
    let d = normalize(direction);
    let f = k * (dot(d, pos) - c * time);
    let a = amplitude;
    
    return vec3<f32>(
        d.x * a * cos(f),
        a * sin(f),
        d.y * a * cos(f)
    );
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Apply waves
    var pos = in.position;
    let wave_pos = pos.xz;
    
    // Multiple wave layers
    let wave1 = gerstner_wave(wave_pos, light.time, 8.0, 0.15, vec2<f32>(1.0, 0.0));
    let wave2 = gerstner_wave(wave_pos, light.time * 1.1, 5.0, 0.1, vec2<f32>(0.7, 0.7));
    let wave3 = gerstner_wave(wave_pos, light.time * 0.9, 3.0, 0.05, vec2<f32>(-0.5, 0.8));
    
    let wave_offset = wave1 + wave2 + wave3;
    pos.y += wave_offset.y;
    
    out.world_pos = pos;
    out.clip_position = camera.view_proj * vec4<f32>(pos, 1.0);
    
    // Calculate wave normal
    let dx = wave1.x + wave2.x + wave3.x;
    let dz = wave1.z + wave2.z + wave3.z;
    out.normal = normalize(vec3<f32>(-dx, 1.0, -dz));
    
    out.uv = in.uv;
    out.view_dir = normalize(camera.position - pos);
    
    return out;
}

// Simple reflection calculation
fn fresnel(view_dir: vec3<f32>, normal: vec3<f32>) -> f32 {
    let f0 = 0.02;
    let cos_theta = max(dot(view_dir, normal), 0.0);
    return f0 + (1.0 - f0) * pow(1.0 - cos_theta, 5.0);
}

// Pattern caustics
fn caustics(pos: vec2<f32>, time: f32) -> f32 {
    let scale = 0.5;
    let p = pos * scale;
    
    var caustic = 0.0;
    for (var i = 1; i < 4; i++) {
        let fi = f32(i);
        let x = sin(p.x * fi + time) * cos(p.y * fi * 1.2 - time * 0.7);
        let y = cos(p.x * fi * 0.9 - time * 0.8) * sin(p.y * fi + time * 0.6);
        caustic += (x + y) * 0.5 / fi;
    }
    
    return caustic * 0.5 + 0.5;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Water colors
    let deep_color = vec3<f32>(0.05, 0.15, 0.3);
    let shallow_color = vec3<f32>(0.1, 0.4, 0.5);
    
    // Depth-based color (simplified)
    let depth = (in.world_pos.y - 25.0) / 10.0;
    let water_color = mix(deep_color, shallow_color, clamp(depth, 0.0, 1.0));
    
    // Fresnel reflection
    let fresnel_factor = fresnel(in.view_dir, in.normal);
    
    // Sky reflection color
    let reflect_dir = reflect(-in.view_dir, in.normal);
    let sky_gradient = reflect_dir.y * 0.5 + 0.5;
    let sky_color = mix(vec3<f32>(0.6, 0.75, 0.9), vec3<f32>(0.3, 0.5, 0.9), sky_gradient);
    
    // Sun reflection
    let sun_reflect = pow(max(dot(reflect_dir, light.sun_direction), 0.0), 256.0);
    let sun_color = vec3<f32>(1.0, 0.95, 0.8) * sun_reflect * 2.0;
    
    // Caustics pattern
    let caustic_value = caustics(in.world_pos.xz, light.time);
    let caustic_color = vec3<f32>(0.2, 0.4, 0.5) * caustic_value * 0.3;
    
    // Wave foam at peaks
    let wave_height = sin(in.world_pos.x * 0.5 + light.time) * sin(in.world_pos.z * 0.5 - light.time * 0.8);
    let foam = smoothstep(0.7, 1.0, wave_height) * 0.3;
    let foam_color = vec3<f32>(0.9, 0.95, 1.0) * foam;
    
    // Combine
    var final_color = mix(water_color + caustic_color, sky_color, fresnel_factor * 0.6);
    final_color += sun_color;
    final_color += foam_color;
    
    // Lighting
    let ndotl = max(dot(in.normal, light.sun_direction), 0.0);
    final_color *= (light.ambient + ndotl * 0.5);
    
    // Fog
    let dist = length(camera.position - in.world_pos);
    let fog_factor = 1.0 - exp(-dist * 0.003);
    let fog_color = vec3<f32>(0.6, 0.75, 0.9);
    final_color = mix(final_color, fog_color, fog_factor);
    
    // Transparency based on depth and fresnel
    let alpha = 0.7 + fresnel_factor * 0.2;
    
    return vec4<f32>(final_color, alpha);
}
