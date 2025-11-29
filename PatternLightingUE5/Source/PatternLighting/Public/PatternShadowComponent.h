// Pattern Lighting System for UE5
// Pattern Shadow Component

#pragma once

#include "CoreMinimal.h"
#include "Components/SceneComponent.h"
#include "PatternTypes.h"
#include "PatternShadowComponent.generated.h"

/**
 * Pattern Shadow Component
 * Enhanced shadow control with cascades and contact shadows
 */
UCLASS(ClassGroup=(PatternLighting), meta=(BlueprintSpawnableComponent))
class PATTERNLIGHTING_API UPatternShadowComponent : public USceneComponent
{
	GENERATED_BODY()

public:
	UPatternShadowComponent();

	// Shadow settings
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Shadow")
	FPatternShadowSettings ShadowSettings;

	// Cascade shadow distances
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Shadow|Cascades")
	TArray<float> CascadeDistances;

	// Dynamic shadow distance
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Shadow", meta = (ClampMin = "1000.0"))
	float DynamicShadowDistance = 20000.0f;

	// Enable shadow caching
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Shadow")
	bool bEnableShadowCaching = true;

	// Shadow color tint
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Shadow")
	FLinearColor ShadowColor = FLinearColor(0.0f, 0.0f, 0.1f, 1.0f);

	// Volumetric shadow settings
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Shadow|Volumetric")
	bool bVolumetricShadows = false;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Shadow|Volumetric", meta = (EditCondition = "bVolumetricShadows"))
	int32 VolumetricShadowSamples = 16;

protected:
	virtual void BeginPlay() override;
	virtual void TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction) override;

public:
	/**
	 * Calculate cascade splits based on camera
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Shadow")
	TArray<float> CalculateCascadeSplits(float NearPlane, float FarPlane) const;

	/**
	 * Get shadow intensity at world location
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Shadow")
	float GetShadowIntensityAt(FVector WorldLocation) const;

	/**
	 * Apply shadow settings to directional light
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Shadow")
	void ApplyToDirectionalLight(class UDirectionalLightComponent* Light);

	/**
	 * Calculate soft shadow penumbra
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Shadow")
	static float CalculatePenumbra(float LightRadius, float OccluderDistance, float ReceiverDistance);

private:
	void UpdateCascades();
	void CalculateContactShadows();
};

/**
 * Directional Light with Pattern Shadows
 */
UCLASS(Blueprintable, ClassGroup=(PatternLighting))
class PATTERNLIGHTING_API APatternDirectionalLight : public AActor
{
	GENERATED_BODY()

public:
	APatternDirectionalLight();

	UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
	class UDirectionalLightComponent* DirectionalLight;

	UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
	UPatternShadowComponent* ShadowComponent;

	UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
	UPatternLightComponent* PatternComponent;

	// Sun/Moon settings
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Sky")
	bool bAutoRotate = false;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Sky", meta = (EditCondition = "bAutoRotate"))
	float DayLength = 1200.0f; // Seconds for full day cycle

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Sky", meta = (EditCondition = "bAutoRotate"))
	float CurrentTimeOfDay = 0.5f; // 0-1 representing 24 hours

	/**
	 * Set time of day (0-1)
	 */
	UFUNCTION(BlueprintCallable, Category = "Sky")
	void SetTimeOfDay(float NormalizedTime);

	/**
	 * Get sun direction for given time
	 */
	UFUNCTION(BlueprintCallable, Category = "Sky")
	FVector GetSunDirection(float NormalizedTime) const;

protected:
	virtual void Tick(float DeltaTime) override;
};

/**
 * Shadow Caster Volume
 * Forces objects inside to cast shadows
 */
UCLASS(Blueprintable, ClassGroup=(PatternLighting))
class PATTERNLIGHTING_API APatternShadowCasterVolume : public AActor
{
	GENERATED_BODY()

public:
	APatternShadowCasterVolume();

	UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
	class UBoxComponent* VolumeBox;

	// Force shadow settings on objects inside
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Shadow Caster")
	bool bForceDynamicShadows = true;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Shadow Caster")
	bool bForceContactShadows = true;

	// Shadow distance override
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Shadow Caster", meta = (ClampMin = "0.0"))
	float ShadowDistanceOverride = 0.0f;

	/**
	 * Apply shadow settings to actors in volume
	 */
	UFUNCTION(BlueprintCallable, Category = "Shadow Caster")
	void ApplyToActorsInVolume();

protected:
	virtual void BeginPlay() override;
};
