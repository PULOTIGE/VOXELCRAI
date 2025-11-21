// PBR (Physically Based Rendering) material system
use glam::Vec3;
use bytemuck::{Pod, Zeroable};

/// PBR Material properties
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct PBRMaterial {
    pub albedo: [f32; 3],       // Base color (RGB)
    pub metallic: f32,          // Metallic factor (0-1)
    pub roughness: f32,         // Roughness factor (0-1)
    pub ao: f32,                // Ambient occlusion (0-1)
    pub emission: [f32; 3],     // Emission color (RGB)
    pub _padding: f32,          // Padding for alignment
}

impl PBRMaterial {
    pub fn new(albedo: Vec3, metallic: f32, roughness: f32) -> Self {
        Self {
            albedo: albedo.into(),
            metallic: metallic.clamp(0.0, 1.0),
            roughness: roughness.clamp(0.0, 1.0),
            ao: 1.0,
            emission: [0.0; 3],
            _padding: 0.0,
        }
    }

    /// Create a standard material preset
    pub fn gold() -> Self {
        Self::new(Vec3::new(1.0, 0.765557, 0.336057), 1.0, 0.2)
    }

    pub fn copper() -> Self {
        Self::new(Vec3::new(0.955, 0.637, 0.538), 1.0, 0.3)
    }

    pub fn plastic() -> Self {
        Self::new(Vec3::new(0.8, 0.8, 0.8), 0.0, 0.5)
    }

    pub fn rubber() -> Self {
        Self::new(Vec3::new(0.2, 0.2, 0.2), 0.0, 0.9)
    }
}

impl Default for PBRMaterial {
    fn default() -> Self {
        Self::new(Vec3::new(0.8, 0.8, 0.8), 0.0, 0.5)
    }
}

// Helper functions for conversion
impl PBRMaterial {
    pub fn albedo_vec3(&self) -> Vec3 {
        Vec3::new(self.albedo[0], self.albedo[1], self.albedo[2])
    }

    pub fn emission_vec3(&self) -> Vec3 {
        Vec3::new(self.emission[0], self.emission[1], self.emission[2])
    }
}

/// Simple lighting model for PBR
pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
        }
    }

    pub fn directional(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            position: -direction.normalize() * 1000.0, // Far away for directional
            color,
            intensity,
        }
    }
}

impl Default for Light {
    fn default() -> Self {
        Self::directional(Vec3::new(0.5, 1.0, 0.3), Vec3::ONE, 1.0)
    }
}
