//! VoxelForge - 3D Game Constructor
//! 
//! A visual editor for creating FPS games based on the VoxelStrike engine.
//! 
//! Features:
//! - Project management (create, save, load)
//! - 3D map editor with placement tools
//! - Asset browser for models and textures
//! - Object inspector with property editing
//! - Gameplay settings (weapons, enemies, rules)
//! - Play mode for testing
//! - Export to standalone game

pub mod project;
pub mod scene;
pub mod assets;
pub mod tools;
pub mod viewport;
pub mod inspector;
pub mod gameplay;
pub mod ui;

pub use project::{Project, ProjectSettings};
pub use scene::{Scene, SceneObject, ObjectType};
pub use assets::{AssetManager, Asset, AssetType};
pub use tools::{EditorTool, ToolType};
pub use viewport::EditorViewport;
pub use inspector::Inspector;
pub use gameplay::GameplaySettings;
