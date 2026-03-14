#!/bin/bash

pnpm build && \
cargo tauri build --no-bundle && \
cargo tauri bundle --bundles app
