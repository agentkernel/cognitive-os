# PERSONAL-PERF-EVAL-014 running assessment

- Campaign: `PERSONAL-PERF-EVAL-014`
- Freeze branch: `evaluation/EVAL-014-freeze`
- Product pin: `adc40499`
- Lease: `lease/personal/EVAL-014/execution-plan-b0` (**closed** 2026-08-20)
- Preregistration: [20260820-personal-perf-eval-014-preregistration.md](../checkpoints/20260820-personal-perf-eval-014-preregistration.md)
- Claim ceiling: `hypothesis` / `not_reviewed`
- Independent reviewer: `not_reviewed`
- Document status: campaign **closed**. Measurement-only. Evaluation routing OFF.

Measurement-only. This report does not promote Gate, release, Profile, B01,
or Agent-benefit. EVAL-002 and EVAL-004 through EVAL-013 remain closed.

## Cell log (`TEST-REPORT-INCREMENTAL-01`)

| Cell | Result | Note |
|---|---|---|
| Freeze / preregistration | **pass** | pin `adc40499`; root `/home/hal9001/perfeval014-20260820`; daemon `48304`; broker `48404`; SecretStore `/26` |
| Guest identity | **pass** | `B01-Desktop-Linux-002` running id 35; MAC `52:54:00:33:27:c1`; Ubuntu 24.04.4; uid 1000; `B01-Clean-Linux-001` shut off; residue `48181`/`48284`/`48383` untouched; `perfeval012`/`013` present unused |
| Source archive | **pass** | 15,155,200 bytes; 1597 members; 0 `.git/`; SHA-256 `0d4552c6b4bdec8b0941e6ea4470549f3b164fce2cc97174e289d465d38ef2ae`; copies used `scp` |
| Exact-source binaries | **pass** | host `DEV-LINUX-NATIVE-01` `--locked --release`; `kernel-server` `436725ec…`; `cognitive` `73bad94d…`; `pi-agent-adapter` `54ce9eaa…`; Extension `index.js` `d27f9776…`; glibc-only `ldd`; Pi `0.81.1` |
| Secret bind / doctor | **pass** | new item `/26`; `secret_material_written: true`; `secret_ref_redacted: true`; `selected_model: deepseek-v4-flash`; daemon pid 344759 on `48304`; doctor overall `ready`; `first_conversation_ready: true` |
| P-arm broker `48404` | **pass** | pid 345123; `secret_material_written: false`; paths `["26"]`; placeholder token only |
| B0 fairness | **pass** (C1 axes) | live P/O both injected frozen-system-task-prompt.txt (211 UTF-8 bytes); 13/13 axes pass; `b0: true` for this C1 record. Full B0 still requires C2a+ |
| B0 C1 O-arm | **pass** | 3 Search warmups + counted Search + counted Read. All `COMPLETED` / `ACCEPTANCE_GRANTED`; verification `passed`/`current`; O4 `lease_acquired` 1. Live `--append-system-prompt`. `retry=0` |
| B0 C1 P-arm | **pass** | broker `48404`; SecretStore paths `["26"]`; `secret_material_written: false`; Pi placeholder token only. Search hit `failing-line`; Read returned both note lines. Live `--append-system-prompt`. No daemon Task. `retry=0` |
| B0 C2a–C2d | see rows below | C1/C2a comparable pass; C2b/C2d split-score; C2c `not-run` |
| B0 C2a Write O/P | **pass** | O: 3 Write warmups + counted Write `COMPLETED`/`ACCEPTANCE_GRANTED`, O4 lease 1. P: fixture writes `c2a-write\n`. Unified live `--append-system-prompt`. `retry=0` |
| B0 C2a Patch O/P | **pass** | O counted Patch `COMPLETED`/`ACCEPTANCE_GRANTED`, verification `passed`/`current`, lease 1, preimage `cb4ff53fe4…`. P counted Patch unified-diff post-state `c2a-patch-v2\n` |
| B0 C2b O-arm | **partial** | public unsealed remember **201** `remembered` (`memory_id` present). Session-2 resume `not-run`. Skill bind `not-run` |
| B0 C2b P-arm | **pass** | exit 0 with live `--append-system-prompt`; WorkspaceRead of fixture `procedure.txt`. Split-score: P does not use daemon Memory/Skill |
| B0 C2c O-arm | `not-run` | no frozen campaign-authorized default-off fault injector on this EVAL |
| B0 C2c P-arm | `not-run` | not opened after O `not-run` |
| B0 C2d O-arm | **pass** | observed counted C2a Patch: `COMPLETED` / verification `passed`/`current` / `reconcile_class=closed` |
| B0 C2d P-arm | **pass** | mechanical oracle stdout `ANSWER: repaired\n` (exit 0). Split-score vs O |
| B0 extras | **pass** | secret-shaped hits 0/81 evidence files; `timeout_ms=120000`; `retry=0`; `max_agent_turn=8` |
| B0 overall | **pass** | C1/C2a comparable + C1 fairness 13/13 + secret/timeout. C2b–d remain split-score / capability-gap |
| B1 C1/C2 paired | **pass** (C1/C2a) | C1 5/5 counted; C2a 5/5 counted; fairness pass each cell; `retry=0`. C2b–d B1 `not-run` (not comparable) |
| B2 N freeze | **pass** | N=30 per comparable class (`freeze.mjs` b2; n=5 B1 cannot reduce below 30) |
| B2 C1/C2 paired | **pass** (C1/C2a) | C1 30/30 counted; C2a 30/30 counted; fairness pass each cell; `retry=0`; fails=0. C2b–d B2 `not-run` |
| C0 / B3–B5 / T/S extras | `not-run` | overlay skip or missing runner |
| Cleanup | **pass** | daemon `48304` pid 344759 stopped; broker pid 345123 gone; SecretStore `/26` cleared; residue listeners untouched; evidence secret-shaped 0/642 |

