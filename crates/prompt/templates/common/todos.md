## TODO

TODO items represent **concrete execution tasks** that move the plan forward. They are not for planning, research, reasoning, or validation.

### Core Rules

* A TODO must represent **a single actionable step** that can be executed immediately.
* A TODO must produce **a tangible outcome or artifact** (code, file edits, configuration, command execution, etc.).
* A TODO must be **specific enough that another agent could execute it without additional clarification**.

### What TODOs MUST NOT be

* TODO items must **not be research tasks** (e.g., “investigate”, “look into”, “research”, “explore”).
* TODO items must **not be verification tasks** (e.g., “check”, “confirm”, “validate”, “test whether”).
* TODO items must **not be planning or reasoning steps** (e.g., “decide”, “figure out”, “determine approach”).
* TODO items must **not duplicate information already covered in the plan section**.

The only exception to the research rule is when the **entire plan is explicitly research-only**. In that case, TODOs may represent structured research steps.

### Required Properties of a TODO

Each TODO must:

* Describe **an execution action**, not a thought process.
* Be **atomic** (one step, not multiple bundled actions).
* Be **deterministic** (clear completion criteria).
* Produce something that **can be verified later**, even though verification itself is not a TODO.

### Good TODO Examples

* Implement `MemoryStore::search()` using cosine similarity over embeddings.
* Add `priority` and `status` columns to the `tasks` database table.
* Update the API handler to return `404` when a task is not found.
* Create a migration for the new `roadmaps` table.

### Bad TODO Examples

* Research how vector search works.
* Investigate why the query might be slow.
* Verify that the API endpoint works.
* Decide how the database schema should look.
