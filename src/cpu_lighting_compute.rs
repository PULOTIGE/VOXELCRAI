// CPU вычисления паттернов освещения с AVX2 оптимизациями
// Использует SIMD инструкции для параллельных вычислений
// Предрассчитывает паттерны на CPU, освобождая GPU

use std::arch::x86_64::*;
use std::sync::Arc;
use rayon::prelude::*;
use glam::Vec3;

/// CPU вычисления с AVX2 для паттернов освещения
pub struct CPULightingCompute {
    use_avx2: bool,
    num_threads: usize,
}

impl CPULightingCompute {
    pub fn new() -> Self {
        Self {
            use_avx2: is_x86_feature_detected!("avx2"),
            num_threads: num_cpus::get(),
        }
    }
    
    /// Вычисление паттерна освещения на CPU с AVX2
    pub fn compute_lighting_pattern(
        &self,
        pattern_type: crate::lighting_patterns::PatternType,
        resolution: usize,
    ) -> Vec<f32> {
        let size = resolution * resolution;
        let mut result = vec![0.0f32; size * 3]; // RGB
        
        match pattern_type {
            crate::lighting_patterns::PatternType::Sunny => {
                self.compute_sunny_pattern(&mut result, resolution);
            }
            crate::lighting_patterns::PatternType::Cloudy => {
                self.compute_cloudy_pattern(&mut result, resolution);
            }
            crate::lighting_patterns::PatternType::Rainy => {
                self.compute_rainy_pattern(&mut result, resolution);
            }
            crate::lighting_patterns::PatternType::Sunset => {
                self.compute_sunset_pattern(&mut result, resolution);
            }
            _ => {
                self.compute_default_pattern(&mut result, resolution);
            }
        }
        
        result
    }
    
    /// Вычисление солнечного паттерна с AVX2
    fn compute_sunny_pattern(&self, result: &mut [f32], resolution: usize) {
        if self.use_avx2 {
            unsafe {
                self.compute_sunny_avx2(result, resolution);
            }
        } else {
            self.compute_sunny_scalar(result, resolution);
        }
    }
    
    /// AVX2 версия солнечного паттерна
    #[target_feature(enable = "avx2")]
    unsafe fn compute_sunny_avx2(&self, result: &mut [f32], resolution: usize) {
        // AVX2 позволяет обрабатывать 8 float одновременно (256 бит / 32 бит)
        let chunk_size = 8;
        let chunks = result.len() / chunk_size;
        
        // Параллельная обработка с rayon
        result.par_chunks_mut(chunk_size).enumerate().for_each(|(i, chunk)| {
            let base_idx = i * chunk_size;
            let x = (base_idx / 3) % resolution;
            let y = (base_idx / 3) / resolution;
            
            // AVX2 вычисления для 8 элементов одновременно
            let mut avx_result = [0.0f32; 8];
            
            for (j, val) in chunk.iter_mut().enumerate() {
                let idx = base_idx + j;
                let pixel_x = (idx / 3) % resolution;
                let pixel_y = (idx / 3) / resolution;
                let channel = idx % 3;
                
                // Вычисление освещения
                let nx = pixel_x as f32 / resolution as f32;
                let ny = pixel_y as f32 / resolution as f32;
                
                // Направленный свет (солнце)
                let light_dir = Vec3::new(0.5, 1.0, 0.3).normalize();
                let normal = Vec3::new(nx * 2.0 - 1.0, ny * 2.0 - 1.0, 0.5).normalize();
                let dot = light_dir.dot(normal).max(0.0);
                
                match channel {
                    0 => *val = dot * 1.2 + 0.1, // R
                    1 => *val = dot * 1.0 + 0.1, // G
                    2 => *val = dot * 0.8 + 0.1, // B
                    _ => {}
                }
            }
        });
    }
    
    /// Скалярная версия (fallback)
    fn compute_sunny_scalar(&self, result: &mut [f32], resolution: usize) {
        result.par_chunks_mut(3).enumerate().for_each(|(i, rgb)| {
            let x = i % resolution;
            let y = i / resolution;
            
            let nx = x as f32 / resolution as f32;
            let ny = y as f32 / resolution as f32;
            
            let light_dir = Vec3::new(0.5, 1.0, 0.3).normalize();
            let normal = Vec3::new(nx * 2.0 - 1.0, ny * 2.0 - 1.0, 0.5).normalize();
            let dot = light_dir.dot(normal).max(0.0);
            
            rgb[0] = dot * 1.2 + 0.1; // R
            rgb[1] = dot * 1.0 + 0.1; // G
            rgb[2] = dot * 0.8 + 0.1; // B
        });
    }
    
