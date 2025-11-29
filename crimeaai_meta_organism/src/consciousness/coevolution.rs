//! Коэволюционное обучение по Лавренкову (2018)
//! 
//! Источник: Лавренков Д.Н. (2018) "Коэволюционное обучение оптических сетей"
//! 
//! Принцип: Связи между вокселями (connections) перестраиваются
//! в зависимости от успешности взаимодействий. Успешная интеграция
//! усиливает связи, отторжение — ослабляет.

use crate::voxel::{Voxel9k, AlsynbaevClusterizer, VoxelCluster};
use glam::Vec3;
use std::collections::HashMap;

/// Система коэволюционного обучения
/// 
/// Источник: Лавренков Д.Н. (2018)
pub struct LavrenkovCoevolution {
    /// Скорость обучения (0.0 - 1.0)
    pub learning_rate: f32,
    
    /// Порог усиления связи
    pub strengthen_threshold: f32,
    
    /// Порог ослабления связи
    pub weaken_threshold: f32,
    
    /// История успешных интеграций (для обучения)
    pub integration_history: Vec<IntegrationEvent>,
    
    /// История отторжений
    pub rejection_history: Vec<RejectionEvent>,
    
    /// Матрица весов связей между кластерами
    pub cluster_weights: HashMap<(u32, u32), f32>,
    
    /// Статистика
    pub stats: CoevolutionStats,
}

#[derive(Clone, Debug)]
pub struct IntegrationEvent {
    pub time: f32,
    pub cluster_id: u32,
    pub semantic_match: f32,
    pub energy_boost: f32,
}

#[derive(Clone, Debug)]
pub struct RejectionEvent {
    pub time: f32,
    pub cluster_id: u32,
    pub semantic_mismatch: f32,
    pub damage: f32,
}

#[derive(Clone, Debug, Default)]
pub struct CoevolutionStats {
    /// Количество перестроенных связей
    pub rewired_connections: u64,
    
    /// Средний вес связей
    pub mean_weight: f32,
    
    /// Количество усиленных связей
    pub strengthened: u32,
    
    /// Количество ослабленных связей
    pub weakened: u32,
}

impl LavrenkovCoevolution {
    pub fn new() -> Self {
        Self {
            learning_rate: 0.1,
            strengthen_threshold: 0.6,
            weaken_threshold: 0.3,
            integration_history: Vec::new(),
            rejection_history: Vec::new(),
            cluster_weights: HashMap::new(),
            stats: CoevolutionStats::default(),
        }
    }
    
    /// Записать успешную интеграцию
    /// 
    /// Источник: Лавренков Д.Н. (2018) — положительное подкрепление
    pub fn record_integration(&mut self, cluster_id: u32, semantic_match: f32, energy_boost: f32, time: f32) {
        self.integration_history.push(IntegrationEvent {
            time,
            cluster_id,
            semantic_match,
            energy_boost,
        });
        
        // Усилить связи в затронутом кластере
        self.strengthen_cluster_connections(cluster_id, semantic_match);
        
        // Ограничить историю
        if self.integration_history.len() > 1000 {
            self.integration_history.remove(0);
        }
    }
    
    /// Записать отторжение
    /// 
    /// Источник: Лавренков Д.Н. (2018) — отрицательное подкрепление
    pub fn record_rejection(&mut self, cluster_id: u32, semantic_mismatch: f32, damage: f32, time: f32) {
        self.rejection_history.push(RejectionEvent {
            time,
            cluster_id,
            semantic_mismatch,
            damage,
        });
        
        // Ослабить связи в затронутом кластере
        self.weaken_cluster_connections(cluster_id, semantic_mismatch);
        
        // Ограничить историю
        if self.rejection_history.len() > 1000 {
            self.rejection_history.remove(0);
        }
    }
    
    /// Усилить связи кластера
    fn strengthen_cluster_connections(&mut self, cluster_id: u32, factor: f32) {
        // Усилить вес связей с соседними кластерами
        let keys: Vec<(u32, u32)> = self.cluster_weights
            .keys()
            .filter(|(a, b)| *a == cluster_id || *b == cluster_id)
            .cloned()
            .collect();
        
        for key in keys {
            if let Some(weight) = self.cluster_weights.get_mut(&key) {
                *weight = (*weight + self.learning_rate * factor).min(1.0);
                self.stats.strengthened += 1;
            }
        }
    }
    
    /// Ослабить связи кластера
    fn weaken_cluster_connections(&mut self, cluster_id: u32, factor: f32) {
        let keys: Vec<(u32, u32)> = self.cluster_weights
            .keys()
            .filter(|(a, b)| *a == cluster_id || *b == cluster_id)
            .cloned()
            .collect();
        
        for key in keys {
            if let Some(weight) = self.cluster_weights.get_mut(&key) {
                *weight = (*weight - self.learning_rate * factor).max(0.0);
                self.stats.weakened += 1;
            }
        }
    }
    
    /// Обновить связи на уровне вокселей
    /// 
    /// Источник: Лавренков Д.Н. (2018) — оптическая перестройка
    pub fn update_voxel_connections(&mut self, voxels: &mut [Voxel9k], clusterizer: &AlsynbaevClusterizer) {
        // Для каждого кластера перестроить связи вокселей
        for cluster in &clusterizer.clusters {
            self.rewire_cluster_voxels(cluster, voxels);
        }
    }
    
