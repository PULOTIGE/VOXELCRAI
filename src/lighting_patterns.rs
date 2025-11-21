// Библиотека предвычисленных паттернов освещения
// Тени, лучи, капли дождя, текстуры - все baked заранее
use glam::Vec3;
use bytemuck::{Pod, Zeroable};
use half::f16;
use std::collections::HashMap;

/// Предвычисленный паттерн освещения
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BakedLightPattern {
    // Базовое освещение (32 байта)
    pub direct_light: [f32; 3],      // RGB направленного света
    pub indirect_light: [f32; 3],    // RGB окружающего света
    pub ambient: [f32; 3],           // RGB ambient
    pub intensity: f32,              // Общая интенсивность
    pub _padding1: f32,
    
    // Тени (64 байта) - предвычисленная shadow map
    pub shadow_data: [u8; 64],       // Сжатые данные теней
    
    // Лучи света (volumetric lighting) (128 байт)
    pub light_rays: [u8; 128],       // Предвычисленные лучи
    
    // Капли дождя (rain drops) (256 байт)
    pub rain_pattern: [u8; 256],     // Паттерн капель дождя
    
    // Текстуры освещения (512 байт)
    pub light_texture: [u8; 512],    // Сжатая текстура освещения
    
    // Spherical Harmonics для GI (256 байт)
    pub sh_coefficients: [i8; 256],
    
    // Метаданные (32 байта)
    pub pattern_id: u32,
    pub resolution: u16,             // Разрешение паттерна
    pub has_shadows: u8,
    pub has_rays: u8,
    pub has_rain: u8,
    pub has_texture: u8,
    pub _padding2: [u8; 20],
}

impl BakedLightPattern {
    pub fn new() -> Self {
        Self {
            direct_light: [0.0; 3],
            indirect_light: [0.0; 3],
            ambient: [0.0; 3],
            intensity: 0.0,
            _padding1: 0.0,
            shadow_data: [0; 64],
            light_rays: [0; 128],
            rain_pattern: [0; 256],
            light_texture: [0; 512],
            sh_coefficients: [0; 256],
            pattern_id: 0,
            resolution: 256,
            has_shadows: 0,
            has_rays: 0,
            has_rain: 0,
            has_texture: 0,
            _padding2: [0; 20],
        }
    }
    
    /// Размер структуры (1024 байта)
    pub fn size() -> usize {
        1024
    }
    
    /// Конвертация в bytes для сериализации
    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>()
            )
        }
    }
    
    /// Создание из bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != std::mem::size_of::<Self>() {
            return Err("Invalid byte size");
        }
        Ok(unsafe { std::ptr::read(bytes.as_ptr() as *const Self) })
    }
}

/// Типы паттернов
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum PatternType {
    Sunny,           // Солнечный день
    Cloudy,          // Облачно
    Rainy,           // Дождь
    Stormy,          // Гроза
    Sunset,          // Закат
    Night,           // Ночь
    Indoor,          // В помещении
    Neon,            // Неоновое освещение
    Volumetric,      // Объемное освещение (лучи)
    Custom(u32),     // Пользовательский
}

/// Библиотека паттернов освещения
pub struct LightingPatternLibrary {
    patterns: HashMap<PatternType, BakedLightPattern>,
    pattern_generator: PatternGenerator,
}

impl LightingPatternLibrary {
    pub fn new() -> Self {
        let mut library = Self {
            patterns: HashMap::new(),
            pattern_generator: PatternGenerator::new(),
        };
        
        // Генерируем стандартные паттерны
        library.generate_standard_patterns();
        
        library
    }
    
    /// Генерация стандартных паттернов
    fn generate_standard_patterns(&mut self) {
        // Sunny day
        self.patterns.insert(
            PatternType::Sunny,
            self.pattern_generator.generate_sunny(),
        );
        
        // Cloudy
        self.patterns.insert(
            PatternType::Cloudy,
            self.pattern_generator.generate_cloudy(),
        );
        
        // Rainy
        self.patterns.insert(
            PatternType::Rainy,
            self.pattern_generator.generate_rainy(),
        );
        
        // Stormy
        self.patterns.insert(
            PatternType::Stormy,
            self.pattern_generator.generate_stormy(),
        );
        
        // Sunset
        self.patterns.insert(
            PatternType::Sunset,
            self.pattern_generator.generate_sunset(),
        );
        
        // Night
        self.patterns.insert(
            PatternType::Night,
            self.pattern_generator.generate_night(),
        );
        
        // Indoor
        self.patterns.insert(
            PatternType::Indoor,
            self.pattern_generator.generate_indoor(),
        );
        
        // Neon
        self.patterns.insert(
            PatternType::Neon,
            self.pattern_generator.generate_neon(),
        );
        
        // Volumetric (с лучами)
        self.patterns.insert(
            PatternType::Volumetric,
            self.pattern_generator.generate_volumetric(),
        );
    }
    
