---
name: code-sanitizer
description: Detect and clean up AI-slop in code: vague abstractions, unnecessary complexity, over-abstraction, generic naming, dead helpers, duplicate logic, placeholder comments, speculative generality, inconsistent patterns, and low-signal boilerplate. Use when code feels over-generated, repetitive, too abstract, more complex than necessary, unlike the surrounding codebase, or the user asks to sanitize, de-slop, simplify, clean up, or make code feel more human and maintainable.
---

# Code Sanitizer

Make code feel intentional again.

This skill focuses on identifying AI-generated slop, unnecessary complexity, and over-abstraction, then replacing them with smaller, clearer, more idiomatic code that matches the local codebase.

## When to Use

- The user says code feels "AI-generated", "sloppy", "generic", "bloated", or "off"
- A patch introduces excessive helpers, wrappers, or abstractions with little value
- Naming is vague, repetitive, or disconnected from the domain
- The code solves the task, but not in a way a careful teammate would usually write
- The implementation ignores nearby patterns and invents its own structure
- Comments explain obvious code or repeat what the code already says
- The code introduces extra layers, types, helpers, or files without a clear payoff

## Primary Goal

Do not just make the code shorter.

Make it:

- easier to read
- more local and direct
- more idiomatic for the language and framework
- more consistent with surrounding files
- less generic and more domain-shaped
- no more abstract than the problem requires

## Opinionated Defaults

Prefer:

- one clear function over a stack of thin wrappers
- domain names over abstract names like `data`, `handler`, `processor`, `manager`, `util`
- existing project patterns over invented patterns
- explicit logic over fake configurability
- deleting dead code over preserving speculative hooks
- fewer layers, fewer files, and fewer concepts when they are not earning their keep
- concrete code shaped around the current problem, not a hypothetical future framework

Avoid:

- "reusable" abstractions with only one caller
- helpers that simply rename a built-in or framework primitive
- comments that narrate the obvious
- generic fallback names such as `CustomManager`, `DataProcessor`, `HelperUtils`
- interfaces, traits, or types introduced only to look architected
- placeholder validation, placeholder error handling, or placeholder TODO-driven structure
- complexity added for symmetry, extensibility, or elegance without present-day need

## What Counts as AI-Slop

Look for these smells:

### 1. Generic naming

Examples:

- `data`
- `item`
- `payload`
- `manager`
- `handler`
- `processor`
- `service` when the domain is more specific

Fix by renaming around the actual domain concept.

### 2. Speculative abstraction

Examples:

- helper layers with one caller
- traits/interfaces added without multiple meaningful implementations
- configuration objects that only hold two obvious values
- extraction of tiny functions that hurt readability instead of helping it

Fix by inlining or collapsing layers until the code matches current complexity.

### 3. Unnecessary complexity

Examples:

- multiple transformation steps when one would do
- extra files created for logic with a single call site
- state machines, factories, or registries for a straightforward flow
- configuration surfaces that make a simple path harder to follow

Fix by reducing the number of moving parts until the implementation matches the real problem size.

### 4. Boilerplate padding

Examples:

- many lines to do one obvious thing
- wrappers around `map`, `filter`, `match`, `fetch`, or simple constructors
- repeated "success/error/loading" scaffolding that can be expressed more directly

Fix by compressing to the simplest readable form.

### 5. Placeholder quality

Examples:

- comments like "Handle the logic here"
- fake validation that checks nothing meaningful
- error handling that just converts everything to a generic string
- TODO-shaped code presented as complete

Fix by implementing the real logic, deleting the placeholder, or leaving a clearly scoped follow-up note only when necessary.

### 6. Mismatched local style

Examples:

- different naming style than nearby files
- different error-handling pattern than the rest of the project
- introducing new architecture vocabulary into a codebase that does not use it
- adding utility modules where neighboring code keeps logic local

Fix by following the dominant local pattern, not the global average pattern.

### 7. Over-commenting

Examples:

- `// Set the value`
- `// Loop through items`
- comments that restate the next line

Keep comments only when they explain intent, invariants, or a non-obvious tradeoff.

### 8. Duplicate logic with cosmetic variation

Examples:

- copied helpers with renamed variables
- nearly identical branches for cases that can share a common path
- repeated normalization or formatting logic spread across files

Fix by unifying real duplication, but only after confirming the shared shape is genuine.

## Sanitizing Workflow

### 1. Detect

Scan the target code for:

- vague names
- repeated patterns
- dead indirection
- unnecessary files or layers
- abstraction that exceeds the actual problem
- weak comments
- style mismatches with neighboring code

### 2. Validate against the local codebase

Before changing structure, inspect nearby files and ask:

- How is this usually named here?
- How is error handling usually done here?
- Do similar features stay local or get extracted?
- Does this project prefer explicitness or abstraction?

Do not sanitize toward your favorite style. Sanitize toward the repo's style.

### 3. Classify each issue

For each smell, decide whether the right move is:

- rename
- inline
- delete
- merge
- simplify
- de-abstract
- re-structure
- leave alone

Only change what has a clear quality payoff.

### 4. Fix in priority order

1. correctness risks hidden by noisy code
2. unnecessary complexity and misleading abstractions
3. poor naming
4. duplication
5. noisy comments and formatting-level clutter

### 5. Re-check for overcorrection

After cleanup, confirm:

- the code is still easy to modify
- you did not collapse meaningful boundaries
- you did not "optimize" away useful explicitness
- the final code reads like it belongs in this repo

## Refactoring Moves

Use these aggressively when justified:

- Inline single-use helpers
- Merge tiny wrapper functions
- Rename generic identifiers to domain terms
- Delete dead parameters and unused configuration
- Collapse unnecessary types
- Remove abstraction layers that only obscure the happy path
- Move logic back next to its only caller
- Replace comment-heavy code with clearer code
- Reuse existing utilities instead of creating parallel ones

Use these cautiously:

- introducing a new shared abstraction
- extracting a helper for future reuse
- adding a trait/interface purely for testability
- splitting code across more files

## Output Expectations

When reviewing or explaining slop, call out:

1. what feels AI-generated or low-signal
2. why it hurts maintainability
3. what the cleaner shape should be

When editing, prefer direct fixes over long lectures.

## Quick Review Checklist

- Are names domain-specific?
- Is every layer earning its existence?
- Is the solution more abstract than the problem?
- Is any helper only hiding obvious logic?
- Are comments explaining intent instead of narrating syntax?
- Does this match surrounding project patterns?
- Is any abstraction solving a real current problem instead of an imagined future one?
- Can a teammate understand the main path without bouncing between files?

## Good Outcomes

Good sanitized code usually feels:

- calmer
- more direct
- less performative
- easier to change
- more obviously written for this codebase

That is the target.
