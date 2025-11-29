// Pattern Lighting System for UE5
// Subsystem Implementation

#include "PatternLightingSubsystem.h"
#include "PatternLightComponent.h"
#include "PatternReflectionComponent.h"
#include "Engine/World.h"
#include "DrawDebugHelpers.h"
#include "Kismet/GameplayStatics.h"

void UPatternLightingSubsystem::Initialize(FSubsystemCollectionBase& Collection)
{
	Super::Initialize(Collection);
	UE_LOG(LogTemp, Log, TEXT("Pattern Lighting Subsystem initialized"));
}

void UPatternLightingSubsystem::Deinitialize()
{
	RegisteredLights.Empty();
	RegisteredReflections.Empty();
	SyncGroups.Empty();
	Super::Deinitialize();
}

bool UPatternLightingSubsystem::DoesSupportWorldType(EWorldType::Type WorldType) const
{
	return WorldType == EWorldType::Game || WorldType == EWorldType::PIE || WorldType == EWorldType::Editor;
}

void UPatternLightingSubsystem::Tick(float DeltaTime)
{
	if (!GlobalConfig.bEnabled || bPaused)
	{
		return;
	}

	// Update master time
	MasterTime += DeltaTime * GlobalConfig.GlobalSpeed;

	// Cleanup stale references periodically
	static float CleanupTimer = 0.0f;
	CleanupTimer += DeltaTime;
	if (CleanupTimer > 5.0f)
	{
		CleanupStaleReferences();
		CleanupTimer = 0.0f;
	}
}

TStatId UPatternLightingSubsystem::GetStatId() const
{
	RETURN_QUICK_DECLARE_CYCLE_STAT(UPatternLightingSubsystem, STATGROUP_Tickables);
}

void UPatternLightingSubsystem::RegisterLight(UPatternLightComponent* Light)
{
	if (Light && !RegisteredLights.Contains(Light))
	{
		RegisteredLights.Add(Light);
		
		// Add to sync group if specified
		if (Light->SyncGroup != NAME_None)
		{
			SyncGroups.FindOrAdd(Light->SyncGroup).Add(Light);
		}
	}
}

void UPatternLightingSubsystem::UnregisterLight(UPatternLightComponent* Light)
{
	RegisteredLights.Remove(Light);
	
	if (Light && Light->SyncGroup != NAME_None)
	{
		if (auto* Group = SyncGroups.Find(Light->SyncGroup))
		{
			Group->Remove(Light);
		}
	}
}

void UPatternLightingSubsystem::RegisterReflection(UPatternReflectionComponent* Reflection)
{
	if (Reflection && !RegisteredReflections.Contains(Reflection))
	{
		RegisteredReflections.Add(Reflection);
	}
}

void UPatternLightingSubsystem::UnregisterReflection(UPatternReflectionComponent* Reflection)
{
	RegisteredReflections.Remove(Reflection);
}

TArray<UPatternLightComponent*> UPatternLightingSubsystem::GetLightsInSyncGroup(FName GroupName) const
{
	TArray<UPatternLightComponent*> Result;
	
	if (const auto* Group = SyncGroups.Find(GroupName))
	{
		for (const auto& WeakLight : *Group)
		{
			if (auto* Light = WeakLight.Get())
			{
				Result.Add(Light);
			}
		}
	}
	
	return Result;
}

TArray<UPatternLightComponent*> UPatternLightingSubsystem::GetLightsAtLocation(FVector Location, float Radius) const
{
	TArray<UPatternLightComponent*> Result;
	
	for (const auto& WeakLight : RegisteredLights)
	{
		if (auto* Light = WeakLight.Get())
		{
			float Distance = FVector::Dist(Location, Light->GetComponentLocation());
			if (Distance <= Radius + Light->LightRadius)
			{
				Result.Add(Light);
			}
		}
	}
	
	return Result;
}

