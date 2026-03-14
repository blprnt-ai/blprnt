#!/bin/bash
set -e

VERSION="3.0.0"
BASE_URL="https://github.com/surrealdb/surrealdb/releases/download/v${VERSION}"
OUT_DIR="tauri-src/binaries"

mkdir -p "$OUT_DIR"

download() {
  local target=$1
  local archive=$2
  
  echo "Downloading ${archive}..."
  
  curl -L "${BASE_URL}/${archive}" -o "/tmp/${archive}"

  
  if [[ "$target" == *"windows"* ]]; then
    echo "Moving /tmp/${archive} to ${OUT_DIR}/surreal-${target}.exe..."
    mkdir -p "${OUT_DIR}"
    mv /tmp/${archive} "${OUT_DIR}/surreal-${target}.exe"
  else
    echo "Extracting ${archive}..."
    tar -xzf "/tmp/${archive}" -C /tmp
    echo "Moving /tmp/surreal to ${OUT_DIR}/surreal-${target}..."
    mkdir -p "${OUT_DIR}"
    mv /tmp/surreal "${OUT_DIR}/surreal-${target}"
    chmod +x "${OUT_DIR}/surreal-${target}"
    rm "/tmp/${archive}"
  fi
  
  echo "  → surreal-${target}"
}

download "aarch64-apple-darwin" "surreal-v3.0.0.darwin-arm64.tgz"
download "x86_64-pc-windows-msvc" "surreal-v3.0.0.windows-amd64.exe"
download "x86_64-unknown-linux-gnu" "surreal-v3.0.0.linux-amd64.tgz"

echo "Done!"
