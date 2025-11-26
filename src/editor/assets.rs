//! Asset management - models, textures, materials, sounds

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use glam::Vec3;

/// Asset types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Model,
    Texture,
    Material,
    Sound,
    Prefab,
    Script,
}

impl AssetType {
    pub fn extensions(&self) -> &[&str] {
        match self {
            AssetType::Model => &["obj", "gltf", "glb", "fbx", "vmdl"],
            AssetType::Texture => &["png", "jpg", "jpeg", "bmp", "tga", "hdr"],
            AssetType::Material => &["vmat", "json"],
            AssetType::Sound => &["wav", "ogg", "mp3"],
            AssetType::Prefab => &["vprefab", "json"],
            AssetType::Script => &["lua", "wasm"],
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            AssetType::Model => "🎲",
            AssetType::Texture => "🖼",
            AssetType::Material => "🎨",
            AssetType::Sound => "🔊",
            AssetType::Prefab => "📦",
            AssetType::Script => "📜",
        }
    }
}

/// Asset metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub asset_type: AssetType,
    pub path: PathBuf,
    pub size: u64,
    pub thumbnail: Option<Vec<u8>>,
    pub metadata: AssetMetadata,
}

impl Asset {
    pub fn from_path(path: &Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_lowercase();
        let asset_type = Self::type_from_extension(&extension)?;
        let name = path.file_stem()?.to_str()?.to_string();
        let size = fs::metadata(path).ok()?.len();

        Some(Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            asset_type,
            path: path.to_path_buf(),
            size,
            thumbnail: None,
            metadata: AssetMetadata::default(),
        })
    }

    fn type_from_extension(ext: &str) -> Option<AssetType> {
        for asset_type in [
            AssetType::Model,
            AssetType::Texture,
            AssetType::Material,
            AssetType::Sound,
            AssetType::Prefab,
            AssetType::Script,
        ] {
            if asset_type.extensions().contains(&ext) {
                return Some(asset_type);
            }
        }
        None
    }
}

/// Asset-specific metadata
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AssetMetadata {
    // Model metadata
    pub vertex_count: Option<u32>,
    pub triangle_count: Option<u32>,
    pub bounds: Option<[Vec3; 2]>,
    
    // Texture metadata
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub has_alpha: Option<bool>,
    
    // Sound metadata
    pub duration_secs: Option<f32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
}

/// Asset manager
pub struct AssetManager {
    pub assets: HashMap<String, Asset>,
    pub asset_dirs: Vec<PathBuf>,
    pub builtin_models: HashMap<String, BuiltinModel>,
    pub builtin_textures: HashMap<String, BuiltinTexture>,
}

