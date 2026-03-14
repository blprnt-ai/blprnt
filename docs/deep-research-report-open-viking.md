# Deploying OpenViking for Secure Multi-User Isolation on Your Server

## Executive summary

OpenViking’s recent “multi-tenant phase 1” design and implementation establishes isolation primarily through an `account_id` boundary, with additional user/agent isolation inside an account via “spaces” (derived from `user_id` and `agent_id`) and enforced at two layers: (1) filesystem-style storage addressing (VikingFS → AGFS path prefixes) and (2) vector index filtering (tenant metadata fields such as `account_id` and `owner_space`). citeturn24search0turn13view2turn9view0turn27search1

For your stated security objective—**User A must never read/write User B data**—the most robust pattern on a single server (while still using a shared deployment) is **“account-per-user”**: create one account per end-user, and issue that user a `USER`-role key scoped to their own account only. This avoids “account-shared” scopes (like `viking://resources`) becoming an unintended cross-user sharing surface. citeturn19view0turn26view0turn24search0

Authentication/authorization in OpenViking Server is API-key based: a **Root Key** configured in `ov.conf` enables RBAC and Admin APIs; **User Keys** are random tokens stored in AGFS (`users.json`) and resolved server-side via an `APIKeyManager` index. Keys do not embed identity; identity is resolved by lookup. citeturn19view0turn21view1turn9view3

Security posture hinges on: (a) always setting `server.root_api_key` before exposing the server to any non-loopback network, and (b) fronting the service with TLS + request controls (reverse proxy / gateway) to reduce exposure to API authorization failures (a top OWASP risk class). OpenViking has had a recent critical issue (CVE-2026-22207) in versions up to 0.1.18 when `root_api_key` is omitted; newer code also adds a startup safety check to refuse binding to non-localhost without a root key. citeturn15search0turn10view0turn19view0turn28search4

Operationally: AGFS is explicitly positioned as the “single data source” for full content, and the vector DB stores URIs + vectors + metadata (not file content) but is still expensive to rebuild. Plan backups accordingly, and be aware the project discusses non-atomic multi-subsystem writes (FS + VectorDB + queue) as an active risk area. citeturn13view2turn27search7

## Components and where user data actually lives

OpenViking can be understood as a multi-layer storage-and-retrieval system that exposes “filesystem-like” URIs (`viking://…`) over an HTTP API (and SDK/CLI). The key deployment-relevant components are:

### HTTP server surface

The HTTP API includes endpoints for filesystem ops (`/api/v1/fs/*`), content reads (`/api/v1/content/*`), semantic search (`/api/v1/search/*`), sessions (`/api/v1/sessions/*`), observer/health endpoints, and multi-tenant admin endpoints (`/api/v1/admin/*`). citeturn23view0turn19view0

Two endpoints are explicitly documented as always unauthenticated (for load balancers / monitoring): `/health`, and a “quick” debug health endpoint exists (`/api/v1/debug/health`). citeturn19view0turn23view0

### Storage architecture: AGFS + VectorDB, mediated by VikingFS

OpenViking uses a **dual-layer storage approach**:

- **AGFS**: content storage for full L0/L1/L2 content, media, and relations.
- **VectorDB**: index storage for semantic retrieval—stores URI + vectors + metadata, not file content.
- **VikingFS**: URI abstraction layer that maps `viking://…` to underlying storage paths and maintains consistency with the vector index. citeturn13view2turn13view3

This separation matters for isolation because:
- AGFS path layout is where hard multi-tenant boundaries are created (e.g., `/{account_id}/...`).
- VectorDB must include and enforce tenant metadata filters (`account_id`, `owner_space`), or cross-tenant search becomes a data leak. citeturn13view2turn24search0turn9view0

### Storage backends and config knobs you can use

OpenViking’s configuration guide shows storage is configured in `ov.conf` under `storage`:

- `storage.workspace`: local data directory path.
- `storage.agfs`: mode (`http-client` or `binding-client`), backend (`local`, `s3`, `memory`), URL/timeout, and for S3 a detailed config including `prefix` (explicitly described as an optional namespacing/isolation key prefix). citeturn20view2turn20view3turn20view4  
- `storage.vectordb`: backend can be local file-based, remote HTTP, or Volcengine’s “VikingDB” (and other variants are mentioned), with fields like `name`, `url`, `project_name`, distance metric, and hybrid search weight. citeturn20view3turn13view2

