// VOXELCRAI Library Surface

pub mod archguard;
pub mod camera;
pub mod consciousness;
pub mod engine;
pub mod evolution;
pub mod lighting;
pub mod renderer;
pub mod simulation;
pub mod voxel;

// Re-export core types
pub use archguard::ArchGuard;
pub use camera::{Camera, CameraController, CameraUniform};
pub use consciousness::{ConsciousnessAction, ConsciousnessCore, ConsciousnessPulse};
pub use evolution::EvolutionEngine;
pub use lighting::{LightPattern, LightingSystem};
pub use renderer::{InstanceRaw, Renderer};
pub use simulation::{Simulation, SimulationMetrics};
pub use voxel::{Genome, Voxel, VoxelWorld, VoxelWorldConfig};
