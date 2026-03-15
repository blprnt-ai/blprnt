#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-${GITHUB_REF_NAME:-}}"
VERSION="${TAG#v}"
OUTPUT="${2:-./latest.json}"

if [[ -z "$TAG" ]]; then
  echo "Usage: $0 <vMAJOR.MINOR.PATCH> [output-path]"
  exit 1
fi

if [[ ! "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: tag must match vMAJOR.MINOR.PATCH"
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "Error: gh is required"
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required"
  exit 1
fi

release_json="$(gh release view "$TAG" --json assets,publishedAt)"
assets="$(echo "$release_json" | jq '.assets')"
pub_date="$(echo "$release_json" | jq -r '.publishedAt // empty')"

if [[ -z "$pub_date" ]]; then
  pub_date="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
fi

asset_url_by_pattern() {
  local pattern="$1"
  echo "$assets" | jq -r --arg p "$pattern" '[.[] | select(.name | test($p))][0].url // empty'
}

download_sig_content() {
  local sig_url="$1"
  curl -fsSL "$sig_url"
}

win_url="$(asset_url_by_pattern '\\.exe$')"
mac_url="$(asset_url_by_pattern '\\.app\\.tar\\.gz$')"

win_sig_url="$(asset_url_by_pattern '\\.exe\\.sig$')"
mac_sig_url="$(asset_url_by_pattern '\\.app\\.tar\\.gz\\.sig$')"

missing=()
[[ -n "$win_url" ]] || missing+=("windows .exe")
[[ -n "$win_sig_url" ]] || missing+=("windows .exe.sig")
[[ -n "$mac_url" ]] || missing+=("macOS .app.tar.gz")
[[ -n "$mac_sig_url" ]] || missing+=("macOS .app.tar.gz.sig")


if [[ "${#missing[@]}" -gt 0 ]]; then
  printf 'Error: missing release assets: %s\n' "${missing[*]}"
  exit 1
fi

win_sig="$(download_sig_content "$win_sig_url")"
mac_sig="$(download_sig_content "$mac_sig_url")"

jq -n \
  --arg version "$VERSION" \
  --arg pub_date "$pub_date" \
  --arg win_url "$win_url" --arg win_sig "$win_sig" \
  --arg mac_url "$mac_url" --arg mac_sig "$mac_sig" '
{
  version: $version,
  pub_date: $pub_date,
  notes: "",
  platforms: {
    "windows-x86_64": { url: $win_url, signature: $win_sig },
    "darwin-aarch64": { url: $mac_url, signature: $mac_sig }
  }
}

' > "$OUTPUT"

echo "Wrote $OUTPUT for $VERSION"
