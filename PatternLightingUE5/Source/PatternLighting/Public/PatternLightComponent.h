// Pattern Lighting System for UE5
// Pattern Light Component

#pragma once

#include "CoreMinimal.h"
#include "Components/LightComponent.h"
#include "PatternTypes.h"
#include "PatternLightComponent.generated.h"

/**
 * Pattern Light Component
 * Extends standard UE5 lighting with pattern-based animations
 */
UCLASS(ClassGroup=(PatternLighting), meta=(BlueprintSpawnableComponent), HideCategories=(Mobility))
class PATTERNLIGHTING_API UPatternLightComponent : public ULocalLightComponent
{
	GENERATED_BODY()

public:
	UPatternLightComponent();

	// Pattern settings
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Lighting")
	FPatternLightSettings PatternSettings;

	// Base light color
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Lighting")
	FLinearColor BaseColor = FLinearColor::White;

	// Base intensity
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Lighting", meta = (ClampMin = "0.0"))
	float BaseIntensity = 5000.0f;

	// Light radius
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Lighting", meta = (ClampMin = "1.0"))
	float LightRadius = 1000.0f;

	// Enable shadows
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Lighting|Shadows")
	bool bCastPatternShadows = true;

	// Shadow settings
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Lighting|Shadows", meta = (EditCondition = "bCastPatternShadows"))
	FPatternShadowSettings ShadowSettings;

	// Sync group (lights in same group sync their patterns)
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern Lighting|Sync")
	FName SyncGroup = NAME_None;

protected:
	virtual void BeginPlay() override;
	virtual void TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction) override;

public:
	/**
	 * Calculate pattern value at current time
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	float CalculatePatternValue(float Time) const;

	/**
	 * Get current light intensity (with pattern applied)
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	float GetCurrentIntensity() const;

	/**
	 * Get current light color (with pattern applied)
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	FLinearColor GetCurrentColor() const;

	/**
	 * Set pattern at runtime
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void SetPattern(ELightPattern NewPattern);

	/**
	 * Set pattern speed
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void SetPatternSpeed(float NewSpeed);

	/**
	 * Trigger a flash effect
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void TriggerFlash(float Duration = 0.1f, float FlashIntensity = 10.0f);

	/**
	 * Sync with another pattern light
	 */
	UFUNCTION(BlueprintCallable, Category = "Pattern Lighting")
	void SyncWith(UPatternLightComponent* Other);

private:
	float CurrentTime = 0.0f;
	float FlashTimer = 0.0f;
	float FlashIntensityMultiplier = 1.0f;
	
	void UpdateLightProperties();
	float EvaluatePattern(float T) const;
};

/**
 * Pattern Point Light Actor
 */
UCLASS(Blueprintable, ClassGroup=(PatternLighting))
class PATTERNLIGHTING_API APatternPointLight : public AActor
{
	GENERATED_BODY()

public:
	APatternPointLight();

	UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
	UPatternLightComponent* LightComponent;
};

/**
 * Pattern Spot Light Actor
 */
UCLASS(Blueprintable, ClassGroup=(PatternLighting))
class PATTERNLIGHTING_API APatternSpotLight : public AActor
{
	GENERATED_BODY()

public:
	APatternSpotLight();

	UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category = "Components")
	UPatternLightComponent* LightComponent;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Spot Light")
	float InnerConeAngle = 25.0f;

	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Spot Light")
	float OuterConeAngle = 45.0f;
};
