//! Воксельная система Meta-Organism
//! 
//! Модули:
//! - `types` — Voxel9k и связанные структуры
//! - `memory` — ANIRLE-компрессия (Сидоров 2017)
//! - `clustering` — Кластеризация Алсынбаева (2016)
//! - `trauma` — Система травмы Никоновой (2013)
//! - `atrophy` — Атрофия Ахмадуллиной (2015)

pub mod types;
pub mod memory;
pub mod clustering;
pub mod trauma;
pub mod atrophy;

pub use types::*;
pub use memory::*;
pub use clustering::*;
pub use trauma::*;
pub use atrophy::*;
