# PERSONAL-PERF-EVAL-011 assessment (final)

- Campaign: `PERSONAL-PERF-EVAL-011`
- Frozen source target: `106cfcc06255fe562d455b9a5c1f0862e9994b5a` (`main`
  after P2-T34 merge)
- Lease: `lease/personal/EVAL-011/c1-c2-paired-freeze` (closed 2026-08-18)
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: campaign **closed**. Measurement-only. Evaluation routing OFF.

This is the campaign's single report (`TEST-REPORT-INCREMENTAL-01`). Cells
were appended as they finished. The campaign is closed; do not append more
cells on this freeze.

Owner 2026-08-18 standing continuous delivery after EVAL-010 close and
P2-T32/T33/T34 merge. EVAL-010 remains **closed**. Adapter unit pass is not
C1/C2 Agent-benefit.

## Cells

| Cell | Status | Note |
|---|---|---|
| EVAL-010 remains closed (coordination) | **pass** | do not reuse `48298` / `/19` / `perfeval010-20260818` runtime |
| Evaluation lease claimed | **pass** | claimed then closed 2026-08-18; Current snapshot row `PERSONAL-PERF-EVAL-011` **closed**; routing OFF |
| Freeze (archive/binaries/root/port) | **pass** | pin `106cfcc0`; zip SHA-256 `47ba70be6b82fe210a51c5cde4e3d0335b2270723797a38db3d6448eaee28f8d`; root `/home/hal9001/perfeval011-20260818` mode `0700`; daemon `127.0.0.1:48300` pid 291495; `log_path` mode `0600`; listeners `48181`/`48284`/`48383` untouched |
| SecretStore import | **pass** | new item `/org/freedesktop/secrets/collection/login/20` via stdin; `busctl --user tree` paths only; never search/lookup |
| Pi 0.81.1 pin | **pass** | `--extension` absolute; doctor ready is **not** C1/C2 |
| Exact-source `pi-agent-adapter` | **pass** | real adapter; SHA-256 `3e7924deeeca901e21cc4203960125938ab76688e89a52f4abe600ea0fbfd6cd` |
| B0 C1 WorkspaceSearch O-arm | **partial** (samples `b0-0` and `b0-1` retained) | both admit 200; both stayed `DRAFT`; `lease_acquired` 0; skip class `private_pi_candidate_adapter_rejected_the_request`. `b0-0`: missing `private_candidate_provider.mjs`. `b0-1`: companion copied; adapter parse `missing field parameters_digest` |
| B0 remaining C1/C2 families | `not-run` | C1-search stayed `DRAFT` with `lease_acquired` 0 |
| B0 P-arm / broker `48400` | `not-run` | O-arm path not fairly measurable |
| B1/B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| Cleanup | **pass** | daemon 48300 stopped; broker 48400 absent; SecretStore `/20` cleared; 48181/48284/48383 and prior EVAL roots untouched |

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or Agent-benefit
claim.

## Freeze (2026-08-18) — pass

Exact source `106cfcc06255fe562d455b9a5c1f0862e9994b5a` (`main` after P2-T34).
Guest root mode `0700`. Listeners `48181` / `48284` / `48383` untouched.
SecretStore item `/20` is new (not `/12`–`/19`). Public doctor was ready
(`first_conversation_ready: true`). That is conversation readiness, not a
C1/C2 Task. Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or
Agent-benefit claim.

Host `DEV-LINUX-NATIVE-01` built release binaries from the extracted
`106cfcc0` zip with a dedicated `CARGO_TARGET_DIR`
(`/home/wuz/agent-kernel-worktrees/eval011-106cfcc0/target`,
`CARGO_NET_OFFLINE=true`, 1m 44s, Rust 1.97.1). Windows GNU Rust build
remains `not-run` (`RUST-LINK-DEV-WIN-GNU-01`). Copies used `scp`.

Public start: endpoint `127.0.0.1:48300`, pid `291495`, `log_path`
`/home/hal9001/perfeval011-20260818/runtime/state/cognitiveos/daemon.log`
(mode `0600`). Nested `completion.sock` under this long root would exceed
Linux `UNIX_PATH_MAX` (108); P2-T33 binds under `$XDG_RUNTIME_DIR`.
EVAL-010 leftover `pc-287493-8.sock` in `/run/user/1000/cognitiveos` is
not this campaign's socket.

