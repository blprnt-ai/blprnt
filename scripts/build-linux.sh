#!/bin/bash
set -e

# Get version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' tauri-src/Cargo.toml | head -n1 | cut -d '"' -f 2)
if [ -z "$CURRENT_VERSION" ]; then
  echo "Error: No version found in tauri-src/Cargo.toml"
  exit 1
fi

echo "Building blprnt v${CURRENT_VERSION} for Linux..."

# Define output paths
export DEB_PATH="$PWD/target/release/bundle/deb/blprnt_${CURRENT_VERSION}_amd64.deb"
export APPIMAGE_PATH="$PWD/target/release/bundle/appimage/blprnt_${CURRENT_VERSION}_amd64.AppImage"
export UPDATER_SIG_PATH="$PWD/target/release/bundle/appimage/blprnt_${CURRENT_VERSION}_amd64.AppImage.tar.gz.sig"

DEB_DEST="$PWD/bin/blprnt.deb"
APPIMAGE_DEST="$PWD/bin/blprnt.AppImage"
UPDATER_SIG_DEST="$PWD/bin/blprnt.AppImage.sig"

# Build frontend
echo "Building frontend..."
pnpm build

# Build Tauri bundles
echo "Building Tauri bundles..."
cargo tauri build --no-bundle && \
cargo tauri bundle --bundles deb

mkdir -p "$PWD/bin"

# Verify outputs
echo ""
echo "Build complete! Artifacts:"
if [ -f "$DEB_PATH" ]; then
  echo "  ✓ DEB: $DEB_PATH"
  cp "$DEB_PATH" "$DEB_DEST"
else
  echo "  ✗ DEB not found at $DEB_PATH"
fi

if [ -f "$APPIMAGE_PATH" ]; then
  echo "  ✓ AppImage: $APPIMAGE_PATH"
  cp "$APPIMAGE_PATH" "$APPIMAGE_DEST"
else
  echo "  ✗ AppImage not found at $APPIMAGE_PATH"
fi

if [ -f "$UPDATER_SIG_PATH" ]; then
  echo "  ✓ Signature: $UPDATER_SIG_PATH"
  cp "$UPDATER_SIG_PATH" "$UPDATER_SIG_DEST"
else
  echo "  ✗ Signature not found at $UPDATER_SIG_PATH"
fi
