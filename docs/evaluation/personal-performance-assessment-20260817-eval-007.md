# PERSONAL-PERF-EVAL-007 assessment (running)

- Campaign: `PERSONAL-PERF-EVAL-007`
- Frozen source target: `main@2a8d4d2f` (P2-T31 closed)
- Lease: `lease/personal/EVAL-007/c1-c2-paired-freeze`
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: freeze **pass**; C1/C2 B0 in progress

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
| Freeze (archive/binaries/root/port) | **pass** | guest `/home/hal9001/perfeval007-20260817` mode `0700`; daemon `127.0.0.1:48292` pid 277358; archive `sha256:ca2a95b09a78…`; kernel-server `sha256:e603edab9a59…` |
| SecretStore import | **pass** | new item `/16` via stdin; D-Bus `SearchItems` paths only; never search/lookup |
| Pi 0.81.1 pin | **pass** | `--extension` absolute; doctor package/pinned/observed `0.81.1`; `first_conversation_ready: true` |
| Exact-source `pi-agent-adapter` | **pass** | same `2a8d4d2f` archive; `sha256:816856b496…`; `o-arm-candidate.mjs` `sha256:29870821…` |
| B0 C1/C2 paired | `not-run` | freeze pass recorded; cell starting next |
| B1 C1/C2 paired | `not-run` | after B0 |
| B2 C1/C2 paired | `not-run` | after B0 |
| Cleanup | `not-run` | stop 48292/48392; clear `/16`; leave 48181/48284/48383 and EVAL-004/005/006 roots |

## Freeze (2026-08-17) — pass

Exact source `main@2a8d4d2f`. Guest root mode `0700`. Listeners `48181` /
`48284` / `48383` untouched. SecretStore item `/16` is new (not `/12` /
`/13` / `/14` / `/15`). Public doctor: all required components `ready`, Pi
`0.81.1`, `first_conversation_ready: true`. That is conversation
readiness, not a C1/C2 Task. Claim ceiling `hypothesis`. No Gate, release,
Profile, B01, or Agent-benefit claim.

| Asset | Value |
|---|---|
| Archive | 14,622,720 bytes; 1536 entries; 0 `.git/` members; SHA-256 `ca2a95b09a78062cc55112211dac2d5de192aa3e353dafbbdd0572bcb4e1efed` |
| `kernel-server` | 16,534,712 bytes; SHA-256 `e603edab9a594e41177f89ac105b2755bff34cdb980c30faece03de87610ec55` |
| `cognitive` | 10,311,312 bytes; SHA-256 `0c443a5c56c55efdd92927d973d4acf9f00ad8d0007f51eca7fc2386baa713f2` |
| `pi-agent-adapter` | 1,126,192 bytes; SHA-256 `816856b49674d06f025f535fe2bf5219dd9744ab899250a489538ea687aa3167` |
| Pi tarball `@earendil-works/pi-coding-agent@0.81.1` | 4,967,228 bytes; SHA-256 `420113c0282160e6181656fd16cf18742f76bf9040ee3dfb9cb67e3e6ad5641c` |
| Pi `package-lock.json` | SHA-256 `6019918b044400744713d2fef985027f092db2d7952e177ca6558f6e2a93c2ca` |
| `pi.json` | SHA-256 `a8058c75fa47661d9d6c2ac6898039d00a08df149a6ad13ee4da7f0951fea862`; absolute paths only |
| `dist/index.js` | SHA-256 `d27f97764e55b9a9b22bbf7e22e48c0ef2a017924ed13684b143b196991c1a57` |
| `o-arm-candidate.mjs` | SHA-256 `29870821488451b5728f88c4612e1616fd65681adaf23011dd898d459428e573` |

Host `ldd` on `kernel-server` resolves only glibc/`libgcc`/`libm`. Windows
GNU Rust build remains `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).

## Non-claims

Hypothesis only. No Gate, release, Profile, B01, or Agent-benefit promotion.
**Rotate the Provider key** leaked by the earlier EVAL-004 `secret-tool search`
incident. Do not print it. Never `secret-tool search`/`lookup`.
