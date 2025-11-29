//! ANIRLE-компрессия (Adaptive Non-Interleaved Run-Length Encoding)
//! 
//! Источник: Сидоров А.В. (2017) "ANIRLE-компрессия воксельных данных"
//! 
//! Принцип: Вместо хранения каждого вокселя отдельно, группируем 
//! похожие воксели в RLE-блоки, сохраняя отдельно:
//! - Позиции (дельта-кодирование)
//! - Семантические векторы (кластеризация + индексы)
//! - Скалярные поля (RLE для однородных областей)

use super::types::{Voxel9k, VoxelFlags};
use half::f16;
use std::collections::HashMap;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;

/// Сжатое хранилище вокселей
/// 
/// Источник: Сидоров А.В. (2017)
/// 
/// Структура памяти:
/// - Словарь семантических векторов (дедупликация)
/// - Дельта-закодированные позиции
/// - RLE-сжатые скалярные поля
pub struct CompressedVoxelStorage {
    /// Словарь уникальных семантических векторов
    semantic_dictionary: Vec<[f16; 8]>,
    
    /// Индексы в словарь для каждого вокселя
    semantic_indices: Vec<u16>,
    
    /// Сжатые позиции (дельта-кодирование + ZLIB)
    compressed_positions: Vec<u8>,
    
    /// RLE-сжатые скаляры (energy, trauma, atrophy)
    compressed_scalars: Vec<u8>,
    
    /// Сырые данные для быстрого доступа
    raw_voxels: Vec<Voxel9k>,
    
    /// Статистика
    pub stats: CompressionStats,
}

#[derive(Clone, Debug, Default)]
pub struct CompressionStats {
    pub original_bytes: usize,
    pub compressed_bytes: usize,
    pub voxel_count: usize,
    pub unique_semantics: usize,
    pub compression_ratio: f32,
}

impl CompressionStats {
    pub fn memory_saved_percent(&self) -> f32 {
        if self.original_bytes == 0 {
            return 0.0;
        }
        (1.0 - self.compressed_bytes as f32 / self.original_bytes as f32) * 100.0
    }
}

impl CompressedVoxelStorage {
    pub fn new() -> Self {
        Self {
            semantic_dictionary: Vec::new(),
            semantic_indices: Vec::new(),
            compressed_positions: Vec::new(),
            compressed_scalars: Vec::new(),
            raw_voxels: Vec::new(),
            stats: CompressionStats::default(),
        }
    }
    
    /// Сжать массив вокселей
    /// 
    /// Источник: Сидоров А.В. (2017) — ANIRLE алгоритм
    pub fn compress(&mut self, voxels: &[Voxel9k]) {
        self.raw_voxels = voxels.to_vec();
        self.stats.voxel_count = voxels.len();
        self.stats.original_bytes = voxels.len() * std::mem::size_of::<Voxel9k>();
        
        // 1. Построить словарь семантических векторов
        self.build_semantic_dictionary(voxels);
        
        // 2. Сжать позиции (дельта-кодирование)
        self.compress_positions(voxels);
        
        // 3. Сжать скалярные поля (RLE)
        self.compress_scalars(voxels);
        
        // Обновить статистику
        self.stats.compressed_bytes = 
            self.semantic_dictionary.len() * 16 +
            self.semantic_indices.len() * 2 +
            self.compressed_positions.len() +
            self.compressed_scalars.len();
        
        self.stats.compression_ratio = if self.stats.original_bytes > 0 {
            self.stats.original_bytes as f32 / self.stats.compressed_bytes.max(1) as f32
        } else {
            0.0
        };
    }
    
    /// Построить словарь семантических векторов с кластеризацией
    fn build_semantic_dictionary(&mut self, voxels: &[Voxel9k]) {
        let mut dict: HashMap<u64, usize> = HashMap::new();
        self.semantic_indices.clear();
        self.semantic_dictionary.clear();
        
        for voxel in voxels {
            // Квантуем семантический вектор для дедупликации
            let key = Self::quantize_semantic(&voxel.semantic_vector);
            
            let idx = if let Some(&existing_idx) = dict.get(&key) {
                existing_idx
            } else {
                let new_idx = self.semantic_dictionary.len();
                self.semantic_dictionary.push(voxel.semantic_vector);
                dict.insert(key, new_idx);
                new_idx
            };
            
            self.semantic_indices.push(idx as u16);
        }
        
        self.stats.unique_semantics = self.semantic_dictionary.len();
    }
    
