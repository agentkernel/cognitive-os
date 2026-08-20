# P9-T12 running validation report

- Task: `P9-T12` live C1/C2 campaign-only paired executor
- Branch: `personal/P9-T12-live-paired-executor`
- Lease: `lease/personal/P9-T12/live-paired-executor`
- Claim ceiling: `hypothesis` / non-claim. No Gate, release, Profile, B01,
  EVAL, or Agent-benefit promotion.

## Why this task exists

Closed measurement on `evaluation/EVAL-013-freeze` recorded C1/C2a B0
fairness pass, then left B1/B2 `not-run` because
`tools/personal/c1-c2-paired/paired-runner.mjs` emitted dry-run fairness only
(`counted_sample: false`, `b0: false`). Execution plan §2.5 forbids cobbling
B0 shell into a formal paired campaign. This task freezes a live paired
executor; it does not run B1/B2 samples.

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. Formal registration — **pass**: P9-T12, slices D01–D03, lease
   `lease/personal/P9-T12/live-paired-executor`, Layer 1 `103/95/1/1/6/8`.
2. Local `node --test tools/test/c1_c2_paired_p_arm.test.mjs` — **pass 24/24**,
   including secret-shaped env refusal, missing `--append-system-prompt`,
   dry-run counted-label refusal, fairness-fail retain without spawn, stub
   live cell schema, and deterministic arm order.
3. Full tools suite — **pass 88/88**.
4. `check-consistency`, `check-handbook`, `generate-handbook --check` — **pass**.
5. Required CI `32370316101` — **pass** at `a7b09edd`: Ubuntu verify, Windows
   verify, `resolve validation route`, and `required-ci`.
6. Merge — **pass**: PR [#252](https://github.com/agentkernel/cognitive-os/pull/252)
   merged at `main@39cf8019`. Lease closed; local `main` matches `origin/main`.
7. Live Provider / B01 / B1/B2 sample execution — **not-run** (out of scope).
8. Windows GNU Rust build/test/Clippy — **not-run** (`RUST-LINK-DEV-WIN-GNU-01`;
   this task has no Rust change).

## Acceptance mapping

| Formal acceptance | Evidence |
|---|---|
| Secret-shaped env refuses | D01 test `live launch refuses secret-shaped env and missing --append-system-prompt` |
| Missing `--append-system-prompt` refuses | same test |
| Dry-run cannot be labeled counted B1/B2 | D01 test `dry-run cannot be labeled counted B1/B2` |
| Fairness fail retained, not counted, no arm spawn | D01 test `live paired cell retains fairness fail without counting or spawning arms` |
| Stub `executeArm` produces live schema with `append_system_prompt: true` | D02 test `stub executeArm live cell is counted only for b1/b2 with fairness pass` |
| `counted_sample` only for b1/b2 when both arms complete | same test; b0 and timeout remain non-counted |
| Supported validation | Local `c1_c2_paired_p_arm` **24/24**; tools suite **88/88**; `check-consistency` / `check-handbook` / `generate-handbook --check` **pass**. Required CI `32370316101` Ubuntu/Windows/required-ci **pass** at `a7b09edd`. |
| Live EVAL / B1 / B2 | **not-run**; later campaign after this task |

## Non-claims

This report does not reopen EVAL-012 or EVAL-013, start EVAL-014, or promote
Gate, release, Profile, B01, or Agent-benefit. Implementing a live paired
runner is not a counted sample.
