# PERSONAL-PERF-EVAL-013 running assessment

- Campaign: `PERSONAL-PERF-EVAL-013`
- Freeze branch: `evaluation/EVAL-013-freeze`
- Product pin: `6c415625`
- Lease: `lease/personal/EVAL-013/execution-plan-b0` (**closed** 2026-08-20)
- Preregistration: [20260820-personal-perf-eval-013-preregistration.md](../checkpoints/20260820-personal-perf-eval-013-preregistration.md)
- Claim ceiling: `hypothesis` / `not_reviewed`
- Independent reviewer: `not_reviewed`
- Document status: campaign **closed**. Measurement-only. Evaluation routing OFF.

Measurement-only. This report does not promote Gate, release, Profile, B01,
or Agent-benefit. EVAL-002 and EVAL-004 through EVAL-012 remain closed. Do not
append more cells on this freeze.

## Cell log (`TEST-REPORT-INCREMENTAL-01`)

| Cell | Result | Note |
|---|---|---|
| Freeze / preregistration | **pass** | pin `6c415625`; root `/home/hal9001/perfeval013-20260820`; daemon `48302` pid 336122; SecretStore `/25`; Pi `0.81.1`; doctor overall `ready`, `first_conversation_ready: true` |
| B0 fairness | **pass** (C1 axes) | live P/O both injected frozen-system-task-prompt.txt (211 UTF-8 bytes); 13/13 axes pass; `b0: true` for this C1 record. Full B0 still requires C2a+ |
| B0 C1 O-arm | **pass** | 3 Search warmups + counted Search + counted Read. All `COMPLETED` / `ACCEPTANCE_GRANTED`; verification `passed`/`current`; O4 `lease_acquired` 1. Live `--append-system-prompt` frozen file (211 UTF-8 bytes). `retry=0`. First admit `eval013-b0-C1-warmup-1` 409 retained unused; samples used fresh lowercase task refs. |
| B0 C1 P-arm | **pass** | broker `48402` pid 339769; SecretStore paths `["25"]`; `secret_material_written: false`; Pi placeholder token only. 3 Search warmups + counted Search + counted Read; Search hit `failing-line`; Read returned both note lines. Live `--append-system-prompt`. No daemon Task. `retry=0` |
| B0 C2a Write O/P | **pass** | O: 3 Write warmups + counted Write `COMPLETED`/`ACCEPTANCE_GRANTED`, O4 lease 1. P: fixture writes `c2a-write\\n`. Unified live `--append-system-prompt`. `retry=0` |
| B0 C2a Patch O/P | **pass** | O counted Patch `COMPLETED`/`ACCEPTANCE_GRANTED`, verification `passed`/`current`, lease 1, preimage `cb4ff53fe4…` (P2-T38 post-state). P counted Patch unified diff `c2a-patch-v1` → `c2a-patch-v2\\n` (P9-T11 comparable payload). |
| B0 C2b O-arm | **partial** | public unsealed `POST /management/resource/v1/memory/remember` **201** `remembered` (`memory_id` present). Session-2 resume `not-run` (would restart campaign daemon). Skill bind `not-run` (no frozen Skill package) |
| B0 C2b P-arm | **pass** | exit 0 with live `--append-system-prompt`; stdout `Done.` (6 bytes, no procedure echo). Split-score: P does not use daemon Memory/Skill |
| B0 C2c O-arm | `not-run` | no frozen campaign-authorized default-off fault profile / original-key injector on this EVAL |
| B0 C2c P-arm | `not-run` | split-score fixture mutation; not opened after O `not-run` |
| B0 C2d O-arm | **pass** | split-score observe of counted C2a Patch: `COMPLETED` / verification `passed`/`current` / acceptance current / `reconcile_class=closed`. Pure-Pi completion is not OS Task completion |
| B0 C2d P-arm | **pass** | mechanical oracle stdout `ANSWER: repaired\n` (exit 0, `--append-system-prompt`). Split-score vs O |
| B0 extras | **pass** | secret-shaped hits 0/86 evidence files; `timeout_ms=120000`; `retry=0`; `max_agent_turn=8`; never `secret-tool search`/`lookup` |
| B0 overall | **pass** | C1/C2a comparable + C1 fairness 13/13 + secret/timeout. C2b–d remain split-score / capability-gap |
| B1 C1/C2 paired | `not-run` | `paired-runner.mjs` is dry-run fairness only; execution plan §2.5 forbids cobbling B0 shell into a formal paired campaign. Frozen B1 seeds unused |
| B2 C1/C2 paired | `not-run` | B1 not opened; no B2 N freeze |
| C0 paired (G/A families) | `not-run` | overlay skip + no frozen live C0 paired executor on this freeze |
| B3 faults | `not-run` | no frozen campaign-authorized fault runner on this EVAL |
| B4 concurrency | `not-run` | no frozen concurrency runner |
| B5 1 h / 8 h / 24 h | `not-run` | no frozen soak runner; 1 h trigger not met; 24 h default deferred |
| T/S/O/UJ extras | `not-run` | overlay skip / missing runner |
| Cleanup | **pass** | daemon `48302` pid 336122 `action=stopped` (`stale_lock_removed=true`); broker pid 339769 gone; listeners `48302`/`48402` absent; residue `48181`/`48284`/`48383` untouched; SecretStore `/25` cleared; login `item_count=0`; evidence secret-shaped 0/87; runtime naive `sk-` hits are all length 13 (regex floor), PEM 0 |

