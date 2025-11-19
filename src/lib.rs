// Adaptive Entity Engine v1.0 Library

pub mod archguard;
pub mod ecs;
pub mod evolution;
pub mod lighting;
pub mod renderer;
pub mod ui;
pub mod voxel;

// Re-export main types
pub use archguard::ArchGuard;
pub use evolution::EvolutionEngine;
pub use lighting::{LightPattern, LightingSystem};
pub use voxel::{Voxel, VoxelWorld, Genome};
