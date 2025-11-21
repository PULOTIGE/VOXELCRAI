// Benchmark configuration for Radeon VII in 4K
// Radeon VII: ~13.4 TFLOPs (FP32), 16GB HBM2, ~40-50 FPS in 4K games
// AMD GPU - может иметь особенности с Vulkan/wgpu

use crate::benchmark::BenchmarkConfig;
use crate::scene::ScenePattern;

impl BenchmarkConfig {
    /// Create benchmark config for Radeon VII 4K
    pub fn radeon_vii_4k() -> Self {
        Self {
            resolution_4k: true,
            particle_count: 1_400_000, // 1.4M particles (немного меньше для AMD)
            agent_count: 3500,         // 3.5K agents
            scene_pattern: ScenePattern::Dense,
            target_fps: 45.0,          // Реалистичная цель для Radeon VII
            duration_seconds: 30.0,
        }
    }

    /// Light benchmark for Radeon VII
    pub fn radeon_vii_4k_light() -> Self {
        Self {
            resolution_4k: true,
            particle_count: 900_000,  // 900K particles
            agent_count: 2000,        // 2K agents
            scene_pattern: ScenePattern::Medium,
            target_fps: 60.0,
            duration_seconds: 15.0,
        }
    }

    /// Heavy benchmark (stress test for Radeon VII)
    pub fn radeon_vii_4k_heavy() -> Self {
        Self {
            resolution_4k: true,
            particle_count: 2_200_000, // 2.2M particles
            agent_count: 6000,         // 6K agents
            scene_pattern: ScenePattern::Dense,
            target_fps: 30.0,
            duration_seconds: 45.0,
        }
    }
}
