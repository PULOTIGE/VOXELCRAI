//! VoxelStrike - A CS-like FPS game built on Adaptive Entity Engine
//!
//! Features:
//! - FPS camera with mouse look
//! - Multiple weapons (AK-47, M4A1, AWP, Deagle, Knife) with 3D models
//! - Simple de_dust inspired map with 4K textures
//! - Enemy bots with detailed 3D models (T/CT)
//! - PBR lighting with shadows and reflections
//! - Glass materials with refraction
//! - Health, armor, and ammo system
//! - HUD with crosshair, health bar, ammo counter, FPS display

pub mod camera;
pub mod enemies;
pub mod graphics;
pub mod hud;
pub mod map;
pub mod player;
pub mod renderer;
pub mod weapons;

pub use camera::{FPSCamera, MoveDirection};
pub use enemies::{Enemy, EnemyManager, Team};
pub use graphics::{AdvancedLighting, CharacterModel, WeaponModel, TextureGenerator, ShadowSystem, Material};
pub use hud::{GameHUD, HudRenderData};
pub use map::{GameMap, MapBlock, AABB, Vertex};
pub use player::Player;
pub use renderer::GameRenderer;
pub use weapons::{Weapon, WeaponInventory, WeaponType};
