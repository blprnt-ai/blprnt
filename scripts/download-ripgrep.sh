#!/bin/bash
set -e

VERSION="15.1.0"
BASE_URL="https://github.com/BurntSushi/ripgrep/releases/download/${VERSION}"
OUT_DIR="tauri-src/binaries"

mkdir -p "$OUT_DIR"

download_ripgrep() {
  local target=$1
  local archive=$2
  local target_name=$3
  
  echo "Downloading ${archive}..."
  
  curl -L "${BASE_URL}/${archive}" -o "/tmp/${archive}"

  echo "Extracting ${archive}..."
  if [[ "${archive}" == *.zip ]]; then
    ARCHIVE_PATH="/tmp/${archive}" python3 - <<'PY'
import os
import zipfile

archive = os.environ["ARCHIVE_PATH"]
with zipfile.ZipFile(archive, "r") as zf:
    zf.extractall("/tmp")
PY
  else
    tar -xzf "/tmp/${archive}" -C /tmp
  fi
  
  if [[ "$target" == *"windows"* ]]; then
    echo "Moving /tmp/${target}/ripgrep.exe to ${OUT_DIR}/rg-${target}.exe..."
    mkdir -p "${OUT_DIR}"
    mv /tmp/${target}/rg.exe "${OUT_DIR}/${target_name}"
  else
    echo "Moving /tmp/${target}/rg to ${OUT_DIR}/${target_name}..."
    mkdir -p "${OUT_DIR}"
    mv /tmp/${target}/rg "${OUT_DIR}/${target_name}"
    chmod +x "${OUT_DIR}/${target_name}"
  fi
  
  rm "/tmp/${archive}"
  echo "  → ripgrep-${target}"
}

download_ripgrep "ripgrep-${VERSION}-aarch64-apple-darwin" "ripgrep-${VERSION}-aarch64-apple-darwin.tar.gz" "rg-aarch64-apple-darwin"
download_ripgrep "ripgrep-${VERSION}-x86_64-pc-windows-msvc" "ripgrep-${VERSION}-x86_64-pc-windows-msvc.zip" "rg-x86_64-pc-windows-msvc.exe"
download_ripgrep "ripgrep-${VERSION}-x86_64-unknown-linux-musl" "ripgrep-${VERSION}-x86_64-unknown-linux-musl.tar.gz" "rg-x86_64-unknown-linux-gnu"

echo "Done!"
