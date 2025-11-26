// Pattern Lighting System for UE5
// Reflection Component Implementation

#include "PatternReflectionComponent.h"
#include "Components/BoxComponent.h"
#include "Components/StaticMeshComponent.h"
#include "Engine/StaticMesh.h"
#include "PatternLightingSubsystem.h"

UPatternReflectionComponent::UPatternReflectionComponent()
{
	PrimaryComponentTick.bCanEverTick = true;
	PrimaryComponentTick.bStartWithTickEnabled = true;
}

void UPatternReflectionComponent::BeginPlay()
{
	Super::BeginPlay();
	
	// Register with subsystem
	if (UWorld* World = GetWorld())
	{
		if (UPatternLightingSubsystem* Subsystem = World->GetSubsystem<UPatternLightingSubsystem>())
		{
			Subsystem->RegisterReflection(this);
		}
	}
	
	// Initial capture
	if (UpdateFrequency == 0.0f)
	{
		CaptureReflection();
	}
}

void UPatternReflectionComponent::TickComponent(float DeltaTime, ELevelTick TickType, FActorComponentTickFunction* ThisTickFunction)
{
	Super::TickComponent(DeltaTime, TickType, ThisTickFunction);
	
	if (UpdateFrequency > 0.0f)
	{
		TimeSinceLastUpdate += DeltaTime;
		
		if (TimeSinceLastUpdate >= UpdateFrequency)
		{
			TimeSinceLastUpdate = 0.0f;
			CaptureReflection();
		}
	}
}

void UPatternReflectionComponent::UpdateCapture()
{
	CaptureReflection();
}

void UPatternReflectionComponent::CaptureReflection()
{
	// In a real implementation, this would trigger a scene capture
	// For this plugin, we rely on UE5's built-in reflection system
	// and enhance it with our custom settings
}

float UPatternReflectionComponent::GetReflectionIntensityAt(FVector WorldLocation) const
{
	float Distance = FVector::Dist(WorldLocation, GetComponentLocation());
	
	if (Distance > ReflectionSettings.Radius)
	{
		return 0.0f;
	}
	
	float Falloff = 1.0f - (Distance / ReflectionSettings.Radius);
	return ReflectionSettings.Intensity * Falloff * BlendWeight;
}

float UPatternReflectionComponent::CalculateFresnel(FVector ViewDirection, FVector SurfaceNormal, float Exponent)
{
	float CosTheta = FMath::Abs(FVector::DotProduct(ViewDirection, SurfaceNormal));
	return FMath::Pow(1.0f - CosTheta, Exponent);
}

// ============================================================================
// SSR Volume
// ============================================================================

APatternSSRVolume::APatternSSRVolume()
{
	VolumeBox = CreateDefaultSubobject<UBoxComponent>(TEXT("VolumeBox"));
	VolumeBox->SetBoxExtent(FVector(500.0f, 500.0f, 500.0f));
	VolumeBox->SetCollisionEnabled(ECollisionEnabled::NoCollision);
	VolumeBox->SetVisibility(true);
	VolumeBox->ShapeColor = FColor::Cyan;
	RootComponent = VolumeBox;
}

bool APatternSSRVolume::IsPointInside(FVector WorldPoint) const
{
	if (!VolumeBox)
	{
		return false;
	}
	
	FVector LocalPoint = VolumeBox->GetComponentTransform().InverseTransformPosition(WorldPoint);
	FVector Extent = VolumeBox->GetUnscaledBoxExtent();
	
	return FMath::Abs(LocalPoint.X) <= Extent.X &&
		   FMath::Abs(LocalPoint.Y) <= Extent.Y &&
		   FMath::Abs(LocalPoint.Z) <= Extent.Z;
}

// ============================================================================
// Planar Reflection
// ============================================================================

APatternPlanarReflection::APatternPlanarReflection()
{
	ReflectionComponent = CreateDefaultSubobject<UPatternReflectionComponent>(TEXT("ReflectionComponent"));
	RootComponent = ReflectionComponent;
	
	PlaneMesh = CreateDefaultSubobject<UStaticMeshComponent>(TEXT("PlaneMesh"));
	PlaneMesh->SetupAttachment(RootComponent);
	PlaneMesh->SetVisibility(false);
}

FVector APatternPlanarReflection::GetReflectedPosition(FVector WorldPosition) const
{
	FVector PlaneOrigin = GetActorLocation();
	FVector ToPoint = WorldPosition - PlaneOrigin;
	float Distance = FVector::DotProduct(ToPoint, ReflectionNormal);
	
	return WorldPosition - 2.0f * Distance * ReflectionNormal;
}
