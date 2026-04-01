#!/usr/bin/env bash
set -euo pipefail

CURRENT_VERSION="$(
  python3 - <<'PY'
from pathlib import Path
import re

text = Path("crates/blprnt/Cargo.toml").read_text()
match = re.search(r'^version\s*=\s*"([^"]+)"', text, re.MULTILINE)
if not match:
    raise SystemExit("missing version in crates/blprnt/Cargo.toml")
print(match.group(1))
PY
)"

TARGET_TRIPLE="x86_64-unknown-linux-gnu"
RELEASE_STEM="blprnt-v${CURRENT_VERSION}-linux-x86_64"
PACKAGE_DIR="$PWD/bin/$RELEASE_STEM"
ARCHIVE_PATH="$PWD/bin/$RELEASE_STEM.tar.gz"

./scripts/check-release-alignment.sh

echo "Building blprnt v${CURRENT_VERSION} for Linux..."
pnpm install --frozen-lockfile
pnpm build
cargo fetch --locked
cargo build --release --locked -p blprnt --target "$TARGET_TRIPLE"

rm -rf "$PACKAGE_DIR"
mkdir -p "$PACKAGE_DIR"

cp "target/$TARGET_TRIPLE/release/blprnt" "$PACKAGE_DIR/blprnt"
cp -R dist "$PACKAGE_DIR/dist"
./scripts/fetch-ripgrep.sh "$TARGET_TRIPLE" "$PACKAGE_DIR/tools/rg"
cp README.md LICENSE "$PACKAGE_DIR/"

tar -C "$PWD/bin" -czf "$ARCHIVE_PATH" "$RELEASE_STEM"

echo "Packaged $ARCHIVE_PATH"
