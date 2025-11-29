# Quick Start Guide

## VOXELCRAI

### Prerequisites

- Rust 1.75+ (`rustup` is recommended)
- GPU драйвер с поддержкой Vulkan/Metal/DX12/GL
- Linux/macOS/Windows с рабочим графическим стеком

### Building

```bash
# Debug
cargo build

# Release
cargo build --release

# Сценарий
./scripts/build-release.sh
```

#### Bare-metal Demo (optional)

```bash
rustup target add aarch64-unknown-none
cargo build --target aarch64-unknown-none --release
```

### Running

```bash
cargo run --release
# или
./target/release/voxelcrai
```

Диагностический стенд без GPU:

```bash
cargo run --bin voxelcrai-components
```

### Controls

- `W/A/S/D` — движение камеры
- `Space` / `Shift` — вверх / вниз
- `ПКМ + мышь` — вращение
- Колесо — ускорение вдоль взгляда
- `Esc` — выход

### Key Systems

- **ConsciousnessCore**: каждые ~0.6–1.2 s принимает решения (ignite, calm, trauma toggle, seed concept).
- **Simulation**: обновляет `VoxelWorld`, запускает эволюцию раз в 2 s, синхронизирует lighting и ArchGuard.
- **Renderer**: instanced wgpu пайплайн (`InstanceRaw` → `voxel.wgsl`), глубина `Depth32Float`.
- **ArchGuard**: circuit breaker, Prometheus метрики, эмпатия сознания.
- **LightPattern**: 1000-байтные шаблоны освещения, анимируются синусом.

### Troubleshooting

- **`SurfaceError::Lost` или пустое окно**: обновите GPU драйверы / проверьте поддержку Vulkan/Metal/DX12.
- **`Validation Error`**: убедитесь, что установлены последнии версии `libvulkan` / `vulkan-sdk`.
- **`OutOfMemory`**: уменьшите разрешение окна или ограничьте количество вокселей в `VoxelWorldConfig`.

Обновление toolchain:

```bash
rustup update
```

### Documentation

- `README.md` — обзор и управление.
- `ARCHITECTURE.md` — подробности подсистем.
- `PROJECT_STRUCTURE.md` — карта файлов.
- `CHANGELOG.md` — история изменений.
