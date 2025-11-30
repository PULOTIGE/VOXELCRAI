struct LightUniform {
    time: f32,
    bias: f32,
    randomness: f32,
    pad: f32,
};

@group(0) @binding(0)
var<storage, read_write> pattern : array<f32>;

@group(0) @binding(1)
var<uniform> light_uniform : LightUniform;

fn lorenz(next: vec3<f32>, dt: f32) -> vec3<f32> {
    let sigma = 10.0;
    let rho = 28.0;
    let beta = 8.0 / 3.0;
    let dx = sigma * (next.y - next.x);
    let dy = next.x * (rho - next.z) - next.y;
    let dz = next.x * next.y - beta * next.z;
    return next + dt * vec3<f32>(dx, dy, dz);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
    let idx = gid.x;
    if (idx >= arrayLength(&pattern)) {
        return;
    }

    let base = sin(light_uniform.time * 0.0618 + f32(idx) * 0.0133) * 0.5 + 0.5;
    let noise = fract(sin((f32(idx) + light_uniform.randomness) * 12.9898) * 43758.5453);
    let energy = clamp(base * (0.5 + light_uniform.bias) + noise * 0.5, 0.0, 1.0);

    let start = vec3<f32>(energy, noise, light_uniform.randomness);
    let chaotic = lorenz(start, 0.016);
    pattern[idx] = clamp((chaotic.x + chaotic.y + chaotic.z) / 3.0, 0.0, 1.0);
}