### Tenant identity: account, user, agent

The multi-tenant design centers around a `UserIdentifier(account_id, user_id, agent_id)` tuple and a request-scoped `RequestContext` that carries identity + role. citeturn21view4turn9view3

In the current server middleware, identity resolution:
- accepts `X-API-Key` or `Authorization: Bearer …` for the key, and
- uses `X-OpenViking-Agent` as the agent discriminator. citeturn9view3turn19view0

### Where tenant state and user keys are stored

OpenViking stores the **account list** and **per-account user registry** in AGFS:

- Global accounts list: `/_system/accounts.json`
- Per-account users registry: `/{account_id}/_system/users.json` containing `{user_id: {role, key}}` entries. citeturn21view0turn21view1turn19view0

`APIKeyManager` loads all accounts on startup into an in-memory key→identity index and persists changes back to these AGFS files. A key design rationale given is multi-node consistency: if multiple server nodes share the same AGFS backend, user creation on one node becomes visible to others. citeturn21view1turn21view2

### The access-control “choke points” in code paths

The practical places where isolation must hold (and thus where you should focus review, hardening, and tests) are:

1. **Auth / identity resolution** in `openviking/server/auth.py`  
   - In “dev mode” (`api_key_manager is None`), it returns `Role.ROOT` with default identity values. citeturn9view3turn19view0  
   - In production mode, it resolves the key via an API-key manager and builds a `RequestContext`. citeturn9view3turn19view0

2. **Server config safety** in `openviking/server/config.py`  
   - `root_api_key` is loaded from the `server` section of `ov.conf`. citeturn10view1turn10view0  
   - If `root_api_key` is missing and the server binds to a non-loopback host, the server refuses to start (to prevent unauthenticated “ROOT” exposure). citeturn10view0

3. **URI → path mapping and path traversal defenses** in `openviking/storage/viking_fs.py`  
   - `VikingFS._uri_to_path()` maps `viking://…` to `/local/{account_id}/…`. citeturn6view3turn24search0  
   - It rejects suspicious path segments (`""`, `"."`, `".."`, `":"`, `"\"`) to prevent traversal and Windows-drive tricks. citeturn6view2  
   - The account prefix is stripped back out when returning URIs (`_path_to_uri`), keeping URIs “WYSIWYG” while preserving tenant separation at rest. citeturn6view1turn6view3

4. **Per-request authorization checks** in VikingFS  
   - The design and code outline an `_ensure_access` / `_is_accessible` gate where user role and “space” membership should prevent cross-user reads/writes, including session scoping that becomes `viking://session/{user_space}/{session_id}`. citeturn27search1turn26view2turn24search0

5. **Vector index tenancy fields** in `openviking/storage/collection_schemas.py`  
   - The context collection schema includes `account_id` and `owner_space` fields and indexes them. citeturn9view0turn9view2  
   - Record IDs are derived from a seed that includes `account_id` (`id_seed = f"{account_id}:{seed_uri}"`), reducing cross-account collisions. citeturn9view0

6. **Role-based vector search filters**  
   The design specifies filter logic typical of “shared DB + tenant_id column” isolation:
   - `ADMIN`: must filter by `account_id`
   - `USER`: must filter by `account_id` and `owner_space ∈ {user_space, agent_space}`
   - `ROOT`: no filter (global). citeturn27search1turn24search0

## Tenant isolation approaches and recommended architecture

OpenViking’s built-in multi-tenancy is naturally aligned with “tenant-id column/prefix” designs:

- In AGFS: tenant boundary is `/{account_id}/...` citeturn24search0turn26view0  
- In VectorDB: tenant boundary is `account_id` + `owner_space` filters citeturn27search1turn9view0

The key architecture choice you control is **what constitutes a “tenant”** (an OpenViking account vs a user inside an account vs a whole server instance).

### Comparison table of isolation patterns