## Guest identity (2026-08-20) — pass

Registered route: `wuz@192.168.1.2` (`hal9000`) `virsh -c qemu:///system`,
then ProxyJump `hal9001@192.168.123.160`. Domain `B01-Desktop-Linux-002`
running (id 35). MAC `52:54:00:33:27:c1`. Ubuntu 24.04.4. uid 1000. Session
bus present. `B01-Clean-Linux-001` shut off. Closed EVAL roots left in place
and unused. Residue listeners `48181` / `48284` / `48383` untouched.

## Source freeze (2026-08-20) — pass

`git archive` of `6c415625`: 15,134,720 bytes, 1596 members, 0 `.git/`,
SHA-256 `d06923c04febece5d7175cadff54a366df07cce031bf8249dc0aaeff7c92e06a`.
Copies used `scp` (no PowerShell pipes). Host `DEV-LINUX-NATIVE-01` built
`--locked --release` `kernel-server` / `admin-cli` / `pi-agent-adapter` and
`pnpm --filter @cognitiveos/pi-cognitiveos run build`. glibc-only `ldd`.

## Secret bind / doctor (2026-08-20) — pass

Login collection was empty. Owner-designated key imported through
`cognitive init --api-key-file -` into new SecretStore item `/25`.
`secret_material_written: true`, `secret_ref_redacted: true`,
`selected_model: deepseek-v4-flash`. Guest temp shredded. No
`secret-tool search`/`lookup`, no `provider.json` copy. Pi configured with
absolute `cli.js` and Extension `index.js`. Daemon started on
`127.0.0.1:48302`. Doctor overall `ready`, `first_conversation_ready: true`.
This is conversation-shell readiness, not a C1/C2 Task.

## B0 C1 O-arm (2026-08-20) — pass; retained

Frozen C1 fixture `note.txt` SHA-256
`4fb26b79e8de937c59f203f9274d76998db1f063ae0de442fdbceedb6d74869b`.
Public admit used UuidV7-like budget/loop ids. `retry=0`. Daemon pid 336122
on `127.0.0.1:48302`. Live
`--append-system-prompt` `/…/frozen-system-task-prompt.txt` (211 UTF-8 bytes).
Secret-shaped scan of Pi launch stdout/stderr: 0 hits.

