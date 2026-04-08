# blprnt

blprnt is a local AI execution runtime for a zero-human company.

![Dashboard](/assets/dashboard.png)

## Quickstart

```bash
npx blprntai
```

## How it works

blprnt uses employees as your autonomous agents. The CEO is your first hire. The rest of the organization is up to you, depending on your project or idea.

The source of truth for all work is the issue system. You create employees and assign issues to them. The employees then do the work and tag each other when a handoff or review is required.

![Dashboard](/assets/issue-description.png)

You can run employees manually, via a timer/heartbeat, issue assignment, or using the @Name mention feature in a issue comment. When a run is done, you can choose to continue the conversation with the employee.

## Requirements

For normal usage, the primary entrypoint is:

```bash
npx blprntai
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

- `pnpm --dir frontend install --frozen-lockfile`
- `pnpm --dir frontend build`
- `cargo check --manifest-path backend/Cargo.toml -p blprnt`
- `./scripts/check-release-alignment.sh`
- `./scripts/build-linux.sh`
- `pwsh ./scripts/build-windows.ps1`
- `./scripts/build-macos.sh`

## Repository map

- `backend/crates/blprnt/` — binary entrypoint
- `backend/crates/api/` — HTTP API, DTOs, and static asset serving
- `backend/crates/coordinator/` — employee scheduling and run execution
- `backend/crates/persistence/` — local SurrealDB-backed persistence
- `backend/crates/shared/` — shared runtime helpers and schemas
- `backend/crates/tools/` — file and host tool implementations
- `npm/blprnt` — `blprntai` wrapper package used by `npx`; ships the launcher plus the shared `dist/` SPA bundle
- `npm/darwin-arm64`, `npm/linux-x64`, `npm/win32-x64` — platform packages; each ships the platform executable plus platform-specific `tools/rg`

## Runtime notes

- The API binds to `0.0.0.0:9171`.
- Persistence is local RocksDB-backed SurrealDB under `~/.blprnt/data`.
- Static assets are served from `BLPRNT_BASE_DIR` when set, otherwise from `dist/` beside the installed executable, with `./dist` as the local dev fallback.
- Set `BLPRNT_OPEN_BROWSER=false` for headless/server/container runs to disable the automatic browser open on startup.

## Docker quick deploy

This repo now includes a `Dockerfile` and a parameterized `docker-compose.yml` for quick single-host deploys.

The Docker image intentionally builds the Rust binary with a dedicated `docker-release` Cargo profile that disables expensive release-LTO linking. That keeps `docker compose up --build` much more practical for quick deploys while still producing an optimized non-debug runtime binary.

### Per-deploy isolation

The compose file isolates each deploy by making both the host port and runtime home directory configurable:

- `BLPRNT_HOST_PORT` controls the published port
- `BLPRNT_INSTANCE_ROOT` controls the host directory mounted to the container's `HOME`

Because blprnt stores runtime state under `HOME/.blprnt`, giving each deploy its own `BLPRNT_INSTANCE_ROOT` prevents overlapping data between multiple deployments on the same machine.

### Example

First deploy:

```bash
BLPRNT_INSTANCE_ROOT=./deployments/app-a \
BLPRNT_HOST_PORT=9171 \
docker compose up -d --build
```

Second deploy on the same host:

```bash
BLPRNT_INSTANCE_ROOT=./deployments/app-b \
BLPRNT_HOST_PORT=9172 \
docker compose up -d --build
```

Each deploy will keep its own runtime state under:

```text
./deployments/<instance>/home/.blprnt
```

### Optional deployed-mode env

You will usually also want to provide your real browser origin and cookie settings:

```bash
BLPRNT_INSTANCE_ROOT=./deployments/prod \
BLPRNT_HOST_PORT=9171 \
BLPRNT_CORS_ORIGINS=https://app.example.com \
BLPRNT_SESSION_COOKIE_SECURE=true \
docker compose up -d --build
```

If you run behind TLS, prefer `BLPRNT_SESSION_COOKIE_SECURE=true`.

## Secure server deployment

For local development, the default runtime behavior is intentionally permissive enough to make first-run setup easy. For an internet-reachable server, turn on deployed mode and run blprnt behind TLS.

### Baseline deployment stance

- terminate HTTPS at a reverse proxy or load balancer in front of blprnt
- keep the browser and API on the same origin when possible
- persist `~/.blprnt/data` on durable storage
- keep secrets out of source control and inject them via environment variables
- expose the public internet only to the reverse proxy, not directly to an unencrypted backend port

### Required environment for deployed mode

```bash
export BLPRNT_DEPLOYED=true

