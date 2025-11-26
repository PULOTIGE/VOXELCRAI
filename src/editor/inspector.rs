//! Inspector panel - object properties editor

use glam::{Vec3, Quat};
use super::scene::{SceneObject, ObjectType, ObjectId, MaterialRef, CustomMaterial, PrimitiveType, LightType, TeamType};
use super::gameplay::{WeaponConfig, WeaponType};

/// Inspector state
pub struct Inspector {
    pub selected_tab: InspectorTab,
    pub transform_expanded: bool,
    pub material_expanded: bool,
    pub properties_expanded: bool,
    pub components_expanded: bool,
}

impl Inspector {
    pub fn new() -> Self {
        Self {
            selected_tab: InspectorTab::Object,
            transform_expanded: true,
            material_expanded: true,
            properties_expanded: true,
            components_expanded: true,
        }
    }

    /// Get property sections for object
    pub fn get_sections(&self, object: &SceneObject) -> Vec<InspectorSection> {
        let mut sections = vec![
            // Transform section
            InspectorSection {
                name: "Transform".to_string(),
                expanded: self.transform_expanded,
                properties: vec![
                    Property::Vec3 {
                        name: "Position".to_string(),
                        value: object.position,
                        step: 0.1,
                    },
                    Property::Vec3 {
                        name: "Rotation".to_string(),
                        value: object.euler_degrees(),
                        step: 1.0,
                    },
                    Property::Vec3 {
                        name: "Scale".to_string(),
                        value: object.scale,
                        step: 0.1,
                    },
                ],
            },
        ];

        // Object-type specific properties
        match &object.object_type {
            ObjectType::Primitive(prim) => {
                sections.push(self.primitive_section(prim));
            }
            ObjectType::Light(light_type) => {
                sections.push(self.light_section(light_type));
            }
            ObjectType::SpawnPoint(team) => {
                sections.push(self.spawn_section(team));
            }
            ObjectType::Model(path) => {
                sections.push(InspectorSection {
                    name: "Model".to_string(),
                    expanded: true,
                    properties: vec![
                        Property::String {
                            name: "Path".to_string(),
                            value: path.clone(),
                        },
                    ],
                });
            }
            _ => {}
        }

        // Material section
        sections.push(self.material_section(&object.material));

        // Common properties
        sections.push(InspectorSection {
            name: "Object".to_string(),
            expanded: true,
            properties: vec![
                Property::String {
                    name: "Name".to_string(),
                    value: object.name.clone(),
                },
                Property::Bool {
                    name: "Visible".to_string(),
                    value: object.visible,
                },
                Property::Bool {
                    name: "Locked".to_string(),
                    value: object.locked,
                },
                Property::Tags {
                    name: "Tags".to_string(),
                    values: object.tags.clone(),
                },
            ],
        });

        sections
    }

    fn primitive_section(&self, prim: &PrimitiveType) -> InspectorSection {
        InspectorSection {
            name: "Primitive".to_string(),
            expanded: true,
            properties: vec![
                Property::Enum {
                    name: "Type".to_string(),
                    value: format!("{:?}", prim),
                    options: vec![
                        "Box".to_string(),
                        "Sphere".to_string(),
                        "Cylinder".to_string(),
                        "Capsule".to_string(),
                        "Plane".to_string(),
                        "Wedge".to_string(),
                        "Arch".to_string(),
                        "Stairs".to_string(),
                    ],
                },
            ],
        }
    }

    fn light_section(&self, light_type: &LightType) -> InspectorSection {
        let mut properties = vec![
            Property::Enum {
                name: "Type".to_string(),
                value: format!("{:?}", light_type),
                options: vec![
                    "Directional".to_string(),
                    "Point".to_string(),
                    "Spot".to_string(),
                    "Area".to_string(),
                ],
            },
            Property::Color {
                name: "Color".to_string(),
                value: [1.0, 1.0, 0.9, 1.0],
            },
            Property::Float {
                name: "Intensity".to_string(),
                value: 1.0,
                min: 0.0,
                max: 100.0,
                step: 0.1,
            },
            Property::Bool {
                name: "Cast Shadows".to_string(),
                value: true,
            },
        ];

        match light_type {
            LightType::Point => {
                properties.push(Property::Float {
                    name: "Range".to_string(),
                    value: 10.0,
                    min: 0.1,
                    max: 1000.0,
                    step: 0.5,
                });
            }
            LightType::Spot => {
                properties.push(Property::Float {
                    name: "Range".to_string(),
                    value: 10.0,
                    min: 0.1,
                    max: 1000.0,
                    step: 0.5,
                });
                properties.push(Property::Float {
                    name: "Inner Angle".to_string(),
                    value: 25.0,
                    min: 0.0,
                    max: 90.0,
                    step: 1.0,
                });
                properties.push(Property::Float {
                    name: "Outer Angle".to_string(),
                    value: 35.0,
                    min: 0.0,
                    max: 90.0,
                    step: 1.0,
                });
            }
            _ => {}
        }

        InspectorSection {
            name: "Light".to_string(),
            expanded: true,
            properties,
        }
    }

