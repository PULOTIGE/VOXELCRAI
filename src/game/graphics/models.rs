//! Detailed 3D models for characters and weapons

use glam::{Vec3, Vec2, Mat4, Quat};
use std::f32::consts::PI;

/// Vertex with all attributes for PBR rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],  // xyz = tangent, w = handedness
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl ModelVertex {
    pub fn new(pos: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        // Calculate tangent from normal
        let n = Vec3::from(normal);
        let t = if n.y.abs() < 0.9 {
            n.cross(Vec3::Y).normalize()
        } else {
            n.cross(Vec3::X).normalize()
        };
        
        Self {
            position: pos,
            normal,
            tangent: [t.x, t.y, t.z, 1.0],
            uv,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Team types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TeamType {
    Terrorist,
    CounterTerrorist,
}

/// Character model with skeleton
#[derive(Clone, Debug)]
pub struct CharacterModel {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub team: TeamType,
    pub height: f32,
    pub width: f32,
}

impl CharacterModel {
    /// Create detailed terrorist model
    pub fn terrorist() -> Self {
        Self::create_detailed_character(TeamType::Terrorist)
    }

    /// Create detailed counter-terrorist model  
    pub fn counter_terrorist() -> Self {
        Self::create_detailed_character(TeamType::CounterTerrorist)
    }

    fn create_detailed_character(team: TeamType) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let (primary_color, secondary_color, accent_color) = match team {
            TeamType::Terrorist => (
                [0.55, 0.35, 0.15, 1.0], // Brown/tan
                [0.3, 0.25, 0.2, 1.0],   // Dark brown
                [0.8, 0.6, 0.3, 1.0],    // Gold accent
            ),
            TeamType::CounterTerrorist => (
                [0.15, 0.25, 0.4, 1.0],  // Navy blue
                [0.1, 0.1, 0.15, 1.0],   // Dark blue
                [0.7, 0.7, 0.8, 1.0],    // Silver accent
            ),
        };

        let height = 1.8;
        let base_y = 0.0;

        // === HEAD ===
        let head_y = base_y + height - 0.15;
        let head_radius = 0.12;
        let (head_v, head_i) = create_sphere(
            Vec3::new(0.0, head_y, 0.0),
            head_radius,
            16, 12,
            [0.85, 0.75, 0.65, 1.0], // Skin tone
        );
        add_mesh(&mut vertices, &mut indices, head_v, head_i);

        // === HELMET/BERET ===
        let helmet_y = head_y + 0.08;
        let (helmet_v, helmet_i) = create_hemisphere(
            Vec3::new(0.0, helmet_y, 0.0),
            head_radius + 0.02,
            12, 8,
            secondary_color,
        );
        add_mesh(&mut vertices, &mut indices, helmet_v, helmet_i);

        // === NECK ===
        let neck_y = head_y - 0.1;
        let (neck_v, neck_i) = create_cylinder(
            Vec3::new(0.0, neck_y, 0.0),
            0.05, 0.1,
            8,
            [0.8, 0.7, 0.6, 1.0],
        );
        add_mesh(&mut vertices, &mut indices, neck_v, neck_i);

        // === TORSO (Upper body) ===
        let torso_y = base_y + height * 0.65;
        let (torso_v, torso_i) = create_box_rounded(
            Vec3::new(0.0, torso_y, 0.0),
            Vec3::new(0.35, 0.4, 0.2),
            primary_color,
        );
        add_mesh(&mut vertices, &mut indices, torso_v, torso_i);

        // === VEST/ARMOR ===
        let vest_y = torso_y + 0.05;
        let (vest_v, vest_i) = create_box_rounded(
            Vec3::new(0.0, vest_y, 0.02),
            Vec3::new(0.32, 0.3, 0.12),
            secondary_color,
        );
        add_mesh(&mut vertices, &mut indices, vest_v, vest_i);

        // === BELT ===
        let belt_y = base_y + height * 0.45;
        let (belt_v, belt_i) = create_box_rounded(
            Vec3::new(0.0, belt_y, 0.0),
            Vec3::new(0.33, 0.06, 0.18),
            [0.15, 0.12, 0.1, 1.0],
        );
        add_mesh(&mut vertices, &mut indices, belt_v, belt_i);

        // === LEGS ===
        let leg_y = base_y + height * 0.25;
        let leg_offset = 0.1;
        
        // Left leg
        let (leg_l_v, leg_l_i) = create_cylinder(
            Vec3::new(-leg_offset, leg_y, 0.0),
            0.08, 0.45,
            8,
            primary_color,
        );
        add_mesh(&mut vertices, &mut indices, leg_l_v, leg_l_i);
        
        // Right leg
        let (leg_r_v, leg_r_i) = create_cylinder(
            Vec3::new(leg_offset, leg_y, 0.0),
            0.08, 0.45,
            8,
            primary_color,
        );
        add_mesh(&mut vertices, &mut indices, leg_r_v, leg_r_i);

        // === BOOTS ===
        let boot_y = base_y + 0.06;
        let (boot_l_v, boot_l_i) = create_box_rounded(
            Vec3::new(-leg_offset, boot_y, 0.02),
            Vec3::new(0.1, 0.12, 0.18),
            [0.1, 0.1, 0.1, 1.0],
        );
        add_mesh(&mut vertices, &mut indices, boot_l_v, boot_l_i);
        
        let (boot_r_v, boot_r_i) = create_box_rounded(
            Vec3::new(leg_offset, boot_y, 0.02),
            Vec3::new(0.1, 0.12, 0.18),
            [0.1, 0.1, 0.1, 1.0],
        );
        add_mesh(&mut vertices, &mut indices, boot_r_v, boot_r_i);

        // === ARMS ===
        let arm_y = base_y + height * 0.7;
        let arm_offset = 0.22;
        
        // Left upper arm
        let (arm_lu_v, arm_lu_i) = create_cylinder(
            Vec3::new(-arm_offset, arm_y, 0.0),
            0.055, 0.2,
            8,
            primary_color,
        );
        add_mesh(&mut vertices, &mut indices, arm_lu_v, arm_lu_i);
        
        // Left lower arm
        let (arm_ll_v, arm_ll_i) = create_cylinder(
            Vec3::new(-arm_offset - 0.05, arm_y - 0.25, 0.1),
            0.045, 0.2,
            8,
            primary_color,
        );
        add_mesh(&mut vertices, &mut indices, arm_ll_v, arm_ll_i);

        // Right upper arm
        let (arm_ru_v, arm_ru_i) = create_cylinder(
            Vec3::new(arm_offset, arm_y, 0.0),
            0.055, 0.2,
            8,
            primary_color,
        );
        add_mesh(&mut vertices, &mut indices, arm_ru_v, arm_ru_i);
        
        // Right lower arm (holding weapon)
        let (arm_rl_v, arm_rl_i) = create_cylinder(
            Vec3::new(arm_offset + 0.05, arm_y - 0.25, 0.15),
            0.045, 0.2,
            8,
            primary_color,
        );
        add_mesh(&mut vertices, &mut indices, arm_rl_v, arm_rl_i);

        // === HANDS ===
        let (hand_l_v, hand_l_i) = create_sphere(
            Vec3::new(-arm_offset - 0.05, arm_y - 0.4, 0.12),
            0.04,
            8, 6,
            [0.85, 0.75, 0.65, 1.0],
        );
        add_mesh(&mut vertices, &mut indices, hand_l_v, hand_l_i);
        
        let (hand_r_v, hand_r_i) = create_sphere(
            Vec3::new(arm_offset + 0.05, arm_y - 0.4, 0.18),
            0.04,
            8, 6,
            [0.85, 0.75, 0.65, 1.0],
        );
        add_mesh(&mut vertices, &mut indices, hand_r_v, hand_r_i);

        // === TEAM INSIGNIA ===
        let (insignia_v, insignia_i) = create_box_rounded(
            Vec3::new(0.12, torso_y + 0.1, 0.12),
            Vec3::new(0.06, 0.06, 0.01),
            accent_color,
        );
        add_mesh(&mut vertices, &mut indices, insignia_v, insignia_i);

        Self {
            vertices,
            indices,
            team,
            height,
            width: 0.6,
        }
    }
}

/// Weapon model types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WeaponType {
    Knife,
    Pistol,
    Rifle,
    Sniper,
}

/// First-person weapon model with animations
#[derive(Clone, Debug)]
pub struct WeaponModel {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub weapon_type: WeaponType,
    pub position_offset: Vec3,
    pub rotation_offset: Vec3,
    pub reload_progress: f32,
    pub recoil: f32,
}

impl WeaponModel {
    pub fn knife() -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let metal = [0.7, 0.7, 0.75, 1.0];
        let handle = [0.25, 0.15, 0.1, 1.0];

        // Blade
        let (blade_v, blade_i) = create_box_rounded(
            Vec3::new(0.0, 0.0, 0.15),
            Vec3::new(0.02, 0.15, 0.25),
            metal,
        );
        add_mesh(&mut vertices, &mut indices, blade_v, blade_i);

        // Handle
        let (handle_v, handle_i) = create_cylinder(
            Vec3::new(0.0, 0.0, -0.05),
            0.015, 0.12,
            8,
            handle,
        );
        add_mesh(&mut vertices, &mut indices, handle_v, handle_i);

        Self {
            vertices,
            indices,
            weapon_type: WeaponType::Knife,
            position_offset: Vec3::new(0.15, -0.15, 0.3),
            rotation_offset: Vec3::new(-0.3, 0.0, 0.0),
            reload_progress: 0.0,
            recoil: 0.0,
        }
    }

    pub fn pistol() -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let metal = [0.2, 0.2, 0.22, 1.0];
        let grip = [0.15, 0.12, 0.1, 1.0];
        let chrome = [0.8, 0.8, 0.85, 1.0];

        // Slide
        let (slide_v, slide_i) = create_box_rounded(
            Vec3::new(0.0, 0.02, 0.08),
            Vec3::new(0.025, 0.035, 0.16),
            chrome,
        );
        add_mesh(&mut vertices, &mut indices, slide_v, slide_i);

        // Frame
        let (frame_v, frame_i) = create_box_rounded(
            Vec3::new(0.0, -0.01, 0.05),
            Vec3::new(0.022, 0.025, 0.12),
            metal,
        );
        add_mesh(&mut vertices, &mut indices, frame_v, frame_i);

        // Grip
        let (grip_v, grip_i) = create_box_rounded(
            Vec3::new(0.0, -0.05, -0.02),
            Vec3::new(0.022, 0.08, 0.04),
            grip,
        );
        add_mesh(&mut vertices, &mut indices, grip_v, grip_i);

        // Barrel
        let (barrel_v, barrel_i) = create_cylinder(
            Vec3::new(0.0, 0.02, 0.2),
            0.008, 0.06,
            8,
            [0.1, 0.1, 0.1, 1.0],
        );
        add_mesh(&mut vertices, &mut indices, barrel_v, barrel_i);

        // Trigger
        let (trigger_v, trigger_i) = create_box_rounded(
            Vec3::new(0.0, -0.025, 0.02),
            Vec3::new(0.005, 0.02, 0.015),
            metal,
        );
        add_mesh(&mut vertices, &mut indices, trigger_v, trigger_i);

        // Front sight
        let (sight_v, sight_i) = create_box_rounded(
            Vec3::new(0.0, 0.045, 0.14),
            Vec3::new(0.003, 0.012, 0.005),
            [0.1, 0.1, 0.1, 1.0],
        );
        add_mesh(&mut vertices, &mut indices, sight_v, sight_i);

        Self {
            vertices,
            indices,
            weapon_type: WeaponType::Pistol,
            position_offset: Vec3::new(0.18, -0.12, 0.25),
            rotation_offset: Vec3::new(0.0, 0.0, 0.0),
            reload_progress: 0.0,
            recoil: 0.0,
        }
    }

    pub fn rifle() -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let metal = [0.15, 0.15, 0.17, 1.0];
        let wood = [0.4, 0.25, 0.12, 1.0];
        let dark = [0.08, 0.08, 0.08, 1.0];

        // Main body/receiver
        let (body_v, body_i) = create_box_rounded(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.035, 0.055, 0.35),
            metal,
        );
        add_mesh(&mut vertices, &mut indices, body_v, body_i);

        // Barrel
        let (barrel_v, barrel_i) = create_cylinder(
            Vec3::new(0.0, 0.01, 0.35),
            0.012, 0.25,
            12,
            dark,
        );
        add_mesh(&mut vertices, &mut indices, barrel_v, barrel_i);

        // Stock
        let (stock_v, stock_i) = create_box_rounded(
            Vec3::new(0.0, -0.02, -0.22),
            Vec3::new(0.03, 0.08, 0.2),
            wood,
        );
        add_mesh(&mut vertices, &mut indices, stock_v, stock_i);

        // Grip
        let (grip_v, grip_i) = create_box_rounded(
            Vec3::new(0.0, -0.06, -0.05),
            Vec3::new(0.025, 0.08, 0.04),
            wood,
        );
        add_mesh(&mut vertices, &mut indices, grip_v, grip_i);

        // Magazine
        let (mag_v, mag_i) = create_box_rounded(
            Vec3::new(0.0, -0.08, 0.08),
            Vec3::new(0.02, 0.1, 0.06),
            metal,
        );
        add_mesh(&mut vertices, &mut indices, mag_v, mag_i);

        // Handguard
        let (guard_v, guard_i) = create_box_rounded(
            Vec3::new(0.0, -0.01, 0.2),
            Vec3::new(0.03, 0.04, 0.15),
            wood,
        );
        add_mesh(&mut vertices, &mut indices, guard_v, guard_i);

        // Front sight
        let (fsight_v, fsight_i) = create_box_rounded(
            Vec3::new(0.0, 0.04, 0.5),
            Vec3::new(0.005, 0.025, 0.01),
            dark,
        );
        add_mesh(&mut vertices, &mut indices, fsight_v, fsight_i);

        // Rear sight
        let (rsight_v, rsight_i) = create_box_rounded(
            Vec3::new(0.0, 0.04, 0.1),
            Vec3::new(0.015, 0.02, 0.02),
            dark,
        );
        add_mesh(&mut vertices, &mut indices, rsight_v, rsight_i);

        // Gas tube
        let (gas_v, gas_i) = create_cylinder(
            Vec3::new(0.0, 0.035, 0.25),
            0.006, 0.2,
            8,
            dark,
        );
        add_mesh(&mut vertices, &mut indices, gas_v, gas_i);

        Self {
            vertices,
            indices,
            weapon_type: WeaponType::Rifle,
            position_offset: Vec3::new(0.2, -0.18, 0.3),
            rotation_offset: Vec3::new(0.0, 0.0, 0.0),
            reload_progress: 0.0,
            recoil: 0.0,
        }
    }

    /// Update weapon animation (reload, recoil)
    pub fn update(&mut self, delta_time: f32, is_reloading: bool, just_fired: bool) {
        // Reload animation
        if is_reloading {
            self.reload_progress = (self.reload_progress + delta_time * 0.5).min(1.0);
        } else {
            self.reload_progress = 0.0;
        }

        // Recoil animation
        if just_fired {
            self.recoil = match self.weapon_type {
                WeaponType::Knife => 0.0,
                WeaponType::Pistol => 0.08,
                WeaponType::Rifle => 0.05,
                WeaponType::Sniper => 0.15,
            };
        } else {
            self.recoil = (self.recoil - delta_time * 5.0).max(0.0);
        }
    }

    /// Get model transform for rendering
    pub fn get_transform(&self) -> Mat4 {
        let mut offset = self.position_offset;
        
        // Apply reload animation
        if self.reload_progress > 0.0 {
            let reload_anim = (self.reload_progress * PI).sin();
            offset.y -= reload_anim * 0.15;
            offset.x += reload_anim * 0.1;
        }
        
        // Apply recoil
        offset.z -= self.recoil;
        let recoil_rot = Quat::from_rotation_x(-self.recoil * 0.5);
        
        let rotation = Quat::from_euler(
            glam::EulerRot::XYZ,
            self.rotation_offset.x,
            self.rotation_offset.y,
            self.rotation_offset.z,
        ) * recoil_rot;

        Mat4::from_rotation_translation(rotation, offset)
    }
}

