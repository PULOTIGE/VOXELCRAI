# Pattern Lighting System - Quick Start Guide

## 🚀 5-Minute Setup

### Step 1: Add a Pattern Light

**Drag and drop from Content Browser:**
1. Open Content Browser
2. Navigate to `PatternLighting Content` (enable Show Plugin Content)
3. Find `BP_PatternPointLight` or `BP_PatternSpotLight`
4. Drag into your level

**Or use Place Actors:**
1. Open Place Actors panel (Shift+1)
2. Search "Pattern"
3. Drag `Pattern Point Light` into level

### Step 2: Configure the Light

Select the light and in Details panel:

| Property | Description |
|----------|-------------|
| **Pattern** | Choose animation type (Pulse, Flicker, Fire, etc.) |
| **Speed** | Animation speed (1.0 = normal) |
| **Base Color** | Light color |
| **Base Intensity** | Brightness in lumens |
| **Light Radius** | Falloff distance |

### Step 3: Play!

Press Play (Alt+P) and watch your light animate!

## 🎨 Common Patterns

### Ambient Lighting
```
Pattern: Breathing
Speed: 0.5
Color: Warm White (255, 245, 230)
Intensity: 3000
```

### Horror Scene
```
Pattern: Flicker
Speed: 1.5
Color: Cool White (230, 240, 255)
Intensity: 2000
```

### Fire/Torch
```
Pattern: Fire
Speed: 1.2
Color: Orange (255, 150, 50)
Intensity: 5000
```

### Alarm/Emergency
```
Pattern: Alarm
Speed: 1.0
Color: Red (255, 0, 0)
Intensity: 10000
```

### Neon Sign
```
Pattern: Pulse
Speed: 0.8
Color: Pink (255, 0, 200)
Intensity: 15000
```

## 🔗 Syncing Lights

Make multiple lights animate together:

1. Select all lights to sync
2. In Details, set the same **Sync Group** name
3. Or in Blueprint:
```cpp
Light1->SyncGroup = FName("MyGroup");
Light2->SyncGroup = FName("MyGroup");
```

## 💡 Blueprint Example

```
Event BeginPlay
    │
    ├─► Get Pattern Lighting Subsystem
    │
    └─► Spawn Pattern Light
            Location: (0, 0, 300)
            Pattern: Pulse
            Color: White
            Intensity: 5000
            Radius: 1000
```

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+Shift+L | Open Pattern Lighting Window |
| | |

## 📊 Performance Tips

1. **Use Sync Groups** - Synced lights update together (faster)
2. **Limit SSR** - Expensive on large surfaces
3. **Shadow Caching** - Enable for static lights
4. **LOD** - Distant lights use simpler patterns

## 🎯 Best Practices

✅ **Do:**
- Use pattern lights for accents
- Combine with static/baked lighting
- Group similar lights
- Test on target hardware

❌ **Don't:**
- Use strobe in player view (epilepsy risk!)
- Overuse flickering lights
- Stack many overlapping lights
- Forget to set appropriate radius

## 📖 Next Steps

- Read full [README.md](README.md)
- Explore material functions
- Try custom patterns with curves
- Check [INSTALLATION.md](INSTALLATION.md) for advanced setup
