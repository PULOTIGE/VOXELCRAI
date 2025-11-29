// Pattern Lighting System for UE5
// Editor Module Build Configuration

using UnrealBuildTool;

public class PatternLightingEditor : ModuleRules
{
	public PatternLightingEditor(ReadOnlyTargetRules Target) : base(Target)
	{
		PCHUsage = ModuleRules.PCHUsageMode.UseExplicitOrSharedPCHs;
		
		PublicDependencyModuleNames.AddRange(
			new string[]
			{
				"Core",
				"CoreUObject",
				"Engine",
				"PatternLighting"
			}
		);
			
		PrivateDependencyModuleNames.AddRange(
			new string[]
			{
				"Slate",
				"SlateCore",
				"UnrealEd",
				"PropertyEditor",
				"EditorStyle",
				"EditorSubsystem",
				"InputCore",
				"AssetTools",
				"AssetRegistry",
				"LevelEditor",
				"WorkspaceMenuStructure",
				"ToolMenus"
			}
		);
	}
}
