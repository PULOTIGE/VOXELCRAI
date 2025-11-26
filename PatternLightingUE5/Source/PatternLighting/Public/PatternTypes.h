// Pattern Lighting System for UE5
// Common types and enums

#pragma once

#include "CoreMinimal.h"
#include "PatternTypes.generated.h"

/**
 * Light pattern types
 */
UENUM(BlueprintType)
enum class ELightPattern : uint8
{
	Steady			UMETA(DisplayName = "Steady"),
	Pulse			UMETA(DisplayName = "Pulse"),
	Flicker			UMETA(DisplayName = "Flicker"),
	Strobe			UMETA(DisplayName = "Strobe"),
	Candle			UMETA(DisplayName = "Candle"),
	Fluorescent		UMETA(DisplayName = "Fluorescent"),
	Lightning		UMETA(DisplayName = "Lightning"),
	Fire			UMETA(DisplayName = "Fire"),
	Alarm			UMETA(DisplayName = "Alarm"),
	Underwater		UMETA(DisplayName = "Underwater Caustics"),
	Heartbeat		UMETA(DisplayName = "Heartbeat"),
	Breathing		UMETA(DisplayName = "Breathing"),
	Custom			UMETA(DisplayName = "Custom Curve")
};

/**
 * Reflection quality levels
 */
UENUM(BlueprintType)
enum class EReflectionQuality : uint8
{
	Low				UMETA(DisplayName = "Low (Cubemap Only)"),
	Medium			UMETA(DisplayName = "Medium (Planar)"),
	High			UMETA(DisplayName = "High (SSR)"),
	Ultra			UMETA(DisplayName = "Ultra (Ray Traced)")
};

/**
 * Shadow quality levels
 */
UENUM(BlueprintType)
enum class EShadowQuality : uint8
{
	Low				UMETA(DisplayName = "Low"),
	Medium			UMETA(DisplayName = "Medium"),
	High			UMETA(DisplayName = "High (Soft Shadows)"),
	Ultra			UMETA(DisplayName = "Ultra (Contact Shadows)")
};

/**
 * Pattern lighting settings structure
 */
USTRUCT(BlueprintType)
struct PATTERNLIGHTING_API FPatternLightSettings
{
	GENERATED_BODY()

	/** Pattern type */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern")
	ELightPattern Pattern = ELightPattern::Steady;

	/** Pattern animation speed multiplier */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern", meta = (ClampMin = "0.01", ClampMax = "10.0"))
	float Speed = 1.0f;

	/** Pattern phase offset (0-1) */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern", meta = (ClampMin = "0.0", ClampMax = "1.0"))
	float PhaseOffset = 0.0f;

	/** Minimum intensity */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern", meta = (ClampMin = "0.0", ClampMax = "1.0"))
	float MinIntensity = 0.0f;

	/** Maximum intensity */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern", meta = (ClampMin = "0.0", ClampMax = "10.0"))
	float MaxIntensity = 1.0f;

	/** Custom animation curve (used when Pattern is Custom) */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Pattern")
	UCurveFloat* CustomCurve = nullptr;

	/** Enable color shifting */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Color")
	bool bEnableColorShift = false;

	/** Color shift gradient */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Color", meta = (EditCondition = "bEnableColorShift"))
	UCurveLinearColor* ColorCurve = nullptr;
};

/**
 * Reflection probe settings
 */
USTRUCT(BlueprintType)
struct PATTERNLIGHTING_API FPatternReflectionSettings
{
	GENERATED_BODY()

	/** Reflection quality level */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Reflection")
	EReflectionQuality Quality = EReflectionQuality::High;

	/** Reflection intensity */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Reflection", meta = (ClampMin = "0.0", ClampMax = "2.0"))
	float Intensity = 1.0f;

	/** Influence radius */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Reflection", meta = (ClampMin = "1.0"))
	float Radius = 1000.0f;

	/** Fresnel exponent */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Reflection", meta = (ClampMin = "1.0", ClampMax = "10.0"))
	float FresnelExponent = 5.0f;

	/** Enable roughness blur */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Reflection")
	bool bRoughnessBlur = true;

	/** SSR max distance (High/Ultra quality) */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SSR", meta = (ClampMin = "100.0", ClampMax = "10000.0"))
	float SSRMaxDistance = 1000.0f;

	/** SSR step count */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SSR", meta = (ClampMin = "16", ClampMax = "256"))
	int32 SSRSteps = 64;

	/** SSR thickness */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "SSR", meta = (ClampMin = "0.1", ClampMax = "10.0"))
	float SSRThickness = 1.0f;
};

/**
 * Shadow settings
 */
USTRUCT(BlueprintType)
struct PATTERNLIGHTING_API FPatternShadowSettings
{
	GENERATED_BODY()

	/** Shadow quality level */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Shadow")
	EShadowQuality Quality = EShadowQuality::High;

	/** Shadow intensity (darkness) */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Shadow", meta = (ClampMin = "0.0", ClampMax = "1.0"))
	float Intensity = 1.0f;

	/** Shadow softness */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Shadow", meta = (ClampMin = "0.0", ClampMax = "10.0"))
	float Softness = 1.0f;

	/** Shadow bias */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Shadow", meta = (ClampMin = "0.0", ClampMax = "10.0"))
	float Bias = 0.5f;

	/** Enable contact shadows */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Shadow")
	bool bContactShadows = true;

	/** Contact shadow length */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Shadow", meta = (ClampMin = "0.0", ClampMax = "1.0", EditCondition = "bContactShadows"))
	float ContactShadowLength = 0.1f;

	/** Cascade shadow map count */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Cascades", meta = (ClampMin = "1", ClampMax = "8"))
	int32 CascadeCount = 4;

	/** Cascade distribution exponent */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Cascades", meta = (ClampMin = "1.0", ClampMax = "5.0"))
	float CascadeDistribution = 2.0f;
};

/**
 * Global pattern lighting configuration
 */
USTRUCT(BlueprintType)
struct PATTERNLIGHTING_API FPatternLightingConfig
{
	GENERATED_BODY()

	/** Enable pattern lighting system */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Global")
	bool bEnabled = true;

	/** Global intensity multiplier */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Global", meta = (ClampMin = "0.0", ClampMax = "2.0"))
	float GlobalIntensity = 1.0f;

	/** Global speed multiplier */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Global", meta = (ClampMin = "0.1", ClampMax = "5.0"))
	float GlobalSpeed = 1.0f;

	/** Enable PBR lighting */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "PBR")
	bool bEnablePBR = true;

	/** Enable screen-space reflections */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Reflections")
	bool bEnableSSR = true;

	/** Enable volumetric lighting */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Volumetrics")
	bool bEnableVolumetrics = false;

	/** Volumetric density */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Volumetrics", meta = (ClampMin = "0.0", ClampMax = "1.0", EditCondition = "bEnableVolumetrics"))
	float VolumetricDensity = 0.1f;

	/** Enable ambient occlusion enhancement */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AO")
	bool bEnhancedAO = true;

	/** AO intensity */
	UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "AO", meta = (ClampMin = "0.0", ClampMax = "2.0", EditCondition = "bEnhancedAO"))
	float AOIntensity = 1.0f;
};
