//! PBR Material system with glass, metal, and other surfaces

use glam::Vec3;

/// Material types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MaterialType {
    Standard,
    Metal,
    Glass,
    Emissive,
    Subsurface,
}

/// PBR Material definition
#[derive(Clone, Debug)]
pub struct Material {
    pub material_type: MaterialType,
    pub albedo: Vec3,
    pub metallic: f32,
    pub roughness: f32,
    pub reflectance: f32,
    pub emission: Vec3,
    pub emission_strength: f32,
    pub alpha: f32,
    pub ior: f32, // Index of refraction for glass
    pub transmission: f32,
    pub clear_coat: f32,
    pub clear_coat_roughness: f32,
    pub anisotropy: f32,
    pub subsurface_color: Vec3,
    pub subsurface_radius: f32,
}

impl Material {
    /// Create standard material
    pub fn standard(albedo: Vec3, roughness: f32) -> Self {
        Self {
            material_type: MaterialType::Standard,
            albedo,
            metallic: 0.0,
            roughness,
            reflectance: 0.5,
            emission: Vec3::ZERO,
            emission_strength: 0.0,
            alpha: 1.0,
            ior: 1.5,
            transmission: 0.0,
            clear_coat: 0.0,
            clear_coat_roughness: 0.0,
            anisotropy: 0.0,
            subsurface_color: Vec3::ZERO,
            subsurface_radius: 0.0,
        }
    }

    /// Create metallic material
    pub fn metal(albedo: Vec3, roughness: f32) -> Self {
        Self {
            material_type: MaterialType::Metal,
            albedo,
            metallic: 1.0,
            roughness,
            reflectance: 0.9,
            emission: Vec3::ZERO,
            emission_strength: 0.0,
            alpha: 1.0,
            ior: 2.5,
            transmission: 0.0,
            clear_coat: 0.0,
            clear_coat_roughness: 0.0,
            anisotropy: 0.0,
            subsurface_color: Vec3::ZERO,
            subsurface_radius: 0.0,
        }
    }

    /// Create glass material
    pub fn glass(tint: Vec3, roughness: f32, ior: f32) -> Self {
        Self {
            material_type: MaterialType::Glass,
            albedo: tint,
            metallic: 0.0,
            roughness,
            reflectance: 0.5,
            emission: Vec3::ZERO,
            emission_strength: 0.0,
            alpha: 0.1,
            ior,
            transmission: 0.95,
            clear_coat: 0.0,
            clear_coat_roughness: 0.0,
            anisotropy: 0.0,
            subsurface_color: Vec3::ZERO,
            subsurface_radius: 0.0,
        }
    }

    /// Create emissive material
    pub fn emissive(color: Vec3, strength: f32) -> Self {
        Self {
            material_type: MaterialType::Emissive,
            albedo: color,
            metallic: 0.0,
            roughness: 1.0,
            reflectance: 0.0,
            emission: color,
            emission_strength: strength,
            alpha: 1.0,
            ior: 1.0,
            transmission: 0.0,
            clear_coat: 0.0,
            clear_coat_roughness: 0.0,
            anisotropy: 0.0,
            subsurface_color: Vec3::ZERO,
            subsurface_radius: 0.0,
        }
    }

    // === Preset materials ===

    pub fn concrete() -> Self {
        Self::standard(Vec3::new(0.5, 0.48, 0.45), 0.9)
    }

    pub fn sandstone() -> Self {
        Self::standard(Vec3::new(0.76, 0.7, 0.5), 0.85)
    }

    pub fn wood() -> Self {
        Self::standard(Vec3::new(0.55, 0.35, 0.2), 0.7)
    }

    pub fn steel() -> Self {
        Self::metal(Vec3::new(0.7, 0.7, 0.72), 0.3)
    }

    pub fn chrome() -> Self {
        Self::metal(Vec3::new(0.95, 0.95, 0.97), 0.1)
    }

    pub fn gold() -> Self {
        Self::metal(Vec3::new(1.0, 0.85, 0.55), 0.2)
    }

    pub fn clear_glass() -> Self {
        Self::glass(Vec3::new(0.95, 0.98, 1.0), 0.0, 1.52)
    }

    pub fn tinted_glass() -> Self {
        Self::glass(Vec3::new(0.3, 0.5, 0.6), 0.0, 1.52)
    }

    pub fn plastic_red() -> Self {
        Self::standard(Vec3::new(0.8, 0.1, 0.1), 0.4)
    }

    pub fn rubber() -> Self {
        Self::standard(Vec3::new(0.1, 0.1, 0.1), 0.95)
    }

    pub fn fabric() -> Self {
        Self::standard(Vec3::new(0.4, 0.35, 0.3), 0.98)
    }

    pub fn skin() -> Self {
        let mut mat = Self::standard(Vec3::new(0.85, 0.7, 0.6), 0.5);
        mat.material_type = MaterialType::Subsurface;
        mat.subsurface_color = Vec3::new(0.8, 0.2, 0.1);
        mat.subsurface_radius = 0.5;
        mat
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::standard(Vec3::new(0.8, 0.8, 0.8), 0.5)
    }
}

/// Material uniform for GPU
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    pub albedo: [f32; 4],           // xyz = color, w = alpha
    pub metallic_roughness: [f32; 4], // x = metallic, y = roughness, z = reflectance, w = ior
    pub emission: [f32; 4],         // xyz = emission, w = strength
    pub params: [f32; 4],           // x = transmission, y = clear_coat, z = anisotropy, w = material_type
}

impl MaterialUniform {
    pub fn from_material(mat: &Material) -> Self {
        Self {
            albedo: [mat.albedo.x, mat.albedo.y, mat.albedo.z, mat.alpha],
            metallic_roughness: [mat.metallic, mat.roughness, mat.reflectance, mat.ior],
            emission: [mat.emission.x, mat.emission.y, mat.emission.z, mat.emission_strength],
            params: [
                mat.transmission,
                mat.clear_coat,
                mat.anisotropy,
                mat.material_type as u32 as f32,
            ],
        }
    }
}
