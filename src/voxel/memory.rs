use super::types::{EmotionVectors, Voxel9k};

#[derive(Clone, Copy)]
pub struct EmotionSample {
    pub joy: f64,
    pub fear: f64,
    pub salience: f64,
}

pub struct VoxelMemory {
    side: u32,
    cursor: u32,
    scratch: Voxel9k,
}

impl VoxelMemory {
    pub fn new(side: u32) -> Self {
        Self {
            side,
            cursor: 0,
            scratch: Voxel9k::new(0xDEC0DE),
        }
    }

    pub fn sample(&mut self, time: f32) -> EmotionSample {
        self.cursor = (self.cursor + 1) % (self.side * self.side);
        let phase = time + self.cursor as f32 * 0.002;
        let vectors: EmotionVectors = self.scratch.pulse(phase);

        EmotionSample {
            joy: (vectors.joy + 1.0) * 0.5,
            fear: (vectors.fear + 1.0) * 0.5,
            salience: (vectors.salience + 1.0) * 0.5,
        }
    }
}
