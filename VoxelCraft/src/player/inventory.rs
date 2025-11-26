// VoxelCraft - Inventory System

use crate::world::BlockType;
use serde::{Serialize, Deserialize};

pub const HOTBAR_SIZE: usize = 9;
pub const INVENTORY_SIZE: usize = 36;
pub const MAX_STACK_SIZE: u32 = 64;

/// Item stack in inventory
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemStack {
    pub item: Item,
    pub count: u32,
}

impl ItemStack {
    pub fn new(item: Item, count: u32) -> Self {
        Self { item, count: count.min(MAX_STACK_SIZE) }
    }

    pub fn from_block(block: BlockType) -> Self {
        Self::new(Item::Block(block), 1)
    }

    pub fn can_stack(&self, other: &ItemStack) -> bool {
        self.item == other.item && self.count < MAX_STACK_SIZE
    }

    pub fn add(&mut self, count: u32) -> u32 {
        let space = MAX_STACK_SIZE - self.count;
        let added = count.min(space);
        self.count += added;
        count - added // Return overflow
    }

    pub fn remove(&mut self, count: u32) -> u32 {
        let removed = count.min(self.count);
        self.count -= removed;
        removed
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn as_block(&self) -> Option<BlockType> {
        match self.item {
            Item::Block(block) => Some(block),
            _ => None,
        }
    }
}

/// Item types
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Item {
    Block(BlockType),
    Tool(ToolItem),
    Food(FoodItem),
    Material(MaterialItem),
}

impl Item {
    pub fn name(&self) -> &'static str {
        match self {
            Item::Block(block) => block.name(),
            Item::Tool(tool) => tool.name(),
            Item::Food(food) => food.name(),
            Item::Material(mat) => mat.name(),
        }
    }