    fn spawn_section(&self, team: &TeamType) -> InspectorSection {
        InspectorSection {
            name: "Spawn Point".to_string(),
            expanded: true,
            properties: vec![
                Property::Enum {
                    name: "Team".to_string(),
                    value: format!("{:?}", team),
                    options: vec![
                        "Terrorist".to_string(),
                        "CounterTerrorist".to_string(),
                    ],
                },
                Property::Int {
                    name: "Priority".to_string(),
                    value: 0,
                    min: 0,
                    max: 99,
                },
            ],
        }
    }

    fn material_section(&self, material: &MaterialRef) -> InspectorSection {
        let properties = match material {
            MaterialRef::Default => {
                vec![
                    Property::Enum {
                        name: "Type".to_string(),
                        value: "Default".to_string(),
                        options: vec!["Default".to_string(), "Builtin".to_string(), "Asset".to_string(), "Custom".to_string()],
                    },
                ]
            }
            MaterialRef::Builtin(name) => {
                vec![
                    Property::Enum {
                        name: "Type".to_string(),
                        value: "Builtin".to_string(),
                        options: vec!["Default".to_string(), "Builtin".to_string(), "Asset".to_string(), "Custom".to_string()],
                    },
                    Property::String {
                        name: "Material".to_string(),
                        value: name.clone(),
                    },
                ]
            }
            MaterialRef::Asset(path) => {
                vec![
                    Property::Enum {
                        name: "Type".to_string(),
                        value: "Asset".to_string(),
                        options: vec!["Default".to_string(), "Builtin".to_string(), "Asset".to_string(), "Custom".to_string()],
                    },
                    Property::String {
                        name: "Path".to_string(),
                        value: path.clone(),
                    },
                ]
            }
            MaterialRef::Custom(mat) => {
                vec![
                    Property::Enum {
                        name: "Type".to_string(),
                        value: "Custom".to_string(),
                        options: vec!["Default".to_string(), "Builtin".to_string(), "Asset".to_string(), "Custom".to_string()],
                    },
                    Property::Color {
                        name: "Albedo".to_string(),
                        value: [mat.albedo[0], mat.albedo[1], mat.albedo[2], 1.0],
                    },
                    Property::Float {
                        name: "Metallic".to_string(),
                        value: mat.metallic,
                        min: 0.0,
                        max: 1.0,
                        step: 0.01,
                    },
                    Property::Float {
                        name: "Roughness".to_string(),
                        value: mat.roughness,
                        min: 0.0,
                        max: 1.0,
                        step: 0.01,
                    },
                    Property::Color {
                        name: "Emission".to_string(),
                        value: [mat.emission[0], mat.emission[1], mat.emission[2], 1.0],
                    },
                    Property::Float {
                        name: "Emission Strength".to_string(),
                        value: mat.emission_strength,
                        min: 0.0,
                        max: 100.0,
                        step: 0.1,
                    },
                ]
            }
        };

        InspectorSection {
            name: "Material".to_string(),
            expanded: self.material_expanded,
            properties,
        }
    }
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}

/// Inspector tabs
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InspectorTab {
    Object,
    Scene,
    Project,
}

/// Inspector section
#[derive(Clone, Debug)]
pub struct InspectorSection {
    pub name: String,
    pub expanded: bool,
    pub properties: Vec<Property>,
}

/// Property types
#[derive(Clone, Debug)]
pub enum Property {
    Bool {
        name: String,
        value: bool,
    },
    Int {
        name: String,
        value: i32,
        min: i32,
        max: i32,
    },
    Float {
        name: String,
        value: f32,
        min: f32,
        max: f32,
        step: f32,
    },
    String {
        name: String,
        value: String,
    },
    Vec2 {
        name: String,
        value: [f32; 2],
        step: f32,
    },
    Vec3 {
        name: String,
        value: Vec3,
        step: f32,
    },
    Vec4 {
        name: String,
        value: [f32; 4],
        step: f32,
    },
    Color {
        name: String,
        value: [f32; 4],
    },
    Enum {
        name: String,
        value: String,
        options: Vec<String>,
    },
    Asset {
        name: String,
        value: String,
        asset_type: String,
    },
    Tags {
        name: String,
        values: Vec<String>,
    },
}
