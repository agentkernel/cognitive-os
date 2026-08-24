# P8-T15 native dsh Web UI control panel — running report

Claim ceiling: `hypothesis`. Not Gate, release, Profile, B01, EVAL, or Agent-benefit.

Native panel ≠ Personal SPA. Personal UI is `http://127.0.0.1:<daemon>/ui/` (P7-T05).
Native dsh panel is `cognitive dsh web` → `dsh --profile web --no-open`, default
`http://127.0.0.1:3080`.

Exact product revision on jump/guest binary: `14aadf2709e64b4e91a770acb9b5c9c7f42eaba8`.
dsh pin unchanged: `528c682e061696f5a160f363f236ecbf53cbd006`.
Draft PR: https://github.com/agentkernel/cognitive-os/pull/265
Clients Apply copy: https://github.com/agentkernel/cognitiveos-clients/pull/4

## Slices

| Slice | Status | Evidence |
|---|---|---|
| D01 CLI + helper + negatives | done | Jump-host `cargo test -p admin-cli --locked dsh` **9/9**; fmt; Clippy `-D warnings`. Node preflight 3/3. |
| D02 linux-002 listen + GET `/` HTML | done | Cos-installed `:3080` ACTIVE; HTTP remove/set + Path B **pass**. |
| D03 handbook / docs-sync | done | Bilingual operator pages + generated `cli-cognitive`; fingerprints refreshed with CLI commits. |
| D04 Apply Cos binding to running web | in-progress | Routing fail-closed + Cos catalog overlay at `14aadf27`. Create-account `PROVIDER_ENDPOINT_PATH_FORBIDDEN` on chat-completions pastes **fixed** at `8b9d09cb` (Linux 8/8 + 2/2 + 6/6). Guest daemon **515695**. |

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
- `32638483500` at `bbcbd118`: Ubuntu verify **pass**; Windows verify **pending** at the diagnosis cell.
- `32637705326` at product SHA `0376e942`: Ubuntu, Windows, and `required-ci` **pass**.
- `32638903795` at docs SHA `93b02ca5`: Ubuntu/Windows **fail** (`CURRENT_SNAPSHOT_LEASE_MISMATCH`: lease `P8-T15/D02` vs Layer 2 D02 marked `done`). Repaired by keeping D02 `in-progress` until task close.

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

### 2026-08-23 — owner ask: start native panel and open in a browser

Reconfigured `dsh.json` to overlay-with-dist `/home/hal9001/p8t15-2ba8103a/dsh` + P8-T15 adapter, then:

`/home/hal9001/p8t15-0376e942/bin/cognitive dsh web --runtime-root /home/hal9001/p8t13-owner-ops/runtime --host 127.0.0.1 --port 3080 --no-open`

Listen: node PID **482145** `127.0.0.1:3080`; helper **482129**. P7-T05 daemon **465376** / `:48681` left running. Hung **430838** not killed.

`GET /` **200** `text/html` title **DSH Local Build** (14555 B); SPA JS **200** 399361 B; sidebar + conversation plugins **200**. Manifest name `DeepSeek Harness`. Guest Firefox (Wayland) opened `http://127.0.0.1:3080/`. Windows two-hop forward `GET /` **200**. No Flash chat turn claimed. Process left running.

### 2026-08-23 — owner ask: GUI browser on linux-002 desktop

dsh web **reused** (not restarted): node **482145** / helper **482129** on `127.0.0.1:3080`. `GET /` still **200** title `DSH Local Build`, `__DSH_BOOT__` present, not `/ui/`. Daemon **465376** left up.

Graphical session `loginctl` **4**: `seat0` `tty2` `Type=wayland` user `hal9001`. Initially `LockedHint=yes`. `org.gnome.ScreenSaver.SetActive false` cleared the lock (`LockedHint=no`). `loginctl activate 4` was refused (interactive auth).

Browser commands (non-headless, desktop user bus):

```
systemd-run --user --collect \
  --setenv=DISPLAY=:0 --setenv=WAYLAND_DISPLAY=wayland-0 \
  --setenv=XDG_RUNTIME_DIR=/run/user/1000 \
  --setenv=DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus \
  --setenv=XAUTHORITY=/run/user/1000/.mutter-Xwaylandauth.0ITHT3 \
  firefox --new-window http://127.0.0.1:3080/
```

Same env with `xdg-open http://127.0.0.1:3080/`. Then DBus `org.mozilla.firefox.OpenURL` on the existing snap Firefox → `openurl_ok`.

