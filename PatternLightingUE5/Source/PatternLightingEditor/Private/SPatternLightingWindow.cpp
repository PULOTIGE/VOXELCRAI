// Pattern Lighting System for UE5
// Main Editor Window Implementation

#include "SPatternLightingWindow.h"
#include "PatternLightingSubsystem.h"
#include "Widgets/Layout/SScrollBox.h"
#include "Widgets/Layout/SBox.h"
#include "Widgets/Layout/SSeparator.h"
#include "Widgets/Text/STextBlock.h"
#include "Widgets/Input/SCheckBox.h"
#include "Widgets/Input/SSpinBox.h"
#include "Widgets/Input/SButton.h"
#include "EditorStyleSet.h"
#include "Engine/World.h"
#include "Editor.h"

#define LOCTEXT_NAMESPACE "SPatternLightingWindow"

void SPatternLightingWindow::Construct(const FArguments& InArgs)
{
	RefreshSettings();
	
	ChildSlot
	[
		SNew(SVerticalBox)
		
		// Header
		+ SVerticalBox::Slot()
		.AutoHeight()
		.Padding(10)
		[
			SNew(STextBlock)
			.Text(LOCTEXT("PatternLightingHeader", "Pattern Lighting System"))
			.Font(FCoreStyle::GetDefaultFontStyle("Bold", 18))
		]
		
		+ SVerticalBox::Slot()
		.AutoHeight()
		[
			SNew(SSeparator)
		]
		
		// Content
		+ SVerticalBox::Slot()
		.FillHeight(1.0f)
		.Padding(10)
		[
			SAssignNew(ContentBox, SScrollBox)
			
			// Global Settings Section
			+ SScrollBox::Slot()
			[
				SNew(SVerticalBox)
				
				// Section Header
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 10, 0, 5)
				[
					SNew(STextBlock)
					.Text(LOCTEXT("GlobalSettings", "Global Settings"))
					.Font(FCoreStyle::GetDefaultFontStyle("Bold", 14))
				]
				
				// Enable
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 5)
				[
					SNew(SHorizontalBox)
					+ SHorizontalBox::Slot()
					.AutoWidth()
					[
						SNew(SCheckBox)
						.IsChecked(GlobalConfig.bEnabled ? ECheckBoxState::Checked : ECheckBoxState::Unchecked)
						.OnCheckStateChanged_Lambda([this](ECheckBoxState State)
						{
							GlobalConfig.bEnabled = (State == ECheckBoxState::Checked);
						})
					]
					+ SHorizontalBox::Slot()
					.Padding(5, 0)
					[
						SNew(STextBlock)
						.Text(LOCTEXT("Enable", "Enable Pattern Lighting"))
					]
				]
				
				// Global Intensity
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 5)
				[
					SNew(SHorizontalBox)
					+ SHorizontalBox::Slot()
					.FillWidth(0.4f)
					[
						SNew(STextBlock)
						.Text(LOCTEXT("GlobalIntensity", "Global Intensity"))
					]
					+ SHorizontalBox::Slot()
					.FillWidth(0.6f)
					[
						SNew(SSpinBox<float>)
						.MinValue(0.0f)
						.MaxValue(2.0f)
						.Value(GlobalConfig.GlobalIntensity)
						.OnValueChanged(this, &SPatternLightingWindow::OnGlobalIntensityChanged)
					]
				]
				
				// Global Speed
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 5)
				[
					SNew(SHorizontalBox)
					+ SHorizontalBox::Slot()
					.FillWidth(0.4f)
					[
						SNew(STextBlock)
						.Text(LOCTEXT("GlobalSpeed", "Global Speed"))
					]
					+ SHorizontalBox::Slot()
					.FillWidth(0.6f)
					[
						SNew(SSpinBox<float>)
						.MinValue(0.1f)
						.MaxValue(5.0f)
						.Value(GlobalConfig.GlobalSpeed)
						.OnValueChanged(this, &SPatternLightingWindow::OnGlobalSpeedChanged)
					]
				]
				
				// Separator
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 10)
				[
					SNew(SSeparator)
				]
				
				// Features Section
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 5)
				[
					SNew(STextBlock)
					.Text(LOCTEXT("Features", "Features"))
					.Font(FCoreStyle::GetDefaultFontStyle("Bold", 14))
				]
				
				// PBR
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 5)
				[
					SNew(SHorizontalBox)
					+ SHorizontalBox::Slot()
					.AutoWidth()
					[
						SNew(SCheckBox)
						.IsChecked(GlobalConfig.bEnablePBR ? ECheckBoxState::Checked : ECheckBoxState::Unchecked)
						.OnCheckStateChanged(this, &SPatternLightingWindow::OnEnablePBRChanged)
					]
					+ SHorizontalBox::Slot()
					.Padding(5, 0)
					[
						SNew(STextBlock)
						.Text(LOCTEXT("EnablePBR", "Enable PBR Lighting"))
					]
				]
				
				// SSR
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 5)
				[
					SNew(SHorizontalBox)
					+ SHorizontalBox::Slot()
					.AutoWidth()
					[
						SNew(SCheckBox)
						.IsChecked(GlobalConfig.bEnableSSR ? ECheckBoxState::Checked : ECheckBoxState::Unchecked)
						.OnCheckStateChanged(this, &SPatternLightingWindow::OnEnableSSRChanged)
					]
					+ SHorizontalBox::Slot()
					.Padding(5, 0)
					[
						SNew(STextBlock)
						.Text(LOCTEXT("EnableSSR", "Enable Screen-Space Reflections"))
					]
				]
				
				// Volumetrics
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 5)
				[
					SNew(SHorizontalBox)
					+ SHorizontalBox::Slot()
					.AutoWidth()
					[
						SNew(SCheckBox)
						.IsChecked(GlobalConfig.bEnableVolumetrics ? ECheckBoxState::Checked : ECheckBoxState::Unchecked)
						.OnCheckStateChanged(this, &SPatternLightingWindow::OnEnableVolumetricsChanged)
					]
					+ SHorizontalBox::Slot()
					.Padding(5, 0)
					[
						SNew(STextBlock)
						.Text(LOCTEXT("EnableVolumetrics", "Enable Volumetric Lighting"))
					]
				]
				
				// Enhanced AO
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 5)
				[
					SNew(SHorizontalBox)
					+ SHorizontalBox::Slot()
					.AutoWidth()
					[
						SNew(SCheckBox)
						.IsChecked(GlobalConfig.bEnhancedAO ? ECheckBoxState::Checked : ECheckBoxState::Unchecked)
						.OnCheckStateChanged(this, &SPatternLightingWindow::OnEnableAOChanged)
					]
					+ SHorizontalBox::Slot()
					.Padding(5, 0)
					[
						SNew(STextBlock)
						.Text(LOCTEXT("EnhancedAO", "Enhanced Ambient Occlusion"))
					]
				]
				
				// Separator
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 10)
				[
					SNew(SSeparator)
				]
				
				// Actions Section
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 5)
				[
					SNew(STextBlock)
					.Text(LOCTEXT("Actions", "Actions"))
					.Font(FCoreStyle::GetDefaultFontStyle("Bold", 14))
				]
				
				// Buttons
				+ SVerticalBox::Slot()
				.AutoHeight()
				.Padding(0, 5)
				[
					SNew(SHorizontalBox)
					+ SHorizontalBox::Slot()
					.Padding(0, 0, 5, 0)
					[
						SNew(SButton)
						.Text(LOCTEXT("SelectAll", "Select All Lights"))
						.OnClicked(this, &SPatternLightingWindow::OnSelectAllLights)
					]
					+ SHorizontalBox::Slot()
					.Padding(5, 0)
					[
						SNew(SButton)
						.Text(LOCTEXT("SyncAll", "Sync All Lights"))
						.OnClicked(this, &SPatternLightingWindow::OnSyncAllLights)
					]
				]
			]
		]
		
		// Footer
		+ SVerticalBox::Slot()
		.AutoHeight()
		[
			SNew(SSeparator)
		]
		
		+ SVerticalBox::Slot()
		.AutoHeight()
		.Padding(10)
		[
			SNew(SHorizontalBox)
			+ SHorizontalBox::Slot()
			.FillWidth(1.0f)
			[
				SNew(SButton)
				.Text(LOCTEXT("Apply", "Apply"))
				.OnClicked(this, &SPatternLightingWindow::OnApplySettings)
			]
			+ SHorizontalBox::Slot()
			.AutoWidth()
			.Padding(10, 0, 0, 0)
			[
				SNew(SButton)
				.Text(LOCTEXT("Reset", "Reset"))
				.OnClicked(this, &SPatternLightingWindow::OnResetSettings)
			]
		]
	];
}

