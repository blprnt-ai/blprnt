# CSS Variables & Tokens

## Intent
Use CSS variables to keep design decisions consistent and reusable.

## Recommendations
- Define tokens for color, spacing, radius, and elevation.
- Use semantic names (e.g., --surface, --surface-muted, --accent).
- Keep raw values centralized and map them to semantic roles.

## Example
```css
:root {
  --surface: #0f1012;
  --surface-muted: #15181c;
  --border-subtle: #1f242a;
  --accent: #6d8dff;
  --radius-sm: 6px;
  --radius-md: 10px;
  --shadow-soft: 0 6px 20px rgba(0, 0, 0, 0.18);
}
```
