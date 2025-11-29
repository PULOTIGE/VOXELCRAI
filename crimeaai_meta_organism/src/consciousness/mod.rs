//! Система сознания Meta-Organism
//! 
//! Модули:
//! - `core` — Главный организм и его состояние
//! - `coevolution` — Коэволюционное обучение (Лавренков 2018)

pub mod core;
pub mod coevolution;

pub use core::*;
pub use coevolution::*;
