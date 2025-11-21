// Benchmark system for testing performance at 4K resolution
// Target: RTX 4070 equivalent performance (~29 TFLOPs FP32, 12GB VRAM)
use crate::scene::ScenePattern;
use crate::performance::PerformanceMonitor;
use std::time::{Duration, Instant};

/// Benchmark configuration targeting RTX 4070 performance
pub struct BenchmarkConfig {
    pub resolution_4k: bool,
    pub particle_count: usize,
    pub agent_count: usize,
    pub scene_pattern: ScenePattern,
    pub target_fps: f32,
    pub duration_seconds: f32,
}

impl BenchmarkConfig {
    /// Create benchmark config for RTX 4070 equivalent
    pub fn rtx4070_4k() -> Self {
        Self {
            resolution_4k: true,
            particle_count: 2_000_000, // 2M particles for 4K (balanced load)
            agent_count: 5000,         // 5K agents
            scene_pattern: ScenePattern::Dense,
            target_fps: 60.0,
            duration_seconds: 30.0, // 30 second benchmark
        }
    }

    /// Create lighter benchmark for testing
    pub fn rtx4070_4k_light() -> Self {
        Self {
            resolution_4k: true,
            particle_count: 1_000_000, // 1M particles
            agent_count: 2000,         // 2K agents
            scene_pattern: ScenePattern::Medium,
            target_fps: 60.0,
            duration_seconds: 10.0,
        }
    }

    /// Create heavy benchmark (stress test)
    pub fn rtx4070_4k_heavy() -> Self {
        Self {
            resolution_4k: true,
            particle_count: 4_000_000, // 4M particles
            agent_count: 10000,        // 10K agents
            scene_pattern: ScenePattern::Dense,
            target_fps: 60.0,
            duration_seconds: 60.0,
        }
    }
}

/// Benchmark results
#[derive(Clone)]
pub struct BenchmarkResults {
    pub avg_fps: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub avg_frame_time_ms: f32,
    pub p1_low_fps: f32,      // 1% low FPS
    pub p0_1_low_fps: f32,    // 0.1% low FPS
    pub total_frames: u32,
    pub duration_seconds: f32,
    pub gpu_utilization_avg: f32,
    pub vram_usage_mb: f32,
}

impl BenchmarkResults {
    pub fn print_summary(&self) {
        println!("\n=== Benchmark Results ===");
        println!("Resolution: 4K (3840x2160)");
        println!("Duration: {:.2}s", self.duration_seconds);
        println!("Total Frames: {}", self.total_frames);
        println!("\nPerformance:");
        println!("  Average FPS: {:.2}", self.avg_fps);
        println!("  Min FPS: {:.2}", self.min_fps);
        println!("  Max FPS: {:.2}", self.max_fps);
        println!("  Average Frame Time: {:.3}ms", self.avg_frame_time_ms);
        println!("  1% Low FPS: {:.2}", self.p1_low_fps);
        println!("  0.1% Low FPS: {:.2}", self.p0_1_low_fps);
        println!("\nGPU Metrics:");
        println!("  Average GPU Utilization: {:.1}%", self.gpu_utilization_avg);
        println!("  VRAM Usage: {:.1} MB", self.vram_usage_mb);
        println!("========================\n");
    }
}

/// Benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    frame_times: Vec<f32>,
    fps_samples: Vec<f32>,
    start_time: Option<Instant>,
    gpu_samples: Vec<f32>,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            frame_times: Vec::new(),
            fps_samples: Vec::new(),
            start_time: None,
            gpu_samples: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.frame_times.clear();
        self.fps_samples.clear();
        self.gpu_samples.clear();
    }

    pub fn record_frame(&mut self, frame_time: f32, gpu_utilization: f32) {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs_f32();
            if elapsed < self.config.duration_seconds {
                self.frame_times.push(frame_time);
                let fps = 1.0 / frame_time.max(0.0001);
                self.fps_samples.push(fps);
                self.gpu_samples.push(gpu_utilization);
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        if let Some(start) = self.start_time {
            start.elapsed().as_secs_f32() >= self.config.duration_seconds
        } else {
            false
        }
    }

    pub fn get_results(&self) -> BenchmarkResults {
        if self.frame_times.is_empty() {
            return BenchmarkResults {
                avg_fps: 0.0,
                min_fps: 0.0,
                max_fps: 0.0,
                avg_frame_time_ms: 0.0,
                p1_low_fps: 0.0,
                p0_1_low_fps: 0.0,
                total_frames: 0,
                duration_seconds: 0.0,
                gpu_utilization_avg: 0.0,
                vram_usage_mb: 0.0,
            };
        }

        // Calculate statistics
        let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        let avg_fps = self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32;
        let min_fps = self.fps_samples.iter().copied().fold(f32::INFINITY, f32::min);
        let max_fps = self.fps_samples.iter().copied().fold(0.0, f32::max);

        // Calculate percentiles for low FPS (frame time percentiles)
        let mut sorted_frame_times = self.frame_times.clone();
        sorted_frame_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p99_index = (sorted_frame_times.len() as f32 * 0.99) as usize;
        let p99_9_index = (sorted_frame_times.len() as f32 * 0.999) as usize;
        
        let p1_low_fps = if p99_index < sorted_frame_times.len() {
            1.0 / sorted_frame_times[p99_index].max(0.0001)
        } else {
            0.0
        };
        
        let p0_1_low_fps = if p99_9_index < sorted_frame_times.len() {
            1.0 / sorted_frame_times[p99_9_index].max(0.0001)
        } else {
            0.0
        };

        let gpu_avg = if !self.gpu_samples.is_empty() {
            self.gpu_samples.iter().sum::<f32>() / self.gpu_samples.len() as f32
        } else {
            0.0
        };

        BenchmarkResults {
            avg_fps,
            min_fps,
            max_fps,
            avg_frame_time_ms: avg_frame_time * 1000.0,
            p1_low_fps,
            p0_1_low_fps,
            total_frames: self.frame_times.len() as u32,
            duration_seconds: self.config.duration_seconds,
            gpu_utilization_avg: gpu_avg,
            vram_usage_mb: 0.0, // Would be filled from actual VRAM monitoring
        }
    }

    pub fn get_config(&self) -> &BenchmarkConfig {
        &self.config
    }
}
