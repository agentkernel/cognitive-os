# P7-T05 Web UI control panel — running report

- Task: `P7-T05` (owner delivery 2026-08-23; **done**)
- Slice: `P7-T05/D08` done; `P7-T05/D09` done (Linux evidence below)
- Lease: `lease/personal/P7-T05/web-ui-control-panel` (**closed**)
- Kernel: merged PR [#262](https://github.com/agentkernel/cognitive-os/pull/262) at `main@962463fed10e30bcf9668d993d735732188b9048`; product HEAD `881ebe8260d52fa33581682f7f5736169daa9d25`; docs HEAD `f374abc9`
- Clients: merged PR [cognitiveos-clients#2](https://github.com/agentkernel/cognitiveos-clients/pull/2) at `main@db563744f1bfe6b42fa977d59f4ee48a16cee3c2` (`c6b763b8d3f5a6053cf47cbe056e2e77ca92c993`)
- Approved checkout: `D:\cognitiveos-clients` (`pc/web/`)
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion.
  dsh assistant text is not Task completion.

## Gaps versus D01–D07 closure

Prior closure ([P7-T05 report](./20260823-personal-p7-t05-web-ui-report.md))
left live Provider/SecretStore key entry and post-session Agent/Provider/Task
click-through as `not-run`. Audit of `pc/web` at clients `c9a3b34` plus daemon
P8-T13 routes found:

| Gap | Disposition after D08/D09 |
|---|---|
| Create form used `openai` / `anthropic` | **fixed** — daemon tokens `openai_official` / `anthropic_official` / `openai_compatible` |
| No trust confirmation before persist | **fixed** — extra confirm only for private/HTTP custom endpoints |
| Binding POST omitted `expected_revision` | **fixed** — optional CAS; mismatch HTTP 409 `PROVIDER_BINDING_REVISION_STALE` |
| UI rejected bind on revoked accounts | **fixed** — bind allowed; dispatch blocked until account usable |
| Tasks page had no preview/admit/watch | **fixed** — `intent.record` / `intent.interpret` / `preview` / `admit` / `watch` |
| Live key / SecretStore | **pass** on D09 Linux UI driver (key never in Git/chat/argv/logs/evidence) |
| HTTP cancel / Agent pause/resume/stop/restart/quarantine | remain **`not-run`** (no typed HTTP); UI still does not invent routes |
| `GET /task/watch` resume gap click-through | unit-tested; dedicated UI 409 click **not-run** |

## Owner capabilities

| Capability | Result | Evidence |
|---|---|---|
| 1. Enter LLM API key via localhost UI | **pass** | D09 Chrome CDP driver: password field → `POST /management/providers/accounts/key` → SecretStore; field cleared; secret-shape scan of daemon log/UI HTML/driver result **pass** |
| 2. Bind Agent to account+provider+model | **pass** | UI bind of `dsh` + `deepseek-chat`; stale `expected_revision=0` rejected (`stale_cas=stale`); later same UI account rebound to `deepseek-v4-flash` (revision 4) for a usable catalog id |
| 3. Agent runs correctly | **pass** (hypothesis) | UI Task preview+admit **pass**. Typed `cognitive dsh launch --print --path b` on the UI-created account: Workspace Read/Search/Write `COMPLETED`; `assistant_ok: true`; `selected_model: deepseek-v4-flash`; `dsh_response_is_not_task_completion: true`. First launch against UI-bound `deepseek-chat` was `INVALID_REQUEST` (assumed model id). No HTTP Agent launch exists |

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. D08 SPA/unit work on the clients branch (kinds, trust, binding CAS
   client+daemon, Task intent/preview/admit/watch poll, manual model add).
2. Local Windows GNU Rust **not-run** (`RUST-LINK-DEV-WIN-GNU-01`).
3. Clients `pc/web` `vitest run` **pass** 27/27 (10 files).
4. Clients `pc/web` `pnpm build` **pass**.
5. Kernel `node --test tools/test/p7_t05_web_ui_inventory.test.mjs` **pass** 8/8.
6. `pnpm run check:consistency` **pass**.
7. Handbook: `check-handbook` **pass** (55×2); `generate-handbook --check`
   **pass** (18 pages).
8. Kernel Draft PR [#262](https://github.com/agentkernel/cognitive-os/pull/262);
   clients Draft PR [#2](https://github.com/agentkernel/cognitiveos-clients/pull/2).
9. D09 `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2`, host `hal9000`) exact
   kernel `881ebe82` + clients `c6b763b`:
   - SPA `vitest` 27/27 and production build **pass**
   - `binding_expected_revision_rejects_stale_cas` **pass** 1/1
   - kernel-server Origin/UI serving tests **pass** 6/6
   - daemon-served `GET /ui/` HTTP 200
   - Chrome CDP UI driver **pass**: session, live key, probe, manual model,
     bind, stale CAS, Task preview/admit, DOM/URL/storage redaction
   - secret-shape scan of daemon log, UI HTML, driver result **pass**
   - `cognitive dsh launch --print --path b` **pass** (`assistant_ok: true`)
     after binding the UI account to catalog model `deepseek-v4-flash`
   - Cleanup: control-plane keys removed, accounts deleted, temp key shredded,
     disposable runtime/Chrome/worktree removed. HTTP cancel remains `not-run`.

## Git / closure

- Kernel PR [#262](https://github.com/agentkernel/cognitive-os/pull/262) merged
  at `main@962463fe`. Required CI `32622026657` at `f374abc9` passed Ubuntu,
  Windows, and required-ci.
- Clients PR [cognitiveos-clients#2](https://github.com/agentkernel/cognitiveos-clients/pull/2)
  merged at `main@db56374`.
- Lease `lease/personal/P7-T05/web-ui-control-panel` closed; Current snapshot
  Active task lease is `none`.
- Residual honest gaps: HTTP cancel / Agent pause/resume/stop/restart/quarantine
  remain `not-run` (no typed HTTP). Dedicated UI watch-resume 409 click remains
  `not-run` (unit-tested only). These do not retract the three owner capabilities.

## Unique next action

Lease closed. Wait for a fresh owner delivery instruction. Do not auto-claim
P6 / P7-T06 / P7-T07.
