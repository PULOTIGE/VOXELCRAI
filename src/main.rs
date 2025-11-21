// Main entry point for the 3D Engine
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use winit::window::Window;
use std::time::Instant;
use adaptive_entity_engine::engine::Engine3D;
use adaptive_entity_engine::scene::ScenePattern;
use adaptive_entity_engine::benchmark::BenchmarkConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    let window_attributes = Window::default_attributes()
        .with_title("Minimalistic 3D Engine - Dynamic Scene Prototyping")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
    let window = event_loop.create_window(window_attributes)?;

    // Initialize engine
    let mut engine = Engine3D::new(&window)?;

    // Timing
    let mut last_frame_time = Instant::now();
    let mut frame_count = 0u32;

    println!("=== Minimalistic 3D Engine ===");
    println!("Features:");
    println!("  - Rendering Pipeline with PBR materials");
    println!("  - Particle System (GPU compute, up to 6M particles)");
    println!("  - Voxel Agent System (FSM behaviors, spatial hash, LOD)");
    println!("  - Scene Manager (Sparse/Medium/Dense patterns)");
    println!("  - Async Compute Management");
    println!("  - Performance Monitoring");
    println!("==============================\n");
    println!("Controls:");
    println!("  1/2/3 - Switch scene pattern (Sparse/Medium/Dense)");
    println!("  B - Start 4K benchmark (RTX 4070 target)");
    println!("  L - Start light 4K benchmark");
    println!("  H - Start heavy 4K benchmark");
    println!();

    // Main loop
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        engine.resize(physical_size.width, physical_size.height);
                    }
                    WindowEvent::RedrawRequested => {
                        // Calculate delta time
                        let now = Instant::now();
                        let delta_time = last_frame_time.elapsed().as_secs_f32();
                        last_frame_time = now;

                        // Update engine
                        engine.update(delta_time);

                        // Render
                        match engine.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => {
                                engine.resize(engine.config.width, engine.config.height);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                elwt.exit();
                            }
                            Err(e) => eprintln!("Render error: {:?}", e),
                        }

                        // Request next frame
                        window.request_redraw();

                        // Log FPS every 60 frames
                        frame_count += 1;
                        if frame_count % 60 == 0 {
                            let fps = 1.0 / delta_time.max(0.0001);
                            println!("FPS: {:.2}, Frame Time: {:.3}ms", fps, delta_time * 1000.0);
                        }
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        // Handle keyboard input
                        if event.state == winit::event::ElementState::Pressed {
                            match event.logical_key.as_ref() {
                                winit::keyboard::Key::Character(c) if c == "1" => {
                                    if !engine.is_benchmark_mode {
                                        engine.set_scene_pattern(ScenePattern::Sparse);
                                        println!("Switched to Sparse pattern");
                                    }
                                }
                                winit::keyboard::Key::Character(c) if c == "2" => {
                                    if !engine.is_benchmark_mode {
                                        engine.set_scene_pattern(ScenePattern::Medium);
                                        println!("Switched to Medium pattern");
                                    }
                                }
                                winit::keyboard::Key::Character(c) if c == "3" => {
                                    if !engine.is_benchmark_mode {
                                        engine.set_scene_pattern(ScenePattern::Dense);
                                        println!("Switched to Dense pattern");
                                    }
                                }
                                winit::keyboard::Key::Character(c) if c.to_lowercase() == "b" => {
                                    if !engine.is_benchmark_mode {
                                        println!("\nStarting 4K Benchmark (RTX 4070 target)...");
                                        if let Err(e) = engine.init_benchmark_4k(BenchmarkConfig::rtx4070_4k()) {
                                            eprintln!("Failed to start benchmark: {}", e);
                                        }
                                    }
                                }
                                winit::keyboard::Key::Character(c) if c.to_lowercase() == "l" => {
                                    if !engine.is_benchmark_mode {
                                        println!("\nStarting Light 4K Benchmark...");
                                        if let Err(e) = engine.init_benchmark_4k(BenchmarkConfig::rtx4070_4k_light()) {
                                            eprintln!("Failed to start benchmark: {}", e);
                                        }
                                    }
                                }
                                winit::keyboard::Key::Character(c) if c.to_lowercase() == "h" => {
                                    if !engine.is_benchmark_mode {
                                        println!("\nStarting Heavy 4K Benchmark (Stress Test)...");
                                        if let Err(e) = engine.init_benchmark_4k(BenchmarkConfig::rtx4070_4k_heavy()) {
                                            eprintln!("Failed to start benchmark: {}", e);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}
