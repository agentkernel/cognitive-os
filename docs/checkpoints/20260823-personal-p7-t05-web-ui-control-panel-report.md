# P7-T05 Web UI control panel — running report

- Task: `P7-T05` (reopened owner delivery 2026-08-23)
- Slice: `P7-T05/D08` in-progress; `P7-T05/D09` ready
- Lease: `lease/personal/P7-T05/web-ui-control-panel`
- Kernel branch: `personal/P7-T05-web-ui-control-panel`
- Clients branch: `personal/P7-T05-web-ui-control-panel`
- Approved checkout: `D:\cognitiveos-clients` (`pc/web/`)
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion

## Gaps versus D01–D07 closure

Prior closure ([P7-T05 report](./20260823-personal-p7-t05-web-ui-report.md))
left live Provider/SecretStore key entry and post-session Agent/Provider/Task
click-through as `not-run`. Audit of `pc/web` at clients `c9a3b34` plus daemon
P8-T13 routes found:

| Gap | Evidence |
|---|---|
| Create form used `openai` / `anthropic` | Daemon `ProviderKind::parse` accepts only `openai_official` / `anthropic_official` / `openai_compatible` |
| No trust confirmation before persist | Product sequence requires it for private/HTTP custom endpoints |
| Binding POST omitted `expected_revision` | Daemon did not CAS; D08 adds optional `expected_revision` → 409 `PROVIDER_BINDING_REVISION_STALE` |
| UI rejected bind on revoked accounts | P8-T13 binds then keys; dispatch (not bind) must fail closed |
| Tasks page had no preview/admit/watch | Inventory already listed those routes as available; SPA now calls them |
| Live key / SecretStore | still pending D09 |
| HTTP cancel / Agent lifecycle | remain `not-run` (no typed HTTP); UI still does not invent routes |

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

1. D08 SPA/unit work on the clients branch (kinds, trust, binding CAS
   client+daemon, Task intent/preview/admit/watch poll).
2. Local Windows GNU Rust **not-run** (`RUST-LINK-DEV-WIN-GNU-01`).
3. Clients `pc/web` `vitest run` **pass** 27/27 (10 files), including probe
   class, policy, task draft, watch SSE, and DOM redaction.
4. Clients `pc/web` `pnpm build` (`tsc --noEmit` + Vite) **pass**.
5. Kernel `node --test tools/test/p7_t05_web_ui_inventory.test.mjs` **pass** 8/8.
6. `pnpm run check:consistency` **pass**.
7. Handbook: `check-handbook` **pass** (55×2); `generate-handbook --check`
   **pass** (18 pages). Documented optional `expected_revision` /
   `PROVIDER_BINDING_REVISION_STALE` and the localhost Web UI as a daemon
   client (not a second writer).
8. Live Provider key entry **not-run** until D09 exact Linux UI driver.

## Unique next action

Commit/push both Draft PRs, then D09 on `wuz@192.168.1.2` with a real UI driver
(key file never in argv/logs/evidence).
