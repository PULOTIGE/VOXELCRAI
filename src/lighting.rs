use half::f16;

/// LightPattern: exactly 1000 bytes
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct LightPattern {
    // Direct/indirect f16 (4 bytes)
    pub direct_light: f16,
    pub indirect_light: f16,

    // Spherical Harmonics i8 256 bytes (16 coefficients * 16 bytes each)
    pub sh_coefficients: [i8; 256],

    // Materials 512 bytes
    pub materials: [u8; 512],

    // AO/reflection/refraction/emission (228 bytes)
    pub ambient_occlusion: f16,          // 2 bytes
    pub reflection: f16,                 // 2 bytes
    pub refraction: f16,                 // 2 bytes
    pub emission: f16,                   // 2 bytes
    pub material_properties: [f16; 110], // 220 bytes (110 * 2)

    // Padding to exactly 1000 bytes
    _padding: [u8; 0],
}

impl LightPattern {
    pub fn new() -> Self {
        Self {
            direct_light: f16::ZERO,
            indirect_light: f16::ZERO,
            sh_coefficients: [0; 256],
            materials: [0; 512],
            ambient_occlusion: f16::ZERO,
            reflection: f16::ZERO,
            refraction: f16::ZERO,
            emission: f16::ZERO,
            material_properties: [f16::ZERO; 110],
            _padding: [],
        }
    }

    pub fn set_sh_coefficient(&mut self, index: usize, value: i8) {
        if index < 256 {
            self.sh_coefficients[index] = value;
        }
    }

    pub fn get_sh_coefficient(&self, index: usize) -> i8 {
        if index < 256 {
            self.sh_coefficients[index]
        } else {
            0
        }
    }

    pub fn set_material(&mut self, index: usize, value: u8) {
        if index < 512 {
            self.materials[index] = value;
        }
    }

    pub fn calculate_lighting(&self, normal: [f32; 3], _view_dir: [f32; 3]) -> f32 {
        // Simple lighting calculation using direct + indirect + SH
        let direct = self.direct_light.to_f32();
        let indirect = self.indirect_light.to_f32();

        // Sample SH (simplified)
        let sh_sample = self.sample_sh(normal);

        // Combine
        (direct + indirect * 0.5 + sh_sample * 0.3).max(0.0)
    }

    fn sample_sh(&self, direction: [f32; 3]) -> f32 {
        // Simplified SH sampling (would need proper SH basis functions)
        let idx = ((direction[0] + 1.0) * 127.0) as usize % 256;
        self.sh_coefficients[idx] as f32 / 127.0
    }
}

impl Default for LightPattern {
    fn default() -> Self {
        Self::new()
    }
}

// Verify size is exactly 1000 bytes
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_pattern_size() {
        assert_eq!(std::mem::size_of::<LightPattern>(), 1000);
    }
}

/// Lighting System
pub struct LightingSystem {
    pub patterns: Vec<LightPattern>,
}

impl LightingSystem {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    pub fn add_pattern(&mut self, pattern: LightPattern) {
        self.patterns.push(pattern);
    }

    pub fn update_lighting(&mut self, time: f32) {
        // Animate lighting patterns
        for pattern in &mut self.patterns {
            // Oscillate direct light
            let oscillation = (time * 0.5).sin() * 0.5 + 0.5;
            pattern.direct_light = f16::from_f32(oscillation);
        }
    }
}

impl Default for LightingSystem {
    fn default() -> Self {
        Self::new()
    }
}
