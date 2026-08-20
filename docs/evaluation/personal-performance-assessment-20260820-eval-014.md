# PERSONAL-PERF-EVAL-014 running assessment

- Campaign: `PERSONAL-PERF-EVAL-014`
- Freeze branch: `evaluation/EVAL-014-freeze`
- Product pin: `adc40499`
- Lease: `lease/personal/EVAL-014/execution-plan-b0`
- Preregistration: [20260820-personal-perf-eval-014-preregistration.md](../checkpoints/20260820-personal-perf-eval-014-preregistration.md)
- Claim ceiling: `hypothesis` / `not_reviewed`
- Independent reviewer: `not_reviewed`
- Document status: **active**. Measurement-only.

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
| B2 C1/C2 paired | `not-run` | running after N freeze |
| B2 C1/C2 paired | `not-run` | after B1 |
| C0 / B3–B5 / T/S extras | `not-run` | overlay skip or missing runner |
| Cleanup | `not-run` | stop `48304`/`48404` only |

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

## Unique next action

B2 C1/C2a N=30 via `runLivePairedCell`. Then parent-plan remainder (`not-run`
where runners are missing) and cleanup of `48304` / `48404` / SecretStore
`/26`. Claim ceiling `hypothesis`.

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
