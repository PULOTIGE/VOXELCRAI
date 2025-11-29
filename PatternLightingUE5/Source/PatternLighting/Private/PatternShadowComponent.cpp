// Pattern Lighting System for UE5
// Shadow Component Implementation

#include "PatternShadowComponent.h"
#include "Components/BoxComponent.h"
#include "Components/DirectionalLightComponent.h"
#include "PatternLightComponent.h"

UPatternShadowComponent::UPatternShadowComponent()
{
	PrimaryComponentTick.bCanEverTick = true;
	PrimaryComponentTick.bStartWithTickEnabled = true;
	
	// Default cascade distances
	CascadeDistances.Add(1000.0f);
	CascadeDistances.Add(3000.0f);
	CascadeDistances.Add(10000.0f);
	CascadeDistances.Add(20000.0f);
}

void UPatternShadowComponent::BeginPlay()
{
	Super::BeginPlay();
	UpdateCascades();
}

void UPatternShadowComponent::TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction)
{
	Super::TickComponent(DeltaTime, TickType, ThisTickFunction);
	
	if (ShadowSettings.bContactShadows)
	{
		CalculateContactShadows();
	}
}

TArray<float> UPatternShadowComponent::CalculateCascadeSplits(float NearPlane, float FarPlane) const
{
	TArray<float> Splits;
	
	int32 NumCascades = ShadowSettings.CascadeCount;
	float Lambda = ShadowSettings.CascadeDistribution;
	
	for (int32 i = 0; i <= NumCascades; i++)
	{
		float P = static_cast<float>(i) / static_cast<float>(NumCascades);
		
		// Logarithmic split
		float LogSplit = NearPlane * FMath::Pow(FarPlane / NearPlane, P);
		
		// Linear split
		float LinearSplit = NearPlane + (FarPlane - NearPlane) * P;
		
		// Blend between log and linear
		float Split = FMath::Lerp(LinearSplit, LogSplit, Lambda);
		
		Splits.Add(Split);
	}
	
	return Splits;
}

float UPatternShadowComponent::GetShadowIntensityAt(FVector WorldLocation) const
{
	// Simplified shadow intensity based on distance
	// In a real implementation, this would sample the shadow map
	return ShadowSettings.Intensity;
}

void UPatternShadowComponent::ApplyToDirectionalLight(UDirectionalLightComponent* Light)
{
	if (!Light)
	{
		return;
	}
	
	Light->CastShadows = true;
	Light->CastDynamicShadows = true;
	
	// Set cascade settings
	Light->DynamicShadowDistanceMovableLight = DynamicShadowDistance;
	Light->DynamicShadowCascades = ShadowSettings.CascadeCount;
	Light->CascadeDistributionExponent = ShadowSettings.CascadeDistribution;
	
	// Shadow softness
	Light->ShadowSlopeBias = ShadowSettings.Bias;
	
	// Contact shadows
	Light->bUseContactShadows = ShadowSettings.bContactShadows;
	Light->ContactShadowLength = ShadowSettings.ContactShadowLength;
}

float UPatternShadowComponent::CalculatePenumbra(float LightRadius, float OccluderDistance, float ReceiverDistance)
{
	if (OccluderDistance <= 0.0f || ReceiverDistance <= OccluderDistance)
	{
		return 0.0f;
	}
	
	// Penumbra size based on geometry
	float PenumbraSize = LightRadius * (ReceiverDistance - OccluderDistance) / OccluderDistance;
	return FMath::Clamp(PenumbraSize, 0.0f, LightRadius);
}

void UPatternShadowComponent::UpdateCascades()
{
	// Calculate cascade distances based on settings
	CascadeDistances.Empty();
	
	auto Splits = CalculateCascadeSplits(1.0f, DynamicShadowDistance);
	CascadeDistances = Splits;
}

void UPatternShadowComponent::CalculateContactShadows()
{
	// Contact shadow calculation would be done in a post-process
	// This is a placeholder for per-frame updates
}

// ============================================================================
// Pattern Directional Light
// ============================================================================

