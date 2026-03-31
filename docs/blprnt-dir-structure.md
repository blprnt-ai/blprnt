# .blprnt dir structure:

## AGENT_HOME

```
employees/<employee_id>/
  .learnings/
    ERRORS.md
  life/
    index.md
    projects/<name>/
      summary.md
      items.yaml
    areas/<topic>/<name>/
      summary.md
      items.yaml
    resources/<topic>/
      summary.md
      items.yaml
    archives/<topic>/<name>/
      summary.md
      items.yaml
  memory/
    <date>.md
  AGENTS.md
  HEARTBEAT.md
  MEMORY.md
  SOUL.md
  TOOLS.md
```

Purpose:

- `HEARTBEAT.md`: the employee's current operating loop, priorities, and active execution posture
- `AGENTS.md`: stable contribution rules and repo-local working conventions
- `MEMORY.md`: tacit knowledge about the user's preferences and recurring collaboration patterns
- `SOUL.md`: durable identity, role framing, and style guidance when present
- `TOOLS.md`: tool-specific operating instructions, caveats, and preferred usage patterns
- `.learnings/ERRORS.md`: repeated mistakes, failure modes, and preventative lessons
- `memory/YYYY-MM-DD.md`: chronological daily notes; transient observations that may later be promoted
- `life/projects/*`: active efforts with an end condition
- `life/areas/*`: ongoing responsibilities or relationships without a fixed end
- `life/resources/*`: reusable background knowledge and reference material
- `life/archives/*`: inactive entities moved out of the active PARA buckets
- `summary.md`: fast-loading synthesized context for an entity
- `items.yaml`: append-only atomic facts, with supersession instead of destructive rewriting

Rules:

- daily notes belong under `AGENT_HOME/memory/`, never at the root of `AGENT_HOME`
- employee-specific habits and user preferences belong in `MEMORY.md`, not in project memory
- durable entity knowledge belongs in `life/`, not in daily notes once it proves recurring
- write these files with `apply_patch`, not through memory API mutations

## PROJECT_HOME

```
projects/
  <project_id>/
    memory/
      SUMMARY.md
      archives/<topic>/<name>/
        summary.md
        items.yaml
    plans/
      <plan-name>-<date>.md
    resources/<topic>/<name>/
      summary.md
      items.yaml
```

Purpose:

- `memory/SUMMARY.md`: shared current-state summary for the project
- `memory/archives/*`: inactive or superseded shared project memory
- `resources/*`: reusable project background, decision records, and topic references
- `plans/*`: live or historical execution plans owned by blprnt

Rules:

- project memory files belong under `PROJECT_HOME/memory/`
- plans belong under `PROJECT_HOME/plans/`
- do not place project summaries at the root of `PROJECT_HOME`
- do not mix employee-private working notes into project memory unless they are genuinely shared project context
- write these files with `apply_patch`, not through memory API mutations