| Isolation approach | What is isolated | How it maps to OpenViking primitives | Pros | Cons / failure modes | Operational complexity | Perf/cost impact |
|---|---|---|---|---|---|---|
| Account-per-user (recommended for strict “User A ≠ User B”) | AGFS data, VectorDB records, and “resources” scope are all per-user | Create one `account_id` per end-user and issue one `USER` key; each request implicitly selects the account via key resolution | Strong isolation without running N instances; avoids accidental sharing via `viking://resources` (account-shared) | Admin/user lifecycle count grows with users; ROOT key remains a global “break glass” | Moderate (needs provisioning automation around Admin API) | Efficient (single service), minimal incremental overhead per user |
| Multi-user within a shared account | User/agent “spaces” only; account-shared directories remain shared | One account for a whole team/org; many `USER` keys + optional several `ADMIN` keys | Easy to share resources within a team; fewer accounts | If app writes user-private data to account-shared scopes, users can see each other; admins can read all users in account by design | Low to moderate | Efficient |
| One OpenViking instance per user (container/VM per tenant) | Everything (process/memory/config/network) per tenant | Run separate server + separate storage/vectordb per tenant | Strongest blast-radius boundary; easiest reasoning for compliance | Expensive: N processes, ports, storage configs; more patching and monitoring | High | Highest cost; more overhead |
| One OpenViking instance per “tenant group” (e.g., small orgs) | Everything per group | Same as above but per org/team | Good balance if you have few tenant orgs | Still heavier than account isolation; migrations between groups are harder | Medium-high | Medium-high |

The strict requirement “User A must never read/write User B” usually implies you should **avoid any “shared” namespace** unless you have a higher-level ACL model (which the multi-tenant design treats as a later extension). The design explicitly states `viking://resources` is in-account shared, whereas user and agent data are in per-user spaces. citeturn27search1turn24search0

### RBAC roles and how they affect isolation

OpenViking’s documented roles are:

- `ROOT`: global; can use Admin APIs (create/delete accounts, manage users).
- `ADMIN`: scoped to an account; can manage users in that account and (by design) can access account user data.
- `USER`: scoped to an account; regular ops with access limited to their own isolated spaces. citeturn19view0turn27search1turn23view0

For strict user-to-user isolation:
- Issue **USER** keys to end-users.
- Do not issue **ADMIN** keys to end-users.
- Keep the **ROOT** key offline / internal-only (or in a secrets manager plus tight network access controls). citeturn19view0turn27search1turn28search1

### Recommended deployment architecture

The recommended pattern for your objectives is:

- A single OpenViking Server (or small HA pool), fronted by a reverse proxy that terminates TLS.
- An internal “provisioning / auth gateway” component (optional but recommended if you want SSO via JWT/OIDC) that maps your identity provider (IdP) identities to OpenViking user keys (and/or rotates them).
- Data plane isolation by OpenViking account-per-user, with shared storage backends (S3-compatible or local) using account-prefixed paths. citeturn21view1turn20view4turn29search0turn28search9

Mermaid architecture diagram:

```mermaid
flowchart TB
  user[End User / Client App] -->|HTTPS| rp[Reverse Proxy<br/>TLS + limits]
  rp -->|HTTP (internal)| ov[OpenViking Server<br/>/api/v1/*]
  rp -->|optional: auth subrequest| authgw[Auth Gateway<br/>OIDC/JWT -> OpenViking key]

  authgw -->|Admin API (ROOT key)<br/>provision users| ov
  authgw -->|stores mapping| secrets[(Secrets store)]

  ov -->|RequestContext<br/>account_id/user_id/agent_id| vfs[VikingFS]
  vfs --> agfs[AGFS<br/>Content store]
  vfs --> vdb[VectorDB<br/>Index store]

  agfs -->|accounts.json<br/>users.json| agfsmeta[(AGFS _system)]
```

## Secure deployment recipes with isolation enforced

### Baseline security requirements and known pitfalls

A very recent critical issue (CVE-2026-22207) affects OpenViking “through 0.1.18” when `root_api_key` is omitted, allowing unauthenticated ROOT access. Treat **“root_api_key absent” as unsafe** unless you are strictly loopback-only dev. citeturn15search0turn19view0turn10view0

Even in newer versions, the project documentation explicitly states: if `root_api_key` is not configured, auth is disabled and all requests behave as ROOT in the default account, and this mode is only allowed when binding to localhost; binding to non-loopback without `root_api_key` is rejected. citeturn19view0turn10view0

From an API-security perspective, the main class of failure you must defend against is broken authorization (“BOLA”), where a user manipulates an object identifier (e.g., a URI) to access others’ data. That risk is explicitly ranked #1 in OWASP’s API Security Top 10. citeturn28search4turn28search0

