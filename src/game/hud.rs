//! Game HUD - Health, Armor, Ammo, Crosshair, Kill feed

use super::player::Player;
use super::weapons::WeaponType;

/// HUD element position/size data for rendering
#[derive(Clone, Debug)]
pub struct HudElement {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],
}

/// Kill feed entry
#[derive(Clone, Debug)]
pub struct KillFeedEntry {
    pub killer_name: String,
    pub victim_name: String,
    pub weapon_name: String,
    pub headshot: bool,
    pub time: f32,
}

pub struct GameHUD {
    pub screen_width: f32,
    pub screen_height: f32,
    pub kill_feed: Vec<KillFeedEntry>,
    pub show_scoreboard: bool,
}

impl GameHUD {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            screen_width: width,
            screen_height: height,
            kill_feed: Vec::new(),
            show_scoreboard: false,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    pub fn add_kill(&mut self, killer: &str, victim: &str, weapon: &str, headshot: bool, time: f32) {
        self.kill_feed.push(KillFeedEntry {
            killer_name: killer.to_string(),
            victim_name: victim.to_string(),
            weapon_name: weapon.to_string(),
            headshot,
            time,
        });

        // Keep only last 5 kills
        if self.kill_feed.len() > 5 {
            self.kill_feed.remove(0);
        }
    }

    pub fn update(&mut self, current_time: f32) {
        // Remove old kill feed entries (older than 5 seconds)
        self.kill_feed.retain(|entry| current_time - entry.time < 5.0);
    }

    /// Generate HUD render data
    pub fn generate_hud_data(&self, player: &Player) -> HudRenderData {
        let mut data = HudRenderData::default();

        // Crosshair (center of screen)
        let cx = self.screen_width / 2.0;
        let cy = self.screen_height / 2.0;
        let crosshair_size = 20.0;
        let crosshair_thickness = 2.0;
        let crosshair_gap = 4.0;
        let crosshair_color = [0.0, 1.0, 0.0, 0.9]; // Green

        // Horizontal lines
        data.crosshair_elements.push(HudElement {
            x: cx - crosshair_size - crosshair_gap,
            y: cy - crosshair_thickness / 2.0,
            width: crosshair_size,
            height: crosshair_thickness,
            color: crosshair_color,
        });
        data.crosshair_elements.push(HudElement {
            x: cx + crosshair_gap,
            y: cy - crosshair_thickness / 2.0,
            width: crosshair_size,
            height: crosshair_thickness,
            color: crosshair_color,
        });
        // Vertical lines
        data.crosshair_elements.push(HudElement {
            x: cx - crosshair_thickness / 2.0,
            y: cy - crosshair_size - crosshair_gap,
            width: crosshair_thickness,
            height: crosshair_size,
            color: crosshair_color,
        });
        data.crosshair_elements.push(HudElement {
            x: cx - crosshair_thickness / 2.0,
            y: cy + crosshair_gap,
            width: crosshair_thickness,
            height: crosshair_size,
            color: crosshair_color,
        });

        // Health bar (bottom left)
        let health_pct = player.health / player.max_health;
        let health_color = if health_pct > 0.5 {
            [0.2, 0.8, 0.2, 0.9] // Green
        } else if health_pct > 0.25 {
            [0.9, 0.7, 0.1, 0.9] // Yellow
        } else {
            [0.9, 0.2, 0.1, 0.9] // Red
        };

        // Health background
        data.health_elements.push(HudElement {
            x: 20.0,
            y: self.screen_height - 60.0,
            width: 200.0,
            height: 24.0,
            color: [0.1, 0.1, 0.1, 0.7],
        });
        // Health bar
        data.health_elements.push(HudElement {
            x: 22.0,
            y: self.screen_height - 58.0,
            width: (196.0 * health_pct).max(0.0),
            height: 20.0,
            color: health_color,
        });

        // Armor bar (if player has armor)
        if player.armor > 0.0 {
            let armor_pct = player.armor / player.max_armor;
            // Armor background
            data.armor_elements.push(HudElement {
                x: 20.0,
                y: self.screen_height - 90.0,
                width: 200.0,
                height: 20.0,
                color: [0.1, 0.1, 0.1, 0.7],
            });
            // Armor bar
            data.armor_elements.push(HudElement {
                x: 22.0,
                y: self.screen_height - 88.0,
                width: 196.0 * armor_pct,
                height: 16.0,
                color: [0.2, 0.5, 0.9, 0.9], // Blue
            });
        }

        // Ammo display (bottom right)
        let weapon = player.weapons.current();
        data.weapon_name = weapon.name.clone();
        data.current_ammo = weapon.current_ammo;
        data.reserve_ammo = weapon.reserve_ammo;
        data.is_reloading = weapon.is_reloading;

        // Ammo background
        data.ammo_elements.push(HudElement {
            x: self.screen_width - 220.0,
            y: self.screen_height - 60.0,
            width: 200.0,
            height: 40.0,
            color: [0.1, 0.1, 0.1, 0.7],
        });

        // Weapon icon indicator
        let weapon_color = match weapon.weapon_type {
            WeaponType::Knife => [0.7, 0.7, 0.7, 0.9],
            WeaponType::Pistol => [0.9, 0.7, 0.3, 0.9],
            WeaponType::Rifle => [0.9, 0.4, 0.1, 0.9],
            WeaponType::Sniper => [0.5, 0.9, 0.3, 0.9],
        };
        data.ammo_elements.push(HudElement {
            x: self.screen_width - 215.0,
            y: self.screen_height - 55.0,
            width: 30.0,
            height: 30.0,
            color: weapon_color,
        });

        // Player stats
        data.health = player.health as u32;
        data.armor = player.armor as u32;
        data.money = player.money;
        data.kills = player.kills;
        data.deaths = player.deaths;

        // Kill feed
        data.kill_feed = self.kill_feed.clone();

        data
    }
}

#[derive(Clone, Debug, Default)]
pub struct HudRenderData {
    pub crosshair_elements: Vec<HudElement>,
    pub health_elements: Vec<HudElement>,
    pub armor_elements: Vec<HudElement>,
    pub ammo_elements: Vec<HudElement>,
    
    pub health: u32,
    pub armor: u32,
    pub money: u32,
    pub kills: u32,
    pub deaths: u32,
    
    pub weapon_name: String,
    pub current_ammo: u32,
    pub reserve_ammo: u32,
    pub is_reloading: bool,
    
    pub kill_feed: Vec<KillFeedEntry>,
}

impl Default for GameHUD {
    fn default() -> Self {
        Self::new(1920.0, 1080.0)
    }
}
