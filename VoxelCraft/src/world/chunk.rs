// VoxelCraft - Chunk System

use super::{BlockType, CHUNK_SIZE, CHUNK_HEIGHT};
use serde::{Serialize, Deserialize};

/// A 16x128x16 chunk of blocks
#[derive(Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub x: i32,
    pub z: i32,
    blocks: Vec<BlockType>,
    pub dirty: bool,
    #[serde(skip)]
    pub mesh_valid: bool,
}

impl Chunk {
    pub fn new(x: i32, z: i32) -> Self {
        Self {
            x,
            z,
            blocks: vec![BlockType::Air; (CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE) as usize],
            dirty: true,
            mesh_valid: false,
        }
    }

    #[inline]
    fn index(x: i32, y: i32, z: i32) -> usize {
        ((y * CHUNK_SIZE * CHUNK_SIZE) + (z * CHUNK_SIZE) + x) as usize
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockType {
        if x < 0 || x >= CHUNK_SIZE || y < 0 || y >= CHUNK_HEIGHT || z < 0 || z >= CHUNK_SIZE {
            return BlockType::Air;
        }
        self.blocks[Self::index(x, y, z)]
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: BlockType) {
        if x < 0 || x >= CHUNK_SIZE || y < 0 || y >= CHUNK_HEIGHT || z < 0 || z >= CHUNK_SIZE {
            return;
        }
        self.blocks[Self::index(x, y, z)] = block;
        self.dirty = true;
        self.mesh_valid = false;
    }

    pub fn fill_layer(&mut self, y: i32, block: BlockType) {
        if y < 0 || y >= CHUNK_HEIGHT {
            return;
        }
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                self.blocks[Self::index(x, y, z)] = block;
            }
        }
    }

    pub fn fill_column(&mut self, x: i32, z: i32, from_y: i32, to_y: i32, block: BlockType) {
        for y in from_y.max(0)..to_y.min(CHUNK_HEIGHT) {
            self.blocks[Self::index(x, y, z)] = block;
        }
    }

    pub fn get_blocks(&self) -> &[BlockType] {
        &self.blocks
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|&b| b == BlockType::Air)
    }

    pub fn count_blocks(&self) -> usize {
        self.blocks.iter().filter(|&&b| b != BlockType::Air).count()
    }
}

/// Chunk mesh for rendering
#[derive(Default)]
pub struct ChunkMesh {
    pub vertices: Vec<ChunkVertex>,
    pub indices: Vec<u32>,
    pub water_vertices: Vec<ChunkVertex>,
    pub water_indices: Vec<u32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub ao: f32,
    pub light: f32,
}

impl ChunkMesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.water_vertices.clear();
        self.water_indices.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

/// Mesh generator for chunks
pub struct ChunkMesher;

