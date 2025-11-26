// VoxelCraft - Entity System

use crate::world::World;
use crate::player::Player;
use glam::Vec3;
use serde::{Serialize, Deserialize};

/// Entity trait
pub trait Entity: Send + Sync {
    fn update(&mut self, dt: f32, world: &World, player: &Player);
    fn position(&self) -> Vec3;
    fn is_alive(&self) -> bool;
    fn entity_type(&self) -> EntityType;
    fn take_damage(&mut self, amount: f32);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum EntityType {
    // Hostile
    Zombie,
    Skeleton,
    Spider,
    Creeper,
    
    // Passive
    Pig,
    Cow,
    Sheep,
    Chicken,
    
    // Items
    DroppedItem,
}

impl EntityType {
    pub fn is_hostile(&self) -> bool {
        matches!(self, EntityType::Zombie | EntityType::Skeleton | EntityType::Spider | EntityType::Creeper)
    }

    pub fn is_passive(&self) -> bool {
        matches!(self, EntityType::Pig | EntityType::Cow | EntityType::Sheep | EntityType::Chicken)
    }
}

/// Mob entity
#[derive(Clone, Serialize, Deserialize)]
pub struct Mob {
    pub entity_type: EntityType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: f32,
    pub health: f32,
    pub max_health: f32,
    ai_state: MobAIState,
    target_pos: Option<Vec3>,
    attack_cooldown: f32,
    wander_timer: f32,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum MobAIState {
    Idle,
    Wandering,
    Chasing,
    Attacking,
    Fleeing,
}

impl Mob {
    pub fn new(entity_type: EntityType, position: Vec3) -> Self {
        let max_health = match entity_type {
            EntityType::Zombie => 20.0,
            EntityType::Skeleton => 20.0,
            EntityType::Spider => 16.0,
            EntityType::Creeper => 20.0,
            EntityType::Pig | EntityType::Cow | EntityType::Sheep => 10.0,
            EntityType::Chicken => 4.0,
            _ => 10.0,
        };

        Self {
            entity_type,
            position,
            velocity: Vec3::ZERO,
            rotation: 0.0,
            health: max_health,
            max_health,
            ai_state: MobAIState::Idle,
            target_pos: None,
            attack_cooldown: 0.0,
            wander_timer: 0.0,
        }
    }

    fn update_ai(&mut self, dt: f32, world: &World, player: &Player) {
        let to_player = player.position - self.position;
        let distance = to_player.length();

        // Update cooldowns
        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }
        self.wander_timer -= dt;

        // AI behavior based on entity type
        if self.entity_type.is_hostile() {
            self.hostile_ai(dt, distance, to_player, player);
        } else {
            self.passive_ai(dt, distance, to_player);
        }
    }

    fn hostile_ai(&mut self, dt: f32, distance: f32, to_player: Vec3, player: &Player) {
        let detection_range = match self.entity_type {
            EntityType::Zombie => 35.0,
            EntityType::Skeleton => 16.0,
            EntityType::Spider => 16.0,
            EntityType::Creeper => 16.0,
            _ => 16.0,
        };

        let attack_range = match self.entity_type {
            EntityType::Skeleton => 15.0,
            _ => 2.0,
        };

        if distance < detection_range {
            self.ai_state = MobAIState::Chasing;
            self.target_pos = Some(player.position);

            if distance < attack_range {
                self.ai_state = MobAIState::Attacking;
            }
        } else if self.wander_timer <= 0.0 {
            self.ai_state = MobAIState::Wandering;
            self.target_pos = Some(self.position + Vec3::new(
                (rand::random::<f32>() - 0.5) * 10.0,
                0.0,
                (rand::random::<f32>() - 0.5) * 10.0,
            ));
            self.wander_timer = 3.0 + rand::random::<f32>() * 5.0;
        }

        // Move towards target
        if let Some(target) = self.target_pos {
            let dir = (target - self.position).normalize_or_zero();
            let speed = match self.entity_type {
                EntityType::Spider => 3.5,
                EntityType::Creeper => 2.5,
                _ => 3.0,
            };

            self.velocity.x = dir.x * speed;
            self.velocity.z = dir.z * speed;
            self.rotation = dir.z.atan2(dir.x);
        }
    }