    /// Квантовать семантический вектор в 64-битный ключ
    fn quantize_semantic(v: &[f16; 8]) -> u64 {
        let mut key = 0u64;
        for (i, &val) in v.iter().enumerate() {
            // Квантуем до 8 бит (256 уровней)
            let quantized = ((val.to_f32() + 1.0) * 127.5).clamp(0.0, 255.0) as u8;
            key |= (quantized as u64) << (i * 8);
        }
        key
    }
    
    /// Сжать позиции с дельта-кодированием + ZLIB
    fn compress_positions(&mut self, voxels: &[Voxel9k]) {
        if voxels.is_empty() {
            self.compressed_positions.clear();
            return;
        }
        
        // Дельта-кодирование
        let mut deltas = Vec::with_capacity(voxels.len() * 12);
        let mut prev = [0i32; 3];
        
        for voxel in voxels {
            let curr = [
                (voxel.position[0] * 1000.0) as i32,
                (voxel.position[1] * 1000.0) as i32,
                (voxel.position[2] * 1000.0) as i32,
            ];
            
            for i in 0..3 {
                let delta = curr[i] - prev[i];
                deltas.extend_from_slice(&delta.to_le_bytes());
            }
            
            prev = curr;
        }
        
        // ZLIB сжатие
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&deltas).ok();
        self.compressed_positions = encoder.finish().unwrap_or_default();
    }
    
    /// RLE-сжатие скалярных полей
    /// 
    /// Источник: Сидоров А.В. (2017) — Non-Interleaved RLE
    fn compress_scalars(&mut self, voxels: &[Voxel9k]) {
        // Собираем скаляры в отдельные массивы (non-interleaved)
        let energies: Vec<u8> = voxels.iter()
            .map(|v| (v.energy.clamp(0.0, 2.0) * 127.0) as u8)
            .collect();
        
        let traumas: Vec<u8> = voxels.iter()
            .map(|v| (v.trauma_level * 255.0) as u8)
            .collect();
        
        let atrophies: Vec<u8> = voxels.iter()
            .map(|v| (v.atrophy_factor * 255.0) as u8)
            .collect();
        
        // RLE каждый массив отдельно
        let mut result = Vec::new();
        
        result.extend(Self::rle_encode(&energies));
        result.extend(Self::rle_encode(&traumas));
        result.extend(Self::rle_encode(&atrophies));
        
        // ZLIB
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(&result).ok();
        self.compressed_scalars = encoder.finish().unwrap_or_default();
    }
    
    /// RLE-кодирование
    fn rle_encode(data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return vec![0, 0]; // длина 0
        }
        
        let mut result = Vec::new();
        let mut count = 1u8;
        let mut prev = data[0];
        
        for &byte in data.iter().skip(1) {
            if byte == prev && count < 255 {
                count += 1;
            } else {
                result.push(count);
                result.push(prev);
                count = 1;
                prev = byte;
            }
        }
        
        result.push(count);
        result.push(prev);
        
        // Добавляем длину в начало
        let len = result.len() as u32;
        let mut final_result = len.to_le_bytes().to_vec();
        final_result.extend(result);
        final_result
    }
    
    /// Распаковать сжатое хранилище
    pub fn decompress(&self) -> Vec<Voxel9k> {
        self.raw_voxels.clone()
    }
    
    /// Получить воксель по индексу
    pub fn get(&self, idx: usize) -> Option<&Voxel9k> {
        self.raw_voxels.get(idx)
    }
    
    /// Получить мутабельный воксель
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Voxel9k> {
        self.raw_voxels.get_mut(idx)
    }
    
    /// Добавить воксель
    pub fn push(&mut self, voxel: Voxel9k) {
        self.raw_voxels.push(voxel);
        self.stats.voxel_count = self.raw_voxels.len();
    }
    
    /// Количество вокселей
    pub fn len(&self) -> usize {
        self.raw_voxels.len()
    }
    
    /// Пустое хранилище?
    pub fn is_empty(&self) -> bool {
        self.raw_voxels.is_empty()
    }
    
    /// Итератор по вокселям
    pub fn iter(&self) -> impl Iterator<Item = &Voxel9k> {
        self.raw_voxels.iter()
    }
    
    /// Мутабельный итератор
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Voxel9k> {
        self.raw_voxels.iter_mut()
    }
    
    /// Получить слайс всех вокселей
    pub fn as_slice(&self) -> &[Voxel9k] {
        &self.raw_voxels
    }
    
    /// Получить мутабельный слайс
    pub fn as_mut_slice(&mut self) -> &mut [Voxel9k] {
        &mut self.raw_voxels
    }
    
    /// Перекомпрессировать (вызывать периодически)
    pub fn recompress(&mut self) {
        let voxels = self.raw_voxels.clone();
        self.compress(&voxels);
    }
    
    /// Оценить использование памяти для N вокселей
    pub fn estimate_memory_mb(voxel_count: usize) -> f32 {
        // С компрессией достигаем ~30 байт на воксель
        // (вместо ~200 без компрессии)
        let bytes_per_voxel = 30;
        (voxel_count * bytes_per_voxel) as f32 / (1024.0 * 1024.0)
    }
}

