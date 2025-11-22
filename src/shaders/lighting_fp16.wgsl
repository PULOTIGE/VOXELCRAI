// FP16 оптимизированный шейдер для Radeon VII
// Использует Rapid Packed Math (RPM) для 2x производительности

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    padding: f32,
}

// FP16 оптимизированные данные
struct LightPatternFP16 {
    direct_light: vec3<f16>,      // FP16 для RPM
    indirect_light: vec3<f16>,
    ambient: vec3<f16>,
    intensity: f16,
    shadow_data: array<i8, 32>,
    light_rays: array<i8, 64>,
    rain_pattern: array<i8, 128>,
    light_texture: array<i8, 256>,
    sh_coefficients: array<i8, 64>,
}

struct PBRMaterial {
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    ao: f32,
    emission: vec3<f32>,
    padding: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<uniform> material: PBRMaterial;
@group(0) @binding(2) var<uniform> light_pattern: LightPatternFP16;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_position = model.position;
    out.normal = model.normal;
    out.uv = model.uv;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.normal);
    let V = normalize(camera.position - in.world_position);
    
    // Используем FP16 данные напрямую (RPM на Vega 20)
    let direct_light = vec3<f32>(light_pattern.direct_light);
    let indirect_light = vec3<f32>(light_pattern.indirect_light);
    let ambient = vec3<f32>(light_pattern.ambient);
    let intensity = f32(light_pattern.intensity);
    
    // Простое освещение с FP16 данными
    let lighting = direct_light * intensity + indirect_light * 0.5 + ambient * 0.3;
    
    // Sample shadow (INT8 данные)
    let shadow_idx = (in.uv.x * 32.0) as u32;
    let shadow = f32(light_pattern.shadow_data[shadow_idx]) / 127.0;
    
    // Sample light rays (INT8 данные)
    let ray_idx = (in.uv.y * 64.0) as u32;
    let rays = f32(light_pattern.light_rays[ray_idx]) / 127.0;
    
    // Sample texture (INT8 данные)
    let tex_idx = ((in.uv.x + in.uv.y) * 128.0) as u32;
    let texture = f32(light_pattern.light_texture[tex_idx]) / 127.0;
    
    // Combine
    let final_lighting = lighting * shadow * (1.0 + rays * 0.2) * (1.0 + texture * 0.1);
    let color = material.albedo * final_lighting + material.emission;
    
    return vec4<f32>(color, 1.0);
}
