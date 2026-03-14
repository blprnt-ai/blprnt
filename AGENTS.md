# blprnt Project Primer

## Overview

Tauri 2.x desktop application for multi-agent LLM orchestration. Rust 2024 edition backend (17+ crates) with React 18 + TypeScript frontend.

---

## Backend Architecture (Rust)

```
┌─────────────────────────────────────────────────────────────┐
│ Frontend (Tauri UI) ◄──IPC──► Rust Runtime ◄──► SurrealDB   │
├─────────────────────────────────────────────────────────────┤
│  tauri-src ─► app_core ─► engine_v2 ─► providers            │
│                   │           │           │                 │
│                   ▼           ▼           ▼                 │
│             persistence    tools      session               │
│                   │           │           │                 │
│                   └─────► sandbox ◄───────┘                 │
│                              │                              │
│                          vault / common / macros            │
└─────────────────────────────────────────────────────────────┘
```

### Crate Deep Dive

---

#### **app_core** — Orchestration Layer

**Purpose**: Bridge between Tauri IPC and engine_v2. Manages session lifecycle, MCP runtime, Slack integration.

**Main State Struct** (`EngineManager`):

```rust
pub struct EngineManager {
  controllers: Arc<Mutex<HashMap<SurrealId, Arc<RwLock<Controller>>>>>,
  user: RwLock<Option<Arc<User>>>,
  mcp_runtime: Arc<McpRuntimeManager>,
  preview_manager: Arc<PreviewManager>,
  slack_question_message_refs: Arc<Mutex<HashMap<String, SlackAskQuestionMessageReference>>>,
  slack_reconciliation_queue: Arc<Mutex<HashMap<String, SlackAskQuestionReconciliationWorkItem>>>,
}
```

**Tauri Commands (35+ across 9 domains)**:


| Domain          | Key Commands                                                                                        |
| --------------- | --------------------------------------------------------------------------------------------------- |
| **Session**     | `session_create`, `session_start`, `session_stop`, `send_prompt`, `answer_question`, `rewind_to`    |
| **Project**     | `new_project`, `edit_project`, `delete_project`, `plan_list`, `plan_create`, `plan_update`          |
| **Provider**    | `create_provider`, `list_providers`, `delete_provider`, `link_codex_account`, `link_claude_account` |
| **MCP**         | `mcp_server_create`, `mcp_server_update`, `mcp_server_status_list`, `mcp_server_test_connection`    |
| **Slack**       | `slack_start_oauth`, `slack_notify_interactive`, `slack_status`                                     |
| **Billing**     | `billing_create_checkout_session`, `billing_get_credit_balance`                                     |
| **Personality** | `personality_list`, `personality_create`, `personality_update`                                      |
| **Preview**     | `preview_start`, `preview_stop`, `preview_status`                                                   |
| **Control**     | `frontend_ready`, `sign_in`, `sign_out`, `get_build_hash`                                           |


**Controller Lifecycle**:

1. `EngineManager::init_engine()` → `Controller::new(config)`
2. `Controller::run()` spawned as background task
3. Commands push prompts → `Controller::push_prompt()`
4. Streaming via `Blprnt::emit()` event dispatch
5. `Controller::stop()` on session close

---

#### **engine_v2** — Execution Engine

**Purpose**: Turn orchestration, tool dispatch, subagent spawning, ask-question flow.

**Runtime Flow**:

```
Controller::run()
  → Runtime::execute_turn()
    → ProviderAdapter::stream_conversation()
      → SSE events → ProviderDispatch
        → Tool invocations → Tools::run()
          → Tool results → next turn
```

**Key Structs**:

- `Controller` - Session execution manager with prompt queue
- `Runtime` - Turn/step orchestration with hook system
- `ToolUseContext` - Execution environment (sandbox key, working dirs, agent kind)

**Hook System**:

- `PreTurn` - Before LLM call
- `PreStep` - Before tool execution
- `PostStep` - After tool execution
- `PostTurn` - After turn completes

