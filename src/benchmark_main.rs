// Standalone benchmark runner for 4K RTX 4070 testing
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use winit::window::Window;
use std::time::Instant;
use adaptive_entity_engine::engine::Engine3D;
use adaptive_entity_engine::benchmark::BenchmarkConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== 4K Benchmark Runner ===");
    println!("Target: RTX 4070 equivalent performance");
    println!("Resolution: 3840x2160 (4K)");
    println!();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let config = if args.len() > 1 {
        match args[1].as_str() {
            "light" => {
                println!("Using Light benchmark preset");
                BenchmarkConfig::rtx4070_4k_light()
            }
            "heavy" => {
                println!("Using Heavy benchmark preset (Stress Test)");
                BenchmarkConfig::rtx4070_4k_heavy()
            }
            _ => {
                println!("Using Standard benchmark preset (RTX 4070 target)");
                BenchmarkConfig::rtx4070_4k()
            }
        }
    } else {
        println!("Using Standard benchmark preset (RTX 4070 target)");
        println!("Usage: cargo run --bin benchmark [light|heavy]");
        BenchmarkConfig::rtx4070_4k()
    };

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    let window_attributes = Window::default_attributes()
        .with_title("4K Benchmark - RTX 4070 Target")
        .with_inner_size(winit::dpi::LogicalSize::new(3840, 2160)); // Start with 4K
    let window = event_loop.create_window(window_attributes)?;

    // Initialize engine
    let mut engine = Engine3D::new(&window)?;

    // Start benchmark immediately
    println!("\nInitializing benchmark...");
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
