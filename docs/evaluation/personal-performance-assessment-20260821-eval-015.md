# PERSONAL-PERF-EVAL-015 running assessment

- Campaign: `PERSONAL-PERF-EVAL-015`
- Freeze branch: `evaluation/EVAL-015-freeze`
- Product pin: `adc40499`
- Lease: `lease/personal/EVAL-015/remaining-plan-cells`
- Preregistration: [20260821-personal-perf-eval-015-preregistration.md](../checkpoints/20260821-personal-perf-eval-015-preregistration.md)
- Claim ceiling: `hypothesis` / `not_reviewed`
- Independent reviewer: `not_reviewed`
- Document status: campaign **active**. Measurement-only.

This freeze completes parent plan §9 remainder after closed EVAL-014.
EVAL-014 C1/C2a B0/B1/B2 on this pin are carried as prior evidence and are
not re-run. This report does not promote Gate, release, Profile, B01, or
Agent-benefit. EVAL-002 and EVAL-004 through EVAL-014 remain closed.

## Cell log (`TEST-REPORT-INCREMENTAL-01`)

| Cell | Result | Note |
|---|---|---|
| Owner activation / lease | **pass** | `PERSONAL-PERF-EVAL-015`; `lease/personal/EVAL-015/remaining-plan-cells` |
| Guest identity | **pass** | `B01-Desktop-Linux-002` running id 35; MAC `52:54:00:33:27:c1`; Ubuntu 24.04.4; uid 1000; `B01-Clean-Linux-001` shut off; residue `48181`/`48284`/`48383` untouched; `perfeval012`/`013`/`014` present unused |
| Carry EVAL-014 C1/C2a | **carried** | B0 pass; B1 5/5; B2 30/30 counted on pin `adc40499`. Not re-run. Report: [EVAL-014](personal-performance-assessment-20260820-eval-014.md) |
| Source archive | **pass** | 15,155,200 bytes; SHA-256 `0d4552c6b4bdec8b0941e6ea4470549f3b164fce2cc97174e289d465d38ef2ae`; `scp` into `/home/hal9001/perfeval015-20260821` |
| Exact-source binaries | **pass** | host `DEV-LINUX-NATIVE-01` `--locked --release` reused by digest; `kernel-server` `436725ec…`; `cognitive` `73bad94d…`; `pi-agent-adapter` `54ce9eaa…`; Extension `d27f9776…`; Pi `0.81.1`; glibc-only |
| Secret bind / doctor | **pass** | new item `/27`; `secret_material_written: true`; `secret_ref_redacted: true`; `selected_model: deepseek-v4-flash`; daemon pid 369399 on `48306`; doctor overall `ready`; `first_conversation_ready: true` |
| P-arm broker `48406` | **pass** | stdin broker pid 369469; D-Bus GetSecret paths `["27"]`; `material_written: true` to broker stdin only; health `key_loaded: true`; never `search`/`lookup` |
| C0 B0 warmups | **pass** (discarded) | 3/3 G1 pairs both oracle True; not in denominator |
| C0 B0 qualification | **pass** | 9/9 blocks started/retained; 0 timeout; 7/9 oracle both arms; G6/G9 both-fail (hardness). Broker 12/12 upstream_ok including warmups. Evidence secret-shaped 0/10. `retry=0` |
| C0 B1 | **pass** | 90/90 retained; P 81/90 (90.0%); O 80/90 (88.9%); 0 timeout; `retry=0`. B2 N frozen at 30 (not shrunk). Secret-shaped 0/12 |
| C0 B2 N freeze | **pass** | N=30 per C0 family (formal minimum; B1 does not reduce N) |
| C0 B2 | **pass** | 270/270 retained; P 250/270 (92.6%); O 247/270 (91.5%); delta −1.1 pp; P-only 7 / O-only 4; paired wall median `O−P` +91.1 ms; 0 timeout; `retry=0`. Evidence `b2-c0-confirmatory.jsonl` |
| MS-AUTH negatives | **pass** (5/6) | `bind_unknown` 400 `RESOURCE_SKILL_ID_INVALID` (`ok: false` vs historical 409). Tighter validation, not a product defect. Unsealed Memory 20/20 HTTP 400; sealed-header `not-run` (no fabricated `GovernanceSeed`) |
| MS-AUTH Skill | **pass** | unique-digest continuation 9/9 import 201 + inspect 200 + bind 201; combined with prior round-0 **10/10** |
| T-GOV | **pass** | 7/7 `execution_ready` (Workspace* + process + http + registered_check) |
| T2 | **pass** | 65/65 ok lifecycle (enable/disable/quarantine/channel/unauth) |
| UJ3 daily | **pass** | health 200/200 p50 0.379 ms; CLI status n=100 p50 1.742 s; doctor n=50 p50 1.791 s; six-resource GET 50/50; isolation 401/403 as designed |
| UJ3 task-watch | **partial** | 20/20 HTTP 403 `SHELL_CHANNEL_BINDING_MISMATCH` (expected channel isolation; same class as EVAL-004) |
| UJ4 | **pass** | 30/30 unique read-only preview+admit; same-process watch 200/200; durable post-restart query `not_available`; no raw SQLite. Scheduler skip: `private Pi candidate adapter is not configured` (no extra Provider) |
| UJ2 cold | **pass** | 10/10 confirmatory-A4 both arms oracle True; `--bind 127.0.0.1:48306` before each O arm; 10/10 health 200 after restart; 0 timeout; `retry=0`; wall median `O−P` +366.0 ms (cold-start descriptive) |
| B3 mismatch/restart/pi-kill | **pass** | mismatch 10/10 `PERSONAL_PROVIDER_SELECTED_MODEL_MISMATCH`; restart 10/10 stop/start, 0 locks, 0 orphans, down_refused 10; pi_kill n=10 `killed: 0`, returncodes `{1}` |
| B3 remaining | **pass** | deadline n=10 completed 8 / timeouts 2 (retained); broker-unavailable 10/10 then health 200; upstream timeout 10/10; oversize 10/10. Stale/`OUTCOME_UNKNOWN` `not-run` (no mutation injector in this cell) |
| B4 local | **pass** | 932 requests, `total_non_ok=0`; profiles 1/8/16 + overload 17/33 + health-after recovery. Mixed Agent `not-run` (optional; budget reserved for B5 paired soak) |
| C2b session-2 / Skill | **partial** (split-score) | O: unsealed remember 201 `memory_id` present; unique Skill import+bind 201; Task admitted; session-1/2 GET consumption 404 `RESOURCE_CONSUMPTION_NOT_FOUND` **without daemon restart**; restatement 400 `RESOURCE_CONSUMPTION_RESTATEMENT_FORBIDDEN`. Pins absent until governed resolve (adapter not configured). P-arm cannot share daemon Memory/Skill |
| C2c fault profile | **pass** (split-score) | unauthorized campaign 403 `RESOURCE_FAULT_PROFILE_UNAUTHORIZED`; task-channel 403 `RESOURCE_FAULT_PROFILE_CHANNEL_FORBIDDEN`; `PERSONAL-PERF-EVAL-015` enable 200 `faults_enabled=true` then default-off 200. Original-key `GET /task/effects` 200 after UJ2 restarts (`effects` present, no mutation receipts). P is fixture reference |
| B5 1 h | running | campaign-local plan soak pid 397664; first G1 pair completed/True both arms; local health/projection every minute; paired C0 every 5 min |
| B5 8 h | pending | starts only if 1 h has 0 local non-OK and 0 pair timeouts |
| B5 24 h | default deferred | trigger is 8 h unresolved slope **and** owner budget; not opened |
| T4–T9 / S4/S8 / O2–O6 extras | `not-run` / `not_available` | plan §10 expected-not-run or no public observation plane; not this freeze’s mutex |
| Cleanup | pending | stop `48306`/`48406`; clear `/27` only |