Your hardening goal is therefore: even if a user guesses another user’s URI or session id, any read/write/search touching those objects should be denied at the server. OpenViking’s design intends to provide that via VikingFS checks and VectorDB filters keyed from request context. citeturn24search0turn27search1turn26view2

### Step-by-step: production `ov.conf` template (multi-tenant + storage)

A minimally production-oriented `ov.conf` must include:

- `server.root_api_key` (enables auth)
- `server.host`/`port`
- Restrictive `cors_origins` if you have browser clients
- `storage` configuration (workspace + AGFS and vector backends). citeturn19view1turn25view0turn20view2turn20view3

Example template (adjust model configs as needed; shown with a local workspace and local backends):

```json
{
  "server": {
    "host": "0.0.0.0",
    "port": 1933,
    "root_api_key": "REPLACE_WITH_A_LONG_RANDOM_SECRET",
    "cors_origins": ["https://your-frontend.example"]
  },
  "storage": {
    "workspace": "/var/lib/openviking/data",
    "agfs": {
      "backend": "local",
      "timeout": 10
    },
    "vectordb": {
      "backend": "local",
      "name": "context"
    }
  }
}
```

This structure matches the documented server section and storage sections (workspace, agfs, vectordb). citeturn19view1turn25view0turn20view2

If you want an S3-compatible backend for AGFS, OpenViking’s config supports a detailed S3 block and includes a `prefix` field explicitly described as an optional namespace-isolation prefix (useful for separating environments like `prod/` vs `staging/`, or separating OpenViking deployments). citeturn20view4turn20view3

### Step-by-step: deploy with the official container image via Docker Compose

The repository includes a `docker-compose.yml` that runs the image `ghcr.io/volcengine/openviking:main`, exposes port 1933, and mounts a config file and data directory as persistent volumes with a healthcheck on `/health`. citeturn18view0turn25view0

1. Create host directories:
   - `/var/lib/openviking/ov.conf`
   - `/var/lib/openviking/data/`

2. Put your hardened `ov.conf` at `/var/lib/openviking/ov.conf` (example above). citeturn19view1turn25view0

3. Use the upstream compose file as baseline; a tightened variant (adds a private network and avoids binding to all interfaces unless intended) might look like:

```yaml
version: "3.8"
services:
  openviking:
    image: ghcr.io/volcengine/openviking:main
    container_name: openviking
    ports:
      - "127.0.0.1:1933:1933"   # bind only to localhost; front with reverse proxy
    volumes:
      - /var/lib/openviking/ov.conf:/app/ov.conf
      - /var/lib/openviking/data:/app/data
    healthcheck:
      test: ["CMD-SHELL", "curl -fsS http://127.0.0.1:1933/health || exit 1"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 30s
    restart: unless-stopped
```

The image, mounts, and `/health` healthcheck are taken from the repo’s compose definition. citeturn18view0turn19view0turn23view0

4. Run:
   - `docker compose up -d`

### Step-by-step: deploy as a systemd service (bare metal / VM)

OpenViking docs include a systemd unit example. It runs `openviking-server`, sets `WorkingDirectory`, and provides `OPENVIKING_CONFIG_FILE` pointing to your config. citeturn25view0turn10view1

Example unit file (base from docs; add hardening directives):

```ini
[Unit]
Description=OpenViking HTTP Server
After=network.target

[Service]
Type=simple
User=openviking
Group=openviking
WorkingDirectory=/var/lib/openviking
Environment="OPENVIKING_CONFIG_FILE=/etc/openviking/ov.conf"
ExecStart=/usr/local/bin/openviking-server
Restart=always
RestartSec=5

# Hardening (adapt to your environment)
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/openviking /etc/openviking

[Install]
WantedBy=multi-user.target
```

The “service layout” concept (a `.service` unit supervising a process) is standard systemd behavior. citeturn24search12turn25view0  
Hardening directives and their intent are discussed in systemd-hardening guidance. citeturn24search9

### Tenant provisioning workflow (accounts and users)

Once the server is up with `root_api_key` enabled, you should provision users via Admin API:

- Create an account + its first admin:
  - `POST /api/v1/admin/accounts` with `{account_id, admin_user_id}`
