# Changelog

All notable changes to VOXELCRAI are documented here.

## [1.1.0] - 2025-11-19

### Added
- Переименование проекта в **VOXELCRAI** и новый runtime на чистом winit/wgpu.
- Модуль `engine.rs` с EventLoop, обработкой ошибок surface и логированием телеметрии.
- `camera.rs` с полноценным контроллером (WASD/Space/Shift/ПКМ/колесо).
- `simulation.rs` и `consciousness.rs`: движок VOXELCRAI Core (ignite/calm/trauma/concept).
- Инстансный рендерер кубов (`renderer.rs` + `shaders/voxel.wgsl`) с depth буфером.
- Новая структура `VoxelWorld` без bevy_ecs: генерация террейна, метрики, операции влияния.
- Диагностический бинарь `voxelcrai-components`.
- Обновлённые README/ARCHITECTURE/QUICKSTART/PROJECT_STRUCTURE, скрипты сборки и package.

### Changed
- Карта зависимостей: удалены bevy_ecs/eframe/egui, добавлены winit 0.29, wgpu 0.19, bytemuck, anyhow.
- `LightPattern` и LightingSystem интегрированы в симуляцию, а не UI.
- `ArchGuard` теперь используется сознанием для синхронизации эмпатии.
- `build.rs` следит за `shaders/voxel.wgsl`.
- Cargo бинарники: `voxelcrai` и `voxelcrai-components`.

### Removed
- Старый UI на eframe/egui и point-cloud renderer.
- Модуль `ecs.rs` и зависимость от bevy_ecs.
- Удалён ненужный шейдер `point_cloud.wgsl`.

## [1.0.0] - 2024

### Added
- Initial release of Adaptive Entity Engine v1.0
- High-performance voxel system (9-13 KB per voxel)
  - FP64 for energy and emotions
  - FP16 for perception
  - INT8/INT4 for physics
  - Genome system with up to 10 concepts
  - Echo and resonance (16 bytes + f16)
- NextGen Evolution system (combine + mutate + fitness)
- Lighting system with LightPattern (exactly 1000 bytes)
  - Direct/indirect f16
  - Spherical Harmonics i8[256]
  - Materials u8[512]
  - AO/reflection/refraction/emission
- wgpu-based rendering (Vulkan primary, HIP/ROCm fallback for AMD Vega 20)
- Point cloud rendering support (1-1.5 billion points)
- Color mapping by energy (yellow = maximum)
- ArchGuard Enterprise protection system
  - Circuit-breaker
  - Prometheus metrics
  - Empathy ratio
  - Rhythm detector (0.038 Hz)
- egui + eframe UI
- Trauma mode (increases intensity)
- Bare-metal AArch64 support (boot.s in arm/)
- ECS system using bevy_ecs
- Single executable (50-100 MB target for Windows/Linux)

### Technical Details
- Rust 2021 edition
- Optimized release builds with LTO
- Cross-platform support (Windows/Linux)
- Bare-metal AArch64 support
