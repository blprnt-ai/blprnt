---
name: para-memory-files
description: >
  PARA-based persistent memory for blprnt employees and projects. Use this
  reference when working on blprnt's memory model, API contract, or retrieval
  behavior. Covers employee and project scopes, PARA layout, atomic fact
  storage, default paths, and built-in QMD-backed search.
---

# blprnt Memory Reference

This document describes the memory model implemented by blprnt. It is the system-facing counterpart to the `skills/para-memory-files` skill.

## Public Scope Aliases

blprnt exposes symbolic scope roots in API responses:

- `$AGENT_HOME` for employee-scoped memory
- `$PROJECT_HOME` for project-scoped memory

The underlying directories remain private implementation details:

- `memories/employees/<employee_id>`
- `memories/projects/<project_id>`

Agents and clients should treat the aliases as the contract and send scope-relative paths.

## Employee Memory Layout

```text
$AGENT_HOME/
  .learnings/
    ERRORS.md
  life/
    projects/<name>/
      summary.md
      items.yaml
    areas/
      people/<name>/
      companies/<name>/
      <topic>/<name>/
    resources/<topic>/
      summary.md
      items.yaml
    archives/
  memory/
    YYYY-MM-DD.md
  AGENTS.md
  HEARTBEAT.md
  MEMORY.md
  SOUL.md
  TOOLS.md
```

## Project Memory Layout

```text
$PROJECT_HOME/
  SUMMARY.md
  archives/
    summary.md
    items.yaml
  resources/<topic>/
    summary.md
    items.yaml
```

Employee memory holds personal operating context, daily notes, and tacit knowledge. Project memory holds shared project summaries and reusable project context.

## PARA Knowledge Graph

Within employee memory, the `life/` tree follows PARA:

- `projects`: active efforts with a clear outcome or deadline
- `areas`: ongoing responsibilities and relationships
- `resources`: reference material
- `archives`: inactive items moved out of active use

Entity folders typically contain:

- `summary.md` for quick synthesis
- `items.yaml` for atomic durable facts

## Atomic Fact Schema

`items.yaml` entries use this shape:

```yaml
- id: entity-001
  fact: "The durable fact"
  category: relationship | milestone | status | preference
  timestamp: "YYYY-MM-DD"
  source: "YYYY-MM-DD"
  status: active # active | superseded
  superseded_by: null # e.g. entity-002
  related_entities:
    - companies/acme
    - people/jeff
  last_accessed: "YYYY-MM-DD"
  access_count: 0
```

Guidance:

- append durable facts rather than rewriting history
- supersede obsolete facts instead of deleting them
- keep connection paths in `related_entities`
- use access metadata to support synthesis and retrieval prioritization

## Daily Notes And Tacit Memory

Employee scope defaults to chronological notes under:

- `memory/YYYY-MM-DD.md`

Tacit user-operating knowledge belongs in:

- `MEMORY.md`

Supporting instruction files such as `AGENTS.md`, `HEARTBEAT.md`, `SOUL.md`, and `TOOLS.md` live alongside the memory tree and are indexed with the same employee scope.

## API Contract

Employee routes:

- `GET /api/v1/employees/me/memory`
- `POST /api/v1/employees/me/memory`
- `GET /api/v1/employees/me/memory/file?path=...`
- `PATCH /api/v1/employees/me/memory/file`
- `POST /api/v1/employees/me/memory/search`

Project routes:

- `GET /api/v1/projects/{project_id}/memory`
- `POST /api/v1/projects/{project_id}/memory`
- `GET /api/v1/projects/{project_id}/memory/file?path=...`
- `PATCH /api/v1/projects/{project_id}/memory/file`
- `POST /api/v1/projects/{project_id}/memory/search`

Create payload:

```json
{
  "content": "# Notes",
  "path": "optional/scope-relative.md"
}
```

Update payload:

```json
{
  "path": "life/projects/runtime/summary.md",
  "content": "# Updated summary"
}
```

Search payload:

```json
{
  "query": "what did we decide about memory storage",
  "limit": 10
}
```

Default write behavior:

- employee create without `path` appends to `memory/YYYY-MM-DD.md`
- project create without `path` appends to `SUMMARY.md`

All explicit paths must be markdown file paths relative to the scope root alias.

## Search And Retrieval

QMD is embedded in the blprnt runtime through the memory service. Search is collection-scoped and the relevant collection is synchronized before query execution.

Use search when:

- the exact file or wording is unknown
- you need recall across many memory files
- you want concept-level retrieval rather than exact string matching

Use direct file reads when:

- the target path is already known
- you are editing a specific memory file
- the exact source file matters more than ranked recall

No separate external `qmd` package or manual indexing step is part of the expected workflow.

## Memory Decay

Decay affects summary prominence, not persistence.

Suggested tiers during synthesis:

- `hot`: accessed in the last 7 days
- `warm`: accessed in the last 8 to 30 days
- `cold`: older than 30 days or never accessed

Cold facts may drop out of `summary.md` while remaining preserved in `items.yaml`.
