# P9-T06 readiness SecretStore coalesce — running validation

- Task: `P9-T06/D01`
- Lease: `lease/personal/P9-T06/readiness-secretstore-coalesce`
- Branch: `personal/P9-T06-readiness-secretstore-coalesce`
- Claim ceiling: non-claim (no Gate/release/Profile)

## Why this task exists

The primary CI flake (`p1_t05_personal_readiness` 2 s startup wait) is already
leased on Draft PR #213 with both required jobs green. This slice is the
preregistered fallback: stop `status`/`doctor` paying a duplicated SecretStore
round-trip (~1.8 s post-P2-T11) without caching secret material or introducing
a stale-ready TTL.

## Change

`evaluate_personal_readiness` binds one SecretStore per evaluation:

- secret probe and provider `secret_ref` resolve share that backend;
- `get` is skipped when the probe already proved the backend cannot answer;
- resolved material is dropped immediately and never enters a fact;
- a later request is evaluated again (no cross-request ready cache).

Existing injected overrides keep their previous precedence, so locked/dangling
and config-snapshot negatives are unchanged.

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

| Unit | Environment | Outcome | Notes |
|---|---|---|---|
| `cargo fmt --all -- --check` | `DEV-WIN-GNU-01` | **pass** | format only; no linking |
| focused `personal::readiness` | required CI / native Linux | pending | local GNU linking is `not-run` per `RUST-LINK-DEV-WIN-GNU-01` |
| `check:consistency` | `DEV-WIN-GNU-01` | **pass** | 275 requirements, leases verified |
| `check-handbook` | `DEV-WIN-GNU-01` | **pass** | 54 docs × 2 locales |
| `generate-handbook --check` | `DEV-WIN-GNU-01` | **pass** | 18 pages byte-identical |
| docs-sync gate | `DEV-WIN-GNU-01` | pending commit | mapped `daemon-http` pages updated bilingually; fingerprints refreshed |

### 2026-08-14 integration recovery

- P9-T05 completed first through separate PR #213, merge
  `d24f7d00cec358f973eeb805671a37cac9a942c7`.
- That main revision was merged into this branch without rewriting the existing
  P9-T06 commit. The only conflicts were the independently added P9-T05/P9-T06
  task rows and totals in `PERSONAL-DEVELOPMENT-PLAN.md` and `PROGRESS.md`;
  resolution retains both tasks, P9-T05 `done`, P9-T06 `in-progress`, and the
  P9-T06 active exact-path lease.
- `node tools/src/check-consistency.mjs` after resolution — **pass**: 275
  requirements, 55 error codes, 74 schemas, 89 vectors, task counts, delivery
  slices and leases verified.
- `cargo fmt --all -- --check` after integration — **pass** on
  `DEV-WIN-GNU-01` (format-only; no Rust compile/link).
- `node tools/src/check-handbook.mjs` after integration — **pass**: 54
  documents × 2 locales, generated-page coverage, links, fingerprints, dynamic
  status and secret checks verified.
- `node tools/src/generate-handbook.mjs --check` after integration — **pass**:
  18 generated pages byte-identical.
- `node tools/src/docs-sync-gate.mjs --staged` on the merge resolution —
  **pass**: the 15 staged integration paths introduced no new handbook-routed
  delta beyond the already synchronized P9-T06 commit.
- `git diff --check` and `git diff --cached --check` — **pass**.

## Non-claims

This is not a Gate, release, Profile, B01, or measured latency claim. The
counting-store negatives prove call coalescing, not wall-clock improvement on
a production Secret Service.
