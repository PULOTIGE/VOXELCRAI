// VoxelCraft - World System
// Voxel world with chunks, biomes, and procedural generation

mod chunk;
mod block;
mod biome;
mod generator;
mod structure;

pub use chunk::*;
pub use block::*;
pub use biome::*;
pub use generator::*;
pub use structure::*;

use std::collections::HashMap;
use glam::{Vec3, IVec3};

pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_HEIGHT: i32 = 128;

/// Voxel world
pub struct World {
    pub seed: u64,
    pub chunks: HashMap<(i32, i32), Chunk>,
    generator: WorldGenerator,
}

impl World {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            chunks: HashMap::new(),
            generator: WorldGenerator::new(seed),
        }
    }

    pub fn world_to_chunk(&self, pos: Vec3) -> (i32, i32) {
        (
            (pos.x / CHUNK_SIZE as f32).floor() as i32,
            (pos.z / CHUNK_SIZE as f32).floor() as i32,
        )
    }

    pub fn world_to_local(&self, x: i32, y: i32, z: i32) -> (i32, i32, (i32, i32, i32)) {
        let chunk_x = x.div_euclid(CHUNK_SIZE);
        let chunk_z = z.div_euclid(CHUNK_SIZE);
        let local_x = x.rem_euclid(CHUNK_SIZE);
        let local_z = z.rem_euclid(CHUNK_SIZE);
        (chunk_x, chunk_z, (local_x, y, local_z))
    }

    pub fn update_around(&mut self, center: (i32, i32), radius: i32) {
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let cx = center.0 + dx;
                let cz = center.1 + dz;
                
                if !self.chunks.contains_key(&(cx, cz)) {
                    let chunk = self.generator.generate_chunk(cx, cz);
                    self.chunks.insert((cx, cz), chunk);
                }
            }
        }

        // Unload distant chunks
        let max_dist = (radius + 2) * (radius + 2);
        self.chunks.retain(|&(cx, cz), _| {
            let dx = cx - center.0;
            let dz = cz - center.1;
            dx * dx + dz * dz <= max_dist
        });
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<BlockType> {
        if y < 0 || y >= CHUNK_HEIGHT {
            return None;
        }

        let (chunk_x, chunk_z, (local_x, local_y, local_z)) = self.world_to_local(x, y, z);
        
        self.chunks.get(&(chunk_x, chunk_z))
            .and_then(|chunk| Some(chunk.get_block(local_x, local_y, local_z)))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockType) {
        if y < 0 || y >= CHUNK_HEIGHT {
            return;
        }

        let (chunk_x, chunk_z, (local_x, local_y, local_z)) = self.world_to_local(x, y, z);
        
        if let Some(chunk) = self.chunks.get_mut(&(chunk_x, chunk_z)) {
            chunk.set_block(local_x, local_y, local_z, block);
            chunk.dirty = true;
        }
    }

    pub fn get_height_at(&self, x: i32, z: i32) -> i32 {
        for y in (0..CHUNK_HEIGHT).rev() {
            if let Some(block) = self.get_block(x, y, z) {
                if block != BlockType::Air {
                    return y;
                }
            }
        }
        0
    }

    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_dist: f32) -> (bool, IVec3, IVec3) {
        let step = direction.normalize() * 0.1;
        let mut pos = origin;
        let mut last_air = IVec3::ZERO;

        for _ in 0..(max_dist / 0.1) as i32 {
            let block_pos = IVec3::new(
                pos.x.floor() as i32,
                pos.y.floor() as i32,
                pos.z.floor() as i32,
            );

            if let Some(block) = self.get_block(block_pos.x, block_pos.y, block_pos.z) {
                if block != BlockType::Air && block != BlockType::Water {
                    // Calculate face normal
                    let face = last_air - block_pos;
                    return (true, block_pos, face);
                }
                last_air = block_pos;
            }

            pos += step;
        }

        (false, IVec3::ZERO, IVec3::ZERO)
    }

    pub fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        if let Some(block) = self.get_block(x, y, z) {
            block.is_solid()
        } else {
            y < 0
        }
    }

    pub fn get_light_level(&self, x: i32, y: i32, z: i32) -> f32 {
        // Simple sky light calculation
        let mut light: f32 = 0.0;
        
        for dy in 1..=CHUNK_HEIGHT - y {
            if self.is_solid(x, y + dy, z) {
                break;
            }
            light = 1.0;
        }

        // Add ambient light based on exposure
        light.max(0.1)
    }

    pub fn get_biome_at(&self, x: i32, z: i32) -> Biome {
        self.generator.get_biome(x, z)
    }
}
