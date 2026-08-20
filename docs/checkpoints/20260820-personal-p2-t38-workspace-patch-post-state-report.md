# P2-T38 running validation report

- Task: `P2-T38` public WorkspacePatch post-state (EVAL-012 Priority 1)
- Branch: `personal/P2-T38-workspace-patch-post-state`
- Lease: `lease/personal/P2-T38/workspace-patch-post-state`
- Claim ceiling: `hypothesis` / non-claim. No Gate, release, Profile, B01,
  EVAL, or Agent-benefit promotion.

## Why this task exists

`PERSONAL-PERF-EVAL-012` closed on `evaluation/EVAL-012-freeze` @ `177669b7`.
Counted C2a O-arm Patch retained `ACTIVE` / `must_reconcile` with scheduler
`native Tool execution failed closed: fixed post-state is unavailable` while
the seeded file stayed `c2a-patch-v1` plus LF
(`sha256:cb4ff53fe48499826134116581f605c9ed95cc37cfb3d0e42aac028b87c99c0f`).
P2-T37 already completes Patch when the candidate uses the domain-tagged
workspace-image digest `sha256:575ba073…`. The paired P-arm fixture and
`sha256sum` name the raw file bytes instead.

Two product defects follow:

1. Expected preimage `digest:sha256:<raw file SHA-256>` did not match the
   domain-tagged snapshot, so Patch returned `NotExecuted` and the file was
   unchanged.
2. The production worker ignored `SchedulerEffectClosure::PendingReconciliation`
   and still called `begin_verification`, which fail-closed as
   `fixed post-state is unavailable`.

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. Formal registration — **pass**: P2-T38, slices D01–D03, lease, Layer 1
   `99/91/1/1/6/8`. `pnpm run check:consistency` **pass** (275 requirements,
   55 error codes, 74 schemas, 89 vectors).
2. Local Windows GNU Rust build/test — **not-run** (`RUST-LINK-DEV-WIN-GNU-01`).
3. Focused tests added (not yet executed on a supported host):
   - `workspace_patch_accepts_raw_file_sha256_as_equivalent_cas_preimage`
   - `c2a_raw_file_sha256_preimage_reaches_production_patch_sink`
   - `c2a_preimage_mismatch_does_not_request_verification`
4. Implementation — written: `preimage_matches_snapshot` accepts raw file
   SHA-256; `run_bounded_scheduler_attempt` returns
   `AwaitingReconciliation` instead of requesting verification when native
   dispatch leaves the Effect pending.
5. Local docs/format — **pass**: `cargo fmt --all`; `git diff --check`;
   `pnpm run check:handbook` (54 docs × 2 locales); `generate-handbook --check`
   (18 pages byte-identical). Handbook fingerprints refreshed; no
   `DOCS_IMPACT_NONE`.
6. Exact-revision Linux / required CI — **not-run** (next after push).

## Non-claims

This report does not reopen EVAL-012, start a new EVAL, or promote Gate,
release, Profile, B01, or Agent-benefit. Replacement-bytes Patch payload
(EVAL-012 Priority 3) remains out of scope.
