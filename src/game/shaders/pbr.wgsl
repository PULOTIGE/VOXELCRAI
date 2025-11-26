// VoxelStrike - Advanced PBR shader with shadows, reflections, and glass

// === Uniforms ===
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    view_position: vec4<f32>,
    screen_size: vec2<f32>,
    near_far: vec2<f32>,
};

struct LightingUniform {
    sun_direction: vec4<f32>,
    sun_color: vec4<f32>,
    ambient_color: vec4<f32>,
    view_position: vec4<f32>,
    exposure: f32,
    time: f32,
    shadow_bias: f32,
    gamma: f32,
};

struct ShadowUniform {
    light_space_matrix: mat4x4<f32>,
    cascade_splits: vec4<f32>,
    shadow_params: vec4<f32>,
};

struct MaterialUniform {
    albedo: vec4<f32>,
    metallic_roughness: vec4<f32>,
    emission: vec4<f32>,
    params: vec4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<uniform> lighting: LightingUniform;
@group(0) @binding(2) var<uniform> shadow: ShadowUniform;

@group(1) @binding(0) var albedo_texture: texture_2d<f32>;
@group(1) @binding(1) var normal_texture: texture_2d<f32>;
@group(1) @binding(2) var roughness_texture: texture_2d<f32>;
@group(1) @binding(3) var ao_texture: texture_2d<f32>;
@group(1) @binding(4) var shadow_map: texture_depth_2d;
@group(1) @binding(5) var reflection_map: texture_cube<f32>;
@group(1) @binding(6) var texture_sampler: sampler;
@group(1) @binding(7) var shadow_sampler: sampler_comparison;

// === Vertex Shader ===
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
    @location(2) world_tangent: vec3<f32>,
    @location(3) world_bitangent: vec3<f32>,
    @location(4) uv: vec2<f32>,
    @location(5) color: vec4<f32>,
    @location(6) shadow_coord: vec4<f32>,
    @location(7) view_depth: f32,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    let world_pos = vec4<f32>(input.position, 1.0);
    output.clip_position = camera.view_proj * world_pos;
    output.world_position = input.position;
    output.world_normal = normalize(input.normal);
    output.world_tangent = normalize(input.tangent.xyz);
    output.world_bitangent = cross(output.world_normal, output.world_tangent) * input.tangent.w;
    output.uv = input.uv;
    output.color = input.color;
    
    // Shadow coordinates
    let light_space_pos = shadow.light_space_matrix * world_pos;
    output.shadow_coord = light_space_pos;
    
    // View depth for cascade selection
    let view_pos = camera.view * world_pos;
    output.view_depth = -view_pos.z;
    
    return output;
}

// === Fragment Shader ===

const PI: f32 = 3.14159265359;

// Fresnel-Schlick approximation
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(1.0 - cos_theta, 5.0);
}

// Fresnel with roughness
fn fresnel_schlick_roughness(cos_theta: f32, f0: vec3<f32>, roughness: f32) -> vec3<f32> {
    return f0 + (max(vec3<f32>(1.0 - roughness), f0) - f0) * pow(1.0 - cos_theta, 5.0);
}

// GGX Distribution
fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

// Smith's geometry function
fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    
    let ggx_v = n_dot_v / (n_dot_v * (1.0 - k) + k);
    let ggx_l = n_dot_l / (n_dot_l * (1.0 - k) + k);
    
    return ggx_v * ggx_l;
}

// PCF Shadow sampling
fn sample_shadow_pcf(shadow_coord: vec4<f32>, bias: f32) -> f32 {
    let proj_coord = shadow_coord.xyz / shadow_coord.w;
    
    // Transform to [0, 1] UV space
    let uv = proj_coord.xy * 0.5 + 0.5;
    let depth = proj_coord.z - bias;
    
    // Check bounds
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 || depth > 1.0) {
        return 1.0;
    }
    
    // 3x3 PCF
    var shadow = 0.0;
    let texel_size = 1.0 / 2048.0;
    
    for (var x: i32 = -1; x <= 1; x++) {
        for (var y: i32 = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            shadow += textureSampleCompare(shadow_map, shadow_sampler, uv + offset, depth);
        }
    }
    
    return shadow / 9.0;
}

