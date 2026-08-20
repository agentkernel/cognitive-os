# PERSONAL-PERF-EVAL-012 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-012`
- Lease: `lease/personal/EVAL-012/c1-c2-paired-b0`
- Date: 2026-08-20
- Frozen product source: `370b26fcc05976c7c1c97e5510a99ed3ebc23f2c`
  (pushed `origin/main` after P9-T08 merge / PR [#247](https://github.com/agentkernel/cognitive-os/pull/247)).
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reopen EVAL-002 or EVAL-004 through
EVAL-011. It does not reuse those campaign roots, loopback ports
`48286`–`48298` / `48386`–`48398` / `48383`, SecretStore items `/12`–`/19`,
P2-T37 roots `p2-t37-c2a-write-20260820` / `p2-t37-c2a-patch-20260820`,
broker, runner, corpus, oracle, or evidence denominator.

Owner 2026-08-20 activated this campaign (“激活”). Snapshot restore/delete
and P9-T04 residue remain outside the allowlist. The owner plaintext key
file is not an allowed bind path.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval012-20260820` mode `0700` | `perfeval004`…`perfeval011`, `e009`, `~/perfeval002`, `~/p9t04`, `cos-current`, P2-T37 roots |
| Loopback port | daemon `127.0.0.1:48300`; broker `127.0.0.1:48400` | `48181`, `48282`, `48284`, `48286`–`48298`, `48383`, `48386`–`48398` |
| SecretStore entry | planned `/20` via `--reuse-existing-secret-binding` only | `/12`–`/19`; never `secret-tool search`/`lookup`; never keyfile copy |
| Source pin | `370b26fc` | closed-EVAL pins; unmerged task branches |

`B01-Clean-Linux-001` stays out of bounds. Guest control:
`virsh -c qemu:///system` on `hal9000` only. The domain is used as-is. Do
not start, restore, or delete snapshots.

PowerShell SSH pipes corrupt tar digests; copies use `scp`.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| EVAL-002 and EVAL-004–011 remain closed | **pass** | do not reopen; do not reuse their roots/ports/SecretStore items |
| Owner activation | **pass** | Current snapshot names `PERSONAL-PERF-EVAL-012` active |
| Evaluation lease claimed | **pass** | `lease/personal/EVAL-012/c1-c2-paired-b0` |
| Product source pin | **pass** (docs) | `370b26fcc05976c7c1c97e5510a99ed3ebc23f2c`; guest archive not yet copied |
| Source archive + SHA-256 | `not-run` | `git archive` of exact `370b26fc` on Linux; copy with `scp` |
| Guest identity | `not-run` | confirm `B01-Desktop-Linux-002` before product install |
| New campaign root/port | `not-run` | `/home/hal9001/perfeval012-20260820`; `48300`/`48400` unused |
| Exact-source daemon/CLI/adapter binaries | `not-run` | `DEV-LINUX-NATIVE-01` release build from the archive |
| Secret bind | `not-run` | `--reuse-existing-secret-binding` only; live doctor is E9 |
| Local Pi `0.81.1` pin | `not-run` | `--extension` absolute; doctor ready is not C1/C2 |
| P-arm broker `48400` | `not-run` | after O-arm bind; loopback-only |
| `cognitive doctor` | `not-run` | readiness only; required before first B0 sample |
| C1/C2 paired B0 | `not-run` | one qualification seed per class; three warmups per arm |
| C1/C2 paired B1/B2 | `not-run` | forbidden until B0 pass |
| Cleanup | `not-run` | stop `48300`/`48400`; clear only the campaign SecretStore item |

## Unique next action

Confirm `B01-Desktop-Linux-002` identity on the registered route, then
create the new root and freeze exact `370b26fc` binaries. Do not start a
counted sample before E7–E9 pass or honest `not-run` with recovery.

Claim ceiling `hypothesis`; `not_reviewed`. No Gate, release, Profile, B01,
or Agent-benefit promotion.
