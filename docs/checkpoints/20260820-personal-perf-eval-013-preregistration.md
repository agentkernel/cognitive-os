# PERSONAL-PERF-EVAL-013 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-013`
- Lease: `lease/personal/EVAL-013/execution-plan-b0`
- Date: 2026-08-20
- Frozen product source: `6c415625` (`origin/main` after P9-T11 lease close;
  product merge PR [#251](https://github.com/agentkernel/cognitive-os/pull/251)
  at `49b66200`).
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reopen EVAL-002 or EVAL-004 through
EVAL-012. It does not reuse those campaign roots, loopback ports
`48286`–`48298` / `48300` / `48386`–`48398` / `48383` / `48400`, SecretStore
items `/12`–`/19` / `/24`, P2-T37 roots, brokers, runners, corpora as
denominators, or evidence files.

Owner 2026-08-20 activated this campaign by directing completion of
[personal-performance-benchmark-execution-plan.md](../evaluation/personal-performance-benchmark-execution-plan.md)
after P9-T11 closure. Snapshot restore/delete and P9-T04 residue remain
outside the allowlist. The owner plaintext key file is not an allowed bind
path.

`tools/personal/c1-c2-paired/cells.json` still names reserved historical id
`PERSONAL-PERF-EVAL-012`. That file is not edited mid-campaign. This campaign
id is `PERSONAL-PERF-EVAL-013`.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval013-20260820` mode `0700` | `perfeval004`…`perfeval012`, `e009`, `~/perfeval002`, `~/p9t04`, `cos-current`, P2-T37 roots |
| Loopback port | daemon `127.0.0.1:48302`; broker `127.0.0.1:48402` | `48181`, `48282`, `48284`, `48286`–`48298`, `48300`, `48383`, `48386`–`48398`, `48400` |
| SecretStore entry | new item suffix `/25` | `/12`–`/19`, `/24`; never `secret-tool search`/`lookup`; never keyfile copy |
| Source pin | `6c415625` | closed-EVAL pins; unmerged task branches |

`B01-Clean-Linux-001` stays out of bounds. Guest control:
`virsh -c qemu:///system` on `hal9000` only. The domain is used as-is. Do
not start, restore, or delete snapshots.

PowerShell SSH pipes corrupt tar digests; copies use `scp`.

## Parent-plan cell order

Execute [execution plan](../evaluation/personal-performance-benchmark-execution-plan.md)
§9: freeze → fairness/secret/denominator review → **B0** → B1 → freeze B2 N →
B2 → B3 → B4 → B5 (1 h then 8 h; 24 h conditional/default deferred) → cleanup
+ secret scan → analysis → report. Missing runner or capability is
`not-run`/`not_available`. `retry=0`. Retain every started sample. B6 is
later optimization replay, not this campaign's mutex.

C1/C2 overlay: [PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md](../plan/PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md).
Packages 1–14 and P9-T09–T11 are product/readiness closed. Packages 15–17
under EVAL-012 remain closed measurement and must not be resumed.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| EVAL-002 and EVAL-004–012 remain closed | **pass** | do not reopen; do not reuse their roots/ports/SecretStore items |
| Owner activation | **pass** | Current snapshot names `PERSONAL-PERF-EVAL-013` active |
| Evaluation lease claimed | **pass** | `lease/personal/EVAL-013/execution-plan-b0` |
| Product source pin | **pass** | `6c415625` |
| Source archive + SHA-256 | pending | scp; no `.git/`; PowerShell pipes forbidden |
| Guest identity | pending | `B01-Desktop-Linux-002` only |
| New campaign root/port | pending | `/home/hal9001/perfeval013-20260820`; `48302`/`48402` unused |
| Exact-source daemon/CLI/adapter binaries | pending | glibc-only `ldd` |
| Secret bind | pending | new item `/25`; never search/lookup |
| Local Pi `0.81.1` pin | pending | `--extension <absolute-path>` |
| P-arm broker `48402` | pending | `secret_material_written: false` |
| `cognitive doctor` | pending | live bind |
| C1/C2 paired B0 | pending | fairness + C1/C2a samples; `retry=0` |
| Remainder of parent plan | pending | B1 only after B0 pass; honest `not-run` otherwise |
| Cleanup | pending | stop `48302`/`48402`; clear only `/25` |

## Unique next action

Complete the freeze on `B01-Desktop-Linux-002`: guest identity, new root
`perfeval013-20260820`, exact `6c415625` binaries, SecretStore `/25`, doctor,
then B0 qualification. Do not open B1 until B0 fairness passes.
