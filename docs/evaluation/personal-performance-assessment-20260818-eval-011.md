# PERSONAL-PERF-EVAL-011 assessment (running)

- Campaign: `PERSONAL-PERF-EVAL-011`
- Frozen source target: `106cfcc06255fe562d455b9a5c1f0862e9994b5a` (`main`
  after P2-T34 merge)
- Lease: `lease/personal/EVAL-011/c1-c2-paired-freeze`
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: campaign **active**. Measurement-only.

This is the campaign's single report (`TEST-REPORT-INCREMENTAL-01`). Append
each finished cell immediately. Do not hold conclusions until batch end.

Owner 2026-08-18 standing continuous delivery after EVAL-010 close and
P2-T32/T33/T34 merge. EVAL-010 remains **closed**. Adapter unit pass is not
C1/C2 Agent-benefit.

## Cells

| Cell | Status | Note |
|---|---|---|
| EVAL-010 remains closed (coordination) | **pass** | do not reuse `48298` / `/19` / `perfeval010-20260818` runtime |
| Evaluation lease claimed | **pass** | claimed 2026-08-18; Current snapshot row `PERSONAL-PERF-EVAL-011` **active** |
| Freeze (archive/binaries/root/port) | **pass** | pin `106cfcc0`; zip SHA-256 `47ba70be6b82fe210a51c5cde4e3d0335b2270723797a38db3d6448eaee28f8d`; root `/home/hal9001/perfeval011-20260818` mode `0700`; daemon `127.0.0.1:48300` pid 291495; `log_path` mode `0600`; listeners `48181`/`48284`/`48383` untouched |
| SecretStore import | **pass** | new item `/org/freedesktop/secrets/collection/login/20` via stdin; `busctl --user tree` paths only; never search/lookup |
| Pi 0.81.1 pin | **pass** | `--extension` absolute; doctor ready is **not** C1/C2 |
| Exact-source `pi-agent-adapter` | **pass** | real adapter; SHA-256 `3e7924deeeca901e21cc4203960125938ab76688e89a52f4abe600ea0fbfd6cd` |
| B0 C1 WorkspaceSearch O-arm | **partial** (sample `b0-0` retained) | admit 200; lifecycle `DRAFT`; `lease_acquired` 0; skip class `private_pi_candidate_adapter_rejected_the_request`; cause: freeze copy missed `private_candidate_provider.mjs`; short sockets `pc-291495-*.sock`; real adapter spawned |
| B0 remaining C1/C2 families | `not-run` | after C1-search leaves `DRAFT` with `lease_acquired` ≥ 1 |
| B0 P-arm / broker `48400` | `not-run` | after O-arm is fairly measurable |
| B1/B2 C1/C2 paired | `not-run` | after B0 path/fairness |
| Cleanup | `not-run` | stop `48300`; clear SecretStore; leave `48181`/`48284`/`48383` and prior EVAL roots |

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
