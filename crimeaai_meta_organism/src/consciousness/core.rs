//! Ядро сознания Meta-Organism
//! 
//! Центральный организм — пульсирующая сфера из вокселей,
//! способная принимать или отторгать новые существа.

use crate::voxel::{
    Voxel9k, VoxelFlags, Creature, CreatureState,
    CompressedVoxelStorage, AlsynbaevClusterizer,
    NikonovaTraumaSystem, AkhmadullinaAtrophySystem,
    LightPattern1KB, VoxelGpuInstance,
};
use glam::Vec3;
use half::f16;
use rand::Rng;
use sha2::{Sha256, Digest};
use std::collections::VecDeque;

/// Главный организм — Meta-Organism
pub struct MetaOrganism {
    /// Хранилище вокселей
    pub voxels: CompressedVoxelStorage,
    
    /// Система кластеризации
    pub clusterizer: AlsynbaevClusterizer,
    
    /// Система травмы
    pub trauma_system: NikonovaTraumaSystem,
    
    /// Система атрофии
    pub atrophy_system: AkhmadullinaAtrophySystem,
    
    /// Центр организма
    pub center: Vec3,
    
    /// Радиус организма
    pub radius: f32,
    
    /// Активные существа (из drag-and-drop)
    pub creatures: Vec<Creature>,
    
    /// Счётчик ID для существ
    next_creature_id: u32,
    
    /// Время жизни
    pub lifetime: f32,
    
    /// Глобальный уровень здоровья (0.0 - 1.0)
    pub health: f32,
    
    /// Пульсация (для визуального эффекта)
    pub pulse_phase: f32,
    
    /// Паттерны освещения
    pub light_patterns: Vec<LightPattern1KB>,
    
    /// Текущий режим: Normal / Ignite / Trauma
    pub mode: OrganismMode,
    
    /// История событий для UI
    pub event_log: VecDeque<OrganismEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrganismMode {
    Normal,
    Ignite,  // Зелёный свет — интеграция
    Trauma,  // Красный свет — отторжение
}

#[derive(Clone, Debug)]
pub struct OrganismEvent {
    pub time: f32,
    pub event_type: EventType,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum EventType {
    Integration,
    Rejection,
    FileDropped,
    AtrophyWarning,
    HealthCritical,
}

impl MetaOrganism {
    /// Создать новый организм
    pub fn new() -> Self {
        let mut organism = Self {
            voxels: CompressedVoxelStorage::new(),
            clusterizer: AlsynbaevClusterizer::new(),
            trauma_system: NikonovaTraumaSystem::new(),
            atrophy_system: AkhmadullinaAtrophySystem::new(),
            center: Vec3::ZERO,
            radius: 15.0,
            creatures: Vec::new(),
            next_creature_id: 1,
            lifetime: 0.0,
            health: 1.0,
            pulse_phase: 0.0,
            light_patterns: vec![LightPattern1KB::default()],
            mode: OrganismMode::Normal,
            event_log: VecDeque::with_capacity(100),
        };
        
        // Создать начальную структуру — пульсирующую сферу
        organism.spawn_initial_organism(5000);
        
        organism
    }
    
