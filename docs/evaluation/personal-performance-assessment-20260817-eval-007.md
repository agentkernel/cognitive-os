# PERSONAL-PERF-EVAL-007 assessment (final)

- Campaign: `PERSONAL-PERF-EVAL-007`
- Frozen source target: `main@2a8d4d2f` (P2-T31 closed)
- Lease: `lease/personal/EVAL-007/c1-c2-paired-freeze` (closed 2026-08-17)
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: campaign **closed**. Measurement-only. Evaluation routing OFF.

This is the campaign's single running report. Append each finished cell before
starting the next (`TEST-REPORT-INCREMENTAL-01`).

Owner 2026-08-17 authorized C1/C2 re-measure after P2-T31. EVAL-006 B0 on
`main@103fe776` skipped with `scheduler_row_skip_before_lease` on the live
daemon. P2-T31's focused test spawned `kernel-server --personal` with piped
stderr and passed. This campaign measured the public `cognitive daemon start`
launcher.

## Cells

| Cell | Status | Note |
|---|---|---|
| Freeze (archive/binaries/root/port) | **pass** | guest `/home/hal9001/perfeval007-20260817` mode `0700`; daemon `127.0.0.1:48292` pid 277358; archive `sha256:ca2a95b09a78…`; kernel-server `sha256:e603edab9a59…` |
| SecretStore import | **pass** | new item `/16` via stdin; D-Bus `SearchItems` paths only; never search/lookup |
| Pi 0.81.1 pin | **pass** | `--extension` absolute; doctor package/pinned/observed `0.81.1`; `first_conversation_ready: true` |
| Exact-source `pi-agent-adapter` | **pass** | same `2a8d4d2f` archive; `sha256:816856b496…`; `o-arm-candidate.mjs` `sha256:29870821…` |
| B0 C1/C2 paired | **partial** | one retained O-arm C1-search Task; public lifecycle `DRAFT`; `lease_acquired` 0; no Pi child; P-arm not-run |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| Cleanup | **pass** | daemon 48292 stopped; broker 48392 absent; SecretStore `/16` cleared; 48181/48284/48383 and EVAL-004/005/006 roots untouched; redactor 18/0 |

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

## B0 C1-search O-arm (2026-08-17) — partial; retained

One O-arm C1-search qualification Task was started with `retry=0` against
the public Task admit surface on the public `cognitive daemon start`
launcher. It is retained. It did not leave `DRAFT`. No Intent, Effect,
verification, or acceptance row exists. WorkspaceRead is still not
advertised; this cell used WorkspaceSearch only.

| Seed | Task ref | Admit | Probe | Lifecycle |
|---|---|---|---|---|
| `b0-0` | `task://local/eval007-b0-C1-search-b0-0-d37aaefff202` | 200 (record 14 ms, interpret 5 ms, preview 1 ms, admit 29 ms) | wall 179206 ms; `acceptance_ref` absent | `DRAFT`; minted `2026-08-17T15:11:37.942Z` |

Guest evidence file:
`/home/hal9001/perfeval007-20260817/evidence/b0-oarm-C1-search-b0-0.json`
`sha256:77ed3b392ab40398420285ea8756283a1497a9279eaf244f3c85bd42f1dee2fe`.
Instrument `eval007-b0-c1-search.py`
`sha256:f3338064e716822a5cdee19f7f75c75138af4b2f36bde0593e00c96c672e4709`.
Public `cognitive task evidence`: `lifecycle.current_state=DRAFT`;
`intent_refs` / `effect_refs` empty; `latest_verification` /
`latest_acceptance` null. Bounded O4: `runnable_count` 32,
`lease_acquired` 0 (observed_zero true). O5 named zero. No
`pi-agent-adapter` or `pi-coding-agent` child. Campaign kernel-server pid
277358. Listeners `48181` / `48284` / `48383` untouched. P-arm / broker
`48392` not started.

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or Agent-benefit
claim.

## Private-candidate skip (2026-08-17) — public facts match EVAL-006

