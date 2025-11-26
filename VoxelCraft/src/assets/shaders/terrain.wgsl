// VoxelCraft - Terrain Shader with Pattern Lighting

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
    @location(3) ao: f32,
    @location(4) view_dir: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    out.world_pos = in.position;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.normal = in.normal;
    out.uv = in.uv;
    out.ao = in.ao;
    out.view_dir = normalize(camera.position - in.position);
    
    return out;
}

// Pattern-based lighting evaluation
fn evaluate_pattern(pattern_type: i32, time: f32, world_pos: vec3<f32>) -> f32 {
    let t = time + dot(world_pos, vec3<f32>(0.1, 0.2, 0.15));
    
    switch pattern_type {
        case 0: { // Steady
            return 1.0;
        }
        case 1: { // Pulse
            return 0.5 + 0.5 * sin(t * 2.0);
        }
        case 2: { // Flicker
            let noise = fract(sin(dot(vec2<f32>(t, t * 0.7), vec2<f32>(12.9898, 78.233))) * 43758.5453);
            return 0.7 + 0.3 * noise;
        }
        case 3: { // Fire
            let n1 = sin(t * 3.0) * 0.5 + 0.5;
            let n2 = sin(t * 7.0 + 1.5) * 0.5 + 0.5;
            return 0.6 + 0.4 * n1 * n2;
        }
        default: {
            return 1.0;
        }
    }
}

// Procedural texture based on UV and world position
fn procedural_texture(uv: vec2<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    // Determine block type from UV atlas position
    let atlas_pos = floor(uv * 16.0);
    
    // Grass
    if atlas_pos.x == 0.0 && atlas_pos.y == 0.0 {
        let noise = fract(sin(dot(world_pos.xz, vec2<f32>(12.9898, 78.233))) * 43758.5453);
        return mix(vec3<f32>(0.2, 0.5, 0.15), vec3<f32>(0.3, 0.6, 0.2), noise);
    }
    // Stone
    if atlas_pos.x == 1.0 && atlas_pos.y == 0.0 {
        let noise = fract(sin(dot(world_pos, vec3<f32>(12.9898, 78.233, 45.678))) * 43758.5453);
        return mix(vec3<f32>(0.4, 0.4, 0.4), vec3<f32>(0.5, 0.5, 0.5), noise);
    }
    // Dirt
    if atlas_pos.x == 2.0 && atlas_pos.y == 0.0 {
        let noise = fract(sin(dot(world_pos.xz * 2.0, vec2<f32>(12.9898, 78.233))) * 43758.5453);
        return mix(vec3<f32>(0.4, 0.25, 0.1), vec3<f32>(0.5, 0.35, 0.15), noise);
    }
    // Wood
    if atlas_pos.x == 4.0 && atlas_pos.y == 1.0 {
        let ring = sin(length(world_pos.xz) * 10.0) * 0.5 + 0.5;
        return mix(vec3<f32>(0.4, 0.25, 0.1), vec3<f32>(0.55, 0.35, 0.15), ring);
    }
    // Sand
    if atlas_pos.x == 2.0 && atlas_pos.y == 1.0 {
        let noise = fract(sin(dot(world_pos.xz, vec2<f32>(12.9898, 78.233))) * 43758.5453);
        return mix(vec3<f32>(0.85, 0.8, 0.55), vec3<f32>(0.9, 0.85, 0.6), noise);
    }
    // Leaves
    if atlas_pos.x == 4.0 && atlas_pos.y == 3.0 {
        let noise = fract(sin(dot(world_pos, vec3<f32>(12.9898, 78.233, 45.678))) * 43758.5453);
        return mix(vec3<f32>(0.1, 0.4, 0.1), vec3<f32>(0.2, 0.5, 0.15), noise);
    }
    // Diamond ore
    if atlas_pos.x == 2.0 && atlas_pos.y == 3.0 {
        let sparkle = sin(light.time * 5.0 + world_pos.x * 10.0) * 0.5 + 0.5;
        return mix(vec3<f32>(0.3, 0.4, 0.5), vec3<f32>(0.5, 0.8, 0.9), sparkle * 0.3);
    }
    // Coal ore
    if atlas_pos.x == 2.0 && atlas_pos.y == 2.0 {
        let noise = fract(sin(dot(world_pos, vec3<f32>(12.9898, 78.233, 45.678))) * 43758.5453);
        return mix(vec3<f32>(0.15, 0.15, 0.15), vec3<f32>(0.3, 0.3, 0.3), noise);
    }
    // Planks
    if atlas_pos.x == 4.0 && atlas_pos.y == 0.0 {
        let grain = sin(world_pos.z * 20.0) * 0.1;
        return vec3<f32>(0.6 + grain, 0.4 + grain * 0.5, 0.2);
    }

    // Default gray
    return vec3<f32>(0.5, 0.5, 0.5);
}

// Shadow calculation with soft edges
fn calculate_shadow(world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    let ndotl = max(dot(normal, light.sun_direction), 0.0);
    return ndotl;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get procedural texture color
    let base_color = procedural_texture(in.uv, in.world_pos);
    
    // Pattern lighting
    let pattern_value = evaluate_pattern(3, light.time, in.world_pos);
    
    // Diffuse lighting
    let ndotl = max(dot(in.normal, light.sun_direction), 0.0);
    let diffuse = ndotl * pattern_value;
    
    // Specular (subtle)
    let half_dir = normalize(light.sun_direction + in.view_dir);
    let spec = pow(max(dot(in.normal, half_dir), 0.0), 32.0) * 0.3;
    
    // Ambient occlusion
    let ao = in.ao * in.ao;
    
    // Combine lighting
    let ambient_color = base_color * light.ambient;
    let diffuse_color = base_color * diffuse * 0.8;
    let specular = vec3<f32>(spec, spec, spec);
    
    var final_color = (ambient_color + diffuse_color + specular) * ao;
    
    // Fog
    let dist = length(camera.position - in.world_pos);
    let fog_factor = 1.0 - exp(-dist * 0.005);
    let fog_color = vec3<f32>(0.6, 0.75, 0.9);
    final_color = mix(final_color, fog_color, fog_factor);
    
    return vec4<f32>(final_color, 1.0);
}
