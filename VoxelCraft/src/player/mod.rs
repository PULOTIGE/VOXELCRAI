// VoxelCraft - Player System

mod inventory;
mod stats;

pub use inventory::*;
pub use stats::*;

use crate::world::{World, BlockType};
use glam::Vec3;
use serde::{Serialize, Deserialize};

/// Player entity
#[derive(Serialize, Deserialize)]
pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: (f32, f32), // (yaw, pitch)
    pub inventory: Inventory,
    pub stats: PlayerStats,
    pub is_grounded: bool,
    pub is_swimming: bool,
    pub selected_slot: usize,
    #[serde(skip)]
    pub is_sprinting: bool,
    #[serde(skip)]
    pub is_sneaking: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub const HEIGHT: f32 = 1.8;
    pub const EYE_HEIGHT: f32 = 1.62;
    pub const WIDTH: f32 = 0.6;
    pub const WALK_SPEED: f32 = 4.3;
    pub const SPRINT_SPEED: f32 = 5.6;
    pub const SWIM_SPEED: f32 = 2.0;
    pub const JUMP_VELOCITY: f32 = 8.0;
    pub const GRAVITY: f32 = 28.0;

    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 64.0, 0.0),
            velocity: Vec3::ZERO,
            rotation: (0.0, 0.0),
            inventory: Inventory::new(),
            stats: PlayerStats::new(),
            is_grounded: false,
            is_swimming: false,
            selected_slot: 0,
            is_sprinting: false,
            is_sneaking: false,
        }
    }

    pub fn update(&mut self, dt: f32, world: &World) {
        // Update stats
        self.stats.update(dt, self.is_sprinting);

        // Check if swimming
        self.is_swimming = world.get_block(
            self.position.x as i32,
            (self.position.y + Self::EYE_HEIGHT * 0.5) as i32,
            self.position.z as i32,
        ) == Some(BlockType::Water);

        // Apply gravity
        if !self.is_grounded && !self.is_swimming {
            self.velocity.y -= Self::GRAVITY * dt;
        } else if self.is_swimming {
            self.velocity.y = self.velocity.y.max(-2.0);
        }

        // Apply velocity
        let new_pos = self.position + self.velocity * dt;

        // Collision detection
        self.position = self.resolve_collisions(new_pos, world);

        // Ground check
        let ground_y = (self.position.y - 0.1) as i32;
        self.is_grounded = world.is_solid(
            self.position.x as i32,
            ground_y,
            self.position.z as i32,
        );

        if self.is_grounded {
            self.velocity.y = 0.0;
        }

        // Friction
        self.velocity.x *= 0.9;
        self.velocity.z *= 0.9;

        // Clamp position
        self.position.y = self.position.y.max(0.0);
    }

    pub fn move_direction(&mut self, dir: Vec3, dt: f32) {
        let speed = if self.is_swimming {
            Self::SWIM_SPEED
        } else if self.is_sprinting {
            Self::SPRINT_SPEED
        } else {
            Self::WALK_SPEED
        };

        // Transform direction by yaw
        let yaw = self.rotation.0;
        let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
        let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());

        let movement = (forward * dir.z + right * dir.x).normalize_or_zero() * speed;

        self.velocity.x = movement.x;
        self.velocity.z = movement.z;

        if self.is_swimming && dir.y != 0.0 {
            self.velocity.y = dir.y * Self::SWIM_SPEED;
        }
    }

    pub fn jump(&mut self) {
        if self.is_grounded {
            self.velocity.y = Self::JUMP_VELOCITY;
            self.is_grounded = false;
        } else if self.is_swimming {
            self.velocity.y = Self::SWIM_SPEED;
        }
    }

    pub fn rotate(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.rotation.0 += yaw_delta;
        self.rotation.1 = (self.rotation.1 + pitch_delta).clamp(-1.5, 1.5);
    }

    pub fn get_eye_position(&self) -> Vec3 {
        self.position + Vec3::new(0.0, Self::EYE_HEIGHT, 0.0)
    }

    pub fn get_look_direction(&self) -> Vec3 {
        let (yaw, pitch) = self.rotation;
        Vec3::new(
            -yaw.sin() * pitch.cos(),
            -pitch.sin(),
            -yaw.cos() * pitch.cos(),
        ).normalize()
    }

    fn resolve_collisions(&self, new_pos: Vec3, world: &World) -> Vec3 {
        let mut pos = new_pos;
        let half_width = Self::WIDTH / 2.0;

        // Check Y collision
        for y_offset in [0.0, Self::HEIGHT * 0.5, Self::HEIGHT] {
            let check_y = (pos.y + y_offset) as i32;
            
            if world.is_solid(pos.x as i32, check_y, pos.z as i32) {
                if self.velocity.y > 0.0 {
                    pos.y = check_y as f32 - Self::HEIGHT - 0.01;
                } else {
                    pos.y = check_y as f32 + 1.0;
                }
            }
        }

        // Check X collision
        for y_offset in [0.5, Self::HEIGHT - 0.1] {
            let check_y = (pos.y + y_offset) as i32;
            
            if world.is_solid((pos.x + half_width) as i32, check_y, pos.z as i32) {
                pos.x = (pos.x + half_width).floor() - half_width - 0.01;
            }
            if world.is_solid((pos.x - half_width) as i32, check_y, pos.z as i32) {
                pos.x = (pos.x - half_width).ceil() + half_width + 0.01;
            }
        }

        // Check Z collision
        for y_offset in [0.5, Self::HEIGHT - 0.1] {
            let check_y = (pos.y + y_offset) as i32;
            
            if world.is_solid(pos.x as i32, check_y, (pos.z + half_width) as i32) {
                pos.z = (pos.z + half_width).floor() - half_width - 0.01;
            }
            if world.is_solid(pos.x as i32, check_y, (pos.z - half_width) as i32) {
                pos.z = (pos.z - half_width).ceil() + half_width + 0.01;
            }
        }

        pos
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.stats.health = (self.stats.health - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.stats.health = (self.stats.health + amount).min(self.stats.max_health);
    }

    pub fn eat(&mut self, food: f32, saturation: f32) {
        self.stats.hunger = (self.stats.hunger + food).min(20.0);
        self.stats.saturation = (self.stats.saturation + saturation).min(self.stats.hunger);
    }

    pub fn is_alive(&self) -> bool {
        self.stats.health > 0.0
    }

    pub fn respawn(&mut self) {
        self.position = Vec3::new(0.0, 64.0, 0.0);
        self.velocity = Vec3::ZERO;
        self.stats = PlayerStats::new();
    }
}
