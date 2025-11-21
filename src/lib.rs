// Adaptive Entity Engine v1.0 Library
// Minimalistic 3D Engine for Dynamic Scene Prototyping

pub mod archguard;
pub mod ecs;
pub mod evolution;
pub mod lighting;
pub mod renderer;
#[cfg(feature = "gui")]
pub mod ui;
pub mod voxel;

// New 3D Engine modules
pub mod camera;
pub mod pbr;
pub mod particles;
pub mod agents;
pub mod scene;
pub mod performance;
pub mod async_compute;
pub mod engine;
pub mod benchmark;
pub mod test_scene;
pub mod lighting_patterns;
pub mod pattern_baker;
pub mod benchmark_rtx3060;
pub mod benchmark_radeon_vii;

// Re-export main types
pub use archguard::ArchGuard;
pub use evolution::EvolutionEngine;
pub use lighting::{LightPattern, LightingSystem};
pub use voxel::{Voxel, VoxelWorld, Genome};

// Re-export 3D Engine types
pub use camera::Camera;
pub use pbr::{PBRMaterial, Light};
pub use particles::ParticleSystem;
pub use agents::{Agent, AgentSystem, AgentState};
pub use scene::{SceneManager, SceneObject, ScenePattern, ObjectType};
pub use performance::{PerformanceMonitor, PerformanceStats};
pub use async_compute::AsyncComputeManager;
pub use engine::Engine3D;
pub use benchmark::{BenchmarkConfig, BenchmarkRunner, BenchmarkResults};
pub use test_scene::{TestScene4K, SceneComplexity};
