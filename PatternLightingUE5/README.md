# Pattern Lighting System for Unreal Engine 5

🔦 **Advanced pattern-based lighting, reflections, and shadows plugin for UE5!**

![Version](https://img.shields.io/badge/version-1.0.0-blue)
![UE5](https://img.shields.io/badge/Unreal%20Engine-5.0+-purple)
![License](https://img.shields.io/badge/license-MIT-green)

Based on the lighting system from **Adaptive Entity Engine**.

## ✨ Features

### 🎭 Pattern Light Component
12 built-in light patterns with customization:

| Pattern | Description |
|---------|-------------|
| Steady | Constant intensity |
| Pulse | Smooth sine wave |
| Flicker | Random flicker |
| Strobe | On/off strobe |
| Candle | Organic flame effect |
| Fluorescent | Startup + buzz |
| Lightning | Random flash |
| Fire | Flickering fire |
| Alarm | Emergency pulse |
| Underwater | Caustic effect |
| Heartbeat | Medical monitor |
| Breathing | Slow fade |
| Custom | Use your own curve! |

### 🪞 Reflection System
- **Screen-Space Reflections (SSR)** with configurable quality
- **Planar reflections** for floors and water
- **Reflection probes** with box projection
- **Fresnel-based** intensity
- **Roughness blur** for realistic surfaces

### 🌑 Shadow System
- **Cascaded Shadow Maps** with custom splits
- **Contact shadows** for detail
- **Volumetric shadows** for atmosphere
- **Soft shadow** penumbra calculation
- **Shadow color** tinting

### 🎨 PBR Materials
- **Cook-Torrance BRDF** implementation
- **GGX Distribution** for specular
- **Smith Geometry** term
- **Schlick Fresnel** approximation
- Material functions for custom shaders

## 📦 Installation

1. Copy `PatternLighting` folder to your project's `Plugins/` directory
2. Restart Unreal Editor
3. Enable plugin in Edit → Plugins → Pattern Lighting
4. Restart editor when prompted

## 🚀 Quick Start

### Adding a Pattern Light

**Blueprint:**
```
1. Place "Pattern Point Light" or "Pattern Spot Light" actor
2. Configure pattern in Details panel:
   - Pattern Type
   - Speed
   - Base Color
   - Intensity
```

**C++:**
```cpp
#include "PatternLightComponent.h"

// Create pattern light
UPatternLightComponent* Light = NewObject<UPatternLightComponent>(this);
Light->PatternSettings.Pattern = ELightPattern::Fire;
Light->PatternSettings.Speed = 1.5f;
Light->BaseColor = FLinearColor(1.0f, 0.5f, 0.1f);
Light->BaseIntensity = 5000.0f;
Light->RegisterComponent();
```

### Using the Subsystem

```cpp
// Get subsystem
UPatternLightingSubsystem* Subsystem = GetWorld()->GetSubsystem<UPatternLightingSubsystem>();

// Query lights at location
TArray<UPatternLightComponent*> Lights = Subsystem->GetLightsAtLocation(Location, 1000.0f);

// Get combined intensity
float Intensity = Subsystem->GetCombinedIntensityAt(Location);

// Trigger flash effect
Subsystem->TriggerFlashAtLocation(Location, 500.0f, 0.1f, 10.0f);
```

### Blueprint Functions

| Function | Description |
|----------|-------------|
| `GetPatternLightingSubsystem` | Get the subsystem reference |
| `SpawnPatternLight` | Create a pattern light at location |
| `EvaluatePattern` | Get pattern value at time |
| `CalculatePBRSpecular` | Calculate PBR specular lighting |
| `CalculateFresnelReflection` | Get Fresnel term |
| `ColorTemperatureToRGB` | Convert Kelvin to color |

## ⚙️ Configuration

### Global Settings

```cpp
FPatternLightingConfig Config;
Config.bEnabled = true;
Config.GlobalIntensity = 1.0f;
Config.GlobalSpeed = 1.0f;
Config.bEnablePBR = true;
Config.bEnableSSR = true;
Config.bEnhancedAO = true;

Subsystem->GlobalConfig = Config;
```

### Pattern Settings

```cpp
FPatternLightSettings Settings;
Settings.Pattern = ELightPattern::Candle;
Settings.Speed = 1.0f;
Settings.PhaseOffset = 0.0f;
Settings.MinIntensity = 0.2f;
Settings.MaxIntensity = 1.0f;
Settings.bEnableColorShift = true;
Settings.ColorCurve = MyColorCurve;

Light->PatternSettings = Settings;
```

### Reflection Settings

```cpp
FPatternReflectionSettings ReflSettings;
ReflSettings.Quality = EReflectionQuality::High;
ReflSettings.Intensity = 1.0f;
ReflSettings.Radius = 1000.0f;
ReflSettings.FresnelExponent = 5.0f;
ReflSettings.bRoughnessBlur = true;
ReflSettings.SSRMaxDistance = 1000.0f;
ReflSettings.SSRSteps = 64;

Reflection->ReflectionSettings = ReflSettings;
```

### Shadow Settings

```cpp
FPatternShadowSettings ShadowSettings;
ShadowSettings.Quality = EShadowQuality::High;
ShadowSettings.Intensity = 1.0f;
ShadowSettings.Softness = 1.0f;
ShadowSettings.bContactShadows = true;
ShadowSettings.ContactShadowLength = 0.1f;
ShadowSettings.CascadeCount = 4;

Shadow->ShadowSettings = ShadowSettings;
```

## 🎨 Material Functions

The plugin includes HLSL material functions in `Content/Materials/`:

- **MF_EvaluatePattern** - Evaluate light patterns in materials
- **MF_PBRLighting** - Full PBR lighting calculation
- **MF_Fresnel** - Enhanced Fresnel with IOR
- **MF_ContactShadow** - Contact shadow calculation
- **MF_VolumetricScatter** - Volumetric scattering

### Using in Materials

1. Create new Material Function
2. Add Custom node
3. Paste HLSL code from `MF_PatternLighting.txt`
4. Configure inputs/outputs
5. Use in your materials

## 🔧 Sync Groups

Synchronize multiple lights together:

```cpp
// Set same sync group
Light1->SyncGroup = FName("RoomLights");
Light2->SyncGroup = FName("RoomLights");
Light3->SyncGroup = FName("RoomLights");

// Sync all lights in group
Subsystem->SyncGroup(FName("RoomLights"));
```

## 📊 Performance

| Feature | Performance Impact |
|---------|-------------------|
| Pattern Lights (10) | ~0.1ms |
| Pattern Lights (100) | ~0.5ms |
| SSR (64 steps) | ~1.0ms |
| SSR (128 steps) | ~2.0ms |
| Contact Shadows | ~0.3ms |
| Volumetric Shadows | ~0.5ms |

### Optimization Tips

1. Use sync groups to batch updates
2. Lower SSR steps for distant reflections
3. Use reflection quality tiers
4. Enable shadow caching for static lights
5. Reduce cascade count for indoor scenes

## 🎮 Actors

### APatternPointLight
Point light with pattern animation.

### APatternSpotLight
Spot light with inner/outer cone and patterns.

### APatternDirectionalLight
Directional light with day/night cycle and cascaded shadows.

### APatternSSRVolume
Volume that enables SSR with custom settings.

### APatternPlanarReflection
Planar reflection for floors, water, mirrors.

### APatternShadowCasterVolume
Force shadow settings on objects inside.

## 📝 API Reference

See header files in `Source/PatternLighting/Public/` for full API documentation.

## 🔗 Integration

### With Niagara
The plugin works with Niagara particle systems for light-reactive particles.

### With Lumen
Pattern lights work alongside UE5's Lumen GI. For best results:
- Use pattern lights for dynamic accents
- Let Lumen handle ambient lighting
- Adjust global intensity accordingly

### With Virtual Shadow Maps
Compatible with UE5's Virtual Shadow Maps when using the shadow component.

## 🐛 Troubleshooting

### Lights not updating
- Check `bEnabled` in global config
- Verify component is registered
- Check tick is enabled

### SSR artifacts
- Increase `SSRThickness`
- Lower `SSRMaxDistance`
- Reduce `SSRSteps` if performance-bound

### Sync groups not working
- Ensure same `SyncGroup` name
- Call `SyncGroup()` after all lights registered

## 📜 License

MIT License - Based on Adaptive Entity Engine

## 🙏 Credits

- **Adaptive Entity Engine Team** - Original lighting system
- **Epic Games** - Unreal Engine 5

---

Made with ❤️ for Unreal Engine developers
