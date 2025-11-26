//! Simple CS-style map with boxes and walls (de_dust inspired)

use glam::Vec3;

#[derive(Clone, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    pub fn intersects_ray(&self, origin: Vec3, direction: Vec3) -> Option<f32> {
        let inv_dir = Vec3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z);
        
        let t1 = (self.min.x - origin.x) * inv_dir.x;
        let t2 = (self.max.x - origin.x) * inv_dir.x;
        let t3 = (self.min.y - origin.y) * inv_dir.y;
        let t4 = (self.max.y - origin.y) * inv_dir.y;
        let t5 = (self.min.z - origin.z) * inv_dir.z;
        let t6 = (self.max.z - origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax {
            None
        } else {
            Some(if tmin < 0.0 { tmax } else { tmin })
        }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockType {
    Wall,
    Floor,
    Crate,
    Platform,
    SpawnT,    // Terrorist spawn
    SpawnCT,   // Counter-Terrorist spawn
    BombsiteA,
    BombsiteB,
}

#[derive(Clone, Debug)]
pub struct MapBlock {
    pub aabb: AABB,
    pub block_type: BlockType,
    pub color: [f32; 3],
}

impl MapBlock {
    pub fn new(aabb: AABB, block_type: BlockType) -> Self {
        let color = match block_type {
            BlockType::Wall => [0.6, 0.55, 0.45],      // Tan/sandstone
            BlockType::Floor => [0.4, 0.38, 0.35],     // Dark brown
            BlockType::Crate => [0.55, 0.35, 0.15],    // Wood brown
            BlockType::Platform => [0.5, 0.48, 0.42],  // Light stone
            BlockType::SpawnT => [0.8, 0.4, 0.2],      // Orange
            BlockType::SpawnCT => [0.2, 0.4, 0.8],     // Blue
            BlockType::BombsiteA => [0.9, 0.2, 0.2],   // Red
            BlockType::BombsiteB => [0.9, 0.6, 0.1],   // Yellow
        };
        Self { aabb, block_type, color }
    }
}

pub struct GameMap {
    pub blocks: Vec<MapBlock>,
    pub spawn_points_t: Vec<Vec3>,
    pub spawn_points_ct: Vec<Vec3>,
}

impl GameMap {
    pub fn de_dust_simple() -> Self {
        let mut blocks = Vec::new();
        let mut spawn_points_t = Vec::new();
        let mut spawn_points_ct = Vec::new();

        // ========== GROUND FLOOR ==========
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(0.0, -0.5, 0.0), Vec3::new(120.0, 1.0, 80.0)),
            BlockType::Floor,
        ));

        // ========== OUTER WALLS ==========
        // North wall
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(0.0, 4.0, 40.0), Vec3::new(120.0, 8.0, 2.0)),
            BlockType::Wall,
        ));
        // South wall
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(0.0, 4.0, -40.0), Vec3::new(120.0, 8.0, 2.0)),
            BlockType::Wall,
        ));
        // East wall
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(60.0, 4.0, 0.0), Vec3::new(2.0, 8.0, 80.0)),
            BlockType::Wall,
        ));
        // West wall
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(-60.0, 4.0, 0.0), Vec3::new(2.0, 8.0, 80.0)),
            BlockType::Wall,
        ));

        // ========== CENTRAL CORRIDOR ==========
        // Long mid wall with gap
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(-15.0, 3.0, 0.0), Vec3::new(30.0, 6.0, 1.5)),
            BlockType::Wall,
        ));
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(20.0, 3.0, 0.0), Vec3::new(20.0, 6.0, 1.5)),
            BlockType::Wall,
        ));

        // ========== CRATES - Cover positions ==========
        // T Side crates
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(-40.0, 1.5, 20.0), Vec3::new(4.0, 3.0, 4.0)),
            BlockType::Crate,
        ));
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(-40.0, 1.5, -20.0), Vec3::new(4.0, 3.0, 4.0)),
            BlockType::Crate,
        ));
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(-35.0, 1.0, 0.0), Vec3::new(3.0, 2.0, 3.0)),
            BlockType::Crate,
        ));

        // Mid crates
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(0.0, 1.5, 15.0), Vec3::new(5.0, 3.0, 5.0)),
            BlockType::Crate,
        ));
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(0.0, 1.5, -15.0), Vec3::new(5.0, 3.0, 5.0)),
            BlockType::Crate,
        ));
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(5.0, 4.0, 15.0), Vec3::new(3.0, 2.0, 3.0)),
            BlockType::Crate,
        )); // Stacked

        // CT Side crates
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(40.0, 1.5, 25.0), Vec3::new(4.0, 3.0, 4.0)),
            BlockType::Crate,
        ));
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(40.0, 1.5, -25.0), Vec3::new(4.0, 3.0, 4.0)),
            BlockType::Crate,
        ));
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(35.0, 1.0, 10.0), Vec3::new(3.0, 2.0, 3.0)),
            BlockType::Crate,
        ));
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(35.0, 1.0, -10.0), Vec3::new(3.0, 2.0, 3.0)),
            BlockType::Crate,
        ));

        // ========== BUILDINGS/STRUCTURES ==========
        // A Site building
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(45.0, 3.0, 30.0), Vec3::new(15.0, 6.0, 10.0)),
            BlockType::Wall,
        ));
        // B Site building
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(45.0, 3.0, -30.0), Vec3::new(15.0, 6.0, 10.0)),
            BlockType::Wall,
        ));

        // T Spawn building
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(-50.0, 3.0, 0.0), Vec3::new(8.0, 6.0, 20.0)),
            BlockType::Wall,
        ));

        // ========== PLATFORMS ==========
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(25.0, 2.0, 30.0), Vec3::new(8.0, 0.5, 8.0)),
            BlockType::Platform,
        ));
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(25.0, 2.0, -30.0), Vec3::new(8.0, 0.5, 8.0)),
            BlockType::Platform,
        ));

        // ========== BOMBSITES (visual markers) ==========
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(50.0, 0.1, 25.0), Vec3::new(10.0, 0.2, 10.0)),
            BlockType::BombsiteA,
        ));
        blocks.push(MapBlock::new(
            AABB::from_center_size(Vec3::new(50.0, 0.1, -25.0), Vec3::new(10.0, 0.2, 10.0)),
            BlockType::BombsiteB,
        ));

        // ========== SPAWN POINTS ==========
        // T Spawn
        spawn_points_t.push(Vec3::new(-55.0, 1.0, 5.0));
        spawn_points_t.push(Vec3::new(-55.0, 1.0, 0.0));
        spawn_points_t.push(Vec3::new(-55.0, 1.0, -5.0));
        spawn_points_t.push(Vec3::new(-52.0, 1.0, 8.0));
        spawn_points_t.push(Vec3::new(-52.0, 1.0, -8.0));

        // CT Spawn
        spawn_points_ct.push(Vec3::new(55.0, 1.0, 5.0));
        spawn_points_ct.push(Vec3::new(55.0, 1.0, 0.0));
        spawn_points_ct.push(Vec3::new(55.0, 1.0, -5.0));
        spawn_points_ct.push(Vec3::new(52.0, 1.0, 8.0));
        spawn_points_ct.push(Vec3::new(52.0, 1.0, -8.0));

        Self {
            blocks,
            spawn_points_t,
            spawn_points_ct,
        }
    }

    pub fn check_collision(&self, position: Vec3, radius: f32) -> bool {
        for block in &self.blocks {
            // Skip floor and non-solid blocks
            if block.block_type == BlockType::Floor 
                || block.block_type == BlockType::BombsiteA 
                || block.block_type == BlockType::BombsiteB 
                || block.block_type == BlockType::SpawnT 
                || block.block_type == BlockType::SpawnCT {
                continue;
            }

            // Expand AABB by radius for collision check
            let expanded = AABB {
                min: block.aabb.min - Vec3::splat(radius),
                max: block.aabb.max + Vec3::splat(radius),
            };

            if expanded.contains_point(position) {
                return true;
            }
        }
        false
    }

    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<(f32, &MapBlock)> {
        let mut closest: Option<(f32, &MapBlock)> = None;

        for block in &self.blocks {
            // Skip non-solid blocks for raycast
            if block.block_type == BlockType::BombsiteA 
                || block.block_type == BlockType::BombsiteB 
                || block.block_type == BlockType::SpawnT 
                || block.block_type == BlockType::SpawnCT {
                continue;
            }

            if let Some(t) = block.aabb.intersects_ray(origin, direction) {
                if t > 0.0 && t < max_distance {
                    if closest.is_none() || t < closest.unwrap().0 {
                        closest = Some((t, block));
                    }
                }
            }
        }

        closest
    }

    /// Generate mesh data for rendering
    pub fn generate_mesh_data(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for block in &self.blocks {
            let base_index = vertices.len() as u32;
            let (v, i) = generate_box_mesh(&block.aabb, block.color);
            vertices.extend(v);
            for idx in i {
                indices.push(base_index + idx);
            }
        }

        (vertices, indices)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

fn generate_box_mesh(aabb: &AABB, color: [f32; 3]) -> (Vec<Vertex>, Vec<u32>) {
    let min = aabb.min;
    let max = aabb.max;

    // 8 corners of the box
    let corners = [
        [min.x, min.y, min.z], // 0: back-bottom-left
        [max.x, min.y, min.z], // 1: back-bottom-right
        [max.x, max.y, min.z], // 2: back-top-right
        [min.x, max.y, min.z], // 3: back-top-left
        [min.x, min.y, max.z], // 4: front-bottom-left
        [max.x, min.y, max.z], // 5: front-bottom-right
        [max.x, max.y, max.z], // 6: front-top-right
        [min.x, max.y, max.z], // 7: front-top-left
    ];

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Each face has 4 vertices with proper normals
    let faces = [
        // Front face (+Z)
        ([4, 5, 6, 7], [0.0, 0.0, 1.0]),
        // Back face (-Z)
        ([1, 0, 3, 2], [0.0, 0.0, -1.0]),
        // Top face (+Y)
        ([3, 7, 6, 2], [0.0, 1.0, 0.0]),
        // Bottom face (-Y)
        ([4, 0, 1, 5], [0.0, -1.0, 0.0]),
        // Right face (+X)
        ([5, 1, 2, 6], [1.0, 0.0, 0.0]),
        // Left face (-X)
        ([0, 4, 7, 3], [-1.0, 0.0, 0.0]),
    ];

    for (corner_indices, normal) in faces {
        let base = vertices.len() as u32;
        
        for &idx in &corner_indices {
            vertices.push(Vertex {
                position: corners[idx],
                normal,
                color,
            });
        }

        // Two triangles per face
        indices.extend_from_slice(&[
            base, base + 1, base + 2,
            base, base + 2, base + 3,
        ]);
    }

    (vertices, indices)
}

impl Default for GameMap {
    fn default() -> Self {
        Self::de_dust_simple()
    }
}
