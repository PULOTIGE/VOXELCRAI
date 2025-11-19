use crate::voxel::{Genome, Voxel};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// NextGen Evolution: combine + mutate + fitness
#[derive(Clone, Serialize, Deserialize)]
pub struct EvolutionEngine {
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub fitness_threshold: f64,
}

impl EvolutionEngine {
    pub fn new() -> Self {
        Self {
            mutation_rate: 0.1,
            crossover_rate: 0.7,
            fitness_threshold: 0.5,
        }
    }
    
    /// Combine two genomes (crossover)
    pub fn combine(&self, parent1: &Genome, parent2: &Genome) -> Genome {
        let mut rng = rand::thread_rng();
        let mut child = Genome::new();
        
        // Combine concepts from both parents
        let all_concepts: Vec<String> = parent1.concepts.iter()
            .chain(parent2.concepts.iter())
            .cloned()
            .collect();
        
        // Randomly select concepts for child
        let num_concepts = (all_concepts.len() / 2).min(child.max_concepts);
        for _ in 0..num_concepts {
            if let Some(concept) = all_concepts.get(rng.gen_range(0..all_concepts.len())) {
                child.add_concept(concept.clone());
            }
        }
        
        child
    }
    
    /// Mutate genome
    pub fn mutate(&self, genome: &mut Genome) {
        let mut rng = rand::thread_rng();
        
        if rng.gen_bool(self.mutation_rate) {
            // Add random concept
            if genome.concepts.len() < genome.max_concepts {
                let new_concept = format!("mutated_{}", rng.gen::<u32>());
                genome.add_concept(new_concept);
            }
        }
        
        if rng.gen_bool(self.mutation_rate) {
            // Remove random concept
            if !genome.concepts.is_empty() {
                let idx = rng.gen_range(0..genome.concepts.len());
                genome.concepts.remove(idx);
            }
        }
        
        if rng.gen_bool(self.mutation_rate) {
            // Modify random concept
            if !genome.concepts.is_empty() {
                let idx = rng.gen_range(0..genome.concepts.len());
                genome.concepts[idx] = format!("{}_mut", genome.concepts[idx]);
            }
        }
    }
    
    /// Calculate fitness based on voxel properties
    pub fn fitness(&self, voxel: &Voxel) -> f64 {
        let mut fitness = 0.0;
        
        // Energy contributes to fitness
        fitness += voxel.energy * 0.3;
        
        // Genome complexity
        fitness += voxel.genome.concepts.len() as f64 * 0.1;
        
        // Resonance
        fitness += voxel.resonance.to_f32() as f64 * 0.2;
        
        // Perception diversity
        let perception_sum = voxel.perception_visual.to_f32() +
            voxel.perception_auditory.to_f32() +
            voxel.perception_tactile.to_f32();
        fitness += perception_sum as f64 * 0.1;
        
        // Emotion balance
        let emotion_balance = 1.0 - (voxel.emotion_valence.abs() + 
            voxel.emotion_arousal.abs() + 
            voxel.emotion_dominance.abs()) / 3.0;
        fitness += emotion_balance * 0.3;
        
        fitness
    }
    
    /// Evolve a population of voxels
    pub fn evolve(&self, voxels: &mut [Voxel]) {
        // Calculate fitness for all
        let mut fitness_scores: Vec<(usize, f64)> = voxels.iter()
            .enumerate()
            .map(|(i, v)| (i, self.fitness(v)))
            .collect();
        
        // Sort by fitness
        fitness_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Select top performers
        let top_count = (voxels.len() / 2).max(1);
        let mut rng = rand::thread_rng();
        
        // Create new generation
        for i in top_count..voxels.len() {
            let parent1_idx = fitness_scores[rng.gen_range(0..top_count)].0;
            let parent2_idx = fitness_scores[rng.gen_range(0..top_count)].0;
            
            if rng.gen_bool(self.crossover_rate) {
                // Crossover
                let mut new_genome = self.combine(
                    &voxels[parent1_idx].genome,
                    &voxels[parent2_idx].genome,
                );
                self.mutate(&mut new_genome);
                voxels[i].genome = new_genome;
            } else {
                // Mutation only
                voxels[i].genome = voxels[parent1_idx].genome.clone();
                self.mutate(&mut voxels[i].genome);
            }
        }
    }
}

impl Default for EvolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}