impl AssetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            assets: HashMap::new(),
            asset_dirs: Vec::new(),
            builtin_models: HashMap::new(),
            builtin_textures: HashMap::new(),
        };
        manager.register_builtins();
        manager
    }

    /// Register built-in assets
    fn register_builtins(&mut self) {
        // Built-in models
        let builtin_models = [
            ("box", BuiltinModel::Box),
            ("sphere", BuiltinModel::Sphere),
            ("cylinder", BuiltinModel::Cylinder),
            ("capsule", BuiltinModel::Capsule),
            ("plane", BuiltinModel::Plane),
            ("stairs", BuiltinModel::Stairs),
            ("arch", BuiltinModel::Arch),
            ("wedge", BuiltinModel::Wedge),
            ("crate", BuiltinModel::Crate),
            ("barrel", BuiltinModel::Barrel),
            ("door", BuiltinModel::Door),
            ("window", BuiltinModel::Window),
            ("fence", BuiltinModel::Fence),
            ("lamp", BuiltinModel::Lamp),
            ("terrorist", BuiltinModel::Terrorist),
            ("counter_terrorist", BuiltinModel::CounterTerrorist),
            ("hostage", BuiltinModel::Hostage),
            ("ak47", BuiltinModel::WeaponAK47),
            ("m4a1", BuiltinModel::WeaponM4A1),
            ("awp", BuiltinModel::WeaponAWP),
            ("deagle", BuiltinModel::WeaponDeagle),
            ("knife", BuiltinModel::WeaponKnife),
            ("bomb", BuiltinModel::Bomb),
        ];
        
        for (name, model) in builtin_models {
            self.builtin_models.insert(name.to_string(), model);
        }

        // Built-in textures
        let builtin_textures = [
            ("concrete", BuiltinTexture::Concrete),
            ("brick", BuiltinTexture::Brick),
            ("sandstone", BuiltinTexture::Sandstone),
            ("wood", BuiltinTexture::Wood),
            ("metal", BuiltinTexture::Metal),
            ("glass", BuiltinTexture::Glass),
            ("grass", BuiltinTexture::Grass),
            ("dirt", BuiltinTexture::Dirt),
            ("water", BuiltinTexture::Water),
            ("lava", BuiltinTexture::Lava),
            ("tile", BuiltinTexture::Tile),
            ("plaster", BuiltinTexture::Plaster),
        ];

        for (name, texture) in builtin_textures {
            self.builtin_textures.insert(name.to_string(), texture);
        }
    }

    /// Add asset directory to scan
    pub fn add_asset_dir(&mut self, path: &Path) {
        if path.is_dir() && !self.asset_dirs.contains(&path.to_path_buf()) {
            self.asset_dirs.push(path.to_path_buf());
        }
    }

    /// Scan all asset directories
    pub fn scan_assets(&mut self) {
        for dir in self.asset_dirs.clone() {
            self.scan_directory(&dir);
        }
    }

    /// Scan a directory for assets
    fn scan_directory(&mut self, dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.scan_directory(&path);
                } else if let Some(asset) = Asset::from_path(&path) {
                    self.assets.insert(asset.id.clone(), asset);
                }
            }
        }
    }

    /// Import an asset file
    pub fn import_asset(&mut self, source: &Path, dest_dir: &Path) -> Option<String> {
        // Copy file to project assets
        let filename = source.file_name()?;
        let dest_path = dest_dir.join(filename);
        fs::copy(source, &dest_path).ok()?;

        // Create asset entry
        let asset = Asset::from_path(&dest_path)?;
        let id = asset.id.clone();
        self.assets.insert(id.clone(), asset);
        
        Some(id)
    }

    /// Get asset by ID
    pub fn get_asset(&self, id: &str) -> Option<&Asset> {
        self.assets.get(id)
    }

    /// Get assets by type
    pub fn assets_by_type(&self, asset_type: AssetType) -> Vec<&Asset> {
        self.assets
            .values()
            .filter(|a| a.asset_type == asset_type)
            .collect()
    }

    /// Get builtin model names
    pub fn builtin_model_names(&self) -> Vec<&str> {
        self.builtin_models.keys().map(|s| s.as_str()).collect()
    }

    /// Get builtin texture names
    pub fn builtin_texture_names(&self) -> Vec<&str> {
        self.builtin_textures.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in procedural models
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BuiltinModel {
    // Primitives
    Box,
    Sphere,
    Cylinder,
    Capsule,
    Plane,
    Wedge,
    Arch,
    Stairs,
    
    // Props
    Crate,
    Barrel,
    Door,
    Window,
    Fence,
    Lamp,
    
    // Characters
    Terrorist,
    CounterTerrorist,
    Hostage,
    
    // Weapons
    WeaponAK47,
    WeaponM4A1,
    WeaponAWP,
    WeaponDeagle,
    WeaponKnife,
    
    // Gameplay
    Bomb,
}

impl BuiltinModel {
    /// Get category for organization
    pub fn category(&self) -> &str {
        match self {
            BuiltinModel::Box | BuiltinModel::Sphere | BuiltinModel::Cylinder |
            BuiltinModel::Capsule | BuiltinModel::Plane | BuiltinModel::Wedge |
            BuiltinModel::Arch | BuiltinModel::Stairs => "Primitives",
            
            BuiltinModel::Crate | BuiltinModel::Barrel | BuiltinModel::Door |
            BuiltinModel::Window | BuiltinModel::Fence | BuiltinModel::Lamp => "Props",
            
            BuiltinModel::Terrorist | BuiltinModel::CounterTerrorist |
            BuiltinModel::Hostage => "Characters",
            
            BuiltinModel::WeaponAK47 | BuiltinModel::WeaponM4A1 | 
            BuiltinModel::WeaponAWP | BuiltinModel::WeaponDeagle |
            BuiltinModel::WeaponKnife => "Weapons",
            
            BuiltinModel::Bomb => "Gameplay",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            BuiltinModel::Box => "Box",
            BuiltinModel::Sphere => "Sphere",
            BuiltinModel::Cylinder => "Cylinder",
            BuiltinModel::Capsule => "Capsule",
            BuiltinModel::Plane => "Plane",
            BuiltinModel::Wedge => "Wedge",
            BuiltinModel::Arch => "Arch",
            BuiltinModel::Stairs => "Stairs",
            BuiltinModel::Crate => "Crate",
            BuiltinModel::Barrel => "Barrel",
            BuiltinModel::Door => "Door",
            BuiltinModel::Window => "Window",
            BuiltinModel::Fence => "Fence",
            BuiltinModel::Lamp => "Lamp",
            BuiltinModel::Terrorist => "Terrorist",
            BuiltinModel::CounterTerrorist => "Counter-Terrorist",
            BuiltinModel::Hostage => "Hostage",
            BuiltinModel::WeaponAK47 => "AK-47",
            BuiltinModel::WeaponM4A1 => "M4A1",
            BuiltinModel::WeaponAWP => "AWP",
            BuiltinModel::WeaponDeagle => "Desert Eagle",
            BuiltinModel::WeaponKnife => "Knife",
            BuiltinModel::Bomb => "C4 Bomb",
        }
    }
}

/// Built-in procedural textures  
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BuiltinTexture {
    Concrete,
    Brick,
    Sandstone,
    Wood,
    Metal,
    Glass,
    Grass,
    Dirt,
    Water,
    Lava,
    Tile,
    Plaster,
}

impl BuiltinTexture {
    pub fn display_name(&self) -> &str {
        match self {
            BuiltinTexture::Concrete => "Concrete",
            BuiltinTexture::Brick => "Brick",
            BuiltinTexture::Sandstone => "Sandstone",
            BuiltinTexture::Wood => "Wood",
            BuiltinTexture::Metal => "Metal",
            BuiltinTexture::Glass => "Glass",
            BuiltinTexture::Grass => "Grass",
            BuiltinTexture::Dirt => "Dirt",
            BuiltinTexture::Water => "Water",
            BuiltinTexture::Lava => "Lava",
            BuiltinTexture::Tile => "Tile",
            BuiltinTexture::Plaster => "Plaster",
        }
    }
}

/// Model mesh data for rendering
#[derive(Clone, Debug)]
pub struct MeshData {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
}

/// Vertex with all attributes
#[derive(Clone, Copy, Debug)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
}

