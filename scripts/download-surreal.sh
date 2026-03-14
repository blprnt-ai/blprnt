#!/bin/bash
set -e

VERSION="3.0.0"
BASE_URL="https://github.com/surrealdb/surrealdb/releases/download/v${VERSION}"
OUT_DIR="tauri-src/binaries"

mkdir -p "$OUT_DIR"

download() {
  local target=$1
  local archive=$2
  local tmp_root
  tmp_root="$(mktemp -d /tmp/download-surreal.XXXXXX)"
  local archive_path="${tmp_root}/${archive}"
  local extract_dir="${tmp_root}/extract"
  local binary_path
  
  echo "Downloading ${archive}..."
  
  curl -L "${BASE_URL}/${archive}" -o "$archive_path"

  
  if [[ "$target" == *"windows"* ]]; then
    echo "Moving ${archive_path} to ${OUT_DIR}/surreal-${target}.exe..."
    mkdir -p "${OUT_DIR}"
    mv "$archive_path" "${OUT_DIR}/surreal-${target}.exe"
  else
    echo "Extracting ${archive}..."
    mkdir -p "$extract_dir"
    tar -xzf "$archive_path" -C "$extract_dir"
    echo "Moving surreal from ${extract_dir} to ${OUT_DIR}/surreal-${target}..."
    mkdir -p "${OUT_DIR}"
    binary_path="$(find "$extract_dir" -type f -name 'surreal' -print -quit)"
    if [[ -z "$binary_path" ]]; then
      echo "Expected surreal in ${extract_dir}, but none exists"
      rm -rf "$tmp_root"
      exit 1
    fi
    mv "$binary_path" "${OUT_DIR}/surreal-${target}"
    chmod +x "${OUT_DIR}/surreal-${target}"
  fi
  
  rm -rf "$tmp_root"
  echo "  → surreal-${target}"
}

download "aarch64-apple-darwin" "surreal-v3.0.0.darwin-arm64.tgz"
download "x86_64-pc-windows-msvc" "surreal-v3.0.0.windows-amd64.exe"
download "x86_64-unknown-linux-gnu" "surreal-v3.0.0.linux-amd64.tgz"

echo "Done!"
