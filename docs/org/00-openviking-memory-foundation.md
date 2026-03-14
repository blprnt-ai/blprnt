# Phase 0 — OpenViking Memory Foundation

## Overview
Before blprnt-corp exists, blprnt itself needs a better memory substrate. This phase upgrades core blprnt memory toward an OpenViking-style model: explicit identity metadata, layered retrieval, and cleaner boundaries between private and shared context. If this phase is skipped, every later org feature will be built on session cosplay and memory guesswork.

## What we have now
- Memory tools are still thin and generic in `crates/common/src/tools/memory.rs:9-42`; writes only take `content`, searches only take `query` and `limit`.
- Runtime prompt assembly injects app-global summary plus project-scoped QMD results in `crates/engine_v2/src/runtime/context.rs:221-237`.
- QMD search is instantiated per project via `QmdMemorySearchService::new(self.project_id.key().to_string())` in `crates/common/src/memory/qmd.rs:121-178`, with collection names shaped as `memories-{project_id}` in `crates/common/src/memory/qmd.rs:269-271`.
- Managed memory summaries are still effectively root-oriented in `crates/common/src/memory/store.rs:190-220`.
- Dirty extraction is driven from top-level sessions in `crates/persistence/src/models_v2/messages.rs:466-499` and swept per project in `crates/app_core/src/engine_manager.rs:351-391`.
- There is no durable employee identity in the current memory path, so retrieval is project-centric by default.

## What this phase changes
- Keep the current per-project QMD topology, but add explicit identity metadata so memory can later become employee-centric without tearing the system apart.
- Rework memory retrieval around layered filtering: scoped identity first, project shared context second.
- Treat OpenViking as the design reference for tenant-style boundaries and metadata-filtered retrieval, not as something to cargo-cult line by line.
- Finish this phase inside plain blprnt before starting blprnt-corp runtime work.

## Three options considered
### Option A — Jump straight to employee memory during org runtime work
Bolt identity-aware memory changes onto the org migration itself.

### Option B — Replace the entire memory stack with a brand-new external store first
Pause blprnt feature work and swap the whole substrate before doing anything else.

### Option C — Upgrade core blprnt memory first using OpenViking-style identity metadata and layered retrieval
Keep current project-scoped QMD collections, add better scoping semantics, and harden the memory system before org runtime phases.

## Winner and why
Winner: Option C.

Why:
- It fixes the actual dependency. blprnt-corp needs memory continuity more than it needs a fancy org chart.
- It keeps the current QMD/project structure intact while improving the semantics that ride on top of it.
- It gives you a real proving ground inside normal blprnt usage before org-swarms start piling on more moving parts.

Rejected:
- Option A: that just smears foundational memory work across later phases and guarantees mess.
- Option B: too much replacement risk, not enough leverage. New storage theology is not the bottleneck.

## Concrete code touchpoints
- `crates/common/src/tools/memory.rs:9-42`
  - Current tool contracts are too narrow to express richer scoped memory behavior directly.
- `crates/common/src/memory/qmd.rs:121-178,269-271`
  - Existing per-project QMD lifecycle should stay; filtering semantics should improve.
- `crates/common/src/memory/store.rs:190-220`
  - Summary generation must evolve beyond one broad root-oriented view.
- `crates/persistence/src/models_v2/messages.rs:39-45,466-499`
  - Dirty extraction is session-shaped today and must become compatible with future identity tagging.
- `crates/app_core/src/engine_manager.rs:351-391`
  - The periodic sweep coordinator is already the right place to harden memory behavior before org runtime changes.
- `crates/engine_v2/src/runtime/context.rs:221-237`
  - This is where scoped retrieval behavior is assembled into prompt context.
- `docs/deep-research-report-open-viking.md:3-12,91-109`
  - Reference for layered identity, tenant-style isolation, and metadata-filtered retrieval.

## Risks / anti-patterns
- Treating OpenViking like a copy-paste implementation recipe. It is a design input, not scripture.
- Creating per-employee collections immediately and exploding QMD management overhead.
- Leaving memory project-centric while pretending later phases are employee-centric.
- Doing this work inside blprnt-corp instead of hardening core blprnt first.

## Definition of done
- Core blprnt memory supports explicit identity-aware tagging and retrieval semantics.
- Retrieval can prefer narrower scoped context before broader project context.
- Existing project-scoped QMD collections remain in place.
- Sweep/extraction behavior is hardened enough to support later employee-centric phases.
- OpenViking-inspired boundaries are reflected in behavior, not just in vibes.
- No org runtime or global COO behavior is introduced yet.