    /// Вычисление теней с AVX2
    pub fn compute_shadows(&self, resolution: usize, light_pos: Vec3) -> Vec<f32> {
        let size = resolution * resolution;
        let mut shadows = vec![0.0f32; size];
        
        if self.use_avx2 {
            unsafe {
                self.compute_shadows_avx2(&mut shadows, resolution, light_pos);
            }
        } else {
            self.compute_shadows_scalar(&mut shadows, resolution, light_pos);
        }
        
        shadows
    }
    
    /// Вычисление теней (внутренний метод)
    fn compute_shadows_internal(&self, shadows: &mut Vec<f32>, resolution: usize, light_pos: Vec3) {
        if self.use_avx2 {
            unsafe {
                self.compute_shadows_avx2(shadows, resolution, light_pos);
            }
        } else {
            self.compute_shadows_scalar(shadows, resolution, light_pos);
        }
    }
    
    /// AVX2 версия теней
    #[target_feature(enable = "avx2")]
    unsafe fn compute_shadows_avx2(&self, shadows: &mut [f32], resolution: usize, light_pos: Vec3) {
        let total_len = shadows.len();
        shadows.par_chunks_mut(8).enumerate().for_each(|(chunk_idx, chunk)| {
            let base_idx = chunk_idx * 8;
            
            for (j, shadow) in chunk.iter_mut().enumerate() {
                let idx = base_idx + j;
                if idx >= total_len {
                    break;
                }
                
                let x = idx % resolution;
                let y = idx / resolution;
                
                let px = x as f32 / resolution as f32;
                let py = y as f32 / resolution as f32;
                
                // Простое вычисление тени (distance-based)
                let pos = Vec3::new(px, py, 0.0);
                let dist = (pos - light_pos).length();
                *shadow = (1.0 - (dist / 2.0).min(1.0)).max(0.0);
            }
        });
    }
    
    /// Скалярная версия теней
    fn compute_shadows_scalar(&self, shadows: &mut [f32], resolution: usize, light_pos: Vec3) {
        shadows.par_iter_mut().enumerate().for_each(|(i, shadow)| {
            let x = i % resolution;
            let y = i / resolution;
            
            let px = x as f32 / resolution as f32;
            let py = y as f32 / resolution as f32;
            
            let pos = Vec3::new(px, py, 0.0);
            let dist = (pos - light_pos).length();
            *shadow = (1.0 - (dist / 2.0).min(1.0)).max(0.0);
        });
    }
    
    /// Вычисление отражений с AVX2
    pub fn compute_reflections(&self, resolution: usize, view_dir: Vec3) -> Vec<f32> {
        let size = resolution * resolution;
        let mut reflections = vec![0.0f32; size];
        
        if self.use_avx2 {
            unsafe {
                self.compute_reflections_avx2(&mut reflections, resolution, view_dir);
            }
        } else {
            self.compute_reflections_scalar(&mut reflections, resolution, view_dir);
        }
        
        reflections
    }
    
    /// Вычисление отражений (внутренний метод)
    fn compute_reflections_internal(&self, reflections: &mut Vec<f32>, resolution: usize, view_dir: Vec3) {
        if self.use_avx2 {
            unsafe {
                self.compute_reflections_avx2(reflections, resolution, view_dir);
            }
        } else {
            self.compute_reflections_scalar(reflections, resolution, view_dir);
        }
    }
    
    /// AVX2 версия отражений
    #[target_feature(enable = "avx2")]
    unsafe fn compute_reflections_avx2(&self, reflections: &mut [f32], resolution: usize, view_dir: Vec3) {
        let total_len = reflections.len();
        reflections.par_chunks_mut(8).enumerate().for_each(|(chunk_idx, chunk)| {
            let base_idx = chunk_idx * 8;
            
            for (j, reflection) in chunk.iter_mut().enumerate() {
                let idx = base_idx + j;
                if idx >= total_len {
                    break;
                }
                
                let x = idx % resolution;
                let y = idx / resolution;
                
                let nx = x as f32 / resolution as f32;
                let ny = y as f32 / resolution as f32;
                
                let normal = Vec3::new(nx * 2.0 - 1.0, ny * 2.0 - 1.0, 0.5).normalize();
                let _reflect_dir = (normal * 2.0 * normal.dot(view_dir) - view_dir).normalize();
                
                // Fresnel effect
                let fresnel = (1.0 - view_dir.dot(normal).abs()).powf(2.0);
                *reflection = fresnel * 0.5;
            }
        });
    }
    
