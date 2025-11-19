# Quick Start Guide

## Adaptive Entity Engine v1.0

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)
- Vulkan drivers (for rendering)
- Cargo (comes with Rust)

### Building

#### Standard Build (Windows/Linux)

```bash
# Debug build
cargo build

# Release build (optimized, 50-100 MB target)
cargo build --release

# Or use the build script
./scripts/build-release.sh
```

#### Bare-metal AArch64 Build

```bash
# Install target
rustup target add aarch64-unknown-none

# Build
cargo build --target aarch64-unknown-none --release
```

### Running

```bash
# Run release version
cargo run --release

# Or directly
./target/release/adaptive-entity-engine
```

### Features

#### Trauma Mode
Toggle trauma mode in the UI to increase energy and emotional intensity:
- Energy: ×1.5
- Emotional arousal: ×1.3

#### Evolution
Click "Evolve Population" to run the NextGen Evolution algorithm on voxels.

#### Lighting
Add light patterns using the "Add Light Pattern" button. Each pattern is exactly 1000 bytes.

#### Point Cloud
The engine supports rendering up to 1.5 billion points, colored by energy (yellow = maximum).

### Architecture

- **Voxels**: 9-13 KB each with FP64/FP16/INT8/INT4, genome, echo, resonance
- **Lighting**: LightPattern structure (exactly 1000 bytes)
- **Rendering**: wgpu (Vulkan) with HIP/ROCm fallback for AMD Vega 20
- **Protection**: ArchGuard Enterprise (circuit-breaker, prometheus, empathy_ratio, 0.038 Hz rhythm detector)
- **UI**: egui + eframe

### Troubleshooting

#### Vulkan Not Found
Install Vulkan drivers for your platform:
- Windows: Install latest GPU drivers
- Linux: Install `vulkan-loader` and GPU-specific drivers

#### Build Errors
Ensure you have the latest Rust toolchain:
```bash
rustup update
```

### Documentation

- `README.md`: Main documentation
- `ARCHITECTURE.md`: Detailed architecture
- `PROJECT_STRUCTURE.md`: Project structure overview
- `CHANGELOG.md`: Version history
