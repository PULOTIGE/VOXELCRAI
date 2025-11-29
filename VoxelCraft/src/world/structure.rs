// VoxelCraft - Structure Generation

use super::{Chunk, BlockType, CHUNK_SIZE};

/// Structure templates for world generation
pub struct StructureGenerator;

impl StructureGenerator {
    /// Generate a simple house
    pub fn generate_house(chunk: &mut Chunk, x: i32, y: i32, z: i32) {
        let width = 7;
        let depth = 7;
        let height = 4;

        // Floor
        for dx in 0..width {
            for dz in 0..depth {
                Self::set_if_valid(chunk, x + dx, y, z + dz, BlockType::Planks);
            }
        }

        // Walls
        for dy in 1..=height {
            for dx in 0..width {
                Self::set_if_valid(chunk, x + dx, y + dy, z, BlockType::Planks);
                Self::set_if_valid(chunk, x + dx, y + dy, z + depth - 1, BlockType::Planks);
            }
            for dz in 0..depth {
                Self::set_if_valid(chunk, x, y + dy, z + dz, BlockType::Planks);
                Self::set_if_valid(chunk, x + width - 1, y + dy, z + dz, BlockType::Planks);
            }
        }

        // Clear interior
        for dy in 1..=height {
            for dx in 1..(width - 1) {
                for dz in 1..(depth - 1) {
                    Self::set_if_valid(chunk, x + dx, y + dy, z + dz, BlockType::Air);
                }
            }
        }

        // Door
        Self::set_if_valid(chunk, x + width / 2, y + 1, z, BlockType::Air);
        Self::set_if_valid(chunk, x + width / 2, y + 2, z, BlockType::Air);

        // Windows
        Self::set_if_valid(chunk, x + 2, y + 2, z, BlockType::Glass);
        Self::set_if_valid(chunk, x + width - 3, y + 2, z, BlockType::Glass);

        // Roof
        for dx in 0..width {
            for dz in 0..depth {
                Self::set_if_valid(chunk, x + dx, y + height + 1, z + dz, BlockType::Planks);
            }
        }

        // Torch inside
        Self::set_if_valid(chunk, x + width / 2, y + 2, z + depth / 2, BlockType::Torch);
    }

    /// Generate a well
    pub fn generate_well(chunk: &mut Chunk, x: i32, y: i32, z: i32) {
        // Base
        for dx in -1..=1 {
            for dz in -1..=1 {
                Self::set_if_valid(chunk, x + dx, y, z + dz, BlockType::Cobblestone);
            }
        }

        // Walls
        for dy in 1..=3 {
            Self::set_if_valid(chunk, x - 1, y + dy, z - 1, BlockType::Cobblestone);
            Self::set_if_valid(chunk, x + 1, y + dy, z - 1, BlockType::Cobblestone);
            Self::set_if_valid(chunk, x - 1, y + dy, z + 1, BlockType::Cobblestone);
            Self::set_if_valid(chunk, x + 1, y + dy, z + 1, BlockType::Cobblestone);
        }

        // Water inside
        Self::set_if_valid(chunk, x, y + 1, z, BlockType::Water);

        // Roof
        for dx in -1..=1 {
            Self::set_if_valid(chunk, x + dx, y + 4, z - 1, BlockType::Planks);
            Self::set_if_valid(chunk, x + dx, y + 4, z + 1, BlockType::Planks);
        }
    }

    /// Generate a mineshaft entrance
    pub fn generate_mineshaft(chunk: &mut Chunk, x: i32, y: i32, z: i32) {
        // Entrance frame
        for dx in -1..=1 {
            Self::set_if_valid(chunk, x + dx, y, z, BlockType::Planks);
            Self::set_if_valid(chunk, x + dx, y + 3, z, BlockType::Planks);
        }
        Self::set_if_valid(chunk, x - 1, y + 1, z, BlockType::Wood);
        Self::set_if_valid(chunk, x - 1, y + 2, z, BlockType::Wood);
        Self::set_if_valid(chunk, x + 1, y + 1, z, BlockType::Wood);
        Self::set_if_valid(chunk, x + 1, y + 2, z, BlockType::Wood);

        // Clear entrance
        Self::set_if_valid(chunk, x, y + 1, z, BlockType::Air);
        Self::set_if_valid(chunk, x, y + 2, z, BlockType::Air);

        // Stairs going down
        for i in 1..10 {
            Self::set_if_valid(chunk, x, y + 1 - i, z + i, BlockType::Air);
            Self::set_if_valid(chunk, x, y + 2 - i, z + i, BlockType::Air);
            Self::set_if_valid(chunk, x, y - i, z + i, BlockType::Cobblestone);
        }

        // Torch at entrance
        Self::set_if_valid(chunk, x, y + 2, z - 1, BlockType::Torch);
    }

    fn set_if_valid(chunk: &mut Chunk, x: i32, y: i32, z: i32, block: BlockType) {
        if x >= 0 && x < CHUNK_SIZE && y >= 0 && y < 128 && z >= 0 && z < CHUNK_SIZE {
            chunk.set_block(x, y, z, block);
        }
    }
}

/// Dungeon generator
pub struct DungeonGenerator;

impl DungeonGenerator {
    pub fn generate(chunk: &mut Chunk, x: i32, y: i32, z: i32) {
        let size = 7;

        // Clear room
        for dx in 0..size {
            for dy in 0..4 {
                for dz in 0..size {
                    StructureGenerator::set_if_valid(chunk, x + dx, y + dy, z + dz, BlockType::Air);
                }
            }
        }

        // Floor
        for dx in 0..size {
            for dz in 0..size {
                StructureGenerator::set_if_valid(chunk, x + dx, y, z + dz, BlockType::Cobblestone);
            }
        }

        // Walls
        for dy in 1..=3 {
            for dx in 0..size {
                StructureGenerator::set_if_valid(chunk, x + dx, y + dy, z, BlockType::Cobblestone);
                StructureGenerator::set_if_valid(chunk, x + dx, y + dy, z + size - 1, BlockType::Cobblestone);
            }
            for dz in 0..size {
                StructureGenerator::set_if_valid(chunk, x, y + dy, z + dz, BlockType::Cobblestone);
                StructureGenerator::set_if_valid(chunk, x + size - 1, y + dy, z + dz, BlockType::Cobblestone);
            }
        }

        // Ceiling
        for dx in 0..size {
            for dz in 0..size {
                StructureGenerator::set_if_valid(chunk, x + dx, y + 4, z + dz, BlockType::Cobblestone);
            }
        }

        // Chest
        StructureGenerator::set_if_valid(chunk, x + size / 2, y + 1, z + size / 2, BlockType::Chest);

        // Torches
        StructureGenerator::set_if_valid(chunk, x + 1, y + 2, z + 1, BlockType::Torch);
        StructureGenerator::set_if_valid(chunk, x + size - 2, y + 2, z + 1, BlockType::Torch);
    }
}
