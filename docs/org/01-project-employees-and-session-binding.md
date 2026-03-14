# Phase 1 — Project Employees and Session Binding

## Overview
Move blprnt-corp from session-first execution to project-scoped employee identity. This phase assumes Phase 0 already hardened the memory substrate. Keep scope local to one project. No global COO, no cross-project staff graph.

## What we have now
- Projects store one `agent_primer` for the whole project in `crates/persistence/src/models_v2/projects.rs:17-26`.
- Sessions are the durable unit for identity, behavior, and execution in `crates/persistence/src/models_v2/sessions.rs:27-50` and are only linked to `project` plus optional `parent_id` in `crates/persistence/src/models_v2/sessions.rs:75-102`.
- Session creation API already exposes `personality_key`, `queue_mode`, `model_override`, and guardrails, but only at session scope in `crates/app_core/src/cmd/session_commands.rs:45-89`.
- Prompt assembly loads one project primer, one session personality, one volatile `current_skills` list, and one project memory blob in `crates/engine_v2/src/runtime/context.rs:190-257`.
- Prompt rendering only understands one primer, one skill list, and one memory blob in `crates/prompt/src/lib.rs:62-109`.
- Phase 0 should have improved the memory substrate first, but there is still no first-class employee model yet.

## What this phase changes
- Add a first-class employee identity inside a project.
- Bind every top-level and child session to `project + employee`; keep sessions as execution artifacts, not the durable owner of identity.
- Move durable primer, persistent skills, and memory ownership to the employee.
- Build prompt context in layers: employee primer + employee persistent skills + project plan/project memory + session overrides.

## Three options considered
### Option A — Keep sessions as the durable org member
Store org metadata on sessions and treat long-lived sessions as employees.

### Option B — Add employee identity with project-specific session binding
Create first-class employees, bind sessions to them, and keep sessions disposable execution shells.

### Option C — Put org identity directly on projects
Store all employee behavior in project config and resolve a virtual employee at runtime.

## Winner and why
Winner: Option B.

Why:
- Current sessions are obviously execution-shaped, not org-shaped. They track runtime knobs and parent/child lineage, not durable employment state.
- Project-level storage is too flat. One `agent_primer` in `projects` is not enough for a real org.
- Employee identity cleanly absorbs primer, persistent skills, and memory scope without exploding `SessionModelV2` into a junk drawer.

Rejected:
- Option A: turns session history into identity state. That is a design hangover, not architecture.
- Option C: centralizes too much on the project record and makes per-employee evolution awkward.

## Concrete code touchpoints
- `crates/persistence/src/models_v2/projects.rs:17-26,99-109`
  - Evidence that project state is single-primer and project-wide.
- `crates/persistence/src/models_v2/sessions.rs:27-50,75-102,243-293`
  - Evidence that sessions currently carry behavior knobs and are listed per project.
- `crates/app_core/src/cmd/session_commands.rs:45-89`
  - Existing creation surface can be extended to pass `employee_id` without rethinking the entire API.
- `crates/engine_v2/src/runtime/context.rs:190-257`
  - Current request assembly is the place to layer employee-owned prompt state.
- `crates/prompt/src/lib.rs:62-109`
  - Prompt renderer must accept layered employee/project/session context instead of one flat primer/skills blob.
- `crates/tools/src/tool_use.rs:7-18`
  - Tool context lacks employee identity today; phase 1 must fix that.

## Risks / anti-patterns
- Smuggling employee identity into `personality_key` or session name. That is brittle and stupid.
- Letting project primer and employee primer fight without explicit precedence.
- Migrating existing sessions without a default employee mapping strategy.
- Pretending child sessions do not need employee binding; they do, or ownership gets muddy fast.

## Definition of done
- A project-scoped employee model exists and is persisted.
- Top-level and child sessions store `employee_id` alongside `project`.
- Prompt assembly resolves employee primer and employee persistent skills before session overrides.
- Tool/runtime context carries employee identity.
- Existing projects/sessions migrate with a sensible default employee.
- Phase remains project-local; no multi-project staff identity exists.