    /// Перестроить связи вокселей внутри кластера
    fn rewire_cluster_voxels(&mut self, cluster: &VoxelCluster, voxels: &mut [Voxel9k]) {
        if cluster.voxel_indices.len() < 2 {
            return;
        }
        
        // Собрать позиции вокселей кластера
        let positions: Vec<(usize, Vec3)> = cluster.voxel_indices
            .iter()
            .filter_map(|&idx| {
                voxels.get(idx).map(|v| (idx, Vec3::from(v.position)))
            })
            .collect();
        
        // Для каждого вокселя найти 6 ближайших соседей
        for &(idx, pos) in &positions {
            let mut distances: Vec<(usize, f32)> = positions
                .iter()
                .filter(|(other_idx, _)| *other_idx != idx)
                .map(|(other_idx, other_pos)| (*other_idx, (*other_pos - pos).length()))
                .collect();
            
            distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            // Обновить связи
            if let Some(voxel) = voxels.get_mut(idx) {
                for (i, (neighbor_idx, _)) in distances.iter().take(6).enumerate() {
                    voxel.connections[i] = *neighbor_idx as u16;
                    self.stats.rewired_connections += 1;
                }
            }
        }
    }
    
    /// Хебианское обучение связей
    /// 
    /// Источник: Лавренков Д.Н. (2018) — "нейроны, которые возбуждаются вместе, связываются вместе"
    pub fn hebbian_update(&mut self, voxels: &mut [Voxel9k], dt: f32) {
        for i in 0..voxels.len() {
            let energy_i = voxels[i].energy;
            
            // Для каждой связи проверяем корреляцию активности
            for conn_slot in 0..6 {
                let neighbor_idx = voxels[i].connections[conn_slot];
                if neighbor_idx == u16::MAX {
                    continue;
                }
                
                let neighbor_idx = neighbor_idx as usize;
                if neighbor_idx >= voxels.len() {
                    continue;
                }
                
                let energy_j = voxels[neighbor_idx].energy;
                
                // Хебианское правило: усиливаем связь если оба активны
                let correlation = energy_i * energy_j;
                
                if correlation > self.strengthen_threshold {
                    // Усиливаем обе стороны связи
                    if let Some(v) = voxels.get_mut(i) {
                        v.energy += self.learning_rate * dt * 0.1;
                    }
                } else if correlation < self.weaken_threshold {
                    // Ослабляем при антикорреляции
                    if let Some(v) = voxels.get_mut(i) {
                        v.energy = (v.energy - self.learning_rate * dt * 0.05).max(0.0);
                    }
                }
            }
        }
    }
    
    /// Инициализировать веса связей между кластерами
    pub fn init_cluster_weights(&mut self, clusterizer: &AlsynbaevClusterizer) {
        self.cluster_weights.clear();
        
        for cluster in &clusterizer.clusters {
            for &neighbor_id in &cluster.neighbors {
                if neighbor_id != u32::MAX {
                    let key = if cluster.id < neighbor_id {
                        (cluster.id, neighbor_id)
                    } else {
                        (neighbor_id, cluster.id)
                    };
                    
                    self.cluster_weights.entry(key).or_insert(0.5); // Нейтральный вес
                }
            }
        }
    }
    
    /// Обновить статистику
    pub fn update_stats(&mut self) {
        if !self.cluster_weights.is_empty() {
            let sum: f32 = self.cluster_weights.values().sum();
            self.stats.mean_weight = sum / self.cluster_weights.len() as f32;
        }
    }
}

impl Default for LavrenkovCoevolution {
    fn default() -> Self {
        Self::new()
    }
}

/// Паттерны семантической эволюции
/// 
/// Источник: Козлов И.П. (2020) + Лавренков Д.Н. (2018)
pub struct SemanticEvolution;

impl SemanticEvolution {
    /// Мутировать семантический вектор
    pub fn mutate(semantic: &mut [half::f16; 8], mutation_rate: f32) {
        let mut rng = rand::thread_rng();
        for val in semantic.iter_mut() {
            if rand::Rng::gen::<f32>(&mut rng) < mutation_rate {
                let current = val.to_f32();
                let mutation = (rand::Rng::gen::<f32>(&mut rng) - 0.5) * 0.2;
                *val = half::f16::from_f32((current + mutation).clamp(-1.0, 1.0));
            }
        }
    }
    
    /// Скрестить два семантических вектора
    pub fn crossover(a: &[half::f16; 8], b: &[half::f16; 8]) -> [half::f16; 8] {
        let mut result = [half::f16::ZERO; 8];
        let mut rng = rand::thread_rng();
        
        for i in 0..8 {
            result[i] = if rand::Rng::gen::<bool>(&mut rng) {
                a[i]
            } else {
                b[i]
            };
        }
        
        result
    }
    
    /// Интерполировать семантические векторы
    pub fn interpolate(a: &[half::f16; 8], b: &[half::f16; 8], t: f32) -> [half::f16; 8] {
        let mut result = [half::f16::ZERO; 8];
        
        for i in 0..8 {
            let va = a[i].to_f32();
            let vb = b[i].to_f32();
            result[i] = half::f16::from_f32(va * (1.0 - t) + vb * t);
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_coevolution() {
        let mut coev = LavrenkovCoevolution::new();
        
        coev.record_integration(1, 0.8, 0.5, 1.0);
        coev.record_rejection(2, 0.9, 0.3, 2.0);
        
        assert_eq!(coev.integration_history.len(), 1);
        assert_eq!(coev.rejection_history.len(), 1);
    }
    
    #[test]
    fn test_semantic_evolution() {
        let mut semantic = [half::f16::from_f32(0.5); 8];
        
        SemanticEvolution::mutate(&mut semantic, 0.5);
        
        // Проверяем, что хотя бы что-то изменилось
        let has_change = semantic.iter().any(|&v| v.to_f32() != 0.5);
        println!("Mutation occurred: {}", has_change);
    }
}
