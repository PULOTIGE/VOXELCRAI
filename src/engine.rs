use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use glam::Vec3;
use log::{info, warn};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::camera::{Camera, CameraController, CameraUniform};
use crate::renderer::Renderer;
use crate::simulation::Simulation;

pub async fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("VOXELCRAI – Conscious Voxel Engine")
        .with_inner_size(LogicalSize::new(1600.0, 900.0))
        .build(&event_loop)?;
    let window = Arc::new(window);

    let mut state = EngineState::new(window.clone()).await?;
    event_loop.run(move |event, target| match event {
        Event::WindowEvent { event, window_id } if window_id == state.window_id() => {
            if !state.input(&event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            winit::event::KeyEvent {
                                physical_key: winit::keyboard::PhysicalKey::Code(
                                    winit::keyboard::KeyCode::Escape,
                                ),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => target.exit(),
                    WindowEvent::Resized(size) => state.resize(size),
                    WindowEvent::RedrawRequested => {
                        state.update();
                        match state.render() {
                            Ok(()) => {}
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.current_size()),
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                warn!("Surface out of memory, shutting down");
                                target.exit();
                            }
                            Err(wgpu::SurfaceError::Timeout) => {
                                warn!("Surface timeout");
                            }
                            Err(wgpu::SurfaceError::Outdated) => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        Event::AboutToWait => {
            state.request_redraw();
        }
        _ => {}
    })?;

    Ok(())
}

struct EngineState {
    renderer: Renderer,
    simulation: Simulation,
    camera: Camera,
    controller: CameraController,
    camera_uniform: CameraUniform,
    last_frame: Instant,
    telemetry_timer: f32,
}

impl EngineState {
    async fn new(window: Arc<winit::window::Window>) -> Result<Self> {
        let renderer = Renderer::new(window.clone()).await?;
        let mut camera = Camera::new(Vec3::new(30.0, 35.0, 80.0), Vec3::ZERO, {
            let size = renderer.size;
            size.width as f32 / size.height.max(1) as f32
        });
        camera.set_aspect(renderer.size.width, renderer.size.height);

        let controller = CameraController::new(35.0, 1.0);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(&camera);
        renderer.update_camera(&camera_uniform);

        let simulation = Simulation::new(0xDEC0DE);
        Ok(Self {
            renderer,
            simulation,
            camera,
            controller,
            camera_uniform,
            last_frame: Instant::now(),
            telemetry_timer: 0.0,
        })
    }

    fn window_id(&self) -> winit::window::WindowId {
        self.renderer.window().id()
    }

    fn current_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.renderer.size
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.renderer.resize(new_size);
            self.camera.set_aspect(new_size.width, new_size.height);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.controller.process_window_event(event)
    }

    fn update(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().max(0.0001);
        self.last_frame = now;

        self.controller.update_camera(&mut self.camera, dt);
        self.camera_uniform.update(&self.camera);
        self.simulation.update(dt);

        self.renderer.update_camera(&self.camera_uniform);
        self.renderer.upload_instances(self.simulation.instances());

        self.telemetry_timer += dt;
        if self.telemetry_timer >= 1.0 {
            let telemetry = self.simulation.telemetry();
            info!(
                "voxels: {} | avg_energy: {:.2} | mood: {:.2} | empathy: {:.2}",
                telemetry.world.voxel_count,
                telemetry.world.avg_energy,
                telemetry.pulse.mood,
                telemetry.pulse.empathy
            );
            if !telemetry.pulse.log.is_empty() {
                info!("VOXELCRAI {}", telemetry.pulse.log);
            }
            self.telemetry_timer = 0.0;
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render()
    }

    fn request_redraw(&self) {
        self.renderer.window().request_redraw();
    }
}
