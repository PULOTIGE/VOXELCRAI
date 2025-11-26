// Pattern Lighting System for UE5
// World Subsystem for global management

#pragma once

#include "CoreMinimal.h"
#include "Subsystems/WorldSubsystem.h"
#include "PatternTypes.h"
#include "PatternLightingSubsystem.generated.h"

class UPatternLightComponent;
class UPatternReflectionComponent;
class UPatternShadowComponent;

/**
 * Pattern Lighting Subsystem
 * Manages all pattern lighting in the world
 */
UCLASS()
class PATTERNLIGHTING_API UPatternLightingSubsystem : public UWorldSubsystem
{
	GENERATED_BODY()

public:
	// Global configuration
	UPROPERTY(BlueprintReadWrite, Category = "Pattern Lighting")
	FPatternLightingConfig GlobalConfig;

	// Master time (for synced patterns)
	UPROPERTY(BlueprintReadOnly, Category = "Pattern Lighting")
	float MasterTime = 0.0f;

protected:
	virtual void Initialize(FSubsystemCollectionBase& Collection) override;
	virtual void Deinitialize() override;
	virtual bool DoesSupportWorldType(EWorldType::Type WorldType) const override;

public:
	virtual void Tick(float DeltaTime);
	virtual TStatId GetStatId() const override;

	// ========================================================================
	// Registration
	// ========================================================================

	/**
	 * Register a pattern light
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void RegisterLight(UPatternLightComponent* Light);

	/**
	 * Unregister a pattern light
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void UnregisterLight(UPatternLightComponent* Light);

	/**
	 * Register a reflection probe
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void RegisterReflection(UPatternReflectionComponent* Reflection);

	/**
	 * Unregister a reflection probe
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void UnregisterReflection(UPatternReflectionComponent* Reflection);

	// ========================================================================
	// Queries
	// ========================================================================

	/**
	 * Get all pattern lights in sync group
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	TArray<UPatternLightComponent*> GetLightsInSyncGroup(FName SyncGroup) const;

	/**
	 * Get pattern lights affecting location
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	TArray<UPatternLightComponent*> GetLightsAtLocation(FVector Location, float Radius = 1000.0f) const;

	/**
	 * Get combined light intensity at location
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	float GetCombinedIntensityAt(FVector Location) const;

	/**
	 * Get combined light color at location
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	FLinearColor GetCombinedColorAt(FVector Location) const;

	/**
	 * Get best reflection probe for location
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	UPatternReflectionComponent* GetBestReflectionAt(FVector Location) const;

	// ========================================================================
	// Global Control
	// ========================================================================

	/**
	 * Set global intensity multiplier
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void SetGlobalIntensity(float Intensity);

	/**
	 * Set global speed multiplier
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void SetGlobalSpeed(float Speed);

	/**
	 * Pause all pattern animations
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void PauseAll();

	/**
	 * Resume all pattern animations
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void ResumeAll();

	/**
	 * Trigger flash on all lights in radius
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void TriggerFlashAtLocation(FVector Location, float Radius, float Duration = 0.1f, float Intensity = 10.0f);

	/**
	 * Sync all lights in a group
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void SyncGroup(FName GroupName);

	// ========================================================================
	// Debug
	// ========================================================================

	/**
	 * Draw debug visualization
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting|Debug")
	void DrawDebug(float Duration = 0.0f);

	/**
	 * Get stats string
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting|Debug")
	FString GetStatsString() const;

private:
	TArray<TWeakObjectPtr<UPatternLightComponent>> RegisteredLights;
	TArray<TWeakObjectPtr<UPatternReflectionComponent>> RegisteredReflections;
	TMap<FName, TArray<TWeakObjectPtr<UPatternLightComponent>>> SyncGroups;
	
	bool bPaused = false;
	
	void CleanupStaleReferences();
};

/**
 * Blueprint Function Library
 */
UCLASS()
class PATTERNLIGHTING_API UPatternLightingBlueprintLibrary : public UBlueprintFunctionLibrary
{
	GENERATED_BODY()

public:
	/**
	 * Get pattern lighting subsystem
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting", meta = (WorldContext = "WorldContextObject"))
	static UPatternLightingSubsystem* GetPatternLightingSubsystem(UObject* WorldContextObject);

	/**
	 * Create pattern light at location
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting", meta = (WorldContext = "WorldContextObject"))
	static APatternPointLight* SpawnPatternLight(
		UObject* WorldContextObject,
		FVector Location,
		ELightPattern Pattern = ELightPattern::Steady,
		FLinearColor Color = FLinearColor::White,
		float Intensity = 5000.0f,
		float Radius = 1000.0f
	);

	/**
	 * Evaluate pattern value
	 */
	UFUNCTION(BlueprintCallable, BlueprintPure, Category = "Pattern Lighting")
	static float EvaluatePattern(ELightPattern Pattern, float Time, float Speed = 1.0f);

	/**
	 * Calculate Fresnel reflection
	 */
	UFUNCTION(BlueprintCallable, BlueprintPure, Category = "Pattern Lighting")
	static float CalculateFresnelReflection(FVector ViewDir, FVector Normal, float IOR = 1.5f);

	/**
	 * Calculate PBR specular
	 */
	UFUNCTION(BlueprintCallable, BlueprintPure, Category = "Pattern Lighting")
	static FLinearColor CalculatePBRSpecular(
		FVector Normal,
		FVector ViewDir,
		FVector LightDir,
		FLinearColor LightColor,
		float Roughness,
		float Metallic
	);

	/**
	 * Get shadow softness from distance
	 */
	UFUNCTION(BlueprintCallable, BlueprintPure, Category = "Pattern Lighting")
	static float GetShadowSoftness(float LightRadius, float Distance);

	/**
	 * Lerp between two patterns
	 */
	UFUNCTION(BlueprintCallable, BlueprintPure, Category = "Pattern Lighting")
	static float LerpPatterns(ELightPattern PatternA, ELightPattern PatternB, float Time, float Alpha);

	/**
	 * Convert color temperature to RGB
	 */
	UFUNCTION(BlueprintCallable, BlueprintPure, Category = "Pattern Lighting")
	static FLinearColor ColorTemperatureToRGB(float Kelvin);

	/**
	 * Get recommended light intensity for lux
	 */
	UFUNCTION(BlueprintCallable, BlueprintPure, Category = "Pattern Lighting")
	static float LuxToIntensity(float Lux);
};
