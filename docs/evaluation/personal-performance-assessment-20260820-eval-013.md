# PERSONAL-PERF-EVAL-013 running assessment

- Campaign: `PERSONAL-PERF-EVAL-013`
- Freeze branch: `evaluation/EVAL-013-freeze`
- Product pin: `6c415625`
- Preregistration: [20260820-personal-perf-eval-013-preregistration.md](../checkpoints/20260820-personal-perf-eval-013-preregistration.md)
- Claim ceiling: `hypothesis` / `not_reviewed`
- Independent reviewer: `not_reviewed`

Measurement-only. This report does not promote Gate, release, Profile, B01,
or Agent-benefit. EVAL-002 and EVAL-004 through EVAL-012 remain closed.

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
| B0 C2b P-arm | **pass** | WorkspaceRead of frozen `procedure.txt` (exit 0, `--append-system-prompt`); P does not use daemon Memory/Skill |
| B0 C2c O-arm | `not-run` | no frozen campaign-authorized default-off fault profile / original-key injector on this EVAL |
| B0 C2c P-arm | `not-run` | split-score fixture mutation; not opened after O `not-run` |
| B0 C2d O/P | `not-run` | next |

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

## Non-claims

A closed product train (P9-T09–T11) is not a B0 pass. Matching prompt bytes
and Patch payload format in instruments is not a counted sample.
