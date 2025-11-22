// Benchmark runner for Radeon VII in 4K with baked lighting patterns
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use winit::window::Window;
use std::time::Instant;
use adaptive_entity_engine::engine::Engine3D;
use adaptive_entity_engine::benchmark::BenchmarkConfig;
use adaptive_entity_engine::lighting_patterns::PatternType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== Radeon VII 4K Benchmark ===");
    println!("Target: Radeon VII equivalent performance");
    println!("Resolution: 3840x2160 (4K)");
    println!("Features: Baked lighting patterns (shadows, rays, rain)");
    println!("GPU: AMD Radeon VII (Vega 20, 16GB HBM2)");
    println!();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let (config, pattern_name) = if args.len() > 1 {
        match args[1].as_str() {
            "light" => {
                println!("Using Light benchmark preset");
                (BenchmarkConfig::radeon_vii_4k_light(), "Sunny")
            }
            "heavy" => {
                println!("Using Heavy benchmark preset (Stress Test)");
                (BenchmarkConfig::radeon_vii_4k_heavy(), "Stormy")
            }
            _ => {
                println!("Using Standard benchmark preset (Radeon VII target)");
                (BenchmarkConfig::radeon_vii_4k(), "Sunny")
            }
        }
    } else {
        println!("Using Standard benchmark preset (Radeon VII target)");
        println!("Usage: cargo run --bin benchmark-radeon-vii [light|heavy]");
        (BenchmarkConfig::radeon_vii_4k(), "Sunny")
    };

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    let window_attributes = Window::default_attributes()
        .with_title("Radeon VII 4K Benchmark - Baked Lighting Patterns")
        .with_inner_size(winit::dpi::LogicalSize::new(3840, 2160));
    let window = event_loop.create_window(window_attributes)?;

    // Initialize engine
    let mut engine = Engine3D::new(&window)?;

    // Set lighting pattern
    let pattern_type = match pattern_name {
        "Sunny" => PatternType::Sunny,
        "Stormy" => PatternType::Stormy,
        _ => PatternType::Sunny,
    };
    engine.current_lighting_pattern = Some(pattern_type);
    println!("Lighting pattern: {:?}", pattern_type);
    println!();

    // Start benchmark immediately
    println!("Initializing benchmark...");
    engine.init_benchmark_4k(config)?;

    // Timing
    let mut last_frame_time = Instant::now();

    println!("Benchmark started! Running for {} seconds...\n", 
             engine.benchmark_runner.as_ref().map(|b| b.get_config().duration_seconds).unwrap_or(30.0));

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
                                eprintln!("Out of memory! Benchmark stopped.");
                                elwt.exit();
                            }
                            Err(e) => eprintln!("Render error: {:?}", e),
                        }

                        // Request next frame
                        window.request_redraw();

                        // Exit if benchmark is complete
                        if !engine.is_benchmark_mode {
                            println!("\nBenchmark completed! Exiting...");
                            elwt.exit();
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
