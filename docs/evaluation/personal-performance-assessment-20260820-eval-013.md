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
| B0 fairness | not-run | requires live P/O observation; both arms must inject frozen prompt (211 UTF-8 bytes) |
| B0 C1 O-arm | **pass** | 3 Search warmups + counted Search + counted Read. All `COMPLETED` / `ACCEPTANCE_GRANTED`; verification `passed`/`current`; O4 `lease_acquired` 1. Live `--append-system-prompt` frozen file (211 UTF-8 bytes). `retry=0`. First admit `eval013-b0-C1-warmup-1` 409 retained unused; samples used fresh lowercase task refs. |
| B0 C1 P-arm | not-run | next; broker `48402` |
| B0 C2a Write O/P | not-run | |
| B0 C2a Patch O/P | not-run | P-arm unified-diff is product-closed on `main`; live sample still required |
| B1–B5, C0, extras | not-run | forbidden until B0 pass, or `not-run` if no runner |

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

## Non-claims

A closed product train (P9-T09–T11) is not a B0 pass. Matching prompt bytes
and Patch payload format in instruments is not a counted sample.