## Non-claims

Activation is not B0. P9-T12 live executor existence is not a counted sample.
No Gate / release / Profile / B01 / Agent-benefit promotion.

## Source freeze (2026-08-20) — pass

`git archive` of `adc40499`: 15,155,200 bytes, 1597 members, 0 `.git/`,
SHA-256 `0d4552c6b4bdec8b0941e6ea4470549f3b164fce2cc97174e289d465d38ef2ae`.
Copies used `scp` (no PowerShell pipes). Host `DEV-LINUX-NATIVE-01` rustc
1.97.1 built `--locked --release` `kernel-server` / `admin-cli` /
`pi-agent-adapter` and `pnpm --filter @cognitiveos/pi-cognitiveos run build`.
glibc-only `ldd`. Guest Pi `cli.js --version` is `0.81.1`. Closed EVAL
roots and residue listeners were not used as this campaign's binds.

## Secret bind / doctor (2026-08-20) — pass

Login collection was empty. Owner-designated key imported through
`cognitive init --api-key-file -` into new SecretStore item `/26`.
`secret_material_written: true`, `secret_ref_redacted: true`,
`selected_model: deepseek-v4-flash`. No temp key file. No
`secret-tool search`/`lookup`, no `provider.json` copy. Pi configured with
absolute `cli.js` and Extension `index.js`. Daemon started on
`127.0.0.1:48304` (pid 344759). Doctor overall `ready`,
`first_conversation_ready: true`. This is conversation-shell readiness, not
a C1/C2 Task.

## B0 C1 O-arm (2026-08-20) — pass; retained

Frozen C1 fixture `note.txt` SHA-256
`4fb26b79e8de937c59f203f9274d76998db1f063ae0de442fdbceedb6d74869b`.
Public admit used UuidV7-like budget/loop ids. `retry=0`. Daemon pid 344759
on `127.0.0.1:48304`. Live `--append-system-prompt`
`frozen-system-task-prompt.txt` (211 UTF-8 bytes). Secret-shaped scan of Pi
launch stdout/stderr: 0 hits.

| Role | Task ref | O4 `lease_acquired` | Lifecycle | Verification | Acceptance |
|---|---|---:|---|---|---|
| warmup 1 (non-counted) | `task://personal/eval014-b0-c1-w1` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |
| warmup 2 (non-counted) | `task://personal/eval014-b0-c1-w2` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |
| warmup 3 (non-counted) | `task://personal/eval014-b0-c1-w3` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |
| counted Search | `task://personal/eval014-b0-c1-search` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |
| counted Read | `task://personal/eval014-b0-c1-read` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |

Private-candidate adapter skip lines remain on daemon.log (`expected_state_version`); they did not block the public Search/Read completions.