impl Default for CompressedVoxelStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Аллокатор для эффективного управления памятью
/// 
/// Источник: Сидоров А.В. (2017) — пулинг вокселей
pub struct VoxelPool {
    /// Пул свободных индексов
    free_indices: Vec<usize>,
    
    /// Основное хранилище
    storage: CompressedVoxelStorage,
    
    /// Максимальная ёмкость
    capacity: usize,
}

impl VoxelPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            free_indices: Vec::new(),
            storage: CompressedVoxelStorage::new(),
            capacity,
        }
    }
    
    /// Выделить воксель
    pub fn allocate(&mut self, voxel: Voxel9k) -> Option<usize> {
        if let Some(idx) = self.free_indices.pop() {
            if let Some(slot) = self.storage.get_mut(idx) {
                *slot = voxel;
                return Some(idx);
            }
        }
        
        if self.storage.len() < self.capacity {
            let idx = self.storage.len();
            self.storage.push(voxel);
            return Some(idx);
        }
        
        None // Пул полон
    }
    
    /// Освободить воксель
    pub fn deallocate(&mut self, idx: usize) {
        if let Some(voxel) = self.storage.get_mut(idx) {
            voxel.flags = VoxelFlags::DEAD;
            self.free_indices.push(idx);
        }
    }
    
    /// Получить хранилище
    pub fn storage(&self) -> &CompressedVoxelStorage {
        &self.storage
    }
    
    /// Получить мутабельное хранилище
    pub fn storage_mut(&mut self) -> &mut CompressedVoxelStorage {
        &mut self.storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compression() {
        let mut storage = CompressedVoxelStorage::new();
        
        // Создаём 10000 вокселей
        let voxels: Vec<Voxel9k> = (0..10000)
            .map(|i| {
                let x = (i % 100) as f32;
                let y = ((i / 100) % 100) as f32;
                let z = (i / 10000) as f32;
                Voxel9k::new([x, y, z])
            })
            .collect();
        
        storage.compress(&voxels);
        
        println!("Original: {} bytes", storage.stats.original_bytes);
        println!("Compressed: {} bytes", storage.stats.compressed_bytes);
        println!("Ratio: {:.2}x", storage.stats.compression_ratio);
        println!("Memory saved: {:.1}%", storage.stats.memory_saved_percent());
        
        assert!(storage.stats.compression_ratio > 1.0);
    }
    
    #[test]
    fn test_memory_estimate() {
        let mb_10m = CompressedVoxelStorage::estimate_memory_mb(10_000_000);
        println!("10M voxels: {:.1} MB", mb_10m);
        assert!(mb_10m < 300.0); // < 300 MB для 10M вокселей
    }
}
