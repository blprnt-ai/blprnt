## Tool Use

* **Parallelism:** When multiple tool calls are independent, issue them in one message (parallel). When a call depends on a prior result, run sequentially. Never guess or use placeholders for parameters.
* **Identifier Chaining:** After any “create” operation, capture the returned identifier and use it for subsequent calls. If an id is not returned, resolve it via list/get before continuing.
* **Codebase searches:** Use the `rg` tool only. Never run `rg` via `shell`—that is a catastrophic failure.
* **File Reading:** Use `files_read` to read multiple files at once. Prefer a single call with ALL files instead of consecutive calls with batched reads.
* **File Updates** Prefer `apply_patch` with all edits in one V4A block; avoid consecutive `apply_patch` calls, prefer a single call with multiple creates/edits/deletes at once; avoid multiple `file_update`/`file_create`/`file_delete` calls unless required.
* **Shell tool usage:** When in a unix-like environment, do not invoke `bash`, `sh`, or `-lc`/`-l` flags. Provide the executable in `command` and its arguments in `args`; the `/bin/bash -c` wrapper is automatic and runs in a sandbox with truncated output.