impl BuiltinModel {
    /// Generate mesh data for builtin model
    pub fn generate_mesh(&self) -> MeshData {
        match self {
            BuiltinModel::Box => Self::generate_box(1.0, 1.0, 1.0),
            BuiltinModel::Sphere => Self::generate_sphere(0.5, 32, 16),
            BuiltinModel::Cylinder => Self::generate_cylinder(0.5, 1.0, 32),
            BuiltinModel::Capsule => Self::generate_capsule(0.5, 1.0, 32),
            BuiltinModel::Plane => Self::generate_plane(1.0, 1.0),
            BuiltinModel::Wedge => Self::generate_wedge(1.0, 1.0, 1.0),
            BuiltinModel::Arch => Self::generate_arch(1.0, 2.0, 1.0, 16),
            BuiltinModel::Stairs => Self::generate_stairs(4.0, 2.0, 2.0, 8),
            BuiltinModel::Crate => Self::generate_crate(1.0),
            BuiltinModel::Barrel => Self::generate_barrel(0.5, 1.0),
            _ => Self::generate_box(1.0, 1.0, 1.0), // Default fallback
        }
    }

    fn generate_box(width: f32, height: f32, depth: f32) -> MeshData {
        let hw = width / 2.0;
        let hh = height / 2.0;
        let hd = depth / 2.0;

        let positions = [
            // Front
            [-hw, -hh, hd], [hw, -hh, hd], [hw, hh, hd], [-hw, hh, hd],
            // Back
            [hw, -hh, -hd], [-hw, -hh, -hd], [-hw, hh, -hd], [hw, hh, -hd],
            // Top
            [-hw, hh, hd], [hw, hh, hd], [hw, hh, -hd], [-hw, hh, -hd],
            // Bottom
            [-hw, -hh, -hd], [hw, -hh, -hd], [hw, -hh, hd], [-hw, -hh, hd],
            // Right
            [hw, -hh, hd], [hw, -hh, -hd], [hw, hh, -hd], [hw, hh, hd],
            // Left
            [-hw, -hh, -hd], [-hw, -hh, hd], [-hw, hh, hd], [-hw, hh, -hd],
        ];

        let normals = [
            [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0, -1.0],
            [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.0, -1.0, 0.0],
            [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
        ];

        let uvs = [
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        ];

        let vertices: Vec<ModelVertex> = (0..24)
            .map(|i| ModelVertex {
                position: positions[i],
                normal: normals[i],
                uv: uvs[i],
                tangent: [1.0, 0.0, 0.0, 1.0],
            })
            .collect();

        let indices: Vec<u32> = (0..6)
            .flat_map(|face| {
                let base = face * 4;
                [base, base + 1, base + 2, base, base + 2, base + 3]
            })
            .collect();

        MeshData { vertices, indices }
    }

    fn generate_sphere(radius: f32, segments: u32, rings: u32) -> MeshData {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for y in 0..=rings {
            let v = y as f32 / rings as f32;
            let phi = v * std::f32::consts::PI;
            
            for x in 0..=segments {
                let u = x as f32 / segments as f32;
                let theta = u * std::f32::consts::TAU;
                
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();
                
                let px = radius * sin_phi * cos_theta;
                let py = radius * cos_phi;
                let pz = radius * sin_phi * sin_theta;
                
                let nx = sin_phi * cos_theta;
                let ny = cos_phi;
                let nz = sin_phi * sin_theta;
                
                vertices.push(ModelVertex {
                    position: [px, py, pz],
                    normal: [nx, ny, nz],
                    uv: [u, v],
                    tangent: [-sin_theta, 0.0, cos_theta, 1.0],
                });
            }
        }

        for y in 0..rings {
            for x in 0..segments {
                let stride = segments + 1;
                let a = y * stride + x;
                let b = a + 1;
                let c = (y + 1) * stride + x;
                let d = c + 1;
                
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }

        MeshData { vertices, indices }
    }

    fn generate_cylinder(radius: f32, height: f32, segments: u32) -> MeshData {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let hh = height / 2.0;

        // Side
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            let u = i as f32 / segments as f32;
            
            // Bottom vertex
            vertices.push(ModelVertex {
                position: [radius * cos_a, -hh, radius * sin_a],
                normal: [cos_a, 0.0, sin_a],
                uv: [u, 1.0],
                tangent: [-sin_a, 0.0, cos_a, 1.0],
            });
            
            // Top vertex
            vertices.push(ModelVertex {
                position: [radius * cos_a, hh, radius * sin_a],
                normal: [cos_a, 0.0, sin_a],
                uv: [u, 0.0],
                tangent: [-sin_a, 0.0, cos_a, 1.0],
            });
        }

        // Side indices
        for i in 0..segments {
            let base = i * 2;
            indices.extend_from_slice(&[
                base, base + 2, base + 1,
                base + 1, base + 2, base + 3,
            ]);
        }

        // Top cap
        let top_center = vertices.len() as u32;
        vertices.push(ModelVertex {
            position: [0.0, hh, 0.0],
            normal: [0.0, 1.0, 0.0],
            uv: [0.5, 0.5],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            vertices.push(ModelVertex {
                position: [radius * cos_a, hh, radius * sin_a],
                normal: [0.0, 1.0, 0.0],
                uv: [cos_a * 0.5 + 0.5, sin_a * 0.5 + 0.5],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });
        }
        
        for i in 0..segments {
            indices.extend_from_slice(&[top_center, top_center + 1 + i, top_center + 2 + i]);
        }

        // Bottom cap
        let bottom_center = vertices.len() as u32;
        vertices.push(ModelVertex {
            position: [0.0, -hh, 0.0],
            normal: [0.0, -1.0, 0.0],
            uv: [0.5, 0.5],
            tangent: [1.0, 0.0, 0.0, 1.0],
        });
        
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let cos_a = angle.cos();
            let sin_a = angle.sin();
            vertices.push(ModelVertex {
                position: [radius * cos_a, -hh, radius * sin_a],
                normal: [0.0, -1.0, 0.0],
                uv: [cos_a * 0.5 + 0.5, sin_a * 0.5 + 0.5],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });
        }
        
        for i in 0..segments {
            indices.extend_from_slice(&[bottom_center, bottom_center + 2 + i, bottom_center + 1 + i]);
        }

        MeshData { vertices, indices }
    }

    fn generate_capsule(radius: f32, height: f32, segments: u32) -> MeshData {
        // Simplified capsule - cylinder with hemispheres
        Self::generate_cylinder(radius, height, segments)
    }

    fn generate_plane(width: f32, depth: f32) -> MeshData {
        let hw = width / 2.0;
        let hd = depth / 2.0;
        
        let vertices = vec![
            ModelVertex { position: [-hw, 0.0, -hd], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            ModelVertex { position: [hw, 0.0, -hd], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            ModelVertex { position: [hw, 0.0, hd], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            ModelVertex { position: [-hw, 0.0, hd], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
        ];
        
        let indices = vec![0, 2, 1, 0, 3, 2];
        
        MeshData { vertices, indices }
    }

    fn generate_wedge(width: f32, height: f32, depth: f32) -> MeshData {
        let hw = width / 2.0;
        let hd = depth / 2.0;
        
        let vertices = vec![
            // Bottom
            ModelVertex { position: [-hw, 0.0, -hd], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            ModelVertex { position: [hw, 0.0, -hd], normal: [0.0, -1.0, 0.0], uv: [1.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            ModelVertex { position: [hw, 0.0, hd], normal: [0.0, -1.0, 0.0], uv: [1.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            ModelVertex { position: [-hw, 0.0, hd], normal: [0.0, -1.0, 0.0], uv: [0.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            // Back (vertical)
            ModelVertex { position: [-hw, 0.0, -hd], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            ModelVertex { position: [hw, 0.0, -hd], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            ModelVertex { position: [hw, height, -hd], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            ModelVertex { position: [-hw, height, -hd], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            // Slope
            ModelVertex { position: [-hw, height, -hd], normal: [0.0, 0.5, 0.5], uv: [0.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            ModelVertex { position: [hw, height, -hd], normal: [0.0, 0.5, 0.5], uv: [1.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            ModelVertex { position: [hw, 0.0, hd], normal: [0.0, 0.5, 0.5], uv: [1.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            ModelVertex { position: [-hw, 0.0, hd], normal: [0.0, 0.5, 0.5], uv: [0.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] },
            // Left
            ModelVertex { position: [-hw, 0.0, hd], normal: [-1.0, 0.0, 0.0], uv: [0.0, 1.0], tangent: [0.0, 0.0, 1.0, 1.0] },
            ModelVertex { position: [-hw, 0.0, -hd], normal: [-1.0, 0.0, 0.0], uv: [1.0, 1.0], tangent: [0.0, 0.0, 1.0, 1.0] },
            ModelVertex { position: [-hw, height, -hd], normal: [-1.0, 0.0, 0.0], uv: [1.0, 0.0], tangent: [0.0, 0.0, 1.0, 1.0] },
            // Right
            ModelVertex { position: [hw, 0.0, -hd], normal: [1.0, 0.0, 0.0], uv: [0.0, 1.0], tangent: [0.0, 0.0, -1.0, 1.0] },
            ModelVertex { position: [hw, 0.0, hd], normal: [1.0, 0.0, 0.0], uv: [1.0, 1.0], tangent: [0.0, 0.0, -1.0, 1.0] },
            ModelVertex { position: [hw, height, -hd], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0], tangent: [0.0, 0.0, -1.0, 1.0] },
        ];
        
        let indices = vec![
            0, 1, 2, 0, 2, 3,           // Bottom
            4, 6, 5, 4, 7, 6,           // Back
            8, 10, 9, 8, 11, 10,        // Slope
            12, 14, 13,                  // Left tri
            15, 17, 16,                  // Right tri
        ];
        
        MeshData { vertices, indices }
    }

    fn generate_arch(width: f32, height: f32, depth: f32, segments: u32) -> MeshData {
        // Simplified arch
        Self::generate_box(width, height, depth)
    }

    fn generate_stairs(width: f32, height: f32, depth: f32, steps: u32) -> MeshData {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        let step_height = height / steps as f32;
        let step_depth = depth / steps as f32;
        let hw = width / 2.0;
        
        for i in 0..steps {
            let y = i as f32 * step_height;
            let z = i as f32 * step_depth - depth / 2.0;
            let base = vertices.len() as u32;
            
            // Top of step
            vertices.push(ModelVertex { position: [-hw, y + step_height, z], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] });
            vertices.push(ModelVertex { position: [hw, y + step_height, z], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] });
            vertices.push(ModelVertex { position: [hw, y + step_height, z + step_depth], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] });
            vertices.push(ModelVertex { position: [-hw, y + step_height, z + step_depth], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] });
            
            // Front of step
            vertices.push(ModelVertex { position: [-hw, y, z], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] });
            vertices.push(ModelVertex { position: [hw, y, z], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0], tangent: [1.0, 0.0, 0.0, 1.0] });
            vertices.push(ModelVertex { position: [hw, y + step_height, z], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] });
            vertices.push(ModelVertex { position: [-hw, y + step_height, z], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0], tangent: [1.0, 0.0, 0.0, 1.0] });
            
            indices.extend_from_slice(&[
                base, base + 2, base + 1, base, base + 3, base + 2,
                base + 4, base + 6, base + 5, base + 4, base + 7, base + 6,
            ]);
        }
        
        MeshData { vertices, indices }
    }

    fn generate_crate(size: f32) -> MeshData {
        Self::generate_box(size, size, size)
    }

    fn generate_barrel(radius: f32, height: f32) -> MeshData {
        Self::generate_cylinder(radius, height, 16)
    }
}
