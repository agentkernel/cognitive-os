---
doc_id: dev.authority-kernel
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-kernel/src/engine.rs
    symbols: ["TransitionEngine", "prepare_object_admission", "prepare_transition", "validate_registered_transition"]
  - path: crates/cognitive-kernel/src/intent_chain.rs
    symbols: ["record_user_intent", "admit_interpretation", "mint_schedulable_task_contract", "prepare_task_execution_bootstrap", "supersede_task_contract", "verify_task_binding_current"]
  - path: crates/cognitive-kernel/src/effects.rs
    symbols: ["EffectProtocol", "mint_intent", "COMMIT_SINKS"]
  - path: crates/cognitive-kernel/src/authz.rs
    symbols: ["authorize", "revalidate_grant", "capability_and_revocation_current"]
  - path: crates/cognitive-kernel/src/budget.rs
    symbols: ["check_and_debit"]
  - path: crates/cognitive-kernel/src/recovery.rs
    symbols: ["RECOVERY_ORDER", "run_recovery"]
  - path: crates/cognitive-kernel/src/harness.rs
    symbols: ["return_to_decide_after_closed_effect", "advance_start_to_decide_after_context_view"]
contracts:
  - specs/transitions/effect.transitions.json
  - specs/transitions/task.transitions.json
  - specs/registry/errors.yaml
tests:
  - crates/cognitive-kernel/tests/engine_gate.rs
  - crates/cognitive-kernel/tests/governance_gate.rs
  - crates/cognitive-store/tests/m4_effects.rs
  - crates/cognitive-store/tests/m4_recovery.rs
fingerprint: "sha256:250dff539a72f06829c1e1f5b90b62eb592eebbe4814281c4a056fb64aa56368"
non_claims:
  - Kernel correctness evidence is focused-test evidence; it is not a Gate, release, or Profile result.
---

# Authority kernel

`cognitive-kernel` is the deterministic core: no HTTP, no SQLite, no model SDKs.
Native Tool pre-executor validators in this crate are pure (HTTPS origin shape
allows an optional explicit port as part of the exact origin; userinfo, query,
and fragment stay forbidden). Adapters implement its port traits; the reference adapter is `cognitive-store`.
The daemon-private `MemorySkillConsumptionStore` port records exact Memory/Skill
pins for Context loading; it does not grant client write authority or complete a
Task.

## The ten-step transition gate

`TransitionEngine::prepare_transition` validates, in fixed order: (1) table pin
(version + canonical digest of the registered transition table), (2) authoritative
row load, (3) from-state currency, (4) CAS on `expected_version`, (5) edge lookup
`(from, to, reason)`, (6) every guard present in the caller-attested set — absence
fails closed, (7) required evidence as strong references, (8) optional hard-budget
debit (pure `check_and_debit`, joins the same commit), (9) schema-shaped committed
record + canonical event, (10) one atomic `TransitionCommit` (object CAS + event +
record + budget CAS + outbox + fencing epoch). Rejections carry authoritative
state/version, sorted legal exits, and map deterministically onto registered error
codes (`STATE_CONFLICT`, `DIGEST_MISMATCH`, `STATE_STORE_UNAVAILABLE`,
`RESOURCE_BUDGET_EXHAUSTED`, and the pinned `EFFECT_OUTCOME_UNKNOWN` special case).

Three sanctioned preparation seams exist for compound atomic transactions only:
the pure validator `validate_registered_transition` (used by candidate
admission), `PreparedTransition` (committed unchanged inside
verified-continuation consumption), and
`TransitionEngine::prepare_object_admission` (used to place an unchanged,
registered-initial-state object admission beside inseparable authority rows).
All preserve the exact validated commit.

## Intent chain

`record_user_intent` fixes raw text before interpretation; interpretation
candidates persist as proposals whose status is **derived** (material ambiguity ⇒
`clarification_required`); `admit_interpretation` is the only constructor of an
admitted interpretation (authority identity + exact digest); `mint_task_contract`
requires a decidable acceptance condition and mints under contract-epoch CAS.
The production `mint_schedulable_task_contract` path additionally publishes the
contract event plus its governed Task at `DRAFT`, contract-named Loop at
`START`, contract-named hard Budget, and current-epoch runnable scheduler row in
one fenced store transaction:
successful admission cannot expose only a subset. `supersede_task_contract`
uses the same schedulable publication, fences old-epoch work
(`INTENT_VERSION_SUPERSEDED` at both mint and dispatch sinks), and classifies
pending Effects for reconciliation. Startup repair calls the same pure
`prepare_task_execution_bootstrap` composition for the current immutable
contract; it can restore missing prerequisites but cannot replace existing
Task/Loop/Budget/scheduler authority. A current daemon-issued WIA then derives
the registered `DRAFT -> READY -> ACTIVE` Task guards before Tool I/O.

## Effects: seven properties, four sinks

`mint_intent` enforces durable idempotency arithmetic: same key + same canonical
parameter digest replays; same key + different digest is
`EFFECT_IDEMPOTENCY_CONFLICT`. `EffectProtocol` drives
PROPOSED→AUTHORIZED→EXECUTING→…→COMMITTED with guards derived only from durable
reloads (`intent_durably_persisted`, `capability_and_revocation_current`,
`verification_still_current`); dispatch commits EXECUTING **before** the external
call; unknown outcomes reconcile with the original key or quarantine. All four
commit sinks (executor, authority commit, admission+outbox, checkpoint) re-check
the writer fencing epoch inside the store transaction. Verification entry is
also one compound authority commit: a current closed Effect pin, its request,
and Loop `ACT -> VERIFY` either all persist or all roll back.
A closed intermediate Effect that is not RegisteredCheckRun on a
RegisteredCheck-terminated Task instead walks
`ACT -> OBSERVE -> RESOLVE -> ORIENT -> DECIDE` from durable Effect and
contract facts so the next candidate can be admitted without completing the Task.
Public-admit Tasks start Loop at `START`; after a sealed ContextView exists,
`LoopDriver::advance_start_to_decide_after_context_view` walks
`START -> OBSERVE -> RESOLVE -> ORIENT -> DECIDE` before Pi. That walk is not
Task acceptance.
Task acceptance remains a separate authority: candidate and final acceptance
transitions are prepared by the same deterministic engine, then SQLite
transactionally rechecks current contract epoch, the complete closed Effect set,
fixed post-state, latest passed report, and fencing. The final principal is the
daemon-private acceptance authority, never the worker or verifier.

## Authorization and budgets

`authorize` runs six fail-closed steps (authn/chain → tenant/membership →
capability intersection + revocation currency → explicit deny wins → lease window
→ scope/purpose/action). Denials are existence-safe (denied ≡ not-found bytes).
`revalidate_grant` re-checks the F-007 race points at dispatch and commit time.
Budgets are pure integer ledgers over nine registered dimensions.

## Recovery

`RECOVERY_ORDER` fixes eight steps (barrier → identity/epoch → fence → replay →
reconcile → reauthorize → re-resolve context → resume loops); `run_recovery`
re-dispatches AUTHORIZED work exactly once with original keys, forces EXECUTING to
OUTCOME_UNKNOWN then reconciles, quarantines indeterminates, and resumes only
loops whose checkpoints validate (older epoch, watermark within replayed history).
