// VoxelCraft - World Generator

use super::{Chunk, BlockType, Biome, CHUNK_SIZE, CHUNK_HEIGHT};
use noise::{NoiseFn, Perlin, Seedable, SuperSimplex};

pub struct WorldGenerator {
    seed: u64,
    terrain_noise: Perlin,
    detail_noise: SuperSimplex,
    biome_noise: Perlin,
    cave_noise: Perlin,
    ore_noise: SuperSimplex,
}

impl WorldGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            terrain_noise: Perlin::new(seed as u32),
            detail_noise: SuperSimplex::new(seed as u32 + 1),
            biome_noise: Perlin::new(seed as u32 + 2),
            cave_noise: Perlin::new(seed as u32 + 3),
            ore_noise: SuperSimplex::new(seed as u32 + 4),
        }
    }

    pub fn generate_chunk(&self, chunk_x: i32, chunk_z: i32) -> Chunk {
        let mut chunk = Chunk::new(chunk_x, chunk_z);
        let world_x = chunk_x * CHUNK_SIZE;
        let world_z = chunk_z * CHUNK_SIZE;

        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let x = world_x + lx;
                let z = world_z + lz;

                let biome = self.get_biome(x, z);
                let height = self.get_terrain_height(x, z, biome);
                let water_level = biome.water_level();

                // Generate terrain column
                self.generate_column(&mut chunk, lx, lz, height, water_level, biome);
            }
        }

        // Generate caves
        self.generate_caves(&mut chunk, world_x, world_z);

        // Generate ores
        self.generate_ores(&mut chunk, world_x, world_z);

        // Generate structures (trees, etc.)
        self.generate_structures(&mut chunk, world_x, world_z);

        chunk
    }

    fn generate_column(&self, chunk: &mut Chunk, x: i32, z: i32, height: i32, water_level: i32, biome: Biome) {
        // Bedrock
        chunk.set_block(x, 0, z, BlockType::Bedrock);

        // Stone layers
        for y in 1..height.min(CHUNK_HEIGHT - 4) {
            chunk.set_block(x, y, z, BlockType::Stone);
        }

        // Surface layers
        let surface_block = biome.surface_block();
        let subsurface = biome.subsurface_block();

        if height < CHUNK_HEIGHT - 1 {
            // Subsurface (3 blocks)
            for y in (height - 3).max(1)..height {
                chunk.set_block(x, y, z, subsurface);
            }

            // Surface
            if height > water_level {
                chunk.set_block(x, height, z, surface_block);
            } else {
                chunk.set_block(x, height, z, subsurface);
            }
        }

        // Water
        if height < water_level {
            for y in (height + 1)..=water_level {
                chunk.set_block(x, y, z, BlockType::Water);
            }
        }
    }

    fn generate_caves(&self, chunk: &mut Chunk, world_x: i32, world_z: i32) {
        let scale = 0.05;

        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                for y in 5..60 {
                    let x = world_x + lx;
                    let z = world_z + lz;

                    let cave_value = self.cave_noise.get([
                        x as f64 * scale,
                        y as f64 * scale * 1.5,
                        z as f64 * scale,
                    ]);

                    // Carve cave if noise is high enough
                    if cave_value > 0.6 {
                        let block = chunk.get_block(lx, y, lz);
                        if block != BlockType::Water && block != BlockType::Air && block != BlockType::Bedrock {
                            chunk.set_block(lx, y, lz, BlockType::Air);
                        }
                    }
                }
            }
        }
    }

    fn generate_ores(&self, chunk: &mut Chunk, world_x: i32, world_z: i32) {
        let ore_configs = [
            (BlockType::Coal, 0.03, 5, 60, 0.7),
            (BlockType::Iron, 0.02, 1, 45, 0.75),
            (BlockType::Gold, 0.01, 1, 30, 0.8),
            (BlockType::Diamond, 0.005, 1, 16, 0.85),
        ];

        for lx in 0..CHUNK_SIZE {
            for lz in 0..CHUNK_SIZE {
                let x = world_x + lx;
                let z = world_z + lz;

                for y in 1..CHUNK_HEIGHT - 1 {
                    if chunk.get_block(lx, y, lz) != BlockType::Stone {
                        continue;
                    }

                    for (ore_type, frequency, min_y, max_y, threshold) in &ore_configs {
                        if y < *min_y || y > *max_y {
                            continue;
                        }

                        let ore_value = self.ore_noise.get([
                            x as f64 * 0.1 + *ore_type as u8 as f64 * 100.0,
                            y as f64 * 0.1,
                            z as f64 * 0.1,
                        ]);

                        if ore_value > *threshold && rand::random::<f32>() < *frequency {
                            chunk.set_block(lx, y, lz, *ore_type);
                        }
                    }
                }
            }
        }
    }

    fn generate_structures(&self, chunk: &mut Chunk, world_x: i32, world_z: i32) {
        for lx in 2..(CHUNK_SIZE - 2) {
            for lz in 2..(CHUNK_SIZE - 2) {
                let x = world_x + lx;
                let z = world_z + lz;

                let biome = self.get_biome(x, z);
                let tree_density = biome.tree_density();

                if tree_density <= 0.0 {
                    continue;
                }

                // Use noise for tree placement
                let tree_value = self.detail_noise.get([x as f64 * 0.3, z as f64 * 0.3]);
                
                if tree_value > 1.0 - tree_density as f64 * 2.0 {
                    // Find ground level
                    for y in (1..CHUNK_HEIGHT - 10).rev() {
                        let block = chunk.get_block(lx, y, lz);
                        if block == BlockType::Grass || block == BlockType::Dirt || block == BlockType::Sand {
                            if biome == Biome::Desert {
                                self.place_cactus(chunk, lx, y + 1, lz);
                            } else {
                                self.place_tree(chunk, lx, y + 1, lz, biome);
                            }
                            break;
                        }
                        if block != BlockType::Air && block != BlockType::Water {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn place_tree(&self, chunk: &mut Chunk, x: i32, y: i32, z: i32, biome: Biome) {
        let height = match biome {
            Biome::Jungle => 7 + (rand::random::<i32>() % 4),
            Biome::Taiga => 8 + (rand::random::<i32>() % 3),
            _ => 4 + (rand::random::<i32>() % 3),
        };

        // Trunk
        for dy in 0..height {
            if y + dy < CHUNK_HEIGHT {
                chunk.set_block(x, y + dy, z, BlockType::Wood);
            }
        }

        // Leaves
        let leaf_start = height - 3;
        for dy in leaf_start..height + 2 {
            let radius = if dy == height + 1 { 1 } else if dy >= height - 1 { 2 } else { 3 };
            
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    if dx == 0 && dz == 0 && dy < height {
                        continue; // Skip trunk
                    }
                    
                    let dist = (dx * dx + dz * dz) as f32;
                    if dist <= (radius * radius) as f32 + 0.5 {
                        let lx = x + dx;
                        let ly = y + dy;
                        let lz = z + dz;
                        
                        if lx >= 0 && lx < CHUNK_SIZE && ly >= 0 && ly < CHUNK_HEIGHT && lz >= 0 && lz < CHUNK_SIZE {
                            if chunk.get_block(lx, ly, lz) == BlockType::Air {
                                chunk.set_block(lx, ly, lz, BlockType::Leaves);
                            }
                        }
                    }
                }
            }
        }
    }

    fn place_cactus(&self, chunk: &mut Chunk, x: i32, y: i32, z: i32) {
        let height = 2 + (rand::random::<i32>() % 3);
        
        for dy in 0..height {
            if y + dy < CHUNK_HEIGHT {
                chunk.set_block(x, y + dy, z, BlockType::Cactus);
            }
        }
    }

    pub fn get_biome(&self, x: i32, z: i32) -> Biome {
        let scale = 0.005;
        
        let temperature = self.biome_noise.get([x as f64 * scale, 0.0, z as f64 * scale]);
        let humidity = self.biome_noise.get([x as f64 * scale + 1000.0, 0.0, z as f64 * scale + 1000.0]);
        let altitude = self.terrain_noise.get([x as f64 * 0.01, z as f64 * 0.01]);

        // Determine biome from climate values
        if altitude > 0.5 {
            return Biome::Mountains;
        }

        if altitude < -0.3 {
            return Biome::Ocean;
        }

        if altitude < -0.2 && altitude > -0.3 {
            return Biome::Beach;
        }

        if temperature > 0.4 {
            if humidity > 0.3 {
                Biome::Jungle
            } else if humidity < -0.2 {
                Biome::Desert
            } else {
                Biome::Savanna
            }
        } else if temperature < -0.3 {
            if humidity > 0.2 {
                Biome::Taiga
            } else {
                Biome::Tundra
            }
        } else {
            if humidity > 0.4 {
                Biome::Swamp
            } else if humidity > 0.1 {
                Biome::Forest
            } else {
                Biome::Plains
            }
        }
    }

    fn get_terrain_height(&self, x: i32, z: i32, biome: Biome) -> i32 {
        let scale1 = 0.01;
        let scale2 = 0.05;
        let scale3 = 0.1;

        let base_height = biome.base_height();
        let variation = biome.height_variation();

        // Multi-octave noise
        let noise1 = self.terrain_noise.get([x as f64 * scale1, z as f64 * scale1]) * variation;
        let noise2 = self.detail_noise.get([x as f64 * scale2, z as f64 * scale2]) * variation * 0.5;
        let noise3 = self.terrain_noise.get([x as f64 * scale3, z as f64 * scale3]) * variation * 0.25;

        let height = base_height + noise1 + noise2 + noise3;

        height.max(1.0).min((CHUNK_HEIGHT - 10) as f64) as i32
    }
}