// === Helper functions to create geometry ===

fn add_mesh(vertices: &mut Vec<ModelVertex>, indices: &mut Vec<u32>, new_verts: Vec<ModelVertex>, new_indices: Vec<u32>) {
    let base = vertices.len() as u32;
    vertices.extend(new_verts);
    indices.extend(new_indices.iter().map(|i| i + base));
}

fn create_sphere(center: Vec3, radius: f32, segments: u32, rings: u32, color: [f32; 4]) -> (Vec<ModelVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for ring in 0..=rings {
        let phi = PI * ring as f32 / rings as f32;
        let y = phi.cos();
        let ring_radius = phi.sin();

        for seg in 0..=segments {
            let theta = 2.0 * PI * seg as f32 / segments as f32;
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let normal = [x, y, z];
            let pos = [
                center.x + x * radius,
                center.y + y * radius,
                center.z + z * radius,
            ];
            let uv = [seg as f32 / segments as f32, ring as f32 / rings as f32];

            let mut v = ModelVertex::new(pos, normal, uv);
            v.color = color;
            vertices.push(v);
        }
    }

    for ring in 0..rings {
        for seg in 0..segments {
            let curr = ring * (segments + 1) + seg;
            let next = curr + segments + 1;

            indices.push(curr);
            indices.push(next);
            indices.push(curr + 1);

            indices.push(curr + 1);
            indices.push(next);
            indices.push(next + 1);
        }
    }

    (vertices, indices)
}

