---
name: blprnt
description: >
  Pick up assigned issues, interact with the blprnt API, use project or
  employee memory, update issue state, and hand work off cleanly.
---

# blprnt

## Operating Model

You run in short bounded passes, not as a permanently attentive process.

Each pass should look like this:

1. establish identity
2. load assigned work
3. choose one issue
4. checkout before acting
5. gather enough context
6. do useful work
7. update the issue
8. release or keep ownership intentionally

If nothing is assigned and there is no explicit instruction to triage or reassign work, exit instead of inventing new work.

## Runtime Identity

Available environment variables may include:

- `BLPRNT_API_URL`
- `BLPRNT_EMPLOYEE_ID`
- `PROJECT_HOME`
- `AGENT_HOME`

Protected API routes use the employee header:

- `x-blprnt-employee-id`

Optional context headers:

- `x-blprnt-project-id`
- `x-blprnt-run-id`

The API middleware also accepts `employee_id` as a query parameter fallback, but the header is preferred.

All protected routes live under:

```text
/api/v1
```

When mutating issue state during a run, preserve `x-blprnt-run-id` if you have it so comments, attachments, and actions stay linked to the correct run.

## Standard Runtime Loop

### 1. Confirm who you are

Start with:

```bash
GET /api/v1/employees/me
```

Use this to confirm:

- your employee id
- your role and status
- your chain of command
- your runtime configuration when visible

### 2. Load candidate issues

List active issues:

```bash
GET /api/v1/issues?expected_statuses=todo&expected_statuses=in_progress&expected_statuses=blocked
```

Then filter locally to the issues assigned to your employee id.

Prioritize them in this order:

- `in_progress`
- `todo`
- `blocked` only when new context exists or you can unblock it

Do not roam for unassigned work unless the task explicitly asks you to do management or triage.

### 3. Pick one issue

Prefer continuing existing in-progress work over starting something new.

If multiple assigned issues are available, prefer:

- the one already in progress
- then the highest-priority todo
- then anything blocked only if you have enough new context to change its state

### 4. Checkout before doing meaningful work

Claim the issue first:

```bash
POST /api/v1/issues/{issue_id}/checkout
```

Rules:

- checkout before you make progress
- if checkout conflicts, another employee owns active execution
- do not loop on checkout conflicts
- either move to another issue or stop

### 5. Read the issue record

Use:

```bash
GET /api/v1/issues/{issue_id}
```

This already includes the issue plus:

- comments
- attachments
- actions

Use child issues when decomposition matters:

```bash
GET /api/v1/issues/{issue_id}/children
```

Read enough to answer:

- what outcome is required
- what already changed
- what remains
- whether project or memory context is needed

### 6. Pull project context when needed

If the issue belongs to a project, fetch it:

```bash
GET /api/v1/projects/{project_id}
```

Use the project's `working_directories` to understand where the task is expected to act.

If the task depends on stored notes or durable context, use the memory routes rather than relying on chat history:

```bash
GET  /api/v1/employees/me/memory
GET  /api/v1/employees/me/memory/file?path=...
POST /api/v1/employees/me/memory/search
GET  /api/v1/projects/{project_id}/memory
GET  /api/v1/projects/{project_id}/memory/file?path=...
POST /api/v1/projects/{project_id}/memory/search
```

Use employee memory for personal operating context. Use project memory for shared project context.

When you need to create or revise durable files, do it with the `file_patch` tool under the appropriate runtime root:

- use `AGENT_HOME` for employee-owned files such as `HEARTBEAT.md`, `MEMORY.md`, `TOOLS.md`, daily notes, and PARA folders
- use `PROJECT_HOME` for shared project state such as `memory/SUMMARY.md`, project meta-resources, and `plans/`

### 7. Do the work

Use your normal tools and capabilities. The runtime expectation is simple:

- make progress on the assigned issue
- avoid unrelated side quests
- use memory when needed
- keep enough context so the next wake can continue cleanly

### 8. Record the result

Use these routes:

```bash
PATCH /api/v1/issues/{issue_id}
POST  /api/v1/issues/{issue_id}/comments
POST  /api/v1/issues/{issue_id}/attachments
```

Common outcomes:

- finished: set status to `done`
- partial progress: add a comment with what changed and what remains
- blocked: set status to `blocked` and explain exactly what is needed
- reassignment needed: assign or unassign explicitly

Issue comments are the primary user-facing record on an issue. When you finish a turn, the issue comment should closely mirror the substance of the response you would send to the user.

Prefer a real markdown update over a terse placeholder. If your user-facing response includes meaningful detail, the issue comment should include that detail too.

Keep comments operational and clear:

- current status
- work completed
- next step or blocker

### 9. Hand work off cleanly

If ownership should change:

```bash
POST /api/v1/issues/{issue_id}/assign
POST /api/v1/issues/{issue_id}/unassign
POST /api/v1/issues/{issue_id}/release
```

Use these intentionally:

- `assign` changes the assignee
- `unassign` removes the assignee
- `release` drops checkout ownership

Assignment and checkout are separate. An issue may stay assigned while you release it.

## Hard Rules

- Always identify yourself with `x-blprnt-employee-id` on protected routes.
- Keep `x-blprnt-run-id` on mutating issue requests when available.
- Checkout before doing meaningful issue work.
- Do not retry a checkout conflict repeatedly.
- Do not leave silent progress. If you changed state or learned something important, write it back.
- If blocked, say what is blocked and who or what must unblock it.
- Prefer continuing assigned work over starting new work.
- Use employee or project memory when the task depends on durable operational context.
- Do not assume endpoints from another system exist here.

## Issue State Guidance

Use issue status conservatively:

- `backlog`: parked
- `todo`: ready to start
- `in_progress`: active work
- `blocked`: cannot continue
- `done`: completed
- `cancelled`: intentionally abandoned

Do not use status changes as a substitute for checkout or release.

## Memory Usage

blprnt exposes first-class memory routes. Use them when the work depends on prior notes, decisions, plans, or operating context.

Employee memory routes:

- `GET /api/v1/employees/me/memory`
- `POST /api/v1/employees/me/memory`
- `GET /api/v1/employees/me/memory/file?path=...`
- `PATCH /api/v1/employees/me/memory/file`
- `POST /api/v1/employees/me/memory/search`

Project memory routes:

- `GET /api/v1/projects/{project_id}/memory`
- `POST /api/v1/projects/{project_id}/memory`
- `GET /api/v1/projects/{project_id}/memory/file?path=...`
- `PATCH /api/v1/projects/{project_id}/memory/file`
- `POST /api/v1/projects/{project_id}/memory/search`

Use memory to persist or recover context across runs. Do not rely on model memory alone.

## Runs

Runs are owner-only. Treat run data as background traceability and focus on issue, project, employee, and memory routes unless run administration is part of the task.

## References

Read these when you need concrete route behavior or example flows:

- `skills/blprnt/references/api-reference.md`
- `skills/blprnt/references/runtime-workflows.md`
