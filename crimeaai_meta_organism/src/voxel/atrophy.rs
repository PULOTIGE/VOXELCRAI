//! Система атрофии по Ахмадуллиной (2015)
//! 
//! Источник: Ахмадуллина Р.Ф. (2015) "VBM-паттерны атрофии нейронных структур"
//! 
//! Принцип: Воксели с высокой травмой и низкой энергией подвергаются
//! атрофии — постепенному отмиранию. VBM (Voxel-Based Morphometry)
//! паттерны определяют, какие области организма наиболее уязвимы.

use super::types::{Voxel9k, VoxelFlags};
use super::clustering::AlsynbaevClusterizer;
use glam::Vec3;

/// Система атрофии
/// 
/// Источник: Ахмадуллина Р.Ф. (2015)
pub struct AkhmadullinaAtrophySystem {
    /// Порог травмы для начала атрофии
    pub trauma_threshold: f32,
    
    /// Порог энергии для защиты от атрофии
    pub energy_protection_threshold: f32,
    
    /// Скорость атрофии (за секунду)
    pub atrophy_rate: f32,
    
    /// Порог смерти (atrophy_factor при котором воксель умирает)
    pub death_threshold: f32,
    
    /// VBM карта уязвимости (по кластерам)
    pub vbm_vulnerability: Vec<f32>,
    
    /// Статистика
    pub stats: AtrophyStats,
}

#[derive(Clone, Debug, Default)]
pub struct AtrophyStats {
    /// Количество отмерших вокселей
    pub dead_voxels: u32,
    
    /// Количество атрофирующихся вокселей
    pub atrophying_voxels: u32,
    
    /// Средний уровень атрофии
    pub mean_atrophy: f32,
    
    /// Максимальный уровень атрофии
    pub max_atrophy: f32,
}

impl AkhmadullinaAtrophySystem {
    pub fn new() -> Self {
        Self {
            trauma_threshold: 0.4,
            energy_protection_threshold: 0.3,
            atrophy_rate: 0.05,
            death_threshold: 0.95,
            vbm_vulnerability: Vec::new(),
            stats: AtrophyStats::default(),
        }
    }
    
    /// Построить VBM карту уязвимости
    /// 
    /// Источник: Ахмадуллина Р.Ф. (2015) — анализ морфометрии
    pub fn build_vbm_map(&mut self, _voxels: &[Voxel9k], clusterizer: &AlsynbaevClusterizer) {
        self.vbm_vulnerability.clear();
        
        for cluster in &clusterizer.clusters {
            // Вычислить уязвимость кластера на основе:
            // - Расстояния от центра (периферия более уязвима)
            // - Связности (изолированные кластеры уязвимее)
            // - Среднего уровня энергии
            
            let distance_from_center = cluster.centroid.length();
            let connectivity = cluster.neighbors.iter()
                .filter(|&&n| n != u32::MAX)
                .count() as f32;
            
            // Уязвимость обратно пропорциональна связности и здоровью
            let vulnerability = (distance_from_center / 20.0).min(1.0) 
                * (1.0 - connectivity / 4.0).max(0.2)
                * (1.0 - cluster.health);
            
            self.vbm_vulnerability.push(vulnerability);
        }
    }
    
    /// Обновить атрофию вокселей
    /// 
    /// Источник: Ахмадуллина Р.Ф. (2015) — прогрессия атрофии
    pub fn update(&mut self, voxels: &mut [Voxel9k], clusterizer: &AlsynbaevClusterizer, dt: f32) {
        self.stats = AtrophyStats::default();
        
        let mut sum_atrophy = 0.0f32;
        let mut atrophy_count = 0u32;
        
        for voxel in voxels.iter_mut() {
            if !voxel.flags.contains(VoxelFlags::ALIVE) {
                continue;
            }
            
            // Получить уязвимость по VBM карте
            let vulnerability = self.get_cluster_vulnerability(voxel.cluster_id, clusterizer);
            
            // Условия для атрофии:
            // 1. Высокая травма ИЛИ
            // 2. Низкая энергия И высокая уязвимость
            let should_atrophy = 
                voxel.trauma_level > self.trauma_threshold ||
                (voxel.energy < self.energy_protection_threshold && vulnerability > 0.5);
            
            if should_atrophy {
                // Скорость атрофии зависит от травмы и уязвимости
                let rate = self.atrophy_rate * dt * (1.0 + vulnerability) * (1.0 + voxel.trauma_level);
                voxel.apply_atrophy(rate);
                
                self.stats.atrophying_voxels += 1;
            } else {
                // Восстановление при хорошем состоянии
                if voxel.energy > 0.7 && voxel.trauma_level < 0.1 {
                    voxel.atrophy_factor = (voxel.atrophy_factor - self.atrophy_rate * dt * 0.3).max(0.0);
                }
            }
            
            // Проверка на смерть
            if voxel.atrophy_factor >= self.death_threshold {
                voxel.flags |= VoxelFlags::DEAD;
                voxel.flags.remove(VoxelFlags::ALIVE);
                self.stats.dead_voxels += 1;
            }
            
            // Статистика
            sum_atrophy += voxel.atrophy_factor;
            atrophy_count += 1;
            self.stats.max_atrophy = self.stats.max_atrophy.max(voxel.atrophy_factor);
        }
        
        if atrophy_count > 0 {
            self.stats.mean_atrophy = sum_atrophy / atrophy_count as f32;
        }
    }
    
