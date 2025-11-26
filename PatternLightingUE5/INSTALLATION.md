# Pattern Lighting System - Installation Guide

## Requirements

- Unreal Engine 5.0 or higher
- Visual Studio 2022 (Windows) or Xcode (Mac)
- C++ project (or convert Blueprint project to C++)

## Installation Methods

### Method 1: Plugin Folder (Recommended)

1. **Download** the plugin folder `PatternLighting`

2. **Create Plugins folder** in your project root (if it doesn't exist):
   ```
   YourProject/
   ├── Content/
   ├── Plugins/           <-- Create this
   │   └── PatternLighting/
   └── YourProject.uproject
   ```

3. **Copy** the `PatternLighting` folder into `Plugins/`

4. **Rebuild** your project:
   - Right-click on `.uproject` file
   - Select "Generate Visual Studio project files"
   - Open the solution and build

5. **Enable** the plugin:
   - Open your project in Unreal Editor
   - Go to Edit → Plugins
   - Find "Pattern Lighting System" under Rendering
   - Enable it and restart the editor

### Method 2: Engine Plugin

For all projects to use this plugin:

1. Copy `PatternLighting` folder to:
   - **Windows**: `C:/Program Files/Epic Games/UE_5.x/Engine/Plugins/`
   - **Mac**: `/Users/Shared/Epic Games/UE_5.x/Engine/Plugins/`

2. Restart Unreal Engine

### Method 3: Source Integration

1. Copy source files to your project's `Source/` folder
2. Add module dependencies to your `.Build.cs`:
   ```csharp
   PublicDependencyModuleNames.AddRange(new string[] { 
       "PatternLighting" 
   });
   ```

## Verification

After installation, verify the plugin is working:

1. Open Window → Pattern Lighting (or press Ctrl+Shift+L)
2. Check Output Log for "Pattern Lighting System initialized"
3. Place a `Pattern Point Light` actor in your level

## Troubleshooting

### Plugin not appearing
- Make sure the folder structure is correct
- Check that `.uplugin` file is in the root of `PatternLighting` folder
- Regenerate project files

### Compilation errors
- Ensure you have the correct UE5 version
- Check that all dependencies are installed
- Try building from Visual Studio instead of editor

### Runtime errors
- Check Output Log for error messages
- Verify your GPU supports the required features
- Try disabling SSR or other advanced features

## Updating

1. Backup any modified files
2. Delete old `PatternLighting` folder
3. Copy new version
4. Regenerate project files
5. Rebuild

## Uninstalling

1. Disable the plugin in Edit → Plugins
2. Delete the `PatternLighting` folder from Plugins
3. Regenerate project files

## Support

For issues, feature requests, or contributions:
- GitHub: https://github.com/PULOTIGE/VOXELCRAI
- Create an issue with:
  - UE5 version
  - OS and hardware
  - Steps to reproduce
  - Error messages from Output Log
