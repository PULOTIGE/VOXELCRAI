// Pattern Lighting System for UE5
// Editor Commands Implementation

#include "PatternLightingEditorCommands.h"

#define LOCTEXT_NAMESPACE "FPatternLightingEditorModule"

void FPatternLightingEditorCommands::RegisterCommands()
{
	UI_COMMAND(OpenPluginWindow, "Pattern Lighting", "Open Pattern Lighting settings window", EUserInterfaceActionType::Button, FInputChord(EModifierKey::Control | EModifierKey::Shift, EKeys::L));
	UI_COMMAND(OpenPatternPreview, "Pattern Preview", "Open Pattern Preview window", EUserInterfaceActionType::Button, FInputChord());
	UI_COMMAND(SelectAllPatternLights, "Select All Pattern Lights", "Select all pattern lights in level", EUserInterfaceActionType::Button, FInputChord());
	UI_COMMAND(SyncAllLights, "Sync All Lights", "Synchronize all pattern lights", EUserInterfaceActionType::Button, FInputChord());
}

#undef LOCTEXT_NAMESPACE
