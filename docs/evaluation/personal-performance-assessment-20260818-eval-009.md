# PERSONAL-PERF-EVAL-009 assessment (final)

- Campaign: `PERSONAL-PERF-EVAL-009`
- Frozen source target: `fb85cfff25d8dd9fc5e3a8743ab9fdb3b3586630` (P2-T32
  public launcher; unmerged freeze; same pin as EVAL-008)
- Lease: `lease/personal/EVAL-009/c1-c2-paired-freeze` (closed 2026-08-18)
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: campaign **closed**. Measurement-only. Evaluation routing OFF.

This is the campaign's single report (`TEST-REPORT-INCREMENTAL-01`). Cells
were appended as they finished. The campaign is closed; do not append more
cells on this freeze.

Owner 2026-08-18 authorized continuing C1/C2 真机 after EVAL-008 close, and
authorized solving recoverable blockers. EVAL-008 is **closed** and is not
reopened. This freeze kept pin `fb85cfff` and used a short unique root so
Linux Unix-domain socket bind could be measured against `UNIX_PATH_MAX`
(108). That was a campaign isolation parameter, not a product patch.

Owner later authorized product changes. Those changes are **not** this
campaign. Closure here does not resume development by itself; the owner
delivery instruction for the next product task is separate.

## Cells

| Cell | Status | Note |
|---|---|---|
| EVAL-008 remains closed (coordination) | **pass** | do not reuse `48294` / `/17` / `perfeval008-20260818` runtime |
| Freeze (archive/binaries/root/port) | **pass** | pin `fb85cfff`; root `/home/hal9001/e009` mode `0700`; daemon `127.0.0.1:48296` pid 283539; `log_path` mode `0600`; binaries SHA-256-equal to EVAL-008 |
| SecretStore import | **pass** | new item `/org/freedesktop/secrets/collection/login/18` via stdin; D-Bus `SearchItems` paths only; never search/lookup |
| Pi 0.81.1 pin | **pass** | `--extension` absolute; doctor ready is **not** C1/C2 |
| Exact-source `pi-agent-adapter` | **pass** | real adapter, not the P2-T32 stub; SHA-256 `816856b49674d06f025f535fe2bf5219dd9744ab899250a489538ea687aa3167` |
| B0 C1 WorkspaceSearch O-arm | **partial** | one retained sample; admit 200; lifecycle `DRAFT`; `lease_acquired` 0; skip class `private_pi_candidate_adapter_rejected_the_request`; socket **was** created |
| B0 remaining C1/C2 families | `not-run` | C1-search did not leave `DRAFT` with `lease_acquired` ≥ 1 |
| B0 P-arm / broker `48396` | `not-run` | O-arm path not fairly measurable |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B3 faults | `not-run` | no frozen fault runner on this freeze; do not cobble |
| B4 concurrency | `not-run` | B0 path/fairness incomplete |
| B5 soak | `not-run` | no frozen soak runner; 1 h trigger not met |
| C0 paired (G1/G2/G3/G4/G6/G9, A1/A4/A5) | `not-run` | broker `48396` never started; do not cobble a paired shell |
| Cleanup | **pass** | daemon 48296 stopped; broker 48396 absent; SecretStore `/18` cleared; 48181/48284/48383 and EVAL-004/005/006/007/008 roots untouched |

## Freeze (2026-08-18) — pass

Exact source `fb85cfff25d8dd9fc5e3a8743ab9fdb3b3586630` (P2-T32 public
launcher; unmerged). Guest root mode `0700`. Listeners `48181` / `48284` /
`48383` untouched. SecretStore item `/18` is new (not `/12`–`/17`). Public
doctor was ready (`first_conversation_ready: true`). That is conversation
readiness, not a C1/C2 Task. Claim ceiling `hypothesis`. No Gate, release,
Profile, B01, or Agent-benefit claim.

