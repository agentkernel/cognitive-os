# PERSONAL-PERF-EVAL-005 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-005`
- Lease: `lease/personal/EVAL-005/c1-c2-paired-freeze`
- Date: 2026-08-17
- Frozen product source: `origin/main@b16d29556eb4113ead3661f186e615c3183962a9`
  (merge of P2-T29 closure PR #231). Product enablement merged as PR #230 at
  `98cb23d1`.
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reuse EVAL-004 campaign root
`/home/hal9001/perfeval004` or `/home/hal9001/perfeval004-20260816`, loopback
ports `48286`/`48386`, SecretStore items `/12`/`/13`, broker, runner, corpus,
oracle, or evidence denominator. `PERSONAL-PERF-EVAL-004` and
`PERSONAL-PERF-EVAL-002` remain closed.

## Owner authorization

Owner 2026-08-17: after EVAL-004 closure, deliver P2-T29 (C1/C2 product
mutexes) then measure with a new freeze. P2-T29 merged PR #230; lease closed
PR #231 at `main@b16d2955`.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval005-20260817` mode `0700` | `perfeval004`, `perfeval004-20260816`, `~/perfeval002`, `~/p9t04`, `cos-current` |
| Loopback port | `127.0.0.1:48288` daemon; broker `127.0.0.1:48388` | `48181`, `48282`, `48284`, `48286`, `48383`, `48386` |
| SecretStore entry | new item via product stdin (`cognitive init --api-key-file -`) | `/11`, `/12`, `/13`; never `secret-tool search`/`lookup` |
| Source pin | `main@b16d2955` (P2-T29 closed) | EVAL-004 pin `1e71344a` |

`B01-Clean-Linux-001` stays out of bounds. Snapshot revert/delete, P9-T04
residue, and the owner plaintext key file are not in this freeze's allowlist.
Rotate the previously leaked Provider key if that item is still in use.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| P2-T29 merged and lease closed | **pass** | PR #230 product; PR #231 closure at `main@b16d2955` |
| Evaluation lease claimed | **pass** | this document + Current snapshot row |
| Product source pin | **pass** | `b16d29556eb4113ead3661f186e615c3183962a9` |
| Source archive + SHA-256 | **not-run** | `git archive` of the pinned SHA |
| New campaign root/port | **not-run** | `/home/hal9001/perfeval005-20260817`; `48288` |
| Exact-source daemon/CLI binaries | **not-run** | `DEV-LINUX-NATIVE-01` release build from the archive |
| Campaign daemon on `48288` | **not-run** | |
| New SecretStore entry | **not-run** | product stdin from guest `~/下载/deepseek.txt`; no search/lookup |
| Local Pi `0.81.1` pin | **not-run** | `--extension` absolute path only |
| C1/C2 paired B0 | **not-run** | first measurement cell after freeze |

## Non-claims

This campaign creates no Gate, release, Profile, B01, or Agent-benefit
promotion. `retry=0` for Provider cells. Every started sample is retained.
Each finished cell is appended to the running report immediately.
