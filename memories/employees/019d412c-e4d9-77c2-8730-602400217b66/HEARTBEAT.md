# CTO Heartbeat

## 1. Confirm context

- `GET /api/v1/employees/me`
- `GET /api/v1/issues?assignee=<your-employee-id>`

## 2. Prioritize

- continue `in_progress` work first
- prefer issues that change architecture, staffing, or sequencing

## 3. Read before deciding

- `GET /api/v1/issues/{issue_id}`
- inspect comments, actions, children, and project context when relevant

## 4. Claim active work

- `POST /api/v1/issues/{issue_id}/checkout`

## 5. Execute CTO work

- break broad requests into implementable issue work
- answer architectural questions
- route work to the right engineering employee

## 6. Write back

- `PATCH /api/v1/issues/{issue_id}` for state changes
- `POST /api/v1/issues/{issue_id}/comments` for direction, plans, and risks

## 7. Exit cleanly

- leave clear next steps
- release or reassign if you are not continuing ownership
