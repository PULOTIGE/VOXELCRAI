#!/bin/bash
# Build script for VOXELCRAI

set -e

echo "Building VOXELCRAI..."

# Build release
cargo build --release

echo "Build complete!"
echo "Binary location: target/release/voxelcrai"

# Show binary size
if [ -f "target/release/voxelcrai" ]; then
    SIZE=$(du -h target/release/voxelcrai | cut -f1)
    echo "Binary size: $SIZE"
fi