impl ChunkMesher {
    pub fn generate_mesh(chunk: &Chunk, neighbors: &[Option<&Chunk>; 4]) -> ChunkMesh {
        let mut mesh = ChunkMesh::new();

        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = chunk.get_block(x, y, z);
                    if block == BlockType::Air {
                        continue;
                    }

                    let world_x = chunk.x * CHUNK_SIZE + x;
                    let world_z = chunk.z * CHUNK_SIZE + z;

                    // Check each face
                    let faces = [
                        (0, 1, 0, Face::Top),
                        (0, -1, 0, Face::Bottom),
                        (1, 0, 0, Face::Right),
                        (-1, 0, 0, Face::Left),
                        (0, 0, 1, Face::Front),
                        (0, 0, -1, Face::Back),
                    ];

                    for (dx, dy, dz, face) in faces {
                        let nx = x + dx;
                        let ny = y + dy;
                        let nz = z + dz;

                        let neighbor_block = Self::get_neighbor_block(chunk, neighbors, nx, ny, nz);

                        if !neighbor_block.is_solid() || (block == BlockType::Water && neighbor_block != BlockType::Water) {
                            let is_water = block == BlockType::Water;
                            
                            Self::add_face(
                                if is_water { &mut mesh.water_vertices } else { &mut mesh.vertices },
                                if is_water { &mut mesh.water_indices } else { &mut mesh.indices },
                                world_x as f32,
                                y as f32,
                                world_z as f32,
                                face,
                                block,
                            );
                        }
                    }
                }
            }
        }

        mesh
    }

    fn get_neighbor_block(chunk: &Chunk, neighbors: &[Option<&Chunk>; 4], x: i32, y: i32, z: i32) -> BlockType {
        if y < 0 || y >= CHUNK_HEIGHT {
            return BlockType::Air;
        }

        if x >= 0 && x < CHUNK_SIZE && z >= 0 && z < CHUNK_SIZE {
            return chunk.get_block(x, y, z);
        }

        // Check neighbor chunks
        if x < 0 {
            if let Some(n) = neighbors[0] { // -X neighbor
                return n.get_block(CHUNK_SIZE - 1, y, z);
            }
        } else if x >= CHUNK_SIZE {
            if let Some(n) = neighbors[1] { // +X neighbor
                return n.get_block(0, y, z);
            }
        }

        if z < 0 {
            if let Some(n) = neighbors[2] { // -Z neighbor
                return n.get_block(x, y, CHUNK_SIZE - 1);
            }
        } else if z >= CHUNK_SIZE {
            if let Some(n) = neighbors[3] { // +Z neighbor
                return n.get_block(x, y, 0);
            }
        }

        BlockType::Air
    }

    fn add_face(vertices: &mut Vec<ChunkVertex>, indices: &mut Vec<u32>, x: f32, y: f32, z: f32, face: Face, block: BlockType) {
        let base_idx = vertices.len() as u32;
        let uv = block.get_uv(face);
        let normal = face.normal();
        let ao = 1.0; // TODO: Calculate AO
        let light = 1.0; // TODO: Calculate light

        let (positions, uvs) = face.vertices();

        for i in 0..4 {
            vertices.push(ChunkVertex {
                position: [x + positions[i].0, y + positions[i].1, z + positions[i].2],
                normal,
                uv: [uv.0 + uvs[i].0 * 0.0625, uv.1 + uvs[i].1 * 0.0625],
                ao,
                light,
            });
        }

        // Two triangles
        indices.extend_from_slice(&[
            base_idx, base_idx + 1, base_idx + 2,
            base_idx, base_idx + 2, base_idx + 3,
        ]);
    }
}

#[derive(Copy, Clone)]
pub enum Face {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

impl Face {
    pub fn normal(&self) -> [f32; 3] {
        match self {
            Face::Top => [0.0, 1.0, 0.0],
            Face::Bottom => [0.0, -1.0, 0.0],
            Face::Left => [-1.0, 0.0, 0.0],
            Face::Right => [1.0, 0.0, 0.0],
            Face::Front => [0.0, 0.0, 1.0],
            Face::Back => [0.0, 0.0, -1.0],
        }
    }

    pub fn vertices(&self) -> ([(f32, f32, f32); 4], [(f32, f32); 4]) {
        match self {
            Face::Top => (
                [(0.0, 1.0, 0.0), (1.0, 1.0, 0.0), (1.0, 1.0, 1.0), (0.0, 1.0, 1.0)],
                [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            ),
            Face::Bottom => (
                [(0.0, 0.0, 1.0), (1.0, 0.0, 1.0), (1.0, 0.0, 0.0), (0.0, 0.0, 0.0)],
                [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)],
            ),
            Face::Front => (
                [(0.0, 0.0, 1.0), (0.0, 1.0, 1.0), (1.0, 1.0, 1.0), (1.0, 0.0, 1.0)],
                [(0.0, 1.0), (0.0, 0.0), (1.0, 0.0), (1.0, 1.0)],
            ),
            Face::Back => (
                [(1.0, 0.0, 0.0), (1.0, 1.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 0.0)],
                [(0.0, 1.0), (0.0, 0.0), (1.0, 0.0), (1.0, 1.0)],
            ),
            Face::Right => (
                [(1.0, 0.0, 1.0), (1.0, 1.0, 1.0), (1.0, 1.0, 0.0), (1.0, 0.0, 0.0)],
                [(0.0, 1.0), (0.0, 0.0), (1.0, 0.0), (1.0, 1.0)],
            ),
            Face::Left => (
                [(0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 1.0, 1.0), (0.0, 0.0, 1.0)],
                [(0.0, 1.0), (0.0, 0.0), (1.0, 0.0), (1.0, 1.0)],
            ),
        }
    }
}
