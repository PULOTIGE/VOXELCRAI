// VoxelCraft - Biome System

use super::BlockType;
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Biome {
    Plains,
    Forest,
    Desert,
    Mountains,
    Tundra,
    Jungle,
    Swamp,
    Ocean,
    Beach,
    Savanna,
    Taiga,
}

impl Default for Biome {
    fn default() -> Self {
        Biome::Plains
    }
}

impl Biome {
    /// Get surface block for this biome
    pub fn surface_block(&self) -> BlockType {
        match self {
            Biome::Plains | Biome::Forest | Biome::Jungle | Biome::Savanna => BlockType::Grass,
            Biome::Desert | Biome::Beach => BlockType::Sand,
            Biome::Mountains => BlockType::Stone,
            Biome::Tundra | Biome::Taiga => BlockType::Snow,
            Biome::Swamp => BlockType::Dirt,
            Biome::Ocean => BlockType::Sand,
        }
    }

    /// Get subsurface block
    pub fn subsurface_block(&self) -> BlockType {
        match self {
            Biome::Desert | Biome::Beach => BlockType::Sandstone,
            Biome::Tundra => BlockType::Ice,
            _ => BlockType::Dirt,
        }
    }

    /// Get water level
    pub fn water_level(&self) -> i32 {
        match self {
            Biome::Ocean => 50,
            Biome::Swamp => 32,
            _ => 30,
        }
    }

    /// Get base terrain height
    pub fn base_height(&self) -> f64 {
        match self {
            Biome::Ocean => 25.0,
            Biome::Beach => 32.0,
            Biome::Plains | Biome::Savanna => 35.0,
            Biome::Forest | Biome::Jungle => 38.0,
            Biome::Swamp => 30.0,
            Biome::Desert => 36.0,
            Biome::Tundra | Biome::Taiga => 40.0,
            Biome::Mountains => 50.0,
        }
    }

    /// Get terrain variation
    pub fn height_variation(&self) -> f64 {
        match self {
            Biome::Ocean => 5.0,
            Biome::Beach | Biome::Swamp => 3.0,
            Biome::Plains | Biome::Desert | Biome::Savanna => 8.0,
            Biome::Forest | Biome::Jungle | Biome::Taiga => 12.0,
            Biome::Tundra => 10.0,
            Biome::Mountains => 30.0,
        }
    }

    /// Tree density (0.0 - 1.0)
    pub fn tree_density(&self) -> f32 {
        match self {
            Biome::Forest | Biome::Taiga => 0.15,
            Biome::Jungle => 0.25,
            Biome::Plains | Biome::Savanna => 0.02,
            Biome::Swamp => 0.08,
            Biome::Desert | Biome::Ocean | Biome::Beach | Biome::Tundra | Biome::Mountains => 0.0,
        }
    }

    /// Grass/plant density
    pub fn vegetation_density(&self) -> f32 {
        match self {
            Biome::Jungle => 0.4,
            Biome::Forest => 0.3,
            Biome::Swamp => 0.25,
            Biome::Plains | Biome::Savanna => 0.2,
            Biome::Taiga => 0.15,
            Biome::Tundra => 0.05,
            _ => 0.0,
        }
    }

    /// Mob spawn rates
    pub fn passive_mob_rate(&self) -> f32 {
        match self {
            Biome::Plains | Biome::Forest | Biome::Savanna => 0.3,
            Biome::Jungle | Biome::Taiga => 0.25,
            Biome::Swamp => 0.15,
            Biome::Mountains | Biome::Tundra => 0.1,
            _ => 0.05,
        }
    }

    pub fn hostile_mob_rate(&self) -> f32 {
        match self {
            Biome::Swamp => 0.4,
            Biome::Forest | Biome::Jungle | Biome::Taiga => 0.3,
            Biome::Plains | Biome::Savanna | Biome::Mountains => 0.25,
            Biome::Desert | Biome::Tundra => 0.2,
            _ => 0.1,
        }
    }

    /// Fog color for atmosphere
    pub fn fog_color(&self) -> [f32; 3] {
        match self {
            Biome::Desert => [0.9, 0.85, 0.7],
            Biome::Jungle => [0.6, 0.75, 0.6],
            Biome::Swamp => [0.5, 0.55, 0.5],
            Biome::Tundra => [0.85, 0.9, 0.95],
            Biome::Ocean => [0.5, 0.6, 0.8],
            _ => [0.7, 0.8, 0.9],
        }
    }

    /// Ambient color modifier
    pub fn ambient_modifier(&self) -> f32 {
        match self {
            Biome::Jungle | Biome::Forest => 0.85,
            Biome::Swamp => 0.7,
            Biome::Desert => 1.1,
            Biome::Tundra => 1.05,
            _ => 1.0,
        }
    }
}

/// Biome blend for smooth transitions
pub struct BiomeBlend {
    pub biomes: [(Biome, f32); 4],
}

impl BiomeBlend {
    pub fn new(biome: Biome) -> Self {
        Self {
            biomes: [(biome, 1.0), (biome, 0.0), (biome, 0.0), (biome, 0.0)],
        }
    }

    pub fn primary(&self) -> Biome {
        self.biomes[0].0
    }

    pub fn blend_surface(&self) -> BlockType {
        self.biomes[0].0.surface_block()
    }

    pub fn blend_height(&self) -> f64 {
        let mut height = 0.0;
        for (biome, weight) in &self.biomes {
            height += biome.base_height() * (*weight as f64);
        }
        height
    }

    pub fn blend_variation(&self) -> f64 {
        let mut variation = 0.0;
        for (biome, weight) in &self.biomes {
            variation += biome.height_variation() * (*weight as f64);
        }
        variation
    }
}
