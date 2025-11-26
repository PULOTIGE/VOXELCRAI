// VoxelCraft - Block Types

use super::chunk::Face;
use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum BlockType {
    Air = 0,
    Stone = 1,
    Dirt = 2,
    Grass = 3,
    Sand = 4,
    Water = 5,
    Wood = 6,
    Leaves = 7,
    Coal = 8,
    Iron = 9,
    Gold = 10,
    Diamond = 11,
    Cobblestone = 12,
    Planks = 13,
    Glass = 14,
    Brick = 15,
    Gravel = 16,
    Snow = 17,
    Ice = 18,
    Cactus = 19,
    Clay = 20,
    Sandstone = 21,
    Bedrock = 22,
    Obsidian = 23,
    Torch = 24,
    CraftingTable = 25,
    Furnace = 26,
    Chest = 27,
}

impl Default for BlockType {
    fn default() -> Self {
        BlockType::Air
    }
}

impl BlockType {
    pub fn is_solid(&self) -> bool {
        match self {
            BlockType::Air | BlockType::Water | BlockType::Torch => false,
            BlockType::Leaves | BlockType::Glass => true, // Solid but transparent
            _ => true,
        }
    }

    pub fn is_transparent(&self) -> bool {
        match self {
            BlockType::Air | BlockType::Water | BlockType::Glass | BlockType::Leaves | BlockType::Ice => true,
            _ => false,
        }
    }

    pub fn is_liquid(&self) -> bool {
        matches!(self, BlockType::Water)
    }

    pub fn is_light_source(&self) -> bool {
        matches!(self, BlockType::Torch)
    }

    pub fn light_level(&self) -> u8 {
        match self {
            BlockType::Torch => 14,
            _ => 0,
        }
    }

    pub fn hardness(&self) -> f32 {
        match self {
            BlockType::Air | BlockType::Water => 0.0,
            BlockType::Leaves => 0.2,
            BlockType::Dirt | BlockType::Sand | BlockType::Gravel | BlockType::Clay => 0.5,
            BlockType::Grass => 0.6,
            BlockType::Wood | BlockType::Planks => 2.0,
            BlockType::Stone | BlockType::Cobblestone | BlockType::Sandstone => 3.0,
            BlockType::Coal | BlockType::Iron => 4.0,
            BlockType::Gold | BlockType::Diamond => 5.0,
            BlockType::Obsidian => 50.0,
            BlockType::Bedrock => f32::INFINITY,
            _ => 1.0,
        }
    }

    pub fn get_tool_type(&self) -> ToolType {
        match self {
            BlockType::Stone | BlockType::Cobblestone | BlockType::Coal | 
            BlockType::Iron | BlockType::Gold | BlockType::Diamond |
            BlockType::Sandstone | BlockType::Obsidian | BlockType::Brick |
            BlockType::Furnace => ToolType::Pickaxe,
            
            BlockType::Dirt | BlockType::Grass | BlockType::Sand |
            BlockType::Gravel | BlockType::Clay | BlockType::Snow => ToolType::Shovel,
            
            BlockType::Wood | BlockType::Planks | BlockType::Leaves |
            BlockType::CraftingTable | BlockType::Chest => ToolType::Axe,
            
            _ => ToolType::None,
        }
    }

