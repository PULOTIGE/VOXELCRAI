# Pattern Lighting Mod for Xonotic

🔦 **Advanced lighting system with PBR, reflections, and pattern-based effects!**

![Version](https://img.shields.io/badge/version-1.0.0-blue)
![Engine](https://img.shields.io/badge/engine-DarkPlaces-orange)
![Xonotic](https://img.shields.io/badge/xonotic-0.8.6+-green)

Based on the lighting system from **Adaptive Entity Engine**.

## ✨ Features

### 🌟 PBR Lighting
- **Physically Based Rendering** with Cook-Torrance BRDF
- Normal mapping support
- Metallic/roughness workflow
- Fresnel reflections

### 🎭 Pattern System
10 built-in light patterns:
| ID | Pattern | Description |
|----|---------|-------------|
| 0 | Steady | Constant light |
| 1 | Pulse | Smooth sine wave |
| 2 | Flicker | Random flicker |
| 3 | Strobe | On/off strobe |
| 4 | Candle | Organic flame |
| 5 | Fluorescent | Startup + buzz |
| 6 | Lightning | Random flash |
| 7 | Fire | Flickering fire |
| 8 | Alarm | Emergency pulse |
| 9 | Underwater | Caustic effect |

### 🪞 Reflections
- **Screen-Space Reflections (SSR)** for dynamic surfaces
- **Cubemap reflections** for environment
- **Fresnel-based** intensity
- Roughness blur

### 🎨 Post-Processing
- **Bloom** with lens dirt
- **ACES tonemapping**
- **Color grading** (exposure, contrast, saturation, temperature)
- **Vignette**
- **Chromatic aberration**
- **Film grain**

## 📦 Installation

### Method 1: pk3 file
1. Download `pattern_lighting.pk3`
2. Place in `Xonotic/data/`
3. Restart Xonotic

### Method 2: Manual
1. Copy `glsl/` folder to `Xonotic/data/glsl/`
2. Copy `cfg/` files to `Xonotic/data/`
3. Run: `exec pattern_lighting.cfg`

## ⚙️ Configuration

### In-Game Menu
Press **F8** to open the settings menu.

### Console Commands

```
// Toggle pattern lighting
r_pattern_lighting 1

// Adjust intensity
r_pattern_intensity 1.2

// Enable SSR
r_ssr_enable 1

// Apply presets
pl_preset_cinematic
pl_preset_performance
pl_preset_ultra
```

### All CVars

| CVar | Default | Description |
|------|---------|-------------|
| `r_pattern_lighting` | 1 | Enable pattern system |
| `r_pattern_intensity` | 1.0 | Global intensity |
| `r_pattern_speed` | 1.0 | Animation speed |
| `r_pattern_direct_color` | "1 1 0.9" | Direct light RGB |
| `r_pattern_indirect_color` | "0.4 0.5 0.7" | Indirect light RGB |
| `r_ssr_enable` | 1 | Enable SSR |
| `r_ssr_maxsteps` | 64 | SSR quality |
| `r_pp_bloom` | 1 | Enable bloom |
| `r_pp_bloom_intensity` | 0.3 | Bloom strength |
| `r_pp_exposure` | 0 | Exposure (-2 to +2) |
| `r_pp_contrast` | 1.0 | Contrast |
| `r_pp_saturation` | 1.0 | Saturation |
| `r_pp_vignette` | 1 | Enable vignette |

## 🎮 For Mappers

### Pattern Light Entity
```
classname: light_pattern
pattern_id: 0-9 (see table above)
pattern_speed: 1.0
pattern_color: "1 0.5 0"
pattern_intensity: 1.0
pattern_radius: 300
```

### Emissive Surface
```
classname: func_emissive
emit_pattern: 0-9
emit_speed: 1.0
emit_color: "1 0.5 0"
emit_intensity: 2.0
```

### Reflection Probe
```
classname: env_reflection_probe
probe_resolution: 256
probe_range: 1000
probe_update_rate: 1
```

## 📊 Performance

| Preset | FPS Impact | Quality |
|--------|------------|---------|
| Performance | ~5% | Basic lighting |
| Default | ~15% | Balanced |
| Cinematic | ~25% | Film look |
| Ultra | ~35% | Maximum quality |

## 🔧 Troubleshooting

### Shaders not loading
- Make sure `r_glsl 1` is enabled
- Check that DarkPlaces version supports GLSL

### SSR not working
- Verify `r_ssr_enable 1`
- Some maps may not support it

### Performance issues
- Use `pl_preset_performance`
- Lower `r_ssr_maxsteps` to 32
- Disable `r_pp_bloom`

## 📝 Technical Details

### Shader Files
- `pattern_lighting.glsl` - Main PBR shader
- `ssr_reflection.glsl` - Screen-space reflections
- `postprocess.glsl` - Post-processing effects
- `emissive.glsl` - Emissive materials

### Engine Requirements
- DarkPlaces engine with GLSL support
- OpenGL 3.0+ / GLES 3.0+
- At least 512MB VRAM

## 📜 License

MIT License - Based on Adaptive Entity Engine

## 🙏 Credits

- **Adaptive Entity Engine Team** - Original lighting system
- **Xonotic Team** - DarkPlaces engine
- **id Software** - Quake engine foundation

---

Made with ❤️ for the Xonotic community
