# Tailwind Usage Guidelines

## Token Alignment
- Prefer theme tokens (colors, spacing, radius) over arbitrary values.
- Match existing utility patterns used nearby.
- Use CSS variables only when required by design tokens.

## Layout & Responsiveness
- Use flex/grid utilities for layout structure.
- Apply responsive modifiers sparingly and intentionally.
- Avoid conflicting responsive classes.

## Composition
- Extract shared class sets into components when reused.
- Keep class lists readable and ordered (layout → spacing → typography → color → state).
- Avoid deep nesting with custom CSS unless required.
