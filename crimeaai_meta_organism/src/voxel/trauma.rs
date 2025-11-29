//! Система травмы по Никоновой (2013)
//! 
//! Источник: Никонова М.А. (2013) "Коллизии и травма в многоагентных системах"
//! 
//! Принцип: При столкновении несовместимых агентов возникает травма,
//! которая распространяется каскадно по связям и вызывает:
//! - Локальное повреждение (trauma_level)
//! - Эмоциональный шок (emotion сдвиг)
//! - Активацию защитных механизмов

use super::types::{Voxel9k, VoxelFlags, Creature, CreatureState};
use super::clustering::AlsynbaevClusterizer;
use glam::Vec3;

/// Система обработки травмы
/// 
/// Источник: Никонова М.А. (2013)
pub struct NikonovaTraumaSystem {
    /// Порог расстояния для детекции коллизий
    pub collision_threshold: f32,
    
    /// Базовый урон от коллизии
    pub base_damage: f32,
    
    /// Множитель каскадного распространения
    pub cascade_factor: f32,
    
    /// Скорость восстановления
    pub recovery_rate: f32,
    
    /// Статистика травм
    pub stats: TraumaStats,
}

#[derive(Clone, Debug, Default)]
pub struct TraumaStats {
    /// Всего обработанных коллизий
    pub total_collisions: u64,
    
    /// Суммарный нанесённый урон
    pub total_damage: f32,
    
    /// Количество отторжений
    pub rejections: u32,
    
    /// Количество интеграций
    pub integrations: u32,
}

impl NikonovaTraumaSystem {
    pub fn new() -> Self {
        Self {
            collision_threshold: 1.5,
            base_damage: 0.3,
            cascade_factor: 0.5,
            recovery_rate: 0.02,
            stats: TraumaStats::default(),
        }
    }
    
    /// Проверить коллизию между существом и организмом
    /// 
    /// Источник: Никонова М.А. (2013) — детекция столкновений
    pub fn check_collision(
        &mut self,
        creature: &Creature,
        organism_voxels: &[Voxel9k],
        organism_center: Vec3,
        organism_radius: f32,
    ) -> Option<CollisionResult> {
        // Проверяем расстояние до центра организма
        let distance = (creature.center - organism_center).length();
        
        // Если существо внутри радиуса организма
        if distance < organism_radius + self.collision_threshold {
            // Найти ближайший воксель организма
            let mut nearest_idx = 0;
            let mut min_dist = f32::MAX;
            
            for (idx, v) in organism_voxels.iter().enumerate() {
                if v.creature_id == 0 && v.flags.contains(VoxelFlags::ALIVE) {
                    let d = (Vec3::from(v.position) - creature.center).length();
                    if d < min_dist {
                        min_dist = d;
                        nearest_idx = idx;
                    }
                }
            }
            
            if min_dist < self.collision_threshold {
                self.stats.total_collisions += 1;
                return Some(CollisionResult {
                    organism_voxel_idx: nearest_idx,
                    distance: min_dist,
                    impact_force: 1.0 - (min_dist / self.collision_threshold),
                });
            }
        }
        
        None
    }
    
    /// Обработать результат коллизии: интеграция или отторжение
    /// 
    /// Источник: Никонова М.А. (2013) — реакция на столкновение
    pub fn process_collision(
        &mut self,
        creature: &mut Creature,
        creature_voxels: &mut [Voxel9k],
        organism_voxels: &mut [Voxel9k],
        collision: &CollisionResult,
        clusterizer: &mut AlsynbaevClusterizer,
    ) -> CollisionOutcome {
        // Получить целевой воксель организма
        let target_voxel = &organism_voxels[collision.organism_voxel_idx];
        
        // Найти представительный воксель существа
        let creature_sample = if let Some(&first_idx) = creature.voxel_indices.first() {
            &creature_voxels[first_idx]
        } else {
            return CollisionOutcome::NoReaction;
        };
        
        // Проверить совместимость
        let semantic_sim = creature_sample.semantic_similarity(target_voxel);
        let emotion_sim = creature_sample.emotion_similarity(target_voxel);
        
        let is_compatible = semantic_sim > 0.5 && emotion_sim > 0.4;
        
        if is_compatible {
            // ИНТЕГРАЦИЯ: Зелёный свет, +energy
            self.integrate(creature, creature_voxels, organism_voxels, collision, clusterizer);
            self.stats.integrations += 1;
            CollisionOutcome::Integration {
                semantic_match: semantic_sim,
                emotion_match: emotion_sim,
            }
        } else {
            // ОТТОРЖЕНИЕ: Красный свет, травма
            let damage = self.base_damage * collision.impact_force;
            self.reject(creature, creature_voxels, organism_voxels, collision, clusterizer, damage);
            self.stats.rejections += 1;
            self.stats.total_damage += damage;
            CollisionOutcome::Rejection {
                damage,
                semantic_mismatch: 1.0 - semantic_sim,
                emotion_mismatch: 1.0 - emotion_sim,
            }
        }
    }
    
