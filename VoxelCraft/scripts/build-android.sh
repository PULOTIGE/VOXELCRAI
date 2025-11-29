#!/bin/bash
# VoxelCraft Android Build Script

set -e

echo "🎮 VoxelCraft Android Build"
echo "=========================="

# Check for Android NDK
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo "⚠️  ANDROID_NDK_HOME not set. Please install Android NDK and set the environment variable."
    echo "   You can install it via Android Studio or command line:"
    echo "   sdkmanager \"ndk;25.2.9519653\""
    exit 1
fi

# Check for Rust Android targets
if ! rustup target list | grep -q "aarch64-linux-android (installed)"; then
    echo "📦 Installing Rust Android targets..."
    rustup target add aarch64-linux-android
    rustup target add armv7-linux-androideabi
    rustup target add x86_64-linux-android
fi

# Check for cargo-ndk
if ! command -v cargo-ndk &> /dev/null; then
    echo "📦 Installing cargo-ndk..."
    cargo install cargo-ndk
fi

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_DIR"

echo "📁 Project directory: $PROJECT_DIR"

# Build for different architectures
echo ""
echo "🔨 Building native library for Android..."

# ARM64 (most modern devices)
echo "  → Building for arm64-v8a..."
cargo ndk -t arm64-v8a -o android/app/src/main/jniLibs build --release

# ARMv7 (older devices)
echo "  → Building for armeabi-v7a..."
cargo ndk -t armeabi-v7a -o android/app/src/main/jniLibs build --release

# x86_64 (emulators)
echo "  → Building for x86_64..."
cargo ndk -t x86_64 -o android/app/src/main/jniLibs build --release

echo ""
echo "✅ Native libraries built successfully!"

# Build APK
echo ""
echo "📱 Building APK..."
cd android

if [ -f "gradlew" ]; then
    chmod +x gradlew
    ./gradlew assembleRelease
else
    echo "⚠️  Gradle wrapper not found. Creating..."
    gradle wrapper
    chmod +x gradlew
    ./gradlew assembleRelease
fi

APK_PATH="app/build/outputs/apk/release/app-release-unsigned.apk"

if [ -f "$APK_PATH" ]; then
    echo ""
    echo "✅ APK built successfully!"
    echo "📍 Location: $PROJECT_DIR/android/$APK_PATH"
    
    # Copy to project root
    cp "$APK_PATH" "$PROJECT_DIR/VoxelCraft-unsigned.apk"
    echo "📍 Copied to: $PROJECT_DIR/VoxelCraft-unsigned.apk"
    
    echo ""
    echo "📝 To sign the APK for release:"
    echo "   1. Create a keystore: keytool -genkey -v -keystore voxelcraft.keystore -keyalg RSA -keysize 2048 -validity 10000 -alias voxelcraft"
    echo "   2. Sign: apksigner sign --ks voxelcraft.keystore --out VoxelCraft.apk VoxelCraft-unsigned.apk"
else
    echo "❌ APK build failed!"
    exit 1
fi

echo ""
echo "🎮 Build complete!"
