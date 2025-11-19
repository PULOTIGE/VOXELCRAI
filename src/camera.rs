use glam::{Mat4, Quat, Vec3};
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone)]
pub struct Projection {
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub aspect: f32,
}

impl Projection {
    pub fn new(fovy: f32, znear: f32, zfar: f32, aspect: f32) -> Self {
        Self {
            fovy,
            znear,
            zfar,
            aspect,
        }
    }

    pub fn matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fovy, self.aspect.max(0.1), self.znear, self.zfar)
    }
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub projection: Projection,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, aspect: f32) -> Self {
        let forward = (target - position).normalize_or_zero();
        let yaw = forward.x.atan2(-forward.z);
        let pitch = forward.y.asin();

        Self {
            position,
            yaw,
            pitch,
            projection: Projection::new(45_f32.to_radians(), 0.1, 500.0, aspect),
        }
    }

    pub fn view_projection(&self) -> Mat4 {
        let view = Mat4::look_to_rh(self.position, self.forward_vector(), Vec3::Y);
        self.projection.matrix() * view
    }

    pub fn forward_vector(&self) -> Vec3 {
        let yaw_rot = Quat::from_rotation_y(self.yaw);
        let pitch_rot = Quat::from_rotation_x(self.pitch);
        (yaw_rot * pitch_rot) * -Vec3::Z
    }

    pub fn set_aspect(&mut self, width: u32, height: u32) {
        if height > 0 {
            self.projection.aspect = width as f32 / height as f32;
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.view_proj = camera.view_projection().to_cols_array_2d();
    }
}

pub struct CameraController {
    speed: f32,
    pub sensitivity: f32,
    scroll_speed: f32,
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    rotating: bool,
    rotate_delta: (f32, f32),
    last_cursor_pos: Option<(f32, f32)>,
    scroll: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            scroll_speed: 10.0,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            rotating: false,
            rotate_delta: (0.0, 0.0),
            scroll: 0.0,
            last_cursor_pos: None,
        }
    }

    pub fn process_window_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => self.process_key_event(event),
            WindowEvent::MouseWheel { delta, .. } => {
                self.scroll = match delta {
                    MouseScrollDelta::LineDelta(_x, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
                };
                true
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Right {
                    self.rotating = *state == ElementState::Pressed;
                    if !self.rotating {
                        self.last_cursor_pos = None;
                    }
                    true
                } else {
                    false
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.rotating {
                    let current = (position.x as f32, position.y as f32);
                    if let Some(prev) = self.last_cursor_pos {
                        self.rotate_delta.0 += current.0 - prev.0;
                        self.rotate_delta.1 += current.1 - prev.1;
                    }
                    self.last_cursor_pos = Some(current);
                    true
                } else {
                    self.last_cursor_pos = Some((position.x as f32, position.y as f32));
                    false
                }
            }
            _ => false,
        }
    }

    fn process_key_event(&mut self, event: &KeyEvent) -> bool {
        let pressed = event.state == ElementState::Pressed;
        if let PhysicalKey::Code(code) = event.physical_key {
            match code {
                KeyCode::KeyW => {
                    self.move_forward = pressed;
                    true
                }
                KeyCode::KeyS => {
                    self.move_backward = pressed;
                    true
                }
                KeyCode::KeyA => {
                    self.move_left = pressed;
                    true
                }
                KeyCode::KeyD => {
                    self.move_right = pressed;
                    true
                }
                KeyCode::Space => {
                    self.move_up = pressed;
                    true
                }
                KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                    self.move_down = pressed;
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: f32) {
        let mut forward = camera.forward_vector();
        if forward.length_squared() == 0.0 {
            forward = Vec3::NEG_Z;
        }
        let right = forward.cross(Vec3::Y).normalize_or_zero();
        let mut velocity = Vec3::ZERO;

        if self.move_forward {
            velocity += forward;
        }
        if self.move_backward {
            velocity -= forward;
        }
        if self.move_right {
            velocity += right;
        }
        if self.move_left {
            velocity -= right;
        }
        if self.move_up {
            velocity.y += 1.0;
        }
        if self.move_down {
            velocity.y -= 1.0;
        }

        if velocity.length_squared() > 0.0 {
            camera.position += velocity.normalize() * self.speed * dt;
        }

        if self.scroll.abs() > f32::EPSILON {
            camera.position += forward * self.scroll * self.scroll_speed * dt;
            self.scroll = 0.0;
        }

        if self.rotating {
            camera.yaw -= self.rotate_delta.0 * self.sensitivity * dt * 0.001;
            camera.pitch -= self.rotate_delta.1 * self.sensitivity * dt * 0.001;
            camera.pitch = camera.pitch.clamp(-1.3, 1.3);
            self.rotate_delta = (0.0, 0.0);
        }
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }
}
