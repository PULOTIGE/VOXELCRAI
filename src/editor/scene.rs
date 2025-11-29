//! Scene management - objects, hierarchy, serialization

use glam::{Vec3, Quat, Mat4};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique object identifier
pub type ObjectId = Uuid;

/// Scene containing all objects
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub objects: HashMap<ObjectId, SceneObject>,
    pub hierarchy: Vec<ObjectId>,  // Root objects
    pub spawn_points_t: Vec<Vec3>,
    pub spawn_points_ct: Vec<Vec3>,
    pub bombsite_a: Option<ObjectId>,
    pub bombsite_b: Option<ObjectId>,
    
    #[serde(skip)]
    pub selected: Vec<ObjectId>,
}

impl Scene {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            objects: HashMap::new(),
            hierarchy: Vec::new(),
            spawn_points_t: Vec::new(),
            spawn_points_ct: Vec::new(),
            bombsite_a: None,
            bombsite_b: None,
            selected: Vec::new(),
        }
    }

    /// Create a default scene with ground plane
    pub fn default_scene() -> Self {
        let mut scene = Self::new("Main");
        
        // Add ground plane
        let ground = SceneObject::new("Ground", ObjectType::Primitive(PrimitiveType::Box))
            .with_position(Vec3::new(0.0, -0.5, 0.0))
            .with_scale(Vec3::new(100.0, 1.0, 100.0))
            .with_material(MaterialRef::Builtin("concrete".to_string()));
        scene.add_object(ground);
        
        // Add directional light
        let sun = SceneObject::new("Sun", ObjectType::Light(LightType::Directional))
            .with_rotation(Quat::from_euler(glam::EulerRot::XYZ, -0.8, -0.3, 0.0));
        scene.add_object(sun);
        
        // Add player spawn
        let spawn = SceneObject::new("Spawn_T_1", ObjectType::SpawnPoint(TeamType::Terrorist))
            .with_position(Vec3::new(-20.0, 1.0, 0.0));
        scene.add_object(spawn);
        
        let spawn_ct = SceneObject::new("Spawn_CT_1", ObjectType::SpawnPoint(TeamType::CounterTerrorist))
            .with_position(Vec3::new(20.0, 1.0, 0.0));
        scene.add_object(spawn_ct);
        
        scene
    }

    /// Add object to scene
    pub fn add_object(&mut self, object: SceneObject) -> ObjectId {
        let id = object.id;
        self.hierarchy.push(id);
        self.objects.insert(id, object);
        id
    }

    /// Remove object from scene
    pub fn remove_object(&mut self, id: ObjectId) -> Option<SceneObject> {
        self.hierarchy.retain(|&oid| oid != id);
        self.selected.retain(|&oid| oid != id);
        
        // Remove children recursively
        if let Some(obj) = self.objects.get(&id) {
            let children = obj.children.clone();
            for child_id in children {
                self.remove_object(child_id);
            }
        }
        
        self.objects.remove(&id)
    }

    /// Get object by ID
    pub fn get_object(&self, id: ObjectId) -> Option<&SceneObject> {
        self.objects.get(&id)
    }

    /// Get mutable object by ID
    pub fn get_object_mut(&mut self, id: ObjectId) -> Option<&mut SceneObject> {
        self.objects.get_mut(&id)
    }

    /// Duplicate object
    pub fn duplicate_object(&mut self, id: ObjectId) -> Option<ObjectId> {
        if let Some(original) = self.objects.get(&id).cloned() {
            let mut copy = original.clone();
            copy.id = Uuid::new_v4();
            copy.name = format!("{}_copy", original.name);
            copy.position.x += 2.0; // Offset copy
            
            let new_id = copy.id;
            self.add_object(copy);
            Some(new_id)
        } else {
            None
        }
    }

    /// Select object
    pub fn select(&mut self, id: ObjectId, add_to_selection: bool) {
        if !add_to_selection {
            self.selected.clear();
        }
        if !self.selected.contains(&id) {
            self.selected.push(id);
        }
    }

    /// Deselect all
    pub fn deselect_all(&mut self) {
        self.selected.clear();
    }

    /// Get all objects of specific type
    pub fn objects_of_type(&self, obj_type: &ObjectType) -> Vec<ObjectId> {
        self.objects
            .iter()
            .filter(|(_, obj)| std::mem::discriminant(&obj.object_type) == std::mem::discriminant(obj_type))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Update spawn points from scene objects
    pub fn update_spawn_points(&mut self) {
        self.spawn_points_t.clear();
        self.spawn_points_ct.clear();
        
        for obj in self.objects.values() {
            match &obj.object_type {
                ObjectType::SpawnPoint(TeamType::Terrorist) => {
                    self.spawn_points_t.push(obj.position);
                }
                ObjectType::SpawnPoint(TeamType::CounterTerrorist) => {
                    self.spawn_points_ct.push(obj.position);
                }
                _ => {}
            }
        }
    }

    /// Save scene to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Load scene from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::default_scene()
    }
}

