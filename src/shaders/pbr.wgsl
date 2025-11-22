// PBR (Physically Based Rendering) shader
// Supports metallic-roughness workflow

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

struct PBRMaterial {
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    ao: f32,
    emission: vec3<f32>,
    padding: f32,
}

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
    padding: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<uniform> material: PBRMaterial;
@group(0) @binding(2) var<uniform> light: Light;

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
    let L = normalize(light.position - in.world_position);
    let H = normalize(V + L);

    // Calculate lighting
    let NdotL = max(dot(N, L), 0.0);
    let NdotV = max(dot(N, V), 0.0);
    let NdotH = max(dot(N, H), 0.0);
    let VdotH = max(dot(V, H), 0.0);

    // Fresnel-Schlick approximation
    let F0 = mix(vec3<f32>(0.04), material.albedo, material.metallic);
    let F = F0 + (1.0 - F0) * pow(1.0 - VdotH, 5.0);

    // Normal Distribution Function (GGX/Trowbridge-Reitz)
    let alpha = material.roughness * material.roughness;
    let alpha2 = alpha * alpha;
    let denom = (NdotH * NdotH * (alpha2 - 1.0) + 1.0);
    let D = alpha2 / (3.14159 * denom * denom);

    // Geometry function (Schlick-GGX)
    let k = (material.roughness + 1.0) * (material.roughness + 1.0) / 8.0;
    let G1_L = NdotL / (NdotL * (1.0 - k) + k);
    let G1_V = NdotV / (NdotV * (1.0 - k) + k);
    let G = G1_L * G1_V;

    // Cook-Torrance BRDF
    let specular = (D * G * F) / (4.0 * NdotV * NdotL + 0.001);

    // Diffuse (Lambertian)
    let kS = F;
    let kD = (1.0 - kS) * (1.0 - material.metallic);
    let diffuse = kD * material.albedo / 3.14159;

    // Combine
    let radiance = light.color * light.intensity;
    let Lo = (diffuse + specular) * radiance * NdotL;

    // Ambient
    let ambient = material.albedo * material.ao * 0.03;

    // Emission
    let emission = material.emission;

    let color = ambient + Lo + emission;
    return vec4<f32>(color, 1.0);
}