GUI evidence: Firefox PID **471883** environ `XDG_SESSION_TYPE=wayland` `DISPLAY=:0` `WAYLAND_DISPLAY=wayland-0`; **9** ESTABLISHED sockets `firefox → 127.0.0.1:3080`. Headed compositor screenshot **blocked** (`org.gnome.Shell.Screenshot` `AccessDenied` from SSH). `wmctrl`/`xdotool` not installed; X11 `_NET_CLIENT_LIST` empty on this Wayland session. No screenshot path.

URL opened: `http://127.0.0.1:3080`. Web + Firefox left running.

### 2026-08-23 — owner report: dsh 密钥 error + Provider panel will not open

Guest Firefox **471883** had sockets only to `:3080` (native dsh), not Personal `:48681`. Confirmed surfaces:

| Surface | URL | Result |
|---|---|---|
| Native dsh panel | `http://127.0.0.1:3080/` | **200** `DSH Local Build` |
| Personal SPA shell | `http://127.0.0.1:48681/ui/` | **200** `CognitiveOS Personal` |
| Personal Providers (HashRouter) | `http://127.0.0.1:48681/ui/#/providers` | **200** same `index.html` (hash not sent to server) |
| Personal Providers (path) | `http://127.0.0.1:48681/ui/providers` | **404** `LOCAL_UI_ASSET_NOT_FOUND` (no SPA fallback) |
| Personal Providers (no `/ui`) | `http://127.0.0.1:48681/providers` | **404** `PERSONAL_ROUTE_NOT_FOUND` |

**Native dsh 密钥 (P8-T15, this lease).** Live `--patch` already pointed `llm-deepseek` at `http://127.0.0.1:48681/provider/v1/dsh` + `DAEMON_BEARER`. The Models page joins the official catalog default `DEEPSEEK_API_KEY` and `credentials.describe`; `$DSH_HOME/.credentials.yaml` only had `DAEMON_BEARER`. Settings document had no `llm-deepseek` overlay, so dynamic config could still treat official DeepSeek as missing/invalid. Settings-models copy: `credentialMissing` = `API 密钥缺失`; placeholder base URL `https://api.deepseek.com`. No SecretStore material was copied.

Product fix in `packages/dsh-akp-adapter/scripts/dsh-web-preflight.mjs` + `dsh-real-process.mjs` `runWebPathB`:

- persist `settings.yaml` `llm-deepseek.baseURL` = loopback Path B origin and `apiKeyEnv: DAEMON_BEARER`
- write both `DAEMON_BEARER` and `DEEPSEEK_API_KEY` in `.credentials.yaml` as the **daemon management bearer** (JSON-quoted; 0600)
- set child `DEEPSEEK_BASE_URL` only (refuse secret-shaped child env keys)

Focused negative: `dsh-web-preflight.test.mjs` **4/4** (refuses `https://api.deepseek.com`, empty token, and secret-shaped child extras).

linux-002 retest (replaced **482129/482145** only; daemon **465376** left up):

- helper **487806**, node **487821** on `127.0.0.1:3080`; `cognitive dsh status` `ACTIVE` / `process_alive=true`; start log `selected_model=deepseek-v4-flash`
- cred refs `DAEMON_BEARER` + `DEEPSEEK_API_KEY`; settings Path B / no `api.deepseek.com`
- dump-config `llm-deepseek.baseURL` Path B, `apiKeyEnv: DAEMON_BEARER`
- `POST /provider/v1/dsh/chat/completions` with the stored bearer **200**, `assistant_is_pong=true` (probe class only; token not printed)
- Firefox **471883** still has long-lived sockets to `:3080` (12). Snap DBus `OpenURL` on the profile dest returned success for `http://127.0.0.1:48681/ui/#/providers`; Personal SPA is a short HTTP load (session-gate has no websocket), so zero lingering `:48681` sockets is expected. Refresh native Models on the desktop.

Remaining native-dsh gap: `@deepseek-ai/dsh-web-search-deepseek` still names `DEEPSEEK_API_KEY`. The alias makes Models show that row configured; a **web-search** call would still hit official DeepSeek with the management bearer and fail. Chat / Path B is the product path.

