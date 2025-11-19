use std::ops::{Index, IndexMut};

/// Minimal half-precision float implementation (binary16) without extra dependencies.
#[repr(transparent)]
#[derive(Clone, Copy, Default)]
pub struct F16(pub u16);

impl F16 {
    pub fn from_f32(value: f32) -> Self {
        Self(f32_to_f16_bits(value))
    }

    pub fn to_f32(self) -> f32 {
        f16_bits_to_f32(self.0)
    }
}

#[repr(C, align(16))]
pub struct Voxel9k {
    pub metadata: [u16; 256], // 512 B
    pub sensory: [F16; 768],  // 1536 B
    pub physics: [i8; 1024],  // 1024 B
    pub logic: [f32; 512],    // 2048 B
    pub emotions: [f64; 256], // 2048 B
    pub memory: [f32; 512],   // 2048 B
}

impl Voxel9k {
    pub fn new(seed: u32) -> Self {
        let mut voxel = Self {
            metadata: [0; 256],
            sensory: [F16::default(); 768],
            physics: [0; 1024],
            logic: [0.0; 512],
            emotions: [0.0; 256],
            memory: [0.0; 512],
        };
        voxel.seed(seed);
        voxel
    }

    pub fn seed(&mut self, seed: u32) {
        let mut state = lcg(seed);
        for meta in &mut self.metadata {
            *meta = (state.next() & 0xFFFF) as u16;
        }
        for sense in &mut self.sensory {
            let val = (state.next() as f32 / u32::MAX as f32) * 2.0 - 1.0;
            *sense = F16::from_f32(val);
        }
        for phys in &mut self.physics {
            *phys = (state.next() & 0x7F) as i8;
        }
        for (i, logic) in self.logic.iter_mut().enumerate() {
            *logic = ((i as f32).sin() + (state.next() as f32 * 0.00001)).tanh();
        }
        for (i, emo) in self.emotions.iter_mut().enumerate() {
            *emo = ((i as f64).cos() + (state.next() as f64 * 1e-9)).tanh();
        }
        for (i, mem) in self.memory.iter_mut().enumerate() {
            *mem = ((i as f32 * 0.031).sin()).tanh();
        }
    }

    pub fn pulse(&mut self, phase: f32) -> EmotionVectors {
        for (i, sense) in self.sensory.iter_mut().enumerate() {
            let base = phase * 0.5 + i as f32 * 0.01;
            *sense = F16::from_f32(base.sin());
        }
        let mut joy = 0.0f64;
        let mut fear = 0.0f64;
        let mut salience = 0.0f64;

        for (i, emo) in self.emotions.iter_mut().enumerate() {
            let heat = (phase as f64 * 0.2 + i as f64 * 0.01).cos();
            *emo = (*emo * 0.9) + heat * 0.1;
            joy += heat.abs();
            fear += heat.sin().abs();
        }

        for (i, logic) in self.logic.iter_mut().enumerate() {
            let wave = (phase + i as f32 * 0.003).sin();
            *logic = (*logic * 0.8) + wave * 0.2;
            salience += wave as f64;
        }

        EmotionVectors {
            joy: joy / self.emotions.len() as f64,
            fear: fear / self.emotions.len() as f64,
            salience: salience / self.logic.len() as f64,
        }
    }
}

pub struct EmotionVectors {
    pub joy: f64,
    pub fear: f64,
    pub salience: f64,
}

impl Index<usize> for Voxel9k {
    type Output = i8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.physics[index % self.physics.len()]
    }
}

impl IndexMut<usize> for Voxel9k {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.physics[index % self.physics.len()]
    }
}

const _: [u8; 9 * 1024] = [0; std::mem::size_of::<Voxel9k>()];

fn f32_to_f16_bits(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = (bits >> 16) & 0x8000;
    let mut exp = ((bits >> 23) & 0xff) as i32;
    let mut mant = bits & 0x7fffff;

    if exp == 0xff {
        if mant != 0 {
            return sign | 0x7e00;
        } else {
            return sign | 0x7c00;
        }
    }

    exp -= 127;
    if exp > 15 {
        return sign | 0x7c00;
    }
    if exp < -14 {
        let shift = -14 - exp;
        mant |= 1 << 23;
        mant >>= shift + 1;
        return sign | ((mant + 0x1000) >> 13) as u16;
    }

    exp += 15;
    let half = sign | ((exp as u16) << 10) | ((mant + 0x1000) >> 13) as u16;
    half
}

fn f16_bits_to_f32(bits: u16) -> f32 {
    let sign = ((bits & 0x8000) as u32) << 16;
    let exp = ((bits & 0x7c00) >> 10) as i32;
    let mant = (bits & 0x03ff) as u32;

    let value = if exp == 0 {
        if mant == 0 {
            sign
        } else {
            let mut mantissa = mant;
            let mut exponent = -1;
            while (mantissa & 0x0400) == 0 {
                mantissa <<= 1;
                exponent -= 1;
            }
            mantissa &= 0x03ff;
            let exp_bits = ((exponent + 127) as u32) << 23;
            let mant_bits = mantissa << 13;
            sign | exp_bits | mant_bits
        }
    } else if exp == 0x1f {
        sign | 0x7f800000 | (mant << 13)
    } else {
        let exp_bits = ((exp + (127 - 15)) as u32) << 23;
        let mant_bits = mant << 13;
        sign | exp_bits | mant_bits
    };
    f32::from_bits(value)
}

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.0 >> 32) as u32
    }
}

fn lcg(seed: u32) -> Lcg {
    Lcg(seed as u64 + 1)
}

unsafe impl bytemuck::Zeroable for F16 {}
unsafe impl bytemuck::Pod for F16 {}