**Ask-Question State Machine**:

```
unanswered_pending → [claim] → answer_accepted (terminal)
                   → [timeout] → expired
```

Idempotent claim contract with Slack delivery key normalization.

---

#### **providers** — LLM Adapters

**Purpose**: SSE streaming, token counting, credential management for Anthropic/OpenAI/OpenRouter.

**ProviderAdapter Enum**:

```rust
pub enum ProviderAdapter {
  Anthropic(Arc<AnthropicProvider>),
  OpenAi(Arc<OpenAiProvider>),
  OpenRouter(Arc<OpenRouterProvider>),
  Mock(Arc<MockProvider>),
}
```

**ProviderAdapterTrait**:

```rust
#[async_trait]
pub trait ProviderAdapterTrait: Send + Sync {
  fn provider(&self) -> Provider;
  async fn stream_conversation(&self, req: ChatRequest, tools: Option<Arc<ToolSchemaRegistry>>,
    dispatch: Arc<ProviderDispatch>, cancel_token: CancellationToken);
  async fn count_tokens(&self, req: ChatRequest, tools: &ToolSchemaRegistry) -> Result<u32>;
}
```

**SSE Event Flow (Anthropic)**:

```
HTTP POST /messages (stream=true)
  → SSE → AnthropicStreamEvent (tagged enum)
    ├── MessageStart { message }
    ├── ContentBlockStart { index, content_block }
    │   └── AnthropicContentBlock: Text | Thinking | ToolUse
    └── ContentBlockDelta { index, delta }
        └── Delta: TextDelta | ThinkingDelta | ToolUseDelta
  → AnthropicSseParser::parse_event()
  → ProviderDispatch::send(ProviderEvent::*)
```

**Rate Limiting (Anthropic)**:

- Per-API-key client registry (keyed by hash)
- Four rate limit windows: requests, tokens, input_tokens, output_tokens
- Headers: `anthropic-ratelimit-*-remaining`, `*-reset`

**Error Handling** (three-tier):

1. Match error type (`authentication_error`, `rate_limit_error`, etc.)
2. Match HTTP status code (401, 429, 500, etc.)
3. Fallback to human-readable message

---

#### **tools** — Tool Execution

**Purpose**: 23+ sandboxed tools for file ops, shell, memory, skills, plans.

**Tools Enum**:

```rust
pub enum Tools {
  Dir(Dir),      // tree, search
  File(File),    // create, read, files_read, update, delete, apply_patch
  Memory(Memory), // create, search, delete
  Host(Host),    // shell
  Project(Project), // primer_get, primer_update, plan_*
  Skill(Skill),  // list_skills, apply_skill, get_reference
  Rg(Rg),        // ripgrep search
  #[cfg(feature = "worktrees")]
  Worktree(Worktree),
}
```

**File Operations**:


| Tool         | Args                                                    | Purpose                      |
| ------------ | ------------------------------------------------------- | ---------------------------- |
| `FileCreate` | `{path, content}`                                       | Create file with parent dirs |
| `FileRead`   | `{path, line_start?, line_end?, include_line_numbers?}` | Read with line slicing       |
| `FilesRead`  | `{items: [{path, line_start?, line_end?}]}`             | Batch multi-file read        |
| `FileUpdate` | `{path, find, replace}`                                 | Simple string replacement    |
| `FileDelete` | `{path}`                                                | Delete file                  |
| `ApplyPatch` | `{diff}`                                                | V4A unified diff format      |


**V4A Patch Format**:

```
*** Begin Patch
*** Add File: path/to/new.txt
+line1
+line2
*** Update File: path/to/existing.rs
@@ context
-old_line
+new_line
*** Delete File: path/to/remove.txt
*** End Patch
```

**Shell Execution** (platform-specific):

- **macOS (thor.rs)**: Native process APIs
- **Linux (loki.rs)**: Landlock/seccomp MAC
- **Windows (baldr.rs)**: PowerShell-native

