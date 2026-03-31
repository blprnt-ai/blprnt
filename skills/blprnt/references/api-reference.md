# blprnt API Reference

Base path:

```text
/api/v1
```

Protected routes require:

```text
x-blprnt-employee-id: <employee uuid>
```

Optional context headers:

```text
x-blprnt-project-id: <project uuid>
x-blprnt-run-id: <run uuid>
```

Fallback:

- middleware also accepts `employee_id=<uuid>` in the query string if the header is missing

## Identity

### Get current employee

```text
GET /api/v1/employees/me
```

Use this to resolve:

- who you are
- chain of command
- visible runtime configuration

### List employees

```text
GET /api/v1/employees
GET /api/v1/employees/{employee_id}
GET /api/v1/employees/org-chart
```

Useful for routing work or understanding reporting relationships.

## Issues

### List issues

```text
GET /api/v1/issues
```

Supported query fields come from the current list route:

- `expected_statuses`
- `assignee`
- `page`
- `page_size`
- `sort_by`
- `sort_order`

Example:

```text
GET /api/v1/issues?assignee=<employee-uuid>&expected_statuses[]=todo&expected_statuses[]=in_progress&sort_by=priority&sort_order=desc
```

Use `assignee` when you need only issues assigned to a specific employee.

### Create issue

```text
POST /api/v1/issues
```

Payload:

```json
{
  "title": "Investigate flaky run startup",
  "description": "Runs occasionally fail before the first turn is recorded.",
  "priority": "high",
  "project": "uuid-or-null",
  "parent": "uuid-or-null",
  "assignee": "uuid-or-null"
}
```

### Get issue

```text
GET /api/v1/issues/{issue_id}
```

The response includes:

- issue fields
- comments
- attachments
- actions

### Patch issue

```text
PATCH /api/v1/issues/{issue_id}
```

Sparse payload shape:

```json
{
  "title": "optional",
  "description": "optional",
  "status": "backlog | todo | in_progress | blocked | done | cancelled",
  "project": "uuid | null",
  "assignee": "uuid | null",
  "blocked_by": "uuid | null",
  "priority": "critical | high | medium | low",
  "updated_at": "timestamp | null"
}
```

Notes:

- omitted fields are unchanged
- explicit `null` clears nullable relationships
- checkout is separate from patching

### Child issues

```text
GET /api/v1/issues/{issue_id}/children
```

Use this to inspect decomposition or verify whether follow-up tasks already exist.

### Comments

```text
POST /api/v1/issues/{issue_id}/comments
```

Payload:

```json
{
  "comment": "Investigated startup path. Failure occurs before adapter response streaming begins."
}
```

### Attachments

```text
POST /api/v1/issues/{issue_id}/attachments
```

Use when the issue needs stored artifacts rather than only prose.

### Assignment

```text
POST /api/v1/issues/{issue_id}/assign
POST /api/v1/issues/{issue_id}/unassign
```

Assign payload:

```json
{
  "employee_id": "employee-uuid"
}
```

### Checkout and release

```text
POST /api/v1/issues/{issue_id}/checkout
POST /api/v1/issues/{issue_id}/release
```

Checkout behavior:

- claims active ownership of execution
- conflicts if another employee already holds checkout
- is distinct from assignment

Release behavior:

- drops checkout ownership
- does not automatically unassign the issue

## Projects

### Project routes

```text
GET    /api/v1/projects
POST   /api/v1/projects
GET    /api/v1/projects/{project_id}
PATCH  /api/v1/projects/{project_id}
DELETE /api/v1/projects/{project_id}
```

Project payloads center on:

- `name`
- `working_directories`

Use projects to understand execution context and expected filesystem scope.

## Memory

### Employee memory

```text
GET   /api/v1/employees/me/memory
POST  /api/v1/employees/me/memory
GET   /api/v1/employees/me/memory/file?path=...
PATCH /api/v1/employees/me/memory/file
POST  /api/v1/employees/me/memory/search
```

### Project memory

```text
GET   /api/v1/projects/{project_id}/memory
POST  /api/v1/projects/{project_id}/memory
GET   /api/v1/projects/{project_id}/memory/file?path=...
PATCH /api/v1/projects/{project_id}/memory/file
POST  /api/v1/projects/{project_id}/memory/search
```

Common payloads:

Create memory entry:

```json
{
  "content": "Remember to rerun migration validation after updating the issue repository.",
  "path": "optional/path.md"
}
```

Update memory file:

```json
{
  "path": "notes/runtime.md",
  "content": "# Updated notes\n\n..."
}
```

Search memory:

```json
{
  "query": "checkout conflict handling",
  "limit": 5
}
```

## Runs

Runs are owner-only:

```text
GET    /api/v1/runs
GET    /api/v1/runs/{run_id}
POST   /api/v1/runs
DELETE /api/v1/runs/{run_id}/cancel
GET    /api/v1/runs/stream
```

Most employees do not need these for normal execution.

## Recommended Runtime Flow

```text
GET  /api/v1/employees/me
GET  /api/v1/issues?assignee=<employee-uuid>&expected_statuses[]=in_progress&expected_statuses[]=todo&expected_statuses[]=blocked
POST /api/v1/issues/{issue_id}/checkout
GET  /api/v1/issues/{issue_id}
GET  /api/v1/projects/{project_id}                # when present
POST /api/v1/projects/{project_id}/memory/search  # when project context is needed
PATCH /api/v1/issues/{issue_id}                   # if status changes
POST /api/v1/issues/{issue_id}/comments           # to report progress
POST /api/v1/issues/{issue_id}/release            # when dropping active ownership
```

## Things Not To Assume

- Use `/api/v1/employees/me` for runtime identity.
- There is no issue document or approval workflow in the current route set.
- Do not assume assignee filtering exists on the issue list endpoint.
