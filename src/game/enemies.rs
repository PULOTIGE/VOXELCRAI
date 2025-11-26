//! Enemy bots with simple AI

use glam::Vec3;
use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Team {
    Terrorist,
    CounterTerrorist,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BotState {
    Idle,
    Patrol,
    Chase,
    Attack,
    Dead,
}

#[derive(Clone, Debug)]
pub struct Enemy {
    pub id: u32,
    pub position: Vec3,
    pub rotation_y: f32,
    pub health: f32,
    pub max_health: f32,
    pub team: Team,
    pub state: BotState,
    pub target_position: Vec3,
    pub speed: f32,
    pub attack_range: f32,
    pub attack_damage: f32,
    pub attack_cooldown: f32,
    pub last_attack_time: f32,
    pub patrol_points: Vec<Vec3>,
    pub current_patrol_index: usize,
    pub size: f32, // Collision radius
}

impl Enemy {
    pub fn new(id: u32, position: Vec3, team: Team) -> Self {
        Self {
            id,
            position,
            rotation_y: 0.0,
            health: 100.0,
            max_health: 100.0,
            team,
            state: BotState::Patrol,
            target_position: position,
            speed: 5.0,
            attack_range: 25.0,
            attack_damage: 10.0,
            attack_cooldown: 0.5,
            last_attack_time: 0.0,
            patrol_points: Vec::new(),
            current_patrol_index: 0,
            size: 0.5,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0 && self.state != BotState::Dead
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.health -= damage;
        if self.health <= 0.0 {
            self.health = 0.0;
            self.state = BotState::Dead;
        }
    }

    pub fn get_color(&self) -> [f32; 3] {
        if !self.is_alive() {
            return [0.3, 0.3, 0.3]; // Gray when dead
        }
        match self.team {
            Team::Terrorist => [0.8, 0.4, 0.1],        // Orange
            Team::CounterTerrorist => [0.2, 0.4, 0.9], // Blue
        }
    }

    pub fn update(&mut self, player_pos: Vec3, current_time: f32, delta_time: f32) {
        if !self.is_alive() {
            return;
        }

        let distance_to_player = (player_pos - self.position).length();

        // State machine
        match self.state {
            BotState::Idle => {
                if distance_to_player < 40.0 {
                    self.state = BotState::Chase;
                }
            }
            BotState::Patrol => {
                if distance_to_player < 30.0 {
                    self.state = BotState::Chase;
                } else if !self.patrol_points.is_empty() {
                    self.move_towards_target(delta_time);
                    
                    let dist_to_target = (self.target_position - self.position).length();
                    if dist_to_target < 2.0 {
                        self.current_patrol_index = (self.current_patrol_index + 1) % self.patrol_points.len();
                        self.target_position = self.patrol_points[self.current_patrol_index];
                    }
                }
            }
            BotState::Chase => {
                self.target_position = player_pos;
                self.move_towards_target(delta_time);

                if distance_to_player < self.attack_range {
                    self.state = BotState::Attack;
                } else if distance_to_player > 50.0 {
                    self.state = BotState::Patrol;
                }
            }
            BotState::Attack => {
                self.look_at(player_pos);
                
                if distance_to_player > self.attack_range * 1.2 {
                    self.state = BotState::Chase;
                }
            }
            BotState::Dead => {}
        }
    }

    fn move_towards_target(&mut self, delta_time: f32) {
        let direction = (self.target_position - self.position).normalize_or_zero();
        let horizontal_dir = Vec3::new(direction.x, 0.0, direction.z).normalize_or_zero();
        
        if horizontal_dir.length() > 0.1 {
            self.position += horizontal_dir * self.speed * delta_time;
            self.look_at(self.target_position);
        }
    }

    fn look_at(&mut self, target: Vec3) {
        let dir = target - self.position;
        self.rotation_y = dir.z.atan2(dir.x);
    }

    pub fn can_attack(&self, current_time: f32) -> bool {
        self.is_alive() 
            && self.state == BotState::Attack 
            && current_time - self.last_attack_time >= self.attack_cooldown
    }

    pub fn attack(&mut self, current_time: f32) -> f32 {
        self.last_attack_time = current_time;
        self.attack_damage
    }

    /// Check if position collides with enemy hitbox
    pub fn check_hit(&self, ray_origin: Vec3, ray_dir: Vec3, max_dist: f32) -> Option<f32> {
        if !self.is_alive() {
            return None;
        }

        // Simple sphere collision for hitbox
        let to_enemy = self.position + Vec3::new(0.0, 1.0, 0.0) - ray_origin; // Center at chest height
        let hitbox_radius = 0.8;

        // Ray-sphere intersection
        let a = ray_dir.dot(ray_dir);
        let b = 2.0 * ray_dir.dot(-to_enemy);
        let c = to_enemy.dot(to_enemy) - hitbox_radius * hitbox_radius;
        
        let discriminant = b * b - 4.0 * a * c;
        
        if discriminant < 0.0 {
            return None;
        }

        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        
        if t > 0.0 && t < max_dist {
            Some(t)
        } else {
            None
        }
    }
}

pub struct EnemyManager {
    pub enemies: Vec<Enemy>,
    next_id: u32,
}

impl EnemyManager {
    pub fn new() -> Self {
        Self {
            enemies: Vec::new(),
            next_id: 0,
        }
    }

    pub fn spawn_enemy(&mut self, position: Vec3, team: Team) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let mut enemy = Enemy::new(id, position, team);
        
        // Add some patrol points
        let mut rng = rand::thread_rng();
        for _ in 0..4 {
            let patrol_point = position + Vec3::new(
                rng.gen_range(-15.0..15.0),
                0.0,
                rng.gen_range(-15.0..15.0),
            );
            enemy.patrol_points.push(patrol_point);
        }
        if !enemy.patrol_points.is_empty() {
            enemy.target_position = enemy.patrol_points[0];
        }
        
        self.enemies.push(enemy);
        id
    }

    pub fn update(&mut self, player_pos: Vec3, current_time: f32, delta_time: f32) {
        for enemy in &mut self.enemies {
            enemy.update(player_pos, current_time, delta_time);
        }
    }

    pub fn get_alive_enemies(&self) -> Vec<&Enemy> {
        self.enemies.iter().filter(|e| e.is_alive()).collect()
    }

    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<(f32, u32)> {
        let mut closest: Option<(f32, u32)> = None;

        for enemy in &self.enemies {
            if let Some(t) = enemy.check_hit(origin, direction, max_distance) {
                if closest.is_none() || t < closest.unwrap().0 {
                    closest = Some((t, enemy.id));
                }
            }
        }

        closest
    }

    pub fn damage_enemy(&mut self, id: u32, damage: f32) -> bool {
        if let Some(enemy) = self.enemies.iter_mut().find(|e| e.id == id) {
            enemy.take_damage(damage);
            return !enemy.is_alive();
        }
        false
    }

    pub fn spawn_initial_enemies(&mut self, spawn_points: &[Vec3], team: Team) {
        for (i, &pos) in spawn_points.iter().enumerate().take(5) {
            self.spawn_enemy(pos, team);
        }
    }
}

impl Default for EnemyManager {
    fn default() -> Self {
        Self::new()
    }
}
