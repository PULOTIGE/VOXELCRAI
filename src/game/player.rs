//! Player state and physics

use glam::Vec3;
use super::camera::{FPSCamera, MoveDirection};
use super::weapons::WeaponInventory;
use super::map::GameMap;

pub struct Player {
    pub camera: FPSCamera,
    pub weapons: WeaponInventory,
    pub health: f32,
    pub max_health: f32,
    pub armor: f32,
    pub max_armor: f32,
    pub is_alive: bool,
    pub money: u32,
    pub kills: u32,
    pub deaths: u32,
    pub movement_speed: f32,
    pub is_crouching: bool,
    pub is_sprinting: bool,
    // Physics
    pub velocity: Vec3,
    pub on_ground: bool,
    pub player_height: f32,
    pub player_radius: f32,
}

impl Player {
    pub fn new(spawn_position: Vec3) -> Self {
        let mut camera = FPSCamera::new(spawn_position + Vec3::new(0.0, 1.6, 0.0));
        camera.speed = 8.0;
        
        Self {
            camera,
            weapons: WeaponInventory::new(),
            health: 100.0,
            max_health: 100.0,
            armor: 0.0,
            max_armor: 100.0,
            is_alive: true,
            money: 800,
            kills: 0,
            deaths: 0,
            movement_speed: 8.0,
            is_crouching: false,
            is_sprinting: false,
            velocity: Vec3::ZERO,
            on_ground: true,
            player_height: 1.8,
            player_radius: 0.4,
        }
    }

    pub fn take_damage(&mut self, damage: f32) {
        if !self.is_alive {
            return;
        }

        let mut remaining_damage = damage;

        // Armor absorbs 50% of damage
        if self.armor > 0.0 {
            let armor_absorption = (remaining_damage * 0.5).min(self.armor);
            self.armor -= armor_absorption;
            remaining_damage -= armor_absorption;
        }

        self.health -= remaining_damage;
        
        if self.health <= 0.0 {
            self.health = 0.0;
            self.is_alive = false;
            self.deaths += 1;
        }
    }

    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn add_armor(&mut self, amount: f32) {
        self.armor = (self.armor + amount).min(self.max_armor);
    }

    pub fn respawn(&mut self, position: Vec3) {
        self.camera.position = position + Vec3::new(0.0, 1.6, 0.0);
        self.health = self.max_health;
        self.armor = 0.0;
        self.is_alive = true;
        self.velocity = Vec3::ZERO;
        self.weapons = WeaponInventory::new();
    }

    pub fn get_position(&self) -> Vec3 {
        self.camera.position - Vec3::new(0.0, 1.6, 0.0)
    }

    pub fn update(&mut self, delta_time: f32, map: &GameMap) {
        if !self.is_alive {
            return;
        }

        // Apply gravity
        if !self.on_ground {
            self.velocity.y -= 20.0 * delta_time;
        }

        // Apply velocity
        let new_pos = self.camera.position + self.velocity * delta_time;
        
        // Check ground collision
        let ground_check_pos = Vec3::new(new_pos.x, 0.5, new_pos.z);
        if new_pos.y <= 1.6 {
            self.camera.position.y = 1.6;
            self.velocity.y = 0.0;
            self.on_ground = true;
        } else {
            self.on_ground = false;
        }

        // Simple wall collision
        let feet_pos = self.camera.position - Vec3::new(0.0, 1.4, 0.0);
        if !map.check_collision(feet_pos, self.player_radius) {
            // Can move
        } else {
            self.velocity = Vec3::ZERO;
        }

        // Friction when on ground
        if self.on_ground {
            self.velocity.x *= 0.9;
            self.velocity.z *= 0.9;
        }
    }

    pub fn jump(&mut self) {
        if self.on_ground && self.is_alive {
            self.velocity.y = 8.0;
            self.on_ground = false;
        }
    }

    pub fn process_movement(&mut self, direction: MoveDirection, delta_time: f32, map: &GameMap) {
        if !self.is_alive {
            return;
        }

        let speed = if self.is_sprinting {
            self.movement_speed * 1.4
        } else if self.is_crouching {
            self.movement_speed * 0.5
        } else {
            self.movement_speed
        };

        let old_speed = self.camera.speed;
        self.camera.speed = speed;

        // Calculate new position
        let old_pos = self.camera.position;
        self.camera.process_keyboard(direction, delta_time);
        
        // Check collision
        let feet_pos = self.camera.position - Vec3::new(0.0, 1.4, 0.0);
        if map.check_collision(feet_pos, self.player_radius) {
            // Blocked, revert
            self.camera.position = old_pos;
        }

        self.camera.speed = old_speed;
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new(Vec3::new(0.0, 0.0, 0.0))
    }
}
