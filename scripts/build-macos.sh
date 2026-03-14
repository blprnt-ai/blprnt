#!/bin/bash
export APP_PATH="$PWD/target/release/bundle/macos/blprnt.app"
export DMG_PATH="$PWD/bin/blprnt.dmg"

pnpm build && \
cargo tauri build --no-bundle && \
cargo tauri bundle --bundles app && \
./scripts/make-dmg.sh
