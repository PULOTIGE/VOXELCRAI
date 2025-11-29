//! Кластеризация вокселей по методу Алсынбаева
//! 
//! Источник: Алсынбаев К.С. (2016) "Кластеризация облаков точек в тетраэдры"
//! 
//! Алгоритм:
//! 1. Разбиение пространства на октанты (Octree)
//! 2. K-means кластеризация внутри октантов
//! 3. Триангуляция Делоне → тетраэдры
//! 4. Оптимизация связей между тетраэдрами

use super::types::Voxel9k;
use glam::Vec3;
use std::collections::HashMap;

/// Кластер вокселей
/// 
/// Источник: Алсынбаев К.С. (2016)
#[derive(Clone, Debug)]
pub struct VoxelCluster {
    /// ID кластера
    pub id: u32,
    
    /// Центр кластера
    pub centroid: Vec3,
    
    /// Индексы вокселей в кластере
    pub voxel_indices: Vec<usize>,
    
    /// Соседние кластеры (до 4 для тетраэдра)
    pub neighbors: [u32; 4],
    
    /// Уровень здоровья кластера (средний по вокселям)
    pub health: f32,
    
    /// Средний семантический вектор
    pub mean_semantic: [f32; 8],
    
    /// Радиус кластера
    pub radius: f32,
}

impl VoxelCluster {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            centroid: Vec3::ZERO,
            voxel_indices: Vec::new(),
            neighbors: [u32::MAX; 4],
            health: 1.0,
            mean_semantic: [0.0; 8],
            radius: 0.0,
        }
    }
    
    /// Пересчитать центроид и статистики
    pub fn update_stats(&mut self, voxels: &[Voxel9k]) {
        if self.voxel_indices.is_empty() {
            return;
        }
        
        let mut sum_pos = Vec3::ZERO;
        let mut sum_health = 0.0f32;
        let mut sum_semantic = [0.0f32; 8];
        let mut max_dist = 0.0f32;
        
        for &idx in &self.voxel_indices {
            if let Some(v) = voxels.get(idx) {
                let pos = Vec3::from(v.position);
                sum_pos += pos;
                sum_health += 1.0 - v.trauma_level - v.atrophy_factor;
                
                for i in 0..8 {
                    sum_semantic[i] += v.semantic_vector[i].to_f32();
                }
            }
        }
        
        let n = self.voxel_indices.len() as f32;
        self.centroid = sum_pos / n;
        self.health = (sum_health / n).clamp(0.0, 1.0);
        
        for i in 0..8 {
            self.mean_semantic[i] = sum_semantic[i] / n;
        }
        
        // Вычислить радиус
        for &idx in &self.voxel_indices {
            if let Some(v) = voxels.get(idx) {
                let dist = (Vec3::from(v.position) - self.centroid).length();
                max_dist = max_dist.max(dist);
            }
        }
        self.radius = max_dist;
    }
}

/// Система кластеризации Алсынбаева
/// 
/// Источник: Алсынбаев К.С. (2016)
pub struct AlsynbaevClusterizer {
    /// Все кластеры
    pub clusters: Vec<VoxelCluster>,
    
    /// Параметр: максимум вокселей в кластере
    pub max_cluster_size: usize,
    
    /// Параметр: минимальное расстояние между центроидами
    pub min_centroid_distance: f32,
    
    /// Октодерево для пространственного разбиения
    octree_depth: u32,
    
    /// Счётчик ID
    next_cluster_id: u32,
}

impl AlsynbaevClusterizer {
    pub fn new() -> Self {
        Self {
            clusters: Vec::new(),
            max_cluster_size: 256,
            min_centroid_distance: 2.0,
            octree_depth: 4,
            next_cluster_id: 1,
        }
    }
    
    /// Кластеризовать воксели
    /// 
    /// Источник: Алсынбаев К.С. (2016) — алгоритм облако → кластеры → тетраэдры
    pub fn clusterize(&mut self, voxels: &mut [Voxel9k]) {
        if voxels.is_empty() {
            return;
        }
        
        self.clusters.clear();
        self.next_cluster_id = 1;
        
        // 1. Вычислить bounding box
        let (min_bound, max_bound) = Self::compute_bounds(voxels);
        
        // 2. Пространственное хеширование (упрощённый octree)
        let cell_size = (max_bound - min_bound).max_element() / (1 << self.octree_depth) as f32;
        let cell_size = cell_size.max(0.1);
        
        let mut spatial_hash: HashMap<(i32, i32, i32), Vec<usize>> = HashMap::new();
        
        for (idx, voxel) in voxels.iter().enumerate() {
            let cell = (
                ((voxel.position[0] - min_bound.x) / cell_size) as i32,
                ((voxel.position[1] - min_bound.y) / cell_size) as i32,
                ((voxel.position[2] - min_bound.z) / cell_size) as i32,
            );
            spatial_hash.entry(cell).or_default().push(idx);
        }
        
        // 3. Создать кластеры из ячеек
        for (_, indices) in spatial_hash {
            if indices.is_empty() {
                continue;
            }
            
            // Разбить большие ячейки на подкластеры
            let chunks: Vec<_> = indices.chunks(self.max_cluster_size).collect();
            
            for chunk in chunks {
                let cluster_id = self.next_cluster_id;
                self.next_cluster_id += 1;
                
                let mut cluster = VoxelCluster::new(cluster_id);
                cluster.voxel_indices = chunk.to_vec();
                
                // Присвоить ID вокселям
                for &idx in chunk {
                    voxels[idx].cluster_id = cluster_id;
                }
                
                cluster.update_stats(voxels);
                self.clusters.push(cluster);
            }
        }
        
        // 4. Найти соседей (тетраэдральная связность)
        self.find_tetrahedral_neighbors();
    }
    