    pub fn max_stack(&self) -> u32 {
        match self {
            Item::Tool(_) => 1,
            _ => MAX_STACK_SIZE,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ToolItem {
    WoodPickaxe,
    StonePickaxe,
    IronPickaxe,
    GoldPickaxe,
    DiamondPickaxe,
    WoodAxe,
    StoneAxe,
    IronAxe,
    GoldAxe,
    DiamondAxe,
    WoodShovel,
    StoneShovel,
    IronShovel,
    GoldShovel,
    DiamondShovel,
    WoodSword,
    StoneSword,
    IronSword,
    GoldSword,
    DiamondSword,
}

impl ToolItem {
    pub fn name(&self) -> &'static str {
        match self {
            ToolItem::WoodPickaxe => "Wooden Pickaxe",
            ToolItem::StonePickaxe => "Stone Pickaxe",
            ToolItem::IronPickaxe => "Iron Pickaxe",
            ToolItem::GoldPickaxe => "Gold Pickaxe",
            ToolItem::DiamondPickaxe => "Diamond Pickaxe",
            ToolItem::WoodAxe => "Wooden Axe",
            ToolItem::StoneAxe => "Stone Axe",
            ToolItem::IronAxe => "Iron Axe",
            ToolItem::GoldAxe => "Gold Axe",
            ToolItem::DiamondAxe => "Diamond Axe",
            ToolItem::WoodShovel => "Wooden Shovel",
            ToolItem::StoneShovel => "Stone Shovel",
            ToolItem::IronShovel => "Iron Shovel",
            ToolItem::GoldShovel => "Gold Shovel",
            ToolItem::DiamondShovel => "Diamond Shovel",
            ToolItem::WoodSword => "Wooden Sword",
            ToolItem::StoneSword => "Stone Sword",
            ToolItem::IronSword => "Iron Sword",
            ToolItem::GoldSword => "Gold Sword",
            ToolItem::DiamondSword => "Diamond Sword",
        }
    }

    pub fn durability(&self) -> u32 {
        match self {
            ToolItem::WoodPickaxe | ToolItem::WoodAxe | ToolItem::WoodShovel | ToolItem::WoodSword => 59,
            ToolItem::StonePickaxe | ToolItem::StoneAxe | ToolItem::StoneShovel | ToolItem::StoneSword => 131,
            ToolItem::IronPickaxe | ToolItem::IronAxe | ToolItem::IronShovel | ToolItem::IronSword => 250,
            ToolItem::GoldPickaxe | ToolItem::GoldAxe | ToolItem::GoldShovel | ToolItem::GoldSword => 32,
            ToolItem::DiamondPickaxe | ToolItem::DiamondAxe | ToolItem::DiamondShovel | ToolItem::DiamondSword => 1561,
        }
    }

    pub fn mining_speed(&self) -> f32 {
        match self {
            ToolItem::WoodPickaxe | ToolItem::WoodAxe | ToolItem::WoodShovel => 2.0,
            ToolItem::StonePickaxe | ToolItem::StoneAxe | ToolItem::StoneShovel => 4.0,
            ToolItem::IronPickaxe | ToolItem::IronAxe | ToolItem::IronShovel => 6.0,
            ToolItem::GoldPickaxe | ToolItem::GoldAxe | ToolItem::GoldShovel => 12.0,
            ToolItem::DiamondPickaxe | ToolItem::DiamondAxe | ToolItem::DiamondShovel => 8.0,
            _ => 1.0,
        }
    }

    pub fn attack_damage(&self) -> f32 {
        match self {
            ToolItem::WoodSword => 4.0,
            ToolItem::StoneSword => 5.0,
            ToolItem::IronSword => 6.0,
            ToolItem::GoldSword => 4.0,
            ToolItem::DiamondSword => 7.0,
            ToolItem::WoodAxe => 3.0,
            ToolItem::StoneAxe => 4.0,
            ToolItem::IronAxe => 5.0,
            ToolItem::GoldAxe => 3.0,
            ToolItem::DiamondAxe => 6.0,
            _ => 1.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FoodItem {
    Apple,
    Bread,
    CookedPork,
    RawPork,
    CookedBeef,
    RawBeef,
    CookedChicken,
    RawChicken,
    GoldenApple,
}

impl FoodItem {
    pub fn name(&self) -> &'static str {
        match self {
            FoodItem::Apple => "Apple",
            FoodItem::Bread => "Bread",
            FoodItem::CookedPork => "Cooked Porkchop",
            FoodItem::RawPork => "Raw Porkchop",
            FoodItem::CookedBeef => "Steak",
            FoodItem::RawBeef => "Raw Beef",
            FoodItem::CookedChicken => "Cooked Chicken",
            FoodItem::RawChicken => "Raw Chicken",
            FoodItem::GoldenApple => "Golden Apple",
        }
    }

    pub fn hunger(&self) -> f32 {
        match self {
            FoodItem::Apple => 4.0,
            FoodItem::Bread => 5.0,
            FoodItem::CookedPork | FoodItem::CookedBeef => 8.0,
            FoodItem::RawPork | FoodItem::RawBeef => 3.0,
            FoodItem::CookedChicken => 6.0,
            FoodItem::RawChicken => 2.0,
            FoodItem::GoldenApple => 4.0,
        }
    }

    pub fn saturation(&self) -> f32 {
        match self {
            FoodItem::Apple => 2.4,
            FoodItem::Bread => 6.0,
            FoodItem::CookedPork | FoodItem::CookedBeef => 12.8,
            FoodItem::RawPork | FoodItem::RawBeef => 1.8,
            FoodItem::CookedChicken => 7.2,
            FoodItem::RawChicken => 1.2,
            FoodItem::GoldenApple => 9.6,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum MaterialItem {
    Stick,
    Coal,
    IronIngot,
    GoldIngot,
    Diamond,
    String,
    Leather,
    Paper,
    Book,
}

impl MaterialItem {
    pub fn name(&self) -> &'static str {
        match self {
            MaterialItem::Stick => "Stick",
            MaterialItem::Coal => "Coal",
            MaterialItem::IronIngot => "Iron Ingot",
            MaterialItem::GoldIngot => "Gold Ingot",
            MaterialItem::Diamond => "Diamond",
            MaterialItem::String => "String",
            MaterialItem::Leather => "Leather",
            MaterialItem::Paper => "Paper",
            MaterialItem::Book => "Book",
        }
    }
}

/// Player inventory
#[derive(Clone, Serialize, Deserialize)]
pub struct Inventory {
    slots: Vec<Option<ItemStack>>,
    pub selected: usize,
}

impl Inventory {
    pub fn new() -> Self {
        let mut inv = Self {
            slots: vec![None; INVENTORY_SIZE],
            selected: 0,
        };

        // Give starter items
        inv.add_item(ItemStack::new(Item::Block(BlockType::Dirt), 32));
        inv.add_item(ItemStack::new(Item::Block(BlockType::Cobblestone), 32));
        inv.add_item(ItemStack::new(Item::Block(BlockType::Planks), 16));
        inv.add_item(ItemStack::new(Item::Tool(ToolItem::WoodPickaxe), 1));
        inv.add_item(ItemStack::new(Item::Block(BlockType::Torch), 8));

        inv
    }

    pub fn add_item(&mut self, mut stack: ItemStack) -> bool {
        // Try to stack with existing
        for slot in self.slots.iter_mut() {
            if let Some(existing) = slot {
                if existing.can_stack(&stack) {
                    let overflow = existing.add(stack.count);
                    stack.count = overflow;
                    if stack.is_empty() {
                        return true;
                    }
                }
            }
        }

        // Find empty slot
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                *slot = Some(stack);
                return true;
            }
        }

        false
    }

    pub fn get_slot(&self, index: usize) -> Option<&ItemStack> {
        self.slots.get(index).and_then(|s| s.as_ref())
    }

    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut ItemStack> {
        self.slots.get_mut(index).and_then(|s| s.as_mut())
    }

    pub fn get_selected(&self) -> Option<&ItemStack> {
        self.get_slot(self.selected)
    }

    pub fn consume_selected(&mut self) -> bool {
        if let Some(slot) = self.slots.get_mut(self.selected) {
            if let Some(stack) = slot {
                stack.count -= 1;
                if stack.count == 0 {
                    *slot = None;
                }
                return true;
            }
        }
        false
    }

    pub fn select_slot(&mut self, index: usize) {
        if index < HOTBAR_SIZE {
            self.selected = index;
        }
    }

    pub fn hotbar(&self) -> &[Option<ItemStack>] {
        &self.slots[0..HOTBAR_SIZE]
    }

    pub fn main_inventory(&self) -> &[Option<ItemStack>] {
        &self.slots[HOTBAR_SIZE..]
    }

    pub fn swap_slots(&mut self, a: usize, b: usize) {
        if a < INVENTORY_SIZE && b < INVENTORY_SIZE {
            self.slots.swap(a, b);
        }
    }

    pub fn count_item(&self, item: &Item) -> u32 {
        self.slots.iter()
            .filter_map(|s| s.as_ref())
            .filter(|s| &s.item == item)
            .map(|s| s.count)
            .sum()
    }

    pub fn remove_item(&mut self, item: &Item, mut count: u32) -> bool {
        // Check if we have enough
        if self.count_item(item) < count {
            return false;
        }

        // Remove items
        for slot in self.slots.iter_mut() {
            if count == 0 {
                break;
            }

            if let Some(stack) = slot {
                if &stack.item == item {
                    let removed = stack.remove(count);
                    count -= removed;
                    
                    if stack.is_empty() {
                        *slot = None;
                    }
                }
            }
        }

        true
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}
