---
name: blprnt-memory
description: >
  PARA-based persistent memory for blprnt employees and projects. Use this
  skill whenever you need to save, retrieve, revise, or organize knowledge
  across runs. Covers three layers: (1) entity memory in PARA folders with
  atomic YAML facts, (2) daily notes as a chronological log, and (3) tacit
  working knowledge about how the user operates. Also covers planning files,
  memory decay, periodic synthesis, and recall through blprnt's built-in QMD
  search. Trigger on any memory task: storing facts, writing notes, creating
  entities, refreshing summaries, recalling prior context, or managing plans.
---

# PARA Memory In blprnt

This skill defines how persistent memory should be handled inside the blprnt runtime. The storage model is PARA-based and file-oriented, the way to interact with it is through blprnt's memory API.

Use this skill when you need durable memory that survives beyond the current run.

## Memory Is Scoped

blprnt exposes two memory scopes:

- employee memory for personal operating context, notes, and tacit knowledge
- project memory for shared project context, summaries, and reusable reference material

The API presents those scopes through symbolic roots:

- `$AGENT_HOME` for employee memory
- `$PROJECT_HOME` for project memory

Those aliases are part of the contract. The physical `memories/employees/<id>` and `memories/projects/<id>` directories are implementation details.

## The Three Layers

### Layer 1: Knowledge Graph (`$AGENT_HOME/life/`)

Durable entities live in PARA folders. Each entity folder has two levels:

1. `summary.md` for fast-loading context
2. `items.yaml` for append-only atomic facts

```text
$AGENT_HOME/life/
  projects/
    <name>/
      summary.md
      items.yaml
  areas/
    people/<name>/
    companies/<name>/
    <topic>/<name>/
  resources/
    <topic>/
      summary.md
      items.yaml
  archives/
  index.md
```

PARA classification rules:

- `projects` are active efforts with a concrete outcome or end condition
- `areas` are ongoing responsibilities or relationships without an end date
- `resources` are reference topics and reusable background material
- `archives` hold inactive items moved out of the active three buckets

Fact handling rules:

- Put durable facts into `items.yaml` as soon as they become worth keeping
- Refresh `summary.md` from the active facts during periodic synthesis
- Do not erase facts just because they became outdated; mark them superseded instead
- When an entity is no longer active, move it into `archives`

Create a dedicated entity when at least one of these is true:

- it has come up several times
- it has a direct relationship to the user or current work
- it is a meaningful project, company, person, or topic likely to recur

If not, keep it in the daily note until it proves durable.

For the atomic fact schema, scope rules, and retrieval behavior, see [references/schemas.md](references/schemas.md).

### Layer 2: Daily Notes (`$AGENT_HOME/memory/YYYY-MM-DD.md`)

Daily notes are the chronological layer. This is where raw events, observations, and conversation fragments belong before they are distilled into structured entity memory.

Guidelines:

- write to the current day's note continuously while working
- use daily notes for transient details and incomplete observations
- promote durable facts from daily notes into Layer 1 during normal maintenance

### Layer 3: Tacit Knowledge (`$AGENT_HOME/MEMORY.md`)

This file captures how the user tends to work: preferences, patterns, recurring constraints, and lessons about collaboration.

Guidelines:

- keep world facts out of this file unless they describe the user's operating style
- update it when you discover a stable preference or recurring behavioral pattern

## Externalize Everything

Run state is not durable. Memory only persists when it lands on disk through blprnt memory storage.

- if something should survive the run, write it down
- if the user says "remember this", store it in the proper memory layer
- if you learn a durable lesson about operating the system, update the right instruction file such as `AGENTS.md`, `TOOLS.md`, or a relevant skill
- if you make or uncover a repeatable mistake, record it explicitly so future runs can avoid it

Prefer durable written memory over relying on temporary context.

## Recall Through blprnt QMD Search

Do not treat memory recall as manual file-grepping by default. blprnt already wires QMD into the core runtime and exposes search through the memory API.

Use:

- `POST /api/v1/employees/me/memory/search` for employee-scoped recall
- `POST /api/v1/projects/{project_id}/memory/search` for project-scoped recall

Example payload:

```json
{
  "query": "what did we decide about the runtime architecture",
  "limit": 10
}
```

blprnt syncs the relevant memory collection before search, so you do not need to run a separate indexing command.

Use direct file reads when you already know the target path. Use search when the remembered wording may differ from the original text.

## API Interaction Model

When operating through blprnt, memory is read and written with the built-in API:

- `GET /api/v1/employees/me/memory`
- `POST /api/v1/employees/me/memory`
- `GET /api/v1/employees/me/memory/file?path=...`
- `PATCH /api/v1/employees/me/memory/file`
- `POST /api/v1/employees/me/memory/search`
- `GET /api/v1/projects/{project_id}/memory`
- `POST /api/v1/projects/{project_id}/memory`
- `GET /api/v1/projects/{project_id}/memory/file?path=...`
- `PATCH /api/v1/projects/{project_id}/memory/file`
- `POST /api/v1/projects/{project_id}/memory/search`

Default write targets:

- employee creates without a path append to `memory/YYYY-MM-DD.md`
- project creates without a path append to `SUMMARY.md`

If you specify a path, it must be a markdown path relative to the exposed scope root, for example:

- `life/projects/runtime/summary.md`
- `life/areas/people/alex/items.yaml`
- `.learnings/ERRORS.md`
- `resources/architecture/summary.md`

## Project Memory

Project memory is shared context for the codebase or initiative, not a substitute for personal working memory.

Typical uses:

- `SUMMARY.md` for the current shared state of the project
- `resources/` for reusable project background and reference material
- `archives/` for inactive project memory

Do not mix employee-specific heuristics, private working notes, or personal collaboration patterns into project memory unless they are genuinely project-relevant and shared.

## Planning Files

Keep live plans in timestamped files under `plans/` at the repository root so other agents and runs can discover them. Plans are not the same as memory, but they are part of the durable working record.

Guidelines:

- search for existing plans before drafting a new one
- prefer the newest non-superseded plan
- if a plan becomes stale, mark it as superseded rather than silently letting multiple conflicting plans drift

## Practical Rule

The mental model is simple:

- entity facts go into PARA folders
- chronology goes into daily notes
- user operating patterns go into tacit memory
- shared project state goes into project memory
- recall goes through blprnt's built-in memory search unless you already know the file you need
