//! FPS Camera with mouse look and WASD movement

use glam::{Mat4, Vec3, Vec2};

pub struct FPSCamera {
    pub position: Vec3,
    pub yaw: f32,   // Horizontal rotation (in radians)
    pub pitch: f32, // Vertical rotation (in radians)
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
    pub speed: f32,
    pub sensitivity: f32,
}

impl FPSCamera {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            yaw: -std::f32::consts::FRAC_PI_2, // Looking towards -Z
            pitch: 0.0,
            fov: 80.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near: 0.1,
            far: 1000.0,
            speed: 8.0,
            sensitivity: 0.003,
        }
    }

    pub fn get_front(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        ).normalize()
    }

    pub fn get_right(&self) -> Vec3 {
        self.get_front().cross(Vec3::Y).normalize()
    }

    pub fn get_up(&self) -> Vec3 {
        self.get_right().cross(self.get_front()).normalize()
    }

    pub fn process_mouse(&mut self, delta: Vec2) {
        self.yaw += delta.x * self.sensitivity;
        self.pitch -= delta.y * self.sensitivity;
        
        // Clamp pitch to prevent camera flip
        let max_pitch = 89.0_f32.to_radians();
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);
    }

    pub fn process_keyboard(&mut self, direction: MoveDirection, delta_time: f32) {
        let velocity = self.speed * delta_time;
        let front = self.get_front();
        let right = self.get_right();
        
        // For movement, we want horizontal-only front vector (no flying)
        let front_horizontal = Vec3::new(front.x, 0.0, front.z).normalize();
        
        match direction {
            MoveDirection::Forward => self.position += front_horizontal * velocity,
            MoveDirection::Backward => self.position -= front_horizontal * velocity,
            MoveDirection::Left => self.position -= right * velocity,
            MoveDirection::Right => self.position += right * velocity,
            MoveDirection::Up => self.position.y += velocity,
            MoveDirection::Down => self.position.y -= velocity,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        let front = self.get_front();
        let center = self.position + front;
        Mat4::look_at_rh(self.position, center, Vec3::Y)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect_ratio, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MoveDirection {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}
