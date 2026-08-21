# PERSONAL-PERF-EVAL-015 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-015`
- Lease: `lease/personal/EVAL-015/remaining-plan-cells`
- Status: **active**. Evaluation routing **ON**.
- Date: 2026-08-21
- Frozen product source: `adc40499` (`origin/main` after P9-T12 lease close;
  product merge PR [#252](https://github.com/agentkernel/cognitive-os/pull/252)
  at `39cf8019`). Same pin as closed EVAL-014.
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reopen EVAL-002 or EVAL-004 through
EVAL-014. It does not reuse those campaign roots, loopback ports
`48286`–`48298` / `48300` / `48302` / `48304` / `48386`–`48398` / `48383` /
`48400` / `48402` / `48404`, SecretStore items `/12`–`/19` / `/24` / `/25` /
`/26`, P2-T37 roots, or closed-EVAL evidence files as denominators.

Owner 2026-08-21 activated this campaign to complete parent plan §9 remainder
after EVAL-014 closed. Snapshot restore/delete and P9-T04 residue remain
outside the allowlist. The owner plaintext key file is not an allowed bind
path; import uses `cognitive init --api-key-file -` by shape only
(`sed -n '8p' | tr -d '\r'`).

C1/C2a B0/B1/B2 counted on EVAL-014 at this pin are **carried**, not re-run.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval015-20260821` mode `0700` | `perfeval004`…`perfeval014`, `e009`, `~/perfeval002`, `~/p9t04`, `cos-current`, P2-T37 roots |
| Loopback port | daemon `127.0.0.1:48306`; broker `127.0.0.1:48406` | `48181`, `48282`, `48284`, `48286`–`48298`, `48300`, `48302`, `48304`, `48383`, `48386`–`48398`, `48400`, `48402`, `48404` |
| SecretStore entry | new item suffix `/27` | `/12`–`/19`, `/24`, `/25`, `/26`; never `secret-tool search`/`lookup`; never keyfile copy |
| Source pin | `adc40499` | closed-EVAL pins; unmerged task branches |

`B01-Clean-Linux-001` stays out of bounds. Guest control:
`virsh -c qemu:///system` on `hal9000` only. The domain is used as-is. Do
not start, restore, or delete snapshots.

PowerShell SSH pipes corrupt tar digests; copies use `scp`. Guest copies
go Windows → ProxyJump `hal9001@192.168.123.160`.

## Parent-plan cell order (this freeze)

Carry EVAL-014 C1/C2a. Execute remaining required cells: freeze → C0 B0 →
C0 B1 → freeze B2 N → C0 B2 → MS-AUTH / T-GOV / UJ2 / UJ3 / UJ4 → B3 → B4
→ B5 (1 h then 8 h; 24 h default deferred) → C2b resume/Skill and C2c
fault profile if campaign-local → cleanup + secret scan → analysis →
report. Missing runner or capability is `not-run`/`not_available`.
`retry=0`. Retain every started sample. B6 is not this campaign's mutex.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| EVAL-002 and EVAL-004–014 remain closed | **pass** | do not reopen; do not reuse their roots/ports/SecretStore items as binds |
| Owner activation | **pass** | Current snapshot names `PERSONAL-PERF-EVAL-015` active |
| Evaluation lease claimed | **pass** | `lease/personal/EVAL-015/remaining-plan-cells` |
| Product source pin | **pass** | `adc40499` (same as EVAL-014; C1/C2a counted cells carried) |
| Guest identity | **pass** (read 2026-08-21) | `B01-Desktop-Linux-002` running (id 35); MAC `52:54:00:33:27:c1`; Ubuntu 24.04.4; uid 1000; `B01-Clean-Linux-001` shut off; residue listeners `48181`/`48284`/`48383` present unused; closed roots `perfeval012`/`013`/`014` present unused |
| Login SecretStore | **pass** (pre-bind) | D-Bus collection `item_count=0`; new item `/27` required |
| Source archive + SHA-256 | **pass** | 15,155,200 bytes; SHA-256 `0d4552c6b4bdec8b0941e6ea4470549f3b164fce2cc97174e289d465d38ef2ae`; copies used `scp` |
| Exact-source binaries | **pass** | reused EVAL-014 host freeze digests; `kernel-server` `436725ec…`; `cognitive` `73bad94d…`; `pi-agent-adapter` `54ce9eaa…`; Extension `d27f9776…`; Pi `0.81.1` |
| Secret bind | **pass** | new item `/27`; `secret_material_written: true`; `secret_ref_redacted: true`; guest stdin pipe; never `search`/`lookup` |
| New campaign root/port | **pass** | `/home/hal9001/perfeval015-20260821` mode `0700`; daemon `48306` pid 369399; broker `48406` pid 369469; residue listeners unused |
| C0 corpus/runner freeze | **pass** | instruments re-pathed into this root; corpus `38e282d4…`; runner adapted; stdin broker (no lookup) |
| C0 B0 / B1 / B2 | B0 **pass**; B1 **pass** 90/90; B2 **pass** 270/270 | N=30 frozen; P 250/270 vs O 247/270; wall median `O−P` +91.1 ms |
| MS-AUTH / T-GOV / T2 / UJ2 / UJ3 / UJ4 | recorded | Skill unique-digest 10/10 combined; UJ4 30/30 admit; UJ2 cold 10/10; UJ3 task-watch **partial** 403 |
| B3 / B4 | **pass** | B3 mismatch/restart/remaining executed; B4 932/932 local non-OK 0; mixed Agent `not-run` |
| C2b / C2c | C2b **partial**; C2c **pass** split-score | session-2 GET without restart (404 pins); fault-profile authorized+default-off; original-key GET 200 after UJ2 |
| B5 | 1 h **pass**; 8 h running (resumed at minute 180) | 60/60 1 h; 8 h minutes 0–181 contiguous `local_non_ok=0`; 19/19 pairs; pid 408358 / kernel 408375; minute-180 stale-lock fail retained then health 200 in 50.4 s; 24 h default deferred |
| Cleanup | pending | stop `48306`/`48406`; clear `/27` only |

## Unique next action

Finish the live B5 8 h continuation (pid **408358**, last minute 181).
Do not start a second continuation. Record 24 h default deferred
unless the 8 h slope trigger is met, then cleanup + secret scan +
close the campaign.
