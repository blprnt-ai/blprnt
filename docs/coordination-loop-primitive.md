# Coordination Loop Primitive

This document expands a blprnt 2.0 feature direction: make structured coordination a first-class runtime primitive.

`HEARTBEAT.md` answers "when does an employee wake up and what do they focus on?"

Org structure answers "who reports to whom?"

What is still missing is "how does a group actually converge on reality, resolve conflict, and change course together?"

This doc uses the neutral term `coordination loop` as a placeholder. The product name can come later.

## Summary

A coordination loop is a recurring or ad hoc operating mechanism that gathers the right participants, requires evidence-backed preparation, forces synthesis, and ends by mutating shared state.

Examples:

- weekly planning
- launch readiness review
- incident response review
- hiring debrief
- customer escalation review
- content review
- pipeline review

The key idea is that the loop itself is an executable system object, not just a calendar event, a prompt, or a scheduled agent job.

## Why this matters

Today, most agent systems are strong at one of these:

- individual execution
- task assignment
- manager-worker delegation
- scheduled wakeups

What they usually do not model is the coordination layer between those pieces.

Real teams do not operate through isolated tasks alone. They rely on recurring mechanisms that:

- force updates into a common format
- require evidence before opinions
- surface conflicts between functions
- assign decisions to the right decision-maker
- create downstream work automatically
- update shared memory when the decision is over

Without this layer, a system can have smart agents and still feel like a pile of tabs, tickets, and background jobs.

## The primitive

A coordination loop would be a new top-level entity in blprnt.

Each loop defines:

- objective
- scope
- cadence or trigger
- participants
- facilitator
- decision owner
- required prep inputs
- evidence sources
- agenda synthesis rules
- decision rules
- output actions
- escalation rules

That turns "weekly planning" from a vague habit into a reproducible operating system component.

## How it builds on heartbeat and org structure

Heartbeat gives each employee an individual operating rhythm.

Org structure gives blprnt a chain of responsibility.

The coordination loop sits above both:

- heartbeats wake people up
- org structure determines who should be present and who can decide
- the loop gathers those people into a real operating event
- the event produces decisions that change issues, plans, heartbeats, and memory

That gives blprnt a missing layer:

`individual rhythm -> management structure -> group coordination`

## What makes it different from a scheduled job

A scheduled job says "run this thing every Friday."

A coordination loop says:

- who must contribute
- what evidence counts
- what conflicts should be surfaced
- who has decision rights
- what actions are allowed to happen automatically at the end

It is closer to a meeting operating system than a cron job.

## Lifecycle

Each loop instance would run through a consistent lifecycle.

### 1. Trigger

The loop starts because:

- its cadence is due
- a threshold is crossed
- a dependency changes
- someone invokes it manually

Examples:

- every Monday at 9am
- when a launch reaches `ready_for_review`
- when incident severity is `sev1`
- when three enterprise deals are blocked on the same objection

### 2. Prep request

Participants get a prep request tied to their role in the loop.

Examples:

- engineering submits delivery status and risk
- support submits top recurring customer pain points
- product submits scope changes and open decisions
- marketing submits launch asset readiness

Each prep request should be short, structured, and evidence-backed.

### 3. Evidence collection

The system gathers raw material from the relevant sources:

- issues
- runs
- project memory
- comments
- docs
- git activity
- analytics
- CRM
- support queue
- external tools later

The important constraint is that statements in the loop should be anchored in inspectable evidence when possible.

### 4. Agenda synthesis

A facilitator agent prepares the loop by:

- finding contradictions
- grouping repeated blockers
- identifying missing prep
- highlighting tradeoffs
- building a short agenda

This is where the loop becomes more than status collection.

### 5. Run

The loop executes as a structured turn, not an open-ended chat.

The runtime should support:

- quorum rules
- time-bounded participant input
- facilitator-led synthesis
- explicit decision points
- explicit unresolved items

The goal is not discussion for its own sake. The goal is convergence.

### 6. Decision and mutation

A loop should end by changing system state.

Examples:

- create or reprioritize issues
- update a project plan
- change an employee heartbeat
- assign follow-up owners
- mark a launch blocked
- escalate to a manager or board role
- write a memory summary

If the loop does not mutate shared state, it is probably just a report.

### 7. Record and memory

The output should leave durable artifacts:

- what evidence was considered
- what decisions were made
- what changed
- what remains unresolved
- when the loop should run again