    /// Создать начальную структуру организма (сфера из вокселей)
    fn spawn_initial_organism(&mut self, count: usize) {
        let mut rng = rand::thread_rng();
        
        for _ in 0..count {
            // Сферическое распределение с вариациями
            let theta = rng.gen::<f32>() * std::f32::consts::TAU;
            let phi = rng.gen::<f32>() * std::f32::consts::PI;
            let r = self.radius * (0.3 + rng.gen::<f32>() * 0.7);
            
            let x = r * phi.sin() * theta.cos();
            let y = r * phi.sin() * theta.sin();
            let z = r * phi.cos();
            
            let mut voxel = Voxel9k::new([x, y, z]);
            voxel.creature_id = 0; // Принадлежит главному организму
            voxel.flags = VoxelFlags::ALIVE | VoxelFlags::CORE;
            voxel.energy = 0.5 + rng.gen::<f32>() * 0.3;
            
            // Семантический вектор организма (базовый паттерн)
            voxel.semantic_vector = [
                f16::from_f32(0.5 + rng.gen::<f32>() * 0.2),
                f16::from_f32(0.5 + rng.gen::<f32>() * 0.2),
                f16::from_f32(0.5),
                f16::from_f32(0.5),
                f16::from_f32(0.3),
                f16::from_f32(0.3),
                f16::from_f32(0.4),
                f16::from_f32(0.4),
            ];
            
            self.voxels.push(voxel);
        }
        
        // Кластеризация
        self.clusterizer.clusterize(self.voxels.as_mut_slice());
        
        // Построить VBM карту
        self.atrophy_system.build_vbm_map(self.voxels.as_slice(), &self.clusterizer);
    }
    
    /// Обработать drag-and-drop файла
    /// 
    /// Создаёт новое существо из хеша файла
    pub fn handle_file_drop(&mut self, file_path: &str, file_data: &[u8]) {
        let creature_id = self.next_creature_id;
        self.next_creature_id += 1;
        
        // Генерируем семантический вектор из хеша файла
        let semantic = Self::file_to_semantic(file_data);
        
        // Создаём существо — маленькая сфера
        let spawn_distance = self.radius * 2.5;
        let mut rng = rand::thread_rng();
        let angle = rng.gen::<f32>() * std::f32::consts::TAU;
        
        let spawn_pos = Vec3::new(
            spawn_distance * angle.cos(),
            spawn_distance * angle.sin(),
            rng.gen::<f32>() * 10.0 - 5.0,
        );
        
        let mut creature = Creature::new(creature_id, spawn_pos, file_path.to_string());
        creature.target = self.center;
        creature.semantic = semantic;
        
        // Размер существа зависит от размера файла
        let voxel_count = (file_data.len() / 100).clamp(50, 500);
        
        // Создать воксели существа
        let _start_idx = self.voxels.len();
        
        for i in 0..voxel_count {
            let theta = (i as f32 / voxel_count as f32) * std::f32::consts::TAU;
            let phi = (i as f32 / (voxel_count as f32 / 10.0)) * std::f32::consts::PI;
            let r = 2.0 + rng.gen::<f32>() * 1.0;
            
            let local_x = r * phi.sin() * theta.cos();
            let local_y = r * phi.sin() * theta.sin();
            let local_z = r * phi.cos();
            
            let pos = [
                spawn_pos.x + local_x,
                spawn_pos.y + local_y,
                spawn_pos.z + local_z,
            ];
            
            let voxel = Voxel9k::new_creature_voxel(pos, creature_id, semantic);
            creature.voxel_indices.push(self.voxels.len());
            self.voxels.push(voxel);
        }
        
        self.creatures.push(creature);
        
        // Записать событие
        self.log_event(EventType::FileDropped, format!("Файл: {} → {} вокселей", file_path, voxel_count));
    }
    
    /// Преобразовать данные файла в семантический вектор
    fn file_to_semantic(data: &[u8]) -> [f16; 8] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        
        let mut semantic = [f16::ZERO; 8];
        for i in 0..8 {
            // Берём байты хеша и нормализуем в [-1, 1]
            let byte_val = hash.get(i * 4).copied().unwrap_or(128) as f32;
            semantic[i] = f16::from_f32((byte_val / 255.0) * 2.0 - 1.0);
        }
        semantic
    }
    