fn create_hemisphere(center: Vec3, radius: f32, segments: u32, rings: u32, color: [f32; 4]) -> (Vec<ModelVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for ring in 0..=rings {
        let phi = (PI / 2.0) * ring as f32 / rings as f32;
        let y = phi.cos();
        let ring_radius = phi.sin();

        for seg in 0..=segments {
            let theta = 2.0 * PI * seg as f32 / segments as f32;
            let x = ring_radius * theta.cos();
            let z = ring_radius * theta.sin();

            let normal = [x, y, z];
            let pos = [
                center.x + x * radius,
                center.y + y * radius,
                center.z + z * radius,
            ];
            let uv = [seg as f32 / segments as f32, ring as f32 / rings as f32];

            let mut v = ModelVertex::new(pos, normal, uv);
            v.color = color;
            vertices.push(v);
        }
    }

    for ring in 0..rings {
        for seg in 0..segments {
            let curr = ring * (segments + 1) + seg;
            let next = curr + segments + 1;

            indices.push(curr);
            indices.push(next);
            indices.push(curr + 1);

            indices.push(curr + 1);
            indices.push(next);
            indices.push(next + 1);
        }
    }

    (vertices, indices)
}

fn create_cylinder(center: Vec3, radius: f32, height: f32, segments: u32, color: [f32; 4]) -> (Vec<ModelVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let half_height = height / 2.0;

    // Side vertices
    for i in 0..=segments {
        let theta = 2.0 * PI * i as f32 / segments as f32;
        let x = theta.cos();
        let z = theta.sin();

        // Bottom
        let mut v = ModelVertex::new(
            [center.x + x * radius, center.y - half_height, center.z + z * radius],
            [x, 0.0, z],
            [i as f32 / segments as f32, 0.0],
        );
        v.color = color;
        vertices.push(v);

        // Top
        let mut v = ModelVertex::new(
            [center.x + x * radius, center.y + half_height, center.z + z * radius],
            [x, 0.0, z],
            [i as f32 / segments as f32, 1.0],
        );
        v.color = color;
        vertices.push(v);
    }

    // Side indices
    for i in 0..segments {
        let base = i * 2;
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);
        indices.push(base + 2);
        indices.push(base + 1);
        indices.push(base + 3);
    }

    // Cap centers
    let bottom_center = vertices.len() as u32;
    let mut v = ModelVertex::new(
        [center.x, center.y - half_height, center.z],
        [0.0, -1.0, 0.0],
        [0.5, 0.5],
    );
    v.color = color;
    vertices.push(v);

    let top_center = vertices.len() as u32;
    let mut v = ModelVertex::new(
        [center.x, center.y + half_height, center.z],
        [0.0, 1.0, 0.0],
        [0.5, 0.5],
    );
    v.color = color;
    vertices.push(v);

    // Cap vertices
    let cap_start = vertices.len() as u32;
    for i in 0..=segments {
        let theta = 2.0 * PI * i as f32 / segments as f32;
        let x = theta.cos();
        let z = theta.sin();

        // Bottom cap
        let mut v = ModelVertex::new(
            [center.x + x * radius, center.y - half_height, center.z + z * radius],
            [0.0, -1.0, 0.0],
            [(x + 1.0) / 2.0, (z + 1.0) / 2.0],
        );
        v.color = color;
        vertices.push(v);

        // Top cap
        let mut v = ModelVertex::new(
            [center.x + x * radius, center.y + half_height, center.z + z * radius],
            [0.0, 1.0, 0.0],
            [(x + 1.0) / 2.0, (z + 1.0) / 2.0],
        );
        v.color = color;
        vertices.push(v);
    }

    // Cap indices
    for i in 0..segments {
        let bi = cap_start + i * 2;
        indices.push(bottom_center);
        indices.push(bi + 2);
        indices.push(bi);

        indices.push(top_center);
        indices.push(bi + 1);
        indices.push(bi + 3);
    }

    (vertices, indices)
}

