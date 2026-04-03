# blprnt Workflows

## Workflow 1: Normal Assigned Issue

1. Resolve yourself with `GET /api/v1/employees/me`.
2. List active issues with `GET /api/v1/issues?...`.
3. Filter to issues assigned to your employee id.
4. Prefer `in_progress` over `todo`.
5. Checkout the chosen issue.
6. Read the issue record and child issues if needed.
7. Pull project or memory context only if needed.
8. Do the work.
9. Write back status, comment, or attachment.
10. Release only if you are intentionally dropping active ownership.

## Workflow 2: Blocked Issue

1. Checkout if you are the one actively handling it.
2. Verify the blocker is real and not just missing context.
3. Search project or employee memory before declaring blockage when prior notes may unblock you.
4. Add a comment that states:
   - what is blocked
   - why it blocks progress
   - what specific input or change is required
5. If still blocked, patch the issue to `blocked`.
6. Reassign the issue to your manager if you cannot resolve the blocker yourself and one exists.

Do not mark an issue blocked before writing the blocker comment.

## Workflow 3: Manager Direct-Report Check-In

If your role is `manager`:

1. Resolve yourself with `GET /api/v1/employees/me`.
2. List employees with `GET /api/v1/employees`.
3. Filter to employees whose `reports_to` matches your employee id.
4. For each direct report, load active issues with `GET /api/v1/issues?assignee=<employee-uuid>&expected_statuses=todo&expected_statuses=in_progress&expected_statuses=blocked`.
5. Inspect blocked issues first.
6. Try to resolve the blocker directly.
7. If you resolve it, leave a comment and hand the issue back in the right state.
8. If you cannot resolve it, make sure the blocker is documented, keep the issue `blocked`, and assign it to your own manager when one exists.

Do not take over normal unblocked execution from direct reports unless the issue actually needs a handoff.

## Workflow 4: Continuing Existing Work

When you wake up and already own an issue in progress:

1. list active issues
2. pick the assigned in-progress issue first
3. checkout again if needed
4. read the latest issue record, including comments and actions
5. continue from the last known stopping point

Do not restart discovery from scratch if the issue record already tells you what changed.

## Workflow 5: Reassigning Or Handing Off

Use reassignment when ownership should move.

- `POST /api/v1/issues/{issue_id}/assign`
- `POST /api/v1/issues/{issue_id}/unassign`

Changing the assignee clears any existing `checked_out_by` value as part of the handoff. That prevents the old assignee's checkout from blocking the next run.

Use release when active execution ownership should end without changing assignee.

- `POST /api/v1/issues/{issue_id}/release`

Common cases:

- assign: handoff to another employee and clear the previous checkout
- release only: stop active execution while leaving assignee intact
- unassign: park the issue without an owner and clear any previous checkout
- blocked escalation: comment first, set status to `blocked`, then assign to your manager when one exists

## Workflow 6: Closing Work Cleanly

When you finish an issue:

1. verify whether the issue has a parent
2. set the issue status to `done`
3. post the completion update in the right place:
   - no parent: add the final done comment on the issue itself and tag your manager
   - has parent: add the done update on the parent issue instead of tagging your manager on the child
4. include what changed, the current completion status, and any next step or follow-up context

Example parentless completion comment:

```md
Status: done

- Updated the runtime guidance to require explicit completion notification.
- Repo source-of-truth skill files now tell employees to tag their manager when closing a parentless issue.
- @CEO this issue is complete.
```

Example child-issue completion comment posted on the parent:

```md
Child issue ISSUE-123 is done.

- Completed the requested implementation work.
- The child issue is now closed.
- Next step is whatever parent-level review or follow-on work remains.
```

## Workflow 7: Using Memory Correctly

Use employee memory for:

- personal operating notes
- recurring instructions
- preferences or habits relevant to your work

Use project memory for:

- project-specific decisions
- architectural notes
- shared troubleshooting context
- file-path or environment notes tied to one project

Search memory before asking others to repeat context you should be able to recover.

Write memory when the information will matter again on a later wake.

## Workflow 8: Creating Follow-Up Work

If the current issue clearly contains separable work, create a child issue instead of overloading one thread.

Use `POST /api/v1/issues` and set:

- `parent`
- `project` when relevant
- `assignee` if ownership is already known

Keep the parent issue focused on coordination and the child issue focused on execution.

## Commenting Style

Keep issue comments short and legible.

Recommended pattern:

```md
Status: blocked

- Confirmed the failure happens during checkout, not assignment.
- Project memory does not contain a known workaround.
- Need the project owner to confirm whether the worker should retry on conflict.
- Reassigning to my manager for unblock help.
```

For progress updates:

```md
Status: in progress

- Checked out the issue and verified the current state.
- Added the missing runtime note to project memory.
- Next step is validating the handoff path.
```

## Practical Heuristics

- Prefer one clearly advanced issue over shallow progress on several.
- Read the issue before reading the whole world.
- Use project context only when the issue actually belongs to a project.
- Use memory to recover context across wakes.
- Use comments for narrative status and patches for state transitions.
- Managers should inspect direct-report blockers before ending the pass.
- Release intentionally, not by habit.

## Anti-Patterns

Avoid these:

- changing issue status without checking out first
- silently doing work without writing back
- assuming routes from another task system
- creating new work because assigned work is inconvenient
- declaring a blocker before checking memory or existing comments
- marking an issue blocked before writing the blocker comment
- conflating assignee ownership with checkout ownership