Binaries were copied from the EVAL-008 leftover freeze after SHA-256 match
(not a runtime reuse of EVAL-008's daemon/port/SecretStore). Host `ldd` on
those binaries resolved only glibc/`libgcc`/`libm` in EVAL-008; this freeze
did not rebuild. Windows GNU Rust build remains `not-run`
(`RUST-LINK-DEV-WIN-GNU-01`).

Public start: endpoint `127.0.0.1:48296`, pid `283539`, `log_path`
`/home/hal9001/e009/runtime/state/cognitiveos/daemon.log` (mode `0600`).
Estimated nested `completion.sock` path length under this short root is
below Linux `UNIX_PATH_MAX` (108).

| Asset | Value |
|---|---|
| `kernel-server` | 16,534,712 bytes; SHA-256 `e603edab9a594e41177f89ac105b2755bff34cdb980c30faece03de87610ec55` (equal to EVAL-008) |
| `cognitive` | 10,313,736 bytes; SHA-256 `6917dca3a0f294c34d1f177dd5ebd3e1a36fff1c71de7661094049b30741a65f` (equal to EVAL-008) |
| `pi-agent-adapter` | 1,126,192 bytes; SHA-256 `816856b49674d06f025f535fe2bf5219dd9744ab899250a489538ea687aa3167` (equal to EVAL-008) |
| `o-arm-candidate.mjs` | SHA-256 `29870821488451b5728f88c4612e1616fd65681adaf23011dd898d459428e573` |
| `private_candidate_provider.mjs` | SHA-256 `2b7e52a6afe205e5997c58fe59b096fc7666dfd8733e196777e915d3a0bc245b` |

SecretStore import (stdin; D-Bus paths only): item
`/org/freedesktop/secrets/collection/login/18` (1 unlocked, 0 locked).
`login` collection `Items` contained only `/18` during the freeze.

## B0 C1-search O-arm (2026-08-18) — partial; retained

One O-arm C1-search qualification Task was started with `retry=0` against
the public Task admit surface on the public `cognitive daemon start`
launcher. It is retained. It did not leave `DRAFT`. No Intent, Effect,
verification, or acceptance row exists. WorkspaceRead is still not
advertised; this cell used WorkspaceSearch only.

UNIX_PATH_MAX hypothesis from EVAL-008: **confirmed as the EVAL-008 skip
class, and not this cell's skip.** A leftover
`private-completions/candidate-283539-*` directory existed; bind succeeded
on this short root. This is therefore **not**
`private_completion_socket_could_not_be_created`.

| Seed | Task ref | Admit | Probe | Lifecycle |
|---|---|---|---|---|
| `b0-0` | `task://local/eval009-b0-C1-search-b0-0-565e367a25c3` | 200 (record 17 ms, interpret 6 ms, preview 1 ms, admit 29 ms) | wall 179038 ms; 88 probes; `acceptance_ref` absent | `DRAFT` |

Guest evidence file:
`/home/hal9001/e009/evidence/b0-oarm-C1-search-b0-0.json`
SHA-256 `010e56b206c2051f927933e18aaa96eaaef773019d6d78641a796ebc58a5cacb`.
Public skip rows on `daemon.log` (mode `0600`):

`kernel-server personal scheduler tick: skip row
task://local/eval009-b0-C1-search-b0-0-565e367a25c3 at epoch 1: scheduler
private Pi candidate proposal failed: private Pi candidate adapter
rejected the request`

Skip class: `private_pi_candidate_adapter_rejected_the_request`. Seven
skip rows were present on `daemon.log` at cleanup; the instrument retained
four. Bounded O4 last probe: `runnable_count` 5, `lease_acquired` 0
(`observed_zero` true). O5 named zero. Intent/Effect counts 0.

The instrument's `adapter_or_pi_count` stayed 0, which is a **false
negative**: `/proc` `Name` truncates to 15 characters (`pi-agent-adapte`).
Last probe `kernel_children` listed pid 284943 `pi-agent-adapte` with
ppid 283539 (campaign `kernel-server`). Real `pi-agent-adapter` **did**
spawn. Adapter stderr is `Stdio::null()` in this pin, so the skip row has
no classified Pi/adapter cause.

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or Agent-benefit
claim. Scheduler ticks after the retained sample may have reached the
Provider proxy; this campaign does not claim zero Provider spend. Do not
start additional Provider cells on this freeze.

## C1/C2 paired remainder and later batches (2026-08-18) — `not-run`

| Cell | Disposition | Cause |
|---|---|---|
| B0 remaining C1/C2 classes (C2a/C2b/C2c/C2d) | `not-run` | C1-search stayed `DRAFT` with `lease_acquired` 0 |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| P-arm / broker `48396` | `not-run` | O-arm not fairly measurable |
| O5/O6 as C1/C2 dependents | `not-run` | no Intent/Effect |
| T4–T5/T8/T9 | `not-run` | no public dispatch |
| B3 faults | `not-run` | no frozen fault runner; do not cobble |
| B4 concurrency | `not-run` | B0 path/fairness incomplete |
| B5 1 h / 8 h / 24 h | `not-run` | no frozen soak runner; 1 h trigger not met |
| C0 paired (G1/G2/G3/G4/G6/G9, A1/A4/A5) | `not-run` | broker/runner not qualified; do not cobble a paired shell |

## Cleanup (2026-08-18) — pass

Guest route unchanged: `wuz@192.168.1.2` ProxyJump
`hal9001@192.168.123.160`. `B01-Desktop-Linux-002` snapshot was not
reverted. `B01-Clean-Linux-001` was not contacted.

| Check | Result |
|---|---|
| campaign daemon `127.0.0.1:48296` pid 283539 | product `cognitive daemon stop` `action=stopped`; lock absent; listener absent |
| campaign broker `127.0.0.1:48396` | never started; listener absent |
| listeners `48181` / `48284` / `48383` | untouched (`cos-current` / EVAL-004 residue / EVAL-002 broker) |
| EVAL-004/005/006/007/008 roots | untouched |
| SecretStore item `/18` | `secret-tool clear` on the product attribute triple (`application=cognitiveos-personal`, `provider=deepseek`, `purpose=provider-api-key`); post-clear D-Bus `SearchItems` unlocked/locked 0; `login` collection `Items` 0; never `secret-tool search`/`lookup` |
| redactor `evidence/` | 1 file, `key_shaped_hits=0` |
| redactor `runtime/` text/json | no `api_key=` hits; SQLite WAL contained 11 binary `sk-` substrings (not decoded; not treated as leaked Provider material) |
| campaign root | retained `0700` at `/home/hal9001/e009` (evidence only) |

## Capability matrix (hypothesis / non-claim)

| Class | This freeze | Note |
|---|---|---|
| Public doctor / first conversation | ready | not a C1/C2 Task |
| C1 WorkspaceSearch O-arm | **partial** | admit 200; stayed `DRAFT`; `lease_acquired` 0; real adapter spawned; skip `private_pi_candidate_adapter_rejected_the_request` |
| C2a mutation / C2b Memory-Skill / C2c Effect recovery / C2d verified completion | `not-run` | C1-search path incomplete |
| C1/C2 P-arm | `not-run` | broker `48396` never started |
| C0 paired G/A families | `not-run` | paired runner/broker not qualified |
| B3 / B4 / B5 | `not-run` | no frozen runners; 1 h trigger not met |

## Evidence-ranked optimization priorities (hypothesis only)

1. **Public `cognitive daemon start` + real `pi-agent-adapter`:** skip class
   `private_pi_candidate_adapter_rejected_the_request` is a public
   `daemon.log` fact. Adapter/Pi stderr is discarded (`Stdio::null()`), so
   the skip has no classified cause on this pin.
2. Bind private completion sockets under Linux `UNIX_PATH_MAX` regardless of
   `--runtime-root` length (EVAL-008). Short root is not a product fix.
3. Linux candidate spawn currently forwards a Windows-shaped env allowlist
   (`PATH` typically survives). Capture a redacted adapter/Pi error class
   before guessing Provider spend or extension load.
4. WorkspaceRead is still not advertised as a Pi tool; C1 remains
   WorkspaceSearch-only.

This campaign does not register a `P*-T*` task. Closure does not resume
development.

## Non-claims

No Gate, release, Profile, B01, B01-W, or Agent-benefit promotion. No
optimization success. Never `secret-tool search`/`lookup`. Do not print
Provider keys. Do not treat P2-T32 stub pass, EVAL-008 short-root
workaround, or this adapter-reject cell as C1/C2 Agent-benefit. Do not
reopen EVAL-007 / PR #238.