    /// Получить уязвимость кластера
    fn get_cluster_vulnerability(&self, cluster_id: u32, clusterizer: &AlsynbaevClusterizer) -> f32 {
        // Найти индекс кластера
        if let Some(idx) = clusterizer.clusters.iter().position(|c| c.id == cluster_id) {
            if idx < self.vbm_vulnerability.len() {
                return self.vbm_vulnerability[idx];
            }
        }
        0.5 // Базовая уязвимость
    }
    
    /// Удалить мёртвые воксели (вызывать периодически)
    pub fn cleanup_dead_voxels(&self, voxels: &mut Vec<Voxel9k>) -> usize {
        let original_len = voxels.len();
        voxels.retain(|v| !v.flags.contains(VoxelFlags::DEAD));
        original_len - voxels.len()
    }
    
    /// Получить области критической атрофии
    /// 
    /// Источник: Ахмадуллина Р.Ф. (2015) — зоны высокого риска
    pub fn get_critical_regions(&self, voxels: &[Voxel9k]) -> Vec<Vec3> {
        let mut critical_points = Vec::new();
        
        for voxel in voxels {
            if voxel.atrophy_factor > 0.7 && voxel.flags.contains(VoxelFlags::ALIVE) {
                critical_points.push(Vec3::from(voxel.position));
            }
        }
        
        critical_points
    }
    
    /// Применить массовую атрофию в области (для визуальных эффектов)
    pub fn apply_area_atrophy(&self, voxels: &mut [Voxel9k], center: Vec3, radius: f32, intensity: f32) {
        for voxel in voxels.iter_mut() {
            let dist = (Vec3::from(voxel.position) - center).length();
            if dist < radius {
                let falloff = 1.0 - (dist / radius);
                voxel.apply_atrophy(intensity * falloff);
            }
        }
    }
}

impl Default for AkhmadullinaAtrophySystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Визуализация атрофии (для отладки)
pub struct AtrophyVisualizer;

impl AtrophyVisualizer {
    /// Сгенерировать цвет для уровня атрофии
    pub fn atrophy_color(atrophy: f32) -> [f32; 4] {
        // От здорового (синий) к атрофированному (серый/прозрачный)
        let health = 1.0 - atrophy;
        [
            0.3 + atrophy * 0.4,  // R: серый при атрофии
            0.3 * health,         // G: уменьшается
            0.6 * health,         // B: уменьшается
            0.3 + health * 0.7,   // A: полупрозрачный при атрофии
        ]
    }
    
    /// Сгенерировать цвет для VBM уязвимости (для отладки)
    pub fn vulnerability_color(vulnerability: f32) -> [f32; 4] {
        // От безопасного (зелёный) к уязвимому (красный)
        [
            vulnerability,
            1.0 - vulnerability,
            0.0,
            0.8,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_atrophy_system() {
        let mut system = AkhmadullinaAtrophySystem::new();
        let clusterizer = AlsynbaevClusterizer::new();
        
        let mut voxels: Vec<Voxel9k> = (0..100)
            .map(|i| Voxel9k::new([i as f32, 0.0, 0.0]))
            .collect();
        
        // Применить травму к некоторым вокселям
        voxels[50].trauma_level = 0.8;
        voxels[51].trauma_level = 0.9;
        
        // Запустить атрофию
        for _ in 0..100 {
            system.update(&mut voxels, &clusterizer, 0.1);
        }
        
        println!("Dead voxels: {}", system.stats.dead_voxels);
        println!("Mean atrophy: {}", system.stats.mean_atrophy);
    }
}
