#!/bin/bash

./scripts/build-macos.sh && \
./scripts/upload-dmg.sh

if [ "$1" == "-s" ] || [ "$1" == "--sleep" ] && [ "$2" -eq "$2" ] 2>/dev/null; then
  echo "Sleeping for $2 seconds"
  sleep "$2"
fi

./scripts/release.sh