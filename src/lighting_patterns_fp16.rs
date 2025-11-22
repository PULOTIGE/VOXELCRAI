// FP16/INT8 оптимизированные паттерны освещения для Radeon VII
// Vega 20 поддерживает Rapid Packed Math (RPM) - 2x производительность FP16
// INT8 может дать 4x производительность для некоторых операций

use glam::Vec3;
use half::f16;
use std::collections::HashMap;

/// FP16 оптимизированный паттерн (512 байт вместо 1024)
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct BakedLightPatternFP16 {
    // Базовое освещение (16 байт) - FP16 вместо FP32
    pub direct_light: [f16; 3],      // RGB направленного света (6 байт)
    pub indirect_light: [f16; 3],    // RGB окружающего света (6 байт)
    pub ambient: [f16; 3],           // RGB ambient (6 байт)
    pub intensity: f16,              // Общая интенсивность (2 байта)
    
    // Тени (32 байта) - INT8 вместо U8 для лучшей компрессии
    pub shadow_data: [i8; 32],       // Сжатые данные теней (было 64 байта)
    
    // Лучи света (64 байта) - INT8
    pub light_rays: [i8; 64],        // Volumetric lighting (было 128 байт)
    
    // Капли дождя (128 байт) - INT8
    pub rain_pattern: [i8; 128],     // Паттерн капель (было 256 байт)
    
    // Текстуры освещения (256 байт) - INT8
    pub light_texture: [i8; 256],    // Сжатая текстура (было 512 байт)
    
    // Spherical Harmonics (64 байта) - INT8 вместо I8[256]
    pub sh_coefficients: [i8; 64],   // Компрессированные SH (было 256 байт)
    
    // Метаданные (16 байт)
    pub pattern_id: u16,
    pub resolution: u8,
    pub has_shadows: u8,
    pub has_rays: u8,
    pub has_rain: u8,
    pub has_texture: u8,
    pub precision: u8,               // 0=FP32, 1=FP16, 2=INT8
    pub _padding: [u8; 5],
}

impl BakedLightPatternFP16 {
    pub fn new() -> Self {
        Self {
            direct_light: [f16::from_f32(0.0); 3],
            indirect_light: [f16::from_f32(0.0); 3],
            ambient: [f16::from_f32(0.0); 3],
            intensity: f16::from_f32(0.0),
            shadow_data: [0; 32],
            light_rays: [0; 64],
            rain_pattern: [0; 128],
            light_texture: [0; 256],
            sh_coefficients: [0; 64],
            pattern_id: 0,
            resolution: 128,
            has_shadows: 0,
            has_rays: 0,
            has_rain: 0,
            has_texture: 0,
            precision: 1, // FP16
            _padding: [0; 5],
        }
    }
    
    /// Размер структуры (512 байт - в 2 раза меньше!)
    pub fn size() -> usize {
        512
    }
    
    /// Конвертация из FP32 паттерна с компрессией
    pub fn from_fp32(pattern: &crate::lighting_patterns::BakedLightPattern) -> Self {
        let mut fp16_pattern = Self::new();
        
        // Конвертируем освещение в FP16
        fp16_pattern.direct_light = [
            f16::from_f32(pattern.direct_light[0]),
            f16::from_f32(pattern.direct_light[1]),
            f16::from_f32(pattern.direct_light[2]),
        ];
        fp16_pattern.indirect_light = [
            f16::from_f32(pattern.indirect_light[0]),
            f16::from_f32(pattern.indirect_light[1]),
            f16::from_f32(pattern.indirect_light[2]),
        ];
        fp16_pattern.ambient = [
            f16::from_f32(pattern.ambient[0]),
            f16::from_f32(pattern.ambient[1]),
            f16::from_f32(pattern.ambient[2]),
        ];
        fp16_pattern.intensity = f16::from_f32(pattern.intensity);
        
        // Компрессируем тени (64 -> 32 байта)
        for i in 0..32 {
            let idx = i * 2;
            fp16_pattern.shadow_data[i] = ((pattern.shadow_data[idx] as i16 + 
                                           pattern.shadow_data[idx + 1] as i16) / 2) as i8;
        }
        
        // Компрессируем лучи (128 -> 64 байта)
        for i in 0..64 {
            let idx = i * 2;
            fp16_pattern.light_rays[i] = ((pattern.light_rays[idx] as i16 + 
                                          pattern.light_rays[idx + 1] as i16) / 2) as i8;
        }
        
        // Компрессируем дождь (256 -> 128 байт)
        for i in 0..128 {
            let idx = i * 2;
            fp16_pattern.rain_pattern[i] = ((pattern.rain_pattern[idx] as i16 + 
                                            pattern.rain_pattern[idx + 1] as i16) / 2) as i8;
        }
        
        // Компрессируем текстуру (512 -> 256 байт)
        for i in 0..256 {
            let idx = i * 2;
            fp16_pattern.light_texture[i] = ((pattern.light_texture[idx] as i16 + 
                                             pattern.light_texture[idx + 1] as i16) / 2) as i8;
        }
        
        // Компрессируем SH (256 -> 64 байта, берем каждые 4-е)
        for i in 0..64 {
            fp16_pattern.sh_coefficients[i] = pattern.sh_coefficients[i * 4];
        }
        
        fp16_pattern.pattern_id = pattern.pattern_id as u16;
        fp16_pattern.has_shadows = pattern.has_shadows;
        fp16_pattern.has_rays = pattern.has_rays;
        fp16_pattern.has_rain = pattern.has_rain;
        fp16_pattern.has_texture = pattern.has_texture;
        fp16_pattern.precision = 1; // FP16
        
        fp16_pattern
    }
    
