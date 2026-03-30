#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <target-triple> <binary-path>" >&2
  exit 1
fi

TARGET_TRIPLE="$1"
BINARY_PATH="$2"

if [[ ! -f "$BINARY_PATH" ]]; then
  echo "binary not found: $BINARY_PATH" >&2
  exit 1
fi

case "$TARGET_TRIPLE" in
  aarch64-apple-darwin)
    DESTINATION="npm/darwin-arm64/blprnt"
    ;;
  x86_64-unknown-linux-gnu)
    DESTINATION="npm/linux-x64/blprnt"
    ;;
  x86_64-pc-windows-msvc)
    DESTINATION="npm/win32-x64/blprnt.exe"
    ;;
  *)
    echo "unsupported target triple: $TARGET_TRIPLE" >&2
    exit 1
    ;;
esac

mkdir -p "$(dirname "$DESTINATION")"
cp "$BINARY_PATH" "$DESTINATION"

if [[ "$DESTINATION" != *.exe ]]; then
  chmod 755 "$DESTINATION"
fi

echo "staged $BINARY_PATH -> $DESTINATION"
