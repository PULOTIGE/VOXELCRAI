//! Система рендеринга Meta-Organism
//! 
//! Модули:
//! - `pipeline` — wgpu рендер пайплайн
//! - `camera` — Камера и управление

pub mod pipeline;
pub mod camera;

pub use pipeline::*;
pub use camera::*;
