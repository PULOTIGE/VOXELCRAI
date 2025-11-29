// Pattern Lighting System for UE5
// Editor Commands

#pragma once

#include "CoreMinimal.h"
#include "Framework/Commands/Commands.h"
#include "PatternLightingEditorStyle.h"

class FPatternLightingEditorCommands : public TCommands<FPatternLightingEditorCommands>
{
public:
	FPatternLightingEditorCommands()
		: TCommands<FPatternLightingEditorCommands>(
			TEXT("PatternLighting"),
			NSLOCTEXT("Contexts", "PatternLighting", "Pattern Lighting Plugin"),
			NAME_None,
			FPatternLightingEditorStyle::GetStyleSetName())
	{
	}

	// TCommands interface
	virtual void RegisterCommands() override;

public:
	TSharedPtr<FUICommandInfo> OpenPluginWindow;
	TSharedPtr<FUICommandInfo> OpenPatternPreview;
	TSharedPtr<FUICommandInfo> SelectAllPatternLights;
	TSharedPtr<FUICommandInfo> SyncAllLights;
};
