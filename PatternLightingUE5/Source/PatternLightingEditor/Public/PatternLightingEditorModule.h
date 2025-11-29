// Pattern Lighting System for UE5
// Editor Module Header

#pragma once

#include "CoreMinimal.h"
#include "Modules/ModuleManager.h"

class FPatternLightingEditorModule : public IModuleInterface
{
public:
	/** IModuleInterface implementation */
	virtual void StartupModule() override;
	virtual void ShutdownModule() override;

private:
	void RegisterMenuExtensions();
	void UnregisterMenuExtensions();
	
	void OnOpenPatternLightingWindow();
	void OnOpenPatternPreview();
	
	TSharedRef<class SDockTab> SpawnPatternLightingTab(const class FSpawnTabArgs& SpawnTabArgs);
	TSharedRef<class SDockTab> SpawnPatternPreviewTab(const class FSpawnTabArgs& SpawnTabArgs);
	
	TSharedPtr<class FUICommandList> PluginCommands;
};