## Non-claims

Activation is not B0. Carrying EVAL-014 is not a new C1/C2a sample. No Gate /
release / Profile / B01 / Agent-benefit promotion.

## Source freeze (2026-08-21) — pass

Same-pin `git archive` of `adc40499` copied with `scp` (no PowerShell pipes):
15,155,200 bytes, SHA-256
`0d4552c6b4bdec8b0941e6ea4470549f3b164fce2cc97174e289d465d38ef2ae`.
Host freeze binaries reused by digest. Guest Pi `cli.js --version` is
`0.81.1`. Closed EVAL roots were file-copied for Pi runtime and C0
instruments only; they were not used as this campaign's binds.

## Secret bind / doctor (2026-08-21) — pass

Login collection was empty. Owner-designated key imported through
`cognitive init --api-key-file -` into new SecretStore item `/27`.
`secret_material_written: true`, `secret_ref_redacted: true`,
`selected_model: deepseek-v4-flash`. No temp key file. No
`secret-tool search`/`lookup`. Daemon `127.0.0.1:48306` pid 369399.
Doctor overall `ready`, `first_conversation_ready: true`. Residue
listeners `48181`/`48284`/`48383` untouched.

## C0 B0 (2026-08-21) — pass (path/fairness; not a performance claim)

Three discarded G1 warmup pairs (`start-index` 100, 3 replicas): both
arms oracle True. Qualification: one pilot seed per C0 family
(`start-index` 0, 1 replica). 9/9 started and retained. Every arm
`completed` (0 timeout). `retry=0`. 180 s timeout. Mechanical `ANSWER:`
oracle. Same Pi `0.81.1`, `--no-tools`, model `deepseek-v4-flash`.

