# PERSONAL-PERF-EVAL-007 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-007`
- Lease: `lease/personal/EVAL-007/c1-c2-paired-freeze`
- Date: 2026-08-17
- Frozen product source: `origin/main@2a8d4d2f9944417c8081edede2f1fd04caa5379d`
  (merge of P2-T31 closure PR #237). Product enablement merged as PR #236 at
  `da6fca2e` (product SHA `49cdfc0c`).
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reuse EVAL-004/005/006 campaign roots,
loopback ports `48286`/`48288`/`48290`/`48386`/`48388`/`48390`, SecretStore
items `/12`/`/13`/`/14`/`/15`, broker, runner, corpus, oracle, or evidence
denominator. `PERSONAL-PERF-EVAL-006`, `PERSONAL-PERF-EVAL-005`,
`PERSONAL-PERF-EVAL-004`, and `PERSONAL-PERF-EVAL-002` remain closed.

## Owner authorization

Owner 2026-08-17: close EVAL-005, deliver the live-daemon scheduler-lease
mutex, then re-measure C1/C2 with a new freeze. P2-T31 merged PR #236;
lease closed PR #237 at `main@2a8d4d2f`.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval007-20260817` mode `0700` | `perfeval004`, `perfeval004-20260816`, `perfeval005-20260817`, `perfeval006-20260817`, `~/perfeval002`, `~/p9t04`, `cos-current` |
| Loopback port | `127.0.0.1:48292` daemon; broker `127.0.0.1:48392` | `48181`, `48282`, `48284`, `48286`, `48288`, `48290`, `48383`, `48386`, `48388`, `48390` |
| SecretStore entry | new item via product stdin (`cognitive init --api-key-file -`) | `/11`, `/12`, `/13`, `/14`, `/15`; never `secret-tool search`/`lookup` |
| Source pin | `main@2a8d4d2f` (P2-T31 closed) | EVAL-006 pin `103fe776`; EVAL-005 pin `b16d2955`; EVAL-004 pin `1e71344a` |

`B01-Clean-Linux-001` stays out of bounds. Snapshot revert/delete, P9-T04
residue, and the owner plaintext key file are not in this freeze's allowlist.
**Rotate the previously leaked Provider key** (EVAL-004 `secret-tool search`
incident) if that item is still in use.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| P2-T31 merged and lease closed | **pass** | PR #236 product; PR #237 closure at `main@2a8d4d2f` |
| Evaluation lease claimed | **pass** | this document + Current snapshot row |
| Product source pin | **pass** | `2a8d4d2f9944417c8081edede2f1fd04caa5379d` |
| Source archive + SHA-256 | **pass** | `git archive --format=tar --prefix=cognitiveos-personal-2a8d4d2f/` of exact `2a8d4d2f`; 14,622,720 bytes; 1536 entries; 0 `.git/` members; SHA-256 `ca2a95b09a78062cc55112211dac2d5de192aa3e353dafbbdd0572bcb4e1efed`. Copied with `scp` (not SSH-pipe) |
| New campaign root/port | **pass** | `/home/hal9001/perfeval007-20260817` mode `0700`; daemon `127.0.0.1:48292` pid 277358. Listeners `48181`/`48284`/`48383` untouched |
| Exact-source daemon/CLI binaries | **pass** | `DEV-LINUX-NATIVE-01` `CARGO_NET_OFFLINE=true cargo build --release --locked -p kernel-server -p admin-cli -p pi-agent-adapter` from extracted archive; Rust 1.97.1. `kernel-server` SHA-256 `e603edab9a594e41177f89ac105b2755bff34cdb980c30faece03de87610ec55`; `cognitive` `0c443a5c56c55efdd92927d973d4acf9f00ad8d0007f51eca7fc2386baa713f2`; `pi-agent-adapter` `816856b49674d06f025f535fe2bf5219dd9744ab899250a489538ea687aa3167` |
| Campaign daemon on `48292` | **pass** | public `cognitive daemon start --bind 127.0.0.1:48292`; pid `277358`; lock `…/runtime/cognitiveos/daemon.lock` |
| New SecretStore entry | **pass** | product stdin import into **new** item `/org/freedesktop/secrets/collection/login/16` (not `/12`/`/13`/`/14`/`/15`). D-Bus `SearchItems` paths only; never `secret-tool search`/`lookup` |
| Local Pi `0.81.1` pin | **pass** | guest-local npm install of tarball SHA-256 `420113c0282160e6181656fd16cf18742f76bf9040ee3dfb9cb67e3e6ad5641c` under the new root; `--extension` absolute. Doctor: package/pinned/observed `0.81.1`, `first_conversation_ready: true` |
| Exact-source `pi-agent-adapter` | **pass** | same extracted `2a8d4d2f` archive; host release build copied to guest. `o-arm-candidate.mjs` SHA-256 `29870821488451b5728f88c4612e1616fd65681adaf23011dd898d459428e573` |
| C1/C2 paired B0 | **partial** | one O-arm C1-search sample retained; public lifecycle `DRAFT`; `lease_acquired` 0; no Pi child; P-arm not started. See running report |
| C1/C2 paired B1/B2 | `not-run` | B0 path/fairness incomplete |
| Cleanup / campaign close | **pass** | daemon 48292 stopped; broker 48392 absent; SecretStore `/16` cleared without search/lookup; leave 48181/48284/48383 and EVAL-004/005/006 roots untouched; redactor evidence 3/0 runtime 15/0 |

## Unique next action

**Campaign closed 2026-08-17** after B0 path/fairness failed on the public
`cognitive daemon start` launcher. Retain the started C1-search sample.
Do not reopen B1/B2 on this freeze. Do not reuse EVAL-004, EVAL-005,
EVAL-006, or EVAL-007 roots/ports or SecretStore `/12`/`/13`/`/14`/`/15`/`/16`.
Never `secret-tool search`/`lookup`. Evaluation routing is OFF. The public
daemon-launcher skip is a product mutex for a new formal P2 task after this
close (do not collide with P2-T29/T30/T31). Rotate the Provider key exposed
earlier by EVAL-004 `secret-tool search`.

## Non-claims

Hypothesis only. No Gate, release, Profile, B01, or Agent-benefit promotion.
Evaluation routing is ON only while this campaign row is active.
