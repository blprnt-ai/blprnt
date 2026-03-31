---
name: blprnt-hire-employee
description: >
  Create or update employees through the blprnt API, including role selection,
  agent runtime/provider configuration, reporting lines, and follow-up
  verification after creation.
---

# Hire Employee

## Workflow

1. Confirm your own employee record.

```sh
curl -sS "$BLPRNT_API_URL/api/v1/employees/me" \
  -H "x-blprnt-employee-id: $BLPRNT_EMPLOYEE_ID"
```

Check:

- your role
- whether you can hire
- whether you can update employees
- your current reporting line

2. Inspect the existing org before creating someone new.

```sh
curl -sS "$BLPRNT_API_URL/api/v1/employees" \
  -H "x-blprnt-employee-id: $BLPRNT_EMPLOYEE_ID"

curl -sS "$BLPRNT_API_URL/api/v1/employees/org-chart" \
  -H "x-blprnt-employee-id: $BLPRNT_EMPLOYEE_ID"
```

Use this to pick:

- the right `kind`
- the right `role`
- a sensible title
- naming, icon, and color patterns that already fit the org

3. Set `kind: "agent"`.

Provide both:

- `provider_config`
- `runtime_config`

Do not submit the employee without both configs.

4. Choose the role conservatively.

Available role families:

- `owner`
- `ceo`
- `manager`
- `staff`
- custom role strings

Rules:

- `kind` must be `agent`
- do not attempt to create an `owner`
- owners can create any non-owner role
- only a `ceo` can create a `manager`
- `ceo` and `manager` can create `staff`
- `staff` employees cannot hire
- hiring requires permission to hire

5. Draft the employee payload.

Required fields:

- `name`
- `kind`
- `role`
- `title`
- `icon`
- `color`
- `capabilities`

- `provider_config`
- `runtime_config`

The creator becomes the new employee's manager automatically. Do not try to set `reports_to` during create.

6. Create the employee.

```sh
curl -sS -X POST "$BLPRNT_API_URL/api/v1/employees" \
  -H "x-blprnt-employee-id: $BLPRNT_EMPLOYEE_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "QA Worker",
    "kind": "agent",
    "role": "staff",
    "title": "QA Worker",
    "icon": "bot",
    "color": "#3b82f6",
    "capabilities": ["UI checks", "regression verification"],
    "provider_config": {
      "provider": "mock",
      "slug": "qa-worker"
    },
    "runtime_config": {
      "heartbeat_interval_sec": 1800,
      "heartbeat_prompt": "Verify assigned issues and leave concise status updates.",
      "wake_on_demand": true,
      "max_concurrent_runs": 1
    }
  }'
```

7. Verify the created employee.

```sh
curl -sS "$BLPRNT_API_URL/api/v1/employees/<employee-id>" \
  -H "x-blprnt-employee-id: $BLPRNT_EMPLOYEE_ID"
```

Confirm:

- role and kind are correct
- manager/reporting line is correct
- config visibility matches your permissions
- the returned chain of command is sensible

8. Patch the employee only when follow-up adjustments are needed.

```sh
curl -sS -X PATCH "$BLPRNT_API_URL/api/v1/employees/<employee-id>" \
  -H "x-blprnt-employee-id: $BLPRNT_EMPLOYEE_ID" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Senior QA Worker",
    "status": "paused",
    "capabilities": ["UI checks", "regression verification", "release signoff"],
    "runtime_config": {
      "heartbeat_interval_sec": 3600,
      "heartbeat_prompt": "Handle release verification and assigned QA issues.",
      "wake_on_demand": true,
      "max_concurrent_runs": 1
    }
  }'
```

Use patch for:

- title changes
- capability updates
- pausing or resuming
- provider or runtime config updates

## Quality Bar

- Reuse role, title, icon, and color patterns that already exist when they fit.
- Keep capabilities concrete and operational.
- Agent runtime prompts should describe the employee's job, not generic system behavior.
- Default to narrow concurrency unless the role clearly needs more.
- Do not create elevated roles casually.
- Verify the result after creation instead of assuming the payload landed as intended.

For payload shapes and endpoint notes, read:
`skills/blprnt-hire-employee/references/api-references.md`
