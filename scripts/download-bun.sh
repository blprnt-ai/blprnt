#!/bin/bash
set -e

VERSION="1.3.10"
BASE_URL="https://github.com/oven-sh/bun/releases/download/bun-v${VERSION}"
OUT_DIR="tauri-src/binaries"

mkdir -p "$OUT_DIR"

download_bun() {
  local target=$1
  local archive=$2
  local target_name=$3
  

  echo "Downloading: ${BASE_URL}/${archive}.zip"
  
  curl -L "${BASE_URL}/${archive}.zip" -o "/tmp/${archive}.zip"

  echo "Extracting ${archive}.zip..."

  ARCHIVE_PATH="/tmp/${archive}.zip" python3 - <<'PY'
import os
import zipfile

archive = os.environ["ARCHIVE_PATH"]
with zipfile.ZipFile(archive, "r") as zf:
    zf.extractall("/tmp")
PY

  
  if [[ "$target" == *"windows"* ]]; then
    echo "Moving /tmp/${archive}/bun.exe to ${OUT_DIR}/${target_name}..."
    mkdir -p "${OUT_DIR}"
    mv /tmp/${archive}/bun.exe "${OUT_DIR}/${target_name}"
  else
    echo "Moving /tmp/${archive}/bun to ${OUT_DIR}/${target_name}..."
    mkdir -p "${OUT_DIR}"
    mv /tmp/${archive}/bun "${OUT_DIR}/${target_name}"
    chmod +x "${OUT_DIR}/${target_name}"
  fi
  
  rm -r "/tmp/${archive}"
  rm "/tmp/${archive}.zip"
  echo "  → bun-${target}"
}

download_bun "aarch64-apple-darwin" "bun-darwin-aarch64" "bun-aarch64-apple-darwin"
download_bun "x86_64-pc-windows-msvc" "bun-windows-x64" "bun-x86_64-pc-windows-msvc.exe"
download_bun "x86_64-unknown-linux-musl" "bun-linux-x64-musl" "bun-x86_64-unknown-linux-gnu"

echo "Done!"