    /// Основной цикл обновления
    pub fn update(&mut self, dt: f32) {
        self.lifetime += dt;
        self.pulse_phase += dt * 2.0;
        
        // 1. Обновить физику и движение существ
        self.update_creatures(dt);
        
        // 2. Проверить коллизии
        self.check_collisions();
        
        // 3. Восстановление от травмы
        self.trauma_system.update_recovery(self.voxels.as_mut_slice(), dt);
        
        // 4. Обновить атрофию
        self.atrophy_system.update(self.voxels.as_mut_slice(), &self.clusterizer, dt);
        
        // 5. Обновить пульсацию организма
        self.update_pulsation(dt);
        
        // 6. Пересчитать здоровье
        self.update_health();
        
        // 7. Определить режим
        self.update_mode();
        
        // 8. Очистка мёртвых существ
        self.cleanup_dead_creatures();
    }
    
    /// Обновить движение существ
    fn update_creatures(&mut self, dt: f32) {
        for creature in &mut self.creatures {
            if creature.state == CreatureState::Moving || creature.state == CreatureState::Approaching {
                // Двигаться к центру организма
                let direction = (creature.target - creature.center).normalize_or_zero();
                let distance = (creature.target - creature.center).length();
                
                // Замедление при приближении
                let speed = if distance < self.radius * 1.5 {
                    creature.speed * 0.5
                } else {
                    creature.speed
                };
                
                if distance < self.radius + 3.0 {
                    creature.state = CreatureState::Approaching;
                }
                
                // Двигать воксели существа
                for &idx in &creature.voxel_indices {
                    if let Some(v) = self.voxels.get_mut(idx) {
                        v.velocity[0] = direction.x * speed;
                        v.velocity[1] = direction.y * speed;
                        v.velocity[2] = direction.z * speed;
                        v.update_physics(dt);
                    }
                }
                
                // Обновить центр масс
                creature.update_center(self.voxels.as_slice());
            }
            
            // Умирающие существа
            if creature.state == CreatureState::Dying {
                for &idx in &creature.voxel_indices {
                    if let Some(v) = self.voxels.get_mut(idx) {
                        v.apply_atrophy(dt * 0.5);
                    }
                }
            }
        }
    }
    
