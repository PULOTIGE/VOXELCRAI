//! Editor UI - panels, menus, dialogs

use glam::Vec2;
use super::project::{Project, ProjectTemplate};
use super::scene::{Scene, ObjectId, ObjectType, PrimitiveType};
use super::assets::{AssetManager, AssetType, BuiltinModel};
use super::tools::{EditorTool, ToolType};
use super::viewport::EditorViewport;
use super::inspector::Inspector;
use super::gameplay::GameplaySettings;

/// Main editor state
pub struct EditorState {
    pub project: Option<Project>,
    pub scenes: Vec<Scene>,
    pub active_scene: usize,
    pub assets: AssetManager,
    pub viewport: EditorViewport,
    pub tools: EditorTool,
    pub inspector: Inspector,
    
    // UI state
    pub show_new_project_dialog: bool,
    pub show_open_project_dialog: bool,
    pub show_save_project_dialog: bool,
    pub show_asset_browser: bool,
    pub show_hierarchy: bool,
    pub show_inspector: bool,
    pub show_settings: bool,
    pub show_about: bool,
    pub show_play_controls: bool,
    
    // Panel sizes
    pub hierarchy_width: f32,
    pub inspector_width: f32,
    pub asset_browser_height: f32,
    
    // Play mode
    pub is_playing: bool,
    pub is_paused: bool,
    
    // Status
    pub status_message: String,
    pub fps: f32,
}

impl EditorState {
    pub fn new() -> Self {
        let mut state = Self {
            project: None,
            scenes: vec![Scene::default_scene()],
            active_scene: 0,
            assets: AssetManager::new(),
            viewport: EditorViewport::default(),
            tools: EditorTool::new(),
            inspector: Inspector::new(),
            show_new_project_dialog: false,
            show_open_project_dialog: false,
            show_save_project_dialog: false,
            show_asset_browser: true,
            show_hierarchy: true,
            show_inspector: true,
            show_settings: false,
            show_about: false,
            show_play_controls: true,
            hierarchy_width: 250.0,
            inspector_width: 300.0,
            asset_browser_height: 200.0,
            is_playing: false,
            is_paused: false,
            status_message: "Ready".to_string(),
            fps: 0.0,
        };
        
        // Show new project dialog on start
        state.show_new_project_dialog = true;
        
        state
    }

    /// Get current scene
    pub fn current_scene(&self) -> Option<&Scene> {
        self.scenes.get(self.active_scene)
    }

    /// Get current scene mutably
    pub fn current_scene_mut(&mut self) -> Option<&mut Scene> {
        self.scenes.get_mut(self.active_scene)
    }

    /// Create new project
    pub fn new_project(&mut self, name: &str, template: ProjectTemplate) {
        let project = Project::from_template(name, template);
        self.project = Some(project);
        self.scenes = vec![Scene::default_scene()];
        self.active_scene = 0;
        self.show_new_project_dialog = false;
        self.status_message = format!("Created project: {}", name);
    }

    /// Add object to current scene
    pub fn add_object(&mut self, object_type: ObjectType) -> Option<ObjectId> {
        if let Some(scene) = self.current_scene_mut() {
            let name = match &object_type {
                ObjectType::Primitive(p) => format!("{:?}", p),
                ObjectType::Model(m) => m.clone(),
                ObjectType::Light(l) => format!("{:?} Light", l),
                ObjectType::SpawnPoint(t) => format!("{:?} Spawn", t),
                _ => "Object".to_string(),
            };
            
            let obj = super::scene::SceneObject::new(&name, object_type);
            let id = scene.add_object(obj);
            scene.select(id, false);
            Some(id)
        } else {
            None
        }
    }

    /// Start play mode
    pub fn start_play(&mut self) {
        self.is_playing = true;
        self.is_paused = false;
        self.status_message = "Playing...".to_string();
    }

    /// Pause play mode
    pub fn pause_play(&mut self) {
        self.is_paused = !self.is_paused;
        self.status_message = if self.is_paused { "Paused" } else { "Playing..." }.to_string();
    }

    /// Stop play mode
    pub fn stop_play(&mut self) {
        self.is_playing = false;
        self.is_paused = false;
        self.status_message = "Ready".to_string();
    }

