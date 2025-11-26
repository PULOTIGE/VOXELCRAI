// Mesh shader for 3D objects in editor

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = camera.view_proj * vec4<f32>(input.position, 1.0);
    output.world_pos = input.position;
    output.normal = input.normal;
    output.uv = input.uv;
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Simple lighting
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ambient = 0.3;
    let diffuse = max(dot(input.normal, light_dir), 0.0);
    let lighting = ambient + diffuse * 0.7;
    
    // Combine with vertex color
    let color = input.color.rgb * lighting;
    
    // Selection highlight would be applied here
    
    return vec4<f32>(color, input.color.a);
}

// Wireframe variant
@fragment
fn fs_wireframe(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.8, 0.8, 0.8, 1.0);
}

// Selected object highlight
@fragment  
fn fs_selected(input: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ambient = 0.3;
    let diffuse = max(dot(input.normal, light_dir), 0.0);
    let lighting = ambient + diffuse * 0.7;
    
    // Orange selection tint
    let base_color = input.color.rgb * lighting;
    let selection_color = mix(base_color, vec3<f32>(1.0, 0.6, 0.2), 0.3);
    
    return vec4<f32>(selection_color, input.color.a);
}
