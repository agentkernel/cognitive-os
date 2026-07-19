# Task, Loop and Verification Standard

- Standard ID: `cognitiveos.standard.task-loop-verification/0.1`
- Version: v0.1 Draft
- Machine version: `0.1.0-draft.1`
- Status: Draft Normative Standard
- Date: 2026-07-20
- Machine assets: `specs/schemas/task-contract.schema.json`,
  `user-intent-record.schema.json`, `intent-interpretation.schema.json`,
  `verification-report.schema.json`, `shell-action-proposal.schema.json`,
  `shell-command-preview.schema.json`, `specs/transitions/task.transitions.json`,
  `loop.transitions.json`, `verification.transitions.json`
- Normative sources: Core companion; RFC-0001 section 19

## 1. Scope and normative language

RFC 2119/8174 language applies. This standard fixes the reference
implementation contract for the intent chain, the bounded Harness Loop, and
Verification/acceptance. Owning requirements: [REQ-INTENT-RECORD-001],
[REQ-INTENT-ADMISSION-001], [REQ-INTENT-ACCEPT-001],
[REQ-INTENT-SUPERSEDE-001], [REQ-RUN-004], [REQ-RUN-005], [REQ-RUN-009],
[REQ-GW-001] through [REQ-GW-004], [REQ-SHELL-AMBIGUITY-001],
[REQ-SHELL-CORRECTION-001]. It registers no new requirement.

## 2. Intent chain

The chain UserIntentRecord → IntentInterpretation (admission) → TaskContract
is explicit and persisted. Interpretation is a probabilistic candidate;
admission into a TaskContract is a deterministic gate
([REQ-INTENT-ADMISSION-001]). Material ambiguity MUST trigger clarification
(`INTENT_CLARIFICATION_REQUIRED`, vector `shell-target-ambiguity-001.json`)
rather than a guess; R0 deployments may relax to a preview obligation
(IMP-14) but never to silent guessing on high-risk actions.

## 3. Correction and supersession

A user correction creates a new intent version that supersedes the old one
and advances the execution epoch: dispatches fenced to the old epoch MUST be
rejected on commit (`INTENT_VERSION_SUPERSEDED`, vector
`intent-supersede-002.json`, [REQ-INTENT-SUPERSEDE-001],
[REQ-SHELL-CORRECTION-001]). Supersession never rewrites history: prior
records remain linked.

## 4. Bounded Loop

A Harness Loop runs under an explicit TaskContract with hard budgets
(steps, tokens, wall deadline, cost) enforced deterministically
([REQ-RUN-004], vector `loop-contract-001.json`). Progress/stagnation is
judged by declared criteria; a stagnating Loop stops or escalates, it does
not spin ([REQ-RUN-005], vector `loop-gate-001.json`). Every externally
effecting step goes through the Intent/Effect gate
(`intent-effect-idempotency.md`); the Loop never bypasses it.

## 5. Verification and acceptance

Task completion is decided only by the acceptance authority consuming
Verification evidence ([REQ-INTENT-ACCEPT-001], [REQ-RUN-009], vector
`loop-verify-003.json`). Verification has its own lifecycle and can expire
independently (`specs/transitions/verification.transitions.json`); expired
verification does not keep a Task complete-able. Remote `completed`,
receipts, tool exit codes, or model self-reports are never acceptance
([REQ-GW-002], vector `remote-completed-not-acceptance.json`). A Task with
an unclosed Effect (`OUTCOME_UNKNOWN`) MUST NOT reach a success terminal
state ([REQ-EFF-STATE-001], vector `effect-state-closure-008.json`).

## 6. Shell semantics

The Shell is a non-authority client: detach/exit does not cancel
([REQ-SHELL-DETACH-001], vector `shell-detach-attach-004.json`); cancel is a
request that resolves through Effect closure — `CANCEL_PENDING` until the
remote outcome is known, and `CANCEL_TOO_LATE` when the effect already
committed ([REQ-SHELL-CONTROL-001], vector
`shell-cancel-semantics-005.json`). Status shown to users comes from
authority projections ([REQ-SHELL-STATUS-001]).

## 7. Compliance checks

M5 exit requires executed evidence for: ambiguity clarification, correction
fencing (epoch advance rejects old dispatch), detach-not-cancel,
cancel-pending closure, and acceptance-only-by-authority. Schema-valid
TaskContracts alone prove nothing.