- Add a user:
  - `POST /api/v1/admin/accounts/{account_id}/users` with `{user_id, role}`
- Rotate a user key:
  - `POST /api/v1/admin/accounts/{account_id}/users/{user_id}/key` citeturn19view0turn23view0

The authentication guide provides concrete curl examples for account and user creation using `X-API-Key: <root-key>`, and it defines the semantics of root vs user keys. citeturn19view0turn21view1

**Strict isolation recipe (account-per-user):**
- For each end-user `U`, create account `acct_U` and a corresponding `USER` key:
  - Use `account_id=acct_U`, `user_id=U`, `role=user`.
- Do not create shared accounts unless you intentionally want `viking://resources` to be shared. citeturn27search1turn19view0

### Reverse proxy examples to enforce TLS, reduce exposure, and protect admin APIs

OWASP guidance is unambiguous that REST services should only be exposed over HTTPS to protect credentials in transit (including API keys/JWTs). citeturn28search5

#### entity["company","NGINX","web server and reverse proxy"] reverse proxy (TLS termination + admin-path restriction)

NGINX’s reverse proxy docs show how to forward requests upstream and adjust headers. citeturn29search0turn29search32

A practical posture:

- Public internet: allow normal `/api/v1/*` requests (still authenticated by OpenViking).
- Admin plane: deny `/api/v1/admin/*` and potentially `/api/v1/system/*` from the public internet; only allow from a VPN / bastion / specific IP range; optionally require mTLS.

Illustrative `server` blocks (TLS skeleton omitted for brevity):

```nginx
# Public API
server {
  listen 443 ssl http2;
  server_name openviking.example.com;

  # TLS config here...

  # Deny admin endpoints from public internet
  location ^~ /api/v1/admin/ {
    return 403;
  }

  location / {
    proxy_pass http://127.0.0.1:1933;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
  }
}

# Separate admin-only vhost (bind to VPN interface / private network)
server {
  listen 8443 ssl http2;
  server_name openviking-admin.internal;

  # TLS (ideally mTLS) here...

  location / {
    proxy_pass http://127.0.0.1:1933;
    proxy_set_header Host $host;
  }
}
```

This uses the standard `proxy_pass` and header-forwarding model NGINX documents. citeturn29search32turn29search0

#### entity["organization","Traefik","reverse proxy and ingress controller"] ForwardAuth pattern (OIDC/JWT gateway in front)

Traefik’s ForwardAuth middleware is explicitly designed to delegate authentication decisions to an external service; on 2xx it forwards the original request, otherwise it returns the auth service’s response. citeturn29search1turn29search13

This is a clean way to put **your own identity system** (OIDC, session cookies, JWT validation) in front of OpenViking without modifying OpenViking itself.

Conceptual dynamic config snippet:

```yaml
http:
  middlewares:
    auth-gateway:
      forwardAuth:
        address: "http://auth-gateway:8080/auth"
        trustForwardHeader: true

  routers:
    openviking:
      rule: "Host(`openviking.example.com`)"
      entryPoints: ["websecure"]
      middlewares: ["auth-gateway"]
      service: "openviking-svc"

  services:
    openviking-svc:
      loadBalancer:
        servers:
          - url: "http://openviking:1933"
```

ForwardAuth semantics come directly from Traefik docs. citeturn29search1turn29search13

### Authentication integration options (API keys vs OAuth2/JWT)

OpenViking Server natively authenticates using API keys (root + user keys), and keys are “random tokens” without embedded identity. citeturn19view0turn21view1

If your platform already uses OAuth2/OIDC, you can integrate without forking OpenViking by using an **auth gateway** that:

1. Validates user session / JWT (OIDC).
2. Maps `(your_user_id)` to an OpenViking `(account_id, user_id)` and retrieves the corresponding OpenViking user key.
3. Proxies the request to OpenViking, injecting `X-API-Key: <openviking-user-key>` and `X-OpenViking-Agent: <agent_id>` if needed. citeturn19view0turn9view3turn28search9

This avoids needing OpenViking to validate JWTs directly, while still aligning with OWASP OAuth2/OIDC security practices. citeturn28search9turn28search16