| Asset | Value |
|---|---|
| source zip | 4,589,620 bytes; SHA-256 `47ba70be6b82fe210a51c5cde4e3d0335b2270723797a38db3d6448eaee28f8d` |
| `kernel-server` | SHA-256 `24b78883500e4c75cddb59c98e89c457f9f9da63e3302fec34862382e4887330` |
| `cognitive` | SHA-256 `04ba65b1ffcd4a50cfcff4b6d3e857b7e6f9c4694f428a78df32d81e4f77a0e7` |
| `pi-agent-adapter` | SHA-256 `3e7924deeeca901e21cc4203960125938ab76688e89a52f4abe600ea0fbfd6cd` |
| `o-arm-candidate.mjs` | SHA-256 `29870821488451b5728f88c4612e1616fd65681adaf23011dd898d459428e573` |
| `private_candidate_provider.mjs` | copied after `b0-0` from closed EVAL-010; SHA-256 `2b7e52a6afe205e5997c58fe59b096fc7666dfd8733e196777e915d3a0bc245b` (absent at `b0-0` start) |

SecretStore import (stdin; `busctl --user tree` paths only): item
`/org/freedesktop/secrets/collection/login/20`. Never `secret-tool
search`/`lookup`. Doctor after configure: provider
`secret_ref_resolves=true`, Pi package/pinned/observed `0.81.1`,
`first_conversation_ready: true`. Daemon pid 291495 still bound to
`127.0.0.1:48300`.

No Gate, release, Profile, B01, or Agent-benefit claim.

## B0 C1-search O-arm sample `b0-0` (2026-08-18) — partial; retained

One O-arm C1-search qualification Task was started with `retry=0` against
the public Task admit surface on the public `cognitive daemon start`
launcher. It is retained. It did not leave `DRAFT`. No Intent, Effect,
verification, or acceptance row exists. WorkspaceRead is still not
advertised; this cell used WorkspaceSearch only.

P2-T33 UNIX_PATH_MAX product fix: **confirmed on this long root.** During
the probe, `$XDG_RUNTIME_DIR/cognitiveos/pc-291495-*.sock` existed (48
names `pc-291495-0.sock` … `pc-291495-47.sock`). Nested `completion.sock`
under `/home/hal9001/perfeval011-20260818` was not required. This is
therefore **not** EVAL-008's `private_completion_socket_could_not_be_created`.

P2-T34 digest skip: **not this sample's skip.** The real adapter spawned
and Pi exited while loading the campaign O-arm extension, before a
structurally parsed candidate reached `validate_untrusted_pi_candidate`.

| Seed | Task ref | Admit | Probe | Lifecycle |
|---|---|---|---|---|
| `b0-0` | `task://local/eval011-b0-C1-search-b0-0-42371727ec06` | 200 (record 14 ms, interpret 6 ms, preview 1 ms, admit 30 ms) | wall 180002 ms; 88 probes; `acceptance_ref` absent | `DRAFT`; `contract_epoch` 1 |

Guest evidence file:
`/home/hal9001/perfeval011-20260818/evidence/b0-oarm-C1-search-b0-0.json`
SHA-256 `591aeb947f18aa849057f647bc09ccef3d2a50b938225d69792889366bec8ce8`.
Instrument `eval011-b0-c1-search.py`
SHA-256 `b609be34a80c6c5c8c68132d53417f6e84f11e18efaf9e7a0b34fbe155d7c5d4`.
Public `cognitive`/`GET /task/evidence`: `lifecycle.current_state=DRAFT`;
`intent_refs` / `effect_refs` empty; `latest_verification` /
`latest_acceptance` null. Bounded O4 last probe: `runnable_count` 32,
`lease_acquired` 0 (`observed_zero` true). O5 named zero.

Public skip rows on `daemon.log` (mode `0600`):

`kernel-server personal scheduler tick: skip row
task://local/eval011-b0-C1-search-b0-0-42371727ec06 at epoch 1: scheduler
private Pi candidate proposal failed: private Pi candidate adapter
rejected the request (daemon candidate Pi exited unsuccessfully: Error:
Failed to load extension
"/home/hal9001/perfeval011-20260818/o-arm-candidate.mjs": Failed to load
extension: Cannot find module './private_candidate_provider.mjs' … Error:
Unknown provider "cognitiveos-private-candidate". Use --list-models )`

