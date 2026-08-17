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
| Source archive + SHA-256 | `not-run` | `git archive` of exact `2a8d4d2f` on `DEV-LINUX-NATIVE-01`; scp to guest (do not SSH-pipe) |
| New campaign root/port | `not-run` | `/home/hal9001/perfeval007-20260817` mode `0700`; daemon `127.0.0.1:48292`. Leave `48181`/`48284`/`48383` untouched |
| Exact-source daemon/CLI binaries | `not-run` | `DEV-LINUX-NATIVE-01` `cargo build --release --locked -p kernel-server -p admin-cli -p pi-agent-adapter` from extracted archive |
| Campaign daemon on `48292` | `not-run` | public `cognitive daemon start --bind 127.0.0.1:48292` |
| New SecretStore entry | `not-run` | product stdin import; D-Bus `SearchItems` paths only; never `secret-tool search`/`lookup` |
| Local Pi `0.81.1` pin | `not-run` | guest-local npm pack+install under the new root; `--extension` absolute path only |
| Exact-source `pi-agent-adapter` | `not-run` | same extracted `2a8d4d2f` archive |
| C1/C2 paired B0 | `not-run` | after freeze pass |
| C1/C2 paired B1/B2 | `not-run` | after B0 path/fairness |
| Cleanup / campaign close | `not-run` | stop 48292/48392; clear new SecretStore item without search/lookup; leave 48181/48284/48383 and EVAL-004/005/006 roots untouched |

## Non-claims

Hypothesis only. No Gate, release, Profile, B01, or Agent-benefit promotion.
Evaluation routing is ON only while this campaign row is active.