If you want a proxy-mode OAuth solution, entity["organization","OAuth2 Proxy","oauth2 reverse proxy"] is commonly used to front upstream apps and supports multiple upstreams. citeturn29search6  
NGINX’s `auth_request` pattern is also commonly used to place OAuth in front of an upstream, and documented walkthroughs describe NGINX calling an auth endpoint like `/oauth2/auth` and allowing/denying based on the response. citeturn29search10turn29search0

## Backups, monitoring, and maintenance

### Backup strategy aligned to OpenViking’s storage model

Because OpenViking explicitly states that **AGFS stores the full content** while the vector DB stores only URIs/vectors/metadata and is not the content store, backups must prioritize AGFS. citeturn13view2turn13view3

What to include in backups depends on backend choices:

- **Local AGFS (`storage.agfs.backend=local`)**: back up `storage.workspace` (and/or the mounted container volume directory) because it contains:
  - per-account user data directories (`/{account_id}/user/...`, `/{account_id}/agent/...`, `/{account_id}/session/...`),
  - account and user key registries (`/_system/accounts.json`, `/{account_id}/_system/users.json`). citeturn26view0turn21view1turn20view2

- **S3 AGFS (`storage.agfs.backend=s3`)**: back up the bucket (and treat the `prefix` as part of your environment boundary), because that bucket becomes your content store. citeturn20view4turn20view3

- **VectorDB**:
  - If local-file backend, include its on-disk files (likely under the same workspace path) in backups. citeturn20view2turn13view2
  - If remote/cloud backend, ensure you have a provider-grade backup/export or snapshot capability for the index, because rebuilding embeddings can be slow/expensive. (The docs emphasize the vector DB is an index, but it still holds expensive derived artifacts like embeddings.) citeturn13view2turn9view0

**Consistency caution:** an issue discussion explicitly notes that core write operations coordinate across multiple subsystems (VikingFS, VectorDB, queue), and that failures mid-operation can leave the system inconsistent (e.g., FS move succeeded, vector update failed). Plan backups to reduce the likelihood of capturing half-finished operations (e.g., quiesce ingestion or use “wait”/drain semantics before snapshotting). citeturn27search7turn23view0

### Monitoring strategy (health, observer endpoints, external metrics)

OpenViking provides:
- `GET /health` (unauthenticated) for load balancers / monitoring. citeturn19view0turn23view0  
- Observer endpoints (`/api/v1/observer/*`) for queue/VLM/VikingDB/system status. citeturn23view0turn30search2  
- CLI command `openviking observer system` for operational checks (documented in server quickstart). citeturn30search5

A pragmatic monitoring setup:

- **Black-box probes**: hit `/health` through the same reverse proxy route your clients use (catches TLS/proxy failures and server hangs). citeturn19view0turn30search0  
- **Control-plane probes** (restricted): query `/api/v1/observer/system` and `/api/v1/system/status` from an internal network only, because these are operational endpoints and were discussed as sensitive in the prior “missing root_api_key” bug report. citeturn30search3turn23view0  
- **Prometheus-style monitoring**: Prometheus’ model is to scrape metrics via HTTP endpoints. If OpenViking does not expose Prometheus metrics, you can still use Prometheus to scrape reverse-proxy metrics and node/container metrics, and rely on HTTP probes to OpenViking. citeturn29search3turn29search0

### Key rotation and lifecycle management

OpenViking supports regenerating a user key via Admin API (`POST /api/v1/admin/accounts/{account_id}/users/{user_id}/key`), which invalidates the old key and issues a new one (the docs describe it as “regenerate user key”). citeturn19view0turn23view0turn21view1

Operational best practice: treat user keys like long-lived credentials with rotation procedures and incident response playbooks (revoke/rotate on suspected leak). This aligns with OWASP’s guidance on robust authorization/authentication handling and avoiding broken authentication patterns. citeturn28search1turn28search35

## Testing, verification, and threat model

### Automated and manual isolation verification plan

Your goal is to produce evidence that **cross-user access is impossible** (except via intended privileged roles). OpenViking returns explicit error codes like `UNAUTHENTICATED` and `PERMISSION_DENIED` in API responses, which you can assert on in tests. citeturn23view0turn19view0

#### Minimal manual tests

Assume you deployed with account-per-user and created:

- Account `acct_alice` with user key `K_ALICE` (role USER)
- Account `acct_bob` with user key `K_BOB` (role USER) citeturn19view0turn21view1

