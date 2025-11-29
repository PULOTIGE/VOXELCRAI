//! Gameplay settings - weapons, enemies, game rules

use serde::{Deserialize, Serialize};
use glam::Vec3;

/// Main gameplay configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameplaySettings {
    // Game mode
    pub game_mode: GameMode,
    
    // Team settings
    pub teams_enabled: bool,
    pub max_players_per_team: u32,
    pub friendly_fire: bool,
    
    // Respawn
    pub respawn_enabled: bool,
    pub respawn_time: f32,
    pub respawn_protection_time: f32,
    
    // Round settings
    pub round_time: f32,
    pub warmup_time: f32,
    pub freeze_time: f32,
    pub rounds_to_win: u32,
    
    // Economy
    pub buy_system_enabled: bool,
    pub start_money: u32,
    pub max_money: u32,
    pub kill_reward: u32,
    pub round_win_reward: u32,
    pub round_loss_reward: u32,
    
    // Bomb mode
    pub bomb_enabled: bool,
    pub bomb_plant_time: f32,
    pub bomb_defuse_time: f32,
    pub bomb_timer: f32,
    
    // Player settings
    pub player_config: PlayerConfig,
    
    // Weapons
    pub weapons: Vec<WeaponConfig>,
    
    // Enemies/Bots
    pub bot_config: BotConfig,
    
    // Environment
    pub gravity: f32,
    pub fall_damage: bool,
    pub fall_damage_threshold: f32,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self {
            game_mode: GameMode::Deathmatch,
            teams_enabled: false,
            max_players_per_team: 5,
            friendly_fire: false,
            respawn_enabled: true,
            respawn_time: 3.0,
            respawn_protection_time: 2.0,
            round_time: 180.0,
            warmup_time: 30.0,
            freeze_time: 10.0,
            rounds_to_win: 16,
            buy_system_enabled: false,
            start_money: 800,
            max_money: 16000,
            kill_reward: 300,
            round_win_reward: 3250,
            round_loss_reward: 1400,
            bomb_enabled: false,
            bomb_plant_time: 3.0,
            bomb_defuse_time: 10.0,
            bomb_timer: 40.0,
            player_config: PlayerConfig::default(),
            weapons: WeaponConfig::default_loadout(),
            bot_config: BotConfig::default(),
            gravity: 20.0,
            fall_damage: true,
            fall_damage_threshold: 4.0,
        }
    }
}

/// Game modes
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    Deathmatch,
    TeamDeathmatch,
    BombDefusal,
    Hostage,
    GunGame,
    Custom,
}

impl Default for GameMode {
    fn default() -> Self {
        Self::Deathmatch
    }
}

impl GameMode {
    pub fn name(&self) -> &str {
        match self {
            GameMode::Deathmatch => "Deathmatch",
            GameMode::TeamDeathmatch => "Team Deathmatch",
            GameMode::BombDefusal => "Bomb Defusal",
            GameMode::Hostage => "Hostage Rescue",
            GameMode::GunGame => "Gun Game",
            GameMode::Custom => "Custom",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            GameMode::Deathmatch => "Free-for-all combat, most kills wins",
            GameMode::TeamDeathmatch => "Team-based combat, highest team score wins",
            GameMode::BombDefusal => "Terrorists plant bomb, CTs defuse it",
            GameMode::Hostage => "CTs rescue hostages from Terrorists",
            GameMode::GunGame => "Get kills to progress through weapons",
            GameMode::Custom => "Custom game rules",
        }
    }
}

/// Player configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub max_health: f32,
    pub max_armor: f32,
    pub move_speed: f32,
    pub sprint_speed: f32,
    pub crouch_speed: f32,
    pub jump_force: f32,
    pub mouse_sensitivity: f32,
    pub fov: f32,
    pub head_hitbox_multiplier: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            max_health: 100.0,
            max_armor: 100.0,
            move_speed: 5.0,
            sprint_speed: 7.5,
            crouch_speed: 2.5,
            jump_force: 8.0,
            mouse_sensitivity: 0.3,
            fov: 90.0,
            head_hitbox_multiplier: 4.0,
        }
    }
}

