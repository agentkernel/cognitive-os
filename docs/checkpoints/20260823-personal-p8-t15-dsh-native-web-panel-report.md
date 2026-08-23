# P8-T15 native dsh Web UI control panel — running report

Claim ceiling: `hypothesis`. Not Gate, release, Profile, B01, EVAL, or Agent-benefit.

Native panel ≠ Personal SPA. Personal UI is `http://127.0.0.1:<daemon>/ui/` (P7-T05).
Native dsh panel is `cognitive dsh web` → `dsh --profile web --no-open`, default
`http://127.0.0.1:3080`.

Exact product revision on jump/guest binary: `0376e94238d8871ccaa9e8fd3dcab5e49c9cb4c9`.
dsh pin unchanged: `528c682e061696f5a160f363f236ecbf53cbd006`.
Draft PR: https://github.com/agentkernel/cognitive-os/pull/265

## Slices

| Slice | Status | Evidence |
|---|---|---|
| D01 CLI + helper + negatives | done | Jump-host `cargo test -p admin-cli --locked dsh` **9/9**; fmt; Clippy `-D warnings`. Node preflight 3/3. |
| D02 linux-002 listen + GET `/` HTML | done | Listen + HTML **pass**. Post-panel Path B with P8-T15 adapter **pass** (`deepseek-v4-flash`, `assistant_ok`, Workspace* `COMPLETED`). Prior `--print` fail was the restored p8t10 helper reading the Pi binding. |
| D03 handbook / docs-sync | done | Bilingual operator pages + generated `cli-cognitive`; fingerprints refreshed with CLI commits. |

## Validation log

### 2026-08-23 — D01 design (pass)

- Product launch path remains Path B (`dsh → AKP → daemon → Flash`) via SecretStore.
- Headless `cognitive dsh launch --print` stays `--profile headless`.
- Web command always `--no-open`, default host `127.0.0.1` port `3080`.
- `--host 0.0.0.0` / `::` refused (native webserver has no TLS/auth).
- Missing `apps/web/dist/index.html` fails closed with operator build hint.

### Node preflight unit

Local Windows (no Rust link): `node --test packages/dsh-akp-adapter/scripts/dsh-web-preflight.test.mjs` **3/3 pass**.

### Admin-cli parse / prepare / Clippy

Jump-host `DEV-LINUX-NATIVE-01` at `0376e942`: focused `dsh` tests **9/9**, `cargo fmt -p admin-cli -- --check`, `cargo clippy -p admin-cli --all-targets --locked -- -D warnings`. Local Windows GNU link **not-run**.

### linux-002 native panel (D02)

Guest: `b01guest` via `wuz@192.168.1.2`. Did not stop P7-T05 `kernel-server` PID **465376** / `127.0.0.1:48681`. Did not kill hung P8-T10 helper PID **430838**.

Disposable trees (not `p8t10` overlay):

- dsh copy with Vite dist: `/home/hal9001/p8t15-2ba8103a/dsh` (`npx vite build` in `apps/web`; `index.html` title `DSH Local Build`)
- adapter overlay: `/home/hal9001/p8t15-0376e942/adapter` (p8t10 plugin bundle + `0376e942` scripts)
- cognitive binary: `/home/hal9001/p8t15-0376e942/bin/cognitive`

Temporarily reconfigured `dsh.json` then restored to `/home/hal9001/p8t10-a17edfad/{dsh,adapter}`.

Negatives (new binary, restored after):

- `--host 0.0.0.0` → rc **2**, message refuses wildcard bind
- missing `apps/web/dist/index.html` → rc **1**, fail-closed

Positive panel:

- `ss`: `127.0.0.1:3080` `node` PID **479898** (`apps/cli/lib/bin.js --profile web --no-open --host 127.0.0.1 --port 3080`)
- helper PID **479884** `dsh-real-process.mjs --mode web` against daemon `:48681`
- `GET /` → **200** `text/html`; `<title>DSH Local Build`
- `GET /assets/index-ClqxG24t.js` → **200** `text/javascript` 399361 bytes
- `GET /manifest.webmanifest` → **200** `application/manifest+json`
- `cognitive dsh status`: `state=ACTIVE`, `process_alive=true`, `process_id=479898`, session `dsh-web-process` ACTIVE, `dsh_response_is_not_task_completion=true`
- helper log: `selected_model=deepseek-v4-flash` at web start

Cleanup: killed **479884/479898** only; `:3080` closed; `:48681` remained; removed disposable `runtime/cognitiveos/dsh-web-home` without reading `.credentials.yaml`.

### Headless Path B `--print` after panel (fail)

Two runs of `cognitive dsh launch --print --path b` with the new binary against restored p8t10 paths:

- `selected-model.json` on disk still `deepseek-v4-flash`
- helper summary `selected_model: grok-4.6`
- `dsh_exit=1`, `assistant_preview_bytes=0`, stderr `INVALID_REQUEST: provider proxy request was not completed`
- Workspace search/write `COMPLETED`; read left `ACTIVE`
- Daemon PID 465376 still listening

