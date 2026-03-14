---
name: task-breakdown
description: Use when converting architecture plans into implementation phases. Use planning tools to create plan items without creating files.
---

# Task Breakdown Skill

## Overview

Transform architecture plans into **independent implementation phases**. Represent phases as plan items using the planning tools, not files.

**Core principle:** Independent items first, dependent items last.

## When to Use

- After architecture plan is approved
- Converting design to executable phases
- Planning incremental implementation
- Enabling parallel work streams
- When the plan must stay within planning tools (no file creation)

## The Iron Law: Independence Ordering

```
INDEPENDENT ITEMS → TOP (implement first)
DEPENDENT ITEMS → BOTTOM (implement last)
```

**Why?** Independent phases can be:
- Implemented in parallel by multiple agents
- Verified without waiting for other phases
- Rolled back without breaking other phases

## Quick Reference: Planning Item Type

| Type | Description |
|------|-------------|
| **Plan** | Single-level planning item |

## Planning Output Format

Create planning items using the project planning tools. The output is a flat list of plan items.

```markdown
# Planning Items: [Feature]

**Status:** Proposed | In Progress | Completed

## Planning Summary

| Type | Name | Status |
|------|------|--------|
| Plan | [Plan 1] | Proposed |
| Plan | [Plan 2] | Pending |

## Execution Order

**Can implement in parallel:**
- Independent plan items with no dependencies

**Must wait:**
- Plan items that explicitly depend on other plan items

## Planning Details

Create planning items with the tool API:

- `plan_create` for new roadmap/spec/task items
- `plan_update` to refine status, title, description, priority
- `plan_list` to confirm siblings or list children
- `plan_get` to inspect a full subtree
- `plan_delete` only when removing mistakes
```

## Planning Item Format

Each planning item description is **self-contained**:

```markdown
# [Planning Item Title]

**Type:** Plan
**Dependencies:** None | [Plan Name]
**Priority:** [integer]

## Objective

[One sentence: what this item accomplishes independently]

## Entry Criteria

- [ ] [What must be true before starting]

## Exit Criteria

- [ ] Verification passes
- [ ] No impact on other items

## Verification

```bash
# Item-specific verification
[commands]
```
```

## Dependency Sorting Algorithm

**Step 1:** List all components from architecture
**Step 2:** For each component, identify what it imports/uses
**Step 3:** Score by dependency count:
- 0 dependencies = early specs/tasks
- 1-2 dependencies = middle specs/tasks
- 3+ dependencies = late specs/tasks

**Step 4:** Within same dependency count, order by:
1. Config/Types first (foundational)
2. Services/Models second
3. Controllers/UI third
4. Integration/E2E last

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Creating files for the plan | Use `plan_create` items instead |
| Dependent items at top | Reorder: independent first |
| Vague verification | Specific commands |

## Output Checklist

Before presenting plan:
- [ ] Planning items created via tools
- [ ] Plan items are a flat list
- [ ] Independent items listed first
- [ ] Dependent items at bottom
- [ ] Each item description is self-contained
- [ ] Verification commands provided
