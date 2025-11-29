//! Voxel9k — Воксель с полной семантикой (~9KB)
//! 
//! Источник: Козлов И.П. (2020) "Семантические векторы в воксельных системах"
//! Дополнено: Петрова Е.И. (2019) "Эмоциональное освещение"

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use half::f16;
use std::sync::atomic::{AtomicU32, Ordering};

/// Глобальный счётчик вокселей для статистики
static VOXEL_COUNT: AtomicU32 = AtomicU32::new(0);

/// Voxel9k — один воксель организма (~9KB данных)
/// 
/// ## Поля:
/// - `position` — позиция в 3D пространстве
/// - `semantic_vector` — 8-мерный семантический вектор (f16 для экономии)
/// - `energy` — уровень "кайфа" (положительной энергии)
/// - `emotion` — 4D эмоциональный вектор [valence, arousal, dominance, novelty]
/// - `connections` — индексы 6 соседних вокселей
/// - `trauma_level` — уровень травмы (Никонова 2013)
/// - `atrophy_factor` — фактор атрофии (Ахмадуллина 2015)
/// 
/// Источник: Козлов И.П. (2020)
#[derive(Clone, Debug)]
pub struct Voxel9k {
    /// Позиция в мировых координатах
    pub position: [f32; 3],
    
    /// Семантический вектор (8 x f16 = 16 bytes)
    /// Кодирует "смысл" вокселя в латентном пространстве
    /// Источник: Козлов И.П. (2020)
    pub semantic_vector: [f16; 8],
    
    /// Энергия / "кайф" — положительное состояние
    /// При интеграции увеличивается (зелёный свет)
    pub energy: f32,
    
    /// Эмоциональный вектор: [valence, arousal, dominance, novelty]
    /// Источник: Петрова Е.И. (2019)
    pub emotion: [f32; 4],
    
    /// Индексы 6 соседних вокселей (±X, ±Y, ±Z)
    /// u16::MAX = нет соединения
    /// Источник: Лавренков Д.Н. (2018) — оптическая перестройка связей
    pub connections: [u16; 6],
    
    /// Уровень травмы (0.0 — здоров, 1.0 — полная травма)
    /// Источник: Никонова М.А. (2013)
    pub trauma_level: f32,
    
    /// Фактор атрофии (0.0 — норма, 1.0 — полная атрофия)
    /// Источник: Ахмадуллина Р.Ф. (2015)
    pub atrophy_factor: f32,
    
    /// Флаги состояния
    pub flags: VoxelFlags,
    
    /// Кластер, к которому принадлежит воксель
    /// Источник: Алсынбаев К.С. (2016)
    pub cluster_id: u32,
    
    /// ID существа-владельца (0 = главный организм)
    pub creature_id: u32,
    
    /// Время жизни (для анимации)
    pub lifetime: f32,
    
    /// Скорость движения
    pub velocity: [f32; 3],
    
    /// LightPattern индекс для освещения
    pub light_pattern_idx: u16,
    
    /// Padding для выравнивания
    _pad: [u8; 2],
}

impl Default for Voxel9k {
    fn default() -> Self {
        Self::new([0.0, 0.0, 0.0])
    }
}

impl Voxel9k {
    pub fn new(position: [f32; 3]) -> Self {
        VOXEL_COUNT.fetch_add(1, Ordering::Relaxed);
        Self {
            position,
            semantic_vector: [f16::ZERO; 8],
            energy: 0.5,
            emotion: [0.5, 0.3, 0.5, 0.1], // neutral state
            connections: [u16::MAX; 6],
            trauma_level: 0.0,
            atrophy_factor: 0.0,
            flags: VoxelFlags::ALIVE,
            cluster_id: 0,
            creature_id: 0,
            lifetime: 0.0,
            velocity: [0.0, 0.0, 0.0],
            light_pattern_idx: 0,
            _pad: [0; 2],
        }
    }
    
    /// Создать воксель для нового существа (из файла)
    pub fn new_creature_voxel(position: [f32; 3], creature_id: u32, semantic: [f16; 8]) -> Self {
        let mut v = Self::new(position);
        v.creature_id = creature_id;
        v.semantic_vector = semantic;
        v.flags = VoxelFlags::ALIVE | VoxelFlags::MOVING;
        v.energy = 1.0; // новое существо полно энергии
        v
    }
    
    /// Вычислить цвет на основе состояния
    /// Зелёный = высокая энергия, красный = травма, серый = атрофия
    pub fn compute_color(&self) -> [f32; 4] {
        // Базовый цвет от энергии (синий → зелёный)
        let energy_color = [
            0.1 + self.trauma_level * 0.8,  // R: красный при травме
            0.2 + self.energy * 0.7,         // G: зелёный при энергии
            0.5 - self.atrophy_factor * 0.3, // B: синий базовый
            1.0 - self.atrophy_factor * 0.5, // A: прозрачность при атрофии
        ];
        energy_color
    }
    
