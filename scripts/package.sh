#!/bin/bash
# Package VOXELCRAI as ZIP

set -e

PROJECT_NAME="voxelcrai"
ZIP_NAME="${PROJECT_NAME}.zip"
TEMP_DIR=$(mktemp -d)

echo "Packaging VOXELCRAI..."

# Copy project files
cp -r src "$TEMP_DIR/"
cp -r arm "$TEMP_DIR/"
cp -r scripts "$TEMP_DIR/"
cp Cargo.toml "$TEMP_DIR/"
cp build.rs "$TEMP_DIR/"
cp README.md "$TEMP_DIR/"
cp ARCHITECTURE.md "$TEMP_DIR/" 2>/dev/null || true
cp PROJECT_STRUCTURE.md "$TEMP_DIR/" 2>/dev/null || true
cp QUICKSTART.md "$TEMP_DIR/" 2>/dev/null || true
cp CHANGELOG.md "$TEMP_DIR/"
cp LICENSE-MIT "$TEMP_DIR/"
cp LICENSE-APACHE "$TEMP_DIR/"
cp Makefile "$TEMP_DIR/"
cp .gitignore "$TEMP_DIR/"
cp rustfmt.toml "$TEMP_DIR/"
cp .clippy.toml "$TEMP_DIR/" 2>/dev/null || true
cp -r .cargo "$TEMP_DIR/" 2>/dev/null || true

# Create ZIP
cd "$TEMP_DIR"
zip -r "$ZIP_NAME" . > /dev/null
mv "$ZIP_NAME" /workspace/

cd /workspace
rm -rf "$TEMP_DIR"

echo "Package created: $ZIP_NAME"
ls -lh "$ZIP_NAME"