fn create_box_rounded(center: Vec3, size: Vec3, color: [f32; 4]) -> (Vec<ModelVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let half = size * 0.5;
    let min = center - half;
    let max = center + half;

    // Define 8 corners
    let corners = [
        [min.x, min.y, min.z],
        [max.x, min.y, min.z],
        [max.x, max.y, min.z],
        [min.x, max.y, min.z],
        [min.x, min.y, max.z],
        [max.x, min.y, max.z],
        [max.x, max.y, max.z],
        [min.x, max.y, max.z],
    ];

    // 6 faces with proper normals
    let faces: [([usize; 4], [f32; 3]); 6] = [
        ([4, 5, 6, 7], [0.0, 0.0, 1.0]),  // Front
        ([1, 0, 3, 2], [0.0, 0.0, -1.0]), // Back
        ([3, 7, 6, 2], [0.0, 1.0, 0.0]),  // Top
        ([0, 1, 5, 4], [0.0, -1.0, 0.0]), // Bottom
        ([5, 1, 2, 6], [1.0, 0.0, 0.0]),  // Right
        ([0, 4, 7, 3], [-1.0, 0.0, 0.0]), // Left
    ];

    for (corner_idx, normal) in faces {
        let base = vertices.len() as u32;

        for (i, &idx) in corner_idx.iter().enumerate() {
            let uv = match i {
                0 => [0.0, 1.0],
                1 => [1.0, 1.0],
                2 => [1.0, 0.0],
                _ => [0.0, 0.0],
            };
            let mut v = ModelVertex::new(corners[idx], normal, uv);
            v.color = color;
            vertices.push(v);
        }

        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);
        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 3);
    }

    (vertices, indices)
}