/// Scene object
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneObject {
    pub id: ObjectId,
    pub name: String,
    pub object_type: ObjectType,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub visible: bool,
    pub locked: bool,
    pub parent: Option<ObjectId>,
    pub children: Vec<ObjectId>,
    pub material: MaterialRef,
    pub properties: HashMap<String, PropertyValue>,
    pub tags: Vec<String>,
}

impl SceneObject {
    pub fn new(name: &str, object_type: ObjectType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            object_type,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            visible: true,
            locked: false,
            parent: None,
            children: Vec::new(),
            material: MaterialRef::Default,
            properties: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_material(mut self, material: MaterialRef) -> Self {
        self.material = material;
        self
    }

    /// Get world transform matrix
    pub fn transform(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Get euler angles in degrees
    pub fn euler_degrees(&self) -> Vec3 {
        let (x, y, z) = self.rotation.to_euler(glam::EulerRot::XYZ);
        Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees())
    }

    /// Set rotation from euler angles in degrees
    pub fn set_euler_degrees(&mut self, euler: Vec3) {
        self.rotation = Quat::from_euler(
            glam::EulerRot::XYZ,
            euler.x.to_radians(),
            euler.y.to_radians(),
            euler.z.to_radians(),
        );
    }
}

/// Object types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ObjectType {
    // Geometry
    Primitive(PrimitiveType),
    Model(String),  // Asset path
    
    // Environment
    Light(LightType),
    Reflection,
    
    // Gameplay
    SpawnPoint(TeamType),
    Bombsite,
    HostageSpawn,
    BuyZone(TeamType),
    
    // Triggers
    Trigger(TriggerType),
    
    // Audio
    SoundEmitter,
    
    // Effects
    ParticleEmitter,
    Decal,
    
    // Grouping
    Group,
    Prefab(String),
}

/// Primitive shapes
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveType {
    Box,
    Sphere,
    Cylinder,
    Capsule,
    Plane,
    Wedge,
    Arch,
    Stairs,
}

/// Light types
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum LightType {
    Directional,
    Point,
    Spot,
    Area,
}

/// Team types
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TeamType {
    Terrorist,
    CounterTerrorist,
}

/// Trigger types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TriggerType {
    Teleport(Vec3),
    Hurt(f32),
    Push(Vec3),
    Script(String),
}

/// Material reference
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MaterialRef {
    Default,
    Builtin(String),
    Asset(String),
    Custom(CustomMaterial),
}

/// Custom material settings
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CustomMaterial {
    pub albedo: [f32; 3],
    pub metallic: f32,
    pub roughness: f32,
    pub emission: [f32; 3],
    pub emission_strength: f32,
}

impl Default for CustomMaterial {
    fn default() -> Self {
        Self {
            albedo: [0.8, 0.8, 0.8],
            metallic: 0.0,
            roughness: 0.5,
            emission: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
        }
    }
}

/// Property value types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Color([f32; 4]),
    Asset(String),
}
