//! Shadow mapping system with PCF soft shadows

use glam::{Vec3, Mat4};

/// Shadow map configuration
pub struct ShadowConfig {
    pub resolution: u32,
    pub cascade_count: u32,
    pub cascade_splits: Vec<f32>,
    pub bias: f32,
    pub normal_bias: f32,
    pub pcf_radius: f32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            resolution: 2048,
            cascade_count: 4,
            cascade_splits: vec![0.1, 0.3, 0.6, 1.0],
            bias: 0.005,
            normal_bias: 0.02,
            pcf_radius: 2.0,
        }
    }
}

/// Shadow cascade for CSM
#[derive(Clone, Debug)]
pub struct ShadowCascade {
    pub view_proj: Mat4,
    pub split_depth: f32,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

/// Shadow mapping system
pub struct ShadowSystem {
    pub config: ShadowConfig,
    pub cascades: Vec<ShadowCascade>,
    pub light_direction: Vec3,
}

impl ShadowSystem {
    pub fn new(config: ShadowConfig) -> Self {
        let cascade_count = config.cascade_count as usize;
        Self {
            config,
            cascades: vec![ShadowCascade {
                view_proj: Mat4::IDENTITY,
                split_depth: 0.0,
                bounds_min: Vec3::ZERO,
                bounds_max: Vec3::ZERO,
            }; cascade_count],
            light_direction: Vec3::new(-0.5, -0.8, -0.3).normalize(),
        }
    }

    /// Update shadow cascades based on camera frustum
    pub fn update(&mut self, camera_pos: Vec3, camera_dir: Vec3, camera_fov: f32, aspect: f32, near: f32, far: f32) {
        let light_dir = self.light_direction.normalize();
        
        // Calculate light space basis
        let up = if light_dir.y.abs() > 0.99 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        let light_right = light_dir.cross(up).normalize();
        let light_up = light_right.cross(light_dir).normalize();
        
        // Calculate cascade splits
        let mut prev_split = near;
        
        for (i, cascade) in self.cascades.iter_mut().enumerate() {
            let split_ratio = self.config.cascade_splits[i.min(self.config.cascade_splits.len() - 1)];
            let split_depth = near + (far - near) * split_ratio;
            
            // Calculate frustum corners for this cascade
            let frustum_corners = calculate_frustum_corners(
                camera_pos,
                camera_dir,
                camera_fov,
                aspect,
                prev_split,
                split_depth,
            );
            
            // Calculate bounding sphere of frustum
            let center: Vec3 = frustum_corners.iter().sum::<Vec3>() / 8.0;
            let radius = frustum_corners.iter()
                .map(|c| (*c - center).length())
                .fold(0.0_f32, f32::max);
            
            // Create orthographic projection for this cascade
            let light_view = Mat4::look_at_rh(
                center - light_dir * radius * 2.0,
                center,
                light_up,
            );
            
            let light_proj = Mat4::orthographic_rh(
                -radius, radius,
                -radius, radius,
                0.1, radius * 4.0,
            );
            
            cascade.view_proj = light_proj * light_view;
            cascade.split_depth = split_depth;
            cascade.bounds_min = center - Vec3::splat(radius);
            cascade.bounds_max = center + Vec3::splat(radius);
            
            prev_split = split_depth;
        }
    }

    /// Get cascade index for a given depth
    pub fn get_cascade_index(&self, depth: f32) -> usize {
        for (i, cascade) in self.cascades.iter().enumerate() {
            if depth < cascade.split_depth {
                return i;
            }
        }
        self.cascades.len() - 1
    }
}

fn calculate_frustum_corners(
    pos: Vec3,
    dir: Vec3,
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,
) -> [Vec3; 8] {
    let right = dir.cross(Vec3::Y).normalize();
    let up = right.cross(dir).normalize();
    
    let tan_half_fov = (fov / 2.0).tan();
    
    let near_height = near * tan_half_fov;
    let near_width = near_height * aspect;
    let far_height = far * tan_half_fov;
    let far_width = far_height * aspect;
    
    let near_center = pos + dir * near;
    let far_center = pos + dir * far;
    
    [
        // Near plane
        near_center - right * near_width - up * near_height,
        near_center + right * near_width - up * near_height,
        near_center + right * near_width + up * near_height,
        near_center - right * near_width + up * near_height,
        // Far plane
        far_center - right * far_width - up * far_height,
        far_center + right * far_width - up * far_height,
        far_center + right * far_width + up * far_height,
        far_center - right * far_width + up * far_height,
    ]
}

impl Default for ShadowSystem {
    fn default() -> Self {
        Self::new(ShadowConfig::default())
    }
}

/// Shadow uniform for GPU
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowUniform {
    pub light_space_matrix: [[f32; 4]; 4],
    pub cascade_splits: [f32; 4],
    pub shadow_params: [f32; 4], // bias, normal_bias, pcf_radius, cascade_count
}

impl ShadowUniform {
    pub fn from_system(system: &ShadowSystem, cascade_index: usize) -> Self {
        let matrix = if cascade_index < system.cascades.len() {
            system.cascades[cascade_index].view_proj
        } else {
            Mat4::IDENTITY
        };

        let mut splits = [0.0; 4];
        for (i, cascade) in system.cascades.iter().enumerate().take(4) {
            splits[i] = cascade.split_depth;
        }

        Self {
            light_space_matrix: matrix.to_cols_array_2d(),
            cascade_splits: splits,
            shadow_params: [
                system.config.bias,
                system.config.normal_bias,
                system.config.pcf_radius,
                system.config.cascade_count as f32,
            ],
        }
    }
}