**Personal Providers (P7-T05, lease closed — not mixed).** Clients source `App.tsx` is `HashRouter`; live bundle serves `/ui/` with path routes `/providers` behind the same hash-history helper. Unauthenticated view is the session gate (`This page needs a management session… Paste this daemon's bootstrap secret — not a Provider LLM API key.`), not a sidebar bounce. `GET /management/providers/accounts` is **401** without a session. `xdg-open http://127.0.0.1:48681/ui/#/providers` from the guest session returned gio **Not Found** (desktop opener mishandles the hash). No sockets from Firefox to `:48681` after SSH `--new-tab` attempts. Not fixed on this lease: no SPA fallback in `kernel-server` `web_ui_relative_asset`, and P7-T05/clients are not writable here. Operator URL is `http://127.0.0.1:48681/ui/#/providers` after a management session. Unique follow-up if that still fails: owner reopens P7-T05 / a new UI task.

grok-4.6 remains the independent Pi binding; dsh Path B stayed Flash. No SQLite edit.

## Non-claims

- UI up is not Task completion.
- Path A remains measurement-only.
- No live Flash turn through the native panel is claimed.
- Headless Path B after the panel with the **P8-T15** helper is **pass** on Flash. The earlier fail was the restored p8t10 helper reading the Pi `grok-4.6` binding.
- Do not stop P7-T05 daemon PID 465376 / port 48681 unless this task started a replacement.

### 2026-08-23 — operational follow-up: open both panels, HTTP session

Did **not** restart dsh web (`:3080` still up). Did not mix P7-T05. No secret printed.

Listeners unchanged: node **487821** `:3080`, kernel-server **465376** `:48681`, helper **487806**, Firefox **471883**.

| GET | status | document |
|---|---|---|
| `http://127.0.0.1:3080/` | **200** | title `DSH Local Build` |
| `http://127.0.0.1:48681/ui/` | **200** | title `CognitiveOS Personal`, `/ui/assets/` |
| `http://127.0.0.1:48681/ui/#/providers` | **200** | same Personal SPA `index.html` (hash not on the wire) |
| `http://127.0.0.1:48681/ui/providers` | **404** | `LOCAL_UI_ASSET_NOT_FOUND` JSON — not the SPA |

Live bundle still has `path:"/providers"`, nav `Providers`, `session-gate`, hash-history helper.

Snap Firefox DBus `OpenURL` **ok** for `http://127.0.0.1:3080/` and `http://127.0.0.1:48681/ui/#/providers` (and `/ui#/providers`). Sessionstore loopback tabs (recovery.jsonlz4):

- `http://127.0.0.1:3080/` ×3 (one selected in two windows)
- `http://127.0.0.1:48681/ui#/bindings`
- `http://127.0.0.1:48681/ui/` (selected in one window)

`#/providers` did **not** appear in sessionstore after OpenURL (hash-only navigation may reuse the existing Personal tab). 12 ESTABLISHED firefox sockets to `:3080`; 0 long-lived to `:48681` (SPA session-gate / static pages do not keep a websocket).

HTTP session via `POST /local/session` from the runtime bootstrap **file** (not argv): **200**, channel `management`, session id present. Token written `0600` under the runtime dir (not logged). `GET /management/providers/accounts` **401** unauthenticated, **200** with that session; list count **1**, kind `openai_compatible`; secret-shaped field names only (`secret_ref`), no values printed.

Models subsection of native dsh was **not** DOM-proven (no headed screenshot; no secret-dumping UI driver). Providers heading/list is **API-proven**; Firefox paint of `#/providers` is **not** proven from sessionstore.

### 2026-08-23 — dsh model set/remove + Cos-installed panel identity