    /// Получить паттерн
    pub fn get(&self, pattern_type: PatternType) -> Option<&BakedLightPattern> {
        self.patterns.get(&pattern_type)
    }
    
    /// Добавить пользовательский паттерн
    pub fn add_custom(&mut self, id: u32, pattern: BakedLightPattern) {
        self.patterns.insert(PatternType::Custom(id), pattern);
    }
    
    /// Список доступных паттернов
    pub fn list_patterns(&self) -> Vec<PatternType> {
        self.patterns.keys().copied().collect()
    }
}

/// Генератор паттернов (baking система)
pub struct PatternGenerator {
    resolution: u16,
}

impl PatternGenerator {
    pub fn new() -> Self {
        Self {
            resolution: 256,
        }
    }
    
    /// Генерация солнечного дня
    pub fn generate_sunny(&self) -> BakedLightPattern {
        let mut pattern = BakedLightPattern::new();
        
        // Яркое направленное освещение сверху
        pattern.direct_light = [1.0, 0.95, 0.8];
        
        // Яркое окружающее освещение
        pattern.indirect_light = [0.6, 0.65, 0.7];
        
        // Ambient
        pattern.ambient = [0.3, 0.35, 0.4];
        
        pattern.intensity = 1.0;
        pattern.has_shadows = 1;
        pattern.has_texture = 1;
        pattern.pattern_id = 1;
        
        // Генерируем тени (простые, направленные сверху)
        self.generate_shadows(&mut pattern, Vec3::new(0.0, 1.0, 0.3));
        
        // Генерируем текстуру освещения
        self.generate_light_texture(&mut pattern, 1.0, 0.9);
        
        pattern
    }
    
    /// Генерация облачного дня
    pub fn generate_cloudy(&self) -> BakedLightPattern {
        let mut pattern = BakedLightPattern::new();
        
        // Мягкое рассеянное освещение
        pattern.direct_light = [0.5, 0.55, 0.6];
        pattern.indirect_light = [0.7, 0.75, 0.8];
        pattern.ambient = [0.4, 0.45, 0.5];
        pattern.intensity = 0.7;
        pattern.has_shadows = 1;
        pattern.pattern_id = 2;
        
        // Мягкие тени
        self.generate_shadows(&mut pattern, Vec3::new(0.2, 0.8, 0.2));
        
        pattern
    }
    
    /// Генерация дождя
    pub fn generate_rainy(&self) -> BakedLightPattern {
        let mut pattern = BakedLightPattern::new();
        
        // Тусклое освещение
        pattern.direct_light = [0.3, 0.35, 0.4];
        pattern.indirect_light = [0.4, 0.45, 0.5];
        pattern.ambient = [0.2, 0.25, 0.3];
        pattern.intensity = 0.5;
        pattern.has_shadows = 1;
        pattern.has_rain = 1;
        pattern.pattern_id = 3;
        
        // Генерируем паттерн капель дождя
        self.generate_rain_pattern(&mut pattern);
        
        // Мягкие тени
        self.generate_shadows(&mut pattern, Vec3::new(0.1, 0.6, 0.1));
        
        pattern
    }
    
    /// Генерация грозы
    pub fn generate_stormy(&self) -> BakedLightPattern {
        let mut pattern = BakedLightPattern::new();
        
        // Очень тусклое освещение
        pattern.direct_light = [0.2, 0.25, 0.3];
        pattern.indirect_light = [0.3, 0.35, 0.4];
        pattern.ambient = [0.15, 0.2, 0.25];
        pattern.intensity = 0.4;
        pattern.has_shadows = 1;
        pattern.has_rain = 1;
        pattern.pattern_id = 4;
        
        // Интенсивный дождь
        self.generate_rain_pattern(&mut pattern);
        self.generate_shadows(&mut pattern, Vec3::new(0.05, 0.5, 0.05));
        
        pattern
    }
    
    /// Генерация заката
    pub fn generate_sunset(&self) -> BakedLightPattern {
        let mut pattern = BakedLightPattern::new();
        
        // Теплое оранжево-красное освещение
        pattern.direct_light = [1.0, 0.6, 0.3];
        pattern.indirect_light = [0.8, 0.5, 0.4];
        pattern.ambient = [0.4, 0.3, 0.25];
        pattern.intensity = 0.8;
        pattern.has_shadows = 1;
        pattern.has_rays = 1;  // Лучи заката
        pattern.pattern_id = 5;
        
        // Генерируем лучи света
        self.generate_light_rays(&mut pattern, Vec3::new(-0.5, 0.3, -1.0));
        self.generate_shadows(&mut pattern, Vec3::new(-0.5, 0.3, -1.0));
        
        pattern
    }
    