## B0 C1 P-arm (2026-08-20) — pass; retained

Broker `127.0.0.1:48404` pid 345123; SecretStore paths `["26"]`;
`secret_material_written: false`; Pi placeholder token only. Live
`--append-system-prompt` same frozen 211-byte file. Search hit
`failing-line`; Read returned both note lines. No daemon Task.

## B0 C1 fairness (2026-08-20) — pass

Live P/O `system_task_prompt_bytes` both 211. Checker `result: pass`,
`failed_axes: 0`. Wrapper `b0: true`. Nested checker `b0: false` means the
record is observability, not a counted sample.

## B0 C2a O-arm (2026-08-20) — pass; retained

Write warmups and counted Write `COMPLETED` / `ACCEPTANCE_GRANTED`, O4
`lease_acquired` 1. Counted Patch `task://personal/eval014-b0-c2a-patch`
`COMPLETED` / `ACCEPTANCE_GRANTED`, verification `passed`/`current`, preimage
SHA-256 `cb4ff53fe48499826134116581f605c9ed95cc37cfb3d0e42aac028b87c99c0f`.

## B0 C2a P-arm (2026-08-20) — pass; retained

Fixture writes `c2a-write\n`. Counted Patch used the same unified-diff
`input_b64` as O-arm; post-state `c2a-patch-v2\n`. Payload format is comparable.

## B0 C2b (2026-08-20) — split-score; retained

O-arm public unsealed remember returned **201** `remembered` with `memory_id`
present. Session-2 resume `not-run`. Skill bind `not-run`. P-arm launch exit 0
with the same frozen 211-byte `--append-system-prompt` and a WorkspaceRead of
fixture `procedure.txt`. These arms do not share a Memory/Skill tool set.

## B0 C2c (2026-08-20) — `not-run`

No frozen campaign-authorized default-off fault profile or original-key
injector on this EVAL. This is a capability gap, not a C1/C2a fairness fail.

## B0 C2d (2026-08-20) — split-score; retained

O-arm observed counted C2a Patch Task `task://personal/eval014-b0-c2a-patch`:
lifecycle `COMPLETED`, verification `passed`/`current`, acceptance current,
`reconcile_class=closed`. P-arm mechanical oracle returned `ANSWER: repaired\n`
(exit 0). Pure-Pi completion is not OS Task completion.

## B0 extras (2026-08-20) — pass

Evidence secret-shaped scan: 0 hits in 81 files (counts only). Frozen
`timeout_ms=120000`, `retry=0`, `max_agent_turn=8`. Never
`secret-tool search`/`lookup`.

## B1 C1 (2026-08-20) — pass; counted; retained

All five frozen B1 C1 seeds ran through `runLivePairedCell` with
campaign-injected `executeArm`. Fairness `pass` on each cell. `retry=0`.
`counted_sample: true`. `b0: false`.

| Index | Seed | Order | P | O |
|---:|---|---|---|---|
| 0 | `sha256:6fb944411a509a3e` | p,o | 0 | 0 |
| 1 | `sha256:e6204d7226dc77ee` | p,o | 0 | 0 |
| 2 | `sha256:4498037803872b19` | p,o | 0 | 0 |
| 3 | `sha256:cf3e9baa8ae123c5` | p,o | 0 | 0 |
| 4 | `sha256:c5de133432d56eb2` | o,p | 0 | 0 |

## B1 C2a (2026-08-20) — pass; counted; retained

All five frozen B1 C2a Write seeds. Same runner. Fairness `pass`. `retry=0`.

| Index | Seed | Order | P | O |
|---:|---|---|---|---|
| 0 | `sha256:e5552c6cc5b9f0c7` | o,p | 0 | 0 |
| 1 | `sha256:35a2f08e3ae9cbd7` | o,p | 0 | 0 |
| 2 | `sha256:188a06516064c71a` | o,p | 0 | 0 |
| 3 | `sha256:4005c50d8f419610` | p,o | 0 | 0 |
| 4 | `sha256:91d3e23bedf89158` | p,o | 0 | 0 |

B1 C2b/C2c/C2d: `not-run` (not comparable / no injector). B1 does not freeze B2
below 30.

## B2 N freeze (2026-08-20)

