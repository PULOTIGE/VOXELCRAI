// Performance monitoring system
// Tracks FPS, GPU load, VRAM usage, frame time, and statistics for different scene patterns
use std::time::{Duration, Instant};
use std::collections::VecDeque;

/// Performance statistics
#[derive(Clone)]
pub struct PerformanceStats {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub gpu_load_percent: f32,
    pub vram_usage_mb: f32,
    pub frame_times: VecDeque<f32>,
    pub sparse_stats: PatternStats,
    pub medium_stats: PatternStats,
    pub dense_stats: PatternStats,
}

#[derive(Clone)]
pub struct PatternStats {
    pub avg_fps: f32,
    pub avg_frame_time: f32,
    pub avg_gpu_load: f32,
    pub sample_count: usize,
}

impl PatternStats {
    pub fn new() -> Self {
        Self {
            avg_fps: 0.0,
            avg_frame_time: 0.0,
            avg_gpu_load: 0.0,
            sample_count: 0,
        }
    }

    pub fn add_sample(&mut self, fps: f32, frame_time: f32, gpu_load: f32) {
        let count = self.sample_count as f32;
        self.avg_fps = (self.avg_fps * count + fps) / (count + 1.0);
        self.avg_frame_time = (self.avg_frame_time * count + frame_time) / (count + 1.0);
        self.avg_gpu_load = (self.avg_gpu_load * count + gpu_load) / (count + 1.0);
        self.sample_count += 1;
    }
}

/// Performance Monitor
pub struct PerformanceMonitor {
    pub stats: PerformanceStats,
    frame_start: Instant,
    frame_count: u32,
    last_log_time: Instant,
    log_interval: Duration,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            stats: PerformanceStats {
                fps: 60.0,
                frame_time_ms: 16.67,
                gpu_load_percent: 0.0,
                vram_usage_mb: 0.0,
                frame_times: VecDeque::with_capacity(60),
                sparse_stats: PatternStats::new(),
                medium_stats: PatternStats::new(),
                dense_stats: PatternStats::new(),
            },
            frame_start: Instant::now(),
            frame_count: 0,
            last_log_time: Instant::now(),
            log_interval: Duration::from_secs(1),
        }
    }

    /// Start frame timing
    pub fn start_frame(&mut self) {
        self.frame_start = Instant::now();
    }

    /// End frame and update statistics
    pub fn end_frame(&mut self, pattern: crate::scene::ScenePattern) {
        let frame_time = self.frame_start.elapsed();
        let frame_time_ms = frame_time.as_secs_f32() * 1000.0;
        
        self.frame_count += 1;
        
        // Update frame times (keep last 60 frames)
        self.stats.frame_times.push_back(frame_time_ms);
        if self.stats.frame_times.len() > 60 {
            self.stats.frame_times.pop_front();
        }

        // Calculate FPS
        if frame_time.as_secs_f32() > 0.0 {
            self.stats.fps = 1.0 / frame_time.as_secs_f32();
        }

        self.stats.frame_time_ms = frame_time_ms;

        // Update pattern-specific stats
        let pattern_stats = match pattern {
            crate::scene::ScenePattern::Sparse => &mut self.stats.sparse_stats,
            crate::scene::ScenePattern::Medium => &mut self.stats.medium_stats,
            crate::scene::ScenePattern::Dense => &mut self.stats.dense_stats,
        };
        pattern_stats.add_sample(self.stats.fps, frame_time_ms, self.stats.gpu_load_percent);

        // Log statistics periodically
        if self.last_log_time.elapsed() >= self.log_interval {
            self.log_stats(pattern);
            self.last_log_time = Instant::now();
        }
    }

    /// Update GPU metrics (would integrate with actual GPU monitoring)
    pub fn update_gpu_metrics(&mut self, gpu_load: f32, vram_usage_mb: f32) {
        self.stats.gpu_load_percent = gpu_load;
        self.stats.vram_usage_mb = vram_usage_mb;
    }

    /// Log performance statistics
    fn log_stats(&self, current_pattern: crate::scene::ScenePattern) {
        println!("=== Performance Statistics ===");
        println!("Current Pattern: {:?}", current_pattern);
        println!("FPS: {:.2}", self.stats.fps);
        println!("Frame Time: {:.2} ms", self.stats.frame_time_ms);
        println!("GPU Load: {:.1}%", self.stats.gpu_load_percent);
        println!("VRAM Usage: {:.1} MB", self.stats.vram_usage_mb);
        println!("\nPattern Statistics:");
        println!("  Sparse:  FPS={:.2}, FrameTime={:.2}ms, GPU={:.1}% ({} samples)",
                 self.stats.sparse_stats.avg_fps,
                 self.stats.sparse_stats.avg_frame_time,
                 self.stats.sparse_stats.avg_gpu_load,
                 self.stats.sparse_stats.sample_count);
        println!("  Medium:  FPS={:.2}, FrameTime={:.2}ms, GPU={:.1}% ({} samples)",
                 self.stats.medium_stats.avg_fps,
                 self.stats.medium_stats.avg_frame_time,
                 self.stats.medium_stats.avg_gpu_load,
                 self.stats.medium_stats.sample_count);
        println!("  Dense:   FPS={:.2}, FrameTime={:.2}ms, GPU={:.1}% ({} samples)",
                 self.stats.dense_stats.avg_fps,
                 self.stats.dense_stats.avg_frame_time,
                 self.stats.dense_stats.avg_gpu_load,
                 self.stats.dense_stats.sample_count);
        println!("=============================\n");
    }

    /// Get average frame time over last N frames
    pub fn get_avg_frame_time(&self, frames: usize) -> f32 {
        let count = frames.min(self.stats.frame_times.len());
        if count == 0 {
            return 0.0;
        }
        let sum: f32 = self.stats.frame_times.iter().rev().take(count).sum();
        sum / count as f32
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}