Config: 120s timeout, 8KB buffer, 10k max lines, 500 token output truncation.

**Memory Tools**:

- `MemoryCreate` → `MemoryRepositoryV2::insert()` → generates 384-dim embedding
- `MemorySearch` → HNSW vector search with decay scoring
- `MemoryDelete` → soft delete

**Tool Schema Generation**:

```rust
fn schema(config: &ToolsSchemaConfig) -> Vec<ToolSpec> {
  if !ToolAllowList::is_tool_allowed_and_enabled(ToolId::FileCreate, config.agent_kind, config.is_subagent) {
    return vec![];
  }
  let schema = schemars::schema_for!(FileCreateArgs);
  // ... build JSON schema
}
```

---

#### **persistence** — Database Layer

**Purpose**: SurrealDB repository pattern for all entities.

**Models**:


| Model            | Fields                                                                          | Relationships                                          |
| ---------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------ |
| `ProjectRecord`  | `name`, `working_directories`, `agent_primer`, timestamps                       | → sessions (1:N)                                       |
| `SessionRecord`  | `name`, `agent_kind`, `model_override`, `reasoning_effort`, `token_usage`, etc. | → project (N:1), → parent (subagent), → messages (1:N) |
| `MessageRecord`  | `rel_id`, `turn_id`, `step_id`, `role`, `content`, `visibility`                 | → session (N:1), → parent (threading)                  |
| `MemoryRecord`   | `content`, `embedding` (Vec), `access_count`, epochs                            | HNSW indexed                                           |
| `ProviderRecord` | `provider` (enum), timestamps                                                   | Singleton per provider type                            |


**SurrealId Type**:

```rust
pub struct SurrealId(pub RecordId);  // RecordId = (table: String, key: Uuid)

// Serializes to SQL format: "sessions:u'019bf534-cdda-7a63-9ccf-350ecd7e5024'"
impl Serialize for SurrealId {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.0.to_sql())
  }
}
```

**Memory Vector Search**:

```sql
SELECT *, semantic_score, decay_factor,
  semantic_score * ($min_decay + (1 - $min_decay) * decay_factor) AS score
FROM (
  SELECT *,
    vector::similarity::cosine(embedding, $embedding) AS semantic_score,
    math::clamp(
      math::pow(0.5, (($epoch - last_access_epoch) / ($half_life * (1 + math::ln(access_count + 1))))),
      $min_decay, 1
    ) AS decay_factor
  FROM memories ORDER BY semantic_score DESC LIMIT $prefetch
) ORDER BY score DESC LIMIT $limit
```

- HNSW index: dimension 384, M=32, M0=64, cosine distance
- Decay: older memories decay, frequent access preserves freshness
- min_decay=0.25, half_life=750 queries

**Relationship Constraints**:

```sql
DEFINE FIELD project ON sessions TYPE option<record<projects>> REFERENCE ON DELETE CASCADE;
DEFINE FIELD parent_id ON sessions TYPE option<record<sessions>> REFERENCE ON DELETE UNSET;
DEFINE FIELD session_id ON messages TYPE option<record<sessions>> REFERENCE ON DELETE CASCADE;
```

---

#### **sandbox** — File Isolation

**Purpose**: Capability-based file system containment using `cap_async_std`.

**Key Functions**:

```rust
pub async fn read_file(sandbox_key: &str, path: &Path) -> Result<String>
pub async fn write_file(sandbox_key: &str, path: &Path, content: &str) -> Result<()>
pub async fn delete_file(sandbox_key: &str, path: &Path) -> Result<()>
pub fn add_writable_directory(sandbox_key: &str, path: &Path) -> Result<()>
```

All paths validated against registered sandbox roots. Attempts to escape containment return permission errors.

---

#### **vault** — Secrets Storage

**Purpose**: IOTA Stronghold encrypted storage for API keys and OAuth tokens.

**Operations**:

