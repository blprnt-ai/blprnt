# blprnt Memory Schemas And Retrieval Rules

This reference defines the durable structure used by the `blprnt-memory` skill inside blprnt.

## Scope Contract

blprnt exposes memory through two symbolic roots:

- `$AGENT_HOME/memory` for employee-scoped memory
- `$PROJECT_HOME/memory` for project-scoped memory

Paths in memory API requests are always relative to one of those roots.

Implementation note: storage lives under the `.blprnt` home directory.

The canonical on-disk structure is provided at runtime through the bundled `Canonical Blprnt Directory Structure` reference.

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
  memory/
    SUMMARY.md
  archives/
    summary.md
    items.yaml
  plans/
    <plan-name>-<date>.md
  resources/<topic>/
    summary.md
    items.yaml
```

## Atomic Fact Schema (`items.yaml`)

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

Field intent:

- `id`: stable identifier for future supersession links
- `fact`: the atomic statement worth preserving
- `category`: lightweight grouping for later synthesis
- `timestamp`: when the fact became true or relevant
- `source`: where it came from, commonly a daily note date
- `status`: `active` until replaced, then `superseded`
- `superseded_by`: pointer to the replacement fact when applicable
- `related_entities`: paths to connected entity folders
- `last_accessed` and `access_count`: retrieval metadata used for synthesis and decay

## Memory Decay And Synthesis

Decay changes retrieval priority, not storage. Facts remain in `items.yaml` even when they stop appearing in a short summary.

Access tracking:

- when a fact is surfaced or used, update `last_accessed`
- increment `access_count` when the fact is materially reused

Recency tiers for rewriting `summary.md`:

- `hot`: accessed within the last 7 days
- `warm`: accessed within the last 8 to 30 days
- `cold`: not accessed for more than 30 days, or never accessed

Synthesis rules:

- hot facts should dominate the summary
- warm facts stay available at lower priority
- cold facts can drop out of `summary.md` while remaining in `items.yaml`
- heavily reused facts can stay prominent longer even as they age

Do not delete old facts unless there is a separate explicit retention policy. Prefer supersession.

## API Contract

Employee scope:

- `GET /api/v1/employees/me/memory`
- `GET /api/v1/employees/me/memory/file?path=...`
- `POST /api/v1/employees/me/memory/search`

Project scope:

- `GET /api/v1/projects/{project_id}/memory`
- `GET /api/v1/projects/{project_id}/memory/file?path=...`
- `POST /api/v1/projects/{project_id}/memory/search`

Search payload:

```json
{
  "query": "what did we decide about memory storage",
  "limit": 10
}
```

Write contract:

- durable writes use the `apply_patch` tool, not memory API
- write against `AGENT_HOME` or `PROJECT_HOME` directly
- use canonical targets such as `memory/YYYY-MM-DD.md`, `memory/SUMMARY.md`, `life/...`, `.learnings/ERRORS.md`, and `plans/...`

## Retrieval Guidance

Use search when:

- you know the concept but not the exact wording
- you need recall across many files
- you are looking for older related context

Use direct file reads when:

- you already know the path
- you need exact file-level structure rather than ranked search results

Use `apply_patch` when:

- you are creating a new durable memory file
- you are updating a specific durable file
- you are fixing a file that landed in the wrong directory and needs to move to the canonical location

QMD is part of the blprnt runtime. Memory services sync the relevant collection before search, so agents should not depend on a separate external indexing step.
