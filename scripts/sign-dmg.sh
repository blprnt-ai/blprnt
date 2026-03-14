#!/usr/bin/env bash
set -euo pipefail


if [[ -z "${APP_PATH:-}" ]]; then
  echo "APP_PATH is not set"
  exit 1
fi

if [[ -z "${DMG_PATH:-}" ]]; then
  echo "DMG_PATH is not set"
  exit 1
fi

VOLUME_NAME=blprnt
CODESIGN_IDENTITY="${CODESIGN_IDENTITY:-}"

ENTITLEMENTS="${ENTITLEMENTS:-}"
APPLE_ID="${APPLE_ID:-}"
TEAM_ID="${TEAM_ID:-}"


die() { echo "error: $*" >&2; exit 1; }

require() { command -v "$1" >/dev/null 2>&1 || die "missing tool: $1"; }

main() {
  if [[ ! -d "${APP_PATH}" ]]; then
    echo "APP_PATH does not exist: ${APP_PATH}"
    exit 1
  fi

  if [[ ! -f "${DMG_PATH}" ]]; then
    echo "DMG_PATH does not exist: ${DMG_PATH}"
    exit 1
  fi

  require security
  require codesign
  require spctl
  require xcrun
  require hdiutil

  if [[ -z "${CODESIGN_IDENTITY}" ]]; then
    CODESIGN_IDENTITY="$(security find-identity -v -p codesigning \
      | awk -F\" '/Developer ID Application:/ {print $2; exit}')"
    [[ -n "${CODESIGN_IDENTITY}" ]] || die "no 'Developer ID Application' identity found in login keychain"
  fi

  echo "Using identity: ${CODESIGN_IDENTITY}"

  sign_bundle_executables
  sign_app
  sign_dmg
  notarize_dmg
  staple_dmg
  verify_dmg
  echo "DMG notarized and stapled: $DMG_PATH"
}

sign_bundle_executables() {
  echo "Signing executable files in bundle…"

  while IFS= read -r -d '' executable; do
    codesign --force --options runtime --timestamp --sign "$CODESIGN_IDENTITY" "$executable"
  done < <(find "$APP_PATH" -type f -perm -111 -print0)
}

sign_app() {
  echo "Signing app bundle: $APP_PATH"
  if [[ -n "${ENTITLEMENTS}" ]]; then
    codesign --force --options runtime --timestamp --sign "$CODESIGN_IDENTITY" \
      --entitlements "${ENTITLEMENTS}" "$APP_PATH"
  else
    codesign --force --options runtime --timestamp --sign "$CODESIGN_IDENTITY" \
      "$APP_PATH"
  fi

  echo "Verifying app signature…"
  codesign --verify --verbose=2 "$APP_PATH"
  spctl --assess --type execute --verbose=4 "$APP_PATH" || true
}

sign_dmg() {
  echo "Signing DMG: $DMG_PATH"
  codesign --force --timestamp --sign "$CODESIGN_IDENTITY" "$DMG_PATH"
}

verify_dmg() {
  echo "Verifying DMG signature…"
  codesign --verify --verbose=2 "$DMG_PATH"
  xcrun stapler validate "$DMG_PATH"
}

notarize_dmg() {
  echo "Submitting to notarization…"
  xcrun notarytool submit "$DMG_PATH" --keychain-profile blprnt --wait
}

staple_dmg() {
  echo "Stapling notarization ticket…"
  xcrun stapler staple "$DMG_PATH"
  xcrun stapler validate "$DMG_PATH"
}

main "$@"
