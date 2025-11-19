# Project Structure

```
adaptive-entity-engine/
├── src/                          # Source code
│   ├── main.rs                   # Entry point
│   ├── lib.rs                    # Library exports
│   ├── voxel.rs                  # Voxel system (9-13 KB per voxel)
│   ├── evolution.rs              # NextGen Evolution (combine + mutate + fitness)
│   ├── lighting.rs               # Lighting system (LightPattern: 1000 bytes)
│   ├── renderer.rs               # wgpu renderer (Vulkan + HIP/ROCm fallback)
│   ├── archguard.rs              # ArchGuard Enterprise protection
│   ├── ui.rs                     # egui + eframe UI
│   ├── ecs.rs                    # ECS utilities (bevy_ecs)
│   └── shaders/
│       └── point_cloud.wgsl      # Point cloud shader
│
├── arm/                          # Bare-metal AArch64 support
│   ├── boot.s                    # Boot code
│   └── linker.ld                 # Linker script
│
├── scripts/                      # Build and packaging scripts
│   ├── build-release.sh          # Release build script
│   └── package.sh                # ZIP packaging script
│
├── .cargo/                       # Cargo configuration
│   └── config.toml               # Cross-compilation settings
│
├── Cargo.toml                    # Project manifest
├── build.rs                      # Build script
├── Makefile                      # Make targets
├── README.md                     # Main documentation
├── ARCHITECTURE.md               # Architecture documentation
├── CHANGELOG.md                  # Version history
├── LICENSE-MIT                   # MIT License
├── LICENSE-APACHE                # Apache 2.0 License
├── .gitignore                    # Git ignore rules
├── rustfmt.toml                  # Rustfmt configuration
└── .clippy.toml                  # Clippy configuration
```

## Module Dependencies

```
main.rs
  ├── voxel.rs
  │   └── bevy_ecs
  ├── evolution.rs
  │   └── voxel::Genome
  ├── lighting.rs
  │   └── half::f16
  ├── renderer.rs
  │   ├── wgpu
  │   └── winit
  ├── archguard.rs
  │   └── prometheus
  ├── ui.rs
  │   ├── eframe
  │   ├── egui
  │   ├── voxel::VoxelWorld
  │   ├── evolution::EvolutionEngine
  │   ├── lighting::LightingSystem
  │   └── archguard::ArchGuard
  └── ecs.rs
      └── bevy_ecs
```

## Key Files

### Core Engine
- `src/voxel.rs`: Воксельная система с FP64/FP16/INT8/INT4, геномом, эволюцией
- `src/evolution.rs`: Эволюционный алгоритм NextGen
- `src/lighting.rs`: Система освещения с LightPattern (1000 байт)

### Rendering
- `src/renderer.rs`: wgpu рендерер с поддержкой Vulkan и HIP/ROCm fallback
- `src/shaders/point_cloud.wgsl`: WGSL шейдер для point cloud

### Protection
- `src/archguard.rs`: ArchGuard Enterprise (circuit-breaker, prometheus, empathy_ratio, rhythm detector)

### UI
- `src/ui.rs`: egui интерфейс с управлением движком

### Build
- `Cargo.toml`: Зависимости и конфигурация сборки
- `build.rs`: Build script для встраивания шейдеров
- `Makefile`: Make targets для сборки
- `scripts/build-release.sh`: Скрипт сборки release версии

### Bare-metal
- `arm/boot.s`: AArch64 boot code
- `arm/linker.ld`: Linker script для bare-metal