```rust
pub async fn store_secret(key: &str, value: &[u8]) -> Result<()>
pub async fn get_secret(key: &str) -> Result<Option<Vec<u8>>>
pub async fn delete_secret(key: &str) -> Result<()>
```

Keys derived from provider ID UUIDs. Stronghold file at `~/.blprnt/vault.stronghold`.

---

#### **prompt** — System Prompts

**Purpose**: Askama templates for agent-kind-specific system prompts.

**Template Structure**:

```rust
#[derive(Template)]
#[template(path = "system_prompt.md")]
pub struct SystemPromptTemplate {
  pub agent_kind: AgentKind,
  pub personality: Option<String>,
  pub skill_content: Option<String>,
  pub primer: Option<String>,
  pub tools_schema: String,
  pub system_info: SystemInfo,
}
```

Templates at `prompt/templates/`. Rendered per-session with injected skill, personality, and primer.

---

#### **common** — Foundation Types

**Purpose**: Shared types across all crates. **ONLY depends on macros crate**.

**Agent System**:

```rust
pub enum AgentKind {
  Crew,      // Main orchestrator
  Planner,   // Plan creation
  Executor,  // Task execution
  Verifier,  // Code review/validation
  Researcher,// Read-only exploration
  Designer,  // UI/UX guidance
}
```

**Tool Allowlist** (per agent kind):

- `Crew`: All tools
- `Planner`: plan_*, primer_*, ask_question
- `Executor`: file_*, shell, rg, memory_*, skill_*
- `Verifier`: file_read, files_read, rg, shell (read-only)
- `Researcher`: file_read, files_read, rg, dir_tree, dir_search
- `Designer`: None (guidance only)

**Event System**:

```rust
// BlprntDispatch - global event channel (10k capacity)
pub struct BlprntDispatch(broadcast::Sender<BlprntEvent>);

// ProviderDispatch - per-session LLM streaming events
pub enum ProviderEvent {
  ResponseDelta { content: String },
  ReasoningDelta { content: String },
  ToolCall { id: ToolId, name: String, arguments: Value },
  TokenUsage { input: u32, output: u32 },
  Error { error: ProviderError },
}
```

**Error Taxonomy** (`common/errors/`):

- `ProviderError` - API failures, rate limits, auth errors
- `ToolError` - Execution failures, permission denied
- `EngineError` - Runtime failures, cancellation
- `AuthError` - JWT, OAuth failures
- `NetworkError` - Connection issues

---

#### **macros** — Proc Macros

**Purpose**: `SurrealEnumValue` derive macro for enum ↔ SurrealDB Value conversion.

```rust
#[derive(SurrealEnumValue)]
pub enum AgentKind {
  Crew,
  Planner,
  // ...
}
// Generates: impl From<AgentKind> for Value, impl TryFrom<Value> for AgentKind
```

---

#### **tauri-src** — Application Entry

**Purpose**: Binary entry point, plugin initialization, SurrealDB lifecycle.

**Bootstrap Sequence**:

1. Initialize Tauri app with plugins (store, shell, clipboard, etc.)
2. Spawn SurrealDB process (embedded, RocksDB storage)
3. Run database migrations (`persistence::migrate()`)
4. Initialize `EngineManager` as Tauri state
5. Register all commands from `app_core::commands()`
6. Show main window, emit `BackendReady`

---

### Backend Design Patterns


| Pattern                | Usage                                                    |
| ---------------------- | -------------------------------------------------------- |
| **Arc**                | Thread-safe async state (EngineManager fields)           |
| **Broadcast Channels** | Event streaming (BlprntDispatch, ProviderDispatch)       |
| **Repository Pattern** | Database CRUD (all *RepositoryV2 structs)                |
| **Enum Dispatch**      | Tool/Provider routing (Tools enum, ProviderAdapter enum) |
| **CancellationToken**  | Hierarchical graceful shutdown                           |
| **Hook Registry**      | Lifecycle plugins (PreTurn, PostTurn, etc.)              |
| **Singleton**          | Global resources (OnceCell, lazy_static!)                |


