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
| Source archive + SHA-256 | **pass** | 15,134,720 bytes; 1596 members; 0 `.git/`; SHA-256 `d06923c04febece5d7175cadff54a366df07cce031bf8249dc0aaeff7c92e06a`; copies used `scp` |
| Guest identity | **pass** | `B01-Desktop-Linux-002` running; MAC `52:54:00:33:27:c1`; Ubuntu 24.04.4; uid 1000; `B01-Clean-Linux-001` shut off |
| New campaign root/port | **pass** | `/home/hal9001/perfeval013-20260820` mode `0700`; daemon `48302` listening; `48402` unused; `48181`/`48284`/`48383`/`48300`/`48400` not reused as campaign binds |
| Exact-source daemon/CLI/adapter binaries | **pass** | `kernel-server` `436725ec…`; `cognitive` `73bad94d…`; `pi-agent-adapter` `54ce9eaa…`; Extension `index.js` `d27f9776…`; glibc-only `ldd` |
| Secret bind | **pass** | new item `/25`; `secret_material_written: true`; `secret_ref_redacted: true`; guest temp shredded |
| Local Pi `0.81.1` pin | **pass** | in-campaign `cli.js --version` `0.81.1`; extension digest `d27f9776…` |
| P-arm broker `48402` | **pass** | pid 339769; `secret_material_written: false`; paths `["25"]`; C1 P-arm Search+Read **pass** |
| `cognitive doctor` | **pass** | overall `ready`; `first_conversation_ready: true`; daemon pid 336122 on `48302` |
| C1/C2 paired B0 | **pass** | C1 O/P **pass**; C2a Write+Patch O/P **pass**; fairness 13/13; secret-shaped 0/86; timeout/`retry=0`. C2b split-score (O remember 201; P `Done.`); C2c `not-run`; C2d split-score pass |
| Remainder of parent plan | `not-run` | B1/B2: no frozen live paired executor (§2.5). C0/B3/B4/B5/T/S/O/UJ: overlay skip or missing runner |
| Cleanup | **pass** | daemon `48302` pid 336122 stopped; broker `48402` pid 339769 gone; SecretStore `/25` cleared; residue listeners untouched |

## Unique next action

Campaign closed 2026-08-20. Do not reopen this freeze. Wait for an explicit
owner delivery instruction before claiming any implementation task.