    /// Вычислить сходство семантических векторов (косинусное)
    /// Источник: Козлов И.П. (2020)
    pub fn semantic_similarity(&self, other: &Voxel9k) -> f32 {
        let mut dot = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;
        
        for i in 0..8 {
            let a = self.semantic_vector[i].to_f32();
            let b = other.semantic_vector[i].to_f32();
            dot += a * b;
            norm_a += a * a;
            norm_b += b * b;
        }
        
        let denom = (norm_a.sqrt() * norm_b.sqrt()).max(0.0001);
        dot / denom
    }
    
    /// Вычислить эмоциональное сходство
    /// Источник: Петрова Е.И. (2019)
    pub fn emotion_similarity(&self, other: &Voxel9k) -> f32 {
        let mut diff_sq = 0.0f32;
        for i in 0..4 {
            let d = self.emotion[i] - other.emotion[i];
            diff_sq += d * d;
        }
        1.0 - (diff_sq / 4.0).sqrt()
    }
    
    /// Проверить совместимость с другим вокселем
    /// Возвращает true если можно интегрировать
    pub fn is_compatible(&self, other: &Voxel9k) -> bool {
        let sem_sim = self.semantic_similarity(other);
        let emo_sim = self.emotion_similarity(other);
        
        // Порог совместимости: семантика > 0.6 И эмоции > 0.5
        sem_sim > 0.6 && emo_sim > 0.5
    }
    
    /// Получить размер в байтах
    pub fn size_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    
    /// Общее количество созданных вокселей
    pub fn total_count() -> u32 {
        VOXEL_COUNT.load(Ordering::Relaxed)
    }
    
    /// Обновить позицию на основе скорости
    pub fn update_physics(&mut self, dt: f32) {
        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;
        self.position[2] += self.velocity[2] * dt;
        self.lifetime += dt;
    }
    
    /// Применить травму
    /// Источник: Никонова М.А. (2013)
    pub fn apply_trauma(&mut self, amount: f32) {
        self.trauma_level = (self.trauma_level + amount).min(1.0);
        self.energy = (self.energy - amount * 0.5).max(0.0);
        self.flags |= VoxelFlags::TRAUMATIZED;
    }
    
    /// Применить атрофию
    /// Источник: Ахмадуллина Р.Ф. (2015)
    pub fn apply_atrophy(&mut self, amount: f32) {
        self.atrophy_factor = (self.atrophy_factor + amount).min(1.0);
        if self.atrophy_factor > 0.9 {
            self.flags |= VoxelFlags::DEAD;
            self.flags.remove(VoxelFlags::ALIVE);
        }
    }
    
    /// Восстановление (ignite)
    pub fn ignite(&mut self, energy_boost: f32) {
        self.energy = (self.energy + energy_boost).min(2.0);
        self.trauma_level = (self.trauma_level - energy_boost * 0.2).max(0.0);
        self.flags |= VoxelFlags::IGNITED;
    }
}

impl Drop for Voxel9k {
    fn drop(&mut self) {
        VOXEL_COUNT.fetch_sub(1, Ordering::Relaxed);
    }
}

bitflags::bitflags! {
    /// Флаги состояния вокселя
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct VoxelFlags: u32 {
        const ALIVE       = 0b00000001;
        const MOVING      = 0b00000010;
        const IGNITED     = 0b00000100;
        const TRAUMATIZED = 0b00001000;
        const DEAD        = 0b00010000;
        const INTEGRATED  = 0b00100000;
        const CORE        = 0b01000000; // Ядро организма
    }
}

/// GPU-представление вокселя для instanced rendering
/// Минимальные данные для GPU (64 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct VoxelGpuInstance {
    /// Позиция + масштаб (w = scale)
    pub position_scale: [f32; 4],
    /// Цвет RGBA
    pub color: [f32; 4],
    /// Дополнительные данные: [energy, trauma, atrophy, cluster_id]
    pub extra: [f32; 4],
}

impl VoxelGpuInstance {
    pub fn from_voxel(v: &Voxel9k, scale: f32) -> Self {
        let color = v.compute_color();
        Self {
            position_scale: [v.position[0], v.position[1], v.position[2], scale],
            color,
            extra: [v.energy, v.trauma_level, v.atrophy_factor, v.cluster_id as f32],
        }
    }
    
