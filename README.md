# blprnt

blprnt is a local AI execution runtime for technical teams.

It helps you turn a goal into a scoped plan, route work through specialist agents, execute in a real repository, and keep an auditable trail of issues, comments, tools, and artifacts.

## Quickstart

```bash
npx @blprnt/blprnt
```

That is the primary product entrypoint.

## What blprnt is for

blprnt is built for teams that want more than one-off chat output.

Use it when you want AI work to look more like a delivery system:

- plans before edits
- explicit ownership and specialist roles
- repo-aware execution
- durable issue state and comments
- inspectable tool history
- local control over runtime behavior and files

## How it works

At a high level:

1. define the work in issues
2. assign the right employee or specialist
3. let the runtime execute against the real repo
4. review the resulting plans, comments, file changes, and handoffs

The live runtime shape in this repository is:

```text
blprnt binary -> API server + coordinator -> local SurrealDB
                    |
                    -> serves web assets from ./dist by default
```

## Why teams use it

### Plan-first execution

blprnt is designed to make planning part of the workflow instead of an afterthought.

### Specialist orchestration

Work can be routed through role-specific employees with bounded responsibilities instead of pushing every task through one general-purpose assistant.

### Local, inspectable operation

The runtime operates on local project state and keeps execution legible enough to review after the run ends.

### Durable workflow memory

Issues, comments, project memory, and plans give the work continuity across runs.

## Run path

The intended user path is:

1. run `npx @blprnt/blprnt`
2. open the local runtime
3. configure your project and employees
4. create or pick an issue
5. execute work through the runtime

## Repository map

- `crates/blprnt/` — binary entrypoint
- `crates/api/` — HTTP API, DTOs, and static asset serving
- `crates/coordinator/` — employee scheduling and run execution
- `crates/persistence/` — local SurrealDB-backed persistence
- `crates/shared/` — shared runtime helpers and schemas
- `crates/tools/` — file and host tool implementations
- `npm/blprnt` — `@blprnt/blprnt` wrapper package used by `npx`; ships the launcher plus the shared `dist/` SPA bundle
- `npm/darwin-arm64`, `npm/linux-x64`, `npm/win32-x64` — platform packages; each ships the platform executable plus platform-specific `tools/rg`

## Development

Current local prerequisites:

- Rust `1.90.0`
- Node.js `22`
- `pnpm` `10.26.1`
- Python `3`
- PowerShell `7` for the Windows archive helper

Useful commands:

- `pnpm install --frozen-lockfile`
- `pnpm build`
- `cargo check -p blprnt`
- `./scripts/check-release-alignment.sh`
- `./scripts/build-linux.sh`
- `pwsh ./scripts/build-windows.ps1`
- `./scripts/build-macos.sh`

## Packaging and npx

The npm wrapper layout ships blprnt as `@blprnt/blprnt` plus platform-specific packages.

The intended invocation remains:

```bash
npx @blprnt/blprnt
```

Tagged release CI is expected to publish the three platform packages first, wait briefly for npm propagation, then publish the wrapper package with the shared `dist/` bundle.

## Runtime notes

- The API binds to `0.0.0.0:9171`.
- Persistence is local RocksDB-backed SurrealDB under `~/.blprnt/data`.
- Static assets are served from `BLPRNT_BASE_DIR` when set, otherwise from `dist/` beside the installed executable, with `./dist` as the local dev fallback.

## License

This repository is licensed under `BUSL-1.1`.

## Contributing

Contributions are welcome. See `CONTRIBUTING.md` for the lightweight pull request process.