Formal B2 N = **30** paired seeds per comparable class (C1, C2a), from
`freeze.mjs` `c1-c2-b2-heldout-v1`. B1 n=5 completion 5/5 both classes does
not authorize shrinking N. Timeout 120000 ms, `retry=0`.

## B2 C1 (2026-08-20) — pass; counted; retained

30/30 frozen held-out C1 seeds. `runLivePairedCell` + campaign `executeArm`.
Fairness `pass` on each cell. Both arms `exit_code=0`, `timed_out=false`.
`counted_sample: true`. `b0: false`. `retry=0`. Evidence
`b2-c1-0.json` … `b2-c1-29.json`.

## B2 C2a (2026-08-20) — pass; counted; retained

30/30 frozen held-out C2a Write seeds. Same runner. Fairness `pass`.
`retry=0`. Evidence `b2-c2a-0.json` … `b2-c2a-29.json`.

The cell record schema is exit/timeout/fairness only. This campaign does not
have a frozen wall-clock or token denominator, so it reports completion
equality and does not compute OS overhead or Agent-benefit.

## Parent-plan remainder (2026-08-20) — `not-run`

C0 paired G/A, B3 faults, B4 concurrency, B5 soak (1 h / 8 h / 24 h), and
T/S/O/UJ extras: overlay skip or no frozen runner on this EVAL. 24 h default
deferred.

## Cleanup (2026-08-20) — pass

Guest route unchanged. Snapshot was not restored or deleted.
`B01-Clean-Linux-001` was not contacted. Closed EVAL roots left in place.

| Check | Result |
|---|---|
| campaign daemon `127.0.0.1:48304` pid 344759 | product `cognitive daemon stop` `action=stopped` (`stale_lock_removed=true`); lock absent |
| campaign broker `127.0.0.1:48404` pid 345123 | process gone; listener absent |
| listeners `48181` / `48284` / `48383` | untouched |
| EVAL-012 / EVAL-013 roots | untouched |
| campaign root | retained `0700` at `/home/hal9001/perfeval014-20260820` |
| SecretStore item `/26` | pre-clear suffixes `["26"]`; `secret-tool clear` on product triple; post-clear login `item_count=0` `item_suffixes=[]`; never `secret-tool search`/`lookup` |
| evidence redactor | 642 files, secret-shaped hits 0 |
| runtime redactor | naive `sk-[A-Za-z0-9]{10,}` matched 82 files; every token length 13 (regex floor); PEM / private-key BEGIN 0. Not treated as campaign key-length material. |

## Capability matrix (hypothesis / non-claim)

| Class | This freeze | Note |
|---|---|---|
| Public doctor / first conversation | ready | not a C1/C2 Task |
| C1 WorkspaceSearch P/O | **pass** | B0 + B1 5/5 + B2 30/30 counted; fairness 13/13 including live 211-byte system prompt |
| C2a WorkspaceWrite P/O | **pass** | B0 Write+Patch; B1/B2 counted Write 5/5 and 30/30 |
| C2b Memory/Skill | split-score | O remember 201; session-2/Skill `not-run`; P fixture Read |
| C2c Effect recovery | `not-run` | no frozen fault injector |
| C2d verified completion | split-score | O observed C2a Patch completion; P mechanical `ANSWER: repaired` |
| C0 paired G/A | `not-run` | no frozen live C0 executor |
| B3 / B4 / B5 | `not-run` | no frozen runners; 24 h default deferred |

## Evidence-ranked optimization priorities (hypothesis only)

1. If the next question is OS overhead, freeze wall-clock / token / oracle
   fields in campaign `executeArm` records. This EVAL's counted schema is
   completion/fairness/timeout only.
2. C2b session-2 resume and Skill bind need a frozen Skill package and a
   resume path that does not restart the campaign daemon.
3. C2c needs a frozen campaign-authorized default-off fault profile.
4. Do not treat 100% instrumented Search/Write completion as Agent-benefit
   or Gate evidence.

## Non-claims

A closed product train (P9-T09–T12) is not a performance result. B0
qualification pass is not B1. B1 5/5 is not B2. B1/B2 counted completion
equality is not Gate / release / Profile / B01 / Agent-benefit. Implementing
or using the live runner is not itself a latency result. Campaign closure
does not resume development.

## Unique next action

Campaign closed 2026-08-20. Do not reopen this freeze. Wait for an explicit
owner delivery instruction before claiming any implementation task.
