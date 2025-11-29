// VoxelCraft - Sky Shader with Day/Night Cycle

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

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) ray_dir: vec3<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Full-screen quad
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );
    
    var out: VertexOutput;
    let pos = positions[vertex_index];
    out.clip_position = vec4<f32>(pos, 0.9999, 1.0);
    
    // Calculate ray direction (inverse view-projection)
    // Simplified: just use position for sky gradient
    out.ray_dir = normalize(vec3<f32>(pos.x, pos.y * 0.5 + 0.5, -1.0));
    
    return out;
}

// Hash function for stars
fn hash(p: vec3<f32>) -> f32 {
    let h = dot(p, vec3<f32>(127.1, 311.7, 74.7));
    return fract(sin(h) * 43758.5453123);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ray_dir = normalize(in.ray_dir);
    let y = ray_dir.y;
    
    // Time of day (0-1, where 0.5 is noon)
    let time_of_day = light.time;
    let is_day = light.ambient > 0.3;
    
    // Sky gradient
    var sky_bottom: vec3<f32>;
    var sky_top: vec3<f32>;
    
    if is_day {
        // Day sky
        sky_bottom = vec3<f32>(0.6, 0.8, 1.0);
        sky_top = vec3<f32>(0.2, 0.4, 0.9);
        
        // Sunset/sunrise colors
        let sun_height = light.sun_direction.y;
        if sun_height < 0.3 {
            let sunset_factor = 1.0 - sun_height / 0.3;
            sky_bottom = mix(sky_bottom, vec3<f32>(1.0, 0.5, 0.2), sunset_factor * 0.7);
            sky_top = mix(sky_top, vec3<f32>(0.8, 0.3, 0.4), sunset_factor * 0.5);
        }
    } else {
        // Night sky
        sky_bottom = vec3<f32>(0.02, 0.02, 0.05);
        sky_top = vec3<f32>(0.0, 0.0, 0.02);
    }
    
    // Gradient based on vertical position
    let t = clamp(y * 2.0, 0.0, 1.0);
    var sky_color = mix(sky_bottom, sky_top, t);
    
    // Sun/Moon
    let sun_dot = max(dot(ray_dir, light.sun_direction), 0.0);
    
    if is_day {
        // Sun
        let sun_disc = smoothstep(0.997, 0.999, sun_dot);
        let sun_glow = pow(sun_dot, 32.0) * 0.5;
        let sun_color = vec3<f32>(1.0, 0.95, 0.8);
        sky_color += sun_color * (sun_disc + sun_glow);
    } else {
        // Moon (opposite direction)
        let moon_dir = -light.sun_direction;
        let moon_dot = max(dot(ray_dir, moon_dir), 0.0);
        let moon_disc = smoothstep(0.995, 0.998, moon_dot);
        let moon_glow = pow(moon_dot, 64.0) * 0.2;
        let moon_color = vec3<f32>(0.9, 0.9, 1.0);
        sky_color += moon_color * (moon_disc + moon_glow);
        
        // Stars
        let star_pos = floor(ray_dir * 200.0);
        let star_brightness = hash(star_pos);
        let twinkle = sin(light.time * 3.0 + star_brightness * 100.0) * 0.5 + 0.5;
        
        if star_brightness > 0.98 && y > 0.1 {
            let star_intensity = (star_brightness - 0.98) * 50.0 * twinkle;
            sky_color += vec3<f32>(1.0, 1.0, 1.0) * star_intensity;
        }
    }
    
    // Clouds (simple layer)
    let cloud_scale = 0.01;
    let cloud_offset = light.time * 0.01;
    let cloud_pos = ray_dir.xz / (ray_dir.y + 0.3) * 10.0 + vec2<f32>(cloud_offset, 0.0);
    
    // Simple cloud noise
    let cloud_noise = sin(cloud_pos.x * 3.0) * sin(cloud_pos.y * 3.0) * 0.5 + 0.5;
    let cloud_density = smoothstep(0.4, 0.6, cloud_noise) * smoothstep(0.0, 0.3, y);
    
    if is_day {
        let cloud_color = vec3<f32>(1.0, 1.0, 1.0);
        sky_color = mix(sky_color, cloud_color, cloud_density * 0.5);
    }
    
    return vec4<f32>(sky_color, 1.0);
}
