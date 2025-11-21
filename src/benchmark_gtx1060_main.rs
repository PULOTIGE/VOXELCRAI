// Main benchmark runner for GTX 1060 4K
use adaptive_entity_engine::benchmark::BenchmarkConfig;
use adaptive_entity_engine::benchmark::BenchmarkRunner;
use std::env;

fn main() {
    env_logger::init();
    
    let config = match env::args().nth(1).as_deref() {
        Some("light") => BenchmarkConfig::gtx1060_4k_light(),
        Some("heavy") => BenchmarkConfig::gtx1060_4k_heavy(),
        _ => BenchmarkConfig::gtx1060_4k(),
    };
    
    println!("=== GTX 1060 4K Benchmark ===");
    println!("Configuration: {:?}", config.scene_pattern);
    println!("Particles: {}", config.particle_count);
    println!("Agents: {}", config.agent_count);
    println!("Resolution: 4K (3840x2160)");
    println!("Target FPS: {}", config.target_fps);
    println!();
    
    let mut runner = BenchmarkRunner::new(config);
    runner.run();
}
