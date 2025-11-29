use crate::voxel::types::F16;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightPattern {
    pub direct_light: F16,
    pub indirect_light: F16,
    pub sh_coefficients: [i8; 256],
    pub materials: [u8; 512],
    pub ambient_occlusion: F16,
    pub reflection: F16,
    pub refraction: F16,
    pub emission: F16,
    pub material_properties: [F16; 110],
}

const _: [u8; 1000] = [0; std::mem::size_of::<LightPattern>()];

unsafe impl Zeroable for LightPattern {}
unsafe impl Pod for LightPattern {}

impl LightPattern {
    fn zeroed() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

pub struct LightPatternBank {
    patterns: Vec<LightPattern>,
}

impl LightPatternBank {
    pub fn new(count: usize) -> Self {
        let mut rng = Lcg::new(0xABCDEF01);
        let mut patterns = Vec::with_capacity(count);
        for _ in 0..count {
            let mut pattern = LightPattern::zeroed();
            pattern.direct_light = F16::from_f32(rng.next_f32());
            pattern.indirect_light = F16::from_f32(rng.next_f32());
            for coeff in &mut pattern.sh_coefficients {
                *coeff = (rng.next_u32() & 0x7F) as i8;
            }
            for mat in &mut pattern.materials {
                *mat = (rng.next_u32() & 0xFF) as u8;
            }
            pattern.ambient_occlusion = F16::from_f32(rng.next_f32());
            pattern.reflection = F16::from_f32(rng.next_f32());
            pattern.refraction = F16::from_f32(rng.next_f32());
            pattern.emission = F16::from_f32(rng.next_f32());
            for prop in &mut pattern.material_properties {
                *prop = F16::from_f32(rng.next_f32());
            }
            patterns.push(pattern);
        }
        Self { patterns }
    }

    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    pub fn bytes(&self) -> usize {
        self.patterns.len() * std::mem::size_of::<LightPattern>()
    }

    pub fn patterns(&self) -> &[LightPattern] {
        &self.patterns
    }
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}
