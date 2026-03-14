#!/bin/bash
set -e

VERSION="0.1.0-alpha.1743007075"
BASE_URL="https://github.com/getgrit/gritql/releases/download/v${VERSION}"
OUT_DIR="tauri-src/binaries"

mkdir -p "$OUT_DIR"

download_grit() {
  local target=$1
  local archive=$2
  local tmp_root
  tmp_root="$(mktemp -d /tmp/download-grit.XXXXXX)"
  local archive_path="${tmp_root}/${archive}"
  local extract_dir="${tmp_root}/extract"
  local binary_path
  
  echo "Downloading ${archive}..."
  
  curl -L "${BASE_URL}/${archive}" -o "$archive_path"

  echo "Extracting ${archive}..."
  mkdir -p "$extract_dir"
  tar -xzf "$archive_path" -C "$extract_dir"
  
  if [[ "$target" == *"windows"* ]]; then
    echo "Moving grit.exe from ${extract_dir} to ${OUT_DIR}/grit-${target}.exe..."
    mkdir -p "${OUT_DIR}"
    binary_path="$(find "$extract_dir" -type f -name 'grit.exe' -print -quit)"
    if [[ -z "$binary_path" ]]; then
      echo "Expected grit.exe in ${extract_dir}, but none exists"
      rm -rf "$tmp_root"
      exit 1
    fi
    mv "$binary_path" "${OUT_DIR}/grit-${target}.exe"
  else
    echo "Moving grit from ${extract_dir} to ${OUT_DIR}/grit-${target}..."
    mkdir -p "${OUT_DIR}"
    binary_path="$(find "$extract_dir" -type f -name 'grit' -print -quit)"
    if [[ -z "$binary_path" ]]; then
      echo "Expected grit in ${extract_dir}, but none exists"
      rm -rf "$tmp_root"
      exit 1
    fi
    mv "$binary_path" "${OUT_DIR}/grit-${target}"
    chmod +x "${OUT_DIR}/grit-${target}"
  fi
  
  rm -rf "$tmp_root"
  echo "  → grit-${target}"
}

download_grit "aarch64-apple-darwin" "grit-aarch64-apple-darwin.tar.gz"
download_grit "x86_64-pc-windows-msvc" "grit-x86_64-pc-windows-msvc.tar.gz"
download_grit "x86_64-unknown-linux-gnu" "grit-x86_64-unknown-linux-gnu.tar.gz"

echo "Done!"
