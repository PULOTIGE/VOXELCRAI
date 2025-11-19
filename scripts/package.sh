#!/bin/bash
# Package Adaptive Entity Engine v1.0 as ZIP

set -e

PROJECT_NAME="adaptive-entity-engine-v1.0"
ZIP_NAME="${PROJECT_NAME}.zip"
TEMP_DIR=$(mktemp -d)

echo "Packaging Adaptive Entity Engine v1.0..."

# Copy project files
cp -r src "$TEMP_DIR/"
cp -r arm "$TEMP_DIR/"
cp -r scripts "$TEMP_DIR/"
cp Cargo.toml "$TEMP_DIR/"
cp build.rs "$TEMP_DIR/"
cp README.md "$TEMP_DIR/"
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