Hypothesis (not proven): native web session changed the live daemon selected model to `grok-4.6` while the file snapshot stayed Flash. Not claimed as Agent-benefit; not claimed as a Flash-through-panel turn.

### 2026-08-23 — grok-4.6 diagnosis + Path B retry (pass)

Redacted facts on the same Personal runtime (`/home/hal9001/p8t13-owner-ops/runtime`) against P7-T05 daemon PID **465376** / `127.0.0.1:48681`. No keys read or printed. SQLite not hand-edited. Pi binding left unchanged.

| Surface | Model | Digest / agent |
|---|---|---|
| Disk `selected-model.json` | `deepseek-v4-flash` | file snapshot (`fnv1a64:2da8aee1…`) |
| `GET /provider/v1/dsh/selected-model` | `deepseek-v4-flash` | `binding` / `agent://personal/dsh` |
| `GET /provider/v1/selected-model` | `grok-4.6` | `binding` / `agent://personal/pi` |
| `cognitive agent binding list` | dsh `deepseek-v4-flash` rev 2; pi `grok-4.6` rev 1 | same account `acct-01a02dee-…` (`d10-sidebar-live`, `openai_compatible`) |

`grok-4.6` is the independent **P7-T05 UI Pi binding**, not a dsh/file mutation by `cognitive dsh web`. Web start on this task's helper already logged `selected_model=deepseek-v4-flash`. No Flash restore was required; dsh was already the intended Path B model.

The earlier post-panel `--print` **fail** used restored **p8t10** adapter scripts, which still `GET /provider/v1/selected-model` and point `llm-deepseek` at `POST /provider/v1/chat/completions` (Pi plane). That helper therefore logged Pi `grok-4.6` and the DeepSeek-compatible account rejected it (`INVALID_REQUEST`). P8-T15 helper uses `/provider/v1/dsh/selected-model` and `/provider/v1/dsh/chat/completions`.

`:3080` was already closed (no native web helper/node). Cleared leftover `CRASHED` dsh runtime via `POST /personal/dsh/runtime` `op=clear` → `INACTIVE` / 0 sessions. Did not kill 465376 or hung 430838.

Retry cell (P8-T15 adapter `/home/hal9001/p8t15-0376e942/adapter` + p8t10 dsh tree; binary `0376e942`):

- `cognitive dsh launch --print --path b` **pass** (`pathb_rc=0`)
- `selected_model=deepseek-v4-flash`
- `dsh_exit=0`, `assistant_ok=true`, `assistant_preview_bytes=95`, `assistant_is_pong=false`
- `elapsed_ms=12270`, `ttft_ms=11925`, `cli_mode=tsx-source`
- Workspace read/search/write all `COMPLETED`
- Restored `dsh.json` to `/home/hal9001/p8t10-a17edfad/{dsh,adapter}`
- Daemon 465376 still listening; dsh status `INACTIVE`

Not a P8-T15 product defect: launching the native panel does not clobber the daemon Path B model. No code change and no focused negative added. Not claimed as Agent-benefit.

### Required CI

- `32636212950` at `2ba8103a` **fail** (slice status `not-started`; lease heartbeat/date; lease owned `PARALLEL-LANES.md`).
- Bookkeeping repair `92e00b74`. rustfmt+fingerprints `b233fcc3`. `DEFAULT_WEB_PORT` Clippy fix `0376e942`.
- `32637705326` at `0376e942`: Ubuntu verify **pass**; Windows superseded by later HEAD run.
- `32638483500` at `bbcbd118`: Ubuntu verify **pass**; Windows verify **pending** at this cell.

## Operator start (linux-002, after overlay)

```
/home/hal9001/p8t15-0376e942/bin/cognitive dsh configure \
  --runtime-root /home/hal9001/p8t13-owner-ops/runtime \
  --dsh-root /home/hal9001/p8t15-2ba8103a/dsh \
  --adapter-root /home/hal9001/p8t15-0376e942/adapter \
  --revision 528c682e061696f5a160f363f236ecbf53cbd006
/home/hal9001/p8t15-0376e942/bin/cognitive dsh web \
  --runtime-root /home/hal9001/p8t13-owner-ops/runtime \
  --host 127.0.0.1 --port 3080 --no-open
# open http://127.0.0.1:3080  (SSH/headless: do not --open)
# restore dsh.json to p8t10 paths when finished
```

`cognitive dsh web` prints JSON and returns; the helper/node pair keep listening.

## Non-claims

- UI up is not Task completion.
- Path A remains measurement-only.
- No live Flash turn through the native panel is claimed.
- Headless Path B after the panel with the **P8-T15** helper is **pass** on Flash. The earlier fail was the restored p8t10 helper reading the Pi `grok-4.6` binding.
- Do not stop P7-T05 daemon PID 465376 / port 48681 unless this task started a replacement.

## Unique next action

Wait for PR 265 Windows/required-ci on `32638483500` at `bbcbd118` (Ubuntu already pass). If Windows or docs-head fails, repair on this branch and push. Keep Draft. Do not auto-claim P6 / P7-T06 / P7-T07.