    /// Проверить и обработать коллизии
    fn check_collisions(&mut self) {
        // Собираем данные о столкновениях
        let mut collision_results: Vec<(usize, crate::voxel::CollisionResult)> = Vec::new();
        
        {
            let voxels_slice = self.voxels.as_slice();
            
            for (idx, creature) in self.creatures.iter().enumerate() {
                if creature.state != CreatureState::Approaching {
                    continue;
                }
                
                // Проверить коллизию
                if let Some(collision) = self.trauma_system.check_collision(
                    creature,
                    voxels_slice,
                    self.center,
                    self.radius,
                ) {
                    collision_results.push((idx, collision));
                }
            }
        }
        
        // Обработать столкновения отдельно
        for (creature_idx, collision) in collision_results {
            let creature = &self.creatures[creature_idx];
            
            // Получить семантические данные для проверки совместимости
            let is_compatible = if let Some(&first_idx) = creature.voxel_indices.first() {
                if let (Some(creature_voxel), Some(target_voxel)) = (
                    self.voxels.get(first_idx),
                    self.voxels.get(collision.organism_voxel_idx)
                ) {
                    let semantic_sim = creature_voxel.semantic_similarity(target_voxel);
                    let emotion_sim = creature_voxel.emotion_similarity(target_voxel);
                    (semantic_sim > 0.5, emotion_sim > 0.4, semantic_sim, emotion_sim)
                } else {
                    (false, false, 0.0, 0.0)
                }
            } else {
                (false, false, 0.0, 0.0)
            };
            
            let compatible = is_compatible.0 && is_compatible.1;
            let semantic_sim = is_compatible.2;
            let emotion_sim = is_compatible.3;
            
            if compatible {
                // ИНТЕГРАЦИЯ
                self.mode = OrganismMode::Ignite;
                
                // Зажечь точку интеграции
                let target_cluster_id = self.voxels.get(collision.organism_voxel_idx)
                    .map(|v| v.cluster_id)
                    .unwrap_or(0);
                self.clusterizer.ignite_cluster(target_cluster_id, 1.0, self.voxels.as_mut_slice());
                
                // Интегрировать воксели существа
                let creature = &mut self.creatures[creature_idx];
                creature.state = CreatureState::Integrated;
                for &idx in &creature.voxel_indices {
                    if let Some(v) = self.voxels.get_mut(idx) {
                        v.creature_id = 0;
                        v.flags |= VoxelFlags::INTEGRATED;
                        v.flags.remove(VoxelFlags::MOVING);
                        v.ignite(0.5);
                        v.velocity = [0.0, 0.0, 0.0];
                    }
                }
                
                self.log_event(
                    EventType::Integration,
                    format!("Интеграция! Sem: {:.0}%, Emo: {:.0}%", 
                        semantic_sim * 100.0, emotion_sim * 100.0)
                );
            } else {
                // ОТТОРЖЕНИЕ
                self.mode = OrganismMode::Trauma;
                let damage = self.trauma_system.base_damage * collision.impact_force;
                
                // Распространить травму
                let target_cluster_id = self.voxels.get(collision.organism_voxel_idx)
                    .map(|v| v.cluster_id)
                    .unwrap_or(0);
                self.clusterizer.propagate_damage(target_cluster_id, damage, self.voxels.as_mut_slice());
                
                // Оттолкнуть существо
                let creature = &mut self.creatures[creature_idx];
                creature.state = CreatureState::Dying;
                
                let impact_pos = self.voxels.get(collision.organism_voxel_idx)
                    .map(|v| Vec3::from(v.position))
                    .unwrap_or(Vec3::ZERO);
                let push_dir = (creature.center - impact_pos).normalize_or_zero();
                
                for &idx in &creature.voxel_indices {
                    if let Some(v) = self.voxels.get_mut(idx) {
                        v.velocity[0] += push_dir.x * 5.0;
                        v.velocity[1] += push_dir.y * 5.0;
                        v.velocity[2] += push_dir.z * 5.0;
                        v.apply_trauma(damage * 0.5);
                    }
                }
                
                self.trauma_system.stats.rejections += 1;
                self.trauma_system.stats.total_damage += damage;
                
                self.log_event(
                    EventType::Rejection,
                    format!("Отторжение! Урон: {:.0}%", damage * 100.0)
                );
            }
        }
    }
    
    /// Обновить пульсацию
    fn update_pulsation(&mut self, dt: f32) {
        let _pulse = (self.pulse_phase).sin() * 0.5 + 0.5;
        
        // Применить пульсацию к core вокселям
        for v in self.voxels.iter_mut() {
            if v.creature_id == 0 && v.flags.contains(VoxelFlags::CORE) {
                // Лёгкое "дыхание" организма
                let dist = Vec3::from(v.position).length();
                let phase_offset = dist * 0.1;
                let local_pulse = ((self.pulse_phase + phase_offset).sin() * 0.5 + 0.5) * 0.1;
                
                // Модулировать энергию пульсацией
                v.energy = (v.energy + local_pulse * dt - 0.05 * dt).clamp(0.1, 1.5);
            }
        }
    }
    
    /// Обновить общее здоровье
    fn update_health(&mut self) {
        let cluster_health = self.clusterizer.total_health();
        let atrophy_penalty = self.atrophy_system.stats.mean_atrophy;
        let trauma_penalty = self.trauma_system.stats.total_damage / 100.0;
        
        self.health = (cluster_health - atrophy_penalty * 0.5 - trauma_penalty * 0.1).clamp(0.0, 1.0);
        
        if self.health < 0.3 {
            self.log_event(EventType::HealthCritical, format!("Здоровье критично: {:.0}%", self.health * 100.0));
        }
    }
    
