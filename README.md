# blprnt

blprnt is a local AI execution runtime for technical teams.

It helps you turn a goal into a scoped plan, route work through specialist agents, execute in a real repository, and keep an auditable trail of issues, comments, tools, and artifacts.

## Quickstart

```bash
npx @blprnt/blprnt
```

That is the primary product entrypoint.

## What blprnt is for

blprnt is built for teams that want AI work to behave like an execution system, not a one-shot chat session.

Use it when you want:

- plans before edits
- explicit ownership and specialist roles
- execution grounded in a real repository
- durable issues, comments, and handoffs
- inspectable tool usage and file changes
- local control over runtime behavior and data

## What you get

### Plan-first execution

blprnt turns work into issues, plans, and tracked runs before broad code changes start.

### Specialist orchestration

You can route tasks through role-specific employees instead of forcing every job through one general assistant.

### Repo-aware delivery

Agents operate against the actual local project, so work stays grounded in your files, structure, and constraints.

### Auditable workflow state

Issues, comments, plans, attachments, and memory make it easy to inspect what happened and why.

### Local runtime control

The runtime executes locally and keeps product, project, and employee state close to the team using it.

## How to use it

Typical flow:

1. run `npx @blprnt/blprnt`
2. open the local runtime UI
3. connect or configure your project
4. create issues for the work you want done
5. assign the right employee or specialist
6. review the resulting plans, comments, edits, and handoffs

## Product overview

At a high level, blprnt provides:

- a local runtime
- an issue-driven execution model
- employee and specialist orchestration
- project and employee memory
- an inspectable API and tool trail

The live runtime shape in this repository is:

```text
blprnt binary -> API server + coordinator -> local SurrealDB
                    |
                    -> serves web assets from ./dist by default
```

## Requirements

For normal usage, the primary entrypoint is:

```bash
npx @blprnt/blprnt
```

For local development in this repository, current prerequisites are:

- Rust `1.90.0`
- Node.js `22`
- `pnpm` `10.26.1`
- Python `3`
- PowerShell `7` for the Windows archive helper

## Useful links for evaluators

- Docs: `https://docs.blprnt.ai`
- GitHub: `https://github.com/blprnt-ai/blprnt`
- License: `BUSL-1.1`

## Development

Useful repository commands:

- `pnpm install --frozen-lockfile`
- `pnpm build`
- `cargo check -p blprnt`
- `./scripts/check-release-alignment.sh`
- `./scripts/build-linux.sh`
- `pwsh ./scripts/build-windows.ps1`
- `./scripts/build-macos.sh`

## Repository map

- `crates/blprnt/` — binary entrypoint
- `crates/api/` — HTTP API, DTOs, and static asset serving
- `crates/coordinator/` — employee scheduling and run execution
- `crates/persistence/` — local SurrealDB-backed persistence
- `crates/shared/` — shared runtime helpers and schemas
- `crates/tools/` — file and host tool implementations
- `npm/blprnt` — `@blprnt/blprnt` wrapper package used by `npx`; ships the launcher plus the shared `dist/` SPA bundle
- `npm/darwin-arm64`, `npm/linux-x64`, `npm/win32-x64` — platform packages; each ships the platform executable plus platform-specific `tools/rg`

## Runtime notes

- The API binds to `0.0.0.0:9171`.
- Persistence is local RocksDB-backed SurrealDB under `~/.blprnt/data`.
- Static assets are served from `BLPRNT_BASE_DIR` when set, otherwise from `dist/` beside the installed executable, with `./dist` as the local dev fallback.

## License

This repository is licensed under `BUSL-1.1`.

## Contributing

Contributions are welcome. See `CONTRIBUTING.md` for the lightweight pull request process.
