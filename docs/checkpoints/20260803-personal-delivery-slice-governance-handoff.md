# Personal Delivery Slice governance handoff

- Date: 2026-08-03
- Lease: `lease/personal/P2-T03/delivery-slice-governance`
- Branch: `main`
- Change class: corrective repository-governance + task-status reconciliation
- Normative/product surface: unchanged
- Gate/release/Profile claims: unchanged

## Outcome

The development workflow now separates three progress layers:

1. formal `P*-T*` task acceptance;
2. task-internal Delivery Slices with stable `<task-id>/DNN` identities;
3. Gate/campaign evidence and promotion.

The formal plan owns each slice's outcome, dependencies, exit and required
validation. `PROGRESS.md` Current snapshot alone owns current slice status.
Only `ready`, `in-progress`, `blocked`, `done` and `cancelled` are allowed.
One formal task may have at most one `in-progress` slice. An enabling slice
must be followed by a real caller, durable authority outcome or closed
end-to-end property; same-task helper-only chains are refused. Implementation
with required supported validation still `not-run` remains `blocked`, not
`done`.

The consistency checker now rejects duplicate/undefined slice definitions,
missing definition fields, missing or duplicate current statuses, unknown
status values, more than one in-progress slice per task, trace status drift,
and disagreement between the formal task summary and the Layer 1 progress
summary. The failure-injection test exercises duplicate slice definitions,
WIP overflow and trace-status drift in addition to its prior governance
checks.

## Corrective task reconciliation

`P2-T01` is now `done`, without changing its acceptance criteria. PR #127 at
`main@7f763c8` already delivered and verified the complete
proposal/clarify/preview/admit/control/query service, raw-intent durability,
preview-digest binding, epoch supersession/fencing and stale-writer refusal.
Linux focused tests, management/store regressions, Clippy/fmt and required CI
were already green. B02/B04/B05/B12 remain `not-run`; no Gate, release or
Profile claim follows from this correction.

The formal task count is now 16 done, 2 in progress, 0 blocked and 35 not
started (37 remaining). P1-T09 and P2-T03 are the two in-progress tasks.

## Current executable queue

- `P2-T03/D01` and `P2-T03/D02`: done with retained Linux evidence.
- `P2-T03/D03` and `P2-T03/D04`: implementation exists, but required
  exact-revision Linux validation did not run because SSH host-key verification
  failed before remote execution; both remain blocked.
- `P2-T03/D05`: blocked by the D03/D04 exits; no new P2-T03 helper slice is
  permitted.
- `P2-T02/D01`: ready as the independent real Task API/watch vertical path.

B01 remains fixed N=20 with two immutable outcomes (one success and one
failure). This governance change does not move the clean-reset checkpoint,
reclassify attempt 2, change the denominator or relax any campaign threshold.

## Verification

| Check | Result |
|---|---|
| `pnpm run check:consistency` | pass |
| `pnpm --filter @cognitiveos/repo-tools test` | pass; 5/5 including governance failure injection |
| `node --check tools/src/check-consistency.mjs` | pass |
| `node --check tools/test/check.test.mjs` | pass |
| `git diff --check` | pass |
| combined PowerShell syntax command using `&&` | not-run; PowerShell 5.1 rejected the separator before either command, then both commands were rerun separately and passed |

## Next action

Choose one non-overlapping executable outlet:

1. restore approved Linux SSH host-key trust and close P2-T03/D03 then D04
   with exact-revision focused validation; or
2. claim P2-T02/D01 and implement the real Task API/watch vertical path while
   P2-T03 validation infrastructure remains blocked.

Do not start P2-T03/D05 or another P2-T03 helper-only slice before D03/D04
required validation closes.
