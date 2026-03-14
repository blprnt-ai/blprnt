---
name: rust
description: "General Rust language guidance for building safe, maintainable, and performant software. Use when the user is working in Rust, designing APIs, handling ownership and lifetimes, structuring crates, or choosing common Rust ecosystem tools."
---

# Rust language

Practical Rust guidance for day-to-day implementation work across libraries, services, CLIs, async systems, and application code.

## When to Use

- The task involves `.rs` files or `Cargo.toml`
- The user asks about ownership, borrowing, lifetimes, traits, or generics
- The code uses `Result`, `Option`, iterators, enums, pattern matching, or macros
- The project needs Rust ecosystem choices like `serde`, `tokio`, `clap`, `thiserror`, or `tracing`
- The task needs safe systems programming or performance-sensitive implementation

## Opinionated Defaults

Prefer the simplest, idiomatic option that preserves correctness:

- Model invalid states with types instead of runtime checks where practical.
- Prefer `Result` and `?` for recoverable failures.
- Avoid `unwrap()` and `expect()` in production paths.
- Borrow first, clone only when ownership genuinely needs to cross a boundary.
- Prefer enums over stringly typed branching.
- Keep modules small and responsibilities obvious.
- Use `Vec`, `HashMap`, and slices before reaching for exotic data structures.
- Reach for `tokio` only when the program is truly async or heavily concurrent.
- Treat `unsafe` as a last resort and document every safety invariant with `// SAFETY:`.

## Core Language Patterns

### Ownership and Borrowing

Default to references when a function only needs to inspect data:

```rust
fn display_name(user: &User) -> &str {
    &user.name
}

fn rename_user(user: &mut User, new_name: String) {
    user.name = new_name;
}
```

Prefer owned return values when the callee creates new data:

```rust
fn slugify(input: &str) -> String {
    input.trim().to_lowercase().replace(' ', "-")
}
```

### `Result` and `Option`

Use `?` to propagate errors and `ok_or_else` when converting from `Option`:

```rust
fn parse_port(input: &str) -> Result<u16, AppError> {
    let port: u16 = input.parse()?;
    if port == 0 {
        return Err(AppError::InvalidPort);
    }
    Ok(port)
}

fn primary_email(user: &User) -> Result<&str, AppError> {
    user.email.as_deref().ok_or_else(|| AppError::MissingEmail)
}
```

### Pattern Matching and `let-else`

Use pattern matching to make branching explicit:

```rust
fn active_user_name(user: Option<User>) -> Result<String, AppError> {
    let Some(user) = user else {
        return Err(AppError::NotFound);
    };

    if !user.is_active {
        return Err(AppError::InactiveUser);
    }

    Ok(user.name)
}
```

### Traits and Generics

Reach for generics when behavior is shared and types differ:

```rust
trait Store<T> {
    fn insert(&mut self, value: T);
    fn get(&self, index: usize) -> Option<&T>;
}

struct InMemoryStore<T> {
    items: Vec<T>,
}

impl<T> Store<T> for InMemoryStore<T> {
    fn insert(&mut self, value: T) {
        self.items.push(value);
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }
}
```

### Iterators

Prefer iterator chains when they stay readable; switch to a loop when the logic becomes opaque:

```rust
fn even_squares(values: &[i32]) -> Vec<i32> {
    values
        .iter()
        .copied()
        .filter(|value| value % 2 == 0)
        .map(|value| value * value)
        .collect()
}
```

## Project Structure

Choose structure based on the size of the program:

- Small library: keep the public API in `lib.rs`, push implementation details into a few focused modules.
- Binary crate: keep `main.rs` thin and delegate work into modules or a library crate.
- Growing app: separate domain logic, I/O boundaries, and glue code.
- Workspace: use when multiple crates need clear boundaries or different build targets.

Example library layout:

```text
src/
├── lib.rs
├── error.rs
├── models.rs
├── parser.rs
└── service/
    ├── mod.rs
    └── cache.rs
```

## Common Ecosystem Choices

Use crates deliberately rather than by habit:

- `serde`: serialization and deserialization
- `thiserror`: ergonomic application error enums
- `anyhow`: top-level application error aggregation
- `tokio`: async runtime for networked or concurrent apps
- `clap`: CLI argument parsing
- `tracing`: structured logging and spans
- `reqwest`: HTTP clients
- `sqlx`: compile-time checked SQL when a SQL-first workflow fits
- `axum`: web services when building an HTTP API

Prefer the standard library first when it already solves the problem cleanly.

## Async and Concurrency

Do not make code async unless it needs async I/O or concurrency.

### Async Basics

Use async for network, database, file, or timer-driven workflows:

```rust
async fn fetch_with_timeout() -> Result<Data, AppError> {
    tokio::select! {
        result = fetch_data() => result,
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => Err(AppError::Timeout),
    }
}
```

### Shared State

Prefer message passing or clear ownership. If shared mutable state is unavoidable, choose the narrowest synchronization primitive that fits:

- `Arc<T>` for shared read-only ownership
- `Arc<Mutex<T>>` for infrequent mutable access
- `Arc<RwLock<T>>` when reads clearly dominate writes
- channels when work should be passed rather than shared

### Concurrency Limits

Limit fan-out explicitly instead of spawning unbounded work:

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

async fn process_all(items: Vec<String>, max_in_flight: usize) -> Result<(), AppError> {
    let semaphore = Arc::new(Semaphore::new(max_in_flight));
    let mut handles = Vec::new();

    for item in items {
        let semaphore = Arc::clone(&semaphore);
        handles.push(tokio::spawn(async move {
            let permit = semaphore
                .acquire_owned()
                .await
                .map_err(|_| AppError::TaskCancelled)?;
            let result = process_item(item).await;
            drop(permit);
            result
        }));
    }

    for handle in handles {
        handle.await??;
    }

    Ok(())
}
```

## Error Handling

Use typed errors inside libraries and domain logic:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid port")]
    InvalidPort,
    #[error("missing email")]
    MissingEmail,
    #[error("user not found")]
    NotFound,
    #[error("inactive user")]
    InactiveUser,
    #[error("operation timed out")]
    Timeout,
    #[error("task was cancelled")]
    TaskCancelled,
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}
```

Use `anyhow` at application boundaries when you need fast aggregation instead of a carefully modeled error surface.

## Serialization

`serde` is the default choice for structured data:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserProfile {
    id: u64,
    display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    #[serde(default)]
    is_active: bool,
}
```

## Testing

Test behavior, not implementation details:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_lowercases_and_replaces_spaces() {
        let slug = slugify("Hello World");
        assert_eq!(slug, "hello-world");
    }
}
```

Use:

- unit tests for pure logic
- integration tests for public behavior
- property tests when invariants matter more than examples
- async tests only when async behavior is the thing being exercised

## Tooling

Prefer the project's documented workflow first.

Typical verification order:

1. `cargo check`
2. `cargo test`
3. `cargo fmt`
4. `cargo clippy`

If a `justfile` exists, prefer its recipes over assuming raw cargo commands.

## Common Pitfalls to Avoid

- Fighting the borrow checker with unnecessary `clone()`
- Returning owned data when a borrowed reference would do
- Making everything generic when a concrete type would be clearer
- Using async for CPU-bound work
- Holding a lock across `.await`
- Treating `String` and `&str` as interchangeable without thinking about ownership
- Hiding domain states in booleans or loose strings instead of enums
- Reaching for `unsafe` before proving the safe approach is inadequate

## Troubleshooting

Common checks:

- Compilation errors: `cargo check`
- Test failures: `cargo test`
- Formatting drift: `cargo fmt --check`
- Lint issues: `cargo clippy`
- Dependency graph confusion: `cargo tree`
- Toolchain mismatch: `rustc --version` and `cargo --version`

When stuck on a borrow checker error:

1. Identify who owns the value.
2. Decide whether the callee needs shared access, mutable access, or ownership.
3. Shorten borrow lifetimes by introducing smaller scopes.
4. Clone only after confirming ownership transfer is actually required.

## Additional Resources

See [references/reference.md](references/reference.md) for broader language notes and ecosystem references.

See [examples/example.md](examples/example.md) for code examples.
