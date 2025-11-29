//! Weapon system - AK-47, M4A1, AWP, Deagle, Knife

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponType {
    Knife,
    Pistol,    // Desert Eagle style
    Rifle,     // AK-47 / M4A1 style  
    Sniper,    // AWP style
}

#[derive(Clone, Debug)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub name: String,
    pub damage: f32,
    pub fire_rate: f32,      // Shots per second
    pub reload_time: f32,    // Seconds
    pub magazine_size: u32,
    pub current_ammo: u32,
    pub reserve_ammo: u32,
    pub range: f32,
    pub accuracy: f32,       // 0.0 - 1.0
    pub recoil: f32,
    pub is_automatic: bool,
    pub last_shot_time: f32,
    pub is_reloading: bool,
    pub reload_start_time: f32,
}

impl Weapon {
    pub fn knife() -> Self {
        Self {
            weapon_type: WeaponType::Knife,
            name: "Knife".to_string(),
            damage: 50.0,
            fire_rate: 1.5,
            reload_time: 0.0,
            magazine_size: 1,
            current_ammo: 1,
            reserve_ammo: 1,
            range: 2.0,
            accuracy: 1.0,
            recoil: 0.0,
            is_automatic: false,
            last_shot_time: 0.0,
            is_reloading: false,
            reload_start_time: 0.0,
        }
    }

    pub fn deagle() -> Self {
        Self {
            weapon_type: WeaponType::Pistol,
            name: "Desert Eagle".to_string(),
            damage: 65.0,
            fire_rate: 2.5,
            reload_time: 2.2,
            magazine_size: 7,
            current_ammo: 7,
            reserve_ammo: 35,
            range: 50.0,
            accuracy: 0.85,
            recoil: 0.15,
            is_automatic: false,
            last_shot_time: 0.0,
            is_reloading: false,
            reload_start_time: 0.0,
        }
    }

    pub fn ak47() -> Self {
        Self {
            weapon_type: WeaponType::Rifle,
            name: "AK-47".to_string(),
            damage: 36.0,
            fire_rate: 10.0,
            reload_time: 2.5,
            magazine_size: 30,
            current_ammo: 30,
            reserve_ammo: 90,
            range: 80.0,
            accuracy: 0.78,
            recoil: 0.08,
            is_automatic: true,
            last_shot_time: 0.0,
            is_reloading: false,
            reload_start_time: 0.0,
        }
    }

    pub fn m4a1() -> Self {
        Self {
            weapon_type: WeaponType::Rifle,
            name: "M4A1-S".to_string(),
            damage: 33.0,
            fire_rate: 11.0,
            reload_time: 3.0,
            magazine_size: 25,
            current_ammo: 25,
            reserve_ammo: 75,
            range: 85.0,
            accuracy: 0.85,
            recoil: 0.05,
            is_automatic: true,
            last_shot_time: 0.0,
            is_reloading: false,
            reload_start_time: 0.0,
        }
    }

    pub fn awp() -> Self {
        Self {
            weapon_type: WeaponType::Sniper,
            name: "AWP".to_string(),
            damage: 115.0,
            fire_rate: 0.75,
            reload_time: 3.7,
            magazine_size: 5,
            current_ammo: 5,
            reserve_ammo: 30,
            range: 200.0,
            accuracy: 0.98,
            recoil: 0.25,
            is_automatic: false,
            last_shot_time: 0.0,
            is_reloading: false,
            reload_start_time: 0.0,
        }
    }

    pub fn can_fire(&self, current_time: f32) -> bool {
        if self.is_reloading {
            return false;
        }
        if self.current_ammo == 0 {
            return false;
        }
        let time_between_shots = 1.0 / self.fire_rate;
        current_time - self.last_shot_time >= time_between_shots
    }

    pub fn fire(&mut self, current_time: f32) -> Option<Shot> {
        if !self.can_fire(current_time) {
            return None;
        }

        self.current_ammo -= 1;
        self.last_shot_time = current_time;

        // Calculate accuracy spread
        let spread = (1.0 - self.accuracy) * 0.1;
        
        Some(Shot {
            damage: self.damage,
            spread,
            range: self.range,
        })
    }

    pub fn start_reload(&mut self, current_time: f32) {
        if self.is_reloading || self.current_ammo == self.magazine_size || self.reserve_ammo == 0 {
            return;
        }
        self.is_reloading = true;
        self.reload_start_time = current_time;
    }

    pub fn update(&mut self, current_time: f32) {
        if self.is_reloading {
            if current_time - self.reload_start_time >= self.reload_time {
                let needed = self.magazine_size - self.current_ammo;
                let available = needed.min(self.reserve_ammo);
                self.current_ammo += available;
                self.reserve_ammo -= available;
                self.is_reloading = false;
            }
        }
    }
}

pub struct Shot {
    pub damage: f32,
    pub spread: f32,
    pub range: f32,
}

pub struct WeaponInventory {
    pub weapons: Vec<Weapon>,
    pub current_index: usize,
}

impl WeaponInventory {
    pub fn new() -> Self {
        Self {
            weapons: vec![
                Weapon::knife(),
                Weapon::deagle(),
                Weapon::ak47(),
            ],
            current_index: 2, // Start with AK-47
        }
    }

    pub fn current(&self) -> &Weapon {
        &self.weapons[self.current_index]
    }

    pub fn current_mut(&mut self) -> &mut Weapon {
        &mut self.weapons[self.current_index]
    }

    pub fn switch(&mut self, index: usize) {
        if index < self.weapons.len() {
            self.current_index = index;
        }
    }

    pub fn next(&mut self) {
        self.current_index = (self.current_index + 1) % self.weapons.len();
    }

    pub fn prev(&mut self) {
        if self.current_index == 0 {
            self.current_index = self.weapons.len() - 1;
        } else {
            self.current_index -= 1;
        }
    }
}

impl Default for WeaponInventory {
    fn default() -> Self {
        Self::new()
    }
}
