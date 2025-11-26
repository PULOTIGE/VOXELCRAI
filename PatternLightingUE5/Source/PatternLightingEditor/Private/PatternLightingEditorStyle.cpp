// Pattern Lighting System for UE5
// Editor Style Implementation

#include "PatternLightingEditorStyle.h"
#include "Framework/Application/SlateApplication.h"
#include "Styling/SlateStyleRegistry.h"
#include "Interfaces/IPluginManager.h"

#define IMAGE_BRUSH(RelativePath, ...) FSlateImageBrush(Style->RootToContentDir(RelativePath, TEXT(".png")), __VA_ARGS__)
#define BOX_BRUSH(RelativePath, ...) FSlateBoxBrush(Style->RootToContentDir(RelativePath, TEXT(".png")), __VA_ARGS__)
#define BORDER_BRUSH(RelativePath, ...) FSlateBorderBrush(Style->RootToContentDir(RelativePath, TEXT(".png")), __VA_ARGS__)

TSharedPtr<FSlateStyleSet> FPatternLightingEditorStyle::StyleInstance = nullptr;

void FPatternLightingEditorStyle::Initialize()
{
	if (!StyleInstance.IsValid())
	{
		StyleInstance = Create();
		FSlateStyleRegistry::RegisterSlateStyle(*StyleInstance);
	}
}

void FPatternLightingEditorStyle::Shutdown()
{
	FSlateStyleRegistry::UnRegisterSlateStyle(*StyleInstance);
	ensure(StyleInstance.IsUnique());
	StyleInstance.Reset();
}

void FPatternLightingEditorStyle::ReloadTextures()
{
	if (FSlateApplication::IsInitialized())
	{
		FSlateApplication::Get().GetRenderer()->ReloadTextureResources();
	}
}

const ISlateStyle& FPatternLightingEditorStyle::Get()
{
	return *StyleInstance;
}

FName FPatternLightingEditorStyle::GetStyleSetName()
{
	static FName StyleSetName(TEXT("PatternLightingEditorStyle"));
	return StyleSetName;
}

TSharedRef<FSlateStyleSet> FPatternLightingEditorStyle::Create()
{
	TSharedRef<FSlateStyleSet> Style = MakeShareable(new FSlateStyleSet("PatternLightingEditorStyle"));
	Style->SetContentRoot(IPluginManager::Get().FindPlugin("PatternLighting")->GetContentDir());
	
	// Define custom styles
	Style->Set("PatternLighting.Icon", new FSlateImageBrush(
		Style->RootToContentDir(TEXT("Resources/Icon128"), TEXT(".png")),
		FVector2D(40.0f, 40.0f)
	));
	
	Style->Set("PatternLighting.SmallIcon", new FSlateImageBrush(
		Style->RootToContentDir(TEXT("Resources/Icon16"), TEXT(".png")),
		FVector2D(16.0f, 16.0f)
	));
	
	return Style;
}

#undef IMAGE_BRUSH
#undef BOX_BRUSH
#undef BORDER_BRUSH
