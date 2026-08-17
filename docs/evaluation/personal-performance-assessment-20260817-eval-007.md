# PERSONAL-PERF-EVAL-007 assessment (running)

- Campaign: `PERSONAL-PERF-EVAL-007`
- Frozen source target: `main@2a8d4d2f` (P2-T31 closed)
- Lease: `lease/personal/EVAL-007/c1-c2-paired-freeze`
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: freeze lease claimed; guest freeze `not-run`

This is the campaign's single running report. Append each finished cell before
starting the next (`TEST-REPORT-INCREMENTAL-01`). Measurement-only: no product
code, contract, negative, test, or handbook source change.

Owner 2026-08-17 authorized C1/C2 re-measure after P2-T31. EVAL-006 B0 on
`main@103fe776` skipped with `scheduler_row_skip_before_lease` on the live
daemon. P2-T31 made live HTTP admit share the daemon store, accept a
stdout-valid stub candidate without waiting on the unused Provider socket,
and treat the first dispatch as not a retry.

## Cells

| Cell | Status | Note |
|---|---|---|
| Freeze (archive/binaries/root/port) | `not-run` | planned root `/home/hal9001/perfeval007-20260817`; daemon `127.0.0.1:48292`; do not reuse EVAL-004/005/006 roots or ports `48286`/`48288`/`48290` |
| SecretStore import | `not-run` | new item via stdin; D-Bus `SearchItems` paths only; never search/lookup |
| Pi 0.81.1 pin | `not-run` | `--extension` absolute |
| Exact-source `pi-agent-adapter` | `not-run` | same `2a8d4d2f` archive |
| B0 C1/C2 paired | `not-run` | after freeze |
| B1 C1/C2 paired | `not-run` | after B0 |
| B2 C1/C2 paired | `not-run` | after B0 |
| Cleanup | `not-run` | stop 48292/48392; clear new SecretStore item; leave 48181/48284/48383 and EVAL-004/005/006 roots |

## Non-claims

Hypothesis only. No Gate, release, Profile, B01, or Agent-benefit promotion.
**Rotate the Provider key** leaked by the earlier EVAL-004 `secret-tool search`
incident. Do not print it. Never `secret-tool search`/`lookup`.
