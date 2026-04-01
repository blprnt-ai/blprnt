#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW_PATH="${ACT_WORKFLOW:-.github/workflows/release.yml}"
EVENT_NAME="${ACT_EVENT:-push}"
DEFAULT_JOB="${ACT_JOB:-prepare}"
CONTAINER_ARCH="${ACT_CONTAINER_ARCH:-linux/amd64}"
IMAGE="${ACT_IMAGE:-ghcr.io/catthehacker/ubuntu:act-latest}"
SECRETS_FILE="${ACT_SECRETS_FILE:-.secrets.act}"
JOB_NAME="$DEFAULT_JOB"

usage() {
  cat <<'EOF'
Usage:
  ./scripts/run-gha-local.sh list
  ./scripts/run-gha-local.sh prepare
  ./scripts/run-gha-local.sh build-linux
  ./scripts/run-gha-local.sh run <job>
  ./scripts/run-gha-local.sh raw -- <act args...>

Notes:
  - This wrapper targets .github/workflows/release.yml by default.
  - Local act runs are useful for prepare/build-linux validation.
  - macOS and Windows jobs in GitHub Actions are not faithfully reproduced by act.
  - By default this simulates a tag push using the version from crates/blprnt/Cargo.toml.
  - Set GH_TOKEN in the environment or create .secrets.act with lines like:
      GH_TOKEN=ghp_xxx

Environment overrides:
  ACT_WORKFLOW=.github/workflows/release.yml
  ACT_EVENT=push
  ACT_JOB=prepare
  ACT_TAG=v2.0.0-alpha.1
  ACT_CONTAINER_ARCH=linux/amd64
  ACT_IMAGE=ghcr.io/catthehacker/ubuntu:act-latest
  ACT_SECRETS_FILE=.secrets.act
EOF
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1"
    exit 1
  fi
}

warn_if_limited_job() {
  case "$1" in
    build-macos|build-windows|npm-publish)
      echo "warning: '$1' depends on hosted-runner behavior and is not a reliable local act target" >&2
      ;;
  esac
}

require_cmd act
require_cmd docker

cd "$ROOT_DIR"

if [[ -n "${ACT_TAG:-}" ]]; then
  TAG_NAME="$ACT_TAG"
else
  APP_VERSION="$(sed -nE 's/^version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' crates/blprnt/Cargo.toml | head -n1)"
  if [[ -z "$APP_VERSION" ]]; then
    echo "could not determine version from crates/blprnt/Cargo.toml"
    exit 1
  fi
  TAG_NAME="v$APP_VERSION"
fi

ACT_EVENT_FILE="$(mktemp "${TMPDIR:-/tmp}/act-release-event.XXXXXX")"
cleanup() {
  rm -f "$ACT_EVENT_FILE"
}
trap cleanup EXIT

cat > "$ACT_EVENT_FILE" <<EOF
{
  "ref": "refs/tags/$TAG_NAME",
  "ref_name": "$TAG_NAME",
  "repository": {
    "full_name": "local/blprnt"
  }
}
EOF

case "${1:-}" in
  ""|-h|--help|help)
    usage
    exit 0
    ;;
  list)
    exec act -W "$WORKFLOW_PATH" -l
    ;;
  prepare|build-linux|build-macos|build-windows|npm-publish)
    JOB_NAME="$1"
    ;;
  run)
    if [[ $# -lt 2 ]]; then
      usage
      exit 1
    fi
    JOB_NAME="$2"
    shift
    shift
    set -- "$@"
    ;;
  raw)
    shift
    if [[ "${1:-}" == "--" ]]; then
      shift
    fi
    exec act "$@"
    ;;
  *)
    usage
    exit 1
    ;;
esac

warn_if_limited_job "$JOB_NAME"

ACT_ARGS=(
  "$EVENT_NAME"
  -W "$WORKFLOW_PATH"
  -j "$JOB_NAME"
  -e "$ACT_EVENT_FILE"
  --container-architecture "$CONTAINER_ARCH"
  -P "ubuntu-latest=$IMAGE"
  -P "ubuntu-22.04=$IMAGE"
)

if [[ -f "$ROOT_DIR/$SECRETS_FILE" ]]; then
  ACT_ARGS+=(--secret-file "$SECRETS_FILE")
fi

if [[ -n "${GH_TOKEN:-}" ]]; then
  ACT_ARGS+=(--secret "GH_TOKEN=$GH_TOKEN")
fi

exec act "${ACT_ARGS[@]}"