    /// Handle keyboard shortcuts
    pub fn handle_shortcut(&mut self, key: KeyCode, ctrl: bool, shift: bool) {
        match (key, ctrl, shift) {
            // File
            (KeyCode::N, true, false) => self.show_new_project_dialog = true,
            (KeyCode::O, true, false) => self.show_open_project_dialog = true,
            (KeyCode::S, true, false) => self.save_project(),
            (KeyCode::S, true, true) => self.show_save_project_dialog = true,
            
            // Edit - undo
            (KeyCode::Z, true, false) => {
                if self.active_scene < self.scenes.len() {
                    let scene = &mut self.scenes[self.active_scene];
                    self.tools.undo(scene);
                }
            }
            // Edit - redo
            (KeyCode::Y, true, false) | (KeyCode::Z, true, true) => {
                if self.active_scene < self.scenes.len() {
                    let scene = &mut self.scenes[self.active_scene];
                    self.tools.redo(scene);
                }
            }
            // Edit - delete
            (KeyCode::Delete, false, false) => {
                if self.active_scene < self.scenes.len() {
                    let scene = &mut self.scenes[self.active_scene];
                    self.tools.delete_selected(scene);
                }
            }
            // Edit - duplicate
            (KeyCode::D, true, false) => {
                if self.active_scene < self.scenes.len() {
                    let scene = &mut self.scenes[self.active_scene];
                    self.tools.duplicate_selected(scene);
                }
            }
            
            // Tools
            (KeyCode::Q, false, false) => self.tools.set_tool(ToolType::Select),
            (KeyCode::W, false, false) => self.tools.set_tool(ToolType::Move),
            (KeyCode::E, false, false) => self.tools.set_tool(ToolType::Rotate),
            (KeyCode::R, false, false) => self.tools.set_tool(ToolType::Scale),
            (KeyCode::T, false, false) => self.tools.set_tool(ToolType::Place),
            
            // View - focus
            (KeyCode::F, false, false) => {
                let focus_pos = if let Some(scene) = self.current_scene() {
                    self.tools.focus_selected(scene)
                } else {
                    None
                };
                if let Some(pos) = focus_pos {
                    self.viewport.camera.focus_on(pos, Some(10.0));
                }
            }
            (KeyCode::G, false, false) => self.viewport.grid_visible = !self.viewport.grid_visible,
            
            // Play
            (KeyCode::P, true, false) => {
                if self.is_playing {
                    self.stop_play();
                } else {
                    self.start_play();
                }
            }
            (KeyCode::Space, false, false) if self.is_playing => {
                self.pause_play();
            }
            
            _ => {}
        }
    }

    /// Save current project
    fn save_project(&mut self) {
        let path_to_save = if let Some(project) = &self.project {
            project.directory().map(|p| p.to_path_buf())
        } else {
            None
        };
        
        if let Some(path) = path_to_save {
            if let Some(project) = &mut self.project {
                match project.save(&path) {
                    Ok(()) => self.status_message = "Project saved".to_string(),
                    Err(e) => self.status_message = format!("Save error: {}", e),
                }
            }
        } else {
            self.show_save_project_dialog = true;
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

/// Keyboard key codes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Control
    Escape, Tab, Space, Enter, Backspace, Delete,
    Left, Right, Up, Down,
    Home, End, PageUp, PageDown,
    Insert,
    
    // Modifiers (for reference)
    Shift, Ctrl, Alt,
}

/// Menu item
#[derive(Clone, Debug)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub checked: bool,
    pub children: Vec<MenuItem>,
}

impl MenuItem {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            shortcut: None,
            enabled: true,
            checked: false,
            children: Vec::new(),
        }
    }

    pub fn with_shortcut(mut self, shortcut: &str) -> Self {
        self.shortcut = Some(shortcut.to_string());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn with_children(mut self, children: Vec<MenuItem>) -> Self {
        self.children = children;
        self
    }
}

