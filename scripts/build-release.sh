#!/bin/bash
# Build script for Adaptive Entity Engine v1.0

set -e

echo "Building Adaptive Entity Engine v1.0..."

# Build release
cargo build --release

echo "Build complete!"
echo "Binary location: target/release/adaptive-entity-engine"

# Show binary size
if [ -f "target/release/adaptive-entity-engine" ]; then
    SIZE=$(du -h target/release/adaptive-entity-engine | cut -f1)
    echo "Binary size: $SIZE"
fi
