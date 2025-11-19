use glam::{IVec3, Vec3};
use half::f16;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

/// Voxel component: 9-13 KB per voxel
#[derive(Clone)]
pub struct Voxel {
    pub energy: f64,
    pub emotion_valence: f64,
    pub emotion_arousal: f64,
    pub emotion_dominance: f64,
    pub perception_visual: f16,
    pub perception_auditory: f16,
    pub perception_tactile: f16,
    pub perception_thermal: f16,
    pub perception_chemical: f16,
    pub perception_pressure: f16,
    pub perception_time: f16,
    pub perception_space: f16,
    pub perception_self: f16,
    pub perception_other: f16,
    pub velocity_x: i8,
    pub velocity_y: i8,
    pub velocity_z: i8,
    pub acceleration_x: i8,
    pub acceleration_y: i8,
    pub acceleration_z: i8,
    pub temperature: i8,
    pub pressure: i8,
    pub density: i8,
    pub elasticity: i8,
    pub friction: i8,
    pub viscosity: i8,
    pub state_flags: u8,
    pub material_flags: u8,
    pub genome: Genome,
    pub echo: [u8; 16],
    pub resonance: f16,
    pub position: [i32; 3],
    pub metadata: HashMap<String, String>,
}

impl Voxel {
    pub fn new(position: [i32; 3]) -> Self {
        Self {
            energy: 0.5,
            emotion_valence: 0.0,
            emotion_arousal: 0.0,
            emotion_dominance: 0.0,
            perception_visual: f16::from_f32(0.5),
            perception_auditory: f16::from_f32(0.5),
            perception_tactile: f16::from_f32(0.5),
            perception_thermal: f16::from_f32(0.5),
            perception_chemical: f16::from_f32(0.5),
            perception_pressure: f16::from_f32(0.5),
            perception_time: f16::from_f32(0.5),
            perception_space: f16::from_f32(0.5),
            perception_self: f16::from_f32(0.5),
            perception_other: f16::from_f32(0.5),
            velocity_x: 0,
            velocity_y: 0,
            velocity_z: 0,
            acceleration_x: 0,
            acceleration_y: 0,
            acceleration_z: 0,
            temperature: 0,
            pressure: 0,
            density: 0,
            elasticity: 0,
            friction: 0,
            viscosity: 0,
            state_flags: 0,
            material_flags: 0,
            genome: Genome::new(),
            echo: [0; 16],
            resonance: f16::from_f32(0.5),
            position,
            metadata: HashMap::new(),
        }
    }

    pub fn size_bytes(&self) -> usize {
        let base = std::mem::size_of::<Self>();
        let genome_size = self.genome.size_bytes();
        let metadata_size: usize = self
            .metadata
            .iter()
            .map(|(k, v)| k.len() + v.len() + 16)
            .sum();
        base + genome_size + metadata_size
    }

    pub fn color_by_energy(&self, max_energy: f64) -> [f32; 3] {
        let normalized = (self.energy / max_energy.max(0.001)).clamp(0.0, 1.0) as f32;
        let hue = normalized * 0.8;
        [
            (hue * 1.2).min(1.0),
            normalized.powf(0.8),
            (1.0 - normalized).powf(1.2),
        ]
    }
}

/// Genome: up to 10 concepts (strings)
#[derive(Clone)]
pub struct Genome {
    pub concepts: Vec<String>,
    pub max_concepts: usize,
}

impl Genome {
    pub fn new() -> Self {
        Self {
            concepts: Vec::new(),
            max_concepts: 10,
        }
    }

    pub fn size_bytes(&self) -> usize {
        self.concepts.iter().map(|s| s.len() + 8).sum::<usize>() + 16
    }

