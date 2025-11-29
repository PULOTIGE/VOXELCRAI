# VoxelCraft Makefile

.PHONY: all clean desktop android check fmt

# Default target
all: desktop

# Build for desktop
desktop:
	cargo build --release
	@echo "✅ Desktop build complete: target/release/voxelcraft"

# Build for Android
android:
	./scripts/build-android.sh

# Check code
check:
	cargo check
	cargo clippy -- -W clippy::all

# Format code
fmt:
	cargo fmt

# Clean build artifacts
clean:
	cargo clean
	rm -rf android/app/build
	rm -rf android/app/src/main/jniLibs
	rm -f VoxelCraft-unsigned.apk
	rm -f voxelcraft

# Run desktop version
run:
	cargo run --release

# Run with debug logging
run-debug:
	RUST_LOG=info cargo run

# Install Android tools
install-android-tools:
	rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
	cargo install cargo-ndk

# Help
help:
	@echo "VoxelCraft Build System"
	@echo ""
	@echo "Targets:"
	@echo "  desktop    - Build for desktop (default)"
	@echo "  android    - Build APK for Android"
	@echo "  run        - Build and run desktop version"
	@echo "  check      - Run cargo check and clippy"
	@echo "  fmt        - Format code"
	@echo "  clean      - Clean build artifacts"
	@echo "  help       - Show this help"