float UPatternLightingSubsystem::GetCombinedIntensityAt(FVector Location) const
{
	float TotalIntensity = 0.0f;
	
	for (const auto& WeakLight : RegisteredLights)
	{
		if (auto* Light = WeakLight.Get())
		{
			float Distance = FVector::Dist(Location, Light->GetComponentLocation());
			if (Distance < Light->LightRadius)
			{
				float Falloff = 1.0f - (Distance / Light->LightRadius);
				Falloff = FMath::Square(Falloff);
				TotalIntensity += Light->GetCurrentIntensity() * Falloff;
			}
		}
	}
	
	return TotalIntensity * GlobalConfig.GlobalIntensity;
}

FLinearColor UPatternLightingSubsystem::GetCombinedColorAt(FVector Location) const
{
	FLinearColor TotalColor = FLinearColor::Black;
	float TotalWeight = 0.0f;
	
	for (const auto& WeakLight : RegisteredLights)
	{
		if (auto* Light = WeakLight.Get())
		{
			float Distance = FVector::Dist(Location, Light->GetComponentLocation());
			if (Distance < Light->LightRadius)
			{
				float Weight = 1.0f - (Distance / Light->LightRadius);
				Weight *= Light->GetCurrentIntensity();
				
				TotalColor += Light->GetCurrentColor() * Weight;
				TotalWeight += Weight;
			}
		}
	}
	
	if (TotalWeight > 0.0f)
	{
		TotalColor /= TotalWeight;
	}
	
	return TotalColor;
}

UPatternReflectionComponent* UPatternLightingSubsystem::GetBestReflectionAt(FVector Location) const
{
	UPatternReflectionComponent* Best = nullptr;
	float BestWeight = 0.0f;
	
	for (const auto& WeakReflection : RegisteredReflections)
	{
		if (auto* Reflection = WeakReflection.Get())
		{
			float Distance = FVector::Dist(Location, Reflection->GetComponentLocation());
			float Radius = Reflection->ReflectionSettings.Radius;
			
			if (Distance < Radius)
			{
				float Weight = Reflection->ReflectionSettings.Intensity * (1.0f - Distance / Radius);
				if (Weight > BestWeight)
				{
					BestWeight = Weight;
					Best = Reflection;
				}
			}
		}
	}
	
	return Best;
}

void UPatternLightingSubsystem::SetGlobalIntensity(float Intensity)
{
	GlobalConfig.GlobalIntensity = FMath::Clamp(Intensity, 0.0f, 2.0f);
}

void UPatternLightingSubsystem::SetGlobalSpeed(float Speed)
{
	GlobalConfig.GlobalSpeed = FMath::Clamp(Speed, 0.1f, 5.0f);
}

void UPatternLightingSubsystem::PauseAll()
{
	bPaused = true;
}

void UPatternLightingSubsystem::ResumeAll()
{
	bPaused = false;
}

void UPatternLightingSubsystem::TriggerFlashAtLocation(FVector Location, float Radius, float Duration, float Intensity)
{
	for (const auto& WeakLight : RegisteredLights)
	{
		if (auto* Light = WeakLight.Get())
		{
			float Distance = FVector::Dist(Location, Light->GetComponentLocation());
			if (Distance < Radius)
			{
				float FalloffIntensity = Intensity * (1.0f - Distance / Radius);
				Light->TriggerFlash(Duration, FalloffIntensity);
			}
		}
	}
}

void UPatternLightingSubsystem::SyncGroup(FName GroupName)
{
	auto Lights = GetLightsInSyncGroup(GroupName);
	if (Lights.Num() > 1)
	{
		UPatternLightComponent* Master = Lights[0];
		for (int32 i = 1; i < Lights.Num(); i++)
		{
			Lights[i]->SyncWith(Master);
		}
	}
}