# Comma-separated browser origins allowed to call the API cross-origin.
# If you serve the SPA and API from the same origin, this can often be omitted.
export BLPRNT_CORS_ORIGINS="https://app.example.com"

# Optional: override cookie behavior if you need something other than the deployed defaults.
# Defaults in deployed mode: Secure=true, SameSite=Lax, TTL=168 hours.
export BLPRNT_SESSION_TTL_HOURS=168
# export BLPRNT_SESSION_COOKIE_SECURE=true
# export BLPRNT_SESSION_COOKIE_SAME_SITE=Lax

# Optional and dangerous: only enable this for a controlled migration where an
# existing owner record has no login credential yet.
# export BLPRNT_ALLOW_OWNER_RECOVERY_BOOTSTRAP=true
```

### What deployed mode changes

With `BLPRNT_DEPLOYED=true`:

- session cookies are marked `Secure` by default
- CORS no longer falls back to arbitrary localhost-style development origins
- legacy public `POST /api/v1/onboarding` owner creation is disabled
- public owner-recovery bootstrap is disabled by default when an owner already exists without login credentials

That last point matters for upgrades: the browser-facing bootstrap endpoint is safe for first-owner setup on a fresh database, but claiming an already-existing owner record from a public server should be an explicit operator action, not the default.

### Reverse proxy and TLS expectations

blprnt itself listens on plain HTTP. In production, put it behind a reverse proxy that:

- serves HTTPS to browsers
- forwards requests to the local blprnt port
- preserves normal `Host` and forwarded headers
- optionally serves the SPA and API from the same public origin

Example shape:

```text
browser --https--> reverse proxy --http--> blprnt :9171
```

Because the browser sees HTTPS at the edge, `Secure` session cookies still work even when TLS terminates at the proxy.

### First-user bootstrap flow

For a fresh deployment:

1. start blprnt with an empty persistent data directory and `BLPRNT_DEPLOYED=true`
2. open the server URL in a browser over `https://`
3. complete the bootstrap form, which calls `POST /api/v1/auth/bootstrap-owner`
4. continue through normal in-app onboarding after the authenticated session is created

### Persistent storage requirements

blprnt stores operational state in `~/.blprnt/data`.

Treat that directory as required persistent application data. If you deploy in a container or VM, mount it on durable storage so you do not lose:

- employees
- issues and comments
- runs and coordination state
- login credentials and session records
- integration configuration

### Secrets to manage

At minimum, manage these outside the repo:

- provider credentials used by your configured AI providers
- Telegram bot token / webhook secret if Telegram is enabled
- any reverse-proxy TLS certificates or upstream secret material

blprnt stores some integration secrets in its own vault/stronghold-backed runtime state, but your deployment still needs secure handling for initial injection and host-level secret management.

### Smoke-test checklist

After deployment, verify this path end to end:

1. `GET /api/v1/auth/status` reports the expected bootstrap state
2. first-owner bootstrap succeeds once on a fresh database
3. the `Set-Cookie` response for `blprnt_session` includes `HttpOnly` and `Secure`
4. authenticated `GET /api/v1/auth/me` succeeds with the browser session cookie
5. `POST /api/v1/auth/logout` clears the session and `GET /api/v1/auth/me` stops working afterward
6. cross-origin browser access is blocked unless the origin is explicitly listed in `BLPRNT_CORS_ORIGINS`

Repository-level validation for the deployment auth slice:

```bash
cargo test -p api auth_ -- --nocapture
cargo test -p api cors_ -- --nocapture
```

### Remaining security follow-ups

This deployment baseline is meant to be safe enough for the first server rollout, not the final auth/security story. Follow-up work still worth considering:

- CSRF-specific defenses if deployment moves beyond same-origin browser usage
- rate limiting and login abuse protection
- operator tooling for safer owner credential migration/reset flows
- explicit proxy-trust configuration if future features depend on forwarded scheme/host interpretation

## License

This repository is licensed under `BUSL-1.1`.

## Contributing

Contributions are welcome. See `CONTRIBUTING.md` for the lightweight pull request process.