void SPatternLightingWindow::OnGlobalIntensityChanged(float NewValue)
{
	GlobalConfig.GlobalIntensity = NewValue;
}

void SPatternLightingWindow::OnGlobalSpeedChanged(float NewValue)
{
	GlobalConfig.GlobalSpeed = NewValue;
}

void SPatternLightingWindow::OnEnablePBRChanged(ECheckBoxState NewState)
{
	GlobalConfig.bEnablePBR = (NewState == ECheckBoxState::Checked);
}

void SPatternLightingWindow::OnEnableSSRChanged(ECheckBoxState NewState)
{
	GlobalConfig.bEnableSSR = (NewState == ECheckBoxState::Checked);
}

void SPatternLightingWindow::OnEnableVolumetricsChanged(ECheckBoxState NewState)
{
	GlobalConfig.bEnableVolumetrics = (NewState == ECheckBoxState::Checked);
}

void SPatternLightingWindow::OnEnableAOChanged(ECheckBoxState NewState)
{
	GlobalConfig.bEnhancedAO = (NewState == ECheckBoxState::Checked);
}

FReply SPatternLightingWindow::OnApplySettings()
{
	ApplyToSubsystem();
	return FReply::Handled();
}

FReply SPatternLightingWindow::OnResetSettings()
{
	GlobalConfig = FPatternLightingConfig();
	// Refresh UI would go here
	return FReply::Handled();
}

FReply SPatternLightingWindow::OnSelectAllLights()
{
	// Select all pattern lights in editor
	// Implementation would use GEditor->SelectNone() then iterate and select
	return FReply::Handled();
}

FReply SPatternLightingWindow::OnSyncAllLights()
{
	UWorld* World = GEditor->GetEditorWorldContext().World();
	if (World)
	{
		if (UPatternLightingSubsystem* Subsystem = World->GetSubsystem<UPatternLightingSubsystem>())
		{
			// Sync all lights in all groups
			// Would iterate sync groups here
		}
	}
	return FReply::Handled();
}

void SPatternLightingWindow::RefreshSettings()
{
	UWorld* World = GEditor->GetEditorWorldContext().World();
	if (World)
	{
		if (UPatternLightingSubsystem* Subsystem = World->GetSubsystem<UPatternLightingSubsystem>())
		{
			GlobalConfig = Subsystem->GlobalConfig;
		}
	}
}

void SPatternLightingWindow::ApplyToSubsystem()
{
	UWorld* World = GEditor->GetEditorWorldContext().World();
	if (World)
	{
		if (UPatternLightingSubsystem* Subsystem = World->GetSubsystem<UPatternLightingSubsystem>())
		{
			Subsystem->GlobalConfig = GlobalConfig;
		}
	}
}

#undef LOCTEXT_NAMESPACE
