# Intent, Effect and Idempotency Standard

- Standard ID: `cognitiveos.standard.intent-effect-idempotency/0.1`
- Version: v0.1 Draft
- Machine version: `0.1.0-draft.1`
- Status: Draft Normative Standard
- Date: 2026-07-20
- Machine assets: `specs/schemas/intent.schema.json`,
  `effect.schema.json`, `specs/transitions/effect.transitions.json`,
  `loop-checkpoint.schema.json`
- Normative sources: Core companion; whitepaper sections 11, 16 (informative)

## 1. Scope and normative language

RFC 2119/8174 language applies. This standard fixes the reference
implementation contract for the Intent → Effect protocol, idempotency, and
crash recovery. Owning requirements: [REQ-EFF-001] through [REQ-EFF-007],
[REQ-EFF-STATE-001], [REQ-AKP-IDEM-001], [REQ-MGMT-IDEM-001],
[REQ-REC-001..003], [REQ-RUN-006]. It registers no new requirement.

## 2. Intent before dispatch

An external side effect MUST be preceded by a persisted Intent carrying: a
stable idempotency key, a parameter digest (canonical bytes, ADR-0004), the
`expected_state_version` for CAS, and the authorization binding. Persisting
the Intent and appending its event MUST be one atomic transaction
([REQ-EFF-001], `state-and-transition-contract.md`). No Intent, no dispatch.

## 3. Idempotency keys

1. The key is minted once per logical effect attempt chain and MUST remain
   stable across timeout, retry, crash and recovery; a timeout MUST NOT mint
   a new key (vector `eff-crash-001.json`, [REQ-EFF-006]).
2. Reuse of a key with a different parameter digest MUST be rejected with
   `EFFECT_IDEMPOTENCY_CONFLICT` — no dedup, no execution (vector
   `effect-idempotency-conflict.json`, [REQ-EFF-002]).
3. Key comparison uses the canonical parameter digest, never source bytes.

## 4. Outcome closure

`EFFECT_OUTCOME_UNKNOWN` is a first-class state, not an error to be retried
blindly. From unknown, the only exits are reconciliation to a confirmed
outcome or quarantine (`EFFECT_RECOVERY_QUARANTINED`); there is no direct
path from unknown to VERIFIED/COMMITTED
(`specs/transitions/effect.transitions.json`, vector
`effect-unknown-outcome.json`, [REQ-EFF-004]). A receipt, a remote
`completed` flag, or a model narrative is never acceptance ([REQ-GW-002],
vector `remote-completed-not-acceptance.json`). Compensation is a new Intent
with fresh, independently checked authorization; it never inherits the
original capability (Core companion Effect protocol).

## 5. Crash points and recovery order

The three canonical crash points MUST each have fault-injection coverage
(vectors `eff-crash-001..003.json`): after Intent persisted / before
dispatch; after dispatch / before outcome; after outcome / before local
commit. Recovery follows the fixed order — install new fencing epoch →
replay committed history → reconcile in-flight Effects → re-authorize →
re-resolve Context → resume Loops ([REQ-REC-001], [REQ-RUN-006], vectors
`crash-recovery.json`, `agent-recovery-reconciliation.json`). Recovery MUST
NOT reorder these steps, mint new idempotency keys, or report unknown
outcomes as success.

## 6. Durability fail-closed

If the authoritative store cannot commit (read-only database, disk full),
the governed write MUST fail with `STATE_STORE_UNAVAILABLE` before any
dispatch: fail-before-effect ([REQ-REC-003], vector
`state-store-degradation.json`). Buffering authoritative commits in memory is
forbidden.

## 7. Checkpoints

A LoopCheckpoint MUST carry the recovery-stable facts pinned by
`loop-checkpoint.schema.json` (event high-watermark, fencing epoch, contract
and specification version pins) so recovery replays deterministically.
Checkpoint content is evidence for resumption, never a substitute for the
committed event history.

## 8. Compliance checks

M4 exit requires: all three crash-point vectors executed with evidence under
`artifacts/evidence/faults/`; unknown-outcome, idempotency-conflict, and
fail-before-effect negatives executed; and the tracer bullet demonstrating
Intent → Effect → Verification → acceptance end to end. Schema-valid Effect
documents alone prove nothing (F-005 history: the machine contract once
admitted `COMMITTED+unknown+pending`).
