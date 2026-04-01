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
assert_present "\"name\": \"@blprnt/blprnt\"" npm/blprnt/package.json
assert_present "\"bin\"" npm/blprnt/package.json
assert_present "\"optionalDependencies\"" npm/blprnt/package.json
assert_present "\"@blprnt/blprnt-darwin-arm64\"" npm/blprnt/package.json
assert_present "\"@blprnt/blprnt-linux-x64\"" npm/blprnt/package.json
assert_present "\"@blprnt/blprnt-win32-x64\"" npm/blprnt/package.json
assert_present "Unsupported platform" npm/blprnt/bin/blprnt.cjs
assert_present "Missing optional dependency" npm/blprnt/bin/blprnt.cjs
assert_present "\"name\": \"@blprnt/blprnt-darwin-arm64\"" npm/darwin-arm64/package.json
assert_present "\"name\": \"@blprnt/blprnt-linux-x64\"" npm/linux-x64/package.json
assert_present "\"name\": \"@blprnt/blprnt-win32-x64\"" npm/win32-x64/package.json
assert_present "\"dist/**/*\"" npm/darwin-arm64/package.json npm/linux-x64/package.json npm/win32-x64/package.json
assert_present "\"cpu\"" npm/darwin-arm64/package.json npm/linux-x64/package.json npm/win32-x64/package.json
assert_present "\"os\"" npm/darwin-arm64/package.json npm/linux-x64/package.json npm/win32-x64/package.json
assert_present "Upload Linux npm package artifact" .github/workflows/release.yml
assert_present "Upload Windows npm package artifact" .github/workflows/release.yml
assert_present "Upload macOS npm package artifact" .github/workflows/release.yml
assert_present "Publish npm packages" .github/workflows/release.yml
assert_present "npm publish ./npm/blprnt --access public" .github/workflows/release.yml
assert_present "npm publish ./npm/darwin-arm64 --access public" .github/workflows/release.yml
assert_present "npm publish ./npm/linux-x64 --access public" .github/workflows/release.yml
assert_present "npm publish ./npm/win32-x64 --access public" .github/workflows/release.yml

assert_script_retired_fails "./scripts/release.sh"
assert_script_retired_fails "./scripts/full-release.sh"
assert_script_retired_fails "./scripts/upload-dmg.sh"
assert_script_retired_fails "./scripts/sign-dmg.sh"
assert_script_retired_fails "./scripts/make-dmg.sh"

echo "release shape checks passed"
