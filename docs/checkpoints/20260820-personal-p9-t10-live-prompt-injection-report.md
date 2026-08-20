# P9-T10 running validation report

- Task: `P9-T10` live C1/C2 P/O `--append-system-prompt` injection
- Branch: `personal/P9-T10-live-prompt-injection`
- Lease: `lease/personal/P9-T10/live-prompt-injection`
- Claim ceiling: `hypothesis` / non-claim. No Gate, release, Profile, B01,
  EVAL, or Agent-benefit promotion.

## Why this task exists

`PERSONAL-PERF-EVAL-012` B0 fairness failed only `system_task_prompt_bytes`.
P9-T09 froze dry-run observation of `frozen-system-task-prompt.txt`. Live
O-arm `cognitive pi launch` still did not forward Pi `--append-system-prompt`,
so a new EVAL would fail the same axis.

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. Formal registration — **pass**: P9-T10, slices D01–D03, lease
   `lease/personal/P9-T10/live-prompt-injection`, Layer 1 `101/93/1/1/6/8`.
2. Local `node --test test/c1_c2_paired_p_arm.test.mjs` — **pass 16/16**,
   including `live P and O command manifests share --append-system-prompt and
   the freeze file`. Full tools suite **80/80**.
3. `DEV-LINUX-NATIVE-01` exact `745c60d8` — **pass**: admin-cli
   `append_system_prompt` **3/3**, print-mode regression **1/1**,
   `cargo clippy -p admin-cli --all-targets --locked -- -D warnings`,
   `cargo fmt --all -- --check`. Rustc 1.97.1.
4. Required CI `32350029054` — **pass** at `745c60d8`: Ubuntu verify,
   Windows verify, `resolve validation route`, and `required-ci`.
5. Live O-arm / P-arm sample execution — **not-run** (out of scope; no new
   EVAL).
6. Windows GNU Rust build/test/Clippy — **not-run** (`RUST-LINK-DEV-WIN-GNU-01`).

## Acceptance mapping

| Formal acceptance | Evidence |
|---|---|
| Relative, missing, and empty `--append-system-prompt` fail closed | Linux `launch_preparation_rejects_relative_empty_or_missing_append_system_prompt` **pass**; parse accepts the flag without secret flags |
| Absolute existing non-empty file is forwarded after `--print` | Linux `launch_preparation_forwards_append_system_prompt_after_print` **pass** |
| Live P/O command manifests share the freeze file | Local test `live P and O command manifests share --append-system-prompt and the freeze file` **pass** |
| `system_task_prompt_bytes` remains the freeze-file byte length | existing dry-run test plus `liveArmCommandManifest().system_task_prompt_bytes` |
| Without the flag, argv stays `--extension`/`--tools`/`--print` | Linux `launch_preparation_disables_pi_native_tools_and_preserves_print_mode` **pass** |
| Supported validation | Required CI `32350029054` Ubuntu/Windows/required-ci **pass**; local tools **80/80**; Linux Clippy/fmt **pass** |
| Live EVAL / B0 | **not-run**; later campaign after this task |

## Non-claims

This report does not reopen EVAL-012, start a new EVAL, or promote Gate,
release, Profile, B01, or Agent-benefit. Replacement-bytes Patch and
private-candidate epoch-1 skips remain out of scope. Shipping the CLI flag
is not a live B0 fairness pass.
