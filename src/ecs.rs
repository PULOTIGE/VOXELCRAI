// Re-export bevy_ecs for convenience
pub use bevy_ecs::prelude::*;

// Additional ECS utilities can be added here
pub mod systems {
    use bevy_ecs::prelude::*;
    use crate::voxel::Voxel;
    
    /// System to update voxel physics
    pub fn update_voxel_physics(mut query: Query<&mut Voxel>) {
        for mut voxel in query.iter_mut() {
            // Update position based on velocity
            voxel.position[0] += voxel.velocity_x as i32;
            voxel.position[1] += voxel.velocity_y as i32;
            voxel.position[2] += voxel.velocity_z as i32;
            
            // Update velocity based on acceleration
            voxel.velocity_x = (voxel.velocity_x as i16 + voxel.acceleration_x as i16)
                .max(-128).min(127) as i8;
            voxel.velocity_y = (voxel.velocity_y as i16 + voxel.acceleration_y as i16)
                .max(-128).min(127) as i8;
            voxel.velocity_z = (voxel.velocity_z as i16 + voxel.acceleration_z as i16)
                .max(-128).min(127) as i8;
        }
    }
    
    /// System to update voxel energy
    pub fn update_voxel_energy(mut query: Query<&mut Voxel>, time: Res<Time>) {
        for mut voxel in query.iter_mut() {
            // Energy decays over time
            voxel.energy *= 0.999;
            
            // Energy increases with resonance
            voxel.energy += voxel.resonance.to_f32() as f64 * time.delta_secs() as f64;
        }
    }
}