/// Weapon configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeaponConfig {
    pub name: String,
    pub weapon_type: WeaponType,
    pub damage: f32,
    pub fire_rate: f32,          // Rounds per second
    pub magazine_size: u32,
    pub reserve_ammo: u32,
    pub reload_time: f32,
    pub accuracy: f32,           // 0-1, 1 = perfect
    pub recoil: f32,             // Vertical kick
    pub range: f32,              // Effective range
    pub penetration: f32,        // Wall penetration power
    pub price: u32,
    pub kill_reward: u32,
    pub model: String,
    pub fire_sound: String,
    pub reload_sound: String,
}

impl WeaponConfig {
    pub fn default_loadout() -> Vec<Self> {
        vec![
            // Knives
            Self {
                name: "Knife".to_string(),
                weapon_type: WeaponType::Melee,
                damage: 40.0,
                fire_rate: 1.5,
                magazine_size: 0,
                reserve_ammo: 0,
                reload_time: 0.0,
                accuracy: 1.0,
                recoil: 0.0,
                range: 2.0,
                penetration: 0.0,
                price: 0,
                kill_reward: 1500,
                model: "knife".to_string(),
                fire_sound: "knife_slash".to_string(),
                reload_sound: "".to_string(),
            },
            // Pistols
            Self {
                name: "Glock-18".to_string(),
                weapon_type: WeaponType::Pistol,
                damage: 28.0,
                fire_rate: 6.0,
                magazine_size: 20,
                reserve_ammo: 120,
                reload_time: 2.2,
                accuracy: 0.85,
                recoil: 0.15,
                range: 30.0,
                penetration: 0.4,
                price: 200,
                kill_reward: 300,
                model: "glock".to_string(),
                fire_sound: "glock_fire".to_string(),
                reload_sound: "pistol_reload".to_string(),
            },
            Self {
                name: "Desert Eagle".to_string(),
                weapon_type: WeaponType::Pistol,
                damage: 63.0,
                fire_rate: 2.5,
                magazine_size: 7,
                reserve_ammo: 35,
                reload_time: 2.2,
                accuracy: 0.8,
                recoil: 0.35,
                range: 40.0,
                penetration: 0.8,
                price: 700,
                kill_reward: 300,
                model: "deagle".to_string(),
                fire_sound: "deagle_fire".to_string(),
                reload_sound: "pistol_reload".to_string(),
            },
            // Rifles
            Self {
                name: "AK-47".to_string(),
                weapon_type: WeaponType::Rifle,
                damage: 36.0,
                fire_rate: 10.0,
                magazine_size: 30,
                reserve_ammo: 90,
                reload_time: 2.5,
                accuracy: 0.7,
                recoil: 0.25,
                range: 50.0,
                penetration: 0.75,
                price: 2700,
                kill_reward: 300,
                model: "ak47".to_string(),
                fire_sound: "ak47_fire".to_string(),
                reload_sound: "rifle_reload".to_string(),
            },
            Self {
                name: "M4A1".to_string(),
                weapon_type: WeaponType::Rifle,
                damage: 33.0,
                fire_rate: 11.0,
                magazine_size: 30,
                reserve_ammo: 90,
                reload_time: 3.1,
                accuracy: 0.75,
                recoil: 0.2,
                range: 55.0,
                penetration: 0.7,
                price: 3100,
                kill_reward: 300,
                model: "m4a1".to_string(),
                fire_sound: "m4a1_fire".to_string(),
                reload_sound: "rifle_reload".to_string(),
            },
            // Sniper
            Self {
                name: "AWP".to_string(),
                weapon_type: WeaponType::Sniper,
                damage: 115.0,
                fire_rate: 0.8,
                magazine_size: 10,
                reserve_ammo: 30,
                reload_time: 3.7,
                accuracy: 0.98,
                recoil: 0.5,
                range: 100.0,
                penetration: 0.95,
                price: 4750,
                kill_reward: 100,
                model: "awp".to_string(),
                fire_sound: "awp_fire".to_string(),
                reload_sound: "sniper_reload".to_string(),
            },
            // SMGs
            Self {
                name: "MP5".to_string(),
                weapon_type: WeaponType::SMG,
                damage: 27.0,
                fire_rate: 13.0,
                magazine_size: 30,
                reserve_ammo: 120,
                reload_time: 2.3,
                accuracy: 0.65,
                recoil: 0.12,
                range: 25.0,
                penetration: 0.3,
                price: 1500,
                kill_reward: 600,
                model: "mp5".to_string(),
                fire_sound: "mp5_fire".to_string(),
                reload_sound: "smg_reload".to_string(),
            },
            // Shotguns
            Self {
                name: "Pump Shotgun".to_string(),
                weapon_type: WeaponType::Shotgun,
                damage: 20.0, // Per pellet, 8 pellets
                fire_rate: 0.9,
                magazine_size: 8,
                reserve_ammo: 32,
                reload_time: 0.5, // Per shell
                accuracy: 0.4,
                recoil: 0.3,
                range: 15.0,
                penetration: 0.1,
                price: 1200,
                kill_reward: 900,
                model: "shotgun".to_string(),
                fire_sound: "shotgun_fire".to_string(),
                reload_sound: "shotgun_reload".to_string(),
            },
        ]
    }
}

