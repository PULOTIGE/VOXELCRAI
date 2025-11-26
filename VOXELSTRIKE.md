# 🎮 VoxelStrike - Counter-Strike Style FPS Game

A simple Counter-Strike inspired FPS game built on the **Adaptive Entity Engine**.

## 🎯 Features

- **FPS Camera** - Smooth mouse look with WASD movement
- **Multiple Weapons**:
  - 🔪 Knife - Melee weapon
  - 🔫 Desert Eagle - Powerful pistol
  - 🔫 AK-47 - Full-auto assault rifle
- **Simple Map** - de_dust inspired arena with walls, crates, and cover
- **Enemy Bots** - AI enemies that patrol, chase, and attack
- **HUD** - Health bar, ammo counter, crosshair
- **Physics** - Basic movement, jumping, collision detection

## 🕹️ Controls

| Key | Action |
|-----|--------|
| **WASD** | Move |
| **Mouse** | Look around |
| **Left Click** | Shoot |
| **Space** | Jump |
| **Shift** | Sprint |
| **Ctrl** | Crouch |
| **R** | Reload |
| **1/2/3** | Switch weapons |
| **Q/E** | Previous/Next weapon |
| **Tab** | Scoreboard |
| **Escape** | Release mouse / Exit |

## 📥 Download

Download the latest release from the [Releases](https://github.com/PULOTIGE/VOXELCRAI/releases) page.

### Windows
Download `voxelstrike.exe` and run it directly. No installation required!

### Building from Source

```bash
# Clone the repository
git clone https://github.com/PULOTIGE/VOXELCRAI.git
cd VOXELCRAI

# Build for your platform
cargo build --release --bin voxelstrike --features gui

# Run the game
cargo run --release --bin voxelstrike --features gui
```

## 🖼️ Screenshots

The game features a simple but functional 3D environment with:
- Sandstone-colored walls and floors
- Brown wooden crates for cover
- Blue (CT) and Orange (T) team colors
- Green crosshair HUD
- Health and ammo indicators

## 🛠️ Technical Details

Built with:
- **Rust** - Systems programming language
- **wgpu** - Cross-platform GPU graphics API
- **winit** - Window handling and input
- **glam** - Fast linear algebra library

The game runs on:
- ✅ Windows (x86_64)
- ✅ Linux (with Vulkan/OpenGL support)
- ✅ macOS (with Metal support)

## 📝 License

MIT OR Apache-2.0 (same as Adaptive Entity Engine)

---

*Built with ❤️ using the Adaptive Entity Engine*
