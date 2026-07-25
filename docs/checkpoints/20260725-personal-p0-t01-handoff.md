# 20260725 Personal P0-T01 Handoff

## 1. Task and base

- Task: `P0-T01 - Fixed reproducible baseline and supported toolchain`.
- Date: 2026-07-25.
- Lane / branch: Lane-DOC / `lane/personal-p0-t01-baseline`.
- Base commit: `f32efd1d188e4da352304228ae73a66a96686d16`.
- Classification: documentation-only baseline evidence and owner-decision
  blocker; no machine-contract, implementation, schema, vector, or Profile
  change.

## 2. Completed

- Created `tests/baseline/README.md` with the reproducible P0-T01 command set,
  the actual local results, and the exact boundary between CI evidence and local
  GNU-host evidence.
- Recorded the current task as `blocked` in
  `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`; Phase 0 now reports one blocked
  task and six not-started tasks.
- Updated `docs/plan/PROGRESS.md` with the Personal planning status. It
  explicitly states that this is not REQ, conformance, or Profile evidence.
- No REQ-ID, error code, finding, drift item, schema, transition, vector, or
  generated binding was touched. `personal-blog/` was not inspected, changed,
  staged, or included.

## 3. Executed verification and evidence

| Command / evidence | Actual result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `pnpm install --frozen-lockfile` | pass |
| `pnpm -r build` | pass |
| `pnpm -r test` | pass |
| `pnpm run check:consistency` | pass: 273 requirements, 55 errors, 63 schemas, 85 vectors |
| `node tools/src/gen-matrix.mjs --check` | pass |
| `cargo build --workspace --locked` | fail, exit 101: local `x86_64-pc-windows-gnu` linker could not resolve GNU runtime libraries including `-lgcc_eh`; linker exit 121 |
| `cargo test --workspace --locked` | not-run; build prerequisite failed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | not-run; build prerequisite failed |
| GitHub Actions [`30135181631`](https://github.com/agentkernel/cognitive-os/actions/runs/30135181631) | pass: both `verify (ubuntu-latest)` and `verify (windows-latest)` completed all CI steps successfully for `f32efd1` |

No local artifacts are claimed as durable evidence. CI is clean Linux runner
evidence for this source commit. The local probe did not access secrets or
produce a release/Profile claim.

## 4. Blocker and status boundary

`P0-T01` remains **blocked**, not done. The formal task acceptance requires a
reproducible conclusion for the supported Windows toolchain. Existing evidence
shows hosted Windows CI/MSVC passes while this local GNU host cannot link the
Rust workspace. Choosing to support MSVC only, repair/support GNU, or declare
Personal Linux-only is a product/compatibility owner decision and must not be
inferred from CI alone.

The open P0 evidence finding F-001 is an existing global evidence-class item;
it did not block this documentation/baseline task and was not changed. No new
drift was found, so `findings-ledger.md` is unchanged.

## 5. Next entry

1. Obtain the owner's Windows support decision.
2. Record it in an ADR or an explicit support-matrix decision as appropriate.
3. Run the matching clean baseline (MSVC, repaired GNU, or Linux-only) and
   update `tests/baseline/README.md`, the Personal plan, PROGRESS, and this
   handoff before moving P0-T01 to `done`.

Suggested prompt: "Continue Personal P0-T01 from
`docs/checkpoints/20260725-personal-p0-t01-handoff.md`; apply the approved
Windows support decision, rerun the required clean baseline, and update the
task status truthfully."

## 6. Snapshot

- PROGRESS updated: yes.
- Personal plan updated: yes, `P0-T01 = blocked`.
- Commit: pending at handoff creation.
- PR / CI for this documentation branch: pending.
