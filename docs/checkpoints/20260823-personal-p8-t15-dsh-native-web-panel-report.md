# P8-T15 native dsh Web UI control panel — running report

Claim ceiling: `hypothesis`. Not Gate, release, Profile, B01, EVAL, or Agent-benefit.

Native panel ≠ Personal SPA. Personal UI is `http://127.0.0.1:<daemon>/ui/` (P7-T05).
Native dsh panel is `cognitive dsh web` → `dsh --profile web --no-open`, default
`http://127.0.0.1:3080`.

## Slices

| Slice | Status | Evidence |
|---|---|---|
| D01 CLI + helper + negatives | in-progress | `cognitive dsh web`; helper `--mode web`; loopback/dist fail-closed; Path B AKP without Workspace* admits |
| D02 linux-002 listen + GET `/` HTML | not-started | requires pushed exact revision |
| D03 handbook / docs-sync | in-progress | bilingual operator pages drafted; fingerprints/generator pending |

## Validation log

### 2026-08-23 — D01 design (pass, local notes)

- Product launch path remains Path B (`dsh → AKP → daemon → Flash`) via SecretStore.
- Headless `cognitive dsh launch --print` stays `--profile headless`.
- Web command always `--no-open`, default host `127.0.0.1` port `3080`.
- `--host 0.0.0.0` / `::` refused (native webserver has no TLS/auth).
- Missing `apps/web/dist/index.html` fails closed with operator build hint.
- Pin remains `528c682e061696f5a160f363f236ecbf53cbd006` unless a later slice proves a web-artifact pin bump is required.

### Node preflight unit

2026-08-23 local Windows (no Rust link): `node --test packages/dsh-akp-adapter/scripts/dsh-web-preflight.test.mjs` **3/3 pass** (loopback/`0.0.0.0`, port, missing dist).

### Admin-cli parse / prepare

Recorded on exact-revision Linux/CI (`RUST-LINK-DEV-WIN-GNU-01`: local Windows GNU linking is not-run).

### linux-002 usability

not-run until D01 is committed and pushed.

## Non-claims

- UI up is not Task completion.
- Path A remains measurement-only.
- No live Flash turn through the native panel is claimed in D01.
- Do not stop P7-T05 daemon PID 465376 / port 48681 unless this task started a replacement.

## Unique next action

Finish D01 tests, docs-sync, checkpoint commit/push, Draft PR, then linux-002 D02.
