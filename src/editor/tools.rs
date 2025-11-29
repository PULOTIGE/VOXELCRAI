//! Editor tools - selection, transform, placement

use glam::{Vec3, Quat, Mat4, Vec2};
use super::scene::{ObjectId, Scene, SceneObject, ObjectType, PrimitiveType};

/// Editor tool types
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ToolType {
    #[default]
    Select,
    Move,
    Rotate,
    Scale,
    Place,
    Paint,
    Measure,
    Vertex,
}

impl ToolType {
    pub fn name(&self) -> &str {
        match self {
            ToolType::Select => "Select",
            ToolType::Move => "Move",
            ToolType::Rotate => "Rotate",
            ToolType::Scale => "Scale",
            ToolType::Place => "Place",
            ToolType::Paint => "Paint",
            ToolType::Measure => "Measure",
            ToolType::Vertex => "Vertex",
        }
    }

    pub fn hotkey(&self) -> &str {
        match self {
            ToolType::Select => "Q",
            ToolType::Move => "W",
            ToolType::Rotate => "E",
            ToolType::Scale => "R",
            ToolType::Place => "T",
            ToolType::Paint => "B",
            ToolType::Measure => "M",
            ToolType::Vertex => "V",
        }
    }
}

/// Transform space
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum TransformSpace {
    #[default]
    World,
    Local,
}

/// Snap settings
#[derive(Clone, Debug)]
pub struct SnapSettings {
    pub position_snap: bool,
    pub position_grid: f32,
    pub rotation_snap: bool,
    pub rotation_angle: f32,
    pub scale_snap: bool,
    pub scale_step: f32,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            position_snap: true,
            position_grid: 0.5,
            rotation_snap: true,
            rotation_angle: 15.0,
            scale_snap: false,
            scale_step: 0.1,
        }
    }
}

/// Main editor tool controller
pub struct EditorTool {
    pub current_tool: ToolType,
    pub transform_space: TransformSpace,
    pub snap: SnapSettings,
    
    // Tool state
    pub is_dragging: bool,
    pub drag_start: Option<Vec3>,
    pub drag_axis: Option<Axis>,
    
    // Placement state
    pub placement_model: Option<String>,
    pub placement_preview: Option<Vec3>,
    
    // Gizmo state
    pub hovered_axis: Option<Axis>,
    
    // Undo/Redo
    history: Vec<HistoryEntry>,
    history_index: usize,
}

impl EditorTool {
    pub fn new() -> Self {
        Self {
            current_tool: ToolType::Select,
            transform_space: TransformSpace::World,
            snap: SnapSettings::default(),
            is_dragging: false,
            drag_start: None,
            drag_axis: None,
            placement_model: None,
            placement_preview: None,
            hovered_axis: None,
            history: Vec::new(),
            history_index: 0,
        }
    }

    /// Set current tool
    pub fn set_tool(&mut self, tool: ToolType) {
        self.current_tool = tool;
        self.is_dragging = false;
        self.drag_start = None;
        self.drag_axis = None;
    }

    /// Handle mouse down
    pub fn on_mouse_down(&mut self, scene: &mut Scene, world_pos: Vec3, screen_pos: Vec2, camera_pos: Vec3) {
        match self.current_tool {
            ToolType::Select => {
                if let Some(id) = self.raycast_object(scene, camera_pos, (world_pos - camera_pos).normalize()) {
                    scene.select(id, false);
                } else {
                    scene.deselect_all();
                }
            }
            ToolType::Move | ToolType::Rotate | ToolType::Scale => {
                if !scene.selected.is_empty() {
                    if let Some(axis) = self.hovered_axis {
                        self.is_dragging = true;
                        self.drag_start = Some(world_pos);
                        self.drag_axis = Some(axis);
                        
                        // Save state for undo
                        self.save_transform_state(scene);
                    }
                }
            }
            ToolType::Place => {
                if let Some(model) = &self.placement_model.clone() {
                    let snapped = self.snap_position(world_pos);
                    let obj = SceneObject::new(model, ObjectType::Model(model.clone()))
                        .with_position(snapped);
                    scene.add_object(obj);
                }
            }
            _ => {}
        }
    }