---

## Frontend Architecture (React + TypeScript)

### Tech Stack

- **React 18** + TypeScript + Vite
- **MobX** for state management (observables, viewmodels)
- **Tailwind CSS** for styling + custom design tokens
- **Radix UI** for accessible primitives
- **Dockview** for panel/tab management
- **Tauri** for desktop integration via specta-generated bindings

### Directory Structure

```
src/
├── components/
│   ├── atoms/          # Base UI primitives (Button, Input, Tree, etc.)
│   ├── molecules/      # Composite components
│   ├── organisms/      # Complex sections (sidebar, trees)
│   ├── panels/         # Panel implementations (session, project, plan)
│   ├── dialogs/        # Modal dialogs (8 dialogs)
│   ├── forms/          # Form components (SessionForm + 8 sections)
│   ├── ai-elements/    # AI message rendering (Response, CodeBlock, ChainOfThought)
│   ├── dockview/       # Dockview integration
│   └── views/          # Page-level views
├── context/            # React context providers
├── hooks/              # Custom React hooks
├── lib/
│   ├── models/         # MobX models (14 domain + 8 message types)
│   ├── api/            # Tauri API wrappers
│   ├── events/         # Event bus and handlers
│   └── utils/          # Utility functions
├── styles/             # Global CSS + Tailwind config
└── bindings.ts         # Auto-generated Tauri bindings (specta)
```

### Frontend Design Patterns

#### MobX State Management

- **Viewmodels**: `makeAutoObservable()` for reactive state containers
- **Registry Pattern**: Deduplication for `ProjectModel`, `SessionModel`, `PlanModel`
- **Lazy Loading**: `onBecomeObserved()` triggers data fetching
- **Event Subscriptions**: `globalEventBus` for cross-component updates

#### Models Layer (src/lib/models/)


| Model                | Purpose                                       | Pattern     |
| -------------------- | --------------------------------------------- | ----------- |
| **AppModel**         | Global state (skills, catalog, personalities) | Singleton   |
| **ProjectModel**     | Workspace management                          | Registry    |
| **SessionModel**     | Session execution state                       | Registry    |
| **PlanModel**        | Todo planning                                 | Registry    |
| **PersonalityModel** | System prompts                                | Static CRUD |
| **ProviderModel**    | LLM credentials                               | Static CRUD |
| **BillingModel**     | Payment integration                           | Instance    |
| **PanelModel**       | UI panel state                                | Array-based |


#### Message Models (8 types)

- `PromptMessageModel` - User inputs
- `ResponseMessageModel` - LLM responses
- `ThinkingMessageModel` - Reasoning blocks
- `ToolUseMessageModel` - Tool invocations
- `SubAgentMessageModel` - Subagent execution
- `SignalMessageModel` - Info/warning/error
- `QuestionAnswerMessageModel` - Interactive Q&A

**Message Pipeline**: `MessageRecord` (backend) → `toMessage()` → TypeScript union → `createMessageModel()` → MobX observable

### Dockview Panel System

**8 Panel Types**: Intro, Personality, Session, Project, Plan, UserAccount, Preview

```typescript
// Panel lifecycle
DockviewLayoutViewModel.openPanel({ id, title, component, params })
DockviewLayoutViewModel.closePanel(panelId)

// Panel ID generation
sessionPanelId(projectId, sessionId) → `session-${projectId}-${sessionId}`
projectPanelId(projectId) → `project-${projectId}`
planPanelId(projectId, planId) → `plan-${projectId}-${planId}`
```

**Layout Persistence**: `localStorage['dockview-layout']`
**Hotkeys**: `Cmd+T` (new Intro), `Cmd+W` (close active)

### AI Message Rendering


