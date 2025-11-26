// VoxelCraft - Crafting System

use crate::player::{Item, ItemStack, Inventory, ToolItem, MaterialItem};
use crate::world::BlockType;
use serde::{Serialize, Deserialize};

/// Recipe for crafting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recipe {
    pub pattern: Vec<Vec<Option<Item>>>,
    pub result: ItemStack,
    pub shapeless: bool,
}

impl Recipe {
    pub fn new(pattern: Vec<Vec<Option<Item>>>, result: ItemStack) -> Self {
        Self {
            pattern,
            result,
            shapeless: false,
        }
    }

    pub fn shapeless(ingredients: Vec<Item>, result: ItemStack) -> Self {
        Self {
            pattern: vec![ingredients.into_iter().map(Some).collect()],
            result,
            shapeless: true,
        }
    }

    pub fn matches(&self, grid: &CraftingGrid) -> bool {
        if self.shapeless {
            return self.matches_shapeless(grid);
        }

        // Try all positions in grid
        let pattern_h = self.pattern.len();
        let pattern_w = self.pattern.get(0).map(|r| r.len()).unwrap_or(0);

        for offset_y in 0..=(grid.size - pattern_h) {
            for offset_x in 0..=(grid.size - pattern_w) {
                if self.matches_at_position(grid, offset_x, offset_y) {
                    return true;
                }
            }
        }

        false
    }

    fn matches_at_position(&self, grid: &CraftingGrid, offset_x: usize, offset_y: usize) -> bool {
        // Check pattern matches
        for (py, row) in self.pattern.iter().enumerate() {
            for (px, required) in row.iter().enumerate() {
                let grid_item = grid.get(offset_x + px, offset_y + py);
                
                match (required, grid_item) {
                    (Some(req), Some(stack)) => {
                        if &stack.item != req {
                            return false;
                        }
                    }
                    (None, Some(_)) => return false,
                    (Some(_), None) => return false,
                    (None, None) => {}
                }
            }
        }

        // Check rest of grid is empty
        for y in 0..grid.size {
            for x in 0..grid.size {
                let in_pattern = x >= offset_x 
                    && x < offset_x + self.pattern.get(0).map(|r| r.len()).unwrap_or(0)
                    && y >= offset_y 
                    && y < offset_y + self.pattern.len();

                if !in_pattern && grid.get(x, y).is_some() {
                    return false;
                }
            }
        }

        true
    }

    fn matches_shapeless(&self, grid: &CraftingGrid) -> bool {
        let required: Vec<&Item> = self.pattern
            .iter()
            .flat_map(|row| row.iter())
            .filter_map(|i| i.as_ref())
            .collect();

        let mut grid_items: Vec<&Item> = grid.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|s| &s.item)
            .collect();

        if required.len() != grid_items.len() {
            return false;
        }

        for req in required {
            if let Some(pos) = grid_items.iter().position(|&i| i == req) {
                grid_items.remove(pos);
            } else {
                return false;
            }
        }

        grid_items.is_empty()
    }
}

/// Crafting grid (2x2 or 3x3)
#[derive(Clone, Serialize, Deserialize)]
pub struct CraftingGrid {
    pub slots: Vec<Option<ItemStack>>,
    pub size: usize,
}