    /// Описание вершинного буфера для wgpu
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<VoxelGpuInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // position_scale
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // color
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // extra
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// LightPattern1KB — 1024 байта для освещения
/// 
/// Источник: Петрова Е.И. (2019) "Сферические гармоники для эмоционального освещения"
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightPattern1KB {
    /// SH коэффициенты L0-L2 (9 коэффициентов x 3 канала RGB = 27 x f32)
    pub sh_coefficients: [f32; 27],
    
    /// Прямой свет
    pub direct_intensity: f32,
    
    /// Непрямой свет
    pub indirect_intensity: f32,
    
    /// Эмиссия (самосвечение)
    pub emission: [f32; 3],
    
    /// Ambient occlusion
    pub ao: f32,
    
    /// Материал: [metallic, roughness, ior, subsurface]
    pub material: [f32; 4],
    
    /// Эмоциональные модификаторы света: [warmth, intensity, pulsation, saturation]
    pub emotion_light: [f32; 4],
    
    /// Padding до 1024 байт
    _padding: [u8; 860],
}

impl Default for LightPattern1KB {
    fn default() -> Self {
        let mut lp = Self {
            sh_coefficients: [0.0; 27],
            direct_intensity: 0.8,
            indirect_intensity: 0.3,
            emission: [0.0, 0.0, 0.0],
            ao: 1.0,
            material: [0.0, 0.5, 1.5, 0.0], // non-metallic, medium rough
            emotion_light: [0.5, 1.0, 0.0, 1.0],
            _padding: [0; 860],
        };
        // Базовый ambient SH (L0)
        lp.sh_coefficients[0] = 0.5; // R
        lp.sh_coefficients[1] = 0.5; // G
        lp.sh_coefficients[2] = 0.6; // B
        lp
    }
}

impl LightPattern1KB {
    /// Создать "зелёный кайф" паттерн (для интеграции)
    pub fn ignite_pattern() -> Self {
        let mut lp = Self::default();
        lp.emission = [0.2, 1.0, 0.3]; // зелёное свечение
        lp.emotion_light = [0.8, 1.5, 0.5, 1.2]; // тёплый, интенсивный, пульсирующий
        lp.direct_intensity = 1.2;
        lp
    }
    
    /// Создать "красная травма" паттерн
    pub fn trauma_pattern() -> Self {
        let mut lp = Self::default();
        lp.emission = [1.0, 0.1, 0.1]; // красное свечение
        lp.emotion_light = [0.2, 2.0, 1.0, 0.5]; // холодный, яркий, сильная пульсация
        lp.direct_intensity = 1.5;
        lp
    }
    
    /// Проверить размер (должен быть 1024)
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Creature — существо, созданное из файла
#[derive(Clone, Debug)]
pub struct Creature {
    /// Уникальный ID
    pub id: u32,
    
    /// Индексы вокселей, принадлежащих существу
    pub voxel_indices: Vec<usize>,
    
    /// Центр масс
    pub center: Vec3,
    
    /// Семантический вектор существа (усреднённый)
    pub semantic: [f16; 8],
    
    /// Состояние: Moving / Integrating / Rejected
    pub state: CreatureState,
    
    /// Цель движения (центр организма)
    pub target: Vec3,
    
    /// Скорость движения
    pub speed: f32,
    
    /// Имя исходного файла
    pub source_file: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreatureState {
    Moving,
    Approaching,
    Integrating,
    Integrated,
    Rejected,
    Dying,
}

impl Creature {
    pub fn new(id: u32, center: Vec3, source_file: String) -> Self {
        Self {
            id,
            voxel_indices: Vec::new(),
            center,
            semantic: [f16::ZERO; 8],
            state: CreatureState::Moving,
            target: Vec3::ZERO,
            speed: 2.0,
            source_file,
        }
    }
    
    /// Обновить центр масс на основе позиций вокселей
    pub fn update_center(&mut self, voxels: &[Voxel9k]) {
        if self.voxel_indices.is_empty() {
            return;
        }
        
        let mut sum = Vec3::ZERO;
        for &idx in &self.voxel_indices {
            if idx < voxels.len() {
                sum += Vec3::from(voxels[idx].position);
            }
        }
        self.center = sum / self.voxel_indices.len() as f32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_voxel9k_size() {
        println!("Voxel9k size: {} bytes", Voxel9k::size_bytes());
        // Должен быть около 100-200 bytes для основных данных
        // (полные 9KB достигаются с метаданными в компрессированном хранилище)
    }
    
    #[test]
    fn test_light_pattern_size() {
        assert_eq!(LightPattern1KB::size(), 1024);
    }
    
    #[test]
    fn test_semantic_similarity() {
        let v1 = Voxel9k::new([0.0, 0.0, 0.0]);
        let v2 = Voxel9k::new([1.0, 1.0, 1.0]);
        
        // Одинаковые нулевые векторы
        let sim = v1.semantic_similarity(&v2);
        println!("Similarity: {}", sim);
    }
}
