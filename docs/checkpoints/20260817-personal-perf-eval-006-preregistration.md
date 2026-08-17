# PERSONAL-PERF-EVAL-006 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-006`
- Lease: `lease/personal/EVAL-006/c1-c2-paired-freeze`
- Date: 2026-08-17
- Frozen product source: `origin/main@103fe776eb7a3aca4d1281aefbda34fdaa445e0b`
  (merge of P2-T30 closure PR #234). Product enablement merged as PR #233 at
  `b13655b9`.
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reuse EVAL-004 campaign roots
`/home/hal9001/perfeval004` or `/home/hal9001/perfeval004-20260816`, EVAL-005
root `/home/hal9001/perfeval005-20260817`, loopback ports
`48286`/`48288`/`48386`/`48388`, SecretStore items `/12`/`/13`/`/14`, broker,
runner, corpus, oracle, or evidence denominator. `PERSONAL-PERF-EVAL-005`,
`PERSONAL-PERF-EVAL-004`, and `PERSONAL-PERF-EVAL-002` remain closed.

## Owner authorization

Owner 2026-08-17: close EVAL-005, deliver the scheduler-lease product mutex,
then re-measure C1/C2 with a new freeze. P2-T30 merged PR #233; lease closed
PR #234 at `main@103fe776`.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval006-20260817` mode `0700` | `perfeval004`, `perfeval004-20260816`, `perfeval005-20260817`, `~/perfeval002`, `~/p9t04`, `cos-current` |
| Loopback port | `127.0.0.1:48290` daemon; broker `127.0.0.1:48390` | `48181`, `48282`, `48284`, `48286`, `48288`, `48383`, `48386`, `48388` |
| SecretStore entry | new item via product stdin (`cognitive init --api-key-file -`) | `/11`, `/12`, `/13`, `/14`; never `secret-tool search`/`lookup` |
| Source pin | `main@103fe776` (P2-T30 closed) | EVAL-005 pin `b16d2955`; EVAL-004 pin `1e71344a` |

`B01-Clean-Linux-001` stays out of bounds. Snapshot revert/delete, P9-T04
residue, and the owner plaintext key file are not in this freeze's allowlist.
**Rotate the previously leaked Provider key** (EVAL-004 `secret-tool search`
incident) if that item is still in use.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| P2-T30 merged and lease closed | **pass** | PR #233 product; PR #234 closure at `main@103fe776` |
| Evaluation lease claimed | **pass** | this document + Current snapshot row |
| Product source pin | **pass** | `103fe776eb7a3aca4d1281aefbda34fdaa445e0b` |
| Source archive + SHA-256 | `not-run` | `git archive` of exact `103fe776` |
| New campaign root/port | `not-run` | `/home/hal9001/perfeval006-20260817`; `127.0.0.1:48290` |
| Exact-source daemon/CLI binaries | `not-run` | `DEV-LINUX-NATIVE-01` release build from extracted archive |
| Campaign daemon on `48290` | `not-run` | public `cognitive daemon start --bind 127.0.0.1:48290` |
| New SecretStore entry | `not-run` | product stdin import; D-Bus paths only; never search/lookup |
| Local Pi `0.81.1` pin | `not-run` | `--extension` absolute path only |
| Exact-source `pi-agent-adapter` | `not-run` | same extracted archive |
| C1/C2 paired B0 | `not-run` | after freeze |
| C1/C2 paired B1/B2 | `not-run` | after B0 |
| Cleanup / campaign close | `not-run` | stop 48290/48390; clear the new SecretStore item without search/lookup; leave 48181/48284/48383 and EVAL-004/005 roots untouched |

## Non-claims

This campaign creates no Gate, release, Profile, B01, or Agent-benefit
promotion. `retry=0` for Provider cells. Every started sample is retained.
Each finished cell is appended to the running report immediately.