void UPatternLightingSubsystem::DrawDebug(float Duration)
{
	UWorld* World = GetWorld();
	if (!World) return;

	for (const auto& WeakLight : RegisteredLights)
	{
		if (auto* Light = WeakLight.Get())
		{
			FVector Location = Light->GetComponentLocation();
			float Intensity = Light->GetCurrentIntensity() / Light->BaseIntensity;
			FColor DebugColor = Light->GetCurrentColor().ToFColor(true);
			
			DrawDebugSphere(World, Location, 50.0f * Intensity, 8, DebugColor, false, Duration);
			DrawDebugString(World, Location + FVector(0, 0, 100), 
				FString::Printf(TEXT("I: %.2f"), Intensity), nullptr, FColor::White, Duration);
		}
	}
}

FString UPatternLightingSubsystem::GetStatsString() const
{
	return FString::Printf(TEXT("Pattern Lights: %d\nReflections: %d\nSync Groups: %d\nMaster Time: %.2f"),
		RegisteredLights.Num(),
		RegisteredReflections.Num(),
		SyncGroups.Num(),
		MasterTime);
}

void UPatternLightingSubsystem::CleanupStaleReferences()
{
	RegisteredLights.RemoveAll([](const TWeakObjectPtr<UPatternLightComponent>& Ptr) { return !Ptr.IsValid(); });
	RegisteredReflections.RemoveAll([](const TWeakObjectPtr<UPatternReflectionComponent>& Ptr) { return !Ptr.IsValid(); });
	
	for (auto& Pair : SyncGroups)
	{
		Pair.Value.RemoveAll([](const TWeakObjectPtr<UPatternLightComponent>& Ptr) { return !Ptr.IsValid(); });
	}
}

// ============================================================================
// Blueprint Library Implementation
// ============================================================================

UPatternLightingSubsystem* UPatternLightingBlueprintLibrary::GetPatternLightingSubsystem(UObject* WorldContextObject)
{
	if (UWorld* World = GEngine->GetWorldFromContextObject(WorldContextObject, EGetWorldErrorMode::ReturnNull))
	{
		return World->GetSubsystem<UPatternLightingSubsystem>();
	}
	return nullptr;
}

APatternPointLight* UPatternLightingBlueprintLibrary::SpawnPatternLight(
	UObject* WorldContextObject,
	FVector Location,
	ELightPattern Pattern,
	FLinearColor Color,
	float Intensity,
	float Radius)
{
	UWorld* World = GEngine->GetWorldFromContextObject(WorldContextObject, EGetWorldErrorMode::ReturnNull);
	if (!World) return nullptr;

	FActorSpawnParameters SpawnParams;
	APatternPointLight* Light = World->SpawnActor<APatternPointLight>(Location, FRotator::ZeroRotator, SpawnParams);
	
	if (Light && Light->LightComponent)
	{
		Light->LightComponent->PatternSettings.Pattern = Pattern;
		Light->LightComponent->BaseColor = Color;
		Light->LightComponent->BaseIntensity = Intensity;
		Light->LightComponent->LightRadius = Radius;
	}
	
	return Light;
}

float UPatternLightingBlueprintLibrary::EvaluatePattern(ELightPattern Pattern, float Time, float Speed)
{
	float T = Time * Speed;
	
	switch (Pattern)
	{
		case ELightPattern::Steady: return 1.0f;
		case ELightPattern::Pulse: return 0.5f + 0.5f * FMath::Sin(T * 2.0f * PI);
		case ELightPattern::Flicker: return 0.7f + 0.3f * FMath::Sin(T * 20.0f) * FMath::Sin(T * 7.3f);
		case ELightPattern::Strobe: return FMath::Sin(T * 10.0f) > 0.0f ? 1.0f : 0.0f;
		case ELightPattern::Candle: return 0.8f + 0.2f * FMath::Sin(T * 12.0f) * FMath::Sin(T * 5.7f) * FMath::Sin(T * 3.1f);
		case ELightPattern::Fire: return 0.7f + 0.3f * FMath::Sin(T * 8.0f) * FMath::Sin(T * 4.3f);
		case ELightPattern::Alarm: return FMath::Sin(T * 4.0f) > 0.0f ? 1.0f : 0.2f;
		default: return 1.0f;
	}
}