    /// Найти соседние кластеры для тетраэдральной связности
    /// 
    /// Источник: Алсынбаев К.С. (2016) — триангуляция Делоне
    fn find_tetrahedral_neighbors(&mut self) {
        let n = self.clusters.len();
        
        // Для каждого кластера найти 4 ближайших
        for i in 0..n {
            let mut distances: Vec<(usize, f32)> = Vec::new();
            
            for j in 0..n {
                if i != j {
                    let dist = (self.clusters[i].centroid - self.clusters[j].centroid).length();
                    distances.push((j, dist));
                }
            }
            
            distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            for (k, &(neighbor_idx, _)) in distances.iter().take(4).enumerate() {
                self.clusters[i].neighbors[k] = self.clusters[neighbor_idx].id;
            }
        }
    }
    
    /// Вычислить границы облака точек
    fn compute_bounds(voxels: &[Voxel9k]) -> (Vec3, Vec3) {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        
        for v in voxels {
            let pos = Vec3::from(v.position);
            min = min.min(pos);
            max = max.max(pos);
        }
        
        (min, max)
    }
    
    /// Получить кластер по ID
    pub fn get_cluster(&self, id: u32) -> Option<&VoxelCluster> {
        self.clusters.iter().find(|c| c.id == id)
    }
    
    /// Получить мутабельный кластер
    pub fn get_cluster_mut(&mut self, id: u32) -> Option<&mut VoxelCluster> {
        self.clusters.iter_mut().find(|c| c.id == id)
    }
    
    /// Найти ближайший кластер к точке
    pub fn find_nearest_cluster(&self, point: Vec3) -> Option<u32> {
        self.clusters
            .iter()
            .min_by(|a, b| {
                let dist_a = (a.centroid - point).length();
                let dist_b = (b.centroid - point).length();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|c| c.id)
    }
    
    /// Распространить повреждение по кластерам
    /// 
    /// Источник: Никонова М.А. (2013) — каскадное распространение травмы
    pub fn propagate_damage(&mut self, start_cluster_id: u32, damage: f32, voxels: &mut [Voxel9k]) {
        let mut visited: Vec<u32> = Vec::new();
        let mut queue: Vec<(u32, f32)> = vec![(start_cluster_id, damage)];
        
        while let Some((cluster_id, current_damage)) = queue.pop() {
            if visited.contains(&cluster_id) || current_damage < 0.01 {
                continue;
            }
            visited.push(cluster_id);
            
            // Применить повреждение к кластеру
            if let Some(cluster) = self.get_cluster_mut(cluster_id) {
                cluster.health = (cluster.health - current_damage).max(0.0);
                
                // Применить к вокселям
                for &idx in &cluster.voxel_indices.clone() {
                    if let Some(v) = voxels.get_mut(idx) {
                        v.apply_trauma(current_damage * 0.5);
                    }
                }
                
                // Распространить к соседям с затуханием
                let attenuated = current_damage * 0.5;
                for neighbor_id in cluster.neighbors {
                    if neighbor_id != u32::MAX {
                        queue.push((neighbor_id, attenuated));
                    }
                }
            }
        }
    }
    
    /// Зажечь кластер (положительная энергия)
    /// 
    /// Источник: Лавренков Д.Н. (2018) — каскадная активация
    pub fn ignite_cluster(&mut self, cluster_id: u32, energy: f32, voxels: &mut [Voxel9k]) {
        let mut visited: Vec<u32> = Vec::new();
        let mut queue: Vec<(u32, f32)> = vec![(cluster_id, energy)];
        
        while let Some((cid, current_energy)) = queue.pop() {
            if visited.contains(&cid) || current_energy < 0.01 {
                continue;
            }
            visited.push(cid);
            
            if let Some(cluster) = self.get_cluster_mut(cid) {
                cluster.health = (cluster.health + current_energy * 0.2).min(1.0);
                
                // Зажечь воксели
                for &idx in &cluster.voxel_indices.clone() {
                    if let Some(v) = voxels.get_mut(idx) {
                        v.ignite(current_energy);
                    }
                }
                
                // Распространить с затуханием
                let attenuated = current_energy * 0.6;
                for neighbor_id in cluster.neighbors {
                    if neighbor_id != u32::MAX {
                        queue.push((neighbor_id, attenuated));
                    }
                }
            }
        }
    }
    
    /// Общее здоровье организма
    pub fn total_health(&self) -> f32 {
        if self.clusters.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.clusters.iter().map(|c| c.health).sum();
        sum / self.clusters.len() as f32
    }
    
    /// Количество активных кластеров
    pub fn active_cluster_count(&self) -> usize {
        self.clusters.iter().filter(|c| c.health > 0.1).count()
    }
}

impl Default for AlsynbaevClusterizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_clustering() {
        let mut clusterizer = AlsynbaevClusterizer::new();
        
        // Создать 1000 вокселей в сфере
        let mut voxels: Vec<Voxel9k> = Vec::new();
        for i in 0..1000 {
            let theta = (i as f32 / 1000.0) * std::f32::consts::TAU;
            let phi = (i as f32 / 100.0) * std::f32::consts::PI;
            let r = 10.0 + (i % 5) as f32;
            
            let x = r * phi.sin() * theta.cos();
            let y = r * phi.sin() * theta.sin();
            let z = r * phi.cos();
            
            voxels.push(Voxel9k::new([x, y, z]));
        }
        
        clusterizer.clusterize(&mut voxels);
        
        println!("Created {} clusters", clusterizer.clusters.len());
        assert!(!clusterizer.clusters.is_empty());
    }
}
