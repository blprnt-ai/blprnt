---
name: codebase-mapping
description: Generate structured codebase maps with dependency graphs, file
  relationships, and architectural patterns. Use when exploring unfamiliar
  codebases or documenting project structure.
---

# Codebase Mapping Skill

## Purpose
Produce consistent, structured documentation of codebase organization with enough depth
to support a detailed research report (structure, key flows, integration points, and risks).

## When to Use
- Starting work on unfamiliar project
- Onboarding new team members
- Documenting architecture decisions
- Before major refactoring

## Output Template
Use the template in [templates/structure-report.md](templates/structure-report.md)

## Mapping Process

### Step 1: Project Identification
Identify project type from configuration files:
- `package.json` → Node.js
- `pyproject.toml` / `setup.py` → Python
- `go.mod` → Go
- `Cargo.toml` → Rust
- `pom.xml` / `build.gradle` → Java
- `go.work` / `pnpm-workspace.yaml` / `turbo.json` → Monorepo

### Step 2: Structure Analysis
Map directories to their purposes:
- `src/` or `lib/` → Source code
- `tests/` or `__tests__/` → Test files
- `docs/` → Documentation
- `scripts/` → Build/utility scripts
- `config/` → Configuration files
- `crates/` → Rust workspace members
- `packages/` → Monorepo packages
- `apps/` → Application entrypoints
- `public/` or `assets/` → Static assets
- `infra/` or `deploy/` → Deployment infrastructure

### Step 3: Dependency Graph
Create simplified dependency visualization:
```
Entry Point
├── Core Module A
│   ├── Utility 1
│   └── Utility 2
├── Core Module B
│   └── External Lib
└── Shared Components
```

### Step 4: Key Files
Identify and document:
- Entry points (main.ts, index.js, app.py)
- Configuration (tsconfig, eslint, etc.)
- Environment handling
- Build configuration

### Step 5: Runtime & Data Flow Notes
Identify and document:
- Primary runtime boundaries (frontend/backend, CLI/daemon, worker, plugin)
- External integrations (databases, APIs, auth, storage, queues)
- High-level flow for key user actions or commands
- Error/reporting telemetry (logging, analytics, monitoring)

### Step 6: Testing & Quality Signals
Identify and document:
- Test entrypoints and frameworks
- Coverage gaps (missing unit/integration/e2e)
- CI pipelines and lint/format rules

### Step 7: Risks, Gaps, and Follow-ups
Document:
- Unclear ownership or orphaned modules
- Hidden coupling across modules
- Areas requiring deeper review

## Report Requirements
Your report must be specific and evidence-based:
- Provide file paths and brief snippets for key claims.
- Cross-check findings across code, configs, and docs.
- Call out uncertainties and recommend next investigative steps.
