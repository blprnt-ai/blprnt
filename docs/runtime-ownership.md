# Runtime Ownership

The active runtime path in this repository is the `blprnt` binary plus its API,
coordinator, persistence, and tool crates:

- `crates/blprnt` boots the process
- `crates/api` exposes the runtime-facing HTTP surface and serves `dist/`
- `crates/coordinator` drives run scheduling and heartbeat execution
- `crates/persistence` owns the local SurrealDB repositories
- `crates/shared` and `crates/tools` hold the shared runtime and tool-dispatch helpers

Legacy orchestration code still exists on disk in `crates/engine_v2` and
`crates/providers`, but those crates are explicitly excluded from the active
Cargo workspace. They are archived reference code, not part of the current
build or release path.
