// Voxel Agent System with FSM behaviors, spatial hash, GPU updates, and LOD
use glam::Vec3;
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use rand::Rng;

/// Agent FSM State
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AgentState {
    Wander,
    Follow,
    Interact,
    Idle,
}

/// Agent component with FSM behavior
#[derive(Component, Clone)]
pub struct Agent {
    pub position: Vec3,
    pub velocity: Vec3,
    pub state: AgentState,
    pub target: Option<Vec3>,
    pub target_entity: Option<Entity>,
    pub speed: f32,
    pub lod_level: u32, // 0 = full detail, higher = lower detail
    pub distance_to_camera: f32,
}

impl Agent {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::ZERO,
            state: AgentState::Wander,
            target: None,
            target_entity: None,
            speed: 2.0,
            lod_level: 0,
            distance_to_camera: 0.0,
        }
    }

    /// Update agent based on current FSM state
    pub fn update(&mut self, delta_time: f32, nearby_agents: &[&Agent]) {
        match self.state {
            AgentState::Wander => self.update_wander(delta_time),
            AgentState::Follow => self.update_follow(delta_time),
            AgentState::Interact => self.update_interact(delta_time),
            AgentState::Idle => self.update_idle(delta_time),
        }

        // Update position
        self.position += self.velocity * delta_time;

        // Check for collisions with nearby agents
        for other in nearby_agents {
            let distance = (self.position - other.position).length();
            if distance < 0.5 && distance > 0.0 {
                let direction = (self.position - other.position).normalize();
                self.velocity += direction * 0.5;
            }
        }

        // Damping
        self.velocity *= 0.9;
    }

    fn update_wander(&mut self, delta_time: f32) {
        // Random wandering behavior
        if self.target.is_none() || (self.position - self.target.unwrap()).length() < 0.5 {
            let mut rng = rand::thread_rng();
            self.target = Some(self.position + Vec3::new(
                (rng.gen::<f32>() - 0.5) * 10.0,
                0.0,
                (rng.gen::<f32>() - 0.5) * 10.0,
            ));
        }

        if let Some(target) = self.target {
            let direction = (target - self.position).normalize();
            self.velocity += direction * self.speed * delta_time;
        }
    }

    fn update_follow(&mut self, delta_time: f32) {
        if let Some(target) = self.target {
            let direction = (target - self.position).normalize();
            self.velocity += direction * self.speed * 1.5 * delta_time;
        } else {
            self.state = AgentState::Wander;
        }
    }

    fn update_interact(&mut self, _delta_time: f32) {
        // Interaction behavior
        if let Some(target) = self.target {
            let distance = (self.position - target).length();
            if distance < 1.0 {
                // Interact with target
                self.state = AgentState::Wander;
            }
        } else {
            self.state = AgentState::Wander;
        }
    }

    fn update_idle(&mut self, _delta_time: f32) {
        // Idle behavior - slow down
        self.velocity *= 0.95;
    }

    /// Set LOD based on distance to camera
    pub fn update_lod(&mut self, camera_pos: Vec3) {
        self.distance_to_camera = (self.position - camera_pos).length();
        
        if self.distance_to_camera > 100.0 {
            self.lod_level = 3;
        } else if self.distance_to_camera > 50.0 {
            self.lod_level = 2;
        } else if self.distance_to_camera > 20.0 {
            self.lod_level = 1;
        } else {
            self.lod_level = 0;
        }
    }
}

/// Spatial Hash for efficient collision detection
pub struct SpatialHash {
    cell_size: f32,
    cells: HashMap<(i32, i32, i32), Vec<Entity>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    /// Get cell coordinates for a position
    fn get_cell(&self, pos: Vec3) -> (i32, i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }

    /// Insert agent into spatial hash
    pub fn insert(&mut self, entity: Entity, position: Vec3) {
        let cell = self.get_cell(position);
        self.cells.entry(cell).or_insert_with(Vec::new).push(entity);
    }

    /// Get nearby entities
    pub fn get_nearby(&self, position: Vec3, radius: f32) -> Vec<Entity> {
        let mut result = Vec::new();
        let cell_radius = (radius / self.cell_size).ceil() as i32;
        let center_cell = self.get_cell(position);

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                for dz in -cell_radius..=cell_radius {
                    let cell = (
                        center_cell.0 + dx,
                        center_cell.1 + dy,
                        center_cell.2 + dz,
                    );
                    if let Some(entities) = self.cells.get(&cell) {
                        result.extend(entities.iter().copied());
                    }
                }
            }
        }

        result
    }

    /// Clear spatial hash
    pub fn clear(&mut self) {
        self.cells.clear();
    }
}

/// Agent System for managing agents
pub struct AgentSystem {
    pub spatial_hash: SpatialHash,
    pub max_agents: usize,
}

impl AgentSystem {
    pub fn new(max_agents: usize) -> Self {
        Self {
            spatial_hash: SpatialHash::new(5.0), // 5 unit cells
            max_agents,
        }
    }

    /// Update all agents
    pub fn update(&mut self, world: &mut World, camera_pos: Vec3, delta_time: f32) {
        // Update spatial hash
        self.spatial_hash.clear();
        
        // Update spatial hash - collect entities first
        let mut entities: Vec<(Entity, Vec3)> = Vec::new();
        {
            let mut agent_query = world.query::<(bevy_ecs::entity::Entity, &Agent)>();
            for (entity, agent) in agent_query.iter(world) {
                entities.push((entity, agent.position));
            }
        }
        
        for (entity, position) in &entities {
            self.spatial_hash.insert(*entity, *position);
        }

        // Update agents - collect first to avoid borrowing issues
        let mut agents_data: Vec<(Entity, Agent)> = Vec::new();
        {
            let mut agent_query = world.query::<(bevy_ecs::entity::Entity, &Agent)>();
            for (entity, agent) in agent_query.iter(world) {
                agents_data.push((entity, agent.clone()));
            }
        }

        // Create a map for quick lookup
        let agents_map: std::collections::HashMap<Entity, Agent> = 
            agents_data.iter().map(|(e, a)| (*e, a.clone())).collect();

        for (entity, mut agent) in agents_data {
            // Update LOD
            agent.update_lod(camera_pos);

            // Get nearby agents
            let nearby_entities = self.spatial_hash.get_nearby(agent.position, 10.0);
            let nearby_agents: Vec<&Agent> = nearby_entities
                .iter()
                .filter_map(|&e| {
                    if e != entity {
                        agents_map.get(&e).or_else(|| world.get::<Agent>(e))
                    } else {
                        None
                    }
                })
                .collect();

            // Update agent
            agent.update(delta_time, &nearby_agents);

            // Write back
            if let Some(mut agent_mut) = world.get_mut::<Agent>(entity) {
                *agent_mut = agent;
            }
        }
    }
}