impl CraftingGrid {
    pub fn new(size: usize) -> Self {
        Self {
            slots: vec![None; size * size],
            size,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&ItemStack> {
        if x < self.size && y < self.size {
            self.slots[y * self.size + x].as_ref()
        } else {
            None
        }
    }

    pub fn set(&mut self, x: usize, y: usize, stack: Option<ItemStack>) {
        if x < self.size && y < self.size {
            self.slots[y * self.size + x] = stack;
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
    }

    pub fn consume_one(&mut self) {
        for slot in &mut self.slots {
            if let Some(stack) = slot {
                stack.count -= 1;
                if stack.count == 0 {
                    *slot = None;
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|s| s.is_none())
    }
}

/// Crafting system with all recipes
pub struct CraftingSystem {
    pub recipes: Vec<Recipe>,
}

impl CraftingSystem {
    pub fn new() -> Self {
        let mut system = Self {
            recipes: Vec::new(),
        };
        system.register_recipes();
        system
    }

    fn register_recipes(&mut self) {
        // Planks from wood
        self.add(Recipe::shapeless(
            vec![Item::Block(BlockType::Wood)],
            ItemStack::new(Item::Block(BlockType::Planks), 4),
        ));

        // Sticks from planks
        self.add(Recipe::new(
            vec![
                vec![Some(Item::Block(BlockType::Planks))],
                vec![Some(Item::Block(BlockType::Planks))],
            ],
            ItemStack::new(Item::Material(MaterialItem::Stick), 4),
        ));

        // Crafting table
        self.add(Recipe::new(
            vec![
                vec![Some(Item::Block(BlockType::Planks)), Some(Item::Block(BlockType::Planks))],
                vec![Some(Item::Block(BlockType::Planks)), Some(Item::Block(BlockType::Planks))],
            ],
            ItemStack::new(Item::Block(BlockType::CraftingTable), 1),
        ));

        // Furnace
        self.add(Recipe::new(
            vec![
                vec![Some(Item::Block(BlockType::Cobblestone)), Some(Item::Block(BlockType::Cobblestone)), Some(Item::Block(BlockType::Cobblestone))],
                vec![Some(Item::Block(BlockType::Cobblestone)), None, Some(Item::Block(BlockType::Cobblestone))],
                vec![Some(Item::Block(BlockType::Cobblestone)), Some(Item::Block(BlockType::Cobblestone)), Some(Item::Block(BlockType::Cobblestone))],
            ],
            ItemStack::new(Item::Block(BlockType::Furnace), 1),
        ));

        // Chest
        self.add(Recipe::new(
            vec![
                vec![Some(Item::Block(BlockType::Planks)), Some(Item::Block(BlockType::Planks)), Some(Item::Block(BlockType::Planks))],
                vec![Some(Item::Block(BlockType::Planks)), None, Some(Item::Block(BlockType::Planks))],
                vec![Some(Item::Block(BlockType::Planks)), Some(Item::Block(BlockType::Planks)), Some(Item::Block(BlockType::Planks))],
            ],
            ItemStack::new(Item::Block(BlockType::Chest), 1),
        ));

        // Torch
        self.add(Recipe::new(
            vec![
                vec![Some(Item::Material(MaterialItem::Coal))],
                vec![Some(Item::Material(MaterialItem::Stick))],
            ],
            ItemStack::new(Item::Block(BlockType::Torch), 4),
        ));

        // Wooden pickaxe
        self.add(Recipe::new(
            vec![
                vec![Some(Item::Block(BlockType::Planks)), Some(Item::Block(BlockType::Planks)), Some(Item::Block(BlockType::Planks))],
                vec![None, Some(Item::Material(MaterialItem::Stick)), None],
                vec![None, Some(Item::Material(MaterialItem::Stick)), None],
            ],
            ItemStack::new(Item::Tool(ToolItem::WoodPickaxe), 1),
        ));

        // Stone pickaxe
        self.add(Recipe::new(
            vec![
                vec![Some(Item::Block(BlockType::Cobblestone)), Some(Item::Block(BlockType::Cobblestone)), Some(Item::Block(BlockType::Cobblestone))],
                vec![None, Some(Item::Material(MaterialItem::Stick)), None],
                vec![None, Some(Item::Material(MaterialItem::Stick)), None],
            ],
            ItemStack::new(Item::Tool(ToolItem::StonePickaxe), 1),
        ));

        // Iron pickaxe
        self.add(Recipe::new(
            vec![
                vec![Some(Item::Material(MaterialItem::IronIngot)), Some(Item::Material(MaterialItem::IronIngot)), Some(Item::Material(MaterialItem::IronIngot))],
                vec![None, Some(Item::Material(MaterialItem::Stick)), None],
                vec![None, Some(Item::Material(MaterialItem::Stick)), None],
            ],
            ItemStack::new(Item::Tool(ToolItem::IronPickaxe), 1),
        ));

        // Diamond pickaxe
        self.add(Recipe::new(
            vec![
                vec![Some(Item::Material(MaterialItem::Diamond)), Some(Item::Material(MaterialItem::Diamond)), Some(Item::Material(MaterialItem::Diamond))],
                vec![None, Some(Item::Material(MaterialItem::Stick)), None],
                vec![None, Some(Item::Material(MaterialItem::Stick)), None],
            ],
            ItemStack::new(Item::Tool(ToolItem::DiamondPickaxe), 1),
        ));

        // Wooden sword
        self.add(Recipe::new(
            vec![
                vec![Some(Item::Block(BlockType::Planks))],
                vec![Some(Item::Block(BlockType::Planks))],
                vec![Some(Item::Material(MaterialItem::Stick))],
            ],
            ItemStack::new(Item::Tool(ToolItem::WoodSword), 1),
        ));

        // Stone sword
        self.add(Recipe::new(
            vec![
                vec![Some(Item::Block(BlockType::Cobblestone))],
                vec![Some(Item::Block(BlockType::Cobblestone))],
                vec![Some(Item::Material(MaterialItem::Stick))],
            ],
            ItemStack::new(Item::Tool(ToolItem::StoneSword), 1),
        ));

        // Glass from sand (simplified - should be furnace)
        self.add(Recipe::shapeless(
            vec![Item::Block(BlockType::Sand), Item::Material(MaterialItem::Coal)],
            ItemStack::new(Item::Block(BlockType::Glass), 1),
        ));

        // Brick from clay
        self.add(Recipe::new(
            vec![
                vec![Some(Item::Block(BlockType::Brick)), Some(Item::Block(BlockType::Brick))],
                vec![Some(Item::Block(BlockType::Brick)), Some(Item::Block(BlockType::Brick))],
            ],
            ItemStack::new(Item::Block(BlockType::Brick), 1),
        ));
    }

    fn add(&mut self, recipe: Recipe) {
        self.recipes.push(recipe);
    }

    pub fn find_recipe(&self, grid: &CraftingGrid) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.matches(grid))
    }

    pub fn craft(&self, grid: &mut CraftingGrid, inventory: &mut Inventory) -> bool {
        if let Some(recipe) = self.find_recipe(grid) {
            let result = recipe.result.clone();
            grid.consume_one();
            inventory.add_item(result);
            true
        } else {
            false
        }
    }
}

impl Default for CraftingSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Smelting recipes for furnace
pub struct SmeltingRecipe {
    pub input: Item,
    pub output: ItemStack,
    pub duration: f32, // seconds
}

impl SmeltingRecipe {
    pub fn all() -> Vec<Self> {
        vec![
            Self { input: Item::Block(BlockType::Iron), output: ItemStack::new(Item::Material(MaterialItem::IronIngot), 1), duration: 10.0 },
            Self { input: Item::Block(BlockType::Gold), output: ItemStack::new(Item::Material(MaterialItem::GoldIngot), 1), duration: 10.0 },
            Self { input: Item::Block(BlockType::Sand), output: ItemStack::new(Item::Block(BlockType::Glass), 1), duration: 10.0 },
            Self { input: Item::Block(BlockType::Cobblestone), output: ItemStack::new(Item::Block(BlockType::Stone), 1), duration: 10.0 },
            Self { input: Item::Block(BlockType::Clay), output: ItemStack::new(Item::Block(BlockType::Brick), 1), duration: 10.0 },
        ]
    }

    pub fn find(input: &Item) -> Option<Self> {
        Self::all().into_iter().find(|r| &r.input == input)
    }
}