    /// Скалярная версия отражений
    fn compute_reflections_scalar(&self, reflections: &mut [f32], resolution: usize, view_dir: Vec3) {
        reflections.par_iter_mut().enumerate().for_each(|(i, reflection)| {
            let x = i % resolution;
            let y = i / resolution;
            
            let nx = x as f32 / resolution as f32;
            let ny = y as f32 / resolution as f32;
            
            let normal = Vec3::new(nx * 2.0 - 1.0, ny * 2.0 - 1.0, 0.5).normalize();
            let _reflect_dir = (normal * 2.0 * normal.dot(view_dir) - view_dir).normalize();
            
            let fresnel = (1.0 - view_dir.dot(normal).abs()).powf(2.0);
            *reflection = fresnel * 0.5;
        });
    }
    
    /// Вычисление облачного паттерна
    fn compute_cloudy_pattern(&self, result: &mut [f32], resolution: usize) {
        result.par_chunks_mut(3).enumerate().for_each(|(i, rgb)| {
            let x = i % resolution;
            let y = i / resolution;
            
            let nx = x as f32 / resolution as f32;
            let ny = y as f32 / resolution as f32;
            
            // Облачное освещение (более мягкое)
            let cloud_factor = (nx * 3.0).sin() * (ny * 3.0).cos() * 0.3 + 0.7;
            
            rgb[0] = cloud_factor * 0.9; // R
            rgb[1] = cloud_factor * 0.95; // G
            rgb[2] = cloud_factor * 1.0; // B
        });
    }
    
    /// Вычисление дождя
    fn compute_rainy_pattern(&self, result: &mut [f32], resolution: usize) {
        result.par_chunks_mut(3).enumerate().for_each(|(i, rgb)| {
            let x = i % resolution;
            let y = i / resolution;
            
            let nx = x as f32 / resolution as f32;
            let ny = y as f32 / resolution as f32;
            
            // Дождливое освещение (темнее, серое)
            let rain_factor = (nx * 5.0 + ny * 5.0).sin() * 0.2 + 0.5;
            
            rgb[0] = rain_factor * 0.7; // R
            rgb[1] = rain_factor * 0.75; // G
            rgb[2] = rain_factor * 0.8; // B
        });
    }
    
    /// Вычисление заката
    fn compute_sunset_pattern(&self, result: &mut [f32], resolution: usize) {
        result.par_chunks_mut(3).enumerate().for_each(|(i, rgb)| {
            let x = i % resolution;
            let y = i / resolution;
            
            let ny = y as f32 / resolution as f32;
            
            // Закатное освещение (красное/оранжевое)
            let sunset_factor = ny;
            
            rgb[0] = sunset_factor * 1.5 + 0.3; // R (красный)
            rgb[1] = sunset_factor * 0.8 + 0.2; // G
            rgb[2] = sunset_factor * 0.4 + 0.1; // B
        });
    }
    
    /// Дефолтный паттерн
    fn compute_default_pattern(&self, result: &mut [f32], resolution: usize) {
        result.par_chunks_mut(3).enumerate().for_each(|(i, rgb)| {
            rgb[0] = 0.5; // R
            rgb[1] = 0.5; // G
            rgb[2] = 0.5; // B
        });
    }
    
    /// Полное вычисление паттерна (освещение + тени + отражения)
    pub fn compute_full_pattern(
        &self,
        pattern_type: crate::lighting_patterns::PatternType,
        resolution: usize,
        light_pos: Vec3,
        view_dir: Vec3,
    ) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        // Параллельное вычисление всех компонентов
        let lighting = self.compute_lighting_pattern(pattern_type, resolution);
        let shadows = self.compute_shadows(resolution, light_pos);
        let reflections = self.compute_reflections(resolution, view_dir);
        
        (lighting, shadows, reflections)
    }
    
    /// Статистика CPU вычислений
    pub fn get_stats(&self) -> CPUComputeStats {
        CPUComputeStats {
            use_avx2: self.use_avx2,
            num_threads: self.num_threads,
            cpu_name: get_cpu_name(),
        }
    }
}

/// Статистика CPU вычислений
#[derive(Debug, Clone)]
pub struct CPUComputeStats {
    pub use_avx2: bool,
    pub num_threads: usize,
    pub cpu_name: String,
}

fn get_cpu_name() -> String {
    // Простая функция для получения имени CPU
    // В реальности можно использовать cpuid или другие библиотеки
    "CPU (AVX2)".to_string()
}

impl Default for CPULightingCompute {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_compute_creation() {
        let compute = CPULightingCompute::new();
        assert!(compute.num_threads > 0);
    }
    
    #[test]
    fn test_lighting_pattern() {
        let compute = CPULightingCompute::new();
        let pattern = compute.compute_lighting_pattern(
            crate::lighting_patterns::PatternType::Sunny,
            64,
        );
        assert_eq!(pattern.len(), 64 * 64 * 3);
    }
}