| Family | Arm order | P oracle | O oracle | P wall ms | O wall ms |
|---|---|---|---|---:|---:|
| A5 | P,O | pass | pass | 16162 | 10270 |
| G2 | O,P | pass | pass | 3552 | 3350 |
| G6 | P,O | fail | fail | 23029 | 7175 |
| G1 | P,O | pass | pass | 3406 | 3741 |
| G3 | P,O | pass | pass | 6133 | 4681 |
| G4 | P,O | pass | pass | 4335 | 3792 |
| A1 | P,O | pass | pass | 4436 | 4126 |
| G9 | O,P | fail | fail | 4727 | 4678 |
| A4 | O,P | pass | pass | 3650 | 3793 |

G6/G9 both-fail is family hardness (same pattern as EVAL-004 B0), not an
arm-specific instrument defect. Evidence secret-shaped hits 0/10.

## C0 B1 (2026-08-21) — pass (hypothesis / non-claim)

90 paired blocks, 180/180 arm-runs retained. Pilot seeds 1–5 × 2 replicas
per family; disjoint from B0 seed 0 and warmup 100. `retry=0`. 0 timeout /
process_error. Evidence `b1-c0-pilot.jsonl`. Secret-shaped scan 0/12 files.

| Endpoint | `P` | `O` |
|---|---:|---:|
| oracle completion | 81/90 = **90.0 %** | 80/90 = **88.9 %** |
| paired wall delta median `O−P` | | **+87.3 ms** (descriptive; N=90) |

Per-family (n=10 each; descriptive):

| Family | `P` | `O` |
|---|---|---|
| G1 G2 G3 G4 A1 A4 | 10/10 | 10/10 |
| G6 | 8/10 | 9/10 |
| G9 | 4/10 | 3/10 |
| A5 | 9/10 | 8/10 |

B1 does not freeze B2 below 30. Independent reviewer `not_reviewed`.

## B2 N freeze (2026-08-21)

Formal B2 N = **30** paired seeds per C0 family from stratum
`confirmatory` (270 pairs, 1 replica). Timeout 180 s, `retry=0`.

## C0 B2 (2026-08-21) — pass (hypothesis / non-claim)

270 confirmatory pairs, 540/540 arm-runs retained. Stratum `confirmatory`,
1 replica, N=30 per family. `retry=0`. 0 timeout / process_error. Evidence
`b2-c0-confirmatory.jsonl`.

| Endpoint | `P` | `O` |
|---|---:|---:|
| oracle completion | 250/270 = **92.6 %** | 247/270 = **91.5 %** |
| paired completion delta `P−O` | | **−1.1 pp** (descriptive) |
| P-only / O-only | 7 | 4 |
| paired wall delta median `O−P` | | **+91.1 ms** (descriptive; N=270) |

Per-family (n=30; descriptive):

| Family | `P` | `O` |
|---|---|---|
| G1 G2 G3 G4 A4 | 30/30 | 30/30 |
| G6 | 21/30 | 21/30 |
| G9 | 22/30 | 17/30 (−16.7 pp) |
| A1 | 29/30 | 30/30 |
| A5 | 28/30 | 29/30 |

Versus closed EVAL-004 (~+1.7 s overhead) this pin’s C0 wall delta is ~90 ms.
That is a descriptive observation, not an Agent-benefit or Gate claim.

Broker after C0 B2: accepted 372 / rejected 0 / upstream_ok 372 (later UJ2
added 10 P-arm calls → 422/0/422 before B5).

## MS-AUTH / T-GOV / T2 / UJ3 / UJ4 (2026-08-21)

Public-surface cells. Claim ceiling `hypothesis`. No raw SQLite.

- **MS-AUTH negatives:** 5/6 instrument `ok`. `bind_unknown` returned 400
  `RESOURCE_SKILL_ID_INVALID` (instrument expected 409). Unsealed Memory
  remember 20/20 HTTP 400. Sealed-header composer `not-run`.
- **MS-AUTH Skill:** unique-digest packages 9/9 import 201; combined with
  prior round-0 **10/10**. Inspect 200 and bind 201 on those nine.
