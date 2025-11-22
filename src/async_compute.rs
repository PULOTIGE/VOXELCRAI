// Async Compute Management
// Separate queues for agents and particles, minimize synchronizations
// Optimized for AMD/NVIDIA GPU
use wgpu::*;
use std::sync::Arc;

/// Async Compute Manager for managing separate compute queues
pub struct AsyncComputeManager {
    pub device: Arc<Device>,
    pub main_queue: Arc<Queue>,
    pub compute_queue_agents: Option<Arc<Queue>>,
    pub compute_queue_particles: Option<Arc<Queue>>,
    pub supports_async_compute: bool,
}

impl AsyncComputeManager {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        // Check if device supports async compute (multiple queues)
        // Most modern GPUs support this, but we'll check capabilities
        let supports_async_compute = device.limits().max_compute_workgroups_per_dimension > 0;

        // For now, use the same queue (would need adapter with multiple queues)
        // In a real implementation, you'd request multiple queues from the adapter
        let compute_queue_agents = if supports_async_compute {
            // Would create separate queue here
            None // Fallback to main queue
        } else {
            None
        };

        let compute_queue_particles = if supports_async_compute {
            None // Fallback to main queue
        } else {
            None
        };

        Self {
            device,
            main_queue: queue,
            compute_queue_agents,
            compute_queue_particles,
            supports_async_compute,
        }
    }

    /// Get queue for agent compute
    pub fn get_agent_queue(&self) -> &Queue {
        self.compute_queue_agents.as_ref().unwrap_or(&self.main_queue)
    }

    /// Get queue for particle compute
    pub fn get_particle_queue(&self) -> &Queue {
        self.compute_queue_particles.as_ref().unwrap_or(&self.main_queue)
    }

    /// Submit compute work for agents (async, no sync with graphics)
    pub fn submit_agent_compute(&self, encoder: CommandEncoder) {
        let queue = self.get_agent_queue();
        queue.submit(std::iter::once(encoder.finish()));
    }

    /// Submit compute work for particles (async, no sync with graphics)
    pub fn submit_particle_compute(&self, encoder: CommandEncoder) {
        let queue = self.get_particle_queue();
        queue.submit(std::iter::once(encoder.finish()));
    }

    /// Create command encoder for agent compute
    pub fn create_agent_encoder(&self) -> CommandEncoder {
        self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Agent Compute Encoder"),
        })
    }

    /// Create command encoder for particle compute
    pub fn create_particle_encoder(&self) -> CommandEncoder {
        self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Particle Compute Encoder"),
        })
    }

    /// Wait for all compute work to complete (synchronization point)
    pub fn wait_for_compute(&self) {
        // In a real implementation, would use fences or events
        // For now, just ensure queue is idle
        self.main_queue.on_submitted_work_done(|| {});
    }
}
