// Pattern Lighting System for UE5
// Editor Module Implementation

#include "PatternLightingEditorModule.h"
#include "PatternLightingEditorStyle.h"
#include "PatternLightingEditorCommands.h"
#include "SPatternLightingWindow.h"
#include "SPatternPreviewWidget.h"

#include "LevelEditor.h"
#include "ToolMenus.h"
#include "Widgets/Docking/SDockTab.h"
#include "Widgets/Layout/SBox.h"
#include "Widgets/Text/STextBlock.h"
#include "Framework/MultiBox/MultiBoxBuilder.h"

static const FName PatternLightingTabName("PatternLighting");
static const FName PatternPreviewTabName("PatternPreview");

#define LOCTEXT_NAMESPACE "FPatternLightingEditorModule"

void FPatternLightingEditorModule::StartupModule()
{
	// Register style
	FPatternLightingEditorStyle::Initialize();
	FPatternLightingEditorStyle::ReloadTextures();
	
	// Register commands
	FPatternLightingEditorCommands::Register();
	
	PluginCommands = MakeShareable(new FUICommandList);
	
	PluginCommands->MapAction(
		FPatternLightingEditorCommands::Get().OpenPluginWindow,
		FExecuteAction::CreateRaw(this, &FPatternLightingEditorModule::OnOpenPatternLightingWindow),
		FCanExecuteAction()
	);
	
	PluginCommands->MapAction(
		FPatternLightingEditorCommands::Get().OpenPatternPreview,
		FExecuteAction::CreateRaw(this, &FPatternLightingEditorModule::OnOpenPatternPreview),
		FCanExecuteAction()
	);
	
	// Register tabs
	FGlobalTabmanager::Get()->RegisterNomadTabSpawner(PatternLightingTabName, 
		FOnSpawnTab::CreateRaw(this, &FPatternLightingEditorModule::SpawnPatternLightingTab))
		.SetDisplayName(LOCTEXT("PatternLightingTabTitle", "Pattern Lighting"))
		.SetMenuType(ETabSpawnerMenuType::Hidden);
		
	FGlobalTabmanager::Get()->RegisterNomadTabSpawner(PatternPreviewTabName,
		FOnSpawnTab::CreateRaw(this, &FPatternLightingEditorModule::SpawnPatternPreviewTab))
		.SetDisplayName(LOCTEXT("PatternPreviewTabTitle", "Pattern Preview"))
		.SetMenuType(ETabSpawnerMenuType::Hidden);
	
	// Register menu extensions
	RegisterMenuExtensions();
	
	UE_LOG(LogTemp, Log, TEXT("Pattern Lighting Editor Module initialized"));
}

void FPatternLightingEditorModule::ShutdownModule()
{
	// Unregister menu extensions
	UnregisterMenuExtensions();
	
	// Unregister tabs
	FGlobalTabmanager::Get()->UnregisterNomadTabSpawner(PatternLightingTabName);
	FGlobalTabmanager::Get()->UnregisterNomadTabSpawner(PatternPreviewTabName);
	
	// Unregister commands
	FPatternLightingEditorCommands::Unregister();
	
	// Shutdown style
	FPatternLightingEditorStyle::Shutdown();
}

void FPatternLightingEditorModule::RegisterMenuExtensions()
{
	UToolMenus::RegisterStartupCallback(FSimpleMulticastDelegate::FDelegate::CreateLambda([this]()
	{
		UToolMenu* Menu = UToolMenus::Get()->ExtendMenu("LevelEditor.MainMenu.Window");
		{
			FToolMenuSection& Section = Menu->FindOrAddSection("WindowGlobalTabSpawners");
			Section.AddMenuEntryWithCommandList(
				FPatternLightingEditorCommands::Get().OpenPluginWindow,
				PluginCommands,
				LOCTEXT("PatternLightingMenuEntry", "Pattern Lighting"),
				LOCTEXT("PatternLightingMenuTooltip", "Open Pattern Lighting settings window")
			);
		}
	}));
}

void FPatternLightingEditorModule::UnregisterMenuExtensions()
{
	UToolMenus::UnRegisterStartupCallback(this);
	UToolMenus::UnregisterOwner(this);
}

void FPatternLightingEditorModule::OnOpenPatternLightingWindow()
{
	FGlobalTabmanager::Get()->TryInvokeTab(PatternLightingTabName);
}

void FPatternLightingEditorModule::OnOpenPatternPreview()
{
	FGlobalTabmanager::Get()->TryInvokeTab(PatternPreviewTabName);
}

TSharedRef<SDockTab> FPatternLightingEditorModule::SpawnPatternLightingTab(const FSpawnTabArgs& SpawnTabArgs)
{
	return SNew(SDockTab)
		.TabRole(ETabRole::NomadTab)
		[
			SNew(SPatternLightingWindow)
		];
}

TSharedRef<SDockTab> FPatternLightingEditorModule::SpawnPatternPreviewTab(const FSpawnTabArgs& SpawnTabArgs)
{
	return SNew(SDockTab)
		.TabRole(ETabRole::NomadTab)
		[
			SNew(SPatternPreviewWidget)
		];
}

#undef LOCTEXT_NAMESPACE

IMPLEMENT_MODULE(FPatternLightingEditorModule, PatternLightingEditor)
