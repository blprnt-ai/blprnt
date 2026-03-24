# blprnt

`blprnt` is the Rust runtime behind the Paperclip control plane. The active executable lives at `crates/blprnt/src/main.rs`; when it starts, it launches the HTTP API, the coordinator heartbeat loop, and local persistence.

## What Runs Today

The live runtime path in this checkout is:

```text
blprnt binary -> API server + coordinator -> local SurrealDB
                    |
                    -> serves web assets from ./dist by default
```

The active crates in that path are:

- `crates/blprnt/` — binary entrypoint that boots the API and coordinator
- `crates/api/` — Axum routes, DTOs, issue/project/run APIs, and static asset serving
- `crates/coordinator/` — employee scheduling, run creation, and heartbeat-driven execution
- `crates/persistence/` — local SurrealDB connection and model repositories
- `crates/shared/` — shared paths, errors, tool schemas, and runtime helpers
- `crates/tools/` — file and host tool implementations used by agents

## Runtime Notes

- The API binds to `0.0.0.0:9171`.
- Persistence is local RocksDB-backed SurrealDB under `~/.blprnt/data`.
- Static assets are served from `./dist` unless `BLPRNT_BASE_DIR` overrides that path.
- `crates/engine_v2/` and `crates/providers/` still exist on disk, but they are not members of the active Cargo workspace and are not part of the current release path.

## Repository Layout

- `crates/` — Rust workspace members for the live runtime plus dormant crates that are not currently built
- `.github/workflows/release.yml` — tagged release workflow for platform archives
- `scripts/build-linux.sh` — local Linux archive build for the current runtime shape
- `scripts/build-windows.ps1` — local Windows archive build for the current runtime shape
- `scripts/build-macos.sh` — local macOS archive build for the current runtime shape
- `public/` — static files copied into the built web asset bundle
- `plans/` — engineering notes and baseline findings

## Development

Current local prerequisites:

- Rust `1.90.0`
- Node.js `22`
- `pnpm` `10.26.1`
- Python `3`
- PowerShell `7` for the local Windows archive helper

Useful commands:

- `./scripts/check-release-alignment.sh` — fail fast if release docs/scripts drift from the live runtime or if the required web entrypoint is missing
- `cargo check -p blprnt` — validate the live binary and its Rust dependencies
- `cargo test -p memory project_memory_service` — run the memory regression test called out in the CTO baseline
- `pnpm install --frozen-lockfile` — install the web build dependencies expected by the runtime
- `pnpm build` — build the `dist/` assets that the API serves at runtime
- `./scripts/build-linux.sh` — package a Linux release archive with the `blprnt` binary plus `dist/`
- `pwsh ./scripts/build-windows.ps1` — package a Windows release archive with `blprnt.exe` plus `dist/`
- `./scripts/build-macos.sh` — package a macOS release archive with the `blprnt` binary plus `dist/`

## Release Shape

Tagged GitHub releases now target the live runtime instead of a desktop bundle. Each platform job is expected to:

1. build `dist/`
2. build `cargo build --release -p blprnt`
3. publish an archive containing the release binary, `dist/`, `README.md`, and `LICENSE`

The local archive helpers mirror that shape:

- Linux: `./scripts/build-linux.sh`
- Windows: `pwsh ./scripts/build-windows.ps1`
- macOS: `./scripts/build-macos.sh`

## Current Validation Snapshot

Verified in this workspace on 2026-03-24:

- `pnpm build` succeeds and produces `dist/index.html` plus the bundled SPA assets.
- `./scripts/check-release-alignment.sh` passes.
- `cargo check -p blprnt` passes.

Operational note:

- The Rust runtime still requires `dist/index.html` at startup via `crates/api/src/routes/static_files.rs`.
- The current workspace also contains a large uncommitted frontend simplification, so treat the frontend shape here as a workspace-level change until that diff is reviewed and either committed or discarded.

## License

This repository is licensed under `BUSL-1.1`.

## Contributing

Contributions are welcome. See `CONTRIBUTING.md` for the lightweight pull request process.
