//! Орбитальная камера для просмотра организма

use glam::{Mat4, Vec3};

/// Орбитальная камера
pub struct OrbitCamera {
    /// Центр, вокруг которого вращается камера
    pub target: Vec3,
    
    /// Расстояние до цели
    pub distance: f32,
    
    /// Угол по горизонтали (radians)
    pub yaw: f32,
    
    /// Угол по вертикали (radians)
    pub pitch: f32,
    
    /// Поле зрения (radians)
    pub fov: f32,
    
    /// Соотношение сторон
    pub aspect: f32,
    
    /// Ближняя плоскость отсечения
    pub near: f32,
    
    /// Дальняя плоскость отсечения
    pub far: f32,
    
    /// Скорость вращения
    pub rotation_speed: f32,
    
    /// Скорость зума
    pub zoom_speed: f32,
}

impl OrbitCamera {
    pub fn new(aspect: f32) -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 50.0,
            yaw: 0.0,
            pitch: 0.3,
            fov: std::f32::consts::FRAC_PI_4,
            aspect,
            near: 0.1,
            far: 1000.0,
            rotation_speed: 0.01,
            zoom_speed: 1.0,
        }
    }
    
    /// Позиция камеры в мировых координатах
    pub fn position(&self) -> Vec3 {
        let x = self.distance * self.pitch.cos() * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        let z = self.distance * self.pitch.cos() * self.yaw.cos();
        
        self.target + Vec3::new(x, y, z)
    }
    
    /// Матрица вида
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position(), self.target, Vec3::Y)
    }
    
    /// Матрица проекции
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }
    
    /// Комбинированная матрица view * projection
    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
    
    /// Повернуть камеру
    pub fn rotate(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw += delta_x * self.rotation_speed;
        self.pitch = (self.pitch + delta_y * self.rotation_speed)
            .clamp(-std::f32::consts::FRAC_PI_2 + 0.1, std::f32::consts::FRAC_PI_2 - 0.1);
    }
    
    /// Приблизить/отдалить
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance - delta * self.zoom_speed).clamp(5.0, 200.0);
    }
    
    /// Обновить соотношение сторон
    pub fn set_aspect(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height.max(1) as f32;
    }
    
    /// Автоматическое вращение (для демо)
    pub fn auto_rotate(&mut self, dt: f32) {
        self.yaw += dt * 0.2;
    }
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self::new(16.0 / 9.0)
    }
}
