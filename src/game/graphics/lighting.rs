//! Advanced pattern-based lighting system with PBR support

use glam::{Vec3, Vec4, Mat4};
use half::f16;

/// Light type enum
#[derive(Clone, Copy, Debug)]
pub enum LightType {
    Directional,
    Point,
    Spot,
    Area,
}

/// Dynamic light source
#[derive(Clone, Debug)]
pub struct Light {
    pub light_type: LightType,
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
    pub radius: f32,
    pub inner_cone: f32,
    pub outer_cone: f32,
    pub casts_shadow: bool,
}

impl Light {
    pub fn directional(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional,
            position: Vec3::ZERO,
            direction: direction.normalize(),
            color,
            intensity,
            radius: f32::INFINITY,
            inner_cone: 0.0,
            outer_cone: 0.0,
            casts_shadow: true,
        }
    }

    pub fn point(position: Vec3, color: Vec3, intensity: f32, radius: f32) -> Self {
        Self {
            light_type: LightType::Point,
            position,
            direction: Vec3::ZERO,
            color,
            intensity,
            radius,
            inner_cone: 0.0,
            outer_cone: 0.0,
            casts_shadow: true,
        }
    }

    pub fn spot(position: Vec3, direction: Vec3, color: Vec3, intensity: f32, inner_cone: f32, outer_cone: f32) -> Self {
        Self {
            light_type: LightType::Spot,
            position,
            direction: direction.normalize(),
            color,
            intensity,
            radius: 50.0,
            inner_cone,
            outer_cone,
            casts_shadow: true,
        }
    }
}

/// Spherical Harmonics for ambient lighting (L2 - 9 coefficients per channel)
#[derive(Clone, Debug)]
pub struct SphericalHarmonics {
    pub coefficients: [[f32; 9]; 3], // RGB, 9 coefficients each
}

impl SphericalHarmonics {
    pub fn new() -> Self {
        Self {
            coefficients: [[0.0; 9]; 3],
        }
    }

    /// Sample SH in a direction
    pub fn sample(&self, direction: Vec3) -> Vec3 {
        let d = direction.normalize();
        
        // SH basis functions (L0 and L1)
        let y00 = 0.282095;
        let y1n1 = 0.488603 * d.y;
        let y10 = 0.488603 * d.z;
        let y11 = 0.488603 * d.x;
        let y2n2 = 1.092548 * d.x * d.y;
        let y2n1 = 1.092548 * d.y * d.z;
        let y20 = 0.315392 * (3.0 * d.z * d.z - 1.0);
        let y21 = 1.092548 * d.x * d.z;
        let y22 = 0.546274 * (d.x * d.x - d.y * d.y);

        let basis = [y00, y1n1, y10, y11, y2n2, y2n1, y20, y21, y22];

        let mut result = Vec3::ZERO;
        for i in 0..9 {
            result.x += self.coefficients[0][i] * basis[i];
            result.y += self.coefficients[1][i] * basis[i];
            result.z += self.coefficients[2][i] * basis[i];
        }
        result.max(Vec3::ZERO)
    }

    /// Create from sky color
    pub fn from_sky(sky_color: Vec3, ground_color: Vec3, sun_direction: Vec3, sun_color: Vec3) -> Self {
        let mut sh = Self::new();
        
        // L0 (constant)
        let ambient = (sky_color + ground_color) * 0.5;
        sh.coefficients[0][0] = ambient.x * 0.886227;
        sh.coefficients[1][0] = ambient.y * 0.886227;
        sh.coefficients[2][0] = ambient.z * 0.886227;

        // L1 (directional)
        let gradient = (sky_color - ground_color) * 0.5;
        sh.coefficients[0][1] = gradient.x * 1.023327;
        sh.coefficients[1][1] = gradient.y * 1.023327;
        sh.coefficients[2][1] = gradient.z * 1.023327;

        // Sun contribution
        let sun_intensity = sun_color * 0.3;
        sh.coefficients[0][2] = sun_intensity.x * sun_direction.z;
        sh.coefficients[1][2] = sun_intensity.y * sun_direction.z;
        sh.coefficients[2][2] = sun_intensity.z * sun_direction.z;
        sh.coefficients[0][3] = sun_intensity.x * sun_direction.x;
        sh.coefficients[1][3] = sun_intensity.y * sun_direction.x;
        sh.coefficients[2][3] = sun_intensity.z * sun_direction.x;

        sh
    }
}

impl Default for SphericalHarmonics {
    fn default() -> Self {
        Self::new()
    }
}

