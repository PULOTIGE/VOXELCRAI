//! VoxelStrike - A CS-like FPS game built on Adaptive Entity Engine
//!
//! Features:
//! - FPS camera with mouse look
//! - Multiple weapons (AK-47, M4A1, AWP, Deagle, Knife)
//! - Simple de_dust inspired map
//! - Enemy bots with basic AI
//! - Health, armor, and ammo system
//! - HUD with crosshair, health bar, ammo counter

pub mod camera;
pub mod enemies;
pub mod hud;
pub mod map;
pub mod player;
pub mod renderer;
pub mod weapons;

pub use camera::{FPSCamera, MoveDirection};
pub use enemies::{Enemy, EnemyManager, Team};
pub use hud::{GameHUD, HudRenderData};
pub use map::{GameMap, MapBlock, AABB, Vertex};
pub use player::Player;
pub use renderer::GameRenderer;
pub use weapons::{Weapon, WeaponInventory, WeaponType};
