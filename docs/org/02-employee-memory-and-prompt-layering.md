# Phase 2 — Employee Memory and Prompt Layering

## Overview
Now that Phase 0 hardened core memory and Phase 1 introduced employees, shift memory and prompt composition from project-centric to employee-centric while staying inside the existing per-project QMD footprint. No global memory pool.

## What we have now
- Memory writes/search args expose only `content`, `query`, and `limit` in `crates/common/src/tools/memory.rs:9-42`.
- Runtime prompt assembly pulls app-wide summary plus project-scoped QMD results using `QmdMemorySearchService::new(self.project_id.key().to_string())` in `crates/engine_v2/src/runtime/context.rs:221-237`.
- QMD collections are created per project and named `memories-{project_id}` in `crates/common/src/memory/qmd.rs:115-138,141-177,269-271`.
- Managed memory summary reads from a single root summary/today view in `crates/common/src/memory/store.rs:190-220`.
- Dirty memory extraction is keyed off top-level sessions, not employees, in `crates/persistence/src/models_v2/messages.rs:466-499` and scheduled per project in `crates/app_core/src/engine_manager.rs:351-391`.
- Employee identity exists after Phase 1, but prompt/memory composition still needs to become meaningfully employee-first.

## What this phase changes
- Make employee identity the primary retrieval and write boundary inside a project.
- Keep one QMD collection per project, but attach employee metadata to memory entries and query with employee filters first.
- Fall back to project-only retrieval when employee-local memory is sparse.
- Layer prompt memory as: employee-local memory, then project-shared memory, then session-specific working context.

## Three options considered
### Option A — One QMD collection per employee
Create a fresh collection for every employee in every project.

### Option B — Keep project collection but duplicate memory into employee and project documents
Write separate documents for employee and shared views.

### Option C — Keep per-project collections and add tenant-style employee metadata filters
Store employee ownership metadata inside the existing project collection and query with filter-first, fallback-second behavior.

## Winner and why
Winner: Option C.

Why:
- Existing QMD lifecycle is already project-scoped. Reusing it avoids a sidecar/bootstrap explosion.
- The OpenViking isolation report supports layered identity plus metadata-filtered retrieval as the sane middle path.
- Employee-first retrieval with project fallback matches how real teams work: private context first, shared context second.

Rejected:
- Option A: too many collections, too much indexing churn, too much operational nonsense.
- Option B: duplicates source of truth and guarantees drift.

## Concrete code touchpoints
- `crates/common/src/tools/memory.rs:9-42`
  - Tool args are too narrow for explicit employee-aware writes/search; phase 2 needs runtime-derived employee scoping or contract expansion.
- `crates/common/src/memory/qmd.rs:115-138,141-177,269-271`
  - Reuse per-project collection setup; add metadata-aware query/write behavior instead of multiplying collections.
- `crates/common/src/memory/store.rs:190-220`
  - Current summary builder is one-root oriented and must become employee-aware.
- `crates/persistence/src/models_v2/messages.rs:39-45,466-499`
  - Dirty extraction currently starts from top-level sessions; employee ownership must be available during extraction.
- `crates/app_core/src/engine_manager.rs:351-391`
  - Periodic sweep orchestration already exists and should stay project-scoped while sweeping employee-tagged memories.
- `crates/engine_v2/src/runtime/context.rs:221-237`
  - This is where employee-local retrieval with project fallback should be assembled.
- `docs/deep-research-report-open-viking.md:3-12,91-109`
  - Reference for tenant metadata boundaries, layered identity, and filtered retrieval instead of isolated deployments.

## Risks / anti-patterns
- Creating employee-private collections per project. That scales like a dumpster fire.
- Hiding employee scope entirely in prompt text rather than in retrieval metadata.
- Over-filtering and starving employees of shared project knowledge.
- Keeping summary docs project-only while pretending memory is employee-first.

## Definition of done
- Memory entries can be attributed to an employee within a project.
- Retrieval attempts employee+project scope first, then project-only fallback.
- Prompt memory layering is employee-first, project-second.
- Sweep/extraction logic preserves project-scoped runtime ownership while tagging employee identity.
- QMD remains one collection per project.
- Phase still avoids any global or cross-project org memory.