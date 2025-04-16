#!/bin/bash
set -e

# Start stopwatch
BUILD_START_TIME=$SECONDS

# Script description
cat << "EOF"
Subconverter WASM Build & Release Script
---------------------------------------
Usage Options:
  --release          Build in release mode
  --prepare-release  Prepare a release: Update version, create temporary tag, and trigger GitHub Actions
  --bump-patch       Bump patch version number, commit change and prepare release (convenient for routine updates)
  --version X.Y.Z    Specify version (used with --release or --prepare-release)

Examples:
  ./build-wasm.sh                      # Build in development mode
  ./build-wasm.sh --release            # Build in release mode
  ./build-wasm.sh --bump-patch         # Auto-bump patch version and prepare release
  ./build-wasm.sh --prepare-release --version 0.3.0  # Prepare specific version release
EOF

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
VERSION=""
PREPARE_RELEASE=false
BUMP_PATCH=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --release)
      RELEASE_MODE=true
      shift
      ;;
    --prepare-release)
      PREPARE_RELEASE=true
      RELEASE_MODE=true
      shift
      ;;
    --bump-patch)
      BUMP_PATCH=true
      PREPARE_RELEASE=true
      RELEASE_MODE=true
      shift
      ;;
    --version)
      VERSION="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--release] [--prepare-release] [--bump-patch] [--version X.Y.Z]"
      exit 1
      ;;
  esac
done

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep -m 1 "version" Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
echo "Current package version: $CURRENT_VERSION"

# Bump patch version if requested
if [ "$BUMP_PATCH" = true ]; then
  # Extract major, minor and patch numbers
  MAJOR=$(echo "$CURRENT_VERSION" | cut -d. -f1)
  MINOR=$(echo "$CURRENT_VERSION" | cut -d. -f2)
  PATCH=$(echo "$CURRENT_VERSION" | cut -d. -f3 | cut -d- -f1)
  
  # Bump patch version
  NEW_PATCH=$((PATCH + 1))
  VERSION="${MAJOR}.${MINOR}.${NEW_PATCH}"
  
  echo "Bumping patch version from $CURRENT_VERSION to $VERSION"
fi

# If version is provided but we're not in release mode, switch to release mode
if [ -n "$VERSION" ] && [ "$RELEASE_MODE" = false ]; then
  echo "Version specified, switching to release mode"
  RELEASE_MODE=true
fi

# If we're in release mode and no version is provided, generate a pre-release version
if [ "$RELEASE_MODE" = true ] && [ -z "$VERSION" ]; then
  # Extract the base version without any pre-release tags (e.g., 0.1.0 from 0.1.0-pre.xxx)
  BASE_VERSION=$(echo "$CURRENT_VERSION" | sed 's/\([0-9]\+\.[0-9]\+\.[0-9]\+\).*/\1/')
  
  # Generate a pre-release version based on base version + date + short git hash
  GIT_HASH=$(git rev-parse --short HEAD)
  DATE_PART=$(date '+%Y%m%d')
  VERSION="${BASE_VERSION}-pre.${DATE_PART}.${GIT_HASH}"
  echo "Auto-generated pre-release version: $VERSION"
fi

# Variable to track if version was updated
VERSION_UPDATED=false

# Prepare release (create temporary tag for CI)
if [ "$PREPARE_RELEASE" = true ]; then
  # Check if git work area is clean
  if [ -n "$(git status --porcelain)" ]; then
    echo "Error: Git working directory is not clean."
    echo "Please commit or stash your changes before running version release."
    exit 1
  fi
  
  # Update version in Cargo.toml if needed
  if [ -n "$VERSION" ] && [ "$VERSION" != "$CURRENT_VERSION" ]; then
    echo "Updating version to $VERSION in Cargo.toml"
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$VERSION\"/" Cargo.toml
    echo "Running cargo check to update Cargo.lock"
    cargo check
    VERSION_UPDATED=true
  fi
  
  # Fetch remote tags to check for previous release attempts
  git fetch --tags
  
  # Count previous release attempts for this version
  BASE_TAG="v${VERSION}"
  ATTEMPT_COUNT=$(git tag -l "${BASE_TAG}-attempt*" | wc -l)
  ATTEMPT_COUNT=$((ATTEMPT_COUNT + 1))
  
  # Create a temporary tag for this release attempt
  TEMP_TAG="${BASE_TAG}-attempt${ATTEMPT_COUNT}"
  
  echo "Creating temporary tag ${TEMP_TAG} for CI workflow..."
  git add Cargo.toml
  git add Cargo.lock
  git commit -m "Prepare release $VERSION (attempt $ATTEMPT_COUNT)"
  git tag -a "${TEMP_TAG}" -m "Preparing release $VERSION (attempt $ATTEMPT_COUNT)"
  
  echo "Pushing changes and temporary tag to remote repository..."
  git push origin main
  git push origin "${TEMP_TAG}"
  
  echo "Temporary tag created. CI workflow will handle the rest of the release process."
  exit 0
fi

# Build the wasm package
if [ "$RELEASE_MODE" = true ]; then
  echo "Building wasm package in release mode..."
  
  # Update version in Cargo.toml if needed
  if [ -n "$VERSION" ] && [ "$VERSION" != "$CURRENT_VERSION" ]; then
    # Check if git work area is clean
    if [ -n "$(git status --porcelain)" ]; then
      echo "Error: Git working directory is not clean."
      echo "Please commit or stash your changes before running version release."
      exit 1
    fi
    
    echo "Updating version to $VERSION in Cargo.toml"
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$VERSION\"/" Cargo.toml
    echo "Running cargo check to update Cargo.lock"
    cargo check
    VERSION_UPDATED=true
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
# Use VERSION if set, otherwise use the version from Cargo.toml
PKG_VERSION=${VERSION:-$CURRENT_VERSION}
jq '.files += ["snippets/"]' pkg/package.json | \
  jq '.dependencies = {"@vercel/kv": "^3.0.0"}' | \
  jq '.name = "subconverter-wasm"' | \
  jq '.dependencies["@vercel/kv"] = "^3.0.0"' | \
  jq '.dependencies["@netlify/blobs"] = "^8.1.2"' | \
  jq ".version = \"$PKG_VERSION\"" > tmp.json && mv tmp.json pkg/package.json

# Install dependencies in pkg
cd pkg
yarn install
cd ..

# Setup development environment if in dev mode
if [ "$RELEASE_MODE" = false ]; then
  echo "Setting up development environment..."
  
  # Check if www directory exists and copy files directly
  if [ -d "www" ]; then
    echo "Copying WASM files to www project..."
    
    # Create necessary directories
    mkdir -p www/node_modules/subconverter-wasm
    
    # Copy all files from pkg to www/node_modules/subconverter-wasm
    cp -r pkg/* www/node_modules/subconverter-wasm/
    
    echo "Successfully copied WASM files to www/node_modules/subconverter-wasm"
    echo "Note: You'll need to run this script again after any changes to the WASM code"
  else
    echo "Warning: www directory not found, skipping copy to www project"
  fi
fi

# Install dependencies in vercel-api-test
if [ -d "vercel-api-test" ]; then
  echo "Installing dependencies in vercel-api-test..."
  cd vercel-api-test
  yarn install
  cd ..
fi

echo "Build script completed successfully!"

# Calculate and print build time
BUILD_END_TIME=$SECONDS
BUILD_DURATION=$((BUILD_END_TIME - BUILD_START_TIME))
echo "Total build time: $((BUILD_DURATION / 60)) minutes and $((BUILD_DURATION % 60)) seconds"