use crate::consciousness::{ConsciousnessAction, ConsciousnessCore, ConsciousnessPulse};
use crate::evolution::EvolutionEngine;
use crate::lighting::LightingSystem;
use crate::renderer::InstanceRaw;
use crate::voxel::{VoxelWorld, VoxelWorldConfig, WorldMetrics};

#[derive(Clone)]
pub struct SimulationMetrics {
    pub world: WorldMetrics,
    pub pulse: ConsciousnessPulse,
    pub instance_count: usize,
}

pub struct Simulation {
    pub world: VoxelWorld,
    lighting: LightingSystem,
    evolution: EvolutionEngine,
    consciousness: ConsciousnessCore,
    instances: Vec<InstanceRaw>,
    metrics: WorldMetrics,
    last_pulse: ConsciousnessPulse,
    evolution_timer: f32,
}

impl Simulation {
    pub fn new(seed: u64) -> Self {
        let config = VoxelWorldConfig::default();
        let world = VoxelWorld::new(config, seed);
        let metrics = world.metrics();
        let mut lighting = LightingSystem::new();
        lighting.add_pattern(Default::default());
        let mut consciousness = ConsciousnessCore::new("VOXELCRAI", seed ^ 0x5eed_c0de);
        let pulse = consciousness.think(&metrics, 0.0);

        Self {
            world,
            lighting,
            evolution: EvolutionEngine::new(),
            consciousness,
            instances: Vec::new(),
            metrics,
            last_pulse: pulse,
            evolution_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.world.update(dt);
        self.lighting.update_lighting(self.world.time);
        self.evolution_timer += dt;

        let metrics = self.world.metrics();
        self.metrics = metrics.clone();
        self.last_pulse = self.consciousness.think(&metrics, dt);
        let actions = self.last_pulse.actions.clone();
        self.apply_actions(&actions);

        if self.evolution_timer >= 2.0 {
            self.evolution.evolve(self.world.voxels_mut());
            self.evolution_timer = 0.0;
        }

        self.rebuild_instances(&metrics);
    }

    fn apply_actions(&mut self, actions: &[ConsciousnessAction]) {
        for action in actions {
            match action {
                ConsciousnessAction::IgniteCluster { center, gain } => {
                    self.world
                        .affect_cluster(*center, 8.0 + *gain * 4.0, *gain as f64);
                }
                ConsciousnessAction::CalmCluster { center, falloff } => {
                    self.world
                        .affect_cluster(*center, 6.0 + *falloff * 4.0, -(*falloff as f64));
                }
                ConsciousnessAction::ToggleTrauma(value) => {
                    self.world.trauma_mode = *value;
                }
                ConsciousnessAction::SeedConcept { concept } => {
                    self.world.embed_concept(concept.clone());
                }
            }
        }
    }

    fn rebuild_instances(&mut self, metrics: &WorldMetrics) {
        self.instances.clear();
        let max_energy = metrics.max_energy.max(1.0);

        for voxel in self.world.voxels() {
            let color = voxel.color_by_energy(max_energy);
            self.instances.push(InstanceRaw {
                position: [
                    voxel.position[0] as f32,
                    voxel.position[1] as f32,
                    voxel.position[2] as f32,
                ],
                scale: 0.9,
                color,
                energy: voxel.energy as f32,
            });
        }
    }

    pub fn instances(&self) -> &[InstanceRaw] {
        &self.instances
    }

    pub fn telemetry(&self) -> SimulationMetrics {
        SimulationMetrics {
            world: self.metrics.clone(),
            pulse: self.last_pulse.clone(),
            instance_count: self.instances.len(),
        }
    }
}