| Component                   | Purpose                                        |
| --------------------------- | ---------------------------------------------- |
| `Response`                  | Markdown wrapper with Streamdown               |
| `ResponseCodeBlock`         | Routes to syntax highlighter, Mermaid, or diff |
| `CodeBlock`                 | Shiki syntax highlighting (synthwave-84 theme) |
| `Mermaid`                   | Diagram rendering with security=strict         |
| `ChainOfThought`            | Collapsible thinking/tool containers           |
| `ToolUseChainOfThoughtStep` | Dispatcher for 17 tool types                   |


### Sidebar Tree System

**Three-tier structure**: Projects → Sessions → Subagents

- `SidebarViewmodel` - Root orchestrator
- `SessionsTreeViewmodel` - Per-project session list
- `TreeProvider` - Expansion state context

### Dialog System

- **Base**: Radix Dialog primitives
- **Forms**: MobX viewmodel (`SessionFormViewModel`)
- **Delete Confirm**: Requires typing "DELETE"

### Styles System

- **CSS Variables**: Design tokens in `variables.css`
- **Color Space**: OKLCH for perceptual uniformity
- **Theme Toggle**: `.dark` class on root
- **Dockview Theme**: `.blprnt-theme`
- **Fonts**: Roboto (sans), Roboto Mono (mono), Lora (serif)

---

## Code Style

### Rust (rustfmt.toml)

```toml
edition = "2024"
max_width = 120
group_imports = "StdExternalCrate"
imports_granularity = "Item"
```

### TypeScript

- Biome for linting/formatting
- MobX `observer()` HOC for reactive components
- Strict TypeScript with specta-generated types

---

## Feature Flags


| Flag            | Scope                                    | Purpose                            |
| --------------- | ---------------------------------------- | ---------------------------------- |
| `testing`       | app_core, engine_v2, common, persistence | In-memory SurrealDB                |
| `worktrees`     | app_core, engine_v2, tools               | Git worktree support               |
| `debug-tracing` | app_core, providers                      | Enhanced event logging             |
| `fnf`           | common                                   | Friends & Family provider variants |


---

## Critical Files for New Features

### Backend


| Area           | Files                                                                        |
| -------------- | ---------------------------------------------------------------------------- |
| New agent kind | `common/src/agent/types.rs`, `common/src/agent/allowlist.rs`                 |
| New event type | `common/src/session_dispatch/events/*.rs`, `common/src/blprnt.rs`            |
| New tool       | `tools/src/{category}/`, `tools/src/tools.rs` enum, `common/src/tools/` args |
| New provider   | `providers/src/providers/`, `providers/src/provider_adapter.rs`              |
| Tauri command  | `app_core/src/cmd/`, `app_core/src/lib.rs` commands()                        |
| DB model       | `persistence/src/models_v2/`, add migrate() + repository                     |


### Frontend


| Area              | Files                                                                     |
| ----------------- | ------------------------------------------------------------------------- |
| New panel type    | `components/dockview/content-components.tsx`, `lib/models/panel.model.ts` |
| New dialog        | `components/dialogs/`, follow Radix pattern                               |
| New model         | `lib/models/`, add to factory if message type                             |
| New tool renderer | `components/panels/session/organisms/session-conversation/tool-use/`      |
| New context       | `context/`, export provider and hook                                      |


---

## Quick Reference

### Backend Entry Points

- **Binary**: `tauri-src/src/main.rs`
- **Commands**: `app_core/src/cmd/*.rs`
- **Execution**: `engine_v2/src/runtime/mod.rs`
- **Tools**: `tools/src/tools.rs`
- **Prompts**: `prompt/templates/`

### Frontend Entry Points

- **App**: `src/main.tsx` → `src/app.tsx`
- **Bindings**: `src/bindings.ts` (auto-generated)
- **Models**: `src/lib/models/`
- **Dockview**: `src/context/dockview-context.tsx`
- **Sidebar**: `src/components/organisms/sidebar/`

---

## Database

