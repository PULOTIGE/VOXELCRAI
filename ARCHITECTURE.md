# Architecture Documentation

## VOXELCRAI

### Overview

VOXELCRAI — рендерер сознательного воксельного мира. Движок работает на чистом wgpu/winit, комбинирует физику вокселей, эволюцию генома и управляющее сознание, которое наблюдает за состоянием мира и изменяет его энергию.

### Data & Simulation Layer

#### Voxel System (`src/voxel.rs`)

- Воксель ~9–13 КБ: FP64 энергия/эмоции, 10 каналов восприятия (FP16), INT8 физика, packed флаги, геном до 10 концептов, echo+resonance, метаданные.
- `VoxelWorld` генерирует террейн по синусоидальному шуму, поддерживает до `max_voxels` сущностей, обновляет энергию, эмоции и вектор скорости по таймеру.
- `WorldMetrics` агрегирует среднее/максимальное значение энергии, энтропию распределения, центроид, холодную и горячую точки.
- Операции: `affect_cluster` (локальное усиление/приглушение энергии), `embed_concept`, `spawn_voxel`.

#### Evolution (`src/evolution.rs`)

- Классический GA: crossover, mutate, fitness (энергия + резонанс + эмоции + разнообразие восприятия).
- Применяется каждые 2 секунды симуляции ко второй половине популяции.

#### Consciousness (`src/consciousness.rs`)

- `ConsciousnessCore` хранит настроение, любопытство, эмпатию и стабилизацию.
- Метод `think` генерирует `ConsciousnessPulse`: набор действий (ignite, calm, trauma toggle, seed concept) и телеметрию.
- Метрики синхронизируются с ArchGuard (empathy gauge).

#### Simulation (`src/simulation.rs`)

- Координирует `VoxelWorld`, `LightingSystem`, `EvolutionEngine` и `ConsciousnessCore`.
- Каждый кадр: update мира → получить `WorldMetrics` → запросить действия сознания → применить → обновить инстансы для GPU.
- `SimulationMetrics` отдаёт статусы для логгера/диагностики.

### Rendering Layer

#### Camera (`src/camera.rs`)

- `Camera` хранит позицию, yaw/pitch, проекцию.
- `CameraController` обрабатывает события winit (WASD, Space/Shift, колесо, ПКМ).
- `CameraUniform` — матрица `view_proj`.

#### Renderer (`src/renderer.rs`)

- Создаёт `Surface<'static>` через `Arc<Window>`, выбирает sRGB формат и `PresentMode::Fifo/Mailbox`.
- Пайплайн: instanced cube mesh (24 вертекса, 36 индексов), uniform с матрицей, Instanced buffer (`InstanceRaw`: позиция, масштаб, цвет, энергия).
- Пересоздаёт depth-текстуру при изменении размеров окна.
- Шейдер `shaders/voxel.wgsl`: вычисляет world-space позицию, освещает нормаль и добавляет свечение пропорционально энергии.

### Runtime / Engine (`src/engine.rs`)

- `EventLoop` из winit, `EngineState` хранит renderer, simulation, камеру и телеметрию.
- Каждую секунду пишет в лог аггрегированные показатели (кол-во вокселей, средняя энергия, настроение, эмпатия и действия VOXELCRAI).
- Обработка ошибок surface: Lost → recreate, OutOfMemory → завершение, Timeout/Outdated → пропуск кадра.

### Supporting Systems

- **Lighting** (`src/lighting.rs`): `LightPattern` на 1000 байт (SH + материалы); `LightingSystem` анимирует прямой свет синусом.
- **ArchGuard** (`src/archguard.rs`): circuit-breaker, счётчики prometheus, эмпатия через async RwLock, детектор ритма 0.038 Гц.
- **Diagnostic Binary** (`src/test_components.rs`): проверяет все подсистемы без GPU.

### Build & Targets

- Rust 2021, release профиль: `opt-level=3`, `lto=true`, `codegen-units=1`, `panic="abort"`.
- Кросс-компиляция на AArch64 поддерживается (arm/boot.s и linker.ld), но основной путь — десктопный wgpu.

### Performance Notes

1. **Инстансинг**: все воксели рендерятся в одном draw-call через `InstanceRaw`.
2. **Память**: воксели хранятся в векторе, пригодны для SIMD обработки.
3. **Сложность**: обновление мира — O(n), пересчёт метрик — O(n); эволюция выполняется значительно реже (каждые 2 секунды).
4. **Ввод**: события winit без доп. UI библиотек обеспечивают низкую задержку.

### Roadmap

- Добавить streaming-чunks и загрузку данных с диска.
- Визуализировать действия сознания на экране (HUD).
- Реализовать HIP/ROCm fallback для конкретных AMD GPU.
- Расширить фитнес-функцию (поддержка сложных материалов и освещения).
