// Benchmark configuration for GTX 1060 4K
// GTX 1060: 6GB GDDR5, Pascal architecture, ~192 GB/s bandwidth

use crate::benchmark::BenchmarkConfig;
use crate::scene::ScenePattern;

impl BenchmarkConfig {
    /// Стандартная конфигурация для GTX 1060 4K
    pub fn gtx1060_4k() -> Self {
        Self {
            resolution_4k: true,
            particle_count: 800_000,      // Меньше частиц для GTX 1060
            agent_count: 2_000,           // Меньше агентов
            scene_pattern: ScenePattern::Medium,
            target_fps: 30.0,             // Реалистичная цель для GTX 1060
            duration_seconds: 30.0,
        }
    }
    
    /// Легкая конфигурация для GTX 1060
    pub fn gtx1060_4k_light() -> Self {
        Self {
            resolution_4k: true,
            particle_count: 500_000,      // Еще меньше частиц
            agent_count: 1_200,           // Еще меньше агентов
            scene_pattern: ScenePattern::Sparse,
            target_fps: 45.0,
            duration_seconds: 30.0,
        }
    }
    
    /// Тяжелая конфигурация (stress test) для GTX 1060
    pub fn gtx1060_4k_heavy() -> Self {
        Self {
            resolution_4k: true,
            particle_count: 1_200_000,    // Больше частиц
            agent_count: 3_000,           // Больше агентов
            scene_pattern: ScenePattern::Dense,
            target_fps: 25.0,             // Низкая цель для stress test
            duration_seconds: 30.0,
        }
    }
}
