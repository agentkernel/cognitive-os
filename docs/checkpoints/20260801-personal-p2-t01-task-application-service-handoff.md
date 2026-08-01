# P2-T01 TaskApplicationService handoff

- Date: 2026-08-01
- Task: P2-T01 TaskApplicationService
- Lease: `lease/personal/P2-T01/task-application-service`
- Branch: `lane/personal-p2-t01-task-application-service`
- Change class: implementation-only
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: `tested-local`
- Normative surface: unchanged

## Delivered slice

The L5 task lifecycle entry point (`TaskApplicationService`) now composes the
L3/L4 intent-chain kernel primitives over the real SQLite WAL store:

- `proposal` — durably fixes the raw user intent BEFORE any interpretation
  (`record_user_intent`; the kernel refuses an interpretation without a
  record);
- `clarify` — persists an interpretation candidate against a fixed record
  (material ambiguities stay `clarification_required`);
- `preview` — emits a canonical digest-bound TaskContract preview
  (non-persisted; `preview_digest` must be carried into admission);
- `admit` — rejects a preview-digest mismatch before any kernel mutation,
  then runs the kernel admission gate (authority identity + accepted digest
  against the persisted candidate) and mints the contract under the store's
  epoch CAS;
- `control` — user-correction supersession mints epoch N+1 and fences
  old-epoch bindings;
- `query` — read-only intent projection.

No SQLite table is added; all persistence is the existing
`user_intent_records` / `intent_interpretations` / `task_contracts` surface.
No parallel task state machine is introduced (DEC-P-07).

## Evidence

| Check | Result |
|---|---|
| `cargo test -p cognitive-runtime --test p2_t01_task_application_service` (Linux host) | pass; 4/4 |
| `cargo clippy -p cognitive-management --all-targets` | pass |
| `cargo clippy -p cognitive-runtime --test p2_t01_task_application_service` | pass |
| `cargo test -p cognitive-management --lib` | pass; 3/3 |
| `cargo test -p cognitive-store --test m5_intent_chain` | pass; 6/6 |
| `cargo fmt --all -- --check` | pass |
| Required CI (Ubuntu + Windows/MSVC) | pass |
| PR | [#127](https://github.com/agentkernel/cognitive-os/pull/127) merged as `main@7f763c8` |

## Coverage of P2-T01 acceptance bullets

- raw intent persisted before interpretation: covered by
  `proposal_persists_raw_intent_before_any_interpretation_or_task_contract`
  (store reopen replay + kernel ordering refusal);
- preview digest binds admission: covered by
  `preview_digest_mismatch_is_refused_before_any_kernel_mutation`
  (mismatch refused, no contract minted);
- revision produces new epoch and fences old task:
  `supersede_mints_new_epoch_and_fences_old_binding`
  (`verify_task_binding_current` rejects epoch 1, accepts epoch 2);
- stale writer lease refused: `stale_writer_lease_is_refused`.

Budget freezing at admission is surfaced through `ContractPreview`'s
`tool_calls_frozen`; the full Tier 0/1/2 authorization and budget-enforcement
integration belongs to P2-T02/P2-T03 and remains not-run here.

## Non-claims

No P2 acceptance Gate (B02/B04/B05/B12) result, no Profile claim, no release
claim, and no authority side effect beyond the kernel's own governed events.
