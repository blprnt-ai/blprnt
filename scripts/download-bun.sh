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
  local tmp_root
  tmp_root="$(mktemp -d /tmp/download-bun.XXXXXX)"
  local archive_path="${tmp_root}/${archive}.zip"
  local extract_dir="${tmp_root}/extract"
  local extract_dir_python="$extract_dir"
  local binary_path
  

  echo "Downloading: ${BASE_URL}/${archive}.zip"
  
  curl -L "${BASE_URL}/${archive}.zip" -o "$archive_path"

  echo "Extracting ${archive}.zip..."
  mkdir -p "$extract_dir"
  if command -v cygpath >/dev/null 2>&1; then
    extract_dir_python="$(cygpath -w "$extract_dir")"
  fi

  ARCHIVE_PATH="$archive_path" EXTRACT_DIR="$extract_dir_python" python3 - <<'PY'
import os
import zipfile

archive = os.environ["ARCHIVE_PATH"]
extract_dir = os.environ["EXTRACT_DIR"]
with zipfile.ZipFile(archive, "r") as zf:
    zf.extractall(extract_dir)
PY

  
  if [[ "$target" == *"windows"* ]]; then
    echo "Moving bun.exe from ${extract_dir} to ${OUT_DIR}/${target_name}..."
    mkdir -p "${OUT_DIR}"
    binary_path="$(find "$extract_dir" -type f -name 'bun.exe' -print -quit)"
    if [[ -z "$binary_path" ]]; then
      echo "Expected bun.exe in ${extract_dir}, but none exists"
      rm -rf "$tmp_root"
      exit 1
    fi
    mv "$binary_path" "${OUT_DIR}/${target_name}"
  else
    echo "Moving bun from ${extract_dir} to ${OUT_DIR}/${target_name}..."
    mkdir -p "${OUT_DIR}"
    binary_path="$(find "$extract_dir" -type f -name 'bun' -print -quit)"
    if [[ -z "$binary_path" ]]; then
      echo "Expected bun in ${extract_dir}, but none exists"
      rm -rf "$tmp_root"
      exit 1
    fi
    mv "$binary_path" "${OUT_DIR}/${target_name}"
    chmod +x "${OUT_DIR}/${target_name}"
  fi
  
  rm -rf "$tmp_root"
  echo "  → bun-${target}"
}

download_bun "aarch64-apple-darwin" "bun-darwin-aarch64" "bun-aarch64-apple-darwin"
download_bun "x86_64-pc-windows-msvc" "bun-windows-x64" "bun-x86_64-pc-windows-msvc.exe"
download_bun "x86_64-unknown-linux-musl" "bun-linux-x64-musl" "bun-x86_64-unknown-linux-gnu"

echo "Done!"