APatternDirectionalLight::APatternDirectionalLight()
{
	DirectionalLight = CreateDefaultSubobject<UDirectionalLightComponent>(TEXT("DirectionalLight"));
	DirectionalLight->SetMobility(EComponentMobility::Movable);
	RootComponent = DirectionalLight;
	
	ShadowComponent = CreateDefaultSubobject<UPatternShadowComponent>(TEXT("ShadowComponent"));
	ShadowComponent->SetupAttachment(RootComponent);
	
	PatternComponent = CreateDefaultSubobject<UPatternLightComponent>(TEXT("PatternComponent"));
	PatternComponent->SetupAttachment(RootComponent);
	
	PrimaryActorTick.bCanEverTick = true;
	PrimaryActorTick.bStartWithTickEnabled = true;
}

void APatternDirectionalLight::Tick(float DeltaTime)
{
	Super::Tick(DeltaTime);
	
	if (bAutoRotate)
	{
		CurrentTimeOfDay += DeltaTime / DayLength;
		if (CurrentTimeOfDay >= 1.0f)
		{
			CurrentTimeOfDay -= 1.0f;
		}
		
		SetTimeOfDay(CurrentTimeOfDay);
	}
	
	// Apply shadow settings
	ShadowComponent->ApplyToDirectionalLight(DirectionalLight);
}

void APatternDirectionalLight::SetTimeOfDay(float NormalizedTime)
{
	CurrentTimeOfDay = FMath::Clamp(NormalizedTime, 0.0f, 1.0f);
	
	FVector SunDir = GetSunDirection(CurrentTimeOfDay);
	FRotator Rotation = SunDir.Rotation();
	SetActorRotation(Rotation);
	
	// Adjust intensity and color based on time
	float Angle = FMath::Abs(SunDir.Z);
	
	if (DirectionalLight)
	{
		// Intensity based on sun angle
		float Intensity = FMath::Lerp(0.1f, 1.0f, FMath::Clamp(Angle, 0.0f, 1.0f));
		DirectionalLight->SetIntensity(Intensity * 10.0f);
		
		// Color temperature based on time
		float ColorTemp = FMath::Lerp(2000.0f, 6500.0f, Angle);
		DirectionalLight->SetLightColor(FLinearColor::MakeFromColorTemperature(ColorTemp));
	}
}

FVector APatternDirectionalLight::GetSunDirection(float NormalizedTime) const
{
	// Simple circular sun path
	float Angle = NormalizedTime * 2.0f * PI;
	
	float X = FMath::Cos(Angle);
	float Y = 0.0f;
	float Z = FMath::Sin(Angle);
	
	// Offset so noon is at the top
	Z = FMath::Cos(Angle - PI * 0.5f);
	X = FMath::Sin(Angle - PI * 0.5f);
	
	return FVector(X, Y, Z).GetSafeNormal();
}

// ============================================================================
// Shadow Caster Volume
// ============================================================================

APatternShadowCasterVolume::APatternShadowCasterVolume()
{
	VolumeBox = CreateDefaultSubobject<UBoxComponent>(TEXT("VolumeBox"));
	VolumeBox->SetBoxExtent(FVector(500.0f, 500.0f, 500.0f));
	VolumeBox->SetCollisionEnabled(ECollisionEnabled::QueryOnly);
	VolumeBox->ShapeColor = FColor::Orange;
	RootComponent = VolumeBox;
}

void APatternShadowCasterVolume::BeginPlay()
{
	Super::BeginPlay();
	ApplyToActorsInVolume();
}

void APatternShadowCasterVolume::ApplyToActorsInVolume()
{
	if (!VolumeBox)
	{
		return;
	}
	
	TArray<AActor*> OverlappingActors;
	VolumeBox->GetOverlappingActors(OverlappingActors);
	
	for (AActor* Actor : OverlappingActors)
	{
		if (!Actor)
		{
			continue;
		}
		
		TArray<UStaticMeshComponent*> MeshComponents;
		Actor->GetComponents<UStaticMeshComponent>(MeshComponents);
		
		for (UStaticMeshComponent* Mesh : MeshComponents)
		{
			if (bForceDynamicShadows)
			{
				Mesh->CastShadow = true;
				Mesh->bCastDynamicShadow = true;
			}
			
			if (bForceContactShadows)
			{
				Mesh->bCastContactShadow = true;
			}
		}
	}
}