- **T-GOV:** tool projection 200; 7/7 `execution_ready`.
- **T2:** 65/65 retained lifecycle outcomes on `native.http.fetch`.
- **UJ3 daily:** health 200/200 p50 0.379 ms; CLI status n=100 p50 1.742 s;
  doctor n=50 p50 1.791 s; six-resource GET 50/50 each; isolation 401/403.
- **UJ3 task-watch:** 20/20 HTTP 403 `SHELL_CHANNEL_BINDING_MISMATCH`
  (**partial**, expected channel isolation).
- **UJ4:** 30 unique `native.workspace.read` record/interpret/preview/admit
  all 200; same-process watch 200/200. Durable post-restart Task query
  remains `not_available`. Scheduler ticks skipped with
  `private Pi candidate adapter is not configured` — no extra Provider
  spend.

## UJ2 cold (2026-08-21) — pass (hypothesis / non-claim)

10 confirmatory-A4 pairs. `retry=0`. Every started sample retained.
Daemon stop/start with `--bind 127.0.0.1:48306` before each O arm. 10/10
post-start health 200. P 10/10 and O 10/10 oracle True. 0 timeout.
Paired wall median `O−P` **+366.0 ms** (cold-start descriptive; not
comparable to warm C0 B2 +91.1 ms as a benefit claim). Evidence
`uj2-cold.jsonl`. Current daemon pid after UJ2: 397384.

## B3 / B4 (2026-08-21)

B3 mismatch 10/10 `PERSONAL_PROVIDER_SELECTED_MODEL_MISMATCH`. Restart
cycle 10/10: stop_ok 10, start_ok 10, locks_after_stop 0, orphans 0,
down_refused 10; start p50 91.7 ms. Pi-kill n=10 `killed: 0`,
returncodes `{1}` (process already gone; retained). Remaining: deadline
8 completed / 2 timeouts retained; broker-unavailable 10/10 (broker
restored, health 200); upstream timeout 10/10; oversize 10/10.
Stale/`OUTCOME_UNKNOWN` `not-run` (no mutation injector in the B3
instrument).

B4 local: 932 requests, `total_non_ok=0`. Health 1/8/16, projection
1/8/16, overload 17/33, health-after recovery 100/100 p50 0.357 ms.
Mixed Agent concurrency `not-run` (optional; Provider budget reserved
for B5 paired soak).

## C2b / C2c (2026-08-21) — split-score; retained

C2b O-arm public lifecycle: unsealed remember **201** (`memory_id`
present), unique frozen Skill import **201** and bind **201**, Task
`task://personal/eval015-c2b-resume` admitted. Session-1 and session-2
`GET /task/resource/v1/consumption` both 404
`RESOURCE_CONSUMPTION_NOT_FOUND` with **no campaign-daemon restart**.
Restatement `query_text` 400 `RESOURCE_CONSUMPTION_RESTATEMENT_FORBIDDEN`.
Consumption pins require governed resolve; this freeze’s public daemon
does not run the private Pi candidate adapter, so pins stay absent.
P-arm cannot share daemon Memory/Skill. Not a C1/C2a fairness fail.

C2c: `POST /management/resource/v1/fault-profile` with
`campaign_id=PERSONAL-PERF-EVAL-015` (≤32 chars, authorized prefix)
returned 200 `faults_enabled=true` /
`fault_point=mutation_after_receipt_before` on
`task://personal/eval015-c2c-fault`, then default-off 200. Unauthorized
`owner-local` 403 `RESOURCE_FAULT_PROFILE_UNAUTHORIZED`. Task-channel
403 `RESOURCE_FAULT_PROFILE_CHANNEL_FORBIDDEN`. After UJ2 cold restarts,
original-key `GET /task/effects` returned 200 with schema keys
`effects` / `contract_epoch` / `effects_truncated` and no mutation
receipts (Task never dispatched). Split-score vs P-arm fixture
reference.

## B5 (2026-08-21) — 1 h running

Campaign-local `b5_plan_soak.py` pid **397664**. 1 h: local
health/projection every minute; paired G1 confirmatory block every 5
minutes (`retry=0`, soak label, not a B2 quality sample). First pair
both arms `completed`/`True`. 8 h starts only if 1 h local non-OK = 0
and pair timeouts = 0. 24 h remains default deferred.

## Unique next action

Finish B5 1 h soak, then B5 8 h if the 1 h exit holds, record 24 h
default deferred, then cleanup + secret scan + final assessment and
close the campaign row/lease. Do not auto-start unrelated backlog.