Test matrix (all should fail with `403 PERMISSION_DENIED` or equivalent error semantics):

1. **Cross-account directory listing**  
   - Write a resource as Alice to `viking://resources/…` (this is account-scoped).  
   - From Bob, `GET /api/v1/fs/ls?uri=viking://resources/` should show only Bob’s account resources (likely empty).  
   This validates account isolation is actually applied at the storage layer (`/{account_id}/…` prefix) and at index filtering. citeturn26view0turn6view3turn27search1

2. **Cross-user (within account) “space” break attempt** (only if you choose multi-user-in-one-account)  
   - Create account `acme` with users alice and bob, both role USER. citeturn19view0turn21view1  
   - Have Alice write a memory under her user space.
   - From Bob, try to read Alice’s user-space URI directly (guess or obtain it). This should be denied by VikingFS access checks and vector filters (`owner_space`). citeturn27search1turn26view2turn24search0

3. **Search isolation test**  
   - Insert a unique token into Bob’s memory (“TOKEN_BOB_ONLY_9f3a…”) and ensure it is embedded/indexed.  
   - From Alice: `POST /api/v1/search/find` for that token should return no results.  
   This specifically validates that vector search is filtering by `account_id` and `owner_space`, as described in the multi-tenant retriever logic. citeturn27search1turn9view0turn23view0

4. **Session isolation test**  
   - Create a session as Alice; verify its backing path includes Alice’s user space (`viking://session/{user_space}/{session_id}`) per design. citeturn27search1turn26view0  
   - From Bob, attempt to fetch Alice’s session by ID; it should be denied. citeturn23view0turn28search4

#### Automated test outline (pytest-style)

Automate the above by:
- provisioning accounts/users via Admin API using the root key,
- performing writes and searches with each user key,
- asserting that unauthorized reads/searches return `PERMISSION_DENIED` and never leak URIs/content. citeturn19view0turn23view0turn28search4

To ensure you also catch “BOLA-style” bugs, fuzz object identifiers:
- randomize URIs, attempt traversal-like segments, and verify VikingFS normalization rejects dangerous segments (`..`, `:` and `\`). citeturn6view2turn28search4

### Threat model and mitigations mapped to OpenViking’s design

#### Primary threats

1. **Broken Object Level Authorization (BOLA)**: user manipulates a URI/session identifier to access another tenant’s objects. citeturn28search4turn23view0turn24search0  
2. **Broken authentication / security misconfiguration**: running without `root_api_key` or exposing admin endpoints improperly; historically linked to CVE-2026-22207 when root key omitted. citeturn15search0turn19view0turn10view0  
3. **Credential leakage**: user keys are bearer secrets; if logged or leaked, attacker can act as that user; root key compromise is catastrophic. citeturn19view0turn28search5  
4. **Inconsistent state after partial writes**: can lead to stale pointers or orphaned index entries, complicating restore and forensics. citeturn27search7  
5. **Resource contention / multi-process conflicts**: multiple processes pointing at the same local data directory can cause lock/contention issues; a reported case recommends using one shared HTTP server rather than multiple stdio instances. citeturn27search4

#### Mitigation checklist

- **Always set `server.root_api_key` before binding to anything other than localhost** (and verify you are on a patched version beyond vulnerable releases). citeturn15search0turn10view0turn19view0  
- **Tenant model**: use **account-per-user** to remove in-account shared scopes from the “user isolation” problem space. citeturn26view0turn27search1  
- **Network segmentation**: block `/api/v1/admin/*` at the reverse proxy for public traffic; expose admin only on private network or separate hostname. citeturn23view0turn29search0  
- **TLS everywhere**: only expose HTTPS externally to protect API keys/JWTs in transit. citeturn28search5turn29search0  
- **SSO integration**: if using OAuth2/OIDC, do it at the gateway (ForwardAuth/auth_request) and inject OpenViking keys upstream; do not pass ROOT credentials to browsers. citeturn29search1turn28search9turn19view0  
- **Explicit authorization tests**: keep automated negative tests for cross-tenant read/write/search as part of CI to guard against regressions (because BOLA failures are common and high-impact). citeturn28search4turn23view0  
- **Operational safeguards**: quiesce ingestion before backups; monitor not just “port open” but “HTTP responsive,” as hangs have been reported even when the process is alive. citeturn30search0turn23view0