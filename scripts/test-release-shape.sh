#!/usr/bin/env bash
set -euo pipefail

assert_present() {
  local pattern="$1"
  shift

  if ! rg -n --fixed-strings -- "$pattern" "$@" >/dev/null; then
    echo "missing expected reference to '$pattern' in: $*"
    exit 1
  fi
}

assert_script_retired_fails() {
  local script_path="$1"
  local output
  local status

  set +e
  output="$(bash "$script_path" 2>&1)"
  status=$?
  set -e

  if [[ $status -eq 0 ]]; then
    echo "expected retired script to fail: $script_path"
    exit 1
  fi

  if [[ "$output" != *retired* ]]; then
    echo "expected retired message from: $script_path"
    exit 1
  fi
}

[[ -f scripts/build-windows.ps1 ]] || {
  echo "missing expected file: scripts/build-windows.ps1"
  exit 1
}

assert_present "scripts/build-windows.ps1" README.md scripts/check-release-alignment.sh
assert_present "pwsh ./scripts/build-windows.ps1" README.md
assert_present "build-windows" .github/workflows/release.yml
assert_present "blprnt.exe" .github/workflows/release.yml scripts/build-windows.ps1
assert_present "dist" .github/workflows/release.yml scripts/build-windows.ps1
assert_present "pnpm install --frozen-lockfile" scripts/build-linux.sh scripts/build-macos.sh scripts/build-windows.ps1
assert_present "cargo fetch --locked" scripts/build-linux.sh scripts/build-macos.sh scripts/build-windows.ps1
assert_present "index.html" scripts/check-release-alignment.sh
assert_present "/src/main.tsx" scripts/check-release-alignment.sh
assert_present "retired" scripts/release.sh
assert_present "retired" scripts/full-release.sh
assert_present "retired" scripts/upload-dmg.sh
assert_present "retired" scripts/sign-dmg.sh
assert_present "retired" scripts/make-dmg.sh

assert_script_retired_fails "./scripts/release.sh"
assert_script_retired_fails "./scripts/full-release.sh"
assert_script_retired_fails "./scripts/upload-dmg.sh"
assert_script_retired_fails "./scripts/sign-dmg.sh"
assert_script_retired_fails "./scripts/make-dmg.sh"

echo "release shape checks passed"
