// VoxelStrike - First-person weapon shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    view_position: vec4<f32>,
    screen_size: vec2<f32>,
    near_far: vec2<f32>,
};

struct WeaponUniform {
    model: mat4x4<f32>,
    color_tint: vec4<f32>,
    params: vec4<f32>, // x = metallic, y = roughness, z = emission, w = time
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<uniform> weapon: WeaponUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Transform weapon vertex
    let world_pos = weapon.model * vec4<f32>(input.position, 1.0);
    
    // For first-person weapons, we use a separate projection with no view matrix
    // The weapon model matrix already includes view-space offset
    output.clip_position = camera.proj * world_pos;
    output.world_position = world_pos.xyz;
    output.world_normal = normalize((weapon.model * vec4<f32>(input.normal, 0.0)).xyz);
    output.uv = input.uv;
    output.color = input.color * weapon.color_tint;
    
    return output;
}

const PI: f32 = 3.14159265359;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = input.color.rgb;
    let metallic = weapon.params.x;
    let roughness = weapon.params.y;
    
    let n = normalize(input.world_normal);
    let v = normalize(-input.world_position); // View direction in view space
    let l = normalize(vec3<f32>(0.5, 0.8, 0.3)); // Fixed light for weapon
    let h = normalize(v + l);
    
    let n_dot_v = max(dot(n, v), 0.001);
    let n_dot_l = max(dot(n, l), 0.0);
    let n_dot_h = max(dot(n, h), 0.0);
    
    // Simplified PBR
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);
    let fresnel = f0 + (1.0 - f0) * pow(1.0 - n_dot_v, 5.0);
    
    // GGX
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    let d = a2 / (PI * denom * denom);
    
    // Specular
    let specular = fresnel * d * 0.25;
    
    // Diffuse
    let k_d = (1.0 - fresnel) * (1.0 - metallic);
    let diffuse = k_d * albedo / PI;
    
    // Light
    let light_color = vec3<f32>(1.0, 0.95, 0.9);
    let direct = (diffuse + specular) * light_color * n_dot_l;
    
    // Ambient
    let ambient = albedo * vec3<f32>(0.15, 0.15, 0.18);
    
    // Rim light for visibility
    let rim = pow(1.0 - n_dot_v, 3.0) * 0.15;
    
    var color = ambient + direct + vec3<f32>(rim);
    
    // Emission (muzzle flash)
    if (weapon.params.z > 0.0) {
        color += vec3<f32>(1.0, 0.8, 0.3) * weapon.params.z;
    }
    
    // Tone mapping
    color = color / (color + vec3<f32>(1.0));
    color = pow(color, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(color, 1.0);
}
