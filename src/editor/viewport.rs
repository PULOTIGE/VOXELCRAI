//! 3D Viewport - camera, rendering, gizmos

use glam::{Vec3, Vec2, Mat4, Quat};
use super::scene::{Scene, ObjectId, SceneObject};
use super::tools::Axis;

/// Editor camera for 3D viewport
#[derive(Clone, Debug)]
pub struct EditorCamera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
    
    // Orbit mode
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    
    // Navigation
    pub move_speed: f32,
    pub rotate_speed: f32,
    pub zoom_speed: f32,
    pub pan_speed: f32,
    
    // State
    pub is_orbiting: bool,
    pub is_panning: bool,
    pub is_flying: bool,
}

impl EditorCamera {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(10.0, 10.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
            aspect: 16.0 / 9.0,
            yaw: -135.0f32.to_radians(),
            pitch: 30.0f32.to_radians(),
            distance: 15.0,
            move_speed: 10.0,
            rotate_speed: 0.3,
            zoom_speed: 1.0,
            pan_speed: 0.01,
            is_orbiting: false,
            is_panning: false,
            is_flying: false,
        }
    }

    /// Get view matrix
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Get projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov.to_radians(), self.aspect, self.near, self.far)
    }

    /// Get view-projection matrix
    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Update camera from orbit parameters
    pub fn update_from_orbit(&mut self) {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        self.position = self.target + Vec3::new(x, y, z);
    }

    /// Orbit around target
    pub fn orbit(&mut self, delta: Vec2) {
        self.yaw += delta.x * self.rotate_speed * 0.01;
        self.pitch = (self.pitch + delta.y * self.rotate_speed * 0.01)
            .clamp(-89.0f32.to_radians(), 89.0f32.to_radians());
        self.update_from_orbit();
    }

    /// Pan camera
    pub fn pan(&mut self, delta: Vec2) {
        let right = (self.target - self.position).cross(self.up).normalize();
        let up = right.cross((self.target - self.position).normalize());
        
        let pan = right * -delta.x * self.pan_speed * self.distance
            + up * delta.y * self.pan_speed * self.distance;
        
        self.position += pan;
        self.target += pan;
    }

    /// Zoom in/out
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance - delta * self.zoom_speed).max(0.5);
        self.update_from_orbit();
    }

    /// Focus on position
    pub fn focus_on(&mut self, position: Vec3, distance: Option<f32>) {
        self.target = position;
        if let Some(d) = distance {
            self.distance = d;
        }
        self.update_from_orbit();
    }

    /// Fly mode movement (WASD)
    pub fn fly_move(&mut self, forward: f32, right: f32, up: f32, delta_time: f32) {
        let dir = (self.target - self.position).normalize();
        let right_dir = dir.cross(self.up).normalize();
        
        let movement = dir * forward + right_dir * right + self.up * up;
        let offset = movement * self.move_speed * delta_time;
        
        self.position += offset;
        self.target += offset;
    }

    /// Screen to world ray
    pub fn screen_to_ray(&self, screen_pos: Vec2, screen_size: Vec2) -> (Vec3, Vec3) {
        let ndc = Vec2::new(
            (screen_pos.x / screen_size.x) * 2.0 - 1.0,
            1.0 - (screen_pos.y / screen_size.y) * 2.0,
        );
        
        let inv_vp = (self.projection_matrix() * self.view_matrix()).inverse();
        
        let near_point = inv_vp.project_point3(Vec3::new(ndc.x, ndc.y, -1.0));
        let far_point = inv_vp.project_point3(Vec3::new(ndc.x, ndc.y, 1.0));
        
        let direction = (far_point - near_point).normalize();
        
        (self.position, direction)
    }

    /// World to screen position
    pub fn world_to_screen(&self, world_pos: Vec3, screen_size: Vec2) -> Option<Vec2> {
        let clip = self.view_projection() * world_pos.extend(1.0);
        
        if clip.w <= 0.0 {
            return None;
        }
        
        let ndc = Vec3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w);
        
        if ndc.z < 0.0 || ndc.z > 1.0 {
            return None;
        }
        
        Some(Vec2::new(
            (ndc.x + 1.0) * 0.5 * screen_size.x,
            (1.0 - ndc.y) * 0.5 * screen_size.y,
        ))
    }

    /// Get camera forward direction
    pub fn forward(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    /// Get camera right direction
    pub fn right(&self) -> Vec3 {
        self.forward().cross(self.up).normalize()
    }
}

impl Default for EditorCamera {
    fn default() -> Self {
        Self::new()
    }
}