| Role | Task ref | O4 `lease_acquired` | Lifecycle | Verification | Acceptance |
|---|---|---:|---|---|---|
| warmup 1 (non-counted) | `task://personal/eval013-b0-c1-w1` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |
| warmup 2 (non-counted) | `task://personal/eval013-b0-c1-w2` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |
| warmup 3 (non-counted) | `task://personal/eval013-b0-c1-w3` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |
| counted Search | `task://personal/eval013-b0-c1-search` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |
| counted Read | `task://personal/eval013-b0-c1-read` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |

A first admit of `task://personal/eval013-b0-C1-warmup-1` returned 409 and is
retained unused. Later admits on fresh lowercase refs succeeded. Private-candidate
adapter skip lines remain on that unused row (`expected_state_version`); they did
not block the public Search/Read completions.

## B0 C1 P-arm (2026-08-20) — pass; retained

Broker `127.0.0.1:48402` pid 339769; SecretStore paths `["25"]`;
`secret_material_written: false`; Pi placeholder token only. Live
`--append-system-prompt` same frozen 211-byte file. Search hit
`failing-line`; Read returned both note lines. No daemon Task.

## B0 C1 fairness (2026-08-20) — pass

Live P/O `system_task_prompt_bytes` both 211. Checker `result: pass`,
`failed_axes: 0`. Wrapper `b0: true`. Nested checker `b0: false` means the
record is observability, not a counted sample.

## B0 C2a O-arm (2026-08-20) — pass; retained

Write warmups and counted Write `COMPLETED` / `ACCEPTANCE_GRANTED`, O4
`lease_acquired` 1. Counted Patch `task://personal/eval013-b0-c2a-patch`
`COMPLETED` / `ACCEPTANCE_GRANTED`, verification `passed`/`current`, preimage
SHA-256 `cb4ff53fe48499826134116581f605c9ed95cc37cfb3d0e42aac028b87c99c0f`.
This closes the EVAL-012 O-arm Patch fail (`fixed post-state is unavailable`)
on this new freeze.

## B0 C2a P-arm (2026-08-20) — pass; retained

Fixture writes `c2a-write\n`. Counted Patch used the same unified-diff
`input_b64` as O-arm; post-state `c2a-patch-v2\n`. Payload format is comparable.

## B0 C2b (2026-08-20) — split-score; retained

O-arm public unsealed remember returned **201** `remembered` with `memory_id`
present. Session-2 resume `not-run` (would restart the campaign daemon). Skill
bind `not-run` (no frozen Skill package). P-arm launch exit 0 with the same
frozen 211-byte `--append-system-prompt`; stdout was `Done.` (6 bytes) and did
not echo `procedure.txt`. These arms do not share a Memory/Skill tool set.

## B0 C2c (2026-08-20) — `not-run`

No frozen campaign-authorized default-off fault profile or original-key
injector on this EVAL. P-arm fixture mutation was not opened after the O-arm
gap. This is a capability gap, not a C1/C2a fairness fail.

## B0 C2d (2026-08-20) — split-score; retained

O-arm observed counted C2a Patch Task `task://personal/eval013-b0-c2a-patch`:
lifecycle `COMPLETED`, verification `passed`/`current`, acceptance current,
`reconcile_class=closed`. P-arm mechanical oracle returned `ANSWER: repaired\n`
(exit 0). Pure-Pi completion is not OS Task completion.

## B0 extras (2026-08-20) — pass

Evidence secret-shaped scan: 0 hits in 86 files (counts only; matching bodies
not copied). Frozen `timeout_ms=120000`, `retry=0`, `max_agent_turn=8`. Counted
C1/C2a/C2d samples completed without timeout. Tool-equivalence for comparable
classes: C1 fairness 13/13 including `visible_tool_set_schema`; C2a Patch
`input_b64` identical across arms. Never `secret-tool search`/`lookup`.