// Tone mapping (ACES)
fn aces_tonemap(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return saturate((color * (a * color + b)) / (color * (c * color + d) + e));
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample textures
    let albedo = input.color.rgb;
    let metallic = 0.0;
    let roughness = 0.7;
    let ao = 1.0;
    
    // Vectors
    let n = normalize(input.world_normal);
    let v = normalize(camera.view_position.xyz - input.world_position);
    let l = normalize(-lighting.sun_direction.xyz);
    let h = normalize(v + l);
    
    // Dot products
    let n_dot_v = max(dot(n, v), 0.001);
    let n_dot_l = max(dot(n, l), 0.0);
    let n_dot_h = max(dot(n, h), 0.0);
    let h_dot_v = max(dot(h, v), 0.0);
    
    // F0 (reflectance at normal incidence)
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);
    
    // Cook-Torrance BRDF
    let d = distribution_ggx(n_dot_h, roughness);
    let g = geometry_smith(n_dot_v, n_dot_l, roughness);
    let f = fresnel_schlick(h_dot_v, f0);
    
    let numerator = d * g * f;
    let denominator = 4.0 * n_dot_v * n_dot_l + 0.001;
    let specular = numerator / denominator;
    
    // Diffuse (energy conservation)
    let k_s = f;
    let k_d = (1.0 - k_s) * (1.0 - metallic);
    
    // Direct lighting
    let light_color = lighting.sun_color.rgb * lighting.sun_color.w;
    let direct = (k_d * albedo / PI + specular) * light_color * n_dot_l;
    
    // Shadow
    let shadow = sample_shadow_pcf(input.shadow_coord, lighting.shadow_bias);
    
    // Ambient
    let ambient = lighting.ambient_color.rgb * albedo * ao;
    
    // Combine
    var color = ambient + direct * shadow;
    
    // Simple reflection for metallic surfaces
    if (metallic > 0.5) {
        let reflect_dir = reflect(-v, n);
        // Approximate reflection with sky color
        let sky_factor = max(reflect_dir.y, 0.0);
        let reflection = mix(vec3<f32>(0.3, 0.35, 0.4), vec3<f32>(0.5, 0.7, 1.0), sky_factor);
        color = mix(color, reflection * f, metallic * (1.0 - roughness));
    }
    
    // Emission (for lights, muzzle flash, etc.)
    // color += emission * emission_strength;
    
    // Exposure and tone mapping
    color *= lighting.exposure;
    color = aces_tonemap(color);
    
    // Gamma correction
    color = pow(color, vec3<f32>(1.0 / lighting.gamma));
    
    return vec4<f32>(color, 1.0);
}

// === Glass Fragment Shader (alternative entry point) ===
@fragment
fn fs_glass(input: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(input.world_normal);
    let v = normalize(camera.view_position.xyz - input.world_position);
    
    let n_dot_v = max(dot(n, v), 0.0);
    
    // Fresnel for glass
    let f0 = vec3<f32>(0.04);
    let fresnel = fresnel_schlick(n_dot_v, f0);
    
    // Reflection
    let reflect_dir = reflect(-v, n);
    let sky_factor = max(reflect_dir.y, 0.0);
    let reflection = mix(vec3<f32>(0.4, 0.45, 0.5), vec3<f32>(0.6, 0.8, 1.0), sky_factor);
    
    // Glass tint
    let tint = input.color.rgb;
    
    // Combine reflection and tint
    let color = mix(tint * 0.1, reflection, fresnel.r);
    
    // Transparency based on fresnel
    let alpha = 0.3 + fresnel.r * 0.6;
    
    return vec4<f32>(color, alpha);
}
