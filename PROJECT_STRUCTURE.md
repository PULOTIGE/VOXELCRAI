# Project Structure

```
voxelcrai/
├── src/
│   ├── main.rs            # запуск event loop
│   ├── engine.rs          # EngineState (renderer + simulation + camera)
│   ├── camera.rs          # Camera + Controller + Uniform
│   ├── renderer.rs        # wgpu рендерер, instancing pipeline
│   ├── simulation.rs      # связка мира, эволюции, сознания
│   ├── consciousness.rs   # VOXELCRAI Core AI
│   ├── voxel.rs           # данные вокселя, генерация мира, метрики
│   ├── evolution.rs       # NextGen Evolution
│   ├── lighting.rs        # LightPattern (1000 B) и LightingSystem
│   ├── archguard.rs       # ArchGuard Enterprise защита
│   ├── test_components.rs # диагностический стенд
│   └── shaders/
│       └── voxel.wgsl     # WGSL шейдер для кубов
│
├── arm/                   # Bare-metal AArch64 артефакты
│   ├── boot.s
│   └── linker.ld
│
├── scripts/               # Утилиты сборки
│   ├── build-release.sh
│   └── package.sh
│
├── Cargo.toml             # Manifest + зависимости
├── build.rs               # Пересборка при изменении шейдера
├── README.md / ARCHITECTURE.md / QUICKSTART.md
└── LICENSE-APACHE / LICENSE-MIT / CHANGELOG.md / др.
```

## Module Dependencies

```
main.rs
 └── engine.rs
      ├── renderer.rs
      │    └── camera.rs
      ├── simulation.rs
      │    ├── voxel.rs
      │    ├── evolution.rs
      │    ├── lighting.rs
      │    └── consciousness.rs
      │          └── archguard.rs
      └── log/env_logger (через main)
```

## Key Files

- `src/engine.rs` — точка входа run-loop, обработка ввода и ошибок поверхности.
- `src/voxel.rs` — определение вокселя, генерация мира, метрики и операции влияния.
- `src/consciousness.rs` — принятие решений VOXELCRAI, действия (ignite/calm/trauma/concept).
- `src/simulation.rs` — общий цикл обновления, эволюция и подготовка GPU-инстансов.
- `src/renderer.rs` + `shaders/voxel.wgsl` — Vulkan/wgpu пайплайн, instanced rendering.
- `src/lighting.rs` — LightPattern ровно 1000 байт и их анимация.
- `src/archguard.rs` — circuit breaker, прометей-метрики, эмпатия, ритм.
- `src/test_components.rs` — CLI-проверка подсистем без GPU.

## Build Assets

- `scripts/build-release.sh` — релизная сборка и вывод размера бинаря.
- `scripts/package.sh` — упаковка проекта в zip.
- `arm/*` — демонстрационный bare-metal bootstrap (не участвует в основном рендерере).
