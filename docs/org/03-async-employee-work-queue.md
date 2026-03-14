# Phase 3 — Async Employee Work Queue

## Overview
Replace blocking subagent delegation with a project-scoped async employee queue. Preserve child-session linkage for auditability. This assumes employees and employee-centric memory/prompt layering already exist. No global dispatcher.

## What we have now
- Subagent tool calls are synchronous from the parent session’s perspective in `crates/common/src/tools/subagent.rs:10-21,79-89` and `crates/engine_v2/src/runtime/subagent_handler.rs:160-305`.
- Subagent status is minimal: `Spawned | Success | Failure | Timeout` in `crates/common/src/shared/subagent.rs:6-13`.
- Session runtime init is still top-level-session centric in `crates/app_core/src/engine_manager.rs:505-549,579-616`.
- UI treats subagents as blocking in-progress children tied to session messages in `src/components/panels/session/session-panel.viewmodel.tsx:854-878`.

## What this phase changes
- Introduce a project-local employee job queue with explicit lifecycle states.
- Let coordinators assign work to employees without blocking the parent conversation on one child turn.
- Keep child sessions as the execution/audit artifact for each job.
- Add wakeup/retry handling for queued, running, blocked, and completed employee work.

## Three options considered
### Option A — Keep synchronous subagents and just improve statuses
Patch the current child-session flow with better labels and timeout handling.

### Option B — Project-scoped async employee queue with child-session linkage
Queue work to employees, track lifecycle separately, and keep child sessions for execution traces.

### Option C — External scheduler first
Push all org work through a separate service or app-global dispatcher.

## Winner and why
Winner: Option B.

Why:
- The current subagent path already creates child sessions; it just blocks like a bad habit.
- Project-scoped queueing is enough for org-swarms first and avoids inventing a global orchestration tier too early.
- Preserving child sessions keeps compatibility with existing history, UI patterns, and audit trails.

Rejected:
- Option A: lipstick on a synchronous pig.
- Option C: wrong scope, wrong time, wrong complexity.

## Concrete code touchpoints
- `crates/common/src/tools/subagent.rs:10-21,79-89`
  - Current tool contract assumes spawn-and-wait semantics.
- `crates/engine_v2/src/runtime/subagent_handler.rs:160-305`
  - Listener, timeout, and response plumbing are built around blocking completion.
- `crates/common/src/shared/subagent.rs:6-13`
  - Status model is too thin for queue lifecycle; phase 3 needs queued/running/blocked/canceled-style states.
- `crates/app_core/src/engine_manager.rs:505-549,579-616`
  - Session creation/startup flow is where queued employee execution will still materialize child sessions.
- `crates/persistence/src/models_v2/sessions.rs:75-102,267-293`
  - Existing `parent_id` linkage is worth keeping as the execution edge.
- `src/components/panels/session/session-panel.viewmodel.tsx:854-878`
  - UI currently assumes in-progress blocking subagent messages; it must handle queued and resumable work.

## Risks / anti-patterns
- Building a queue but still waiting synchronously in the caller. That is fake async.
- Detaching jobs from child sessions and losing audit/debug history.
- Making queue state app-global before project-scoped org behavior is stable.
- Letting queued jobs mutate the same project area without future lock semantics.

## Definition of done
- Employee work can be queued and resumed asynchronously inside one project.
- Queue lifecycle states are explicit and persisted.
- Child sessions remain linked to parent work items for traceability.
- Parent UI can represent queued/running/completed employee work without pretending everything is blocking.
- Coordinator wakeups/retries exist for stalled work.
- No cross-project dispatcher is introduced.