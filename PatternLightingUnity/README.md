# Pattern Lighting System for Unity 6

🎮 **Advanced pattern-based lighting, water, shadows, and materials system for Unity 6!**

![Version](https://img.shields.io/badge/version-1.0.0-blue)
![Unity](https://img.shields.io/badge/Unity-6000.0+-purple)
![URP](https://img.shields.io/badge/URP-17.0+-green)
![License](https://img.shields.io/badge/license-MIT-green)

Based on the lighting system from **Adaptive Entity Engine**.

## ✨ Features

### 🔦 Pattern Light System
12 built-in animated light patterns:

| Pattern | Description |
|---------|-------------|
| Steady | Constant light |
| Pulse | Smooth sine wave |
| Flicker | Random flickering |
| Strobe | On/off strobe |
| Candle | Organic flame |
| Fluorescent | Startup + buzz |
| Lightning | Random flash |
| Fire | Flickering fire |
| Alarm | Emergency pulse |
| Underwater | Caustic effect |
| Heartbeat | Medical monitor |
| Breathing | Slow fade |
| Custom | Your own curve! |

### 🌊 Water System
- **Gerstner waves** for realistic motion
- **Reflections** (planar and SSR)
- **Refraction** with depth-based distortion
- **Foam** with shore detection
- **Caustics** animated underwater patterns
- **Fresnel** effect for edge reflections

### 🌑 Shadow System
- **Cascaded shadows** with custom splits
- **Contact shadows** for fine detail
- **Soft shadows** with configurable softness
- **Shadow color** tinting
- **Volumetric shadows** (optional)

### 🎨 Materials
- **PBR Material** with pattern emission
- **Emissive Material** with all patterns
- **Water Shader** with full feature set
- Material presets and easy creation

### 🖥️ Editor Tools
- **Control Panel** (Window → Pattern Lighting)
- **Live pattern preview** with graph
- **Custom inspectors** for all components
- **Quick actions** and presets

## 📦 Installation

### Via Package Manager (Recommended)

1. Open **Window → Package Manager**
2. Click **+** → **Add package from git URL**
3. Enter: `https://github.com/PULOTIGE/VOXELCRAI.git?path=/PatternLightingUnity`
4. Click **Add**

### Manual Installation

1. Download the package
2. Copy `PatternLightingUnity` folder to your project's `Packages/` folder
3. Unity will automatically import the package

## 🚀 Quick Start

### Create a Pattern Light

1. **GameObject → Light → Pattern Point Light**
2. Or add `PatternLight` component to any Light

### Configure in Inspector

```
Pattern: Pulse
Speed: 1.0
Base Color: Orange
Base Intensity: 5
```

### Use via Script

```csharp
using PatternLighting;

public class Example : MonoBehaviour
{
    PatternLight light;

    void Start()
    {
        light = GetComponent<PatternLight>();
        light.SetPattern(LightPattern.Fire);
        light.SetSpeed(1.5f);
    }

    void OnExplosion()
    {
        light.TriggerFlash(0.2f, 10f);
    }
}
```

### Create Water

1. **GameObject → 3D Object → Pattern Water Plane**
2. Configure waves, reflections, foam in Inspector

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+Shift+L | Open Pattern Lighting Panel |

## 🔧 Components

### PatternLight
Main component for animated lights.

```csharp
[RequireComponent(typeof(Light))]
public class PatternLight : MonoBehaviour
{
    public PatternLightSettings settings;
    public Color baseColor;
    public float baseIntensity;
    public string syncGroup;
    
    public void TriggerFlash(float duration, float intensity);
    public void SyncWith(PatternLight other);
    public void SetPattern(LightPattern pattern);
}
```

### PatternWater
Advanced water surface.

```csharp
public class PatternWater : MonoBehaviour
{
    public WaterSettings settings;
    
    public float GetWaveHeightAt(Vector3 position);
    public bool IsUnderwater(Vector3 position);
}
```

### PatternShadow
Enhanced shadow controller.

```csharp
public class PatternShadow : MonoBehaviour
{
    public PatternShadowSettings settings;
    public float shadowDistance;
    
    public float[] CalculateCascadeSplits(float near, float far);
}
```

### PatternLightingManager
Global system manager (auto-created).

```csharp
public class PatternLightingManager : MonoBehaviour
{
    public static PatternLightingManager Instance;
    public PatternLightingConfig Config;
    
    public float GetCombinedIntensityAt(Vector3 position);
    public Color GetCombinedColorAt(Vector3 position);
    public void TriggerFlashAtPosition(Vector3 pos, float radius);
    public void SyncGroup(string groupName);
}
```

## 🎨 Shaders

### Pattern Lighting/PBR
Full PBR material with pattern emission support.

### Pattern Lighting/Emissive
Emissive material with all pattern types.

### Pattern Lighting/Water
Advanced water with waves, reflections, foam, caustics.

## ⚙️ Configuration

### Global Settings

```csharp
var config = PatternLightingManager.Instance.Config;
config.enabled = true;
config.globalIntensity = 1.0f;
config.globalSpeed = 1.0f;
config.enablePBR = true;
config.enableSSR = true;
```

### Pattern Settings

```csharp
var settings = new PatternLightSettings();
settings.pattern = LightPattern.Candle;
settings.speed = 1.2f;
settings.minIntensity = 0.3f;
settings.maxIntensity = 1.0f;
settings.enableColorShift = true;
```

### Water Settings

```csharp
var water = new WaterSettings();
water.quality = WaterQuality.High;
water.waveHeight = 0.5f;
water.waveSpeed = 1.0f;
water.enableReflections = true;
water.enableFoam = true;
water.enableCaustics = true;
```

## 🎮 Sync Groups

Synchronize multiple lights:

```csharp
// Set same sync group
light1.syncGroup = "RoomLights";
light2.syncGroup = "RoomLights";
light3.syncGroup = "RoomLights";

// Sync all in group
PatternLightingManager.Instance.SyncGroup("RoomLights");
```

## 📊 Performance

| Feature | Performance Impact |
|---------|-------------------|
| Pattern Lights (10) | ~0.1ms |
| Pattern Lights (100) | ~0.5ms |
| Water (1 surface) | ~0.5ms |
| Water Reflections | ~1.0ms |
| Caustics | ~0.2ms |

### Optimization Tips

1. Use sync groups for batching
2. Lower reflection resolution for distant water
3. Disable caustics on mobile
4. Use quality presets per platform

## 🎯 Best Practices

✅ **Do:**
- Use pattern lights for atmosphere
- Combine with baked lighting
- Group similar lights
- Use presets for consistency

❌ **Don't:**
- Overuse strobe (epilepsy warning!)
- Stack many overlapping lights
- Use ultra quality on mobile
- Forget to set light range

## 📁 Samples

Import samples from Package Manager:

- **Basic Setup** - Simple scene with pattern lights
- **Water Demo** - Ocean, pool, and river examples
- **Lighting Demo** - All pattern types showcase

## 🐛 Troubleshooting

### Lights not animating
- Check PatternLightingManager exists
- Verify `enabled` is true in config
- Check Time.timeScale > 0

### Water looks wrong
- Ensure URP is configured
- Check camera has depth texture enabled
- Verify shader is compiled

### Performance issues
- Reduce light count
- Lower water quality
- Disable reflections
- Use sync groups

## 📜 License

MIT License - Based on Adaptive Entity Engine

## 🙏 Credits

- **Adaptive Entity Engine Team** - Original lighting system
- **Unity Technologies** - Unity Engine and URP

---

Made with ❤️ for Unity developers