float UPatternLightingBlueprintLibrary::CalculateFresnelReflection(FVector ViewDir, FVector Normal, float IOR)
{
	float CosI = FMath::Abs(FVector::DotProduct(ViewDir, Normal));
	float F0 = FMath::Square((1.0f - IOR) / (1.0f + IOR));
	return F0 + (1.0f - F0) * FMath::Pow(1.0f - CosI, 5.0f);
}

FLinearColor UPatternLightingBlueprintLibrary::CalculatePBRSpecular(
	FVector Normal,
	FVector ViewDir,
	FVector LightDir,
	FLinearColor LightColor,
	float Roughness,
	float Metallic)
{
	FVector H = (ViewDir + LightDir).GetSafeNormal();
	float NdotH = FMath::Max(0.0f, FVector::DotProduct(Normal, H));
	float NdotV = FMath::Max(0.0f, FVector::DotProduct(Normal, ViewDir));
	float NdotL = FMath::Max(0.0f, FVector::DotProduct(Normal, LightDir));
	
	// GGX Distribution
	float a = Roughness * Roughness;
	float a2 = a * a;
	float denom = NdotH * NdotH * (a2 - 1.0f) + 1.0f;
	float D = a2 / (PI * denom * denom);
	
	// Schlick Fresnel
	float F0 = FMath::Lerp(0.04f, 1.0f, Metallic);
	float F = F0 + (1.0f - F0) * FMath::Pow(1.0f - NdotV, 5.0f);
	
	// Smith Geometry
	float k = (Roughness + 1.0f) * (Roughness + 1.0f) / 8.0f;
	float G1 = NdotV / (NdotV * (1.0f - k) + k);
	float G2 = NdotL / (NdotL * (1.0f - k) + k);
	float G = G1 * G2;
	
	float Specular = (D * F * G) / (4.0f * NdotV * NdotL + 0.001f);
	
	return LightColor * Specular * NdotL;
}

float UPatternLightingBlueprintLibrary::GetShadowSoftness(float LightRadius, float Distance)
{
	return FMath::Clamp(LightRadius / (Distance + 1.0f), 0.0f, 1.0f);
}

float UPatternLightingBlueprintLibrary::LerpPatterns(ELightPattern PatternA, ELightPattern PatternB, float Time, float Alpha)
{
	float ValueA = EvaluatePattern(PatternA, Time);
	float ValueB = EvaluatePattern(PatternB, Time);
	return FMath::Lerp(ValueA, ValueB, Alpha);
}

FLinearColor UPatternLightingBlueprintLibrary::ColorTemperatureToRGB(float Kelvin)
{
	// Approximate Planckian locus
	float Temp = Kelvin / 100.0f;
	
	float R, G, B;
	
	if (Temp <= 66.0f)
	{
		R = 255.0f;
		G = 99.4708025861f * FMath::Loge(Temp) - 161.1195681661f;
		
		if (Temp <= 19.0f)
			B = 0.0f;
		else
			B = 138.5177312231f * FMath::Loge(Temp - 10.0f) - 305.0447927307f;
	}
	else
	{
		R = 329.698727446f * FMath::Pow(Temp - 60.0f, -0.1332047592f);
		G = 288.1221695283f * FMath::Pow(Temp - 60.0f, -0.0755148492f);
		B = 255.0f;
	}
	
	return FLinearColor(
		FMath::Clamp(R / 255.0f, 0.0f, 1.0f),
		FMath::Clamp(G / 255.0f, 0.0f, 1.0f),
		FMath::Clamp(B / 255.0f, 0.0f, 1.0f)
	);
}

float UPatternLightingBlueprintLibrary::LuxToIntensity(float Lux)
{
	// Approximate conversion for UE5's cd/m^2 based system
	// Direct sunlight ~100000 lux, indoor ~500 lux
	return Lux * 0.08f; // Very rough approximation
}
