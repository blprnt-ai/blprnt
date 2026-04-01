# Hire Employee API Reference

## Core Endpoints

- `GET /api/v1/employees/me`
- `GET /api/v1/employees`
- `GET /api/v1/employees/org-chart`
- `GET /api/v1/providers`
- `GET /api/v1/employees/{employee_id}`
- `POST /api/v1/employees`
- `PATCH /api/v1/employees/{employee_id}`
- `DELETE /api/v1/employees/{employee_id}`

Protected routes require:

- `x-blprnt-employee-id`

## Permission Rules

Creation rules:

- `kind` should be `agent`
- creating an `owner` is rejected
- owners may create any non-owner role
- creating a `manager` requires the current employee to be a `ceo`
- creating `staff` requires the current employee to be a `ceo` or `manager`
- `staff` employees cannot hire
- creating any employee requires hire permission

Update rules:

- updating employees requires update permission

Delete rules:

- deleting employees is owner-only

## Employee Create Payload

```json
{
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
}
```

Notes:

- `kind` should be `agent`
- `role` is `owner`, `ceo`, `manager`, `staff`, or a custom string
- `provider_config` and `runtime_config` are required for agent employees
- list configured providers with `GET /api/v1/providers` before choosing `provider_config`
- `provider_config.provider` should match a provider that is already configured
- when uncertain, reuse the current employee's `provider_config` values for the new hire
- `reports_to` is not part of create; the API sets it to the creator automatically

## Employee Patch Payload

```json
{
  "name": "optional",
  "title": "optional",
  "status": "idle | paused | running | terminated",
  "icon": "optional",
  "color": "optional",
  "capabilities": ["optional", "replacement", "list"],
  "provider_config": {
    "provider": "mock",
    "slug": "qa-worker"
  },
  "runtime_config": {
    "heartbeat_interval_sec": 3600,
    "heartbeat_prompt": "Updated instructions",
    "wake_on_demand": true,
    "max_concurrent_runs": 1
  }
}
```

Patch notes:

- patch is sparse
- omitted fields are unchanged
- use it to pause, resume, retitle, or reconfigure an employee

## Response Shape Notes

Employee responses include:

- `id`
- `name`
- `role`
- `kind`
- `icon`
- `color`
- `title`
- `status`
- `capabilities`
- `permissions`
- `reports_to`
- `provider_config`
- `runtime_config`
- `chain_of_command`

Non-owner callers may see sensitive config fields hidden.

## Practical Patterns

### Create an employee

Use `kind: "agent"` and include both configs.

### Pause an employee

```json
{
  "status": "paused"
}
```

### Resume an employee

```json
{
  "status": "idle"
}
```

### Tighten runtime behavior

```json
{
  "runtime_config": {
    "heartbeat_interval_sec": 900,
    "heartbeat_prompt": "Focus only on assigned customer support issues.",
    "wake_on_demand": true,
    "max_concurrent_runs": 1
  }
}
```

## Field Guidance

`provider_config`

- `provider`: the backing model provider and it should already exist in `GET /api/v1/providers`
- `slug`: the provider-specific or local runtime slug for that employee

Hiring guidance:

- do not select a provider that is not already configured
- if several configured providers could work, default to the current employee's own provider config unless there is a clear reason to choose another

`runtime_config`

- `heartbeat_interval_sec`: timer cadence
- `heartbeat_prompt`: role-specific operating prompt
- `wake_on_demand`: whether assignment-triggered runs are allowed
- `max_concurrent_runs`: concurrency cap

## Source Of Truth

If this file and the code diverge, follow:

- `crates/api/src/routes/v1/employees.rs`
- `crates/persistence/src/models/employees/mod.rs`
- `crates/persistence/src/models/employees/types.rs`
