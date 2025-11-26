// Adaptive Entity Engine v1.0 Library

pub mod archguard;
#[cfg(feature = "gui")]
pub mod chat;
pub mod ecs;
pub mod evolution;
#[cfg(feature = "gui")]
pub mod integrations;
#[cfg(feature = "gui")]
pub mod learning;
pub mod lighting;
#[cfg(feature = "gui")]
pub mod renderer;
#[cfg(feature = "gui")]
pub mod ui;
pub mod voxel;

// Re-export main types
pub use archguard::ArchGuard;
pub use evolution::EvolutionEngine;
pub use lighting::{LightPattern, LightingSystem};
pub use voxel::{Voxel, VoxelWorld, Genome};
