// Scene/Module Manager for platforms, stairs, ramps, toys, and other objects
// Supports Sparse/Medium/Dense patterns
use glam::{Vec3, Mat4};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// Scene density pattern
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ScenePattern {
    Sparse,   // Few objects, large spacing
    Medium,   // Moderate object density
    Dense,    // Many objects, tight spacing
}

/// Scene object types
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObjectType {
    Platform,
    Stairs,
    SlideRamp,
    Toy,
    Obstacle,
}

/// Scene object component
#[derive(Component, Clone)]
pub struct SceneObject {
    pub object_type: ObjectType,
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub transform: Mat4,
}

impl SceneObject {
    pub fn new(object_type: ObjectType, position: Vec3) -> Self {
        let scale = match object_type {
            ObjectType::Platform => Vec3::new(5.0, 0.5, 5.0),
            ObjectType::Stairs => Vec3::new(2.0, 1.0, 3.0),
            ObjectType::SlideRamp => Vec3::new(3.0, 1.0, 5.0),
            ObjectType::Toy => Vec3::new(0.5, 0.5, 0.5),
            ObjectType::Obstacle => Vec3::new(1.0, 2.0, 1.0),
        };

        let transform = Mat4::from_translation(position) * 
                       Mat4::from_scale(scale);

        Self {
            object_type,
            position,
            rotation: Vec3::ZERO,
            scale,
            transform,
        }
    }

    pub fn update_transform(&mut self) {
        self.transform = Mat4::from_translation(self.position) *
                        Mat4::from_euler(glam::EulerRot::XYZ, 
                                        self.rotation.x, 
                                        self.rotation.y, 
                                        self.rotation.z) *
                        Mat4::from_scale(self.scale);
    }
}

/// Scene Manager for managing scene objects and patterns
pub struct SceneManager {
    pub pattern: ScenePattern,
    pub objects: Vec<Entity>,
    pub world: World,
}

impl SceneManager {
    pub fn new(pattern: ScenePattern) -> Self {
        let world = World::new();
        let mut manager = Self {
            pattern,
            objects: Vec::new(),
            world,
        };

        // Generate scene based on pattern
        manager.generate_scene();

        manager
    }

    /// Generate scene objects based on pattern
    pub fn generate_scene(&mut self) {
        self.objects.clear();

        let (count, spacing) = match self.pattern {
            ScenePattern::Sparse => (10, 15.0),
            ScenePattern::Medium => (30, 8.0),
            ScenePattern::Dense => (100, 4.0),
        };

        // Generate platforms
        for i in 0..count {
            let x = (i as f32 % 10.0) * spacing - (count as f32 * spacing / 2.0);
            let z = (i as f32 / 10.0).floor() * spacing - (count as f32 * spacing / 2.0);
            let entity = self.world.spawn(SceneObject::new(
                ObjectType::Platform,
                Vec3::new(x, 0.0, z),
            )).id();
            self.objects.push(entity);
        }

        // Generate stairs
        for i in 0..(count / 5) {
            let x = (i as f32) * spacing * 2.0;
            let entity = self.world.spawn(SceneObject::new(
                ObjectType::Stairs,
                Vec3::new(x, 1.0, 0.0),
            )).id();
            self.objects.push(entity);
        }

        // Generate slide ramps
        for i in 0..(count / 10) {
            let x = (i as f32) * spacing * 3.0;
            let entity = self.world.spawn(SceneObject::new(
                ObjectType::SlideRamp,
                Vec3::new(x, 2.0, 5.0),
            )).id();
            self.objects.push(entity);
        }

        // Generate toys
        for i in 0..(count / 3) {
            let x = (i as f32 % 5.0) * spacing * 1.5;
            let z = (i as f32 / 5.0).floor() * spacing * 1.5;
            let entity = self.world.spawn(SceneObject::new(
                ObjectType::Toy,
                Vec3::new(x, 1.0, z),
            )).id();
            self.objects.push(entity);
        }
    }

    /// Change scene pattern
    pub fn set_pattern(&mut self, pattern: ScenePattern) {
        if self.pattern != pattern {
            self.pattern = pattern;
            self.generate_scene();
        }
    }

    /// Get all objects
    pub fn get_objects(&self) -> &[Entity] {
        &self.objects
    }

    /// Check collision with scene objects
    pub fn check_collision(&self, position: Vec3, radius: f32) -> bool {
        for &entity in &self.objects {
            if let Some(obj) = self.world.get::<SceneObject>(entity) {
                let distance = (position - obj.position).length();
                let obj_radius = obj.scale.length() * 0.5;
                if distance < radius + obj_radius {
                    return true;
                }
            }
        }
        false
    }

    /// Get objects in range
    pub fn get_objects_in_range(&self, position: Vec3, range: f32) -> Vec<Entity> {
        self.objects
            .iter()
            .filter_map(|&entity| {
                if let Some(obj) = self.world.get::<SceneObject>(entity) {
                    if (position - obj.position).length() < range {
                        Some(entity)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new(ScenePattern::Medium)
    }
}
