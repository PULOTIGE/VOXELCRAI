#!/bin/bash
# VoxelCraft Desktop Build Script

set -e

echo "🎮 VoxelCraft Desktop Build"
echo "==========================="

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_DIR"

echo "📁 Project directory: $PROJECT_DIR"
echo ""

# Build for current platform
echo "🔨 Building for desktop..."

cargo build --release

# Get the executable name based on OS
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
    EXE_NAME="voxelcraft.exe"
else
    EXE_NAME="voxelcraft"
fi

EXE_PATH="target/release/$EXE_NAME"

if [ -f "$EXE_PATH" ]; then
    echo ""
    echo "✅ Build successful!"
    echo "📍 Executable: $PROJECT_DIR/$EXE_PATH"
    
    # Copy to project root
    cp "$EXE_PATH" "$PROJECT_DIR/"
    echo "📍 Copied to: $PROJECT_DIR/$EXE_NAME"
    
    echo ""
    echo "🚀 Run with: ./$EXE_NAME"
else
    echo "❌ Build failed!"
    exit 1
fi
