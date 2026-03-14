---
name: clean-code
description: Enforce SOLID, DRY, KISS principles during implementation.
  Auto-activated when writing or modifying code.
---

# Clean Code Skill

## Purpose
Ensure all code follows clean code principles.

## Core Principles

### SOLID
Reference: [clean-code/principles/solid.md](clean-code/principles/solid.md)

### DRY (Don't Repeat Yourself)
Reference: [clean-code/principles/dry.md](clean-code/principles/dry.md)
- Extract common logic to functions
- Use constants for magic values
- Create shared utilities

### KISS (Keep It Simple, Stupid)
Reference: [clean-code/principles/kiss.md](clean-code/principles/kiss.md)
- Favor readability over cleverness
- One thing per function
- Obvious over implicit

### YAGNI (You Aren't Gonna Need It)
Reference: [clean-code/principles/yagni.md](clean-code/principles/yagni.md)
- Implement only what's needed now
- No speculative generalization
- Add complexity when required

## Code Quality Checklist
Use before committing: [clean-code/checklists/pre-commit.md](clean-code/checklists/pre-commit.md)

### Naming
- [ ] Variables describe content
- [ ] Functions describe action
- [ ] Classes describe entity
- [ ] No abbreviations (except common ones)

### Functions
- [ ] Single responsibility
- [ ] < 20 lines preferred
- [ ] < 5 parameters
- [ ] No side effects where possible

### Comments
- [ ] Explain why, not what
- [ ] Update when code changes
- [ ] Remove commented-out code

### Error Handling
- [ ] Specific error types
- [ ] Meaningful messages
- [ ] Proper logging
- [ ] Recovery or fail gracefully

## Auto-Checks
When implementing, verify:
```bash
npm run lint
npm run typecheck
```
