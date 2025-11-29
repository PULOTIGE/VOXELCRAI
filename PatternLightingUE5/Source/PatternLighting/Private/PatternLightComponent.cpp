// Pattern Lighting System for UE5
// Pattern Light Component Implementation

#include "PatternLightComponent.h"
#include "Curves/CurveFloat.h"
#include "Curves/CurveLinearColor.h"

UPatternLightComponent::UPatternLightComponent()
{
	PrimaryComponentTick.bCanEverTick = true;
	PrimaryComponentTick.bStartWithTickEnabled = true;
	
	// Default light settings
	SetIntensity(BaseIntensity);
	SetAttenuationRadius(LightRadius);
	SetLightColor(BaseColor);
	CastShadows = true;
}

void UPatternLightComponent::BeginPlay()
{
	Super::BeginPlay();
	
	// Initialize with random phase offset if not synced
	if (SyncGroup == NAME_None && PatternSettings.PhaseOffset == 0.0f)
	{
		PatternSettings.PhaseOffset = FMath::FRand();
	}
	
	CurrentTime = PatternSettings.PhaseOffset;
}

void UPatternLightComponent::TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction)
{
	Super::TickComponent(DeltaTime, TickType, ThisTickFunction);
	
	// Update time
	CurrentTime += DeltaTime * PatternSettings.Speed;
	
	// Update flash timer
	if (FlashTimer > 0.0f)
	{
		FlashTimer -= DeltaTime;
		if (FlashTimer <= 0.0f)
		{
			FlashIntensityMultiplier = 1.0f;
		}
	}
	
	// Update light properties
	UpdateLightProperties();
}

float UPatternLightComponent::CalculatePatternValue(float Time) const
{
	return EvaluatePattern(Time);
}

float UPatternLightComponent::EvaluatePattern(float T) const
{
	float Value = 1.0f;
	
	switch (PatternSettings.Pattern)
	{
		case ELightPattern::Steady:
			Value = 1.0f;
			break;
			
		case ELightPattern::Pulse:
			Value = 0.5f + 0.5f * FMath::Sin(T * 2.0f * PI);
			break;
			
		case ELightPattern::Flicker:
			Value = 0.7f + 0.3f * FMath::Sin(T * 20.0f) * FMath::Sin(T * 7.3f);
			break;
			
		case ELightPattern::Strobe:
			Value = FMath::Sin(T * 10.0f) > 0.0f ? 1.0f : 0.0f;
			break;
			
		case ELightPattern::Candle:
			{
				float Flicker = FMath::Sin(T * 12.0f) * FMath::Sin(T * 5.7f) * FMath::Sin(T * 3.1f);
				Value = 0.8f + 0.2f * Flicker;
			}
			break;
			
		case ELightPattern::Fluorescent:
			{
				float Startup = FMath::Clamp(FMath::Fmod(T, 5.0f) / 2.0f, 0.0f, 1.0f);
				float Buzz = 0.05f * FMath::Sin(T * 120.0f);
				Value = Startup + Buzz * Startup;
			}
			break;
			
		case ELightPattern::Lightning:
			Value = FMath::Pow(FMath::Max(0.0f, FMath::Sin(T * 0.5f)), 20.0f);
			break;
			
		case ELightPattern::Fire:
			Value = 0.7f + 0.3f * FMath::Sin(T * 8.0f) * FMath::Sin(T * 4.3f) * FMath::Sin(T * 2.1f);
			break;
			
		case ELightPattern::Alarm:
			Value = FMath::Sin(T * 4.0f) > 0.0f ? 1.0f : 0.2f;
			break;
			
		case ELightPattern::Underwater:
			{
				FVector Location = GetComponentLocation();
				float Caustic = FMath::Sin(Location.X * 0.01f + T) * FMath::Sin(Location.Y * 0.01f + T * 0.7f);
				Value = 0.7f + 0.3f * Caustic;
			}
			break;
			
		case ELightPattern::Heartbeat:
			{
				float Beat = FMath::Pow(FMath::Sin(T * 2.5f), 12.0f);
				float Beat2 = FMath::Pow(FMath::Sin(T * 2.5f + 0.3f), 12.0f) * 0.5f;
				Value = FMath::Max(Beat, Beat2);
			}
			break;
			
		case ELightPattern::Breathing:
			Value = 0.3f + 0.7f * (FMath::Sin(T * 0.5f) * 0.5f + 0.5f);
			break;
			
		case ELightPattern::Custom:
			if (PatternSettings.CustomCurve)
			{
				float CurveTime = FMath::Fmod(T, PatternSettings.CustomCurve->GetTimeRange().Y);
				Value = PatternSettings.CustomCurve->GetFloatValue(CurveTime);
			}
			break;
	}
	
	// Map to min/max intensity range
	Value = FMath::Lerp(PatternSettings.MinIntensity, PatternSettings.MaxIntensity, Value);
	
	return Value;
}

float UPatternLightComponent::GetCurrentIntensity() const
{
	float PatternValue = EvaluatePattern(CurrentTime);
	return BaseIntensity * PatternValue * FlashIntensityMultiplier;
}

FLinearColor UPatternLightComponent::GetCurrentColor() const
{
	FLinearColor CurrentColor = BaseColor;
	
	if (PatternSettings.bEnableColorShift && PatternSettings.ColorCurve)
	{
		float CurveTime = FMath::Fmod(CurrentTime, PatternSettings.ColorCurve->GetTimeRange().Y);
		CurrentColor = PatternSettings.ColorCurve->GetLinearColorValue(CurveTime);
	}
	
	return CurrentColor;
}

void UPatternLightComponent::SetPattern(ELightPattern NewPattern)
{
	PatternSettings.Pattern = NewPattern;
}

void UPatternLightComponent::SetPatternSpeed(float NewSpeed)
{
	PatternSettings.Speed = FMath::Clamp(NewSpeed, 0.01f, 10.0f);
}

void UPatternLightComponent::TriggerFlash(float Duration, float FlashIntensity)
{
	FlashTimer = Duration;
	FlashIntensityMultiplier = FlashIntensity;
}

void UPatternLightComponent::SyncWith(UPatternLightComponent* Other)
{
	if (Other)
	{
		CurrentTime = Other->CurrentTime;
		PatternSettings.PhaseOffset = Other->PatternSettings.PhaseOffset;
	}
}

void UPatternLightComponent::UpdateLightProperties()
{
	// Update intensity
	float NewIntensity = GetCurrentIntensity();
	SetIntensity(NewIntensity);
	
	// Update color
	FLinearColor NewColor = GetCurrentColor();
	SetLightColor(NewColor);
	
	// Update shadow settings
	if (bCastPatternShadows)
	{
		CastShadows = true;
		// Additional shadow properties would be set here
	}
}

// ============================================================================
// Pattern Point Light Actor
// ============================================================================

APatternPointLight::APatternPointLight()
{
	LightComponent = CreateDefaultSubobject<UPatternLightComponent>(TEXT("LightComponent"));
	RootComponent = LightComponent;
}

// ============================================================================
// Pattern Spot Light Actor
// ============================================================================

APatternSpotLight::APatternSpotLight()
{
	LightComponent = CreateDefaultSubobject<UPatternLightComponent>(TEXT("LightComponent"));
	RootComponent = LightComponent;
}
