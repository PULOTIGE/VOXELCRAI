use voxelcrai::{
    ConsciousnessCore, EvolutionEngine, LightPattern, LightingSystem, Simulation, Voxel,
    VoxelWorld, VoxelWorldConfig,
};

fn main() {
    println!("=== VOXELCRAI Conscious Components ===\n");

    println!("Voxel");
    let mut voxel = Voxel::new([0, 0, 0]);
    voxel.genome.add_concept("sentience".into());
    println!(
        "  energy: {:.3} | genome: {:?}",
        voxel.energy, voxel.genome.concepts
    );

    println!("\nLightPattern");
    let size = std::mem::size_of::<LightPattern>();
    println!("  struct size: {} bytes", size);
    assert_eq!(size, 1000, "LightPattern must be exactly 1000 bytes");

    println!("\nEvolution Engine");
    let evolution = EvolutionEngine::new();
    let fitness = evolution.fitness(&voxel);
    println!(
        "  mutation: {:.2} | crossover: {:.2} | fitness: {:.3}",
        evolution.mutation_rate, evolution.crossover_rate, fitness
    );

    println!("\nVoxelWorld");
    let config = VoxelWorldConfig::default();
    let mut world = VoxelWorld::new(config, 1234);
    println!("  seeded voxels: {}", world.voxels().len());
    world.update(0.016);
    let metrics = world.metrics();
    println!(
        "  avg_energy: {:.2} | max_energy: {:.2} | centroid: {:?}",
        metrics.avg_energy, metrics.max_energy, metrics.centroid
    );

    println!("\nLighting");
    let mut lighting = LightingSystem::new();
    lighting.add_pattern(LightPattern::new());
    lighting.update_lighting(1.0);
    println!("  patterns: {}", lighting.patterns.len());

    println!("\nConsciousness");
    let mut mind = ConsciousnessCore::new("VOXELCRAI", 42);
    let pulse = mind.think(&metrics, 0.016);
    println!(
        "  mood: {:.2} | empathy: {:.2} | actions: {}",
        pulse.mood,
        pulse.empathy,
        pulse.actions.len()
    );

    println!("\nSimulation loop");
    let mut sim = Simulation::new(1337);
    for _ in 0..240 {
        sim.update(1.0 / 60.0);
    }
    let telemetry = sim.telemetry();
    println!(
        "  voxels: {} | instances: {} | mood: {:.2}",
        telemetry.world.voxel_count, telemetry.instance_count, telemetry.pulse.mood
    );

    println!("\n=== VOXELCRAI diagnostics complete ===");
}