Skip class: `private_pi_candidate_adapter_rejected_the_request`.
`o-arm-candidate.mjs` (SHA-256 `29870821…`, same bytes as EVAL-010)
`import`s `./private_candidate_provider.mjs`. That companion file was
present on the closed EVAL-010 root (`SHA-256
2b7e52a6afe205e5997c58fe59b096fc7666dfd8733e196777e915d3a0bc245b`) and
**absent** from `/home/hal9001/perfeval011-20260818` at sample start.
This is a freeze-copy gap, not a product digest failure and not a
product-code change.

Last probe `kernel_children` listed pid 294170 `pi-agent-adapte` (ppid
291495) and pid 294181 `node` (ppid 294170, cmdline matched
`pi-coding-agent`). Real `pi-agent-adapter` **did** spawn.
`adapter_or_pi_seen` true (`/proc` `Name` truncation accounted for).

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or
Agent-benefit claim. Scheduler ticks after the retained sample may have
reached the Provider proxy; this campaign does not claim zero Provider
spend. Sample `b0-0` is not retried (`retry=0`). Unique next: copy the
missing campaign companion module onto this root (same EVAL-010 digest)
and start a **new** C1-search sample `b0-1`. That is freeze-asset
completion, not a product edit and not a retry of `b0-0`.

## B0 C1-search O-arm sample `b0-1` (2026-08-18) — partial; retained

Companion `private_candidate_provider.mjs` was copied from the closed
EVAL-010 root onto this campaign root (SHA-256
`2b7e52a6afe205e5997c58fe59b096fc7666dfd8733e196777e915d3a0bc245b`,
equal to EVAL-010). `pi-cognitiveos/dist/workspace-tools.js` was already
present. A **new** C1-search Task was started with `retry=0`. It is not a
retry of `b0-0`. It is retained. It did not leave `DRAFT`. No Intent,
Effect, verification, or acceptance row exists.

| Seed | Task ref | Admit | Probe | Lifecycle |
|---|---|---|---|---|
| `b0-1` | `task://local/eval011-b0-C1-search-b0-1-56a50515c5f8` | 200 (record 7 ms, interpret 5 ms, preview 1 ms, admit 14 ms) | wall 178644 ms; 87 probes; `acceptance_ref` absent | `DRAFT`; `contract_epoch` 1 |

Guest evidence file:
`/home/hal9001/perfeval011-20260818/evidence/b0-oarm-C1-search-b0-1.json`
SHA-256 `31c33a552b6b0838b4ca4e1bce03332f48abf93407013283e02a3303281564ee`.
Instrument `eval011-b0-c1-search.py`
SHA-256 `4cffeba1445b750f790fd9be805da41426ca337e8cf4187926cab9eae173adb9`.
Public `GET /task/evidence`: `lifecycle.current_state=DRAFT`; `intent_refs`
/ `effect_refs` empty; `latest_verification` / `latest_acceptance` null.
Bounded O4 last instrument probe: `runnable_count` 2, `lease_acquired` 0
(`observed_zero` true). A 90 s follow-up observation of the **same**
`task_ref` (no new admit) stayed `DRAFT` with `lease_acquired` 0 and
`runnable_count` 3→4. O5 named zero.

Short completion sockets for this daemon pid existed
(`pc-291495-126.sock` … `pc-291495-129.sock` during the instrument;
later `pc-291495-131.sock` / `pc-291495-135.sock`). Real
`pi-agent-adapter daemon-candidate` spawned (`/proc` `Name`
`pi-agent-adapte`). `adapter_or_pi_seen` true.

Public skip rows on `daemon.log` (mode `0600`) that name this `task_ref`
(4 rows; earlier `b0-0` skip flood is a different sample):

Skip class: `private_pi_candidate_adapter_rejected_the_request`.

First three named rows: `daemon candidate Pi exited unsuccessfully` plus
the adapter clap usage synopsis (stderr mixed into the skip text). The
fourth named row also contains:

`Pi candidate final response is invalid: daemon candidate response is
invalid: missing field \`parameters_digest\` at line 1 column 118`