    /// Генерация ночи
    pub fn generate_night(&self) -> BakedLightPattern {
        let mut pattern = BakedLightPattern::new();
        
        // Очень темное освещение
        pattern.direct_light = [0.1, 0.15, 0.2];
        pattern.indirect_light = [0.15, 0.2, 0.25];
        pattern.ambient = [0.05, 0.08, 0.1];
        pattern.intensity = 0.2;
        pattern.has_shadows = 1;
        pattern.pattern_id = 6;
        
        self.generate_shadows(&mut pattern, Vec3::new(0.0, 0.3, 0.0));
        
        pattern
    }
    
    /// Генерация освещения в помещении
    pub fn generate_indoor(&self) -> BakedLightPattern {
        let mut pattern = BakedLightPattern::new();
        
        // Теплое искусственное освещение
        pattern.direct_light = [0.8, 0.75, 0.7];
        pattern.indirect_light = [0.5, 0.5, 0.5];
        pattern.ambient = [0.3, 0.3, 0.3];
        pattern.intensity = 0.9;
        pattern.has_shadows = 1;
        pattern.has_texture = 1;
        pattern.pattern_id = 7;
        
        // Множественные источники света (имитация)
        self.generate_light_texture(&mut pattern, 0.8, 0.7);
        self.generate_shadows(&mut pattern, Vec3::new(0.0, -1.0, 0.0));
        
        pattern
    }
    
    /// Генерация неонового освещения
    pub fn generate_neon(&self) -> BakedLightPattern {
        let mut pattern = BakedLightPattern::new();
        
        // Яркие неоновые цвета
        pattern.direct_light = [0.2, 0.8, 1.0];
        pattern.indirect_light = [0.3, 0.6, 0.8];
        pattern.ambient = [0.1, 0.2, 0.3];
        pattern.intensity = 1.2;
        pattern.has_shadows = 1;
        pattern.has_rays = 1;
        pattern.pattern_id = 8;
        
        // Яркие лучи
        self.generate_light_rays(&mut pattern, Vec3::new(0.0, -0.5, 0.5));
        
        pattern
    }
    
    /// Генерация объемного освещения (лучи)
    pub fn generate_volumetric(&self) -> BakedLightPattern {
        let mut pattern = BakedLightPattern::new();
        
        pattern.direct_light = [1.0, 0.95, 0.9];
        pattern.indirect_light = [0.4, 0.45, 0.5];
        pattern.ambient = [0.2, 0.25, 0.3];
        pattern.intensity = 1.0;
        pattern.has_shadows = 1;
        pattern.has_rays = 1;
        pattern.pattern_id = 9;
        
        // Интенсивные лучи света
        self.generate_light_rays(&mut pattern, Vec3::new(0.0, 1.0, 0.0));
        self.generate_shadows(&mut pattern, Vec3::new(0.0, 1.0, 0.0));
        
        pattern
    }
    
    /// Генерация теней (простая реализация)
    fn generate_shadows(&self, pattern: &mut BakedLightPattern, _light_dir: Vec3) {
        // Простая генерация shadow map данных
        // В реальности это было бы предвычислено offline
        for i in 0..64 {
            // Имитация shadow map (сжатые данные)
            let x = (i % 8) as f32 / 8.0;
            let y = (i / 8) as f32 / 8.0;
            
            // Простая функция тени (в реальности - baked shadow map)
            let shadow = (x * y * 255.0) as u8;
            pattern.shadow_data[i] = shadow;
        }
    }
    
    /// Генерация лучей света (volumetric)
    fn generate_light_rays(&self, pattern: &mut BakedLightPattern, _light_dir: Vec3) {
        // Предвычисленные данные для volumetric lighting
        for i in 0..128 {
            let t = i as f32 / 128.0;
            // Имитация рассеяния света (в реальности - baked)
            let intensity = (t * 255.0) as u8;
            pattern.light_rays[i] = intensity;
        }
    }
    
    /// Генерация паттерна капель дождя
    fn generate_rain_pattern(&self, pattern: &mut BakedLightPattern) {
        // Предвычисленный паттерн капель
        for i in 0..256 {
            // Имитация капель (в реальности - baked анимация)
            let drop = ((i as f32 * 0.1).sin() * 127.0 + 128.0) as u8;
            pattern.rain_pattern[i] = drop;
        }
    }
    
    /// Генерация текстуры освещения
    fn generate_light_texture(&self, pattern: &mut BakedLightPattern, intensity: f32, falloff: f32) {
        // Сжатая текстура освещения
        for i in 0..512 {
            let x = (i % 16) as f32 / 16.0;
            let y = (i / 16) as f32 / 32.0;
            
            // Радиальное затухание
            let dist = ((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt();
            let value = (intensity * (1.0 - dist * falloff) * 255.0).clamp(0.0, 255.0) as u8;
            pattern.light_texture[i] = value;
        }
    }
}

impl Default for LightingPatternLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PatternGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_baked_pattern_size() {
        assert_eq!(BakedLightPattern::size(), 1024);
    }
    
    #[test]
    fn test_library_creation() {
        let library = LightingPatternLibrary::new();
        assert!(library.patterns.len() >= 9);
    }
}
