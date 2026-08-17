# PERSONAL-PERF-EVAL-009 assessment (running)

- Campaign: `PERSONAL-PERF-EVAL-009`
- Frozen source target: `fb85cfff25d8dd9fc5e3a8743ab9fdb3b3586630` (P2-T32
  public launcher; unmerged freeze; same pin as EVAL-008)
- Lease: `lease/personal/EVAL-009/c1-c2-paired-freeze`
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: campaign **active**. Measurement-only. Evaluation routing ON.

This is the campaign's single running report. Append each finished cell before
starting the next (`TEST-REPORT-INCREMENTAL-01`).

Owner 2026-08-18 authorized continuing C1/C2 and remaining execution-plan
真机 cells after EVAL-008 close, and authorized solving recoverable blockers.
EVAL-008 is **closed** and is not reopened. This freeze keeps pin `fb85cfff`
and uses a short unique root so Linux Unix-domain socket bind is hypothesized
to succeed (`UNIX_PATH_MAX` 108). That is not a product patch.

## Cells

| Cell | Status | Note |
|---|---|---|
| EVAL-008 remains closed (coordination) | **pass** | do not reuse `48294` / `/17` / `perfeval008-20260818` runtime |
| Freeze (archive/binaries/root/port) | pending | pin `fb85cfff`; root `/home/hal9001/e009`; daemon `127.0.0.1:48296` |
| SecretStore import | pending | new item; D-Bus `SearchItems` paths only; never search/lookup |
| Pi 0.81.1 pin | pending | `--extension` absolute; doctor ready is **not** C1/C2 |
| Exact-source `pi-agent-adapter` | pending | real adapter, not the P2-T32 stub |
| B0 C1 WorkspaceSearch O-arm | pending | first qualification sample |
| B0 remaining C1/C2 families | pending | only if C1-search leaves `DRAFT` with `lease_acquired` ≥ 1 |
| B0 P-arm / broker `48396` | pending | only after O-arm is fairly measurable |
| B1 C1/C2 paired | pending | only after B0 path/fairness |
| B2 C1/C2 paired | pending | freeze N after B1 |
| B3 faults | pending | no cobbled runner |
| B4 concurrency | pending | after B3 |
| B5 soak | pending | 1 h first; 8 h only after 1 h; 24 h default deferred |
| C0 paired (G1/G2/G3/G4/G6/G9, A1/A4/A5) | pending | `not-run` unless broker/runner qualify; do not cobble a paired shell |
| Cleanup | pending | stop `48296`; clear this campaign SecretStore item |

## Non-claims

No Gate, release, Profile, B01, B01-W, or Agent-benefit promotion. Do not
treat P2-T32 stub pass or a short-root workaround as EVAL-007/008 repaired.
Never `secret-tool search`/`lookup`. Do not print Provider keys.
