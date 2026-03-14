# Phase 5 — Career Ladders, Department Guardrails, and Project Locks

## Overview
Complete the project-scoped org model with career ladders, department-level guardrails, and explicit local lock rules for conflicting work. This is the full-dream project company phase. Still no cross-project COO.

## What we have now
- Runtime request/prompt data is flat in `crates/common/src/shared/provider.rs:180-220`; there is no employee layer for department or seniority.
- Tool context carries project/session/parent plus `current_skills`, but no org authority or lock context in `crates/tools/src/tool_use.rs:7-18`.
- Session/project execution remains project-centric in `crates/app_core/src/engine_manager.rs:505-549,579-616`.
- Agent permissions remain hardcoded by `AgentKind` in `crates/common/src/agent/allowlist.rs:45-165`.

## What this phase changes
- Add project-local career ladder metadata to employees and roles.
- Add department guardrails that constrain assignment, review, and escalation paths.
- Introduce explicit project-local lock rules for conflicting work streams.
- Layer governance over the existing employee/role model instead of adding more `AgentKind` values.

## Three options considered
### Option A — Put ladders and departments into `AgentKind`
Make runtime kind represent job family, rank, and permissions.

### Option B — Layer career ladders and department guardrails over project roles, plus local lock rules
Keep `AgentKind` small and build governance on employee metadata and project-local policy.

### Option C — Skip governance and rely on managers to coordinate manually
Trust prompts and social convention instead of encoded constraints.

## Winner and why
Winner: Option B.

Why:
- By phase 5 the system already has employee identity, manager relations, and assigned skills. Governance should extend that model, not replace it.
- Project-local lock rules are the missing piece that keeps async org work from stomping on itself.
- Encoding all of this in `AgentKind` would be architectural vandalism.

Rejected:
- Option A: same `AgentKind` mistake, just bigger.
- Option C: manual coordination is not a system design.

## Concrete code touchpoints
- `crates/common/src/shared/provider.rs:180-220`
  - `ChatRequest` and `PromptParams` need an employee/governance layer for ladder, department, and lock context.
- `crates/tools/src/tool_use.rs:7-18`
  - Tool execution needs employee authority and lock context to prevent conflicting actions.
- `crates/common/src/agent/allowlist.rs:45-165`
  - Department/ladder guardrails should compose with permission decisions here or just above it.
- `crates/app_core/src/engine_manager.rs:505-549,579-616`
  - Queue/session startup must respect project-local lock rules when dispatching work.
- `crates/persistence/src/models_v2/sessions.rs:75-102`
  - Session binding remains project + employee; phase 5 should not unwind that foundation.

## Risks / anti-patterns
- Reintroducing global coordination too early.
- Encoding seniority and department as prompt fluff instead of enforceable metadata.
- Locking too broadly and freezing the org.
- Locking too weakly and letting async employees collide on the same work.

## Definition of done
- Employees have project-local career ladder and department metadata.
- Assignment/review/escalation guardrails use that metadata.
- Conflicting work can be blocked or serialized by explicit project-local lock rules.
- Runtime/tooling receives governance context without bloating `AgentKind`.
- Async org execution respects locks and authority boundaries.
- Phase remains strictly project-scoped; no multi-project COO layer is introduced.