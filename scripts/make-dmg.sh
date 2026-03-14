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

main() {
  echo "Creating DMG: $DMG_PATH"
  python3 -m venv /tmp/venv && \
  source /tmp/venv/bin/activate && \
  python3 -m pip install --upgrade pip && \
  python3 -m pip install dmgbuild && \
  dmgbuild -s $PWD/scripts/make-dmg.py "blprnt" "$DMG_PATH"
  deactivate
  rm -rf /tmp/venv
  echo "DMG created: $DMG_PATH"
}

main "$@"
