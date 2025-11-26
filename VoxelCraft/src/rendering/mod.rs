// VoxelCraft - Simple Renderer for Android
// Minimal version to test wgpu compatibility

use crate::{GameState, GameUI};
use std::sync::Arc;
use winit::window::Window;
use glam::{Vec3, Mat4};

/// Camera
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub aspect: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 64.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 70.0_f32.to_radians(),
            aspect: width / height.max(1.0),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height.max(1.0);
    }
}

/// Simple renderer - just clears screen with color
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    pub camera: Camera,
    time: f32,
}

impl Renderer {
    pub async fn try_new(window: Arc<Window>) -> Result<Self, String> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        log::info!("Renderer: Creating with size {}x{}", width, height);

        // Step 1: Create instance
        log::info!("Step 1: Creating wgpu instance...");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });
        log::info!("Instance created!");

        // Step 2: Create surface
        log::info!("Step 2: Creating surface...");
        let surface = instance.create_surface(window.clone())
            .map_err(|e| {
                log::error!("Surface creation failed: {}", e);
                format!("Surface error: {}", e)
            })?;
        log::info!("Surface created!");

        // Step 3: Get adapter
        log::info!("Step 3: Requesting adapter...");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| {
                log::error!("No adapter found!");
                "No GPU adapter found".to_string()
            })?;
        
        let info = adapter.get_info();
        log::info!("Adapter: {} ({:?})", info.name, info.backend);

        // Step 4: Get device
        log::info!("Step 4: Requesting device...");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    label: Some("Device"),
                },
                None,
            )
            .await
            .map_err(|e| {
                log::error!("Device creation failed: {}", e);
                format!("Device error: {}", e)
            })?;
        log::info!("Device created!");

        // Step 5: Configure surface
        log::info!("Step 5: Configuring surface...");
        let surface_caps = surface.get_capabilities(&adapter);
        log::info!("Formats: {:?}", surface_caps.formats);
        log::info!("Present modes: {:?}", surface_caps.present_modes);
        
        let surface_format = surface_caps.formats.first()
            .copied()
            .ok_or("No surface formats")?;
        log::info!("Using format: {:?}", surface_format);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes.first().copied().unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        log::info!("Surface configured!");

        let camera = Camera::new(width as f32, height as f32);
        
        log::info!("Renderer created successfully!");
        
        Ok(Self {
            device,
            queue,
            surface,
            config,
            camera,
            time: 0.0,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        log::info!("Resize to {}x{}", width, height);
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.camera.resize(width as f32, height as f32);
    }

    pub fn render(&mut self, state: &GameState, _ui: &GameUI) {
        self.time += 0.016;

        // Update camera from player
        self.camera.position = state.player.get_eye_position();
        self.camera.yaw = state.player.rotation.0;
        self.camera.pitch = state.player.rotation.1;

        // Get surface texture
        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                log::warn!("Surface lost, reconfiguring...");
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(e) => {
                log::error!("Surface error: {:?}", e);
                return;
            }
        };
        
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Encoder"),
        });

        // Simple animated background color
        let t = self.time * 0.5;
        let r = (t.sin() * 0.3 + 0.3) as f64;
        let g = ((t + 1.0).sin() * 0.3 + 0.5) as f64;
        let b = ((t + 2.0).sin() * 0.2 + 0.6) as f64;

        // Clear screen with color
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            // Render pass ends here
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
