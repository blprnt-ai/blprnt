# Phase 4 — Org Roles, Manager Oversight, and Assigned Skills

## Overview
Add project-scoped org structure: roles, manager relationships, employee-first navigation, and manager-assigned persistent skills. This phase assumes async employee execution already works. Do not bloat `AgentKind` into an org chart.

## What we have now
- `AgentKind` is a small flat runtime taxonomy in `crates/common/src/agent/types.rs:22-31`.
- Tool permissions are hardcoded by `AgentKind` in `crates/common/src/agent/allowlist.rs:45-165`; `list_skills` is intentionally off.
- Personalities are already file-backed and selectable in `crates/common/src/personality_files.rs:8-12` and `crates/common/src/personality_service.rs:95-200`.
- Skills are runtime `current_skills`, not durable employee assignments, in `crates/tools/src/tool_use.rs:7-18`, `crates/tools/src/skill/get_reference.rs:26-40`, and `crates/tools/src/skill/skill_script.rs:30-46`.
- Sidebar navigation is session-centric and only shows top-level sessions where `parentId === null` in `src/components/organisms/trees/sessions-tree.tsx:22-75`, `src/components/organisms/trees/sessions-tree.viewmodel.tsx:44-52`, and `src/components/organisms/trees/session-tree.tsx:79-176`.
- Session form already exposes personality plus governance-ish controls in `src/components/forms/session/session-form.tsx:24-88` and `src/components/forms/session/sections/personality-select.tsx:17-65`.

## What this phase changes
- Add project-local org roles and manager relationships to employees.
- Shift the sidebar from session-first to employee-first, with sessions nested under employees.
- Allow managers to assign persistent skills to employees.
- Keep `list_skills` disabled by default; assigned skills are curated, not rummaged live.
- Use role/governance metadata on employees rather than exploding `AgentKind` into `manager`, `intern`, `cto`, and other nonsense.

## Three options considered
### Option A — Encode org roles as more `AgentKind` values
Turn the runtime kind enum into the org chart.

### Option B — Soft roles in UI only
Keep runtime unchanged and let the UI fake manager/employee relationships.

### Option C — Project-scoped org roles with manager oversight and employee-first UX
Persist manager relationships, employee roles, assigned skills, and surface them in the project UI.

## Winner and why
Winner: Option C.

Why:
- `AgentKind` already means runtime capability shape. Overloading it with org rank is how systems become unreadable.
- Durable assigned skills belong to employees, not ephemeral session context.
- Employee-first navigation matches the new identity model and gives managers an actual control surface.

Rejected:
- Option A: `AgentKind` explosion is a trap.
- Option B: fake governance in UI-only state is not governance.

## Concrete code touchpoints
- `crates/common/src/agent/types.rs:22-31`
  - Keep this runtime taxonomy small; do not mutate it into org structure.
- `crates/common/src/agent/allowlist.rs:45-165`
  - Governance should compose with permissions here, but not by multiplying agent kinds.
- `crates/common/src/personality_files.rs:8-12` and `crates/common/src/personality_service.rs:95-200`
  - Existing personality system is the right substrate for employee-type prompt layers.
- `crates/tools/src/tool_use.rs:7-18`
  - Tool context needs durable employee-assigned skills, not only transient `current_skills`.
- `crates/tools/src/skill/get_reference.rs:26-40`
  - Today resolves from runtime skills only; phase 4 should source persistent employee assignments into runtime.
- `crates/tools/src/skill/skill_script.rs:30-46`
  - Same gap for executable skills.
- `src/components/organisms/trees/sessions-tree.tsx:22-75`
  - Current root is “Create New Session”; phase 4 should make employee nodes first-class.
- `src/components/organisms/trees/sessions-tree.viewmodel.tsx:44-52`
  - Current filtering by `parentId === null` proves the tree is session-first.
- `src/components/forms/session/session-form.tsx:24-88`
  - Existing form can absorb role/manager assignment without inventing a new creation flow.

## Risks / anti-patterns
- Turning org roles into `AgentKind` variants.
- Enabling unbounded skill browsing/discovery by default; curated assignment is safer.
- Putting manager relationships only in UI state.
- Forgetting that project-scoped orgs still need session-level overrides for one-off work.

## Definition of done
- Employees have project-scoped role metadata and optional manager assignment.
- Persistent skills can be assigned to employees and materialized into runtime skill context.
- Sidebar/navigation is employee-first with sessions beneath employees.
- Session creation/editing can select or show role/manager context.
- `list_skills` remains off by default.
- `AgentKind` remains a compact runtime taxonomy, not the org chart.