    /// Обновить режим (автоматический переход в Normal)
    fn update_mode(&mut self) {
        // Режим возвращается к Normal через некоторое время
        if self.mode != OrganismMode::Normal {
            // Проверяем, прошло ли достаточно времени с последнего события
            if let Some(last_event) = self.event_log.back() {
                if self.lifetime - last_event.time > 2.0 {
                    self.mode = OrganismMode::Normal;
                }
            }
        }
    }
    
    /// Очистка мёртвых существ
    fn cleanup_dead_creatures(&mut self) {
        self.creatures.retain(|c| {
            c.state != CreatureState::Dying || {
                // Проверить, все ли воксели мертвы
                c.voxel_indices.iter().any(|&idx| {
                    self.voxels.get(idx)
                        .map(|v| v.flags.contains(VoxelFlags::ALIVE))
                        .unwrap_or(false)
                })
            }
        });
    }
    
    /// Записать событие в лог
    fn log_event(&mut self, event_type: EventType, message: String) {
        if self.event_log.len() >= 100 {
            self.event_log.pop_front();
        }
        self.event_log.push_back(OrganismEvent {
            time: self.lifetime,
            event_type,
            message,
        });
    }
    
    /// Подготовить данные для GPU рендеринга
    pub fn prepare_gpu_instances(&self) -> Vec<VoxelGpuInstance> {
        let pulse = (self.pulse_phase).sin() * 0.1 + 1.0;
        
        self.voxels.iter()
            .filter(|v| v.flags.contains(VoxelFlags::ALIVE))
            .map(|v| {
                let mut instance = VoxelGpuInstance::from_voxel(v, 0.3 * pulse);
                
                // Модифицировать цвет в зависимости от режима
                match self.mode {
                    OrganismMode::Ignite => {
                        if v.flags.contains(VoxelFlags::IGNITED) {
                            instance.color[1] = (instance.color[1] + 0.5).min(1.0); // Зеленее
                        }
                    }
                    OrganismMode::Trauma => {
                        if v.flags.contains(VoxelFlags::TRAUMATIZED) {
                            instance.color[0] = (instance.color[0] + 0.5).min(1.0); // Краснее
                        }
                    }
                    _ => {}
                }
                
                instance
            })
            .collect()
    }
    
    /// Получить статистику для UI
    pub fn get_stats(&self) -> OrganismStats {
        let total_voxels = self.voxels.len();
        let alive_voxels = self.voxels.iter()
            .filter(|v| v.flags.contains(VoxelFlags::ALIVE))
            .count();
        
        OrganismStats {
            total_voxels,
            alive_voxels,
            health_percent: self.health * 100.0,
            memory_mb: CompressedVoxelStorage::estimate_memory_mb(total_voxels),
            memory_saved_percent: self.voxels.stats.memory_saved_percent(),
            active_creatures: self.creatures.len(),
            clusters: self.clusterizer.clusters.len(),
            integrations: self.trauma_system.stats.integrations,
            rejections: self.trauma_system.stats.rejections,
            mode: self.mode,
        }
    }
}

impl Default for MetaOrganism {
    fn default() -> Self {
        Self::new()
    }
}

/// Статистика для UI
#[derive(Clone, Debug)]
pub struct OrganismStats {
    pub total_voxels: usize,
    pub alive_voxels: usize,
    pub health_percent: f32,
    pub memory_mb: f32,
    pub memory_saved_percent: f32,
    pub active_creatures: usize,
    pub clusters: usize,
    pub integrations: u32,
    pub rejections: u32,
    pub mode: OrganismMode,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_organism_creation() {
        let organism = MetaOrganism::new();
        assert!(!organism.voxels.is_empty());
        println!("Created organism with {} voxels", organism.voxels.len());
    }
    
    #[test]
    fn test_file_drop() {
        let mut organism = MetaOrganism::new();
        let fake_file = vec![0u8; 1000];
        
        organism.handle_file_drop("test.txt", &fake_file);
        
        assert_eq!(organism.creatures.len(), 1);
        println!("Created creature from file");
    }
}
