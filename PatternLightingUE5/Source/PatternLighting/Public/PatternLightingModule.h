// Pattern Lighting System for UE5
// Module header

#pragma once

#include "CoreMinimal.h"
#include "Modules/ModuleManager.h"

class FPatternLightingModule : public IModuleInterface
{
public:
	/** IModuleInterface implementation */
	virtual void StartupModule() override;
	virtual void ShutdownModule() override;

	/**
	 * Singleton-like access to this module's interface.
	 */
	static inline FPatternLightingModule& Get()
	{
		return FModuleManager::LoadModuleChecked<FPatternLightingModule>("PatternLighting");
	}

	/**
	 * Checks if module is loaded
	 */
	static inline bool IsAvailable()
	{
		return FModuleManager::Get().IsModuleLoaded("PatternLighting");
	}
};