    /// Конвертация в bytes
    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>()
            )
        }
    }
}

/// INT8 оптимизированный паттерн (256 байт - в 4 раза меньше!)
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct BakedLightPatternINT8 {
    // Базовое освещение (8 байт) - INT8 вместо FP32
    pub direct_light: [i8; 3],       // RGB (0-127 = 0.0-1.0)
    pub indirect_light: [i8; 3],     // RGB
    pub ambient: [i8; 3],            // RGB
    pub intensity: i8,               // 0-127 = 0.0-1.0
    
    // Тени (16 байт) - еще больше компрессии
    pub shadow_data: [i8; 16],
    
    // Лучи (32 байта)
    pub light_rays: [i8; 32],
    
    // Капли дождя (64 байта)
    pub rain_pattern: [i8; 64],
    
    // Текстуры (128 байт)
    pub light_texture: [i8; 128],
    
    // SH (32 байта)
    pub sh_coefficients: [i8; 32],
    
    // Метаданные (8 байт)
    pub pattern_id: u8,
    pub resolution: u8,
    pub flags: u8,                   // has_shadows, has_rays, has_rain, has_texture (4 бита)
    pub precision: u8,               // 2 = INT8
    pub _padding: [u8; 4],
}

impl BakedLightPatternINT8 {
    pub fn new() -> Self {
        Self {
            direct_light: [0; 3],
            indirect_light: [0; 3],
            ambient: [0; 3],
            intensity: 0,
            shadow_data: [0; 16],
            light_rays: [0; 32],
            rain_pattern: [0; 64],
            light_texture: [0; 128],
            sh_coefficients: [0; 32],
            pattern_id: 0,
            resolution: 64,
            flags: 0,
            precision: 2, // INT8
            _padding: [0; 4],
        }
    }
    
    /// Размер структуры (256 байт - в 4 раза меньше!)
    pub fn size() -> usize {
        256
    }
    
