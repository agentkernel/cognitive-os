# 07 — Contextual attention, approval, recovery, and long-running work

- Requirements:
  [OPC requirements analysis](../../../../personal/docs/product/personal-2.0-opc-requirements-analysis.md)
- Product source:
  [Long-running operations](../../../../personal/docs/product/long-running-operations.md)
- Status: current interaction prototype is post journey-subtraction; archived V2 is historical chrome only
- Current interaction prototype:
  [**personal-20-opc-e2e (post journey-subtraction)**](personal-20-opc-e2e.canvas.tsx)
- Archived historical V2 (not current chrome):
  [pre-subtraction history](history/2026-08-28-pre-subtraction/README.md)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

## Operational thesis

What needs the Owner appears in Today and the affected Project canvas. There is
no permanent first-level attention queue. Contextual attention covers:

- consequential approval or requested input;
- permission, credential, model, connector, or capability block;
- execution failure or outcome unknown/reconciling;
- missed/coalesced Routine occurrence;
- actual/estimated/unknown cost warning or Provider quota failure;
- stale source, preview, or Project revision conflict.

Every item shows Project/Member, goal or affected deliverable, age/freshness,
reason, consequence, reversibility, source/evidence, cost basis, and available
next actions. Priority is textual and explained, not color-only.

The default Operations working view is the daemon authority path:

**Candidate → Intent persisted → Fence → Execute → Independent verify →
Receipt.**

Working is in-progress observation, not completion. Agent self-report is not
verification.

## Approval and HITL

Consequential work opens a daemon-issued structured preview with current and
proposed revisions, targets, Intent/Effect implications, exact permissions,
source/uncertainty, cost basis, reversibility/compensation, freshness, and
edit/narrow/deny/confirm choices.

Chat may announce a HITL pause and link to the center-canvas preview, but chat
is never the confirmable object. There is no chat Approve control and no
“Don’t ask again” grant. Project publish preview is the full AUTONOMY packet
on the canvas; there is no Confirm in chat. Confirmation binds its exact
digest. Rejection
preserves the candidate. Drift forces re-preview. The resulting receipt links
the Project object, originating conversation, and applicable Task/Effect.

The launch-time autonomy envelope permits low-risk reversible internal work.
Primary-goal, team, Provider/model, Tool/MCP, permission, global Role,
external-action-rule, permanent deletion, and first/expanded MCP permissions
cross the applicable Owner boundary.

## Unknown and failure recovery

An unknown external outcome is not safe to retry. The view shows persisted
Effect identity, dispatch fact, observation coverage, reconciliation step, and
why redispatch is blocked. Recoverable failure preserves Task, Attempt, input,
artifacts, and evidence. Retry/restart creates a new Attempt.

Agent or manager self-report, process exit, Provider/Tool success, and engine
checkpoint are neither recovery nor completion. Only independent criteria and
daemon acceptance close the Task.

## Triggers, no-overlap, and queue-latest

Routines may be requested manually or by schedule, accepted artifact, Project
state, qualified external event, or testable data condition. For one Routine:

- active occurrences never overlap;
- at most the latest pending occurrence is queued;
- superseded occurrences remain skipped/coalesced facts with denominator;
- sleep, shutdown, daemon/dependency outage, and expired content create visible
  missed facts;
- low-risk internal work may resume under unchanged policy;
- consequential or stale work receives fresh Context, policy, or Owner review.

Across Projects, ordering is explained by Owner priority, deadline, schedule,
resource availability, and fairness. No ETA or queue authority is inferred from
Agent prose.

## Long-running progress and instruction changes

Progress shows goal/output contract, current durable step, responsible Member,
Task/Attempt, latest openable artifact, Intent/Effect/evidence, cost basis,
queued/missed facts, blocked reason, and only real pause/stop/retry/resume/
reconcile controls. Working is in-progress observation, not completion. Final
receipt separates completed, skipped, failed, unknown, and not-run work.

A new Owner instruction creates a version and applies at a safe point through
continue, pause, or restart. It is never injected silently into a running
prompt.

Closing the Windows window offers **continue eligible work in background** or
**pause at the current safe boundary**, names affected work, and states that
host shutdown stops execution. Missing backend support renders explanation,
not fake controls.

## X/Twitter external-action recovery

The content loop is research -> topic plan -> draft/media artifacts ->
publication package -> applicable preview -> qualified dispatch -> receipt ->
metric/comment readback -> reflection -> next cycle. Success is the deliverable
plus an Owner-confirmed dispatch receipt plus a feedback summary; unknown
metrics stay unknown. Dispatch and readback are separate facts. Comment replies
are a suggestion pack that still requires the applicable confirm. Manual
publication is a degraded fallback, not the primary acceptance route. CAPTCHA,
anti-abuse, account lock, UI drift, or unknown result fails closed; no evasion
or blind retry.

## States and capability honesty

Contextual attention covers empty, loading, partial, stale, permission, error,
unknown, offline, missed, waiting-owner, queued, running, reconciling,
failed-preserved, recovered, success, and archived. Cost warnings never imply
an automatic Personal budget stop.

The complete attention projection, serialized approval, Routine ledger,
queue-latest, catch-up, safe-point revision, close-window choice, and recovery
controls are **Requires-backend**. External connectors are additionally
**Requires-environment**.
