# P9-T11 running validation report

- Task: `P9-T11` comparable C1/C2 P/O WorkspacePatch unified-diff payload
- Branch: `personal/P9-T11-c2a-patch-unified-diff`
- Lease: `lease/personal/P9-T11/c2a-patch-unified-diff`
- Claim ceiling: `hypothesis` / non-claim. No Gate, release, Profile, B01,
  EVAL, or Agent-benefit promotion.

## Why this task exists

`PERSONAL-PERF-EVAL-012` C2a P-arm Patch passed by treating `input_b64` as
replacement bytes. O-arm production Patch applies a strict unified diff.
Those cells are not comparable. P-arm must apply the same UTF-8 unified-diff
payload the daemon would accept and refuse replacement bytes.

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. Formal registration — **pass**: P9-T11, slices D01–D03, lease
   `lease/personal/P9-T11/c2a-patch-unified-diff`, Layer 1 `102/94/1/1/6/8`.
2. Local `node --test test/c1_c2_paired_p_arm.test.mjs` — **pass 19/19**,
   including replacement-bytes refusal, frozen C2a unified diff to oracle, and
   daemon no-newline marker cases.
3. Full tools suite — **pass 83/83**.
4. `check-consistency`, `check-handbook`, `generate-handbook --check` — **pass**.
5. Required CI — **not-run** until the Draft PR head is pushed.
6. Live O-arm / P-arm sample execution — **not-run** (out of scope; no new
   EVAL).
7. Windows GNU Rust build/test/Clippy — **not-run** (`RUST-LINK-DEV-WIN-GNU-01`;
   this task has no Rust change).

## Acceptance mapping

| Formal acceptance | Evidence |
|---|---|
| Replacement-bytes WorkspacePatch fails closed | Local test `C2a WorkspacePatch refuses replacement bytes and applies a unified diff` **pass**; target unchanged |
| Invalid / empty unified diff fails closed | Local test `applyUnifiedPatch matches daemon no-newline marker cases and fails closed` **pass** |
| Valid unified diff applies to the expected post-state | same test plus `C2a frozen unified diff repairs the C2a corpus to the oracle` **pass** |
| Freeze ledger pins `workspace_patch_payload: unified-diff` | Local freeze ledger test **pass**; corpus includes `fixtures/c2a/workspace-patch.unified.diff` |
| Supported validation | Local tools **83/83**; required CI after push |
| Live EVAL / B0 | **not-run**; later campaign after this task |

## Non-claims

This report does not reopen EVAL-012, start a new EVAL, or promote Gate,
release, Profile, B01, or Agent-benefit. Private-candidate epoch-1 skips
remain out of scope. Matching P-arm and O-arm payload format is not a live
C2a sample pass.