- **SurrealDB 3.0.0** (embedded, RocksDB storage)
- Tables: `projects`, `sessions`, `messages`, `memories`, `providers`
- HNSW vector index on memory embeddings (dim 384, cosine)
- Cascade constraints for referential integrity

---

## Platform-Specific

### Shell Execution

- **macOS (thor.rs)**: Native process APIs
- **Linux (loki.rs)**: Landlock/seccomp MAC
- **Windows (baldr.rs)**: PowerShell-native

### File Operations

- All I/O through `sandbox` crate
- `dunce::canonicalize()` for path normalization
- `.blprntignore` for custom patterns

---

## Frontend Design System & Style Guide

Source files: `src/styles/` — all CSS is imported via `src/styles/index.css`.

### Style File Structure

```
src/styles/
├── index.css       # Entry point (imports all others)
├── variables.css   # Core design tokens & theme definitions
├── tailwind.css    # Tailwind framework import
├── fonts.css       # Font face declarations
├── base.css        # HTML/body base styles, scrollbar
├── reset.css       # Typography resets
├── theme.css       # Dockview panel theming (.blprnt-theme)
├── utilities.css   # Custom utility classes & theme toggle
└── animations.css  # Keyframe animations
```

### Branding

- **Name**: `blprnt` — rendered as `<span className="text-primary font-medium font-mono">blprnt</span>` (monospace, brand blue)
- **Primary brand color**: `oklch(0.65 0.18 250)` / `#0f92f7` (bright blue, hue 250°)
- **Color space**: OKLCH throughout for perceptual uniformity

### Color Tokens

#### Brand

| Token | Value | Notes |
|-------|-------|-------|
| `--brand` | `oklch(0.65 0.18 250)` = `#0f92f7` | Primary interactive blue |
| `--brand-light` | `oklch(0.65 0.18 250 / 60%)` | Hover states |
| `--brand-lighter` | `oklch(0.65 0.18 250 / 30%)` | Subtle accents, scrollbar thumb |
| `--brand-lightest` | `oklch(0.65 0.18 250 / 10%)` | Backgrounds, drag targets |

#### Semantic (Light Mode)

| Token | Value |
|-------|-------|
| `--background` | `rgb(231, 229, 228)` — warm off-white |
| `--foreground` | `rgb(30, 41, 59)` — dark slate |
| `--card` | `rgb(245, 245, 244)` |
| `--primary` | `rgb(15, 146, 247)` — matches `--brand` |
| `--primary-foreground` | `rgb(255, 255, 255)` |
| `--secondary` | `rgb(214, 211, 209)` |
| `--muted-foreground` | `rgb(107, 114, 128)` |
| `--accent` | `rgb(229, 243, 253)` |
| `--destructive` | `rgb(239, 68, 68)` |
| `--border` | `rgb(15, 146, 247)` — brand blue |
| `--ring` | `rgb(15, 146, 247)` — focus rings |

#### Semantic (Dark Mode — `.dark` class on root)

| Token | Value |
|-------|-------|
| `--background` | `rgb(2, 6, 24)` — deep navy |
| `--foreground` | `rgb(214, 231, 213)` — light mint-green |
| `--card` | `rgb(15, 23, 43)` |
| `--primary` | `rgb(15, 146, 247)` — unchanged |
| `--primary-foreground` | `rgb(15, 23, 43)` |
| `--secondary` | `rgb(4, 51, 79)` |
| `--muted-foreground` | `rgb(167, 167, 167)` |
| `--accent` | `rgba(15, 146, 247, 0.08)` |
| `--destructive` | `rgb(239, 68, 68)` |
| `--border` | `rgb(4, 72, 124)` — darker blue |
| `--ring` | `rgb(15, 146, 247)` |

#### Status Colors

| Token | Value | Role |
|-------|-------|------|
| `--info` | `#7a9fdd` | Informational (blue-grey) |
| `--success` | `#00bf75` | Positive (green) |
| `--warn` | `#e08700` | Warning (amber) |
| `--error` | `#ff5464` | Error/critical (red-pink) |

