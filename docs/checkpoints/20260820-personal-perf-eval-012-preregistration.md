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
| SecretStore entry | new item suffix `/24` (≠ `/12`–`/19`; planned `/20` was the reservation name) | `/12`–`/19`; never `secret-tool search`/`lookup`; never keyfile copy |
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
| Product source pin | **pass** | `370b26fcc05976c7c1c97e5510a99ed3ebc23f2c` |
| Source archive + SHA-256 | **pass** | 15,073,280 bytes; 1590 entries; 0 `.git/`; SHA-256 `1b41aeb31b70cdd59e60a598174eca00cc3f7f2ad1d51d1a005c370b0b9c1cdd`; copies used `scp` |
| Guest identity | **pass** | `B01-Desktop-Linux-002` running; MAC `52:54:00:33:27:c1` = `192.168.123.160`; Ubuntu 24.04.4; uid 1000; `B01-Clean-Linux-001` shut off |
| New campaign root/port | **pass** | `/home/hal9001/perfeval012-20260820` mode `0700`; `48300`/`48400` unused; `48181`/`48284`/`48383` untouched |
| Exact-source daemon/CLI/adapter binaries | **pass** | `kernel-server` `cfcfdaa2…`; `cognitive` `f02931df…`; `pi-agent-adapter` `54ce9eaa…`; glibc-only `ldd` |
| Secret bind | **pass** | new item `/24`; `secret_material_written: true`; `secret_ref_redacted: true`; guest temp shredded |
| Local Pi `0.81.1` pin | **pass** | in-campaign `cli.js --version` `0.81.1`; extension digest `d27f9776…` |
| P-arm broker `48400` | `not-run` | after O-arm bind; loopback-only |
| `cognitive doctor` | **pass** | overall `ready`; `first_conversation_ready: true`; `secret_ref_resolves: true`; daemon pid 326605 on `48300` |
| C1/C2 paired B0 | **partial** | C1 O-arm Search+Read **pass** (seed `sha256:a194b2f561562663`). C1 P-arm and C2a–d `not-run` |
| C1/C2 paired B1/B2 | `not-run` | forbidden until B0 pass |
| Cleanup | `not-run` | stop `48300`/`48400`; clear only the campaign SecretStore item |

## Unique next action

Start P-arm broker `127.0.0.1:48400` and C1 P-arm qualification. Then
C2a–d O/P. Do not open B1/B2.

Claim ceiling `hypothesis`; `not_reviewed`. No Gate, release, Profile, B01,
or Agent-benefit promotion.
