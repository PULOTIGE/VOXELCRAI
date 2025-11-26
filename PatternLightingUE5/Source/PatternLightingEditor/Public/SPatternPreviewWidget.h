// Pattern Lighting System for UE5
// Pattern Preview Widget

#pragma once

#include "CoreMinimal.h"
#include "Widgets/SCompoundWidget.h"
#include "PatternTypes.h"

/**
 * Widget for previewing light patterns
 */
class SPatternPreviewWidget : public SCompoundWidget
{
public:
	SLATE_BEGIN_ARGS(SPatternPreviewWidget) {}
	SLATE_END_ARGS()

	void Construct(const FArguments& InArgs);
	virtual void Tick(const FGeometry& AllottedGeometry, const double InCurrentTime, const float InDeltaTime) override;

private:
	// Current pattern settings
	ELightPattern CurrentPattern = ELightPattern::Pulse;
	float Speed = 1.0f;
	float MinIntensity = 0.0f;
	float MaxIntensity = 1.0f;
	
	// Animation
	float CurrentTime = 0.0f;
	float CurrentValue = 0.0f;
	
	// Graph data
	TArray<float> GraphHistory;
	int32 MaxGraphPoints = 200;
	
	// UI callbacks
	void OnPatternChanged(TSharedPtr<FString> NewPattern, ESelectInfo::Type SelectInfo);
	void OnSpeedChanged(float NewSpeed);
	void OnMinIntensityChanged(float NewValue);
	void OnMaxIntensityChanged(float NewValue);
	
	// Rendering
	int32 OnPaintGraph(const FPaintArgs& Args, const FGeometry& AllottedGeometry, const FSlateRect& MyCullingRect, FSlateWindowElementList& OutDrawElements, int32 LayerId, const FWidgetStyle& InWidgetStyle, bool bParentEnabled) const;
	
	// Pattern evaluation
	float EvaluatePattern(float Time) const;
	
	TSharedPtr<STextBlock> ValueText;
	TArray<TSharedPtr<FString>> PatternOptions;
};
