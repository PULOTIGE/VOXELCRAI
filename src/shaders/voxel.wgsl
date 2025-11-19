struct Camera {
    view_proj: mat4x4<f32>;
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VsIn {
    @location(0) position: vec3<f32>;
    @location(1) normal: vec3<f32>;
    @location(2) instance_pos_scale: vec4<f32>;
    @location(3) instance_color_energy: vec4<f32>;
};

struct VsOut {
    @builtin(position) clip_position: vec4<f32>;
    @location(0) normal: vec3<f32>;
    @location(1) color: vec3<f32>;
    @location(2) energy: f32;
};

@vertex
fn vs_main(input: VsIn) -> VsOut {
    let scale = input.instance_pos_scale.w;
    let world_pos = input.instance_pos_scale.xyz + input.position * scale;

    var out: VsOut;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.normal = normalize(input.normal);
    out.color = input.instance_color_energy.xyz;
    out.energy = input.instance_color_energy.w;
    return out;
}

@fragment
fn fs_main(input: VsOut) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.32, 0.74, 0.58));
    let intensity = max(dot(normalize(input.normal), light_dir), 0.15);
    let glow = input.energy * 0.05;
    let base_color = input.color * intensity + vec3<f32>(glow, glow * 0.5, glow * 0.2);
    return vec4<f32>(base_color, 1.0);
}
