# PERSONAL-PERF-EVAL-014 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-014`
- Lease: `lease/personal/EVAL-014/execution-plan-b0`
- Date: 2026-08-20
- Frozen product source: `adc40499` (`origin/main` after P9-T12 lease close;
  product merge PR [#252](https://github.com/agentkernel/cognitive-os/pull/252)
  at `39cf8019`).
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reopen EVAL-002 or EVAL-004 through
EVAL-013. It does not reuse those campaign roots, loopback ports
`48286`–`48298` / `48300` / `48302` / `48386`–`48398` / `48383` / `48400` /
`48402`, SecretStore items `/12`–`/19` / `/24` / `/25`, P2-T37 roots,
brokers, runners, corpora as denominators, or evidence files.

Owner 2026-08-20 activated this campaign by directing continuous autonomous
progress after P9-T12. Snapshot restore/delete and P9-T04 residue remain
outside the allowlist. The owner plaintext key file is not an allowed bind
path.

`tools/personal/c1-c2-paired/cells.json` still names reserved historical id
`PERSONAL-PERF-EVAL-012`. That file is not edited mid-campaign. This campaign
id is `PERSONAL-PERF-EVAL-014`.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval014-20260820` mode `0700` | `perfeval004`…`perfeval013`, `e009`, `~/perfeval002`, `~/p9t04`, `cos-current`, P2-T37 roots |
| Loopback port | daemon `127.0.0.1:48304`; broker `127.0.0.1:48404` | `48181`, `48282`, `48284`, `48286`–`48298`, `48300`, `48302`, `48383`, `48386`–`48398`, `48400`, `48402` |
| SecretStore entry | new item suffix `/26` | `/12`–`/19`, `/24`, `/25`; never `secret-tool search`/`lookup`; never keyfile copy |
| Source pin | `adc40499` | closed-EVAL pins; unmerged task branches |

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
Packages 1–14 and P9-T09–T12 are product/readiness closed. Packages 15–17
under EVAL-012/013 remain closed measurement and must not be resumed.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| EVAL-002 and EVAL-004–013 remain closed | **pass** | do not reopen; do not reuse their roots/ports/SecretStore items |
| Owner activation | **pass** | Current snapshot names `PERSONAL-PERF-EVAL-014` active |
| Evaluation lease claimed | **pass** | `lease/personal/EVAL-014/execution-plan-b0` |
| Product source pin | **pass** | `adc40499` |
| Source archive + SHA-256 | `not-run` | bind next; copies use `scp` |
| Guest identity | **pass** | `B01-Desktop-Linux-002` running (id 35); MAC `52:54:00:33:27:c1`; Ubuntu 24.04.4; uid 1000; `B01-Clean-Linux-001` shut off; residue listeners `48181`/`48284`/`48383` present and unused; closed roots `perfeval012`/`perfeval013` present; `perfeval014` absent |
| New campaign root/port | `not-run` | `/home/hal9001/perfeval014-20260820`; daemon `48304`; broker `48404` |
| Exact-source daemon/CLI/adapter binaries | `not-run` | build on `DEV-LINUX-NATIVE-01` at exact pin |
| Secret bind | `not-run` | new item `/26`; never `search`/`lookup` |
| Local Pi `0.81.1` pin | `not-run` | in-campaign `cli.js --version` |
| P-arm broker `48404` | `not-run` | after daemon bind |
| `cognitive doctor` | `not-run` | package-15 start gate |
| C1/C2 paired B0 | `not-run` | after doctor |
| B1/B2 C1/C2 paired | `not-run` | after B0 pass; use P9-T12 `runLivePairedCell` |
| Remainder of parent plan | `not-run` | C0/B3/B4/B5/T/S/O/UJ: overlay skip or missing runner |
| Cleanup | `not-run` | stop `48304`/`48404` only; clear `/26` only |

## Unique next action

Bind guest identity, then archive exact `adc40499` with `scp` (no PowerShell
pipe), create root `0700`, build exact-revision binaries, SecretStore `/26`,
daemon `48304`, broker `48404`, redacted doctor. Then B0. Do not cobble B0
shell into counted B1/B2.
