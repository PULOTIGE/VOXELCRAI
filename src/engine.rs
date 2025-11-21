// Main 3D Engine integrating all systems
use wgpu::*;
use winit::window::Window;
use std::sync::Arc;
use glam::Vec3;

use crate::camera::Camera;
use crate::particles::ParticleSystem;
use crate::agents::{AgentSystem, Agent};
use crate::scene::{SceneManager, ScenePattern};
use crate::performance::PerformanceMonitor;
use crate::async_compute::AsyncComputeManager;
use crate::pbr::Light;
use bevy_ecs::prelude::*;

/// Main 3D Engine
pub struct Engine3D {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    pub camera: Camera,
    pub particle_system: ParticleSystem,
    pub agent_system: AgentSystem,
    pub scene_manager: SceneManager,
    pub performance_monitor: PerformanceMonitor,
    pub async_compute: AsyncComputeManager,
    pub world: World,
    pub light: Light,
    pub delta_time: f32,
}

impl Engine3D {
    pub fn new(window: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();

        // Initialize wgpu
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN | Backends::GL,
            ..Default::default()
        });

        let surface = unsafe {
            instance.create_surface_unsafe(
                wgpu::SurfaceTargetUnsafe::from_window(window)?
            )?
        };

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).ok_or("Failed to find adapter")?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("Engine Device"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
            },
            None,
        ))?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Initialize systems
        let mut camera = Camera::new(Vec3::new(0.0, 10.0, 20.0), Vec3::ZERO);
        camera.set_aspect(size.width as f32 / size.height as f32);

        let mut particle_system = ParticleSystem::new(
            device.clone(),
            queue.clone(),
            6_000_000, // 6M particles
        );
        particle_system.init_gpu()?;

        let agent_system = AgentSystem::new(10000); // 10K agents

        let scene_manager = SceneManager::new(ScenePattern::Medium);

        let performance_monitor = PerformanceMonitor::new();

        let async_compute = AsyncComputeManager::new(device.clone(), queue.clone());

        let mut world = World::new();

        // Spawn some agents
        for i in 0..100 {
            let pos = Vec3::new(
                (i as f32 % 10.0) * 2.0 - 10.0,
                1.0,
                (i as f32 / 10.0).floor() * 2.0 - 10.0,
            );
            world.spawn(Agent::new(pos));
        }

        let light = Light::default();

        Ok(Self {
            device,
            queue,
            surface,
            config,
            camera,
            particle_system,
            agent_system,
            scene_manager,
            performance_monitor,
            async_compute,
            world,
            light,
            delta_time: 0.016,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.camera.set_aspect(width as f32 / height as f32);
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.delta_time = delta_time;
        self.performance_monitor.start_frame();

        // Update camera
        // (Camera controls would go here)

        // Update particle system (GPU compute)
        let mut particle_encoder = self.async_compute.create_particle_encoder();
        self.particle_system.update_gpu(delta_time, &mut particle_encoder);
        self.async_compute.submit_particle_compute(particle_encoder);

        // Update agent system (with spatial hash and LOD)
        self.agent_system.update(&mut self.world, self.camera.position, delta_time);

        // Update scene manager
        // (Scene updates would go here)

        // Update performance monitoring
        let gpu_load = 50.0; // Would get from actual GPU monitoring
        let vram_usage = 1000.0; // Would get from actual VRAM monitoring
        self.performance_monitor.update_gpu_metrics(gpu_load, vram_usage);
        self.performance_monitor.end_frame(self.scene_manager.pattern);
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Render particles, agents, and scene objects here
            // (Full rendering implementation would go here)
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn set_scene_pattern(&mut self, pattern: ScenePattern) {
        self.scene_manager.set_pattern(pattern);
    }
}
