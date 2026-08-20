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
| Source archive + SHA-256 | **pass** | 15,155,200 bytes; 1597 members; 0 `.git/`; SHA-256 `0d4552c6b4bdec8b0941e6ea4470549f3b164fce2cc97174e289d465d38ef2ae`; copies used `scp` |
| Guest identity | **pass** | `B01-Desktop-Linux-002` running (id 35); MAC `52:54:00:33:27:c1`; Ubuntu 24.04.4; uid 1000; `B01-Clean-Linux-001` shut off; residue listeners `48181`/`48284`/`48383` present and unused; closed roots `perfeval012`/`perfeval013` present unused |
| New campaign root/port | **pass** | `/home/hal9001/perfeval014-20260820` mode `0700`; daemon `48304` pid 344759 listening; `48404` not yet; `48181`/`48284`/`48383` unused as campaign binds |
| Exact-source daemon/CLI/adapter binaries | **pass** | host `DEV-LINUX-NATIVE-01` rustc 1.97.1 `--locked --release`; `kernel-server` `436725ec685107142aa0f2298828713438f5d59219ee4b0b4107ab7185cfb548`; `cognitive` `73bad94dc9cdd6cc6c12882afcc7889bf165f573b632eea2471f91adcbfaf638`; `pi-agent-adapter` `54ce9eaa0e61febeff53d8e96b43f0d30570fcfb5fdd95e455715fe061991fce`; Extension `d27f97764e55b9a9b22bbf7e22e48c0ef2a017924ed13684b143b196991c1a57`; glibc-only `ldd` |
| Secret bind | **pass** | new item `/26`; `secret_material_written: true`; `secret_ref_redacted: true`; guest stdin pipe; never `search`/`lookup` |
| Local Pi `0.81.1` pin | **pass** | in-campaign `cli.js --version` `0.81.1` |
| P-arm broker `48404` | **pass** | pid 345123; `secret_material_written: false`; paths `["26"]`; C1 P-arm **pass** |
| `cognitive doctor` | **pass** | overall `ready`; `first_conversation_ready: true`; conversation-shell readiness ≠ C1/C2 Task |
| C1/C2 paired B0 | **pass** | C1 O/P **pass**; C2a Write+Patch O/P **pass**; fairness 13/13; secret-shaped 0/81; timeout/`retry=0`. C2b split-score; C2c `not-run`; C2d split-score |
| B1/B2 C1/C2 paired | B1 **pass** (C1/C2a 5/5); B2 N=30 frozen; B2 executing | `runLivePairedCell` + campaign `executeArm` |
| Remainder of parent plan | `not-run` | C0/B3/B4/B5/T/S/O/UJ: overlay skip or missing runner |
| Cleanup | `not-run` | stop `48304`/`48404` only; clear `/26` only |

## Unique next action

Unique next action: B2 C1/C2a N=30 via `runLivePairedCell`. Then parent-plan
remainder (`not-run` where runners missing) and cleanup of `48304`/`48404`/`/26`.
