// Camera system for 3D rendering
use glam::{Mat4, Vec3};

/// Camera component with position, rotation, and projection settings
#[derive(Clone, Copy)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3) -> Self {
        Self {
            position,
            target,
            up: Vec3::Y,
            fov: 60.0_f32.to_radians(),
            aspect: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// Get view matrix (world to camera space)
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Get projection matrix (camera to clip space)
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    /// Get view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Update camera aspect ratio (for window resize)
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    /// Move camera forward/backward
    pub fn translate(&mut self, delta: Vec3) {
        self.position += delta;
        self.target += delta;
    }

    /// Rotate camera around target
    pub fn orbit(&mut self, yaw: f32, pitch: f32) {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward).normalize();

        // Apply rotation
        let rotation = Mat4::from_axis_angle(up, yaw) * Mat4::from_axis_angle(right, pitch);
        let distance = (self.target - self.position).length();
        let new_forward = rotation.transform_vector3(forward);
        self.position = self.target - new_forward * distance;
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Vec3::new(0.0, 5.0, 10.0), Vec3::ZERO)
    }
}