Campaign-only observation. No product change. Public `cognitive status` /
`doctor` were ready (`first_conversation_ready: true`) before B0; that is
not C1/C2. The admitted Task stayed `DRAFT` for the full 180 s probe.
O4 `lease_acquired` 0. No Pi child. Freeze assets required for a spawn
were present (`pi.json` candidate paths, adapter binary, selected model).
This is therefore **not** a missing-freeze-asset `not_available`.

P2-T31's focused test spawns `kernel-server --personal` and pipes stderr.
This cell used public `cognitive daemon start --bind 127.0.0.1:48292`,
which sends stderr to `/dev/null`, so the exact per-row skip string is
**not a public fact**. Public symptoms are unchanged from EVAL-006:
`DRAFT`, `lease_acquired` 0, no Intent/Effect, no Pi child.

This is not a real public C1/C2 caller. Remaining paired B0 classes,
B1/B2, and P-arm stay `not-run`. Do not open Provider spend on a path
that never leaves `DRAFT`. The skip is a product mutex for a new formal
task after this close (do not collide with P2-T29/T30/T31): exercise the
**public `cognitive daemon start`** path, not only a test-spawned
`kernel-server --personal`.

## C1/C2 paired remainder (2026-08-17) — `not-run`

| Cell | Disposition | Cause |
|---|---|---|
| B0 remaining C1/C2 classes | `not-run` | same public skip; one C1-search O-arm sample was started and is retained |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| P-arm / broker `48392` | `not-run` | equivalent Pi adapter not started; fairness not measurable |
| O5/O6 as C1/C2 dependents | `not-run` | no Intent/Effect |
| T4–T5/T8/T9 | `not-run` | T8 is invocation-time descriptor drift; no public dispatch |
| B3 stale Task/epoch | `not-run` | no mutation path |

## Cleanup (2026-08-17) — pass

Owner 2026-08-17 authorized measurement close after B0 path/fairness failed
on the public daemon launcher. Guest route unchanged:
`wuz@192.168.1.2` ProxyJump `hal9001@192.168.123.160`.

| Check | Result |
|---|---|
| campaign daemon `127.0.0.1:48292` pid 277358 | product `cognitive daemon stop` `action=stopped`; lock absent; listener absent |
| campaign broker `127.0.0.1:48392` | never started; listener absent |
| listeners `48181` / `48284` / `48383` | untouched (`cos-current` / EVAL-004 residue / EVAL-002 broker) |
| EVAL-004/005/006 roots | untouched |
| SecretStore item `/16` | `secret-tool clear` on the product attribute triple (`application=cognitiveos-personal`, `provider=deepseek`, `purpose=provider-api-key`); post-clear D-Bus `SearchItems` unlocked/locked 0; `login` collection `Items` 0; never `secret-tool search`/`lookup` |
| redactor `evidence/` | 3 files, `key_shaped_hits=0` |
| redactor `runtime/` | 15 files, `key_shaped_hits=0` |
| campaign root | retained `0700` at `/home/hal9001/perfeval007-20260817` (evidence only) |

**Campaign closed.** C1/C2 paired remains `partial`/`not-run` on the public
`cognitive daemon start` path (Task stayed `DRAFT`; `lease_acquired` 0;
no Intent/Effect; no Pi child). That skip is a product mutex for a new
formal task after this close. Claim ceiling `hypothesis`, verifier
`not_reviewed`. No Gate, release, Profile, B01, or Agent-benefit
promotion.

The EVAL-004 `secret-tool search` leak is unchanged: the owner must rotate
the Provider key. This session did not print secret material.

## Non-claims

No Gate, release, Profile, B01, B01-W, or Agent-benefit promotion. No
optimization success. Owner 2026-08-17 authorized backlog continuation
after this close; evaluation routing is OFF.
**Rotate the Provider key** leaked by the earlier EVAL-004 `secret-tool search`
incident. Do not print it. Never `secret-tool search`/`lookup`.
