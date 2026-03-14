#!/bin/bash
set -e

VERSION="0.1.0-alpha.1743007075"
BASE_URL="https://github.com/getgrit/gritql/releases/download/v${VERSION}"
OUT_DIR="tauri-src/binaries"

mkdir -p "$OUT_DIR"

download_grit() {
  local target=$1
  local archive=$2
  
  echo "Downloading ${archive}..."
  
  curl -L "${BASE_URL}/${archive}" -o "/tmp/${archive}"

  echo "Extracting ${archive}..."
  tar -xzf "/tmp/${archive}" -C /tmp
  
  if [[ "$target" == *"windows"* ]]; then
    echo "Moving /tmp/grit-${target}/grit.exe to ${OUT_DIR}/grit-${target}.exe..."
    mkdir -p "${OUT_DIR}"
    mv /tmp/grit-${target}/grit.exe "${OUT_DIR}/grit-${target}.exe"
  else
    echo "Moving /tmp/grit-${target}/grit to ${OUT_DIR}/grit-${target}..."
    mkdir -p "${OUT_DIR}"
    mv /tmp/grit-${target}/grit "${OUT_DIR}/grit-${target}"
    chmod +x "${OUT_DIR}/grit-${target}"
  fi
  
  rm "/tmp/${archive}"
  echo "  → grit-${target}"
}

download_grit "aarch64-apple-darwin" "grit-aarch64-apple-darwin.tar.gz"
download_grit "x86_64-pc-windows-msvc" "grit-x86_64-pc-windows-msvc.tar.gz"
download_grit "x86_64-unknown-linux-gnu" "grit-x86_64-unknown-linux-gnu.tar.gz"

echo "Done!"
