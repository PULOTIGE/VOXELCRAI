// Система для baking паттернов освещения offline
// Генерирует паттерны с тенями, лучами, каплями дождя

use crate::lighting_patterns::{BakedLightPattern, PatternType, PatternGenerator};
use std::path::Path;
use std::fs::File;
use std::io::Write;

/// Baker для создания паттернов освещения
pub struct PatternBaker {
    generator: PatternGenerator,
    output_dir: String,
}

impl PatternBaker {
    pub fn new(output_dir: &str) -> Self {
        Self {
            generator: PatternGenerator::new(),
            output_dir: output_dir.to_string(),
        }
    }
    
    /// Bake все стандартные паттерны
    pub fn bake_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("=== Baking Lighting Patterns ===");
        
        // Создаем директорию
        std::fs::create_dir_all(&self.output_dir)?;
        
        // Bake каждый паттерн
        let patterns = vec![
            (PatternType::Sunny, "sunny"),
            (PatternType::Cloudy, "cloudy"),
            (PatternType::Rainy, "rainy"),
            (PatternType::Stormy, "stormy"),
            (PatternType::Sunset, "sunset"),
            (PatternType::Night, "night"),
            (PatternType::Indoor, "indoor"),
            (PatternType::Neon, "neon"),
            (PatternType::Volumetric, "volumetric"),
        ];
        
        for (pattern_type, name) in patterns {
            self.bake_pattern(pattern_type, name)?;
        }
        
        println!("✓ All patterns baked successfully!");
        Ok(())
    }
    
    /// Bake конкретный паттерн
    pub fn bake_pattern(&self, pattern_type: PatternType, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Baking pattern: {:?} ({})", pattern_type, name);
        
        let pattern = match pattern_type {
            PatternType::Sunny => self.generator.generate_sunny(),
            PatternType::Cloudy => self.generator.generate_cloudy(),
            PatternType::Rainy => self.generator.generate_rainy(),
            PatternType::Stormy => self.generator.generate_stormy(),
            PatternType::Sunset => self.generator.generate_sunset(),
            PatternType::Night => self.generator.generate_night(),
            PatternType::Indoor => self.generator.generate_indoor(),
            PatternType::Neon => self.generator.generate_neon(),
            PatternType::Volumetric => self.generator.generate_volumetric(),
            PatternType::Custom(_) => return Err("Custom patterns not supported in baker".into()),
        };
        
        // Сохраняем в бинарный файл
        let file_path = format!("{}/{}.pattern", self.output_dir, name);
        let mut file = File::create(&file_path)?;
        
        // Записываем как raw bytes
        let bytes = pattern.to_bytes();
        file.write_all(bytes)?;
        
        println!("  ✓ Saved to {}", file_path);
        Ok(())
    }
    
    /// Загрузить паттерн из файла
    pub fn load_pattern(path: &Path) -> Result<BakedLightPattern, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        BakedLightPattern::from_bytes(&bytes)
            .map_err(|e| e.into())
    }
}

/// Утилита для baking паттернов
pub fn bake_patterns(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let baker = PatternBaker::new(output_dir);
    baker.bake_all()
}