#### Sidebar Colors

Light: `--sidebar` = `rgb(214, 211, 209)`, foreground = `rgb(30, 41, 59)`
Dark: `--sidebar` = `rgb(58, 54, 51)`, foreground = `rgb(226, 232, 240)`
Sidebar primary and ring always match brand blue.

#### Chart Colors (5-step blue gradient, same in both modes)

`rgb(15,146,247)` → `rgb(7,129,223)` → `rgb(6,115,198)` → `rgb(5,100,173)` → `rgb(5,86,148)`

### Typography

| Token | Stack | Usage |
|-------|-------|-------|
| `--font-sans` | `Roboto, ui-sans-serif, system-ui` | Primary |
| `--font-serif` | `Lora, serif` | Decorative |
| `--font-mono` | `Roboto Mono, monospace` | Code, branding |

Additional bundled fonts (not primary): Tasa, Lilex, Montserrat.

**Heading scale** (`reset.css`):
- `h1`: 1.5rem / 700
- `h2`: 1.25rem / 700
- `h3`: 1.125rem / 600
- `h4`: 1rem / 500

### Border Radius

| Token | Value |
|-------|-------|
| `--radius-sm` | 2px |
| `--radius-md` | 4px |
| `--radius-lg` | 8px (default `--radius`) |
| `--radius-xl` | 14px |

### Spacing & Dimensions

- Base font: `16px`
- Spacing unit: `0.25rem` (4px)
- Scrollbar size: `8px`; thumb color: `--brand-lighter` at 50% opacity

### Shadows

Seven levels (`--shadow-2xs` → `--shadow-2xl`). Light mode base: `hsl(240 3.9% 60%)`. Dark mode base: `hsl(0 0% 10.2%)`. Shadow offset: `2px 2px`, blur `5.5px`, spread `2px`. Opacity steps: 0.14 (xs) → 0.29 (sm–xl) → 0.72 (2xl).

### Dockview Theme (`.blprnt-theme`)

All panel chrome is transparent; only active tab text uses `--color-primary` (brand blue). Key tokens:

| Token | Value |
|-------|-------|
| `--dv-border-radius` | `var(--radius-md)` (4px) |
| `--dv-tabs-and-actions-container-height` | `2rem` |
| `--dv-drag-over-background-color` | `--brand-lightest` |
| `--dv-icon-hover-background-color` | `--brand-lighter` |
| `--dv-scrollbar-background-color` | `--brand-lighter` |
| `--dv-active-sash-transition-duration` | `0.3s` (delay `0.5s`) |
| `--dv-floating-box-shadow` | `8px 8px 8px 0px rgba(83,89,93,0.5)` |

### Animations

- **`.rainbow`**: gradient loop success → info → warn, 1000ms linear infinite
- **`.blink-fade` / `.blink-fade-1–8`**: opacity pulse, 680–750ms, staggered
- **`.text-sweep`**: highlight sweep, duration via `--duration` (default 2s)
- **`@keyframes shine`**: background-position oscillation (used for shimmer effects)

### Utility Classes

- `.bg-grid` / `.bg-grid-medium` / `.bg-grid-small` — dot-grid backgrounds using brand blue at low opacity
- `.bg-gradient-glow` / `.bg-gradient-glow-dark` — radial brand-blue glow + cyan accent
- `.bounce-loader` — ripple spinner

### Syntax Highlighting

Shiki with `synthwave-84` theme — applied identically in both light and dark modes.

### Tailwind Integration

Tailwind CSS 4.x via `@tailwindcss/vite`. No `tailwind.config.ts`; all tokens live in `variables.css` `@theme inline` blocks. Standard pattern: `bg-background text-foreground`, `text-primary`, `border-border`, etc.

### Theme Toggle

`.dark` class on the root element switches all semantic tokens. Toggle implemented via `<input class="theme-checkbox">` in `utilities.css`.
