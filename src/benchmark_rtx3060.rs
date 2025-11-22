// Benchmark configuration for RTX 3060 in 4K
// RTX 3060: ~13 TFLOPs (FP32), 12GB VRAM, ~40-50 FPS in 4K games

use crate::benchmark::BenchmarkConfig;
use crate::scene::ScenePattern;

impl BenchmarkConfig {
    /// Create benchmark config for RTX 3060 4K
    pub fn rtx3060_4k() -> Self {
        Self {
            resolution_4k: true,
            particle_count: 1_500_000, // 1.5M particles (меньше чем для 4070)
            agent_count: 4000,         // 4K agents
            scene_pattern: ScenePattern::Dense,
            target_fps: 45.0,          // Реалистичная цель для 3060
            duration_seconds: 30.0,
        }
    }

    /// Light benchmark for RTX 3060
    pub fn rtx3060_4k_light() -> Self {
        Self {
            resolution_4k: true,
            particle_count: 1_000_000, // 1M particles
            agent_count: 2500,         // 2.5K agents
            scene_pattern: ScenePattern::Medium,
            target_fps: 60.0,
            duration_seconds: 15.0,
        }
    }

    /// Heavy benchmark (stress test for RTX 3060)
    pub fn rtx3060_4k_heavy() -> Self {
        Self {
            resolution_4k: true,
            particle_count: 2_500_000, // 2.5M particles
            agent_count: 7000,         // 7K agents
            scene_pattern: ScenePattern::Dense,
            target_fps: 30.0,          // Низкая цель для stress test
            duration_seconds: 45.0,
        }
    }
}
