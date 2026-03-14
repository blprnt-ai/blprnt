#!/bin/bash
export APP_PATH="$PWD/target/release/bundle/macos/blprnt.app"
export APP_UPDATE_PATH="$PWD/target/release/bundle/macos/blprnt.app.tar.gz"
export APP_UPDATE_SIG_PATH="$PWD/target/release/bundle/macos/blprnt.app.tar.gz.sig"
export DMG_PATH="$PWD/bin/blprnt.dmg"

echo "APP_PATH: $APP_PATH"
echo "DMG_PATH: $DMG_PATH"

CURRENT_VERSION=$(grep '^version = ' tauri-src/Cargo.toml | head -n1 | cut -d '"' -f 2 || true)
if [ -z "$CURRENT_VERSION" ]; then
  echo "Error: No version found in tauri-src/Cargo.toml"
  exit 1
fi

TARGET="bin"

if [[ -n "${1:-}" && "$1" == "--fnf" ]]; then
  TARGET="bin-fnf"
  SHA=$(git rev-parse --short HEAD)
  SHA=${SHA:0:8}
  CURRENT_VERSION="${CURRENT_VERSION}-${SHA}"
fi

echo "Current version: $CURRENT_VERSION"

# Firebase Storage config
BUCKET="downloads.blprnt.ai"
BUCKET_PATH="gs://${BUCKET}/${TARGET}/${CURRENT_VERSION}"

echo "Uploading to Firebase Storage: $BUCKET_PATH"

./scripts/sign-dmg.sh && \
gsutil cp "$DMG_PATH" "${BUCKET_PATH}/blprnt.dmg" && \
gsutil setmeta -h "Cache-Control:public, max-age=31536000" "${BUCKET_PATH}/blprnt.dmg" && \
gsutil cp "$APP_UPDATE_PATH" "${BUCKET_PATH}/blprnt.app.tar.gz" && \
gsutil setmeta -h "Cache-Control:public, max-age=31536000" "${BUCKET_PATH}/blprnt.app.tar.gz" && \
gsutil cp "$APP_UPDATE_SIG_PATH" "${BUCKET_PATH}/blprnt.app.tar.gz.sig" && \
gsutil setmeta -h "Cache-Control:public, max-age=31536000" "${BUCKET_PATH}/blprnt.app.tar.gz.sig"

echo "✓ Upload complete"