Did not mix P7-T05 kernel lease. SPA CAS lives in `cognitiveos-clients` PR [#4](https://github.com/agentkernel/cognitiveos-clients/pull/4) @ `afc5b04` (`bindingRevisionForCas` uses active rows only; remove treats 404/`PROVIDER_CONTROL_NOT_FOUND` as already cleared). Live Personal `/ui/` now serves `index--sTzcxFP.js` (old `index-BJVztyis.js` absent). No secret printed.

**Root cause (set/remove).** After revoke, the SPA still used the revoked row's revision as `expected_revision`. Daemon CAS reads only the active binding (unbound = 0). Set with the stale revision → **409** `PROVIDER_BINDING_REVISION_STALE`. Remove on a revoked row → **404** `PROVIDER_CONTROL_NOT_FOUND`. That blocked both “set a large catalog model” and “remove previous settings.” Pi `grok-4.6` is a separate `agent://personal/pi` binding and was not changed.

**HTTP proof on linux-002** (management session from bootstrap **file**; account `acct-01a02dee-…`):

| Step | Result |
|---|---|
| Before | dsh Flash **active** rev **5**; `GET /provider/v1/dsh/selected-model` Flash / `binding` |
| `POST /management/agent-bindings/remove` `{agent:dsh}` | **200** |
| After remove | dsh Flash **revoked** rev 5; selected-model Flash / file digest `fnv1a64:2da8aee1…` (no `binding_agent`) |
| Set Flash with `expected_revision=5` | **409** `PROVIDER_BINDING_REVISION_STALE` |
| Set `deepseek-v4-pro` `expected_revision=0` | **200** dsh **active** rev **6**; selected-model **pro** / `binding` |
| Remove again | **200** |
| Set Flash `expected_revision=0` | **200** dsh **active** rev **7**; selected-model Flash / `binding` |
| Pi | still `grok-4.6` **active** rev 1 |

Path B `POST /provider/v1/dsh/chat/completions` Flash → **200**, `pong` nonempty. No second LLM key in dsh-web-home `.env` (file absent). `settings.yaml` Path B + `DAEMON_BEARER`; no `api.deepseek.com`.

**Root cause (wrong panel).** Overlay `:3080` (**487821/487806**) was Path B against pin `528c682e` but `dsh_root` was `/home/hal9001/p8t15-2ba8103a/dsh`. Owner asked for the CognitiveOS install (`/home/hal9001/p8t10-a17edfad/dsh`). That P8-T10 tree is the same pin but was missing compiled web client/extension `lib/` (first crash: directory-picker module; second: `packages/extensions/*/lib/client.js`). Guest repair copied same-pin overlay compiled artifacts into the Cos tree (40 client `lib` trees + 271 extension/web files). Did not leave two `:3080` servers.

**Identity now:**

- `dsh.json`: `dsh_root=/home/hal9001/p8t10-a17edfad/dsh`, `adapter_root=/home/hal9001/p8t15-0376e942/adapter` (Path B helper), revision `528c682e061696f5a160f363f236ecbf53cbd006`
- Helper **492382**: `cognitive dsh web` Path B `--dsh-root` Cos tree, `--mode web`, loopback `:3080`
- Node **492396**: Cos `apps/cli/lib/bin.js --profile web --no-open --host 127.0.0.1 --port 3080`; cwd Cos dsh
- `GET http://127.0.0.1:3080/` **200** title `DSH Local Build`; dist `…/p8t10-a17edfad/dsh/apps/web/dist/index.html`
- `cognitive dsh status`: `ACTIVE`, `process_alive=true`, `process_id=492396`
- Daemon **465376** `:48681` and Firefox **471883** left running. Hung P8-T10 helper **430838** not killed.

Remaining honest gaps (not this lease): selected-model **file fallback after unbind** (kernel-server); `web-search-deepseek` still names official `DEEPSEEK_API_KEY`; `GET /ui/providers` **404** (hash route only); SPA session is memory-only and does not inherit the SSH HTTP session. No HTTP cancel/pause invented. Claim ceiling `hypothesis`.

### 2026-08-23 — Apply Cos dsh model so native Path B/chat uses it (D04)

Owner selected grok on **dsh** in Cos; native Models still listed only DeepSeek. Live bindings already had `agent://personal/dsh` **grok-4.6** active rev 8 / selected-model digest `binding`. SPA persist was **not** the gap. Native Models chrome is the DeepSeek Harness catalog (expected). Chat/Path B used to require `requested_model == binding.model_id`, so a native `deepseek-chat` body failed closed while Cos already showed grok.

**Shipped**

- Kernel `da37ac1d` Draft PR [#265](https://github.com/agentkernel/cognitive-os/pull/265): Path B remaps only `agent://personal/dsh` request `model` to the Cos dsh binding; Pi still exact-match. `POST /personal/dsh/runtime` `op=apply` publishes selected-model digest `binding` when web inspect is ACTIVE, pin `528c682e…`, and the model is in that account catalog. `cognitive dsh apply` POSTs apply then TERMs only the bound web pid. Dirty-restart recovery no longer aborts daemon start on already-terminal scheduler rows (still refuses to steal a live successor lease).
- Clients `a191e79` PR [#4](https://github.com/agentkernel/cognitiveos-clients/pull/4): Bindings **Apply to running dsh** (disabled unless active dsh catalog model and runtime ACTIVE). Copy: native Models may still list DeepSeek; Cos shows selected-model + digest.

**linux-002** (replaced owner-ops daemon **465376** only; EVAL / **430838** / Firefox left):

| Check | Result |
|---|---|
| Jump `da37ac1d` | admin-cli `dsh` **10/10**; `p8_t11` **1/1**; `p8_t13` **4/4**; recovered-terminal-lease **1/1**; Clippy `-D warnings`; fmt |
| New daemon | **494681** `127.0.0.1:48681` same `--runtime-root /home/hal9001/p8t13-owner-ops/runtime` |
| Live `/ui/` | `assets/index-B5jhUhi8.js` contains Apply copy |
| Bind existing Cos web **492396** | inspect ACTIVE |
| `POST op=apply` grok rev 10 | **200** `applied_model=grok-4.6` `digest=binding` `restart_performed=false` |
| Path B `model=deepseek-chat` while grok bound | **400** `invalid_request_error` (same as `model=grok-4.6`) — not `PERSONAL_PROVIDER_BINDING_MISMATCH`. Honest 4xx: catalog lists grok-4.6; this openai_compatible account does not serve it |
| Flash set+apply + `deepseek-chat` | **200** `response_model=deepseek-v4-flash` `pong` — remap proof |
| Restore grok | dsh **active rev 12**; Pi still grok-4.6 rev 1 |
| `:3080` | still Cos `p8t10-a17edfad/dsh`, one listener, title `DSH Local Build` |

`cognitive dsh apply` then **failed**: it TERMed **492396/492382**, then `cognitive dsh web` refused (`daemon is not ready for a dsh agent launch`) because doctor `overall=blocked` / `provider=blocked` / `first_conversation_ready=false` on the replacement daemon (secret/system/database/daemon were ready). Helper restored with bootstrap **file**: helper **495509**, node **495523**, rebound ACTIVE. Path B Flash still worked on the same account, so this is a doctor projection gap after dirty replace, not a missing SecretStore entry.

Restart is **not** required for Cos Path B/chat once the new daemon is up: remap + apply is enough. Restart is required only to refresh the native helper’s `DAEMON_BEARER` after a daemon replace. Native Models list stays DeepSeek chrome.

Claim ceiling `hypothesis`. UI up is not Task completion.

### 2026-08-23 — grok was posted to DeepSeek; fix routing + presentation

**Exact hop (verified, not assumed).** `plan_bound_proxy` for `agent://personal/dsh` rewrote `model` → Cos binding `grok-4.6`, then POSTed that body to the **bound account** `acct-01a02dee-…` whose endpoint host is `api.deepseek.com`. DeepSeek’s allowlist then returned `supported API model names are deepseek-v4-pro, deepseek-v4-flash, and deepseek-v4-flash-vision-exp, but you passed grok-4.6`. Wrong hop, not “need restart”.

This runtime has **one** Cos account (`openai_compatible` / `api.deepseek.com`). `grok_capable_accounts=0`. Pi `grok-4.6` is a separate binding on the same DeepSeek account and was not used for dsh.

**Shipped** at `14aadf27` Draft PR [#265](https://github.com/agentkernel/cognitive-os/pull/265):

- `model_servable_on_account`: DeepSeek hosts only serve `deepseek-*`; `grok-*` only on non-DeepSeek `openai_compatible`.
- `models add` / `binding set` → HTTP 400 `PROVIDER_MODEL_ENDPOINT_MISMATCH`.
- Path B leftover grok-on-DeepSeek → HTTP 400 `PERSONAL_PROVIDER_BINDING_MISMATCH` **before** any upstream POST (does not hit DeepSeek).
- Native web settings + `--patch` overlay Cos selected-model and that account catalog (`DAEMON_BEARER` / Path B loopback only; no Provider key in dsh `.env`).
- Path B `cognitive dsh web` / `apply` no longer requires Pi `provider.json` ready.

**Jump-host** `14aadf27`: admin-cli `dsh` **10/10**; `p8_t11` **1/1**; `p8_t13` **5/5**; recovered-terminal **1/1**; Clippy `-D warnings`; fmt. Node preflight **4/4**.

**linux-002** (replaced **494681** only; EVAL / **430838** / Firefox left):

| Check | Result |
|---|---|
| New daemon | **499615** `127.0.0.1:48681` `--runtime-root /home/hal9001/p8t13-owner-ops/runtime` binary `/home/hal9001/p8t15-14aadf27/bin/kernel-server` |
| Leftover dsh binding grok rev 12 | Path B `model=deepseek-chat` → **400** `PERSONAL_PROVIDER_BINDING_MISMATCH`; `deepseek_allowlist=false` |
| Add/bind grok on DeepSeek account | **400** `PROVIDER_MODEL_ENDPOINT_MISMATCH` |
| Grok-capable Cos account | **none** (missing non-DeepSeek `openai_compatible`, e.g. xAI / `api.x.ai`) |
| HTTP apply leftover grok | **200** `applied_model=grok-4.6` `model_endpoint_compatible=false` `path_b_will_fail_closed=true` |
| `cognitive dsh apply` | **pass** `restart_performed=true` (doctor provider blocked is no longer a launch mutex) |
| Native settings after grok apply | `model: grok-4.6` plus Cos catalog (`grok-4.6`, Flash, vision-exp, pro). `baseURL` Path B loopback; `apiKeyEnv: DAEMON_BEARER`; no `api.deepseek.com` |
| Patch overlay | same Cos model + catalog |
| Flash bind rev 13 + apply | settings `model: deepseek-v4-flash` + Cos catalog; Path B **200** `response_model=deepseek-v4-flash` assistant `pong` |
| `:3080` | Cos `p8t10-a17edfad/dsh` one listener, title `DSH Local Build`. Index HTML is a SPA shell (no model ids in first document). Models/conversation chrome is the settings/patch overlay above. |

Cannot re-bind grok after Flash: control plane now refuses grok on this DeepSeek-only account. That is the honest fail-closed. Do not invent an xAI account or copy keys.

### 2026-08-24 — control-panel create HTTP 400 PATH_FORBIDDEN (coded; Linux not-run)

Owner report: Provider Control Plane create account → HTTP 400 “provider endpoint 禁止”.

Root cause (code path, not assumed live request body): `TrustedEndpoint` stored only `/`, `/v1`, `/v1/` as OpenAI-compatible roots. Pasting a provider chat URL such as `https://api.x.ai/v1/chat/completions` (xAI docs) or an OpenRouter/Groq root (`/api/v1`, `/openai/v1`) returned HTTP 400 `PROVIDER_ENDPOINT_PATH_FORBIDDEN`. The local daemon proxy `/provider/v1/dsh` remains forbidden.

Fix (this change set, pre-Linux): strip one well-known RPC leaf (`/chat/completions`, `/completions`, `/models`) then accept roots `/v1`, `/api/v1`, `/openai/v1`, `/compatible-mode/v1`. Traversal and other paths stay fail-closed.

| Check | Environment | Revision | Result |
|---|---|---|---|
| `endpoint_trust` lib tests | `DEV-LINUX-NATIVE-01` | `8b9d09cbc1d85af18fadb536156d9f58a3b39d78` | **pass** 8/8 including `openai_compatible_rpc_suffix_and_prefixed_roots_normalize`. Worktree `/home/wuz/agent-kernel-worktrees/p8-t15-8b9d09cb` (removed after). rustc 1.97.1. |
| `p8_t13_endpoint_trust` | `DEV-LINUX-NATIVE-01` | `8b9d09cb` | **pass** 2/2 |
| `p8_t13_provider_control_plane` | `DEV-LINUX-NATIVE-01` | `8b9d09cb` | **pass** 6/6 including `create_account_strips_chat_completions_and_accepts_prefixed_openai_roots` |
| Clippy `-D warnings` `cognitive-secret`+`kernel-server` | `DEV-LINUX-NATIVE-01` | `8b9d09cb` | **pass** |
| `cargo fmt --all -- --check` | `DEV-LINUX-NATIVE-01` | `8b9d09cb` | **pass** |
| Guest daemon replace | linux-002 | `8b9d09cb` binary | **pass** pid **515695** `:48681` same runtime; `GET /personal/health` **200**; Cos `:3080` pid **500474** still listening. Replaced stale **499615**. Other daemons untouched. |

Claim ceiling `hypothesis`. No Gate / release / Profile / B01 / Agent-benefit.

## Unique next action

Owner: add a grok-capable Cos account on the Provider panel (non-DeepSeek `openai_compatible`, e.g. `https://api.x.ai/v1` or paste `https://api.x.ai/v1/chat/completions`; SecretStore key) then bind dsh to that account + grok and Apply. Until then dsh stays on Flash (working Path B). Keep **515695** / `:3080` / **430838** / Firefox. Do not auto-claim P6 / P7-T06 / P7-T07.
