#!/usr/bin/env bash
set -euo pipefail

TARGET="bin"
CURRENT_VERSION=$(grep '^version = ' tauri-src/Cargo.toml | head -n1 | cut -d '"' -f 2 || true)
if [ -z "$CURRENT_VERSION" ]; then
  echo "Error: No version found in tauri-src/Cargo.toml"
  exit 1
fi

PLAIN_VERSION="${CURRENT_VERSION}"

# Don't fail on no flag
if [[ -n "${1:-}" && "$1" == "--fnf" ]]; then
  # TARGET="bin-fnf"
  SHA=$(git rev-parse --short HEAD)
  SHA=${SHA:0:8}
  CURRENT_VERSION="${CURRENT_VERSION}-${SHA}"
fi

echo "Current version: $CURRENT_VERSION"

# Firebase Storage config
BUCKET="downloads.blprnt.ai"
BASE_URL="https://storage.googleapis.com/${BUCKET}/${TARGET}/${CURRENT_VERSION}"
BUCKET_PATH="gs://${BUCKET}/${TARGET}/${CURRENT_VERSION}"
OUTPUT=./latest.json

echo "Fetching assets from Firebase Storage: $BUCKET_PATH"

# List all files in the version folder
assets_raw=$(gsutil ls "${BUCKET_PATH}/" 2>/dev/null || echo "")

if [ -z "$assets_raw" ]; then
  echo "Error: No assets found in ${BUCKET_PATH}"
  exit 1
fi

echo "Found assets:"
echo "$assets_raw"

# Convert to JSON array with name and url
assets=$(echo "$assets_raw" | while read -r gs_path; do
  [ -z "$gs_path" ] && continue
  filename=$(basename "$gs_path")
  public_url="${BASE_URL}/${filename}"
  jq -n --arg name "$filename" --arg url "$public_url" '{name: $name, url: $url}'
done | jq -s '.')

asset_url_by_pattern() {
  local pattern="$1"
  echo "$assets" | jq -r --arg p "$pattern" '.[] | select(.name | test($p)) | .url' | head -n1
}

download_sig_content() {
  local sig_url="$1"
  [[ -z "$sig_url" ]] && { echo ""; return; }
  curl -sfL "$sig_url"
}

win_url="$(asset_url_by_pattern "\\.exe$")"
mac_url="$(asset_url_by_pattern "\\.app\\.tar\\.gz$")"
linux_url="$(asset_url_by_pattern "\\.AppImage$")"

win_sig_url="$(asset_url_by_pattern "\\.exe\\.sig$")"
mac_sig_url="$(asset_url_by_pattern "\\.app\\.tar\\.gz\\.sig$")"
linux_sig_url="$(asset_url_by_pattern "\\.AppImage\\.sig$")"

# remove storage.googleapis.com/ from the URLs but keep the https://
# But only if they're set
if [ -n "$win_url" ]; then
  win_url="${win_url#https://storage.googleapis.com/}"
  win_url="https://${win_url}"
fi
if [ -n "$mac_url" ]; then
  mac_url="${mac_url#https://storage.googleapis.com/}"
  mac_url="https://${mac_url}"
fi
if [ -n "$linux_url" ]; then
  linux_url="${linux_url#https://storage.googleapis.com/}"
  linux_url="https://${linux_url}"
fi
if [ -n "$win_sig_url" ]; then
  win_sig_url="${win_sig_url#https://storage.googleapis.com/}"
  win_sig_url="https://${win_sig_url}"
fi
if [ -n "$mac_sig_url" ]; then
  mac_sig_url="${mac_sig_url#https://storage.googleapis.com/}"
  mac_sig_url="https://${mac_sig_url}"
fi
if [ -n "$linux_sig_url" ]; then
  linux_sig_url="${linux_sig_url#https://storage.googleapis.com/}"
  linux_sig_url="https://${linux_sig_url}"
fi


echo "Windows URL: $win_url"
echo "Windows Sig URL: $win_sig_url"
echo "Mac URL: $mac_url"
echo "Mac Sig URL: $mac_sig_url"
echo "Linux URL: $linux_url"
echo "Linux Sig URL: $linux_sig_url"

win_sig="$(download_sig_content "$win_sig_url")"
mac_sig="$(download_sig_content "$mac_sig_url")"
linux_sig="$(download_sig_content "$linux_sig_url")"

pub_date=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Build latest.json
jq -n \
  --arg version "v${PLAIN_VERSION}" \
  --arg pub_date "$pub_date" \
  --arg win_url "$win_url" --arg win_sig "$win_sig" \
  --arg mac_url "$mac_url" --arg mac_sig "$mac_sig" \
  --arg linux_url "$linux_url" --arg linux_sig "$linux_sig" '
{
  version: $version,
  pub_date: $pub_date,
  notes: "",
  platforms: (
    {} +
    ( if ($win_url != "" and $win_sig != "") then
        { "windows-x86_64": { url: $win_url, signature: $win_sig } }
      else {} end
    ) +
    ( if ($mac_url != "" and $mac_sig != "") then
        { "darwin-aarch64": { url: $mac_url, signature: $mac_sig } }
      else {} end
    ) +
    ( if ($linux_url != "" and $linux_sig != "") then
        { "linux-x86_64":   { url: $linux_url, signature: $linux_sig } }
      else {} end
    )
  )
}
' > "$OUTPUT"

echo "Wrote $OUTPUT for v${CURRENT_VERSION}"
cat "$OUTPUT"

# Upload latest.json to Firebase Storage (version-specific and root)
echo "Uploading latest.json to ${BUCKET}/${TARGET}/latest.json"
gsutil cp "$OUTPUT" "gs://${BUCKET}/${TARGET}/latest.json"
gsutil setmeta -h "Cache-Control:public, max-age=1" "gs://${BUCKET}/${TARGET}/latest.json"

echo "Uploading latest.json to gs://${BUCKET}/${TARGET}/${CURRENT_VERSION}/latest.json (updater endpoint)"
gsutil cp "$OUTPUT" "gs://${BUCKET}/${TARGET}/${CURRENT_VERSION}/latest.json"
gsutil setmeta -h "Cache-Control:public, max-age=1" "gs://${BUCKET}/${TARGET}/${CURRENT_VERSION}/latest.json"

echo "✓ Release complete"
