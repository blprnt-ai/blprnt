## GritQL Tool Reference

GritQL is a declarative language for searching and transforming source code. Use it when you need to find, lint, or refactor code patterns.

### Syntax Fundamentals

**Patterns** are code snippets in backticks that match structurally equivalent code:
```
`console.log($msg)`
```

**Metavariables** capture matched nodes:
- `$name` — captures single node
- `$...items` — captures zero or more nodes (spread)
- `$_` — wildcard, matches but discards

**Rewrites** transform matches using `=>`:
```
`var $x = $y` => `const $x = $y`
```

### Operators

| Operator | Usage |
|----------|-------|
| `where { }` | Add conditions: `\`$fn($x)\` where { $fn <: \`eval\` }` |
| `<:` | Match operator (left matches right pattern) |
| `and` | Both must match |
| `or { }` | Any branch matches |
| `not` | Negate: `$x <: not \`undefined\`` |
| `contains` | Node contains pattern: `$body <: contains \`return\`` |
| `includes` | String contains: `$s <: includes "foo"` |
| `as` | Bind match: `$call as $original` |
| `maybe` | Optional match |

### Common Patterns

**Find function calls:**
```
`$fn($...args)` where { $fn <: `dangerousFunction` }
```

**Match imports:**
```
`import $binding from $source` where { $source <: includes "lodash" }
```

**Match object properties:**
```
`{ $key: $value }` where { $key <: `password` }
```

**Multiple rewrites:**
```
or {
  `require($path)` => `import($path)`,
  `module.exports = $x` => `export default $x`
}
```

**Nested conditions:**
```
`function $name($...params) { $...body }` where {
  $body <: contains `await $_`,
  $name <: not includes "async"
}
```

**Sequential statements:**
```
`$x = $y; $z = $x;` => `$z = $y;`
```

### Best Practices

1. **Start specific, broaden if needed** — Match exact structure first, generalize with metavariables
2. **Use `$_` for irrelevant captures** — Keeps patterns readable
3. **Prefer `contains` over deep nesting** — `$block <: contains \`return null\`` vs manually matching structure
4. **Use `$...` for variadic positions** — Arguments, array elements, statement blocks
5. **Test with `--dry-run` first** — Preview changes before applying
6. **Quote strings in conditions** — `$x <: includes "text"` not `$x <: includes text`

### Language-Specific Notes

Specify language when ambiguous. GritQL parses according to target language grammar—a pattern for JS won't match Python syntax.

Supported: JavaScript, TypeScript, Python, Rust, Go, Ruby, Java, C#, SQL, HTML, CSS, JSON, YAML, Markdown, and more.

### Output Format

When generating GritQL, output only the pattern. For complex transformations, use `or { }` to group related rewrites. Add brief comments with `//` if logic is non-obvious.