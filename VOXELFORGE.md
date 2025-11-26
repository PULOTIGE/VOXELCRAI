# VoxelForge - 3D Game Constructor

🎮 **Visual editor for creating FPS games based on the VoxelStrike engine!**

![VoxelForge Editor](https://img.shields.io/badge/VoxelForge-v1.0.0-blue?style=for-the-badge)
![Rust](https://img.shields.io/badge/Rust-🦀-orange?style=for-the-badge)
![wgpu](https://img.shields.io/badge/wgpu-Graphics-green?style=for-the-badge)

## ✨ Features

### 🎨 3D Viewport
- **Orbit camera** - Right-click + drag to rotate around scene
- **Pan camera** - Middle-click + drag to pan
- **Zoom** - Mouse scroll to zoom in/out
- **Grid** - Visual reference grid with axis indicators
- **Real-time rendering** - See your changes instantly

### 🔧 Transform Tools
| Hotkey | Tool | Description |
|--------|------|-------------|
| Q | Select | Select objects in scene |
| W | Move | Translate objects on X/Y/Z axes |
| E | Rotate | Rotate objects around axes |
| R | Scale | Scale objects uniformly or per-axis |
| T | Place | Place new objects in scene |

### 📦 Asset Browser
- **Built-in Models**
  - Primitives: Box, Sphere, Cylinder, Capsule, Plane, Wedge, Arch, Stairs
  - Props: Crate, Barrel, Door, Window, Fence, Lamp
  - Characters: Terrorist, Counter-Terrorist, Hostage
  - Weapons: AK-47, M4A1, AWP, Desert Eagle, Knife, Bomb

- **Built-in Textures**
  - Concrete, Brick, Sandstone, Wood, Metal, Glass
  - Grass, Dirt, Water, Lava, Tile, Plaster

### 🎯 Gameplay Settings
- **Game Modes**
  - Deathmatch (FFA)
  - Team Deathmatch
  - Bomb Defusal
  - Hostage Rescue
  - Gun Game
  - Custom

- **Configurable Options**
  - Round time, respawn settings, freeze time
  - Economy system (buy menu, money rewards)
  - Weapon properties (damage, fire rate, accuracy, recoil)
  - Bot AI difficulty and behavior

### 🌟 Scene Objects
- **Lighting**
  - Directional light (sun)
  - Point lights
  - Spot lights

- **Gameplay Elements**
  - Spawn points (T/CT)
  - Bombsites
  - Buy zones
  - Triggers

## ⌨️ Keyboard Shortcuts

### File
| Shortcut | Action |
|----------|--------|
| Ctrl+N | New Project |
| Ctrl+O | Open Project |
| Ctrl+S | Save |
| Ctrl+Shift+S | Save As |

### Edit
| Shortcut | Action |
|----------|--------|
| Ctrl+Z | Undo |
| Ctrl+Y | Redo |
| Delete | Delete selected |
| Ctrl+D | Duplicate selected |

### View
| Shortcut | Action |
|----------|--------|
| F | Focus on selected |
| G | Toggle grid |

### Play Mode
| Shortcut | Action |
|----------|--------|
| Ctrl+P | Play/Stop |
| Space | Pause (while playing) |

## 📥 Download

Download the latest release:

**[VoxelForge v1.0.0 - Windows](https://github.com/PULOTIGE/VOXELCRAI/releases)**

## 🚀 Quick Start

1. **Download** `voxelforge.exe` from releases
2. **Run** the executable
3. **Create** a new project (Ctrl+N)
4. **Choose** a template (Deathmatch, Team DM, Bomb Defusal)
5. **Build** your map using tools and assets
6. **Play** to test your creation (Ctrl+P)
7. **Export** to standalone game

## 🏗️ Project Structure

```
MyGame/
├── MyGame.vforge          # Project file
├── scenes/
│   ├── main.vscene       # Main scene
│   └── level2.vscene     # Additional levels
├── assets/
│   ├── models/           # Custom 3D models
│   ├── textures/         # Custom textures
│   └── sounds/           # Audio files
└── config/
    └── gameplay.json     # Gameplay settings
```

## 🛠️ Technical Details

- **Language**: Rust
- **Graphics**: wgpu (cross-platform GPU abstraction)
- **Window**: winit
- **Serialization**: serde/JSON
- **Target**: Windows (x86_64)

## 📋 Requirements

- Windows 10/11 (64-bit)
- DirectX 12 / Vulkan compatible GPU
- 4GB RAM minimum
- 500MB disk space

## 🎨 Creating Your First Map

### 1. Basic Layout
- Press **1** in Place mode to add a Box
- Scale it to create floor (R key, then drag)
- Add walls around the perimeter

### 2. Add Spawn Points
- From Create menu → Gameplay → T Spawn Point
- Place at least 5 for each team
- Create → Gameplay → CT Spawn Point

### 3. Add Lighting
- Create → Lights → Directional Light
- Rotate to set sun angle
- Add Point Lights for indoor areas

### 4. Props and Cover
- Add crates and barrels for cover
- Use primitives to create custom structures
- Apply materials from Asset Browser

### 5. Gameplay Elements
- For Bomb Defusal: Add Bombsite objects
- Add Buy Zones near spawn points
- Configure in Gameplay Settings

## 📝 License

MIT / Apache-2.0

---

Made with ❤️ using the VoxelStrike Engine
