#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <target-triple> <output-path>" >&2
  exit 1
fi

VERSION="15.1.0"
BASE_URL="https://github.com/BurntSushi/ripgrep/releases/download/${VERSION}"
TARGET_TRIPLE="$1"
OUTPUT_PATH="$2"

case "$TARGET_TRIPLE" in
  aarch64-apple-darwin)
    ASSET_STEM="ripgrep-${VERSION}-aarch64-apple-darwin"
    ARCHIVE_EXT="tar.gz"
    ;;
  x86_64-apple-darwin)
    ASSET_STEM="ripgrep-${VERSION}-x86_64-apple-darwin"
    ARCHIVE_EXT="tar.gz"
    ;;
  x86_64-pc-windows-msvc)
    ASSET_STEM="ripgrep-${VERSION}-x86_64-pc-windows-msvc"
    ARCHIVE_EXT="zip"
    ;;
  x86_64-unknown-linux-gnu|x86_64-unknown-linux-musl)
    ASSET_STEM="ripgrep-${VERSION}-x86_64-unknown-linux-musl"
    ARCHIVE_EXT="tar.gz"
    ;;
  *)
    echo "unsupported ripgrep target: ${TARGET_TRIPLE}" >&2
    exit 1
    ;;
esac

ARCHIVE_NAME="${ASSET_STEM}.${ARCHIVE_EXT}"
TMP_ROOT="$(mktemp -d /tmp/fetch-ripgrep.XXXXXX)"
ARCHIVE_PATH="${TMP_ROOT}/${ARCHIVE_NAME}"
EXTRACT_DIR="${TMP_ROOT}/extract"

cleanup() {
  rm -rf "$TMP_ROOT"
}
trap cleanup EXIT

mkdir -p "$EXTRACT_DIR" "$(dirname "$OUTPUT_PATH")"

echo "Downloading ${ARCHIVE_NAME}..."
curl -fsSL "${BASE_URL}/${ARCHIVE_NAME}" -o "$ARCHIVE_PATH"

echo "Extracting ${ARCHIVE_NAME}..."
if [[ "$ARCHIVE_EXT" == "zip" ]]; then
  ARCHIVE_PATH="$ARCHIVE_PATH" EXTRACT_DIR="$EXTRACT_DIR" python3 - <<'PY'
import os
import zipfile

with zipfile.ZipFile(os.environ["ARCHIVE_PATH"], "r") as zf:
    zf.extractall(os.environ["EXTRACT_DIR"])
PY
else
  tar -xzf "$ARCHIVE_PATH" -C "$EXTRACT_DIR"
fi

if [[ "$TARGET_TRIPLE" == *windows* ]]; then
  BINARY_NAME="rg.exe"
else
  BINARY_NAME="rg"
fi

BINARY_PATH="$(find "$EXTRACT_DIR" -type f -name "$BINARY_NAME" -print -quit)"
if [[ -z "$BINARY_PATH" ]]; then
  echo "expected ${BINARY_NAME} in ${ARCHIVE_NAME}, but none was found" >&2
  exit 1
fi

mv "$BINARY_PATH" "$OUTPUT_PATH"
if [[ "$TARGET_TRIPLE" != *windows* ]]; then
  chmod +x "$OUTPUT_PATH"
fi

echo "Bundled ripgrep at $OUTPUT_PATH"
