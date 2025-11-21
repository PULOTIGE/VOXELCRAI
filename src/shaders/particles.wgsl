// GPU Compute shader for particle system
// Updates particle positions, velocities, and collisions

struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    color: vec4<f32>,
    life: f32,
    size: f32,
    padding: vec2<f32>,
}

struct ParticleParams {
    delta_time: f32,
    gravity: f32,
    damping: f32,
    padding: f32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: ParticleParams;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&particles)) {
        return;
    }

    var particle = particles[index];

    // Update velocity with gravity
    particle.velocity.y += params.gravity * params.delta_time;

    // Apply damping
    particle.velocity *= params.damping;

    // Update position
    particle.position += particle.velocity * params.delta_time;

    // Update life
    particle.life -= params.delta_time * 0.1;

    // Reset particle if dead or out of bounds
    if (particle.life <= 0.0 || particle.position.y < -10.0) {
        // Reset to spawn position
        let angle = f32(index) * 0.1;
        particle.position = vec3<f32>(
            cos(angle) * 5.0,
            10.0,
            sin(angle) * 5.0
        );
        particle.velocity = vec3<f32>(0.0, -1.0, 0.0);
        particle.life = 1.0;
    }

    // Simple boundary collision
    if (particle.position.x < -50.0 || particle.position.x > 50.0) {
        particle.velocity.x *= -0.8;
    }
    if (particle.position.z < -50.0 || particle.position.z > 50.0) {
        particle.velocity.z *= -0.8;
    }

    particles[index] = particle;
}