/// Editor viewport
pub struct EditorViewport {
    pub camera: EditorCamera,
    pub size: Vec2,
    pub grid_visible: bool,
    pub grid_size: f32,
    pub grid_divisions: u32,
    pub show_gizmos: bool,
    pub show_icons: bool,
    pub show_selection_outline: bool,
    pub render_mode: RenderMode,
    pub shading_mode: ShadingMode,
}

impl EditorViewport {
    pub fn new(width: f32, height: f32) -> Self {
        let mut camera = EditorCamera::new();
        camera.aspect = width / height;
        
        Self {
            camera,
            size: Vec2::new(width, height),
            grid_visible: true,
            grid_size: 100.0,
            grid_divisions: 100,
            show_gizmos: true,
            show_icons: true,
            show_selection_outline: true,
            render_mode: RenderMode::Lit,
            shading_mode: ShadingMode::Solid,
        }
    }

    /// Resize viewport
    pub fn resize(&mut self, width: f32, height: f32) {
        self.size = Vec2::new(width, height);
        self.camera.aspect = width / height;
    }

    /// Generate grid mesh data
    pub fn generate_grid_vertices(&self) -> Vec<GridVertex> {
        let mut vertices = Vec::new();
        let half = self.grid_size / 2.0;
        let step = self.grid_size / self.grid_divisions as f32;
        
        let main_color = [0.3, 0.3, 0.3, 1.0];
        let sub_color = [0.2, 0.2, 0.2, 0.5];
        
        for i in 0..=self.grid_divisions {
            let pos = -half + i as f32 * step;
            let color = if i % 10 == 0 { main_color } else { sub_color };
            
            // X lines
            vertices.push(GridVertex { position: [pos, 0.0, -half], color });
            vertices.push(GridVertex { position: [pos, 0.0, half], color });
            
            // Z lines
            vertices.push(GridVertex { position: [-half, 0.0, pos], color });
            vertices.push(GridVertex { position: [half, 0.0, pos], color });
        }
        
        // Axis lines
        // X axis (red)
        vertices.push(GridVertex { position: [0.0, 0.0, 0.0], color: [1.0, 0.2, 0.2, 1.0] });
        vertices.push(GridVertex { position: [half, 0.0, 0.0], color: [1.0, 0.2, 0.2, 1.0] });
        
        // Z axis (blue)
        vertices.push(GridVertex { position: [0.0, 0.0, 0.0], color: [0.2, 0.2, 1.0, 1.0] });
        vertices.push(GridVertex { position: [0.0, 0.0, half], color: [0.2, 0.2, 1.0, 1.0] });
        
        vertices
    }

    /// Generate transform gizmo data
    pub fn generate_gizmo(&self, position: Vec3, gizmo_type: GizmoType, hovered: Option<Axis>) -> Vec<GizmoVertex> {
        let mut vertices = Vec::new();
        let scale = self.camera.distance * 0.1; // Scale with distance
        
        match gizmo_type {
            GizmoType::Translate => {
                // X arrow (red)
                let x_color = if hovered == Some(Axis::X) { [1.0, 0.8, 0.0, 1.0] } else { [1.0, 0.2, 0.2, 1.0] };
                vertices.extend(self.arrow_vertices(position, Vec3::X * scale, x_color));
                
                // Y arrow (green)
                let y_color = if hovered == Some(Axis::Y) { [1.0, 0.8, 0.0, 1.0] } else { [0.2, 1.0, 0.2, 1.0] };
                vertices.extend(self.arrow_vertices(position, Vec3::Y * scale, y_color));
                
                // Z arrow (blue)
                let z_color = if hovered == Some(Axis::Z) { [1.0, 0.8, 0.0, 1.0] } else { [0.2, 0.2, 1.0, 1.0] };
                vertices.extend(self.arrow_vertices(position, Vec3::Z * scale, z_color));
            }
            GizmoType::Rotate => {
                // Rotation circles
                let x_color = if hovered == Some(Axis::X) { [1.0, 0.8, 0.0, 1.0] } else { [1.0, 0.2, 0.2, 1.0] };
                vertices.extend(self.circle_vertices(position, Vec3::X, scale, x_color));
                
                let y_color = if hovered == Some(Axis::Y) { [1.0, 0.8, 0.0, 1.0] } else { [0.2, 1.0, 0.2, 1.0] };
                vertices.extend(self.circle_vertices(position, Vec3::Y, scale, y_color));
                
                let z_color = if hovered == Some(Axis::Z) { [1.0, 0.8, 0.0, 1.0] } else { [0.2, 0.2, 1.0, 1.0] };
                vertices.extend(self.circle_vertices(position, Vec3::Z, scale, z_color));
            }
            GizmoType::Scale => {
                // Scale boxes
                let x_color = if hovered == Some(Axis::X) { [1.0, 0.8, 0.0, 1.0] } else { [1.0, 0.2, 0.2, 1.0] };
                vertices.extend(self.scale_box_vertices(position, Vec3::X * scale, x_color));
                
                let y_color = if hovered == Some(Axis::Y) { [1.0, 0.8, 0.0, 1.0] } else { [0.2, 1.0, 0.2, 1.0] };
                vertices.extend(self.scale_box_vertices(position, Vec3::Y * scale, y_color));
                
                let z_color = if hovered == Some(Axis::Z) { [1.0, 0.8, 0.0, 1.0] } else { [0.2, 0.2, 1.0, 1.0] };
                vertices.extend(self.scale_box_vertices(position, Vec3::Z * scale, z_color));
            }
        }
        
        vertices
    }