## Parent-plan remainder (2026-08-20) — `not-run`

B0 fairness holds for C1/C2a, but a live B1/B2 paired executor is not frozen:
`tools/personal/c1-c2-paired/paired-runner.mjs` emits dry-run fairness only.
Execution plan §2.5 forbids stitching campaign-local B0 shell into a formal
paired campaign. Frozen B1/B2 seeds were not consumed. Overlay skip plus
missing runners: C0, B3, B4, B5 (1 h / 8 h / 24 h), T/S/O/UJ extras.

## Cleanup (2026-08-20) — pass

Guest route unchanged. Snapshot was not restored or deleted.
`B01-Clean-Linux-001` was not contacted. Closed EVAL roots left in place.

| Check | Result |
|---|---|
| campaign daemon `127.0.0.1:48302` pid 336122 | product `cognitive daemon stop` `action=stopped` (`stale_lock_removed=true`); lock absent |
| campaign broker `127.0.0.1:48402` pid 339769 | process gone; listener absent |
| listeners `48181` / `48284` / `48383` | untouched |
| EVAL-012 root | untouched at `/home/hal9001/perfeval012-20260820` |
| campaign root | retained `0700` at `/home/hal9001/perfeval013-20260820` |
| SecretStore item `/25` | pre-clear login suffixes `["25"]` only; `secret-tool clear` on product triple (`application=cognitiveos-personal`, `provider=deepseek`, `purpose=provider-api-key`); post-clear login `item_count=0` `item_suffixes=[]`; never `secret-tool search`/`lookup` |
| evidence redactor | 87 files, secret-shaped hits 0 |
| runtime redactor | naive `sk-[A-Za-z0-9]{10,}` matched 14 files; every token length 13 (regex floor); PEM / private-key BEGIN 0. Not treated as campaign key-length material. |

## Capability matrix (hypothesis / non-claim)

| Class | This freeze | Note |
|---|---|---|
| Public doctor / first conversation | ready | not a C1/C2 Task |
| C1 WorkspaceSearch/Read P/O | **pass** | comparable; fairness 13/13 including live 211-byte system prompt |
| C2a Write/Patch P/O | **pass** | O Patch verification `passed`/`current` (closes EVAL-012 post-state fail); P Patch same unified-diff `input_b64` |
| C2b Memory/Skill | split-score | O remember 201; session-2/Skill `not-run`; P stdout `Done.` |
| C2c Effect recovery | `not-run` | no frozen fault injector |
| C2d verified completion | split-score | O observed C2a Patch completion; P mechanical `ANSWER: repaired` |
| C0 paired G/A | `not-run` | no frozen live C0 executor |
| B1/B2 C1/C2 paired | `not-run` | §2.5; dry-run runner only |
| B3 / B4 / B5 | `not-run` | no frozen runners; 24 h default deferred |

## Evidence-ranked optimization priorities (hypothesis only)

1. Freeze a **live** campaign-only paired B1/B2 executor (output schema, arm order, cleanup) so C1/C2a qualification can become confirmatory without stitching shell. `paired-runner.mjs` today is dry-run fairness only.
2. C2b session-2 resume and Skill bind need a frozen Skill package and a resume path that does not restart the campaign daemon.
3. C2c needs a frozen campaign-authorized default-off fault profile on this guest; do not invent faults on `B01-Desktop-Linux-002`.
4. C2d split-score stays until an independent oracle closes an OS Task on both arms with the same tool set.
5. Do not treat C1/C2a B0 pass as Agent-benefit or Gate evidence.

## Non-claims

A closed product train (P9-T09–T11) is not a B0 pass. Matching prompt bytes
and Patch payload format in instruments is not a counted sample. B0
qualification pass is not B1/B2, not C0 paired performance, not Gate / release
/ Profile / B01 / Agent-benefit, and not a reason to edit product code.
Campaign closure does not resume development.
