// VoxelCraft - Save System

use crate::{GameState, World, Player};
use crate::world::Chunk;
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

/// Save system for game data
pub struct SaveSystem {
    save_dir: PathBuf,
}

impl SaveSystem {
    pub fn new() -> Self {
        #[cfg(target_os = "android")]
        let save_dir = PathBuf::from("/data/data/com.voxelcraft/files/saves");
        
        #[cfg(not(target_os = "android"))]
        let save_dir = PathBuf::from("saves");

        // Create save directory if needed
        let _ = fs::create_dir_all(&save_dir);

        Self { save_dir }
    }

    pub fn save(&self, state: &GameState) -> Result<(), SaveError> {
        let world_save = WorldSave::from_state(state);
        
        // Save to file
        let save_path = self.save_dir.join("world.dat");
        let file = File::create(&save_path)?;
        let writer = BufWriter::new(file);
        
        bincode::serialize_into(writer, &world_save)?;
        
        log::info!("Game saved to {:?}", save_path);
        Ok(())
    }

    pub fn load(&self) -> Option<GameState> {
        let save_path = self.save_dir.join("world.dat");
        
        if !save_path.exists() {
            return None;
        }

        let file = File::open(&save_path).ok()?;
        let reader = BufReader::new(file);
        
        let world_save: WorldSave = bincode::deserialize_from(reader).ok()?;
        
        log::info!("Game loaded from {:?}", save_path);
        Some(world_save.to_state())
    }

    pub fn delete_save(&self) -> Result<(), SaveError> {
        let save_path = self.save_dir.join("world.dat");
        fs::remove_file(&save_path)?;
        Ok(())
    }

    pub fn has_save(&self) -> bool {
        self.save_dir.join("world.dat").exists()
    }

    pub fn list_saves(&self) -> Vec<SaveInfo> {
        let mut saves = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.save_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.path().extension().map(|e| e == "dat").unwrap_or(false) {
                    if let Ok(metadata) = entry.metadata() {
                        saves.push(SaveInfo {
                            name: entry.file_name().to_string_lossy().to_string(),
                            size: metadata.len(),
                            modified: metadata.modified().ok(),
                        });
                    }
                }
            }
        }
        
        saves
    }
}

impl Default for SaveSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable world data
#[derive(Serialize, Deserialize)]
pub struct WorldSave {
    pub seed: u64,
    pub player: PlayerSave,
    pub chunks: Vec<ChunkSave>,
    pub time_of_day: f32,
    pub day_count: u32,
    pub version: u32,
}

impl WorldSave {
    const VERSION: u32 = 1;

    pub fn from_state(state: &GameState) -> Self {
        let chunks = state.world.chunks.iter()
            .map(|((x, z), chunk)| ChunkSave {
                x: *x,
                z: *z,
                data: chunk.clone(),
            })
            .collect();

        Self {
            seed: state.world.seed,
            player: PlayerSave::from_player(&state.player),
            chunks,
            time_of_day: state.time_of_day,
            day_count: state.day_count,
            version: Self::VERSION,
        }
    }

    pub fn to_state(self) -> GameState {
        let mut world = World::new(self.seed);
        
        for chunk_save in self.chunks {
            world.chunks.insert((chunk_save.x, chunk_save.z), chunk_save.data);
        }

        let player = self.player.to_player();

        GameState {
            world,
            player,
            crafting: crate::CraftingSystem::new(),
            entities: Vec::new(),
            time_of_day: self.time_of_day,
            day_count: self.day_count,
            paused: false,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PlayerSave {
    pub position: [f32; 3],
    pub rotation: (f32, f32),
    pub inventory: crate::player::Inventory,
    pub stats: crate::player::PlayerStats,
}

impl PlayerSave {
    pub fn from_player(player: &Player) -> Self {
        Self {
            position: player.position.to_array(),
            rotation: player.rotation,
            inventory: player.inventory.clone(),
            stats: player.stats.clone(),
        }
    }

    pub fn to_player(self) -> Player {
        let mut player = Player::new();
        player.position = glam::Vec3::from_array(self.position);
        player.rotation = self.rotation;
        player.inventory = self.inventory;
        player.stats = self.stats;
        player
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChunkSave {
    pub x: i32,
    pub z: i32,
    pub data: Chunk,
}

#[derive(Debug)]
pub struct SaveInfo {
    pub name: String,
    pub size: u64,
    pub modified: Option<std::time::SystemTime>,
}

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    Serialize(String),
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        SaveError::Io(e)
    }
}

impl From<Box<bincode::ErrorKind>> for SaveError {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        SaveError::Serialize(e.to_string())
    }
}

/// Auto-save manager
pub struct AutoSave {
    interval: f32,
    timer: f32,
    enabled: bool,
}

impl AutoSave {
    pub fn new(interval_seconds: f32) -> Self {
        Self {
            interval: interval_seconds,
            timer: 0.0,
            enabled: true,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        if !self.enabled {
            return false;
        }

        self.timer += dt;
        
        if self.timer >= self.interval {
            self.timer = 0.0;
            true
        } else {
            false
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for AutoSave {
    fn default() -> Self {
        Self::new(60.0) // Auto-save every minute
    }
}