That is serde parse of the JSON-fallback candidate **before** P2-T34
digest recomputation. P2-T34 recomputes `parameters_digest` when the
field is present (including empty/garbage). Live Pi JSON omitted the
key. No `candidate has missing fields or an invalid parameters digest`
scheduler class appears on this freeze (EVAL-010's skip). `invalid
parameters digest` count on `daemon.log` is 0.

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or
Agent-benefit claim. Scheduler ticks may have reached the Provider
proxy; this campaign does not claim zero Provider spend. Do not start
additional Provider cells on this freeze.

## C1/C2 paired remainder and later batches (2026-08-18) — `not-run`

| Cell | Disposition | Cause |
|---|---|---|
| B0 remaining C1/C2 classes (C2a/C2b/C2c/C2d) | `not-run` | C1-search stayed `DRAFT` with `lease_acquired` 0 |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| P-arm / broker `48400` | `not-run` | O-arm not fairly measurable |
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
| campaign daemon `127.0.0.1:48300` pid 291495 | product `cognitive daemon stop` `action=stopped` (`stale_lock_removed=true`); lock absent; listener absent |
| campaign broker `127.0.0.1:48400` | never started; listener absent |
| listeners `48181` / `48284` / `48383` | untouched (`cos-current` / EVAL-004 residue / EVAL-002 broker) |
| EVAL-004/005/006/007/008/009/010 roots | untouched |
| SecretStore item `/20` | `secret-tool clear` on the product attribute triple (`application=cognitiveos-personal`, `provider=deepseek`, `purpose=provider-api-key`); post-clear D-Bus `SearchItems` unlocked/locked 0; never `secret-tool search`/`lookup` |
| redactor `evidence/` | 2 files, `key_shaped_hits=0` |
| redactor `runtime/` | 17 files, `key_shaped_hits=25` (same class as EVAL-009/010: binary `sk-` substrings in SQLite/WAL; not decoded; not treated as leaked Provider material) |
| campaign root | retained `0700` at `/home/hal9001/perfeval011-20260818` (evidence only) |

## Capability matrix (hypothesis / non-claim)

| Class | This freeze | Note |
|---|---|---|
| Public doctor / first conversation | ready | not a C1/C2 Task |
| Private completion socket on a long root | **pass** (host path) | `$XDG_RUNTIME_DIR/cognitiveos/pc-291495-*.sock`; not EVAL-008's skip |
| C1 WorkspaceSearch O-arm `b0-0` | **partial** | admit 200; stayed `DRAFT`; `lease_acquired` 0; missing campaign companion mjs |
| C1 WorkspaceSearch O-arm `b0-1` | **partial** | admit 200; stayed `DRAFT`; `lease_acquired` 0; real adapter spawned; skip `private_pi_candidate_adapter_rejected_the_request` (`missing field parameters_digest`) |
| C2a mutation / C2b Memory-Skill / C2c Effect recovery / C2d verified completion | `not-run` | same DRAFT/`lease_acquired` 0; do not open more Provider spend |
| C1/C2 P-arm | `not-run` | broker `48400` never started |
| C0 paired G/A families | `not-run` | paired runner/broker not qualified |
| B3 / B4 / B5 | `not-run` | no frozen runners; 1 h trigger not met |

## Evidence-ranked optimization priorities (hypothesis only)

1. **JSON-fallback `parameters_digest` key omitted:** live Pi produced a candidate object without the `parameters_digest` field. Adapter serde fails with `missing field parameters_digest` before P2-T34 recomputation. P2-T34 unit tests cover empty/garbage values when the field is **present**. Defaulting the missing field (then recomputing from `parameters`) is the next product follow-up, on a new task/freeze.
2. Keep `/proc` `Name` 15-character truncation in mind (`pi-agent-adapte`). This freeze's instrument matched the truncated name.
3. O-arm `o-arm-candidate.mjs` requires sibling `private_candidate_provider.mjs`. A freeze that copies only the wrapper will skip at extension load (`b0-0`).
4. WorkspaceRead is still not advertised as a Pi tool; C1 remains WorkspaceSearch-only until a later freeze says otherwise.

This campaign does not register a `P*-T*` task. Closure does not resume
development by itself.

## Non-claims

No Gate, release, Profile, B01, B01-W, or Agent-benefit promotion. No
optimization success. Never `secret-tool search`/`lookup`. Do not print
Provider keys. Do not treat P2-T34 adapter unit pass, UNIX_PATH_MAX bind,
companion-mjs copy, or adapter spawn as C1/C2 Agent benefit.

**Campaign closed.** Evaluation routing is OFF. **Rotate the Provider key**
leaked by the earlier EVAL-004 `secret-tool search` incident if that
material is still in use.
