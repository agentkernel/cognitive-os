# PERSONAL-PERF-EVAL-008 assessment (running)

- Campaign: `PERSONAL-PERF-EVAL-008`
- Frozen source target: `fb85cfff25d8dd9fc5e3a8743ab9fdb3b3586630` (P2-T32
  public launcher; unmerged freeze)
- Lease: `lease/personal/EVAL-008/c1-c2-paired-freeze`
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: campaign **active**. Measurement-only. Evaluation routing ON.

This is the campaign's single running report. Append each finished cell before
starting the next (`TEST-REPORT-INCREMENTAL-01`).

Owner 2026-08-18 authorized C1/C2 真机 re-measure after P2-T32 public-launcher
stub proof. EVAL-007 B0 on `main@2a8d4d2f` stayed `DRAFT` (`lease_acquired` 0,
no Pi child) because public `cognitive daemon start` sent stderr to
`/dev/null`. P2-T32 retains `daemon.log` (mode `0600`) and Unix
`process_group(0)`. Stub Workspace* tests pass; this campaign uses a real
`pi-agent-adapter`.

## Cells

| Cell | Status | Note |
|---|---|---|
| P2-T32 lease close (coordination) | **pass** | task remains in-progress pending Windows merge; not a C1/C2 pass |
| Freeze (archive/binaries/root/port) | `not-run` | pin `fb85cfff`; root `/home/hal9001/perfeval008-20260818`; daemon `48294` |
| SecretStore import | `not-run` | expected new item `/17` via stdin; D-Bus paths only |
| Pi 0.81.1 pin | `not-run` | `--extension` absolute; doctor is not C1/C2 |
| Exact-source `pi-agent-adapter` | `not-run` | same `fb85cfff` archive; not test stub |
| B0 C1 WorkspaceSearch O-arm | `not-run` | first qualification sample; WorkspaceSearch only |
| B0 remaining C1/C2 families | `not-run` | C2a/C2b/C2c/C2d only if O-arm leaves `DRAFT` and `lease_acquired` ≥ 1 |
| B0 P-arm / broker `48394` | `not-run` | only after O-arm path is fairly measurable |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B3 faults | `not-run` | after B2 |
| B4 concurrency | `not-run` | after B3 |
| B5 soak | `not-run` | 1 h first; 8 h only if 1 h has no leak; 24 h default deferred |
| C0 paired (G1/G2/G3/G4/G6/G9, A1/A4/A5) | `not-run` | if broker/runner still unqualified, keep `not-run` |
| Cleanup | `not-run` | stop `48294`; clear campaign SecretStore; leave prior roots/ports |

## Non-claims

No Gate, release, Profile, B01, B01-W, or Agent-benefit promotion. No
optimization success. Never `secret-tool search`/`lookup`. Do not print
Provider keys. Do not treat P2-T32 stub pass as EVAL-007 repaired or as
C1/C2 Agent benefit.
