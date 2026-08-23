# P7-T05 D10 Web UI sidebar fix — running report

- Task: `P7-T05` (reopened owner delivery 2026-08-23)
- Slice: `P7-T05/D10` sidebar navigation + Provider/binding verification
- Lease: `lease/personal/P7-T05/web-ui-sidebar-fix` (open until Draft PR acceptance)
- Kernel branch: `personal/P7-T05-web-ui-sidebar-fix`
- Clients branch: `personal/P7-T05-web-ui-sidebar-fix` (`D:\cognitiveos-clients`, `pc/web/`)
- Live daemon: linux-002 PID 465376, `http://127.0.0.1:48681/ui/`
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion.
  A Provider selected-model fact is not Task completion.

## Root cause

Unauthenticated `RequireSession` used `<Navigate to="/session" replace />`.
The SPA is HashRouter (`#/agents`, not `/ui/agents`). Daemon `GET /ui/agents`
is 404 by design (no SPA fallback). Sidebar `NavLink`s were real hash links
and were not covered by `pointer-events: none`.

Clicking Home/Agents/Providers/... changed the hash, then the session gate
immediately replaced the location back to `#/session`. Same Session form, same
URL → sidebar clicks looked like a no-op.

## Fix

`RequireSession` no longer navigates away. Unauthenticated pages keep their
hash and render that page’s title plus an in-place session form. Issuing a
session re-renders the gate via `SessionTick`. Skip-link `#main` is focused in
JS so HashRouter does not consume it.

Failure-first Vitest: hash hrefs, every sidebar item changes hash + `h2`
without a session, Providers/Bindings remain reachable after `rememberBearer`.
Local `pnpm test`: **30/30 pass**.

## Live linux-002 proof (Firefox WebDriver, headless snap binary)

Daemon PID **465376** still serving. SPA bundle `index-BJVztyis.js`.

| Menu | Unauthenticated | Authenticated |
|---|---|---|
| Home | **pass** `#/` heading Home | **pass** heading Home |
| Agents | **pass** `#/agents` | **pass** |
| Providers | **pass** `#/providers` | **pass** |
| Bindings | **pass** `#/bindings` | **pass** |
| Tasks | **pass** `#/tasks` | **pass** (heading Tasks, Effects, Evidence) |
| Activity | **pass** `#/activity` | **pass** |
| Resources | **pass** `#/resources` | **pass** |
| Session | **pass** `#/session` | **pass** |

Session bootstrap: **pass** (`session_ready: true`).

Home `/personal/health|status|readiness|doctor`: HTTP 200 (redacted snippets only).

Agents identity list via `family=agent`: HTTP 400 `RESOURCE_MANAGER_FAMILY_UNKNOWN`
(UI uses `family=runtime`; typed agent lifecycle pause/resume/stop remain
unavailable/not-run).

Provider create + key → SecretStore: **pass**. Account prefix `acct-01a0`.
Status text: key handed to daemon SecretStore; probe class `model_discovery`.
Key never printed; temp key file shredded after the run.

Agent LLM binding: **pass**. First bind stored, stale expected_revision
rejected, retry stored. GET `/management/agent-bindings`:
`agent://personal/dsh` + `deepseek-v4-flash` revision 2 `active`.
GET `/provider/v1/dsh/selected-model`: `selected_model=deepseek-v4-flash`,
`selected_snapshot_digest=binding`. Binding is the runtime selected-model
source for subsequent Agent/dsh Path B dispatch. A full dsh `--path b`
assistant turn was **not-run** in D10 (D09 previously `assistant_ok`).

Tasks preview/admit/watch click-through: **not-run** this slice (routes exist;
D09 previously preview+admit pass). HTTP cancel: **not-run** (no typed HTTP).

## Non-claims

- No Gate, release, Profile, B01, EVAL, or Agent-benefit promotion.
- SecretStore presence is not a dumped credential.
- `selected_model` from the binding is not Task completion.
- linux-002 is the standing Personal guest, not a B01 campaign mutation of
  `B01-Clean-Linux-001` (untouched).

## Unique next action

Owner review of Draft PRs. Do not auto-claim P6 / P7-T06 / P7-T07. Keep PRs
Draft until acceptance. Lease stays open until merge/reconciliation.
