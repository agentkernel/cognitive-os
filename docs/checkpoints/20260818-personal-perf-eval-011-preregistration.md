# PERSONAL-PERF-EVAL-011 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-011`
- Lease: `lease/personal/EVAL-011/c1-c2-paired-freeze`
- Date: 2026-08-18
- Frozen product source: `106cfcc06255fe562d455b9a5c1f0862e9994b5a`
  (`main` after P2-T34 PR [#241](https://github.com/agentkernel/cognitive-os/pull/241)).
  Product bytes for the adapter digest fix land at `a60ceed5`; the freeze pin
  is the merged `main` revision, not EVAL-010 pin `289eebad`.
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted on this freeze (measurement-only)

This is a **new freeze**. It does not reopen EVAL-010 / EVAL-009 / EVAL-008 /
EVAL-007. It does not reuse EVAL-004/005/006/007/008/009/010 campaign roots,
loopback ports `48286`/`48288`/`48290`/`48292`/`48294`/`48296`/`48298`/
`48386`/`48388`/`48390`/`48392`/`48394`/`48396`/`48398`, SecretStore items
`/12`–`/19`, broker, runner, corpus, oracle, or evidence denominator.

Owner 2026-08-18 granted standing continuous delivery after EVAL-010 close.
P2-T32/T33/T34 merged to `main`. This freeze measures the P2-T34 JSON-fallback
digest canonicalization with the real `pi-agent-adapter` on a long unique
root. Adapter unit pass is not C1/C2 Agent-benefit.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval011-20260818` mode `0700` (long; same UNIX_PATH_MAX class as EVAL-010) | `perfeval010-20260818`, `e009`, `perfeval004`, `perfeval004-20260816`, `perfeval005-20260817`, `perfeval006-20260817`, `perfeval007-20260817`, `perfeval008-20260818`, `~/perfeval002`, `~/p9t04`, `cos-current` |
| Loopback port | `127.0.0.1:48300` daemon; broker `127.0.0.1:48400` (P-arm only after O-arm is fairly measurable) | `48181`, `48282`, `48284`, `48286`, `48288`, `48290`, `48292`, `48294`, `48296`, `48298`, `48383`, `48386`, `48388`, `48390`, `48392`, `48394`, `48396`, `48398` |
| SecretStore entry | new item via product stdin (`cognitive init --api-key-file -`); expected `/20` or next unused path | `/11`–`/19`; never `secret-tool search`/`lookup` |
| Source pin | `106cfcc0` (merged P2-T34) | EVAL-010 pin `289eebad`; EVAL-009/008 pin `fb85cfff`; EVAL-007 pin `2a8d4d2f` |

`B01-Clean-Linux-001` stays out of bounds. Snapshot revert/delete, P9-T04
residue, and the owner plaintext key file are not in this freeze's allowlist.
**Rotate the previously leaked Provider key** (EVAL-004 `secret-tool search`
incident) if that item is still in use.

Guest control: `virsh -c qemu:///system` on `hal9000` only. Do not start,
restore, or delete the B01 guest beyond this preregistration. The domain is
used as-is. Do not revert snapshots.

PowerShell SSH pipes corrupt tar digests; copies use `scp`.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| EVAL-010 remains closed | **pass** | do not reopen; do not reuse `/19` / `48298` / `perfeval010-20260818` runtime |
| Evaluation lease claimed | **pass** | claimed 2026-08-18; Current snapshot row `PERSONAL-PERF-EVAL-011` **active** |
| Product source pin | **pass** | `106cfcc06255fe562d455b9a5c1f0862e9994b5a` |
| Source archive + SHA-256 | **pass** | `git archive --format=zip` of exact `106cfcc0`; 4,589,620 bytes; SHA-256 `47ba70be6b82fe210a51c5cde4e3d0335b2270723797a38db3d6448eaee28f8d`. Copied with `scp` (PowerShell pipes corrupt tar) |
| New campaign root/port | **pass** | `/home/hal9001/perfeval011-20260818` mode `0700`; daemon `127.0.0.1:48300` pid 291495. Listeners `48181`/`48284`/`48383` untouched |
| Exact-source daemon/CLI binaries | **pass** | `DEV-LINUX-NATIVE-01` `CARGO_NET_OFFLINE=true` dedicated `CARGO_TARGET_DIR` `cargo build --release --locked` in 1m 44s; Rust 1.97.1. `kernel-server` SHA-256 `24b78883500e4c75cddb59c98e89c457f9f9da63e3302fec34862382e4887330`; `cognitive` `04ba65b1ffcd4a50cfcff4b6d3e857b7e6f9c4694f428a78df32d81e4f77a0e7`; `pi-agent-adapter` `3e7924deeeca901e21cc4203960125938ab76688e89a52f4abe600ea0fbfd6cd`. `ldd` glibc/`libgcc`/`libm` only |
| Campaign daemon on `48300` | **pass** | public `cognitive daemon start --bind 127.0.0.1:48300`; pid `291495`; start JSON `log_path` `…/runtime/state/cognitiveos/daemon.log` mode `0600` |
| New SecretStore entry | **pass** | product stdin import into **new** item `/org/freedesktop/secrets/collection/login/20`. `busctl --user tree` paths only; never `secret-tool search`/`lookup` |
| Local Pi `0.81.1` pin | **pass** | `--extension` absolute; package/pinned/observed `0.81.1` |
| Exact-source `pi-agent-adapter` | **pass** | real adapter, not a stub; SHA-256 `3e7924deeeca901e21cc4203960125938ab76688e89a52f4abe600ea0fbfd6cd`; `o-arm-candidate.mjs` `29870821488451b5728f88c4612e1616fd65681adaf23011dd898d459428e573` |
| `cognitive doctor` | **pass** (readiness only) | all required components `ready`; Pi `0.81.1`; `first_conversation_ready: true`; **not** a C1/C2 pass |
| C1/C2 paired B0 | **partial** | samples `b0-0` and `b0-1` retained; skip class `private_pi_candidate_adapter_rejected_the_request`; Task `DRAFT`; `lease_acquired` 0; short sockets created; real adapter spawned; `b0-1` adapter parse `missing field parameters_digest`; P-arm not started |
| C1/C2 paired B1/B2 | `not-run` | B0 path/fairness incomplete |
| Cleanup / campaign close | **pass** | stop `48300`; SecretStore `/20` cleared without search/lookup; leave `48181`/`48284`/`48383` and prior EVAL roots |

## Unique next action

None for this campaign — closed. Do not reopen EVAL-011. Product follow-up
for omitted JSON-fallback `parameters_digest` requires a new task and a new
freeze. Doctor ready ≠ C1/C2.

Claim ceiling `hypothesis`; `not_reviewed`. No Gate, release, Profile, B01,
or Agent-benefit promotion.