/// Light pattern from the engine (1000 bytes)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightPattern {
    pub direct_light: f16,
    pub indirect_light: f16,
    pub sh_coefficients: [i8; 256],
    pub materials: [u8; 512],
    pub ambient_occlusion: f16,
    pub reflection: f16,
    pub refraction: f16,
    pub emission: f16,
    pub material_properties: [f16; 110],
}

impl LightPattern {
    pub fn new() -> Self {
        Self {
            direct_light: f16::from_f32(1.0),
            indirect_light: f16::from_f32(0.3),
            sh_coefficients: [64; 256], // Neutral ambient
            materials: [128; 512],
            ambient_occlusion: f16::from_f32(1.0),
            reflection: f16::from_f32(0.0),
            refraction: f16::from_f32(0.0),
            emission: f16::from_f32(0.0),
            material_properties: [f16::ZERO; 110],
        }
    }

    pub fn calculate_lighting(&self, normal: Vec3, view_dir: Vec3, light_dir: Vec3, albedo: Vec3, roughness: f32, metallic: f32) -> Vec3 {
        let direct = self.direct_light.to_f32();
        let indirect = self.indirect_light.to_f32();
        let ao = self.ambient_occlusion.to_f32();

        // PBR lighting calculation
        let n_dot_l = normal.dot(light_dir).max(0.0);
        let n_dot_v = normal.dot(view_dir).max(0.0);
        
        // Half vector for specular
        let h = (light_dir + view_dir).normalize();
        let n_dot_h = normal.dot(h).max(0.0);
        
        // Fresnel (Schlick)
        let f0 = Vec3::splat(0.04).lerp(albedo, metallic);
        let fresnel = f0 + (Vec3::ONE - f0) * (1.0 - n_dot_v).powf(5.0);
        
        // Distribution (GGX)
        let a = roughness * roughness;
        let a2 = a * a;
        let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
        let d = a2 / (std::f32::consts::PI * denom * denom);
        
        // Geometry (Smith)
        let k = (roughness + 1.0).powf(2.0) / 8.0;
        let g1_v = n_dot_v / (n_dot_v * (1.0 - k) + k);
        let g1_l = n_dot_l / (n_dot_l * (1.0 - k) + k);
        let g = g1_v * g1_l;
        
        // Specular BRDF
        let specular = (fresnel * d * g) / (4.0 * n_dot_v * n_dot_l + 0.001);
        
        // Diffuse (Lambert)
        let kd = (Vec3::ONE - fresnel) * (1.0 - metallic);
        let diffuse = kd * albedo / std::f32::consts::PI;
        
        // Combine
        let light_color = Vec3::splat(direct);
        let direct_contrib = (diffuse + specular) * light_color * n_dot_l;
        let ambient_contrib = albedo * indirect * ao;
        
        direct_contrib + ambient_contrib
    }
}

impl Default for LightPattern {
    fn default() -> Self {
        Self::new()
    }
}

/// Advanced lighting system
pub struct AdvancedLighting {
    pub lights: Vec<Light>,
    pub ambient_sh: SphericalHarmonics,
    pub patterns: Vec<LightPattern>,
    pub sun_direction: Vec3,
    pub sun_color: Vec3,
    pub sky_color: Vec3,
    pub ground_color: Vec3,
    pub exposure: f32,
    pub time_of_day: f32,
}

impl AdvancedLighting {
    pub fn new() -> Self {
        let sun_direction = Vec3::new(-0.5, -0.8, -0.3).normalize();
        let sun_color = Vec3::new(1.0, 0.95, 0.85);
        let sky_color = Vec3::new(0.4, 0.6, 0.9);
        let ground_color = Vec3::new(0.3, 0.25, 0.2);

        Self {
            lights: vec![
                Light::directional(sun_direction, sun_color, 2.0),
            ],
            ambient_sh: SphericalHarmonics::from_sky(sky_color, ground_color, -sun_direction, sun_color),
            patterns: Vec::new(),
            sun_direction,
            sun_color,
            sky_color,
            ground_color,
            exposure: 1.0,
            time_of_day: 12.0,
        }
    }

    pub fn add_point_light(&mut self, position: Vec3, color: Vec3, intensity: f32, radius: f32) {
        self.lights.push(Light::point(position, color, intensity, radius));
    }

    pub fn add_spot_light(&mut self, position: Vec3, direction: Vec3, color: Vec3, intensity: f32) {
        self.lights.push(Light::spot(position, direction, color, intensity, 0.9, 0.8));
    }

    pub fn update(&mut self, delta_time: f32) {
        // Animate time of day (optional)
        // self.time_of_day += delta_time * 0.01;
        // self.update_sun_position();
    }

