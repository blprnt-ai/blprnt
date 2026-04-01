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

3. List configured providers before choosing a provider config.

```sh
curl -sS "$BLPRNT_API_URL/api/v1/providers" \
  -H "x-blprnt-employee-id: $BLPRNT_EMPLOYEE_ID"
```

Rules:

- do this before drafting `provider_config`
- choose a `provider_config.provider` value that already appears in the configured providers list
- do not invent or guess a provider that is not already configured
- when in doubt, reuse your own employee `provider_config` values for the new hire

4. Optionally list available skills before drafting the runtime config.

```sh
curl -sS "$BLPRNT_API_URL/api/v1/skills" \
  -H "x-blprnt-employee-id: $BLPRNT_EMPLOYEE_ID"
```

Rules:

- this step is optional
- the response only includes skills that are not already on your own employee record
- you may attach a `runtime_config.skill_stack`, but it is optional
- `runtime_config.skill_stack` supports at most 2 skills
- each selected skill must be passed as an object with `name` and `path`

5. Set `kind: "agent"`.

Provide both:

- `provider_config`
- `runtime_config`

Do not submit the employee without both configs.

6. Choose the role conservatively.

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

7. Draft the employee payload.

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

Provider config rules:

- `provider_config.provider` must match an already configured provider from `GET /api/v1/providers`
- prefer copying your own `provider_config` when you are unsure which configured provider or slug to use
- only diverge from your own config when there is a concrete reason and the replacement provider is confirmed in the configured providers list

Runtime config rules:

- `skill_stack` is optional
- when present, include no more than 2 skills
- use entries exactly as returned from `GET /api/v1/skills`

8. Create the employee.

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
      "max_concurrent_runs": 1,
      "skill_stack": [
        {
          "name": "analytics-tracking",
          "path": "/Users/example/.agents/skills/analytics-tracking/SKILL.md"
        }
      ]
    }
  }'
```

9. Verify the created employee.

```sh
curl -sS "$BLPRNT_API_URL/api/v1/employees/<employee-id>" \
  -H "x-blprnt-employee-id: $BLPRNT_EMPLOYEE_ID"
```

Confirm:

- role and kind are correct
- manager/reporting line is correct
- config visibility matches your permissions
- the returned chain of command is sensible

10. Patch the employee only when follow-up adjustments are needed.

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
- Reuse an already configured provider instead of introducing a new one during hire.
- Prefer your own employee `provider_config` as the default template when the correct provider values are unclear.
- Keep capabilities concrete and operational.
- Agent runtime prompts should describe the employee's job, not generic system behavior.
- Default to narrow concurrency unless the role clearly needs more.
- Do not create elevated roles casually.
- Verify the result after creation instead of assuming the payload landed as intended.

For payload shapes and endpoint notes, read:
`skills/blprnt-hire-employee/references/api-references.md`