    fn passive_ai(&mut self, dt: f32, distance: f32, to_player: Vec3) {
        // Flee if damaged recently
        if self.health < self.max_health && distance < 10.0 {
            self.ai_state = MobAIState::Fleeing;
            let flee_dir = -to_player.normalize_or_zero();
            self.velocity.x = flee_dir.x * 4.0;
            self.velocity.z = flee_dir.z * 4.0;
            return;
        }

        // Random wandering
        if self.wander_timer <= 0.0 {
            self.ai_state = MobAIState::Wandering;
            self.target_pos = Some(self.position + Vec3::new(
                (rand::random::<f32>() - 0.5) * 8.0,
                0.0,
                (rand::random::<f32>() - 0.5) * 8.0,
            ));
            self.wander_timer = 2.0 + rand::random::<f32>() * 6.0;
        }

        if let Some(target) = self.target_pos {
            let dir = (target - self.position).normalize_or_zero();
            let speed = 1.5;

            self.velocity.x = dir.x * speed;
            self.velocity.z = dir.z * speed;
            self.rotation = dir.z.atan2(dir.x);

            // Stop if reached target
            if (target - self.position).length() < 1.0 {
                self.velocity = Vec3::ZERO;
                self.ai_state = MobAIState::Idle;
            }
        }
    }

    fn apply_physics(&mut self, dt: f32, world: &World) {
        // Gravity
        self.velocity.y -= 20.0 * dt;

        // Apply velocity
        let new_pos = self.position + self.velocity * dt;

        // Simple ground collision
        let ground_y = world.get_height_at(new_pos.x as i32, new_pos.z as i32) as f32 + 1.0;
        
        if new_pos.y < ground_y {
            self.position.y = ground_y;
            self.velocity.y = 0.0;
        } else {
            self.position.y = new_pos.y;
        }

        // Horizontal movement
        if !world.is_solid(new_pos.x as i32, self.position.y as i32, self.position.z as i32) {
            self.position.x = new_pos.x;
        }
        if !world.is_solid(self.position.x as i32, self.position.y as i32, new_pos.z as i32) {
            self.position.z = new_pos.z;
        }

        // Friction
        self.velocity.x *= 0.8;
        self.velocity.z *= 0.8;
    }

    pub fn attack_damage(&self) -> f32 {
        match self.entity_type {
            EntityType::Zombie => 3.0,
            EntityType::Skeleton => 4.0,
            EntityType::Spider => 2.0,
            EntityType::Creeper => 0.0, // Explodes instead
            _ => 0.0,
        }
    }

    pub fn can_attack(&self) -> bool {
        self.attack_cooldown <= 0.0 && self.ai_state == MobAIState::Attacking
    }

    pub fn do_attack(&mut self) {
        self.attack_cooldown = 1.0;
    }
}

impl Entity for Mob {
    fn update(&mut self, dt: f32, world: &World, player: &Player) {
        self.update_ai(dt, world, player);
        self.apply_physics(dt, world);
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    fn entity_type(&self) -> EntityType {
        self.entity_type
    }

    fn take_damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
    }
}

/// AI behavior system
pub struct MobAI;

impl MobAI {
    pub fn spawn_mobs_in_area(world: &World, center: Vec3, radius: f32, is_night: bool) -> Vec<Mob> {
        let mut mobs = Vec::new();
        let count = if is_night { 5 } else { 2 };

        for _ in 0..count {
            let offset = Vec3::new(
                (rand::random::<f32>() - 0.5) * radius * 2.0,
                0.0,
                (rand::random::<f32>() - 0.5) * radius * 2.0,
            );

            let pos = center + offset;
            let ground_y = world.get_height_at(pos.x as i32, pos.z as i32) as f32 + 1.0;

            if ground_y < 1.0 {
                continue;
            }

            let entity_type = if is_night {
                match rand::random::<u32>() % 4 {
                    0 => EntityType::Zombie,
                    1 => EntityType::Skeleton,
                    2 => EntityType::Spider,
                    _ => EntityType::Creeper,
                }
            } else {
                match rand::random::<u32>() % 4 {
                    0 => EntityType::Pig,
                    1 => EntityType::Cow,
                    2 => EntityType::Sheep,
                    _ => EntityType::Chicken,
                }
            };

            mobs.push(Mob::new(entity_type, Vec3::new(pos.x, ground_y, pos.z)));
        }

        mobs
    }
}