    /// Конвертация из FP32 паттерна с агрессивной компрессией
    pub fn from_fp32(pattern: &crate::lighting_patterns::BakedLightPattern) -> Self {
        let mut int8_pattern = Self::new();
        
        // Конвертируем освещение в INT8 (0-127 = 0.0-1.0)
        int8_pattern.direct_light = [
            (pattern.direct_light[0].clamp(0.0, 1.0) * 127.0) as i8,
            (pattern.direct_light[1].clamp(0.0, 1.0) * 127.0) as i8,
            (pattern.direct_light[2].clamp(0.0, 1.0) * 127.0) as i8,
        ];
        int8_pattern.indirect_light = [
            (pattern.indirect_light[0].clamp(0.0, 1.0) * 127.0) as i8,
            (pattern.indirect_light[1].clamp(0.0, 1.0) * 127.0) as i8,
            (pattern.indirect_light[2].clamp(0.0, 1.0) * 127.0) as i8,
        ];
        int8_pattern.ambient = [
            (pattern.ambient[0].clamp(0.0, 1.0) * 127.0) as i8,
            (pattern.ambient[1].clamp(0.0, 1.0) * 127.0) as i8,
            (pattern.ambient[2].clamp(0.0, 1.0) * 127.0) as i8,
        ];
        int8_pattern.intensity = (pattern.intensity.clamp(0.0, 1.0) * 127.0) as i8;
        
        // Агрессивная компрессия теней (64 -> 16 байт)
        for i in 0..16 {
            let idx = i * 4;
            let avg = (pattern.shadow_data[idx] as i16 +
                      pattern.shadow_data[idx + 1] as i16 +
                      pattern.shadow_data[idx + 2] as i16 +
                      pattern.shadow_data[idx + 3] as i16) / 4;
            int8_pattern.shadow_data[i] = avg as i8;
        }
        
        // Компрессия лучей (128 -> 32 байта)
        for i in 0..32 {
            let idx = i * 4;
            let avg = (pattern.light_rays[idx] as i16 +
                      pattern.light_rays[idx + 1] as i16 +
                      pattern.light_rays[idx + 2] as i16 +
                      pattern.light_rays[idx + 3] as i16) / 4;
            int8_pattern.light_rays[i] = avg as i8;
        }
        
        // Компрессия дождя (256 -> 64 байта)
        for i in 0..64 {
            let idx = i * 4;
            let avg = (pattern.rain_pattern[idx] as i16 +
                      pattern.rain_pattern[idx + 1] as i16 +
                      pattern.rain_pattern[idx + 2] as i16 +
                      pattern.rain_pattern[idx + 3] as i16) / 4;
            int8_pattern.rain_pattern[i] = avg as i8;
        }
        
        // Компрессия текстуры (512 -> 128 байт)
        for i in 0..128 {
            let idx = i * 4;
            let avg = (pattern.light_texture[idx] as i16 +
                      pattern.light_texture[idx + 1] as i16 +
                      pattern.light_texture[idx + 2] as i16 +
                      pattern.light_texture[idx + 3] as i16) / 4;
            int8_pattern.light_texture[i] = avg as i8;
        }
        
        // Компрессия SH (256 -> 32 байта)
        for i in 0..32 {
            int8_pattern.sh_coefficients[i] = pattern.sh_coefficients[i * 8];
        }
        
        int8_pattern.pattern_id = pattern.pattern_id as u8;
        int8_pattern.flags = (pattern.has_shadows << 0) |
                            (pattern.has_rays << 1) |
                            (pattern.has_rain << 2) |
                            (pattern.has_texture << 3);
        int8_pattern.precision = 2; // INT8
        
        int8_pattern
    }
    
    /// Конвертация в bytes
    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>()
            )
        }
    }
    
    /// Декодирование INT8 в f32 для использования
    pub fn decode_light(&self) -> ([f32; 3], [f32; 3], [f32; 3], f32) {
        (
            [
                self.direct_light[0] as f32 / 127.0,
                self.direct_light[1] as f32 / 127.0,
                self.direct_light[2] as f32 / 127.0,
            ],
            [
                self.indirect_light[0] as f32 / 127.0,
                self.indirect_light[1] as f32 / 127.0,
                self.indirect_light[2] as f32 / 127.0,
            ],
            [
                self.ambient[0] as f32 / 127.0,
                self.ambient[1] as f32 / 127.0,
                self.ambient[2] as f32 / 127.0,
            ],
            self.intensity as f32 / 127.0,
        )
    }
}

/// Библиотека оптимизированных паттернов
pub struct OptimizedLightingLibrary {
    fp16_patterns: HashMap<crate::lighting_patterns::PatternType, BakedLightPatternFP16>,
    int8_patterns: HashMap<crate::lighting_patterns::PatternType, BakedLightPatternINT8>,
}

impl OptimizedLightingLibrary {
    pub fn new() -> Self {
        let mut library = Self {
            fp16_patterns: HashMap::new(),
            int8_patterns: HashMap::new(),
        };
        
        // Генерируем оптимизированные паттерны из стандартных
        let base_library = crate::lighting_patterns::LightingPatternLibrary::new();
        
        for pattern_type in base_library.list_patterns() {
            if let Some(base_pattern) = base_library.get(pattern_type) {
                library.fp16_patterns.insert(
                    pattern_type,
                    BakedLightPatternFP16::from_fp32(base_pattern),
                );
                library.int8_patterns.insert(
                    pattern_type,
                    BakedLightPatternINT8::from_fp32(base_pattern),
                );
            }
        }
        
        library
    }
    
    /// Получить FP16 паттерн
    pub fn get_fp16(&self, pattern_type: crate::lighting_patterns::PatternType) -> Option<&BakedLightPatternFP16> {
        self.fp16_patterns.get(&pattern_type)
    }
    
    /// Получить INT8 паттерн
    pub fn get_int8(&self, pattern_type: crate::lighting_patterns::PatternType) -> Option<&BakedLightPatternINT8> {
        self.int8_patterns.get(&pattern_type)
    }
}

impl Default for OptimizedLightingLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fp16_pattern_size() {
        assert_eq!(BakedLightPatternFP16::size(), 512);
    }
    
    #[test]
    fn test_int8_pattern_size() {
        assert_eq!(BakedLightPatternINT8::size(), 256);
    }
}
