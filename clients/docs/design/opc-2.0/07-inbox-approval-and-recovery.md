# 07 — Inbox, approval, recovery, and long-running work

## Operational thesis

Inbox answers **what needs the Owner next and why**. It is a priority queue,
not chat or a generic event log.

Item types:

- consequential approval;
- information/material requested;
- permission or connector block;
- execution failure;
- outcome unknown/reconciling;
- missed/coalesced Routine occurrence;
- budget/quota warning or stop;
- stale source or Project revision conflict.

## Item anatomy

Every item shows Project/employee, age/freshness, reason, affected target,
consequence, reversibility, source/evidence, cost/budget impact, and available
next actions. Priority uses text and rationale, not color alone.

List filters and selection remain stable. Empty states distinguish "nothing
needs you" from data unavailable or filters hiding results.

## Approval

Approval opens a daemon-issued structured preview:

- current/proposed revision;
- targets and external Effects;
- permission and budget changes;
- source and uncertainty;
- reversibility/compensation truth;
- stale-preview status;
- confirm, edit, narrow, reject.

Chat explanation may be adjacent but is not the confirmable object. Confirm
binds the exact preview. Rejection preserves the candidate. Drift forces
re-preview. Receipt returns to the item, originating Conversation, and affected
Project.

## Unknown and failure recovery

Unknown external outcome is not safe retry. The item shows persisted Effect
identity, dispatch fact, observation coverage, reconcile step, and why
redispatch is blocked.

Recoverable failure preserves Task/Attempt, input, artifact, and evidence. A
new retry/fork receives a new Attempt identity. Process exit, engine
checkpoint, Provider response, or Agent claim is not recovery or completion.

## Routine, queue-latest, and missed state

For one Routine:

- no overlapping active occurrence;
- at most the latest pending occurrence is queued;
- superseded occurrences retain skipped/coalesced facts;
- offline/sleep/shutdown produces missed facts;
- low-risk internal work may resume under policy;
- publishing, communication, spending, deletion, permission expansion, or
  changed targets require fresh review.

The queue shows active, latest queued, missed/coalesced denominator, and next
action. It does not infer an ETA from Agent text.

## Close-window decision

When eligible work is active, closing the Windows window asks:

1. continue eligible work in background; or
2. pause at the current safe boundary.

The preview names affected Projects/Routines, work that cannot stop instantly,
host-online limitations, and reopen behavior. If the backend is absent, both
options are explanatory `Requires-backend`, not fake controls.

## Long-running progress

The progress surface shows plan, current durable step, responsible employee,
Task/Attempt, latest artifact, Effect/evidence, actual/unknown cost, blocked
reason, and real pause/stop/retry/resume capabilities. Final receipt separates
completed, skipped, failed, unknown, and not-run work.

## States

Inbox covers empty, loading, partial, stale, permission, error, unknown,
offline, missed, waiting-owner, queued, running, reconciling, stopped,
recovered, success, and archived. Error messages identify the object and safe
next path; user input is preserved.

## Requires-backend

The unified Inbox, serialized approval, Routine ledger, queue-latest,
missed/catch-up policy, close-window choice, and recovery controls are target
behavior. Existing previews, Effects, alerts, and scheduler facts are reusable
but do not make the complete surface available.
