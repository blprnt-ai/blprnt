#!/usr/bin/env bash
set -euo pipefail

search_fixed() {
  if command -v rg >/dev/null 2>&1; then
    rg -n --fixed-strings -- "$@"
  else
    grep -Fn -- "$@"
  fi
}

assert_absent() {
  local pattern="$1"
  shift

  if search_fixed "$pattern" "$@" >/dev/null; then
    echo "unexpected reference to '$pattern' in: $*"
    search_fixed "$pattern" "$@"
    exit 1
  fi
}

assert_present() {
  local pattern="$1"
  shift

  if ! search_fixed "$pattern" "$@" >/dev/null; then
    echo "missing expected reference to '$pattern' in: $*"
    exit 1
  fi
}

assert_absent "tauri-src/" README.md .github/workflows/release.yml scripts/build-linux.sh scripts/build-macos.sh scripts/build-windows.ps1
assert_absent "cargo tauri" .github/workflows/release.yml scripts/build-linux.sh scripts/build-macos.sh scripts/build-windows.ps1
assert_absent "latest.json" .github/workflows/release.yml scripts/release.sh
assert_absent "crates/app_core/" README.md

assert_present "crates/blprnt/Cargo.toml" .github/workflows/release.yml scripts/build-linux.sh scripts/build-macos.sh scripts/build-windows.ps1
assert_present "cargo build --release" .github/workflows/release.yml scripts/build-linux.sh scripts/build-macos.sh scripts/build-windows.ps1
assert_present "-p blprnt" .github/workflows/release.yml scripts/build-linux.sh scripts/build-macos.sh scripts/build-windows.ps1
assert_present "blprnt.exe" .github/workflows/release.yml scripts/build-windows.ps1
assert_present "dist" .github/workflows/release.yml scripts/build-linux.sh scripts/build-macos.sh scripts/build-windows.ps1
assert_present "pwsh ./scripts/build-windows.ps1" README.md
assert_present "stage-npm-binary.sh" .github/workflows/release.yml README.md
assert_present "\"dist/**/*\"" npm/darwin-arm64/package.json npm/linux-x64/package.json npm/win32-x64/package.json
assert_present "\"dist\"" .github/workflows/release.yml
assert_present "npm publish ./npm/blprnt --access public" .github/workflows/release.yml
assert_present "npm publish ./npm/darwin-arm64 --access public" .github/workflows/release.yml
assert_present "npm publish ./npm/linux-x64 --access public" .github/workflows/release.yml
assert_present "npm publish ./npm/win32-x64 --access public" .github/workflows/release.yml
assert_present "/src/main.tsx" index.html
assert_present "retired" scripts/release.sh
assert_present "retired" scripts/full-release.sh
assert_present "retired" scripts/upload-dmg.sh
assert_present "retired" scripts/sign-dmg.sh
assert_present "retired" scripts/make-dmg.sh

[[ -f index.html ]] || {
  echo "missing expected frontend entrypoint manifest: index.html"
  exit 1
}

[[ -f src/main.tsx ]] || {
  echo "missing expected frontend entrypoint: src/main.tsx"
  echo "index.html still references /src/main.tsx, so release archives cannot produce the dist/ bundle the API serves."
  exit 1
}

echo "release alignment checks passed"
