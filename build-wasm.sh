#!/bin/bash
set -ex

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the wasm package in development mode with full debug information
echo "Building wasm package in development mode..."
wasm-pack build --dev --target nodejs
echo "WASM build complete! Output is in the 'pkg' directory."
echo "Use this build for debugging. For production, run with --release." 

echo "Adding snippets to package.json..."
jq '.files += ["snippets/"]' pkg/package.json | \
jq '.dependencies = {"@vercel/kv": "^3.0.0"}' | \
jq '.name = "subconverter-wasm"' | \
jq '.type = "module"' > tmp.json && mv tmp.json pkg/package.json
cd pkg
pnpm install
cd ..

cd vercel-api-test
pnpm install
cd ..