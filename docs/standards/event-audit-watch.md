# Event, Audit and Watch Standard

- Standard ID: `cognitiveos.standard.event-audit-watch/0.1`
- Version: v0.1 Draft
- Machine version: `0.1.0-draft.1`
- Status: Draft Normative Standard
- Date: 2026-07-20
- Machine assets: `specs/schemas/event.schema.json`,
  `watch-subscription.schema.json`, `shell-status-view.schema.json`
- Normative sources: RFC-0001 section 14; Core companion; AKP companion

## 1. Scope and normative language

RFC 2119/8174 language applies. This standard fixes the reference
implementation contract for the append-only event log, audit records, and
snapshot+cursor watch. Owning requirements: [REQ-EVT-001] through
[REQ-EVT-005], [REQ-AUDIT-001], [REQ-AUDIT-002], [REQ-SHELL-WATCH-001],
[REQ-AKP-SHELL-002], [REQ-AKP-CONT-001]. It registers no new requirement.

## 2. Append-only event log

Events are immutable once committed: no in-place edit, no deletion, no
timestamp rewrite ([REQ-EVT-002]; clock correction creates a linked
correction record, ADR-0005). Every authority state transition commits its
event in the same transaction as the state change
(`state-and-transition-contract.md`); an event without its state change, or
a state change without its event, is a defect. Ordering within a stream is
by `logical_version`/sequence, never by wall timestamp or UUID lexical order
(ADR-0005).

## 3. Projections and replay

Projections (status views, dashboards, Console feeds) are derived and
disposable: replaying the committed event history MUST reproduce a
projection with a stable canonical digest ([REQ-EVT-003] family; M2
acceptance "投影重放 digest 稳定"). A projection is never an authority input.

## 4. Audit records

Audit records for governed decisions (authorization, admission, approval,
recovery) MUST carry actor chain, decision, registered error code where
applicable, stage, and correlation IDs, and MUST NOT contain secrets or
cross-tenant content ([REQ-AUDIT-001], [REQ-AUDIT-002]). The audit trail for
an Effect MUST close: Intent → dispatch → outcome/reconciliation → final
state are linkable (vector `eff-crash-001.json` asserts
`audit_chain_closed`).

## 5. Watch: snapshot + cursor

A watch subscription delivers one consistent snapshot, then ordered deltas
with a resumable cursor ([REQ-SHELL-WATCH-001], [REQ-AKP-SHELL-002]).
Delivery is at-least-once; consumers deduplicate by event ID/sequence
(profile manifest guarantee `event_delivery: at_least_once`). A stale or
compacted cursor MUST fail with `WATCH_CURSOR_STALE`, forcing a fresh
snapshot; silently skipping gaps is forbidden (vector
`shell-watch-resume-006.json`). Reconnection semantics follow AKP
continuation ([REQ-AKP-CONT-001]).

## 6. Channel and scope binding

A watch stream is bound to the requesting channel, tenant, and capability
set at subscription time and MUST be re-validated on resume; scope widening
through a resumed cursor is forbidden ([REQ-SHELL-CHANNEL-001],
`shell-channel-isolation-003.json`). Watch payloads are authority
projections; they carry object references and state, not recomputed
client-side aggregates.

## 7. Compliance checks

Vectors `evt-schema-001.json` and `shell-watch-resume-006.json` execute with
evidence; M2 exit adds the replay-digest-stability test and the
no-in-place-edit negative; watch scope re-validation has a negative test in
`tests/security/`.