    pub fn add_concept(&mut self, concept: String) -> bool {
        if self.concepts.len() < self.max_concepts {
            self.concepts.push(concept);
            true
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct VoxelWorldConfig {
    pub extent: IVec3,
    pub base_height: i32,
    pub height_variance: f32,
    pub noise_scale: f32,
    pub max_voxels: usize,
}

impl Default for VoxelWorldConfig {
    fn default() -> Self {
        Self {
            extent: IVec3::new(48, 24, 48),
            base_height: -5,
            height_variance: 18.0,
            noise_scale: 0.12,
            max_voxels: 100_000,
        }
    }
}

#[derive(Clone)]
pub struct WorldMetrics {
    pub voxel_count: usize,
    pub avg_energy: f64,
    pub max_energy: f64,
    pub centroid: Vec3,
    pub hot_spot: Vec3,
    pub cold_spot: Vec3,
    pub entropy: f32,
    pub trauma_mode: bool,
    pub trauma_level: f32,
    pub time: f32,
}

pub struct VoxelWorld {
    voxels: Vec<Voxel>,
    config: VoxelWorldConfig,
    rng: StdRng,
    pub trauma_mode: bool,
    pub time: f32,
}

impl VoxelWorld {
    pub fn new(config: VoxelWorldConfig, seed: u64) -> Self {
        let mut world = Self {
            voxels: Vec::new(),
            config,
            rng: StdRng::seed_from_u64(seed),
            trauma_mode: false,
            time: 0.0,
        };
        world.regenerate();
        world
    }

    pub fn regenerate(&mut self) {
        self.voxels.clear();
        let IVec3 { x, y, z } = self.config.extent;
        for ix in -x..=x {
            for iz in -z..=z {
                let height = self.sample_height(ix, iz).clamp(-y, y);
                for iy in -y..=height {
                    if self.voxels.len() >= self.config.max_voxels {
                        return;
                    }
                    let mut voxel = Voxel::new([ix, iy, iz]);
                    voxel.energy = self.rng.gen_range(0.2..0.8) as f64;
                    voxel.resonance = f16::from_f32(self.rng.gen_range(0.2..0.9));
                    voxel
                        .metadata
                        .insert("origin".to_string(), format!("seed:{}:{}", ix, iz));
                    self.voxels.push(voxel);
                }
            }
        }
    }

    fn sample_height(&self, x: i32, z: i32) -> i32 {
        let fx = x as f32 * self.config.noise_scale;
        let fz = z as f32 * self.config.noise_scale;
        let sine = (fx.sin() * 0.6 + fz.cos() * 0.4) * self.config.height_variance;
        let ridge = ((fx * 0.7 + fz * 0.25).sin() * 0.5 + (fx * 1.7).cos() * 0.25)
            * self.config.height_variance
            * 0.5;
        self.config.base_height + (sine + ridge) as i32
    }

    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
        let trauma_multiplier = if self.trauma_mode { 1.35 } else { 1.0 };

        for voxel in &mut self.voxels {
            let resonance = voxel.resonance.to_f32();
            let oscillation = ((voxel.position[0] as f32 * 0.09)
                + (voxel.position[2] as f32 * 0.04)
                + self.time * 0.7)
                .sin();

            voxel.energy +=
                (resonance as f64 * delta_time as f64 * 0.8) + (oscillation as f64 * 0.015);
            voxel.energy = (voxel.energy * trauma_multiplier as f64).clamp(0.0, 12.0);

            voxel.emotion_valence = (voxel.emotion_valence * 0.98) + oscillation as f64 * 0.02;
            voxel.emotion_arousal =
                (voxel.emotion_arousal * 0.97) + resonance as f64 * 0.03 * trauma_multiplier as f64;
            voxel.emotion_dominance =
                (voxel.emotion_dominance * 0.99) + (voxel.energy - 0.5) * 0.01;

            voxel.velocity_x =
                (voxel.velocity_x as f32 * 0.95 + oscillation * 5.0).clamp(-120.0, 120.0) as i8;
            voxel.velocity_y =
                (voxel.velocity_y as f32 * 0.95 + resonance * 4.0).clamp(-120.0, 120.0) as i8;
            voxel.velocity_z = (voxel.velocity_z as f32 * 0.95 + oscillation.cos() * 5.0)
                .clamp(-120.0, 120.0) as i8;
        }
    }

    pub fn voxels(&self) -> &[Voxel] {
        &self.voxels
    }

    pub fn voxels_mut(&mut self) -> &mut [Voxel] {
        &mut self.voxels
    }

    pub fn spawn_voxel(&mut self, position: [i32; 3], energy: f64) {
        if self.voxels.len() >= self.config.max_voxels {
            return;
        }
        let mut voxel = Voxel::new(position);
        voxel.energy = energy;
        voxel.genome.add_concept("emergent".into());
        self.voxels.push(voxel);
    }

    pub fn affect_cluster(&mut self, center: Vec3, radius: f32, delta: f64) {
        let radius_sq = radius * radius;
        for voxel in &mut self.voxels {
            let pos = Vec3::new(
                voxel.position[0] as f32,
                voxel.position[1] as f32,
                voxel.position[2] as f32,
            );
            let dist_sq = pos.distance_squared(center);
            if dist_sq <= radius_sq {
                let influence = 1.0 - (dist_sq.sqrt() / radius).min(1.0);
                voxel.energy = (voxel.energy + delta * influence as f64).clamp(0.0, 12.0);
                voxel.resonance =
                    f16::from_f32((voxel.resonance.to_f32() + delta as f32 * 0.05).clamp(0.0, 1.5));
            }
        }
    }

    pub fn embed_concept(&mut self, concept: String) {
        if let Some(voxel) = self.voxels.iter_mut().max_by(|a, b| {
            a.energy
                .partial_cmp(&b.energy)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            let added = voxel.genome.add_concept(concept.clone());
            if added {
                voxel.metadata.insert("concept-injection".into(), concept);
            }
        }
    }

    pub fn metrics(&self) -> WorldMetrics {
        let voxel_count = self.voxels.len();
        if voxel_count == 0 {
            return WorldMetrics {
                voxel_count: 0,
                avg_energy: 0.0,
                max_energy: 0.0,
                centroid: Vec3::ZERO,
                hot_spot: Vec3::ZERO,
                cold_spot: Vec3::ZERO,
                entropy: 0.0,
                trauma_mode: self.trauma_mode,
                trauma_level: if self.trauma_mode { 1.0 } else { 0.0 },
                time: self.time,
            };
        }

        let mut sum_energy = 0.0;
        let mut max_energy = f64::MIN;
        let mut min_energy = f64::MAX;
        let mut centroid = Vec3::ZERO;
        let mut hot_spot = Vec3::ZERO;
        let mut cold_spot = Vec3::ZERO;
        let mut buckets = [0usize; 8];

        for voxel in &self.voxels {
            sum_energy += voxel.energy;
            let pos = Vec3::new(
                voxel.position[0] as f32,
                voxel.position[1] as f32,
                voxel.position[2] as f32,
            );
            centroid += pos;

            if voxel.energy > max_energy {
                max_energy = voxel.energy;
                hot_spot = pos;
            }
            if voxel.energy < min_energy {
                min_energy = voxel.energy;
                cold_spot = pos;
            }

            let bucket_index = ((voxel.energy / 1.5).floor() as usize).min(buckets.len() - 1);
            buckets[bucket_index] += 1;
        }

        centroid /= voxel_count as f32;
        let avg_energy = sum_energy / voxel_count as f64;

        let entropy = buckets.iter().fold(0.0, |acc, &count| {
            if count == 0 {
                acc
            } else {
                let p = count as f32 / voxel_count as f32;
                acc - p * p.log2()
            }
        }) / (buckets.len() as f32).log2();

        WorldMetrics {
            voxel_count,
            avg_energy,
            max_energy,
            centroid,
            hot_spot,
            cold_spot,
            entropy: entropy.clamp(0.0, 1.0),
            trauma_mode: self.trauma_mode,
            trauma_level: if self.trauma_mode { 1.0 } else { 0.0 },
            time: self.time,
        }
    }
}

impl Default for VoxelWorld {
    fn default() -> Self {
        Self::new(VoxelWorldConfig::default(), 42)
    }
}