    /// Интеграция существа в организм
    /// 
    /// Источник: Лавренков Д.Н. (2018) — позитивная перестройка связей
    fn integrate(
        &mut self,
        creature: &mut Creature,
        creature_voxels: &mut [Voxel9k],
        organism_voxels: &mut [Voxel9k],
        collision: &CollisionResult,
        clusterizer: &mut AlsynbaevClusterizer,
    ) {
        creature.state = CreatureState::Integrated;
        
        // Зажечь точку интеграции
        let target_cluster_id = organism_voxels[collision.organism_voxel_idx].cluster_id;
        clusterizer.ignite_cluster(target_cluster_id, 1.0, organism_voxels);
        
        // Перевести воксели существа в организм
        for &idx in &creature.voxel_indices {
            if let Some(v) = creature_voxels.get_mut(idx) {
                v.creature_id = 0; // Теперь часть организма
                v.flags |= VoxelFlags::INTEGRATED;
                v.flags.remove(VoxelFlags::MOVING);
                v.ignite(0.5);
                
                // Остановить движение
                v.velocity = [0.0, 0.0, 0.0];
            }
        }
        
        // Усилить энергию вокруг точки интеграции
        let impact_pos = Vec3::from(organism_voxels[collision.organism_voxel_idx].position);
        for v in organism_voxels.iter_mut() {
            if v.creature_id == 0 {
                let dist = (Vec3::from(v.position) - impact_pos).length();
                if dist < 5.0 {
                    v.ignite(0.3 * (1.0 - dist / 5.0));
                }
            }
        }
    }
    
    /// Отторжение существа с травмой организма
    /// 
    /// Источник: Никонова М.А. (2013) — каскадное распространение травмы
    fn reject(
        &mut self,
        creature: &mut Creature,
        creature_voxels: &mut [Voxel9k],
        organism_voxels: &mut [Voxel9k],
        collision: &CollisionResult,
        clusterizer: &mut AlsynbaevClusterizer,
        damage: f32,
    ) {
        creature.state = CreatureState::Rejected;
        
        // Распространить травму от точки столкновения
        let target_cluster_id = organism_voxels[collision.organism_voxel_idx].cluster_id;
        clusterizer.propagate_damage(target_cluster_id, damage, organism_voxels);
        
        // Применить травму к точке удара
        let impact_pos = Vec3::from(organism_voxels[collision.organism_voxel_idx].position);
        
        for v in organism_voxels.iter_mut() {
            if v.creature_id == 0 {
                let dist = (Vec3::from(v.position) - impact_pos).length();
                if dist < 3.0 {
                    let local_damage = damage * (1.0 - dist / 3.0);
                    v.apply_trauma(local_damage);
                }
            }
        }
        
        // Оттолкнуть существо
        let push_dir = (creature.center - impact_pos).normalize_or_zero();
        for &idx in &creature.voxel_indices {
            if let Some(v) = creature_voxels.get_mut(idx) {
                v.velocity[0] += push_dir.x * 5.0;
                v.velocity[1] += push_dir.y * 5.0;
                v.velocity[2] += push_dir.z * 5.0;
                v.apply_trauma(damage * 0.5);
            }
        }
    }
    
    /// Обновление восстановления (вызывать каждый кадр)
    /// 
    /// Источник: Никонова М.А. (2013) — механизмы восстановления
    pub fn update_recovery(&self, voxels: &mut [Voxel9k], dt: f32) {
        let recovery = self.recovery_rate * dt;
        
        for v in voxels.iter_mut() {
            if v.flags.contains(VoxelFlags::ALIVE) && !v.flags.contains(VoxelFlags::DEAD) {
                // Постепенное восстановление от травмы
                v.trauma_level = (v.trauma_level - recovery).max(0.0);
                
                // Если травма упала ниже порога, снять флаг
                if v.trauma_level < 0.05 {
                    v.flags.remove(VoxelFlags::TRAUMATIZED);
                }
                
                // Восстановление энергии
                v.energy = (v.energy + recovery * 0.5).min(1.0);
            }
        }
    }
}

impl Default for NikonovaTraumaSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Результат детекции коллизии
#[derive(Clone, Debug)]
pub struct CollisionResult {
    /// Индекс вокселя организма, с которым столкнулись
    pub organism_voxel_idx: usize,
    
    /// Расстояние до точки столкновения
    pub distance: f32,
    
    /// Сила удара (0.0 - 1.0)
    pub impact_force: f32,
}

/// Исход коллизии
#[derive(Clone, Debug)]
pub enum CollisionOutcome {
    /// Успешная интеграция
    Integration {
        semantic_match: f32,
        emotion_match: f32,
    },
    
    /// Отторжение с травмой
    Rejection {
        damage: f32,
        semantic_mismatch: f32,
        emotion_mismatch: f32,
    },
    
    /// Нет реакции
    NoReaction,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trauma_system() {
        let system = NikonovaTraumaSystem::new();
        
        // Создать воксели
        let mut voxels: Vec<Voxel9k> = (0..100)
            .map(|i| Voxel9k::new([i as f32, 0.0, 0.0]))
            .collect();
        
        // Применить травму
        voxels[50].apply_trauma(0.5);
        assert!(voxels[50].trauma_level > 0.0);
        
        // Восстановление
        system.update_recovery(&mut voxels, 10.0);
        assert!(voxels[50].trauma_level < 0.5);
    }
}
