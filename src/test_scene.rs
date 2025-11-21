// Test scene generator for 4K benchmarking
// Creates scenes optimized for RTX 4070 equivalent performance testing
use glam::Vec3;
use bevy_ecs::prelude::*;
use crate::agents::Agent;
use crate::scene::{SceneManager, ScenePattern, ObjectType, SceneObject};
use crate::particles::Particle;
use crate::pbr::PBRMaterial;

/// Test scene configuration for 4K benchmarking
pub struct TestScene4K {
    pub scene_manager: SceneManager,
    pub agent_count: usize,
    pub particle_count: usize,
}

impl TestScene4K {
    /// Create test scene optimized for RTX 4070 4K performance
    pub fn create_rtx4070_4k_scene(
        world: &mut World,
        agent_count: usize,
        particle_count: usize,
    ) -> Self {
        // Create dense scene
        let mut scene_manager = SceneManager::new(ScenePattern::Dense);

        // Add extra objects for stress testing
        Self::add_stress_objects(&mut scene_manager, world);

        // Spawn agents in a grid pattern
        Self::spawn_agents_grid(world, agent_count);

        Self {
            scene_manager,
            agent_count,
            particle_count,
        }
    }

    /// Add stress test objects (complex geometry)
    fn add_stress_objects(scene_manager: &mut SceneManager, world: &mut World) {
        // Create a large platform structure
        for x in -20..=20 {
            for z in -20..=20 {
                if (x + z) % 3 == 0 {
                    let entity = world.spawn(SceneObject::new(
                        ObjectType::Platform,
                        Vec3::new(x as f32 * 2.0, 0.0, z as f32 * 2.0),
                    )).id();
                    scene_manager.objects.push(entity);
                }
            }
        }

        // Add stairs in a pattern
        for i in 0..10 {
            let entity = world.spawn(SceneObject::new(
                ObjectType::Stairs,
                Vec3::new(i as f32 * 5.0, 1.0, 0.0),
            )).id();
            scene_manager.objects.push(entity);
        }

        // Add slide ramps
        for i in 0..8 {
            let entity = world.spawn(SceneObject::new(
                ObjectType::SlideRamp,
                Vec3::new(i as f32 * 6.0, 2.0, 5.0),
            )).id();
            scene_manager.objects.push(entity);
        }

        // Add toys scattered around
        for i in 0..50 {
            let angle = (i as f32) * 0.5;
            let radius = 15.0 + (i as f32) * 0.3;
            let entity = world.spawn(SceneObject::new(
                ObjectType::Toy,
                Vec3::new(
                    angle.cos() * radius,
                    1.0,
                    angle.sin() * radius,
                ),
            )).id();
            scene_manager.objects.push(entity);
        }
    }

    /// Spawn agents in a grid pattern
    fn spawn_agents_grid(world: &mut World, count: usize) {
        let grid_size = (count as f32).sqrt().ceil() as usize;
        let spacing = 3.0;

        for i in 0..count {
            let x = (i % grid_size) as f32 * spacing - (grid_size as f32 * spacing / 2.0);
            let z = (i / grid_size) as f32 * spacing - (grid_size as f32 * spacing / 2.0);
            
            let mut agent = Agent::new(Vec3::new(x, 2.0, z));
            
            // Vary agent behaviors
            match i % 4 {
                0 => agent.state = crate::agents::AgentState::Wander,
                1 => agent.state = crate::agents::AgentState::Follow,
                2 => agent.state = crate::agents::AgentState::Interact,
                _ => agent.state = crate::agents::AgentState::Idle,
            }

            world.spawn(agent);
        }
    }

    /// Generate initial particles for the scene
    pub fn generate_particles(count: usize) -> Vec<Particle> {
        let mut particles = Vec::with_capacity(count);
        
        for i in 0..count {
            let angle = (i as f32) * 0.01;
            let height = 10.0 + (i as f32 % 100.0) * 0.1;
            let radius = 20.0 + (i as f32 % 50.0) * 0.2;
            
            let position = glam::Vec3::new(
                angle.cos() * radius,
                height,
                angle.sin() * radius,
            );
            
            let velocity = glam::Vec3::new(
                (angle * 2.0).sin() * 0.5,
                -1.0 - (i as f32 % 10.0) * 0.1,
                (angle * 2.0).cos() * 0.5,
            );
            
            // Vary colors
            let color_idx = (i % 5) as f32 / 5.0;
            let color = glam::Vec4::new(
                0.5 + color_idx * 0.5,
                0.3 + (1.0 - color_idx) * 0.5,
                0.2 + color_idx * 0.3,
                1.0,
            );
            
            particles.push(Particle::new(position, velocity, color));
        }
        
        particles
    }
}

/// Scene complexity presets
pub enum SceneComplexity {
    Light,    // ~1M particles, 2K agents
    Medium,   // ~2M particles, 5K agents (RTX 4070 target)
    Heavy,    // ~4M particles, 10K agents
    Extreme,  // ~6M particles, 20K agents
}

impl SceneComplexity {
    pub fn particle_count(&self) -> usize {
        match self {
            SceneComplexity::Light => 1_000_000,
            SceneComplexity::Medium => 2_000_000,
            SceneComplexity::Heavy => 4_000_000,
            SceneComplexity::Extreme => 6_000_000,
        }
    }

    pub fn agent_count(&self) -> usize {
        match self {
            SceneComplexity::Light => 2000,
            SceneComplexity::Medium => 5000,
            SceneComplexity::Heavy => 10000,
            SceneComplexity::Extreme => 20000,
        }
    }

    pub fn scene_pattern(&self) -> ScenePattern {
        match self {
            SceneComplexity::Light => ScenePattern::Sparse,
            SceneComplexity::Medium => ScenePattern::Medium,
            SceneComplexity::Heavy => ScenePattern::Dense,
            SceneComplexity::Extreme => ScenePattern::Dense,
        }
    }
}
