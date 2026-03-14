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
  local tmp_root
  tmp_root="$(mktemp -d /tmp/download-ripgrep.XXXXXX)"
  local archive_path="${tmp_root}/${archive}"
  local extract_dir="${tmp_root}/extract"
  local extract_dir_python="$extract_dir"
  local binary_path
  
  echo "Downloading ${archive}..."
  
  curl -L "${BASE_URL}/${archive}" -o "$archive_path"

  echo "Extracting ${archive}..."
  mkdir -p "$extract_dir"
  if command -v cygpath >/dev/null 2>&1; then
    extract_dir_python="$(cygpath -w "$extract_dir")"
  fi
  if [[ "${archive}" == *.zip ]]; then
    ARCHIVE_PATH="$archive_path" EXTRACT_DIR="$extract_dir_python" python3 - <<'PY'
import os
import zipfile

archive = os.environ["ARCHIVE_PATH"]
extract_dir = os.environ["EXTRACT_DIR"]
with zipfile.ZipFile(archive, "r") as zf:
    zf.extractall(extract_dir)
PY
  else
    tar -xzf "$archive_path" -C "$extract_dir"
  fi
  
  if [[ "$target" == *"windows"* ]]; then
    echo "Moving Windows ripgrep binary from ${extract_dir} to ${OUT_DIR}/${target_name}..."
    mkdir -p "${OUT_DIR}"
    binary_path="$(find "$extract_dir" -type f \( -name 'rg.exe' -o -name 'ripgrep.exe' \) -print -quit)"
    if [[ -n "$binary_path" ]]; then
      mv "$binary_path" "${OUT_DIR}/${target_name}"
    else
      echo "Expected rg.exe or ripgrep.exe in ${extract_dir}, but neither exists"
      rm -rf "$tmp_root"
      exit 1
    fi
  else
    echo "Moving /tmp/${target}/rg to ${OUT_DIR}/${target_name}..."
    mkdir -p "${OUT_DIR}"
    binary_path="$(find "$extract_dir" -type f -name 'rg' -print -quit)"
    if [[ -z "$binary_path" ]]; then
      echo "Expected rg in ${extract_dir}, but none exists"
      rm -rf "$tmp_root"
      exit 1
    fi
    mv "$binary_path" "${OUT_DIR}/${target_name}"
    chmod +x "${OUT_DIR}/${target_name}"
  fi
  
  rm -rf "$extract_dir"
  rm -rf "$tmp_root"
  echo "  → ripgrep-${target}"
}

download_ripgrep "ripgrep-${VERSION}-aarch64-apple-darwin" "ripgrep-${VERSION}-aarch64-apple-darwin.tar.gz" "rg-aarch64-apple-darwin"
download_ripgrep "ripgrep-${VERSION}-x86_64-pc-windows-msvc" "ripgrep-${VERSION}-x86_64-pc-windows-msvc.zip" "rg-x86_64-pc-windows-msvc.exe"
download_ripgrep "ripgrep-${VERSION}-x86_64-unknown-linux-musl" "ripgrep-${VERSION}-x86_64-unknown-linux-musl.tar.gz" "rg-x86_64-unknown-linux-gnu"

echo "Done!"