/// Weapon types
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum WeaponType {
    Melee,
    Pistol,
    Rifle,
    Sniper,
    SMG,
    Shotgun,
    MachineGun,
    Grenade,
}

impl WeaponType {
    pub fn name(&self) -> &str {
        match self {
            WeaponType::Melee => "Melee",
            WeaponType::Pistol => "Pistol",
            WeaponType::Rifle => "Rifle",
            WeaponType::Sniper => "Sniper",
            WeaponType::SMG => "SMG",
            WeaponType::Shotgun => "Shotgun",
            WeaponType::MachineGun => "Machine Gun",
            WeaponType::Grenade => "Grenade",
        }
    }
}

/// Bot/AI configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotConfig {
    pub enabled: bool,
    pub count: u32,
    pub difficulty: BotDifficulty,
    pub skill_levels: BotSkillLevels,
    pub behavior: BotBehavior,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            count: 4,
            difficulty: BotDifficulty::Normal,
            skill_levels: BotSkillLevels::default(),
            behavior: BotBehavior::default(),
        }
    }
}

/// Bot difficulty presets
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum BotDifficulty {
    Easy,
    Normal,
    Hard,
    Expert,
    Custom,
}

impl BotDifficulty {
    pub fn apply_to_skills(&self, skills: &mut BotSkillLevels) {
        match self {
            BotDifficulty::Easy => {
                skills.accuracy = 0.3;
                skills.reaction_time = 0.8;
                skills.awareness = 0.4;
                skills.aggression = 0.3;
            }
            BotDifficulty::Normal => {
                skills.accuracy = 0.5;
                skills.reaction_time = 0.5;
                skills.awareness = 0.6;
                skills.aggression = 0.5;
            }
            BotDifficulty::Hard => {
                skills.accuracy = 0.7;
                skills.reaction_time = 0.3;
                skills.awareness = 0.8;
                skills.aggression = 0.7;
            }
            BotDifficulty::Expert => {
                skills.accuracy = 0.9;
                skills.reaction_time = 0.15;
                skills.awareness = 0.95;
                skills.aggression = 0.8;
            }
            BotDifficulty::Custom => {
                // Keep current values
            }
        }
    }
}

/// Bot skill configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotSkillLevels {
    pub accuracy: f32,        // 0-1, chance to hit
    pub reaction_time: f32,   // Seconds to react
    pub awareness: f32,       // 0-1, detection range multiplier
    pub aggression: f32,      // 0-1, attack vs defense tendency
    pub teamwork: f32,        // 0-1, cooperation level
}

impl Default for BotSkillLevels {
    fn default() -> Self {
        Self {
            accuracy: 0.5,
            reaction_time: 0.5,
            awareness: 0.6,
            aggression: 0.5,
            teamwork: 0.5,
        }
    }
}

/// Bot behavior settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotBehavior {
    pub patrol: bool,
    pub use_cover: bool,
    pub retreat_when_hurt: bool,
    pub call_for_backup: bool,
    pub use_grenades: bool,
    pub camp: bool,
    pub rush: bool,
}

impl Default for BotBehavior {
    fn default() -> Self {
        Self {
            patrol: true,
            use_cover: true,
            retreat_when_hurt: true,
            call_for_backup: true,
            use_grenades: true,
            camp: false,
            rush: false,
        }
    }
}
