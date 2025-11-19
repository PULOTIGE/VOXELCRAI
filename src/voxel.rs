use bevy_ecs::prelude::*;
use half::f16;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Voxel component: 9-13 KB per voxel
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Voxel {
    // FP64 for energy/emotions (8 bytes)
    pub energy: f64,
    pub emotion_valence: f64,
    pub emotion_arousal: f64,
    pub emotion_dominance: f64,
    
    // FP16 for perception (2 bytes each, ~20 bytes total)
    pub perception_visual: f16,
    pub perception_auditory: f16,
    pub perception_tactile: f16,
    pub perception_thermal: f16,
    pub perception_chemical: f16,
    pub perception_pressure: f16,
    pub perception_time: f16,
    pub perception_space: f16,
    pub perception_self: f16,
    pub perception_other: f16,
    
    // INT8 for physics (1 byte each)
    pub velocity_x: i8,
    pub velocity_y: i8,
    pub velocity_z: i8,
    pub acceleration_x: i8,
    pub acceleration_y: i8,
    pub acceleration_z: i8,
    pub temperature: i8,
    pub pressure: i8,
    pub density: i8,
    pub elasticity: i8,
    pub friction: i8,
    pub viscosity: i8,
    
    // INT4 packed (2 bytes for 4 values)
    pub state_flags: u8, // 4-bit flags
    pub material_flags: u8, // 4-bit material flags
    
    // Genome: up to 10 concepts (strings, variable size, ~100-500 bytes)
    pub genome: Genome,
    
    // 16-byte echo + resonance f16 (18 bytes total)
    pub echo: [u8; 16],
    pub resonance: f16,
    
    // Position (12 bytes for i32 x3)
    pub position: [i32; 3],
    
    // Additional metadata (~100-200 bytes)
    pub metadata: HashMap<String, String>,
}

impl Voxel {
    pub fn new(position: [i32; 3]) -> Self {
        Self {
            energy: 0.0,
            emotion_valence: 0.0,
            emotion_arousal: 0.0,
            emotion_dominance: 0.0,
            perception_visual: f16::ZERO,
            perception_auditory: f16::ZERO,
            perception_tactile: f16::ZERO,
            perception_thermal: f16::ZERO,
            perception_chemical: f16::ZERO,
            perception_pressure: f16::ZERO,
            perception_time: f16::ZERO,
            perception_space: f16::ZERO,
            perception_self: f16::ZERO,
            perception_other: f16::ZERO,
            velocity_x: 0,
            velocity_y: 0,
            velocity_z: 0,
            acceleration_x: 0,
            acceleration_y: 0,
            acceleration_z: 0,
            temperature: 0,
            pressure: 0,
            density: 0,
            elasticity: 0,
            friction: 0,
            viscosity: 0,
            state_flags: 0,
            material_flags: 0,
            genome: Genome::new(),
            echo: [0; 16],
            resonance: f16::ZERO,
            position,
            metadata: HashMap::new(),
        }
    }
    
    pub fn size_bytes(&self) -> usize {
        // Approximate size calculation
        let base = std::mem::size_of::<Self>();
        let genome_size = self.genome.size_bytes();
        let metadata_size: usize = self.metadata.iter()
            .map(|(k, v)| k.len() + v.len() + 16)
            .sum();
        base + genome_size + metadata_size
    }
    
    pub fn get_energy_color(&self, max_energy: f64) -> [f32; 3] {
        let normalized = (self.energy / max_energy.max(1.0)).min(1.0) as f32;
        // Yellow = max energy (1.0, 1.0, 0.0)
        // Interpolate from black to yellow
        [normalized, normalized, 0.0]
    }
}

/// Genome: up to 10 concepts (strings)
#[derive(Clone, Serialize, Deserialize)]
pub struct Genome {
    pub concepts: Vec<String>,
    pub max_concepts: usize,
}

impl Genome {
    pub fn new() -> Self {
        Self {
            concepts: Vec::new(),
            max_concepts: 10,
        }
    }
    
    pub fn size_bytes(&self) -> usize {
        self.concepts.iter().map(|s| s.len() + 8).sum::<usize>() + 16
    }
    
    pub fn add_concept(&mut self, concept: String) -> bool {
        if self.concepts.len() < self.max_concepts {
            self.concepts.push(concept);
            true
        } else {
            false
        }
    }
}

/// Voxel World System
#[derive(Resource)]
pub struct VoxelWorld {
    pub voxels: Vec<Entity>,
    pub world: World,
    pub max_points: usize,
    pub trauma_mode: bool,
}

impl VoxelWorld {
    pub fn new() -> Self {
        let mut world = World::new();
        let voxels = Vec::new();
        
        Self {
            voxels,
            world,
            max_points: 1_500_000_000, // 1.5 billion points
            trauma_mode: false,
        }
    }
    
    pub fn add_voxel(&mut self, position: [i32; 3]) -> Entity {
        let entity = self.world.spawn(Voxel::new(position));
        self.voxels.push(entity);
        entity
    }
    
    pub fn update(&mut self, delta_time: f32) {
        // Update voxel physics and evolution
        let mut query = self.world.query::<&mut Voxel>();
        for mut voxel in query.iter_mut(&mut self.world) {
            // Update physics
            voxel.position[0] += voxel.velocity_x as i32;
            voxel.position[1] += voxel.velocity_y as i32;
            voxel.position[2] += voxel.velocity_z as i32;
            
            // Update energy based on resonance
            voxel.energy += voxel.resonance.to_f32() as f64 * delta_time as f64;
            
            // Apply trauma mode intensity
            if self.trauma_mode {
                voxel.energy *= 1.5;
                voxel.emotion_arousal *= 1.3;
            }
        }
    }
    
    pub fn get_point_cloud_data(&self) -> Vec<([f32; 3], [f32; 3])> {
        let mut points = Vec::new();
        let query = self.world.query::<&Voxel>();
        
        let max_energy = query.iter(&self.world)
            .map(|v| v.energy)
            .fold(0.0, f64::max);
        
        for voxel in query.iter(&self.world) {
            let pos = [
                voxel.position[0] as f32,
                voxel.position[1] as f32,
                voxel.position[2] as f32,
            ];
            let color = voxel.get_energy_color(max_energy);
            points.push((pos, color));
        }
        
        points
    }
}

impl Default for VoxelWorld {
    fn default() -> Self {
        Self::new()
    }
}