    /// Handle mouse move
    pub fn on_mouse_move(&mut self, scene: &mut Scene, world_pos: Vec3, delta: Vec2) {
        if self.is_dragging {
            if let (Some(start), Some(axis)) = (self.drag_start, self.drag_axis) {
                let movement = world_pos - start;
                
                match self.current_tool {
                    ToolType::Move => {
                        let axis_vec = axis.to_vec3();
                        let projected = axis_vec * movement.dot(axis_vec);
                        let snapped = self.snap_position(projected);
                        
                        for id in &scene.selected.clone() {
                            if let Some(obj) = scene.get_object_mut(*id) {
                                obj.position += snapped;
                            }
                        }
                        self.drag_start = Some(world_pos);
                    }
                    ToolType::Rotate => {
                        let angle = delta.x * 0.5;
                        let snapped = self.snap_rotation(angle);
                        let rotation = Quat::from_axis_angle(axis.to_vec3(), snapped.to_radians());
                        
                        for id in &scene.selected.clone() {
                            if let Some(obj) = scene.get_object_mut(*id) {
                                obj.rotation = rotation * obj.rotation;
                            }
                        }
                    }
                    ToolType::Scale => {
                        let scale_factor = 1.0 + delta.x * 0.01;
                        let snapped = self.snap_scale(scale_factor);
                        let scale_vec = match axis {
                            Axis::X => Vec3::new(snapped, 1.0, 1.0),
                            Axis::Y => Vec3::new(1.0, snapped, 1.0),
                            Axis::Z => Vec3::new(1.0, 1.0, snapped),
                            Axis::All => Vec3::splat(snapped),
                        };
                        
                        for id in &scene.selected.clone() {
                            if let Some(obj) = scene.get_object_mut(*id) {
                                obj.scale *= scale_vec;
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else if self.current_tool == ToolType::Place {
            self.placement_preview = Some(self.snap_position(world_pos));
        }
    }

    /// Handle mouse up
    pub fn on_mouse_up(&mut self) {
        self.is_dragging = false;
        self.drag_start = None;
        self.drag_axis = None;
    }

    /// Raycast to find object
    fn raycast_object(&self, scene: &Scene, origin: Vec3, direction: Vec3) -> Option<ObjectId> {
        let mut closest: Option<(ObjectId, f32)> = None;
        
        for (id, obj) in &scene.objects {
            if !obj.visible {
                continue;
            }
            
            // Simple AABB intersection
            let bounds = self.get_object_bounds(obj);
            if let Some(t) = self.ray_aabb_intersection(origin, direction, bounds.0, bounds.1) {
                if closest.map_or(true, |(_, dist)| t < dist) {
                    closest = Some((*id, t));
                }
            }
        }
        
        closest.map(|(id, _)| id)
    }

    /// Get object bounding box
    fn get_object_bounds(&self, obj: &SceneObject) -> (Vec3, Vec3) {
        let half_size = obj.scale * 0.5;
        (obj.position - half_size, obj.position + half_size)
    }

    /// Ray-AABB intersection
    fn ray_aabb_intersection(&self, origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
        let inv_dir = Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);
        
        let t1 = (min.x - origin.x) * inv_dir.x;
        let t2 = (max.x - origin.x) * inv_dir.x;
        let t3 = (min.y - origin.y) * inv_dir.y;
        let t4 = (max.y - origin.y) * inv_dir.y;
        let t5 = (min.z - origin.z) * inv_dir.z;
        let t6 = (max.z - origin.z) * inv_dir.z;
        
        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));
        
        if tmax >= tmin && tmax >= 0.0 {
            Some(if tmin >= 0.0 { tmin } else { tmax })
        } else {
            None
        }
    }

    /// Snap position to grid
    fn snap_position(&self, pos: Vec3) -> Vec3 {
        if self.snap.position_snap {
            let grid = self.snap.position_grid;
            Vec3::new(
                (pos.x / grid).round() * grid,
                (pos.y / grid).round() * grid,
                (pos.z / grid).round() * grid,
            )
        } else {
            pos
        }
    }

    /// Snap rotation angle
    fn snap_rotation(&self, angle: f32) -> f32 {
        if self.snap.rotation_snap {
            let step = self.snap.rotation_angle;
            (angle / step).round() * step
        } else {
            angle
        }
    }

    /// Snap scale value
    fn snap_scale(&self, scale: f32) -> f32 {
        if self.snap.scale_snap {
            let step = self.snap.scale_step;
            (scale / step).round() * step
        } else {
            scale
        }
    }

    /// Save transform state for undo
    fn save_transform_state(&mut self, scene: &Scene) {
        let transforms: Vec<(ObjectId, Vec3, Quat, Vec3)> = scene
            .selected
            .iter()
            .filter_map(|id| {
                scene.get_object(*id).map(|obj| (*id, obj.position, obj.rotation, obj.scale))
            })
            .collect();
        
        // Truncate history at current position
        self.history.truncate(self.history_index);
        self.history.push(HistoryEntry::Transform(transforms));
        self.history_index = self.history.len();
    }

    /// Undo last action
    pub fn undo(&mut self, scene: &mut Scene) {
        if self.history_index > 0 {
            self.history_index -= 1;
            if let Some(HistoryEntry::Transform(transforms)) = self.history.get(self.history_index) {
                for (id, pos, rot, scale) in transforms {
                    if let Some(obj) = scene.get_object_mut(*id) {
                        obj.position = *pos;
                        obj.rotation = *rot;
                        obj.scale = *scale;
                    }
                }
            }
        }
    }

    /// Redo last undone action
    pub fn redo(&mut self, scene: &mut Scene) {
        if self.history_index < self.history.len() {
            if let Some(HistoryEntry::Transform(transforms)) = self.history.get(self.history_index) {
                for (id, pos, rot, scale) in transforms {
                    if let Some(obj) = scene.get_object_mut(*id) {
                        obj.position = *pos;
                        obj.rotation = *rot;
                        obj.scale = *scale;
                    }
                }
            }
            self.history_index += 1;
        }
    }

    /// Delete selected objects
    pub fn delete_selected(&mut self, scene: &mut Scene) {
        let selected = scene.selected.clone();
        for id in selected {
            scene.remove_object(id);
        }
    }

    /// Duplicate selected objects
    pub fn duplicate_selected(&mut self, scene: &mut Scene) {
        let selected = scene.selected.clone();
        let mut new_selected = Vec::new();
        for id in selected {
            if let Some(new_id) = scene.duplicate_object(id) {
                new_selected.push(new_id);
            }
        }
        scene.selected = new_selected;
    }

    /// Focus camera on selected objects
    pub fn focus_selected(&self, scene: &Scene) -> Option<Vec3> {
        if scene.selected.is_empty() {
            return None;
        }
        
        let mut center = Vec3::ZERO;
        let mut count = 0;
        
        for id in &scene.selected {
            if let Some(obj) = scene.get_object(*id) {
                center += obj.position;
                count += 1;
            }
        }
        
        if count > 0 {
            Some(center / count as f32)
        } else {
            None
        }
    }
}

impl Default for EditorTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Transform axis
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
    All,
}

impl Axis {
    pub fn to_vec3(&self) -> Vec3 {
        match self {
            Axis::X => Vec3::X,
            Axis::Y => Vec3::Y,
            Axis::Z => Vec3::Z,
            Axis::All => Vec3::ONE.normalize(),
        }
    }

    pub fn color(&self) -> [f32; 3] {
        match self {
            Axis::X => [1.0, 0.2, 0.2],
            Axis::Y => [0.2, 1.0, 0.2],
            Axis::Z => [0.2, 0.2, 1.0],
            Axis::All => [1.0, 1.0, 0.2],
        }
    }
}

/// History entry for undo/redo
#[derive(Clone, Debug)]
enum HistoryEntry {
    Transform(Vec<(ObjectId, Vec3, Quat, Vec3)>),
}
