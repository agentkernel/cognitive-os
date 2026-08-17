# PERSONAL-PERF-EVAL-008 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-008`
- Lease: `lease/personal/EVAL-008/c1-c2-paired-freeze`
- Date: 2026-08-18
- Frozen product source: `fb85cfff25d8dd9fc5e3a8743ab9fdb3b3586630`
  (P2-T32 public `cognitive daemon start` on Draft PR #239; Ubuntu verify
  `32047487272` pass; linux-002 focused `p2_t32` 1/1). Docs commit
  `a653dc7b` has the same product bytes; freeze source/binaries pin
  `fb85cfff`, not `origin/main` (`2a8d4d2f`). Unmerged freeze is allowed.
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reuse EVAL-004/005/006/007 campaign
roots, loopback ports `48286`/`48288`/`48290`/`48292`/`48386`/`48388`/
`48390`/`48392`, SecretStore items `/12`/`/13`/`/14`/`/15`/`/16`, broker,
runner, corpus, oracle, or evidence denominator. `PERSONAL-PERF-EVAL-007`,
`PERSONAL-PERF-EVAL-006`, `PERSONAL-PERF-EVAL-005`, `PERSONAL-PERF-EVAL-004`,
and `PERSONAL-PERF-EVAL-002` remain closed. Do not reopen
`evaluation/EVAL-007-freeze` or PR #238.

P2-T32 stub Workspace* tests **pass**. That is not a C1/C2 Agent-benefit
and is not “EVAL-007 already fixed.” This campaign measures the public
launcher with a **real** `pi-agent-adapter` (EVAL-007-class freeze, not
the test stub).

## Owner authorization

Owner 2026-08-18: this window is `PERSONAL-PERF-EVAL-008` 真机实测, not
development continuation. Close the P2-T32 lease first (task stays
in-progress pending Windows merge), then freeze and execute remaining
real-machine cells including C1/C2 OS arms per
[personal-performance-benchmark-execution-plan.md](../evaluation/personal-performance-benchmark-execution-plan.md)
§9.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval008-20260818` mode `0700` | `perfeval004`, `perfeval004-20260816`, `perfeval005-20260817`, `perfeval006-20260817`, `perfeval007-20260817`, `~/perfeval002`, `~/p9t04`, `cos-current` |
| Loopback port | `127.0.0.1:48294` daemon; broker `127.0.0.1:48394` (P-arm only after O-arm is fairly measurable) | `48181`, `48282`, `48284`, `48286`, `48288`, `48290`, `48292`, `48383`, `48386`, `48388`, `48390`, `48392` |
| SecretStore entry | new item via product stdin (`cognitive init --api-key-file -`); expected `/17` | `/11`–`/16`; never `secret-tool search`/`lookup` |
| Source pin | `fb85cfff` (P2-T32 public launcher) | EVAL-007 pin `2a8d4d2f`; EVAL-006 pin `103fe776` |

`B01-Clean-Linux-001` stays out of bounds. Snapshot revert/delete, P9-T04
residue, and the owner plaintext key file are not in this freeze's allowlist.
**Rotate the previously leaked Provider key** (EVAL-004 `secret-tool search`
incident) if that item is still in use.

Guest control: `virsh -c qemu:///system` on `hal9000` only. Do not start,
restore, or delete the B01 guest beyond this preregistration. The domain
was observed **running** at campaign start; use it as-is. Do not revert
snapshots.

PowerShell SSH pipes corrupt tar digests; copies use `scp`. Guest `ldd`
on campaign binaries may resolve only glibc/`libgcc`/`libm`.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| P2-T32 lease closed (task not done) | **pass** | Lease archived; task `in-progress` pending Windows required-ci merge of PR #239. Stub pass ≠ C1/C2 |
| Evaluation lease claimed | **pass** | this document + Current snapshot row `PERSONAL-PERF-EVAL-008` **active** |
| Product source pin | **pass** | `fb85cfff25d8dd9fc5e3a8743ab9fdb3b3586630` |
| Source archive + SHA-256 | **pass** | `git archive --format=tar --prefix=cognitiveos-personal-fb85cfff/` of exact `fb85cfff`; 14,653,440 bytes; 1538 entries; 0 `.git/` members; SHA-256 `202384ee0b125c6600764042ddc7a2142bb1502da21be642b8c328440325ced3`. Copied with `scp` |
| New campaign root/port | **pass** | `/home/hal9001/perfeval008-20260818` mode `0700`; daemon `127.0.0.1:48294` pid 281083. Listeners `48181`/`48284`/`48383` untouched |
| Exact-source daemon/CLI binaries | **pass** | `DEV-LINUX-NATIVE-01` `CARGO_NET_OFFLINE=true cargo build --release --locked` in 1m 36s; Rust 1.97.1. `kernel-server` SHA-256 `e603edab9a594e41177f89ac105b2755bff34cdb980c30faece03de87610ec55`; `cognitive` `6917dca3a0f294c34d1f177dd5ebd3e1a36fff1c71de7661094049b30741a65f`; `pi-agent-adapter` `816856b49674d06f025f535fe2bf5219dd9744ab899250a489538ea687aa3167`. `ldd` glibc/`libgcc`/`libm` only |
| Campaign daemon on `48294` | **pass** | public `cognitive daemon start --bind 127.0.0.1:48294`; pid `281083`; start JSON `log_path` `…/runtime/state/cognitiveos/daemon.log` mode `0600` |
| New SecretStore entry | **pass** | product stdin import into **new** item `/org/freedesktop/secrets/collection/login/17`. D-Bus `SearchItems` paths only; never `secret-tool search`/`lookup` |
| Local Pi `0.81.1` pin | **pass** | guest-local npm install of tarball SHA-256 `420113c0282160e6181656fd16cf18742f76bf9040ee3dfb9cb67e3e6ad5641c`; `--extension` absolute. Doctor: package/pinned/observed `0.81.1`, `first_conversation_ready: true` (not C1/C2) |
| Exact-source `pi-agent-adapter` | **pass** | same extracted `fb85cfff` archive; `o-arm-candidate.mjs` SHA-256 `29870821488451b5728f88c4612e1616fd65681adaf23011dd898d459428e573` |
| `cognitive doctor` | **pass** (readiness only) | all required components `ready`; **not** a C1/C2 pass |
| C1/C2 paired B0 | **partial** | one O-arm C1-search sample retained; skip class `private_completion_socket_could_not_be_created`; Task `DRAFT`; `lease_acquired` 0; no Pi child; P-arm not started |
| C1/C2 paired B1/B2 | `not-run` | B0 path/fairness incomplete |
| Cleanup / campaign close | `not-run` | stop `48294`; clear this campaign SecretStore item; leave `48181`/`48284`/`48383` and EVAL-004/005/006/007 roots |

## Unique next action

Cleanup this freeze: `cognitive daemon stop` on `48294`; clear SecretStore
`/17` without search/lookup; redactor 0 key-shaped hits; leave
`48181`/`48284`/`48383` and EVAL-004/005/006/007 roots. Then publish the
final assessment. Do not patch product code in this campaign.

## Non-claims

This campaign creates no Gate, release, Profile, B01, or Agent-benefit
promotion. `retry=0` for Provider cells. Every started sample is retained.
Each finished cell is appended to the running report immediately.
WorkspaceRead is still not advertised as a Pi tool; C1 uses WorkspaceSearch
only (same as EVAL-007).
