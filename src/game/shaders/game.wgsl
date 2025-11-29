// VoxelStrike - Main game shader with lighting

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_position: vec4<f32>,
};

struct LightUniform {
    direction: vec4<f32>,
    color: vec4<f32>,
    ambient: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var<uniform> light: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = camera.view_proj * vec4<f32>(input.position, 1.0);
    output.world_position = input.position;
    output.world_normal = input.normal;
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Normalize vectors
    let normal = normalize(input.world_normal);
    let light_dir = normalize(-light.direction.xyz);
    let view_dir = normalize(camera.view_position.xyz - input.world_position);
    
    // Diffuse lighting (Lambert)
    let ndotl = max(dot(normal, light_dir), 0.0);
    let diffuse = ndotl * light.color.rgb;
    
    // Specular lighting (Blinn-Phong)
    let half_dir = normalize(light_dir + view_dir);
    let ndoth = max(dot(normal, half_dir), 0.0);
    let specular = pow(ndoth, 32.0) * light.color.rgb * 0.3;
    
    // Combine lighting
    let ambient = light.ambient.rgb;
    let lighting = ambient + diffuse + specular;
    
    // Apply lighting to color
    let final_color = input.color * lighting;
    
    // Simple fog for distance
    let distance = length(camera.view_position.xyz - input.world_position);
    let fog_factor = clamp((distance - 50.0) / 150.0, 0.0, 0.6);
    let fog_color = vec3<f32>(0.4, 0.6, 0.9); // Sky color
    
    let fogged_color = mix(final_color, fog_color, fog_factor);
    
    return vec4<f32>(fogged_color, 1.0);
}
