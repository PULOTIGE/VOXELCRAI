// Pattern Lighting System for UE5
// Pattern Preview Widget Implementation

#include "SPatternPreviewWidget.h"
#include "Widgets/Layout/SBox.h"
#include "Widgets/Layout/SSeparator.h"
#include "Widgets/Text/STextBlock.h"
#include "Widgets/Input/SComboBox.h"
#include "Widgets/Input/SSpinBox.h"
#include "Widgets/Colors/SColorBlock.h"
#include "Rendering/DrawElements.h"

#define LOCTEXT_NAMESPACE "SPatternPreviewWidget"

void SPatternPreviewWidget::Construct(const FArguments& InArgs)
{
	// Initialize pattern options
	PatternOptions.Add(MakeShareable(new FString(TEXT("Steady"))));
	PatternOptions.Add(MakeShareable(new FString(TEXT("Pulse"))));
	PatternOptions.Add(MakeShareable(new FString(TEXT("Flicker"))));
	PatternOptions.Add(MakeShareable(new FString(TEXT("Strobe"))));
	PatternOptions.Add(MakeShareable(new FString(TEXT("Candle"))));
	PatternOptions.Add(MakeShareable(new FString(TEXT("Fluorescent"))));
	PatternOptions.Add(MakeShareable(new FString(TEXT("Lightning"))));
	PatternOptions.Add(MakeShareable(new FString(TEXT("Fire"))));
	PatternOptions.Add(MakeShareable(new FString(TEXT("Alarm"))));
	PatternOptions.Add(MakeShareable(new FString(TEXT("Underwater"))));
	PatternOptions.Add(MakeShareable(new FString(TEXT("Heartbeat"))));
	PatternOptions.Add(MakeShareable(new FString(TEXT("Breathing"))));
	
	// Initialize graph history
	GraphHistory.SetNum(MaxGraphPoints);
	for (int32 i = 0; i < MaxGraphPoints; i++)
	{
		GraphHistory[i] = 0.5f;
	}
	
	ChildSlot
	[
		SNew(SVerticalBox)
		
		// Header
		+ SVerticalBox::Slot()
		.AutoHeight()
		.Padding(10)
		[
			SNew(STextBlock)
			.Text(LOCTEXT("PatternPreviewHeader", "Pattern Preview"))
			.Font(FCoreStyle::GetDefaultFontStyle("Bold", 16))
		]
		
		+ SVerticalBox::Slot()
		.AutoHeight()
		[
			SNew(SSeparator)
		]
		
		// Pattern selector
		+ SVerticalBox::Slot()
		.AutoHeight()
		.Padding(10, 5)
		[
			SNew(SHorizontalBox)
			+ SHorizontalBox::Slot()
			.FillWidth(0.3f)
			[
				SNew(STextBlock)
				.Text(LOCTEXT("Pattern", "Pattern"))
			]
			+ SHorizontalBox::Slot()
			.FillWidth(0.7f)
			[
				SNew(SComboBox<TSharedPtr<FString>>)
				.OptionsSource(&PatternOptions)
				.OnSelectionChanged(this, &SPatternPreviewWidget::OnPatternChanged)
				.OnGenerateWidget_Lambda([](TSharedPtr<FString> Item)
				{
					return SNew(STextBlock).Text(FText::FromString(*Item));
				})
				[
					SNew(STextBlock)
					.Text_Lambda([this]()
					{
						int32 Index = static_cast<int32>(CurrentPattern);
						if (Index >= 0 && Index < PatternOptions.Num())
						{
							return FText::FromString(*PatternOptions[Index]);
						}
						return FText::GetEmpty();
					})
				]
			]
		]
		
		// Speed
		+ SVerticalBox::Slot()
		.AutoHeight()
		.Padding(10, 5)
		[
			SNew(SHorizontalBox)
			+ SHorizontalBox::Slot()
			.FillWidth(0.3f)
			[
				SNew(STextBlock)
				.Text(LOCTEXT("Speed", "Speed"))
			]
			+ SHorizontalBox::Slot()
			.FillWidth(0.7f)
			[
				SNew(SSpinBox<float>)
				.MinValue(0.1f)
				.MaxValue(10.0f)
				.Value(Speed)
				.OnValueChanged(this, &SPatternPreviewWidget::OnSpeedChanged)
			]
		]
		
		// Min/Max Intensity
		+ SVerticalBox::Slot()
		.AutoHeight()
		.Padding(10, 5)
		[
			SNew(SHorizontalBox)
			+ SHorizontalBox::Slot()
			.FillWidth(0.3f)
			[
				SNew(STextBlock)
				.Text(LOCTEXT("IntensityRange", "Intensity Range"))
			]
			+ SHorizontalBox::Slot()
			.FillWidth(0.35f)
			.Padding(0, 0, 5, 0)
			[
				SNew(SSpinBox<float>)
				.MinValue(0.0f)
				.MaxValue(1.0f)
				.Value(MinIntensity)
				.OnValueChanged(this, &SPatternPreviewWidget::OnMinIntensityChanged)
			]
			+ SHorizontalBox::Slot()
			.FillWidth(0.35f)
			[
				SNew(SSpinBox<float>)
				.MinValue(0.0f)
				.MaxValue(1.0f)
				.Value(MaxIntensity)
				.OnValueChanged(this, &SPatternPreviewWidget::OnMaxIntensityChanged)
			]
		]
		
		+ SVerticalBox::Slot()
		.AutoHeight()
		[
			SNew(SSeparator)
		]
		
		// Preview light
		+ SVerticalBox::Slot()
		.AutoHeight()
		.Padding(10)
		[
			SNew(SBox)
			.HeightOverride(100)
			[
				SNew(SColorBlock)
				.Color_Lambda([this]()
				{
					float V = CurrentValue;
					return FLinearColor(V, V * 0.9f, V * 0.7f);
				})
			]
		]
		
		// Current value
		+ SVerticalBox::Slot()
		.AutoHeight()
		.Padding(10, 5)
		.HAlign(HAlign_Center)
		[
			SAssignNew(ValueText, STextBlock)
			.Text_Lambda([this]()
			{
				return FText::FromString(FString::Printf(TEXT("Intensity: %.2f"), CurrentValue));
			})
			.Font(FCoreStyle::GetDefaultFontStyle("Bold", 14))
		]
		
		// Graph placeholder (would be custom drawn)
		+ SVerticalBox::Slot()
		.FillHeight(1.0f)
		.Padding(10)
		[
			SNew(SBox)
			.MinDesiredHeight(150)
			[
				SNew(STextBlock)
				.Text(LOCTEXT("GraphPlaceholder", "Pattern Graph (Real-time visualization)"))
				.Justification(ETextJustify::Center)
			]
		]
	];
}

