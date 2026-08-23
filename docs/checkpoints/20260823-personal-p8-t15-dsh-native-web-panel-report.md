# P8-T15 native dsh Web UI control panel — running report

Claim ceiling: `hypothesis`. Not Gate, release, Profile, B01, EVAL, or Agent-benefit.

Native panel ≠ Personal SPA. Personal UI is `http://127.0.0.1:<daemon>/ui/` (P7-T05).
Native dsh panel is `cognitive dsh web` → `dsh --profile web --no-open`, default
`http://127.0.0.1:3080`.

## Slices

| Slice | Status | Evidence |
|---|---|---|
| D01 CLI + helper + negatives | in-progress | `cognitive dsh web`; helper `--mode web`; loopback/dist fail-closed; Path B AKP without Workspace* admits |
| D02 linux-002 listen + GET `/` HTML | ready / running | guest copy `/home/hal9001/p8t15-2ba8103a/dsh` has `apps/web/dist/index.html` (title `DSH Local Build`); product CLI binary still pending jump-host fetch/build |
| D03 handbook / docs-sync | in-progress | bilingual operator pages drafted in D01 commit; fingerprints present |

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

- 2026-08-23 frontend dist **pass** on disposable copy `/home/hal9001/p8t15-2ba8103a/dsh` (rsync of `p8t11-e48517cb/dsh`, then `npx vite build` in `apps/web`). `dist/index.html` 672 bytes, title `DSH Local Build`. Pin file `528c682e061696f5a160f363f236ecbf53cbd006`. Did not mutate `p8t10` / live `dsh.json`.
- `GET /` through `cognitive dsh web` **not-run** until jump-host `cargo build -p admin-cli --bin cognitive` at the pushed revision (first `git fetch` failed HTTP/2 framing).
- No `:3080` listener yet. P7-T05 daemon PID 465376 / `:48681` left running.

### Required CI

Run `32636212950` at `2ba8103a` **fail**: `tools test` 2 failures — Delivery Slice status `not-started` is not in `{ready,in-progress,blocked,done,cancelled}`; lease claimed/heartbeat must be `YYYY-MM-DD / YYYY-MM-DD`; lease must not list `docs/plan/PARALLEL-LANES.md` as a writable path. Follow-up commit repairs bookkeeping only.

## Non-claims

- UI up is not Task completion.
- Path A remains measurement-only.
- No live Flash turn through the native panel is claimed in D01.
- Do not stop P7-T05 daemon PID 465376 / port 48681 unless this task started a replacement.

## Unique next action

Push bookkeeping repair, rebuild `cognitive` on jump host at the new exact revision, then on linux-002 run `cognitive dsh web --no-open --host 127.0.0.1 --port 3080` against the disposable dsh copy with dist.
