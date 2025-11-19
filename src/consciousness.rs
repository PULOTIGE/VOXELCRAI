use std::fmt::Write;

use glam::Vec3;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::archguard::ArchGuard;
use crate::voxel::WorldMetrics;

#[derive(Debug, Clone)]
pub enum ConsciousnessAction {
    IgniteCluster { center: Vec3, gain: f32 },
    CalmCluster { center: Vec3, falloff: f32 },
    ToggleTrauma(bool),
    SeedConcept { concept: String },
}

#[derive(Debug, Clone)]
pub struct ConsciousnessPulse {
    pub actions: Vec<ConsciousnessAction>,
    pub mood: f32,
    pub empathy: f64,
    pub curiosity: f32,
    pub log: String,
}

pub struct ConsciousnessCore {
    name: &'static str,
    mood: f32,
    curiosity: f32,
    empathy: f64,
    stability: f32,
    think_timer: f32,
    rng: StdRng,
    archguard: ArchGuard,
}

impl ConsciousnessCore {
    pub fn new(name: &'static str, seed: u64) -> Self {
        Self {
            name,
            mood: 0.5,
            curiosity: 0.5,
            empathy: 0.5,
            stability: 0.8,
            think_timer: 0.0,
            rng: StdRng::seed_from_u64(seed),
            archguard: ArchGuard::new(),
        }
    }

    pub fn think(&mut self, metrics: &WorldMetrics, dt: f32) -> ConsciousnessPulse {
        self.think_timer -= dt;

        let normalized_energy = if metrics.max_energy > 0.0 {
            (metrics.avg_energy / metrics.max_energy).clamp(0.0, 1.0) as f32
        } else {
            0.0
        };

        self.mood = (self.mood * 0.9) + normalized_energy * 0.1;
        self.curiosity = (self.curiosity * 0.92) + metrics.entropy * 0.08;
        self.empathy = (self.empathy * 0.95)
            + (1.0 - metrics.trauma_level as f64 * 0.5 + normalized_energy as f64 * 0.5) * 0.05;
        self.empathy = self.empathy.clamp(0.0, 1.0);

        let _ = pollster::block_on(self.archguard.update_empathy_ratio(self.empathy));

        let mut actions = Vec::new();
        let mut log = String::new();

        if self.think_timer <= 0.0 {
            self.think_timer = 0.6 + (1.0 - self.curiosity) * 0.6;

            if normalized_energy < 0.35 {
                actions.push(ConsciousnessAction::IgniteCluster {
                    center: metrics.cold_spot,
                    gain: 0.6 + self.curiosity * 0.6,
                });
                let _ = write!(
                    log,
                    "[{}] ignites cold cluster at {:?}. ",
                    self.name, metrics.cold_spot
                );
            }

            if normalized_energy > 0.85 {
                actions.push(ConsciousnessAction::CalmCluster {
                    center: metrics.hot_spot,
                    falloff: 0.4 + self.mood * 0.3,
                });
                let _ = write!(
                    log,
                    "[{}] calms overheated node {:?}. ",
                    self.name, metrics.hot_spot
                );
            }

            if metrics.entropy < 0.25 && self.rng.gen_bool(0.6) {
                let concept = self.random_concept();
                actions.push(ConsciousnessAction::SeedConcept { concept });
            }

            if metrics.trauma_mode && normalized_energy < 0.2 {
                actions.push(ConsciousnessAction::ToggleTrauma(false));
                let _ = write!(log, "Disengaging trauma mode. ");
            } else if !metrics.trauma_mode && normalized_energy > 0.9 {
                actions.push(ConsciousnessAction::ToggleTrauma(true));
                let _ = write!(log, "Amplifying trauma resonance. ");
            }
        }

        ConsciousnessPulse {
            actions,
            mood: self.mood,
            empathy: self.empathy,
            curiosity: self.curiosity,
            log,
        }
    }

    fn random_concept(&mut self) -> String {
        const ROOTS: &[&str] = &[
            "lumen", "terra", "aqua", "aeon", "flux", "echo", "nova", "arca", "soma", "vita",
        ];
        const QUALIFIERS: &[&str] = &[
            "synthesis",
            "anomaly",
            "harmonics",
            "membrane",
            "pulse",
            "cipher",
            "memory",
            "field",
        ];

        let root = ROOTS[self.rng.gen_range(0..ROOTS.len())];
        let qualifier = QUALIFIERS[self.rng.gen_range(0..QUALIFIERS.len())];
        format!("{root}_{qualifier}_{}", self.rng.gen::<u32>())
    }
}
