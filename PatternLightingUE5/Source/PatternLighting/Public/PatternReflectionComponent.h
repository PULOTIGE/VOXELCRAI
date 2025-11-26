// Pattern Lighting System for UE5
// Pattern Reflection Component

#pragma once

#include "CoreMinimal.h"
#include "Components/SceneComponent.h"
#include "PatternTypes.h"
#include "PatternReflectionComponent.generated.h"

/**
 * Pattern Reflection Component
 * Enhanced reflection capture with SSR and dynamic updates
 */
UCLASS(ClassGroup=(PatternLighting), meta=(BlueprintSpawnableComponent))
class PATTERNLIGHTING_API UPatternReflectionComponent : public USceneComponent
{
	GENERATED_BODY()

public:
	UPatternReflectionComponent();

	// Reflection settings
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Reflection")
	FPatternReflectionSettings ReflectionSettings;

	// Capture cubemap resolution
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Reflection", meta = (ClampMin = "64", ClampMax = "2048"))
	int32 CubemapResolution = 256;

	// Update frequency (0 = static, >0 = seconds between updates)
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Reflection", meta = (ClampMin = "0.0"))
	float UpdateFrequency = 0.0f;

	// Blend with scene reflections
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Reflection", meta = (ClampMin = "0.0", ClampMax = "1.0"))
	float BlendWeight = 1.0f;

	// Box projection for indoor scenes
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Reflection")
	bool bUseBoxProjection = false;

	// Box projection bounds
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Reflection", meta = (EditCondition = "bUseBoxProjection"))
	FVector BoxExtent = FVector(1000.0f);

protected:
	virtual void BeginPlay() override;
	virtual void TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction) override;

public:
	/**
	 * Force capture update
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Reflection")
	void UpdateCapture();

	/**
	 * Get reflection intensity at world location
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Reflection")
	float GetReflectionIntensityAt(FVector WorldLocation) const;

	/**
	 * Calculate Fresnel term
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Reflection")
	static float CalculateFresnel(FVector ViewDirection, FVector SurfaceNormal, float Exponent = 5.0f);

private:
	float TimeSinceLastUpdate = 0.0f;
	
	void CaptureReflection();
};

/**
 * SSR Post Process Volume
 * Enables screen-space reflections in a volume
 */
UCLASS(Blueprintable, ClassGroup=(PatternLighting))
class PATTERNLIGHTING_API APatternSSRVolume : public AActor
{
	GENERATED_BODY()

public:
	APatternSSRVolume();

	UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
	class UBoxComponent* VolumeBox;

	// SSR Settings
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SSR")
	bool bEnabled = true;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SSR", meta = (ClampMin = "0.0", ClampMax = "1.0"))
	float Intensity = 1.0f;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SSR", meta = (ClampMin = "16", ClampMax = "256"))
	int32 MaxSteps = 64;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SSR", meta = (ClampMin = "100.0", ClampMax = "10000.0"))
	float MaxDistance = 1000.0f;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SSR", meta = (ClampMin = "0.1", ClampMax = "10.0"))
	float Thickness = 1.0f;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SSR", meta = (ClampMin = "0.0", ClampMax = "1.0"))
	float EdgeFade = 0.9f;

	// Blend priority (higher = preferred)
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SSR")
	float Priority = 0.0f;

	/**
	 * Check if point is inside volume
	 */
	UFUNCTION(BlueprintCallable, Category = "SSR")
	bool IsPointInside(FVector WorldPoint) const;
};

/**
 * Planar Reflection Actor
 * For floors, water, mirrors
 */
UCLASS(Blueprintable, ClassGroup=(PatternLighting))
class PATTERNLIGHTING_API APatternPlanarReflection : public AActor
{
	GENERATED_BODY()

public:
	APatternPlanarReflection();

	UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
	UPatternReflectionComponent* ReflectionComponent;

	// Plane mesh for visualization
	UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
	class UStaticMeshComponent* PlaneMesh;

	// Reflection normal direction
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Planar Reflection")
	FVector ReflectionNormal = FVector::UpVector;

	// Distortion settings
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Planar Reflection")
	bool bEnableDistortion = false;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Planar Reflection", meta = (EditCondition = "bEnableDistortion"))
	float DistortionIntensity = 0.02f;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Planar Reflection", meta = (EditCondition = "bEnableDistortion"))
	float DistortionSpeed = 1.0f;

	/**
	 * Calculate reflected position
	 */
	UFUNCTION(BlueprintCallable, Category = "Planar Reflection")
	FVector GetReflectedPosition(FVector WorldPosition) const;
};
