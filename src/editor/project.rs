//! Project management - create, save, load projects

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use super::scene::Scene;
use super::gameplay::{GameplaySettings, GameMode};
use super::assets::AssetManager;

/// Project file extension
pub const PROJECT_EXTENSION: &str = "vforge";

/// Scene file extension  
pub const SCENE_EXTENSION: &str = "vscene";

/// Project metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub created_at: String,
    pub modified_at: String,
    pub thumbnail: Option<String>,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            name: "New Project".to_string(),
            author: "Unknown".to_string(),
            version: "1.0.0".to_string(),
            description: "A VoxelForge game project".to_string(),
            created_at: chrono_lite(),
            modified_at: chrono_lite(),
            thumbnail: None,
        }
    }
}

/// Main project structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub settings: ProjectSettings,
    pub scenes: Vec<String>,           // Scene file names
    pub active_scene: Option<usize>,   // Index of active scene
    pub gameplay: GameplaySettings,
    
    #[serde(skip)]
    pub path: Option<PathBuf>,
    
    #[serde(skip)]
    pub is_modified: bool,
}

impl Project {
    /// Create a new empty project
    pub fn new(name: &str) -> Self {
        Self {
            settings: ProjectSettings {
                name: name.to_string(),
                ..Default::default()
            },
            scenes: vec!["main.vscene".to_string()],
            active_scene: Some(0),
            gameplay: GameplaySettings::default(),
            path: None,
            is_modified: true,
        }
    }

    /// Create a new project from template
    pub fn from_template(name: &str, template: ProjectTemplate) -> Self {
        let mut project = Self::new(name);
        
        match template {
            ProjectTemplate::Empty => {
                // Just empty project
            }
            ProjectTemplate::Deathmatch => {
                project.settings.description = "A deathmatch FPS game".to_string();
                project.gameplay.game_mode = GameMode::Deathmatch;
                project.gameplay.respawn_enabled = true;
                project.gameplay.friendly_fire = false;
            }
            ProjectTemplate::TeamDeathmatch => {
                project.settings.description = "A team-based FPS game".to_string();
                project.gameplay.game_mode = GameMode::TeamDeathmatch;
                project.gameplay.teams_enabled = true;
                project.gameplay.respawn_enabled = true;
            }
            ProjectTemplate::BombDefusal => {
                project.settings.description = "A bomb defusal game like CS".to_string();
                project.gameplay.game_mode = GameMode::BombDefusal;
                project.gameplay.teams_enabled = true;
                project.gameplay.bomb_enabled = true;
                project.gameplay.buy_system_enabled = true;
            }
        }
        
        project
    }

    /// Save project to disk
    pub fn save(&mut self, path: &Path) -> Result<(), ProjectError> {
        // Create project directory if needed
        if !path.exists() {
            fs::create_dir_all(path)?;
        }

        // Update modified time
        self.settings.modified_at = chrono_lite();
        
        // Save project file
        let project_file = path.join(format!("{}.{}", self.settings.name, PROJECT_EXTENSION));
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&project_file, json)?;
        
        self.path = Some(path.to_path_buf());
        self.is_modified = false;
        
        Ok(())
    }

    /// Load project from disk
    pub fn load(path: &Path) -> Result<Self, ProjectError> {
        // Find project file
        let project_file = if path.is_file() {
            path.to_path_buf()
        } else {
            // Look for .vforge file in directory
            let entries = fs::read_dir(path)?;
            let mut project_file = None;
            for entry in entries.flatten() {
                if entry.path().extension().map_or(false, |e| e == PROJECT_EXTENSION) {
                    project_file = Some(entry.path());
                    break;
                }
            }
            project_file.ok_or(ProjectError::NotFound)?
        };

        let json = fs::read_to_string(&project_file)?;
        let mut project: Project = serde_json::from_str(&json)?;
        project.path = Some(project_file.parent().unwrap().to_path_buf());
        project.is_modified = false;
        
        Ok(project)
    }

    /// Get project directory
    pub fn directory(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Add a new scene to project
    pub fn add_scene(&mut self, name: &str) -> usize {
        let scene_name = format!("{}.{}", name, SCENE_EXTENSION);
        self.scenes.push(scene_name);
        self.is_modified = true;
        self.scenes.len() - 1
    }

    /// Remove scene from project
    pub fn remove_scene(&mut self, index: usize) -> bool {
        if index < self.scenes.len() && self.scenes.len() > 1 {
            self.scenes.remove(index);
            if let Some(active) = self.active_scene {
                if active >= self.scenes.len() {
                    self.active_scene = Some(self.scenes.len() - 1);
                }
            }
            self.is_modified = true;
            true
        } else {
            false
        }
    }

    /// Get scene file path
    pub fn scene_path(&self, index: usize) -> Option<PathBuf> {
        if let (Some(dir), Some(scene)) = (self.directory(), self.scenes.get(index)) {
            Some(dir.join("scenes").join(scene))
        } else {
            None
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new("Untitled")
    }
}

/// Project templates
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProjectTemplate {
    Empty,
    Deathmatch,
    TeamDeathmatch,
    BombDefusal,
}

/// Project errors
#[derive(Debug)]
pub enum ProjectError {
    Io(std::io::Error),
    Json(serde_json::Error),
    NotFound,
}

impl From<std::io::Error> for ProjectError {
    fn from(err: std::io::Error) -> Self {
        ProjectError::Io(err)
    }
}

impl From<serde_json::Error> for ProjectError {
    fn from(err: serde_json::Error) -> Self {
        ProjectError::Json(err)
    }
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectError::Io(e) => write!(f, "IO error: {}", e),
            ProjectError::Json(e) => write!(f, "JSON error: {}", e),
            ProjectError::NotFound => write!(f, "Project not found"),
        }
    }
}

/// Simple timestamp without chrono dependency
fn chrono_lite() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}
