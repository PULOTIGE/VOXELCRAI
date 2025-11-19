# Adaptive Entity Engine v1.0

Высокопроизводительный воксельный движок на Rust с поддержкой Vulkan через wgpu, ECS системой, эволюционными алгоритмами и защитой ArchGuard Enterprise.

## Особенности

- **Рендеринг**: wgpu (Vulkan) с fallback на HIP/ROCm для AMD Vega 20
- **ECS**: bevy_ecs для управления сущностями
- **Воксели**: 9-13 КБ на воксель с FP64/FP16/INT8/INT4, геномом и эволюцией
- **Освещение**: LightPattern структура ровно 1000 байт
- **Рендеринг**: Point cloud до 1.5 миллиардов точек, цвет по энергии (жёлтый = максимум)
- **Защита**: ArchGuard Enterprise (circuit-breaker, prometheus, empathy_ratio, детектор ритма 0.038 Гц)
- **UI**: egui + eframe
- **Bare-metal**: Поддержка AArch64 (boot.s в arm/)

## Структура проекта

```
adaptive-entity-engine/
├── src/
│   ├── main.rs              # Точка входа
│   ├── voxel.rs             # Система вокселей
│   ├── evolution.rs         # NextGen Evolution
│   ├── lighting.rs          # Система освещения
│   ├── renderer.rs          # wgpu рендерер
│   ├── archguard.rs         # ArchGuard Enterprise
│   ├── ui.rs                # egui интерфейс
│   ├── ecs.rs               # ECS утилиты
│   └── shaders/
│       └── point_cloud.wgsl # Шейдер для point cloud
├── arm/
│   ├── boot.s               # Bare-metal AArch64 boot
│   └── linker.ld            # Linker script
├── Cargo.toml
└── README.md
```

## Сборка

### Windows/Linux

```bash
cargo build --release
```

Результирующий исполняемый файл будет в `target/release/adaptive-entity-engine` (50-100 МБ).

### Bare-metal AArch64

```bash
cargo build --target aarch64-unknown-none --release
```

## Использование

Запуск:
```bash
./target/release/adaptive-entity-engine
```

### Травма-режим

Включите травма-режим в UI для увеличения интенсивности энергии и эмоций.

### Эволюция

Используйте кнопку "Evolve Population" для запуска эволюционного алгоритма на популяции вокселей.

## Технические детали

### Воксель (9-13 КБ)

- **FP64**: энергия, эмоции (валентность, возбуждение, доминирование)
- **FP16**: восприятие (визуальное, слуховое, тактильное, и др.)
- **INT8**: физика (скорость, ускорение, температура, давление)
- **INT4**: флаги состояния и материалов
- **Геном**: до 10 концептов (строки)
- **Echo + Resonance**: 16 байт echo + f16 resonance

### LightPattern (1000 байт)

- Direct/indirect: f16 (4 байта)
- Spherical Harmonics: i8[256] (256 байт)
- Материалы: u8[512] (512 байт)
- AO/reflection/refraction/emission: f16 + properties (228 байт)

### ArchGuard Enterprise

- Circuit-breaker для защиты от каскадных сбоев
- Prometheus метрики
- Empathy ratio (0.0 - 1.0)
- Детектор ритма 0.038 Гц (~26.3 секунды период)

## Лицензия

MIT OR Apache-2.0
