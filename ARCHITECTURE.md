# Architecture Documentation

## Adaptive Entity Engine v1.0

### Overview

Adaptive Entity Engine v1.0 — высокопроизводительный воксельный движок на Rust с поддержкой Vulkan через wgpu, ECS системой, эволюционными алгоритмами и защитой ArchGuard Enterprise.

### Core Components

#### 1. Voxel System (`src/voxel.rs`)

Воксель размером 9-13 КБ содержит:

- **FP64 (8 bytes each)**: энергия, эмоции (валентность, возбуждение, доминирование)
- **FP16 (2 bytes each)**: 10 типов восприятия (визуальное, слуховое, тактильное, и др.)
- **INT8 (1 byte each)**: физические свойства (скорость, ускорение, температура, давление, плотность, эластичность, трение, вязкость)
- **INT4 (packed)**: флаги состояния и материалов
- **Геном**: до 10 концептов (строки, переменный размер)
- **Echo + Resonance**: 16 байт echo + f16 resonance
- **Позиция**: i32[3] (12 байт)
- **Метаданные**: HashMap<String, String> (переменный размер)

#### 2. Evolution System (`src/evolution.rs`)

NextGen Evolution реализует:

- **Combine (Crossover)**: объединение геномов двух родителей
- **Mutate**: мутация генома (добавление/удаление/изменение концептов)
- **Fitness**: расчёт приспособленности на основе энергии, генома, резонанса, восприятия и эмоций

#### 3. Lighting System (`src/lighting.rs`)

LightPattern — структура ровно 1000 байт:

- Direct/indirect: f16 (4 байта)
- Spherical Harmonics: i8[256] (256 байт)
- Материалы: u8[512] (512 байт)
- AO/reflection/refraction/emission: f16 + properties (228 байт)

#### 4. Rendering System (`src/renderer.rs`)

- **Primary**: wgpu с Vulkan backend
- **Fallback**: HIP/ROCm для AMD Vega 20 (заглушка, требует интеграции)
- **Point Cloud**: поддержка до 1.5 миллиардов точек
- **Color Mapping**: цвет по энергии (жёлтый = максимум)

#### 5. ArchGuard Enterprise (`src/archguard.rs`)

Система защиты включает:

- **Circuit Breaker**: защита от каскадных сбоев
- **Prometheus Metrics**: счётчики, гистограммы, gauges
- **Empathy Ratio**: коэффициент эмпатии (0.0 - 1.0)
- **Rhythm Detector**: детектор ритма 0.038 Гц (~26.3 секунды период)

#### 6. ECS System (`src/ecs.rs`)

Использует bevy_ecs для управления сущностями:

- Компоненты: Voxel
- Системы: update_voxel_physics, update_voxel_energy

#### 7. UI System (`src/ui.rs`)

egui + eframe интерфейс:

- Статистика вокселей и точек
- Управление травма-режимом
- Контроль эволюции
- Управление освещением
- Визуализация point cloud (упрощённая 2D проекция)
- Debug информация

### Build System

#### Release Configuration

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

Целевой размер исполняемого файла: 50-100 МБ

#### Bare-metal AArch64

- Boot code: `arm/boot.s`
- Linker script: `arm/linker.ld`
- Entry point: 0x40000000

### Performance Considerations

1. **Voxel Storage**: Использование ECS для эффективного управления миллионами вокселей
2. **Point Cloud Rendering**: Оптимизированный буфер для 1-1.5 миллиардов точек
3. **Evolution**: Параллельная обработка популяции
4. **Lighting**: Предвычисленные паттерны освещения

### Trauma Mode

Травма-режим увеличивает интенсивность:

- Энергия: ×1.5
- Эмоциональное возбуждение: ×1.3

### Dependencies

- **wgpu**: Vulkan/OpenGL рендеринг
- **bevy_ecs**: ECS система
- **eframe/egui**: UI
- **prometheus**: Метрики
- **half**: FP16 поддержка
- **serde**: Сериализация

### Platform Support

- Windows (x86_64)
- Linux (x86_64, AArch64)
- Bare-metal AArch64

### Future Enhancements

- Полная интеграция HIP/ROCm для AMD Vega 20
- Расширенная визуализация point cloud
- Оптимизация памяти для больших популяций
- Распределённая эволюция