/// Generate main menu structure
pub fn build_main_menu(state: &EditorState) -> Vec<MenuItem> {
    vec![
        MenuItem::new("File").with_children(vec![
            MenuItem::new("New Project...").with_shortcut("Ctrl+N"),
            MenuItem::new("Open Project...").with_shortcut("Ctrl+O"),
            MenuItem::new("--"), // Separator
            MenuItem::new("Save").with_shortcut("Ctrl+S"),
            MenuItem::new("Save As...").with_shortcut("Ctrl+Shift+S"),
            MenuItem::new("--"),
            MenuItem::new("Export Game..."),
            MenuItem::new("--"),
            MenuItem::new("Exit").with_shortcut("Alt+F4"),
        ]),
        MenuItem::new("Edit").with_children(vec![
            MenuItem::new("Undo").with_shortcut("Ctrl+Z"),
            MenuItem::new("Redo").with_shortcut("Ctrl+Y"),
            MenuItem::new("--"),
            MenuItem::new("Cut").with_shortcut("Ctrl+X"),
            MenuItem::new("Copy").with_shortcut("Ctrl+C"),
            MenuItem::new("Paste").with_shortcut("Ctrl+V"),
            MenuItem::new("Delete").with_shortcut("Del"),
            MenuItem::new("--"),
            MenuItem::new("Duplicate").with_shortcut("Ctrl+D"),
            MenuItem::new("Select All").with_shortcut("Ctrl+A"),
        ]),
        MenuItem::new("Create").with_children(vec![
            MenuItem::new("Primitives").with_children(vec![
                MenuItem::new("Box"),
                MenuItem::new("Sphere"),
                MenuItem::new("Cylinder"),
                MenuItem::new("Capsule"),
                MenuItem::new("Plane"),
                MenuItem::new("Wedge"),
                MenuItem::new("Arch"),
                MenuItem::new("Stairs"),
            ]),
            MenuItem::new("Lights").with_children(vec![
                MenuItem::new("Directional Light"),
                MenuItem::new("Point Light"),
                MenuItem::new("Spot Light"),
            ]),
            MenuItem::new("Gameplay").with_children(vec![
                MenuItem::new("T Spawn Point"),
                MenuItem::new("CT Spawn Point"),
                MenuItem::new("Bombsite"),
                MenuItem::new("Buy Zone"),
            ]),
            MenuItem::new("--"),
            MenuItem::new("Empty Object"),
            MenuItem::new("Group"),
        ]),
        MenuItem::new("View").with_children(vec![
            MenuItem::new("Hierarchy").checked(state.show_hierarchy),
            MenuItem::new("Inspector").checked(state.show_inspector),
            MenuItem::new("Asset Browser").checked(state.show_asset_browser),
            MenuItem::new("--"),
            MenuItem::new("Grid").with_shortcut("G").checked(state.viewport.grid_visible),
            MenuItem::new("Gizmos").checked(state.viewport.show_gizmos),
            MenuItem::new("--"),
            MenuItem::new("Focus Selection").with_shortcut("F"),
            MenuItem::new("Frame All").with_shortcut("Shift+F"),
        ]),
        MenuItem::new("Play").with_children(vec![
            MenuItem::new("Play").with_shortcut("Ctrl+P"),
            MenuItem::new("Pause").with_shortcut("Space"),
            MenuItem::new("Stop").with_shortcut("Esc"),
        ]),
        MenuItem::new("Settings").with_children(vec![
            MenuItem::new("Project Settings..."),
            MenuItem::new("Gameplay Settings..."),
            MenuItem::new("Input Settings..."),
            MenuItem::new("--"),
            MenuItem::new("Editor Preferences..."),
        ]),
        MenuItem::new("Help").with_children(vec![
            MenuItem::new("Documentation"),
            MenuItem::new("Tutorials"),
            MenuItem::new("--"),
            MenuItem::new("About VoxelForge"),
        ]),
    ]
}

/// Build toolbar items
pub fn build_toolbar(state: &EditorState) -> Vec<ToolbarItem> {
    vec![
        // File operations
        ToolbarItem::Button { icon: "📄", tooltip: "New Project", action: "new_project" },
        ToolbarItem::Button { icon: "📂", tooltip: "Open Project", action: "open_project" },
        ToolbarItem::Button { icon: "💾", tooltip: "Save", action: "save" },
        ToolbarItem::Separator,
        
        // History
        ToolbarItem::Button { icon: "↩", tooltip: "Undo (Ctrl+Z)", action: "undo" },
        ToolbarItem::Button { icon: "↪", tooltip: "Redo (Ctrl+Y)", action: "redo" },
        ToolbarItem::Separator,
        
        // Transform tools
        ToolbarItem::Toggle { 
            icon: "🖱", 
            tooltip: "Select (Q)", 
            action: "tool_select",
            active: state.tools.current_tool == ToolType::Select,
        },
        ToolbarItem::Toggle { 
            icon: "✥", 
            tooltip: "Move (W)", 
            action: "tool_move",
            active: state.tools.current_tool == ToolType::Move,
        },
        ToolbarItem::Toggle { 
            icon: "🔄", 
            tooltip: "Rotate (E)", 
            action: "tool_rotate",
            active: state.tools.current_tool == ToolType::Rotate,
        },
        ToolbarItem::Toggle { 
            icon: "⇔", 
            tooltip: "Scale (R)", 
            action: "tool_scale",
            active: state.tools.current_tool == ToolType::Scale,
        },
        ToolbarItem::Separator,
        
        // Snap
        ToolbarItem::Toggle { 
            icon: "⊞", 
            tooltip: "Snap to Grid", 
            action: "toggle_snap",
            active: state.tools.snap.position_snap,
        },
        ToolbarItem::Separator,
        
        // Play controls
        ToolbarItem::Toggle { 
            icon: "▶", 
            tooltip: "Play (Ctrl+P)", 
            action: "play",
            active: state.is_playing && !state.is_paused,
        },
        ToolbarItem::Toggle { 
            icon: "⏸", 
            tooltip: "Pause (Space)", 
            action: "pause",
            active: state.is_paused,
        },
        ToolbarItem::Button { 
            icon: "⏹", 
            tooltip: "Stop", 
            action: "stop",
        },
    ]
}

/// Toolbar item types
#[derive(Clone, Debug)]
pub enum ToolbarItem {
    Button {
        icon: &'static str,
        tooltip: &'static str,
        action: &'static str,
    },
    Toggle {
        icon: &'static str,
        tooltip: &'static str,
        action: &'static str,
        active: bool,
    },
    Separator,
}
