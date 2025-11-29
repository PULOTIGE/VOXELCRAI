// Pattern Lighting System for UE5
// Main Editor Window Widget

#pragma once

#include "CoreMinimal.h"
#include "Widgets/SCompoundWidget.h"
#include "PatternTypes.h"

class SPatternLightingWindow : public SCompoundWidget
{
public:
	SLATE_BEGIN_ARGS(SPatternLightingWindow) {}
	SLATE_END_ARGS()

	void Construct(const FArguments& InArgs);

private:
	// Global settings
	FPatternLightingConfig GlobalConfig;
	
	// UI callbacks
	void OnGlobalIntensityChanged(float NewValue);
	void OnGlobalSpeedChanged(float NewValue);
	void OnEnablePBRChanged(ECheckBoxState NewState);
	void OnEnableSSRChanged(ECheckBoxState NewState);
	void OnEnableVolumetricsChanged(ECheckBoxState NewState);
	void OnEnableAOChanged(ECheckBoxState NewState);
	
	FReply OnApplySettings();
	FReply OnResetSettings();
	FReply OnSelectAllLights();
	FReply OnSyncAllLights();
	
	// Helpers
	void RefreshSettings();
	void ApplyToSubsystem();
	
	TSharedPtr<class SScrollBox> ContentBox;
};
