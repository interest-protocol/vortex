#!/bin/bash
set -e

echo "Building Vortex WASM module..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack is not installed"
    echo "Install it with: cargo install wasm-pack"
    exit 1
fi

# Build for Node.js
echo "Building for Node.js..."
wasm-pack build --target nodejs --out-dir pkg/nodejs --release

# Build for web browsers
echo "Building for web..."
wasm-pack build --target web --out-dir pkg/web --release

# Build for bundlers (webpack, rollup, etc.)
echo "Building for bundlers..."
wasm-pack build --target bundler --out-dir pkg/bundler --release

echo "✅ WASM build complete!"
echo "Outputs:"
echo "  - Node.js: pkg/nodejs/"
echo "  - Web: pkg/web/"
echo "  - Bundlers: pkg/bundler/"