# blprnt

`blprnt` is a Tauri 2 desktop application for multi-agent LLM orchestration. It combines a Rust execution runtime, a React + TypeScript desktop UI, sandboxed tools, persistent session state, and provider integrations into one local-first orchestration environment.

## What It Does

`blprnt` is built to run and coordinate AI agents that can:

- manage multi-step sessions
- call tools for files, shell, plans, memory, and skills
- spawn specialized subagents for planning, execution, research, verification, and design
- stream provider responses into the desktop app in real time
- persist projects, sessions, messages, plans, and other runtime state

## Architecture

At a high level, the app looks like this:

```text
Frontend (Tauri UI) <-> Rust Runtime <-> SurrealDB
```

More concretely:

```text
tauri-src -> app_core -> engine_v2 -> providers
                 |           |            |
                 v           v            v
           persistence     tools       session logic
                 \           |           /
                  ---------- sandbox ----
```

### Core Runtime Pieces

- `tauri-src/` — Tauri desktop app entrypoint, packaging config, updater config, and app shell
- `crates/app_core/` — orchestration layer between Tauri IPC and the runtime; manages session lifecycle, MCP runtime, preview flows, and integrations like Slack
- `crates/engine_v2/` — execution engine that runs turns, dispatches tools, handles ask-question flow, and manages subagent orchestration
- `crates/providers/` — LLM provider adapters for services like Anthropic, OpenAI, and OpenRouter
- `crates/persistence/` — SurrealDB-backed persistence layer for projects, sessions, messages, plans, and related records
- `crates/tools/` — sandboxed tool execution for file operations, shell commands, plans, skills, and project-level utilities

### Runtime Model

The runtime is structured around controllers and turns:

1. a session controller is created
2. the controller runs a turn loop
3. provider responses stream back incrementally
4. tool calls are dispatched through the sandboxed tool layer
5. results feed the next turn until the session completes or stops

That flow is what lets `blprnt` act more like an orchestration system than a single-shot prompt runner.

## Major Capabilities

### Multi-Agent Orchestration

`blprnt` is designed around an orchestration model where a primary agent can delegate work to specialized subagents. The system supports planner, executor, verifier, researcher, and designer roles so work can be split into focused units instead of turning one model into an overworked intern.

### Tooling and Sandbox Execution

The app includes a tool layer for:

- reading and editing files
- applying patches
- running shell commands
- searching the codebase
- managing plans
- reading and updating project primers
- executing skill scripts

Tool execution is sandbox-aware and built for desktop usage rather than pretending your laptop is a datacenter.

### Persistent State

The persistence layer tracks core entities such as:

- projects
- sessions
- messages
- providers
- plans

This gives the app durable state across runs instead of losing everything the second the UI sneezes.

### Desktop-First App Shell

The app runs as a Tauri 2 desktop application with a React + TypeScript frontend. It packages resources such as `skills/`, `personalities/`, and `brand/`, and uses a Rust backend to handle orchestration, persistence, and system-level execution.

## Stack

- Rust 2024 edition backend
- React + TypeScript frontend
- Vite build pipeline
- Tauri 2 desktop shell
- SurrealDB persistence

## Repository Layout

- `tauri-src/` — Tauri application shell, packaging, updater, and desktop configuration
- `crates/` — internal Rust workspace crates for orchestration, runtime, providers, persistence, tools, and shared utilities
- `skills/` — bundled skills and scripts available to the runtime
- `personalities/` — personality presets and instruction packs
- `brand/` — packaged brand assets and resources

## Development

Available root commands:

- `pnpm dev` — start the frontend dev server
- `pnpm build` — create a production build
- `pnpm build:dev` — create a development-mode build
- `pnpm lint` — run linting with auto-fix enabled where configured
- `pnpm fix` — run formatting and additional fixers

Useful Tauri commands:

- `cargo tauri dev` — run the full desktop app in development mode
- `cargo tauri build` — build the desktop application
- `cargo tauri bundle` — create platform-specific distributables

## License

This repository is licensed under `BUSL-1.1`.

## Contributing

Contributions are welcome. See `CONTRIBUTING.md` for the lightweight pull request process.