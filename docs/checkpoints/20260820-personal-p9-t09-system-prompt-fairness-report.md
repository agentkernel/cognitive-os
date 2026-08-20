# P9-T09 running validation report

- Task: `P9-T09` C1/C2 P/O `system_task_prompt_bytes` (EVAL-012 Priority 2)
- Branch: `personal/P9-T09-system-prompt-fairness`
- Lease: `lease/personal/P9-T09/system-prompt-fairness`
- Claim ceiling: `hypothesis` / non-claim. No Gate, release, Profile, B01,
  EVAL, or Agent-benefit promotion.

## Why this task exists

`PERSONAL-PERF-EVAL-012` B0 fairness failed only `system_task_prompt_bytes`
(P-arm short campaign instruction vs O-arm CognitiveOS Extension session).
The P9-T08 dry-run forged equality with placeholder `frozen-c1-c2-prompt-v1`.

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. Formal registration — **pass**: P9-T09, slices D01–D03, lease, Layer 1
   `100/92/1/1/6/8`. `pnpm run check:consistency` **pass**.
2. Local `node --test test/c1_c2_paired_p_arm.test.mjs` — **pass 15/15**,
   including `system_task_prompt_bytes is the frozen prompt length, not a
   shared placeholder`. Dry-run observes
   `frozen-system-task-prompt.txt` byte length on both arms.
3. Live O-arm injection of the same bytes — **not-run** (later slice).
4. Required CI — **not-run**.

## Non-claims

This report does not reopen EVAL-012, start a new EVAL, or promote Gate,
release, Profile, B01, or Agent-benefit. Replacement-bytes Patch and
private-candidate epoch-1 skips remain out of scope.