void SPatternPreviewWidget::Tick(const FGeometry& AllottedGeometry, const double InCurrentTime, const float InDeltaTime)
{
	SCompoundWidget::Tick(AllottedGeometry, InCurrentTime, InDeltaTime);
	
	// Update time
	CurrentTime += InDeltaTime * Speed;
	
	// Evaluate pattern
	CurrentValue = EvaluatePattern(CurrentTime);
	
	// Update graph history
	GraphHistory.RemoveAt(0);
	GraphHistory.Add(CurrentValue);
}

void SPatternPreviewWidget::OnPatternChanged(TSharedPtr<FString> NewPattern, ESelectInfo::Type SelectInfo)
{
	for (int32 i = 0; i < PatternOptions.Num(); i++)
	{
		if (*PatternOptions[i] == *NewPattern)
		{
			CurrentPattern = static_cast<ELightPattern>(i);
			break;
		}
	}
}

void SPatternPreviewWidget::OnSpeedChanged(float NewSpeed)
{
	Speed = NewSpeed;
}

void SPatternPreviewWidget::OnMinIntensityChanged(float NewValue)
{
	MinIntensity = NewValue;
}

void SPatternPreviewWidget::OnMaxIntensityChanged(float NewValue)
{
	MaxIntensity = NewValue;
}

float SPatternPreviewWidget::EvaluatePattern(float Time) const
{
	float Value = 1.0f;
	
	switch (CurrentPattern)
	{
		case ELightPattern::Steady:
			Value = 1.0f;
			break;
		case ELightPattern::Pulse:
			Value = 0.5f + 0.5f * FMath::Sin(Time * 2.0f * PI);
			break;
		case ELightPattern::Flicker:
			Value = 0.7f + 0.3f * FMath::Sin(Time * 20.0f) * FMath::Sin(Time * 7.3f);
			break;
		case ELightPattern::Strobe:
			Value = FMath::Sin(Time * 10.0f) > 0.0f ? 1.0f : 0.0f;
			break;
		case ELightPattern::Candle:
			Value = 0.8f + 0.2f * FMath::Sin(Time * 12.0f) * FMath::Sin(Time * 5.7f) * FMath::Sin(Time * 3.1f);
			break;
		case ELightPattern::Fluorescent:
		{
			float Startup = FMath::Clamp(FMath::Fmod(Time, 5.0f) / 2.0f, 0.0f, 1.0f);
			float Buzz = 0.05f * FMath::Sin(Time * 120.0f);
			Value = Startup + Buzz * Startup;
			break;
		}
		case ELightPattern::Lightning:
			Value = FMath::Pow(FMath::Max(0.0f, FMath::Sin(Time * 0.5f)), 20.0f);
			break;
		case ELightPattern::Fire:
			Value = 0.7f + 0.3f * FMath::Sin(Time * 8.0f) * FMath::Sin(Time * 4.3f) * FMath::Sin(Time * 2.1f);
			break;
		case ELightPattern::Alarm:
			Value = FMath::Sin(Time * 4.0f) > 0.0f ? 1.0f : 0.2f;
			break;
		case ELightPattern::Underwater:
			Value = 0.7f + 0.3f * FMath::Sin(Time) * FMath::Sin(Time * 0.7f);
			break;
		case ELightPattern::Heartbeat:
		{
			float Beat = FMath::Pow(FMath::Sin(Time * 2.5f), 12.0f);
			float Beat2 = FMath::Pow(FMath::Sin(Time * 2.5f + 0.3f), 12.0f) * 0.5f;
			Value = FMath::Max(Beat, Beat2);
			break;
		}
		case ELightPattern::Breathing:
			Value = 0.3f + 0.7f * (FMath::Sin(Time * 0.5f) * 0.5f + 0.5f);
			break;
		default:
			Value = 1.0f;
			break;
	}
	
	// Map to min/max range
	Value = FMath::Lerp(MinIntensity, MaxIntensity, Value);
	
	return FMath::Clamp(Value, 0.0f, 1.0f);
}

#undef LOCTEXT_NAMESPACE
