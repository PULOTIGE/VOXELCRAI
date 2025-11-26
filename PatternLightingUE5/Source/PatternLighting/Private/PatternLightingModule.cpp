// Pattern Lighting System for UE5
// Module implementation

#include "PatternLightingModule.h"

#define LOCTEXT_NAMESPACE "FPatternLightingModule"

void FPatternLightingModule::StartupModule()
{
	UE_LOG(LogTemp, Log, TEXT("Pattern Lighting System initialized"));
	UE_LOG(LogTemp, Log, TEXT("Based on Adaptive Entity Engine"));
}

void FPatternLightingModule::ShutdownModule()
{
	UE_LOG(LogTemp, Log, TEXT("Pattern Lighting System shutdown"));
}

#undef LOCTEXT_NAMESPACE
	
IMPLEMENT_MODULE(FPatternLightingModule, PatternLighting)