This matters because recurring coordination is only useful if the next cycle can build on the last one.

## Core behaviors

The feature becomes interesting if blprnt enforces behaviors that teams usually fail to enforce manually.

### Evidence before opinion

Participants should not be able to submit pure vibes when the loop expects evidence.

That does not mean every statement must be quantified. It means claims should point to something inspectable whenever feasible.

### Required participation

Some loops should tolerate missing inputs.

Others should block until a quorum is met or escalate non-participation to the relevant manager.

### Role-specific preparation

Each participant should prepare differently based on role.

A CTO should not submit the same template as a support lead. The loop should know what each role is expected to bring.

### Built-in conflict detection

The interesting cases are not when everybody agrees.

The loop should actively detect cases like:

- engineering says "ready" while support says "still unstable"
- marketing wants launch this week while product changed scope yesterday
- manager says task is on track while assignee has been blocked for four days

### State mutation as the default

The output should create follow-through automatically instead of relying on somebody to remember what happened.

## Example loops

### Weekly product review

Participants:

- product
- engineering
- design
- support

Expected prep:

- top shipped changes
- top blocked work
- top user pain
- top decisions waiting on ownership

Expected outputs:

- priority changes
- new issues
- escalations
- project memory update

### Launch readiness

Participants:

- product
- engineering
- marketing
- support

Expected prep:

- feature readiness
- unresolved risks
- asset completion
- support readiness

Expected outputs:

- go
- no-go
- narrowed launch scope
- launch blocker issue set

### Incident review

Participants:

- on-call owner
- affected service owner
- support
- manager

Expected prep:

- timeline
- impact
- root cause hypothesis
- user-visible failure modes

Expected outputs:

- remediation issues
- changed runbooks
- updated heartbeat priorities
- incident memory record

### Hiring debrief

Participants:

- hiring manager
- interviewers
- recruiter or coordinator role

Expected prep:

- evidence from each interview
- risk flags
- score deltas
- role fit summary

Expected outputs:

- hire
- no-hire
- follow-up interview
- rewritten role scope if the market signal was off

## Runtime model

The feature likely needs a small but explicit runtime model.

Possible entities:

- `CoordinationLoop`
- `CoordinationRun`
- `CoordinationParticipant`
- `CoordinationSubmission`
- `CoordinationDecision`
- `CoordinationAction`

That would let blprnt track:

- loop definitions
- actual runs over time
- missing prep
- evidence attached to each run
- decisions and follow-up actions

## Where this fits in blprnt

This concept fits the current system well:

- the coordinator already knows how to react to events and schedules
- issues already represent owned follow-through
- plans already represent execution intent
- memory already stores durable project and employee context
- runs already provide inspectable traces

The coordination loop ties those existing primitives together into a repeatable operating pattern.

## Why it feels new

A lot of systems can schedule an agent.

A lot of systems can assign a task.

Some systems can model a reporting chain.

Very few treat recurring coordination itself as a first-class object with:

- participants
- evidence
- facilitation
- decision rights
- automatic state mutation

That is a distinct primitive, not just a new screen on top of tickets.

## What to avoid

This idea gets weak if it turns into any of these:

- a prettier recurring task feature
- a calendar integration with summaries
- a generic meeting note generator
- another chat room with a template

The bar is higher.

The system should enforce coordination mechanics that change how a team operates.

## MVP shape

A credible first version could stay narrow.

### MVP capabilities

- define a loop with participants, facilitator, cadence, and outputs
- request structured prep from participants
- collect evidence from existing blprnt entities
- run one facilitator-led synthesis step
- produce decisions and create follow-up issues
- write a run summary into project memory

### Explicitly not in MVP

- full external app integration
- complex voting systems
- free-form custom automation engine
- deep calendar semantics

## Open questions

- Should loops belong to a project, a department, or the whole company?
- Should a loop always have one human-approval point, or can some be fully autonomous?
- How strict should quorum enforcement be?
- Should failed participation affect employee evaluation or heartbeat priority?
- Should loops be defined from scratch, or mostly created from reusable templates?
- Do we want one generic primitive, or a small set of loop types with strong built-in semantics?

## Bottom line

If heartbeat is how an employee wakes up, and org structure is who they answer to, the next primitive is how a group repeatedly comes to ground truth and acts on it.

That is the gap this concept is trying to fill.
