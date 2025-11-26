// VoxelCraft - Player Stats

use serde::{Serialize, Deserialize};

/// Player statistics and survival mechanics
#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub health: f32,
    pub max_health: f32,
    pub hunger: f32,
    pub saturation: f32,
    pub experience: u32,
    pub level: u32,
    hunger_timer: f32,
    regen_timer: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerStats {
    pub fn new() -> Self {
        Self {
            health: 20.0,
            max_health: 20.0,
            hunger: 20.0,
            saturation: 5.0,
            experience: 0,
            level: 0,
            hunger_timer: 0.0,
            regen_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, is_sprinting: bool) {
        // Hunger decreases over time
        self.hunger_timer += dt;
        
        let hunger_rate = if is_sprinting { 3.0 } else { 80.0 }; // Seconds per hunger point
        
        if self.hunger_timer >= hunger_rate {
            self.hunger_timer = 0.0;
            
            if self.saturation > 0.0 {
                self.saturation = (self.saturation - 1.0).max(0.0);
            } else {
                self.hunger = (self.hunger - 1.0).max(0.0);
            }
        }

        // Health regeneration when hunger is full
        if self.hunger >= 18.0 && self.health < self.max_health {
            self.regen_timer += dt;
            
            if self.regen_timer >= 4.0 {
                self.regen_timer = 0.0;
                self.health = (self.health + 1.0).min(self.max_health);
                self.hunger = (self.hunger - 0.5).max(0.0);
            }
        } else {
            self.regen_timer = 0.0;
        }

        // Starvation damage
        if self.hunger <= 0.0 {
            self.hunger_timer += dt;
            
            if self.hunger_timer >= 4.0 {
                self.hunger_timer = 0.0;
                self.health = (self.health - 1.0).max(0.0);
            }
        }
    }

    pub fn add_experience(&mut self, amount: u32) {
        self.experience += amount;
        
        // Level up
        let xp_for_next = self.xp_for_level(self.level + 1);
        while self.experience >= xp_for_next {
            self.experience -= xp_for_next;
            self.level += 1;
        }
    }

    fn xp_for_level(&self, level: u32) -> u32 {
        if level <= 16 {
            2 * level + 7
        } else if level <= 31 {
            5 * level - 38
        } else {
            9 * level - 158
        }
    }

    pub fn health_percentage(&self) -> f32 {
        self.health / self.max_health
    }

    pub fn hunger_percentage(&self) -> f32 {
        self.hunger / 20.0
    }

    pub fn is_hungry(&self) -> bool {
        self.hunger < 6.0
    }

    pub fn is_starving(&self) -> bool {
        self.hunger <= 0.0
    }

    pub fn can_sprint(&self) -> bool {
        self.hunger > 6.0
    }
}

/// Achievement system
#[derive(Clone, Serialize, Deserialize)]
pub struct Achievements {
    pub unlocked: Vec<Achievement>,
}

impl Achievements {
    pub fn new() -> Self {
        Self {
            unlocked: Vec::new(),
        }
    }

    pub fn unlock(&mut self, achievement: Achievement) {
        if !self.unlocked.contains(&achievement) {
            self.unlocked.push(achievement);
        }
    }

    pub fn has(&self, achievement: &Achievement) -> bool {
        self.unlocked.contains(achievement)
    }
}

impl Default for Achievements {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Achievement {
    OpenInventory,
    CraftWorkbench,
    MineWood,
    MineStone,
    CraftPickaxe,
    MineIron,
    SmeltIron,
    MineDiamond,
    CraftDiamondPickaxe,
    BuildHouse,
    KillZombie,
    SurviveNight,
    ReachBedrock,
    MakePortal,
}

impl Achievement {
    pub fn name(&self) -> &'static str {
        match self {
            Achievement::OpenInventory => "Taking Inventory",
            Achievement::CraftWorkbench => "Benchmarking",
            Achievement::MineWood => "Getting Wood",
            Achievement::MineStone => "Stone Age",
            Achievement::CraftPickaxe => "Time to Mine!",
            Achievement::MineIron => "Acquire Hardware",
            Achievement::SmeltIron => "Hot Topic",
            Achievement::MineDiamond => "Diamonds!",
            Achievement::CraftDiamondPickaxe => "Best Pick",
            Achievement::BuildHouse => "Home Sweet Home",
            Achievement::KillZombie => "Monster Hunter",
            Achievement::SurviveNight => "Surviving the Night",
            Achievement::ReachBedrock => "Depth Plumbing",
            Achievement::MakePortal => "Portal Pioneer",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Achievement::OpenInventory => "Open your inventory",
            Achievement::CraftWorkbench => "Craft a crafting table",
            Achievement::MineWood => "Punch a tree!",
            Achievement::MineStone => "Mine stone with a pickaxe",
            Achievement::CraftPickaxe => "Craft a wooden pickaxe",
            Achievement::MineIron => "Mine iron ore",
            Achievement::SmeltIron => "Smelt iron ore in a furnace",
            Achievement::MineDiamond => "Find and mine diamonds",
            Achievement::CraftDiamondPickaxe => "Craft a diamond pickaxe",
            Achievement::BuildHouse => "Build a house with roof",
            Achievement::KillZombie => "Kill a zombie",
            Achievement::SurviveNight => "Survive your first night",
            Achievement::ReachBedrock => "Dig to bedrock layer",
            Achievement::MakePortal => "Create a nether portal",
        }
    }
}
