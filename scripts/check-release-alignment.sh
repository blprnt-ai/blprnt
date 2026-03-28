#!/usr/bin/env bash
set -euo pipefail

assert_absent() {
  local pattern="$1"
  shift

  if rg -n --fixed-strings -- "$pattern" "$@" >/dev/null; then
    echo "unexpected reference to '$pattern' in: $*"
    rg -n --fixed-strings -- "$pattern" "$@"
    exit 1
  fi
}

assert_present() {
  local pattern="$1"
  shift

  if ! rg -n --fixed-strings -- "$pattern" "$@" >/dev/null; then
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
