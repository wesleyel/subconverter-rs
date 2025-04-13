#!/bin/bash
set -e

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "jq is required. Please install it using your package manager."
    exit 1
fi

# Parse arguments
RELEASE_MODE=false
PUBLISH_NPM=false
PUBLISH_CRATES=false
VERSION=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --release)
      RELEASE_MODE=true
      shift
      ;;
    --publish-npm)
      PUBLISH_NPM=true
      shift
      ;;
    --publish-crates)
      PUBLISH_CRATES=true
      shift
      ;;
    --version)
      VERSION="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--release] [--publish-npm] [--publish-crates] [--version X.Y.Z]"
      exit 1
      ;;
  esac
done

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep -m 1 "version" Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
echo "Current package version: $CURRENT_VERSION"

# If version is provided but we're not in release mode, switch to release mode
if [ -n "$VERSION" ] && [ "$RELEASE_MODE" = false ]; then
  echo "Version specified, switching to release mode"
  RELEASE_MODE=true
fi

# If we're in release mode and no version is provided, generate a pre-release version
if [ "$RELEASE_MODE" = true ] && [ -z "$VERSION" ]; then
  # Generate a pre-release version based on current version + date + short git hash
  GIT_HASH=$(git rev-parse --short HEAD)
  DATE_PART=$(date '+%Y%m%d')
  VERSION="${CURRENT_VERSION}-pre.${DATE_PART}.${GIT_HASH}"
  echo "Auto-generated pre-release version: $VERSION"
fi

# Build the wasm package
if [ "$RELEASE_MODE" = true ]; then
  echo "Building wasm package in release mode..."
  
  # Update version in Cargo.toml if needed
  if [ -n "$VERSION" ] && [ "$VERSION" != "$CURRENT_VERSION" ]; then
    echo "Updating version to $VERSION in Cargo.toml"
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$VERSION\"/" Cargo.toml
  fi
  
  wasm-pack build --release --target nodejs
  echo "WASM release build complete! Output is in the 'pkg' directory."
else
  echo "Building wasm package in development mode..."
  wasm-pack build --dev --target nodejs
  echo "WASM development build complete! Output is in the 'pkg' directory."
fi

# Update package.json in pkg
echo "Updating package.json..."
VERSION_FROM_CARGO=$(grep -m 1 "version" Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
jq '.files += ["snippets/"]' pkg/package.json | \
  jq '.dependencies = {"@vercel/kv": "^3.0.0"}' | \
  jq '.name = "subconverter-wasm"' | \
  jq '.dependencies["@vercel/kv"] = "^3.0.0"' | \
  jq '.dependencies["@netlify/blobs"] = "^8.1.2"' | \
  jq ".version = \"$VERSION_FROM_CARGO\"" > tmp.json && mv tmp.json pkg/package.json

# Install dependencies in pkg
cd pkg
pnpm install
cd ..

# Publish to crates.io if requested
if [ "$RELEASE_MODE" = true ] && [ "$PUBLISH_CRATES" = true ]; then
  echo "Publishing to crates.io..."
  cargo publish --allow-dirty
fi

# Publish to npm if requested
if [ "$RELEASE_MODE" = true ] && [ "$PUBLISH_NPM" = true ]; then
  echo "Publishing to npm..."
  cd pkg
  npm publish --access public
  cd ..
fi

# Setup development environment if in dev mode
if [ "$RELEASE_MODE" = false ]; then
  echo "Setting up development environment..."
  
  # Check if www directory exists and use pnpm link
  if [ -d "www" ]; then
    echo "Linking to www project using pnpm..."
    # Use pnpm to create a link from www to the local pkg directory
    cd www
    # Remove any existing package if present
    rm -rf node_modules/subconverter-wasm 2>/dev/null || true
    # Link to the local pkg directory
    pnpm link ../pkg
    cd ..
    echo "Successfully linked pkg to www using pnpm"
  else
    echo "Warning: www directory not found, skipping link to www project"
  fi
fi

# Install dependencies in vercel-api-test
if [ -d "vercel-api-test" ]; then
  echo "Installing dependencies in vercel-api-test..."
  cd vercel-api-test
  pnpm install
  cd ..
fi

echo "Build script completed successfully!"