    fn arrow_vertices(&self, origin: Vec3, direction: Vec3, color: [f32; 4]) -> Vec<GizmoVertex> {
        let end = origin + direction;
        vec![
            GizmoVertex { position: origin.to_array(), color },
            GizmoVertex { position: end.to_array(), color },
        ]
    }

    fn circle_vertices(&self, center: Vec3, axis: Vec3, radius: f32, color: [f32; 4]) -> Vec<GizmoVertex> {
        let segments = 32;
        let mut vertices = Vec::new();
        
        // Create orthonormal basis
        let tangent = if axis.dot(Vec3::Y).abs() < 0.9 {
            axis.cross(Vec3::Y).normalize()
        } else {
            axis.cross(Vec3::X).normalize()
        };
        let bitangent = axis.cross(tangent);
        
        for i in 0..segments {
            let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
            
            let p1 = center + tangent * angle1.cos() * radius + bitangent * angle1.sin() * radius;
            let p2 = center + tangent * angle2.cos() * radius + bitangent * angle2.sin() * radius;
            
            vertices.push(GizmoVertex { position: p1.to_array(), color });
            vertices.push(GizmoVertex { position: p2.to_array(), color });
        }
        
        vertices
    }

    fn scale_box_vertices(&self, origin: Vec3, direction: Vec3, color: [f32; 4]) -> Vec<GizmoVertex> {
        let end = origin + direction;
        let box_size = direction.length() * 0.1;
        
        // Line from origin to end
        vec![
            GizmoVertex { position: origin.to_array(), color },
            GizmoVertex { position: end.to_array(), color },
        ]
    }

    /// Hit test gizmo axis
    pub fn hit_test_gizmo(&self, screen_pos: Vec2, gizmo_pos: Vec3, gizmo_type: GizmoType) -> Option<Axis> {
        let (ray_origin, ray_dir) = self.camera.screen_to_ray(screen_pos, self.size);
        let scale = self.camera.distance * 0.1;
        let threshold = scale * 0.15;
        
        // Test each axis
        let axes = [
            (Axis::X, gizmo_pos + Vec3::X * scale * 0.5, Vec3::X),
            (Axis::Y, gizmo_pos + Vec3::Y * scale * 0.5, Vec3::Y),
            (Axis::Z, gizmo_pos + Vec3::Z * scale * 0.5, Vec3::Z),
        ];
        
        let mut closest: Option<(Axis, f32)> = None;
        
        for (axis, center, _dir) in axes {
            // Simple distance to axis line
            let to_center = center - ray_origin;
            let t = to_center.dot(ray_dir);
            let closest_point = ray_origin + ray_dir * t;
            let dist = (closest_point - center).length();
            
            if dist < threshold {
                if closest.map_or(true, |(_, d)| dist < d) {
                    closest = Some((axis, dist));
                }
            }
        }
        
        closest.map(|(axis, _)| axis)
    }
}

impl Default for EditorViewport {
    fn default() -> Self {
        Self::new(1280.0, 720.0)
    }
}

/// Render modes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderMode {
    Lit,
    Unlit,
    Wireframe,
    Normals,
    UVs,
}

/// Shading modes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShadingMode {
    Solid,
    Material,
    Textured,
}

/// Gizmo types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GizmoType {
    Translate,
    Rotate,
    Scale,
}

/// Grid vertex
#[derive(Clone, Copy, Debug)]
pub struct GridVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

/// Gizmo vertex
#[derive(Clone, Copy, Debug)]
pub struct GizmoVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}