    fn update_sun_position(&mut self) {
        let hour = self.time_of_day % 24.0;
        let angle = (hour - 6.0) / 12.0 * std::f32::consts::PI;
        
        self.sun_direction = Vec3::new(
            -angle.cos() * 0.5,
            -angle.sin().abs().max(0.1),
            -0.3,
        ).normalize();

        // Update sun color based on time
        let sunset_factor = 1.0 - (angle.sin().abs());
        self.sun_color = Vec3::new(
            1.0,
            0.95 - sunset_factor * 0.3,
            0.85 - sunset_factor * 0.5,
        );

        // Update main directional light
        if let Some(light) = self.lights.first_mut() {
            light.direction = self.sun_direction;
            light.color = self.sun_color;
            light.intensity = if angle.sin() > 0.0 { 2.0 } else { 0.1 };
        }

        // Update ambient
        self.ambient_sh = SphericalHarmonics::from_sky(
            self.sky_color,
            self.ground_color,
            -self.sun_direction,
            self.sun_color,
        );
    }

    /// Calculate lighting for a point
    pub fn calculate(&self, position: Vec3, normal: Vec3, view_pos: Vec3, albedo: Vec3, roughness: f32, metallic: f32) -> Vec3 {
        let view_dir = (view_pos - position).normalize();
        let mut total_light = Vec3::ZERO;

        // Ambient from SH
        let ambient = self.ambient_sh.sample(normal) * 0.3;
        total_light += ambient * albedo;

        // Directional and point lights
        for light in &self.lights {
            let light_contribution = match light.light_type {
                LightType::Directional => {
                    let pattern = LightPattern::new();
                    pattern.calculate_lighting(normal, view_dir, -light.direction, albedo, roughness, metallic)
                        * light.color * light.intensity
                }
                LightType::Point => {
                    let light_vec = light.position - position;
                    let distance = light_vec.length();
                    if distance > light.radius {
                        Vec3::ZERO
                    } else {
                        let light_dir = light_vec / distance;
                        let attenuation = 1.0 / (1.0 + distance * 0.1 + distance * distance * 0.01);
                        let pattern = LightPattern::new();
                        pattern.calculate_lighting(normal, view_dir, light_dir, albedo, roughness, metallic)
                            * light.color * light.intensity * attenuation
                    }
                }
                LightType::Spot => {
                    let light_vec = light.position - position;
                    let distance = light_vec.length();
                    let light_dir = light_vec / distance;
                    
                    let theta = light_dir.dot(-light.direction);
                    let epsilon = light.inner_cone - light.outer_cone;
                    let spot_intensity = ((theta - light.outer_cone) / epsilon).clamp(0.0, 1.0);
                    
                    let attenuation = 1.0 / (1.0 + distance * 0.1 + distance * distance * 0.01);
                    let pattern = LightPattern::new();
                    pattern.calculate_lighting(normal, view_dir, light_dir, albedo, roughness, metallic)
                        * light.color * light.intensity * attenuation * spot_intensity
                }
                LightType::Area => Vec3::ZERO, // TODO: Area lights
            };
            total_light += light_contribution;
        }

        // Apply exposure
        total_light * self.exposure
    }
}

impl Default for AdvancedLighting {
    fn default() -> Self {
        Self::new()
    }
}

/// Uniform buffer for GPU lighting
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightingUniform {
    pub sun_direction: [f32; 4],
    pub sun_color: [f32; 4],
    pub ambient_color: [f32; 4],
    pub view_position: [f32; 4],
    pub exposure: f32,
    pub time: f32,
    pub shadow_bias: f32,
    pub _padding: f32,
}

impl LightingUniform {
    pub fn from_system(lighting: &AdvancedLighting, view_pos: Vec3, time: f32) -> Self {
        Self {
            sun_direction: [lighting.sun_direction.x, lighting.sun_direction.y, lighting.sun_direction.z, 0.0],
            sun_color: [lighting.sun_color.x, lighting.sun_color.y, lighting.sun_color.z, 1.0],
            ambient_color: [lighting.sky_color.x * 0.2, lighting.sky_color.y * 0.2, lighting.sky_color.z * 0.2, 1.0],
            view_position: [view_pos.x, view_pos.y, view_pos.z, 1.0],
            exposure: lighting.exposure,
            time,
            shadow_bias: 0.005,
            _padding: 0.0,
        }
    }
}

/// Point light data for GPU
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightUniform {
    pub position: [f32; 4],
    pub color: [f32; 4],
    pub params: [f32; 4], // intensity, radius, falloff, enabled
}