    /// Get UV coordinates in texture atlas (16x16 grid)
    pub fn get_uv(&self, face: Face) -> (f32, f32) {
        let (tx, ty) = match self {
            BlockType::Stone => (1, 0),
            BlockType::Dirt => (2, 0),
            BlockType::Grass => match face {
                Face::Top => (0, 0),
                Face::Bottom => (2, 0),
                _ => (3, 0),
            },
            BlockType::Sand => (2, 1),
            BlockType::Water => (13, 12),
            BlockType::Wood => match face {
                Face::Top | Face::Bottom => (5, 1),
                _ => (4, 1),
            },
            BlockType::Leaves => (4, 3),
            BlockType::Coal => (2, 2),
            BlockType::Iron => (1, 2),
            BlockType::Gold => (0, 2),
            BlockType::Diamond => (2, 3),
            BlockType::Cobblestone => (0, 1),
            BlockType::Planks => (4, 0),
            BlockType::Glass => (1, 3),
            BlockType::Brick => (7, 0),
            BlockType::Gravel => (3, 1),
            BlockType::Snow => (2, 4),
            BlockType::Ice => (3, 4),
            BlockType::Cactus => match face {
                Face::Top | Face::Bottom => (5, 4),
                _ => (6, 4),
            },
            BlockType::Clay => (8, 4),
            BlockType::Sandstone => match face {
                Face::Top => (0, 11),
                Face::Bottom => (0, 13),
                _ => (0, 12),
            },
            BlockType::Bedrock => (1, 1),
            BlockType::Obsidian => (5, 2),
            BlockType::Torch => (0, 5),
            BlockType::CraftingTable => match face {
                Face::Top => (11, 2),
                Face::Front | Face::Right => (11, 3),
                _ => (12, 3),
            },
            BlockType::Furnace => match face {
                Face::Front => (12, 2),
                Face::Top | Face::Bottom => (14, 3),
                _ => (13, 2),
            },
            BlockType::Chest => (9, 1),
            BlockType::Air => (0, 0),
        };

        (tx as f32 * 0.0625, ty as f32 * 0.0625)
    }

    pub fn name(&self) -> &'static str {
        match self {
            BlockType::Air => "Air",
            BlockType::Stone => "Stone",
            BlockType::Dirt => "Dirt",
            BlockType::Grass => "Grass",
            BlockType::Sand => "Sand",
            BlockType::Water => "Water",
            BlockType::Wood => "Wood",
            BlockType::Leaves => "Leaves",
            BlockType::Coal => "Coal Ore",
            BlockType::Iron => "Iron Ore",
            BlockType::Gold => "Gold Ore",
            BlockType::Diamond => "Diamond Ore",
            BlockType::Cobblestone => "Cobblestone",
            BlockType::Planks => "Planks",
            BlockType::Glass => "Glass",
            BlockType::Brick => "Brick",
            BlockType::Gravel => "Gravel",
            BlockType::Snow => "Snow",
            BlockType::Ice => "Ice",
            BlockType::Cactus => "Cactus",
            BlockType::Clay => "Clay",
            BlockType::Sandstone => "Sandstone",
            BlockType::Bedrock => "Bedrock",
            BlockType::Obsidian => "Obsidian",
            BlockType::Torch => "Torch",
            BlockType::CraftingTable => "Crafting Table",
            BlockType::Furnace => "Furnace",
            BlockType::Chest => "Chest",
        }
    }

    pub fn drops(&self) -> Option<BlockType> {
        match self {
            BlockType::Stone => Some(BlockType::Cobblestone),
            BlockType::Grass => Some(BlockType::Dirt),
            BlockType::Coal => Some(BlockType::Coal),
            BlockType::Iron => Some(BlockType::Iron),
            BlockType::Gold => Some(BlockType::Gold),
            BlockType::Diamond => Some(BlockType::Diamond),
            BlockType::Leaves => None, // Random drops handled separately
            BlockType::Glass => None,
            BlockType::Bedrock => None,
            BlockType::Air | BlockType::Water => None,
            _ => Some(*self),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ToolType {
    None,
    Pickaxe,
    Axe,
    Shovel,
    Sword,
    Hoe,
}

/// Block data for special blocks
#[derive(Clone, Serialize, Deserialize)]
pub enum BlockData {
    None,
    Chest { items: Vec<(BlockType, u8)> },
    Furnace { fuel: u8, progress: u8, input: Option<BlockType>, output: Option<BlockType> },
}

impl Default for BlockData {
    fn default() -> Self {
        BlockData::None
    }
}
