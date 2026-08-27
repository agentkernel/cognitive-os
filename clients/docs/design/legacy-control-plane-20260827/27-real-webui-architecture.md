# 27 — Real Web UI Architecture Map (as-built)

- Phase 2.5 (audit only)
- Date: 2026-08-24
- Subject: `D:\cognitiveos-clients\pc\web\` — audited read-only on branch `personal/P7-T05-dsh-binding-cas` @ `0320c1a` (superset of `main` @ `db56374`; deltas in §8). **No inference: every entry cites a file.**

## Current implementation (frozen audit baseline)

The accepted current SPA uses seven routes:
Home / Work / Agents / Providers / Resources / Activity / System. Everything
below remains the as-built pre-closure observation at `0320c1a` / `db56374`:
the single-file React SPA, HashRouter, then-current routes, tests, memory-only
session and policy modules. Later P7-T05 integration and target design must not
be read back into those facts.

## Personal 2.0 target delta

The adopted target replaces the view architecture with a three-region desktop
shell, global Agent Shell, six-space IA, Adapter conversation projection,
Goal/Plan/Task Work, task-oriented seven-family placement (Library holds
Memory/Skills/Tools/MCP), Settings Account Hub, and one provenance timeline.
Most are not present here and therefore remain
`Requires-backend` or new frontend work as mapped in
[Traceability](35-design-to-code-traceability.md). No statement below claims
those target capabilities or authorizes an API.

---

## 1. Toolchain reality

| Fact | Value | Evidence |
|---|---|---|
| Package manager | pnpm 10.33.2 (`packageManager` pinned); `.npmrc` works around pnpm 11 `minimumReleaseAge`; `pnpm-workspace.yaml` only allows esbuild build scripts | `pc/web/package.json:6`, `.npmrc`, `pnpm-workspace.yaml` |
| Framework | React 18.3.1 + react-dom 18.3.1, StrictMode | `package.json:17-18`, `src/main.tsx:6-9` |
| Build | Vite 5.4.11 + `@vitejs/plugin-react` 4.3.4; `base:"/ui/"`; `sourcemap:false` | `vite.config.ts:4-11` |
| Language | TypeScript 5.6.3 strict + `noUnusedLocals/Parameters`; type-check inside `build` | `tsconfig.json:12-17`, `package.json:11` |
| Router | react-router-dom 6.28.0, **HashRouter only** | `src/App.tsx:2,1402` |
| State | none (no store library); per-page `useState` + module-level session object + one `createContext` tick | `src/App.tsx:108-114`, `src/session.ts:3-7` |
| Styling | one global hand-written CSS file, 155 lines, dark-only, ~10 utility classes | `src/styles.css` |
| Tests | Vitest 2.1.8 + jsdom 25; 9 test files; no network mocking; no e2e | `vite.config.ts:12-15`, `src/*.test.ts(x)` |
| Serving | `dist/` copied to daemon `data_dir()/ui`; CSP `default-src 'self'` + `'unsafe-inline'` styles; `connect-src 'self'` | `README.md:17-18`, `index.html:6-9` |

## 2. File map (complete)

```text
pc/web/
  index.html                 CSP meta; module entry
  package.json / pnpm-workspace.yaml / .npmrc / vite.config.ts / tsconfig.json
  README.md                  stack + serving + claim ceiling
  src/
    main.tsx                 StrictMode mount
    App.tsx                  ALL routes, shell, gates, 10 pages (~1485 lines)
    api.ts                   readJson, issueChannelSession, header-injection rejection
    channels.ts              path→channel classification, bearer injection, daemonFetch
    session.ts               memory-only token store + storage self-check
    policy.ts                redaction, CAS derivation, dispatch/apply gates, cost display, escaping
    probe.ts                 probe error classification, provider kinds, trust gate
    taskDraft.ts             uuidV7, fixed workspace-search draft builder
    watch.ts                 watch controller state machine (live|stale|disconnected|reconciling|unknown)
    watchSse.ts              SSE frame parser + resume-stale detection (never opened as a stream)
    identities.ts            9-key agent identity merge
    styles.css               the entire theme (155 lines)
    *.test.ts(x)             9 unit/DOM test files
```

## 3. Architecture characteristics (as-built)

1. **Single-file application**: shell + routing + all pages + shared components live in `App.tsx`. Logic is factored into modules by concern (api/channels/session/policy/probe/taskDraft/watch/identities) — the *logic* layer is reasonably separated; the *view* layer is not.
2. **No component library**: shared view primitives are `Shell`, `RequireSession`, `SessionForm`, `StateNote`, `JsonPanel` — all local to `App.tsx`. Tables/forms/panels are inline HTML per page. No modal/dialog/toast components exist.
3. **Untyped data layer**: `readJson` returns `unknown`; pages coerce via `asRecord`/`asList` and probe list keys (`items/accounts/bindings/events/alerts/models`). No generated bindings, no shared contract types.
4. **Manual refresh model**: every page loads once on mount/param change; mutations call local `refresh()`. No polling timers, no invalidation, no cache, no EventSource (the SSE parser exists but is fed by manual GET).
5. **Channel-aware fetch**: `daemonFetch` injects the right channel bearer and throws `SHELL_CHANNEL_BINDING_MISMATCH` on cross-channel use; secret-in-URL and caller header injection are rejected twice; all responses pass `redactSecrets` before render.
6. **Session**: bootstrap secret → two `POST /local/session` (management + task) → in-memory bearers; `assertMemoryOnlyStore()` throws if web storage contains token-like material; sessions die on reload by design.
7. **State rendering**: one `LoadState` union (`loading|ready|empty|denied|disconnected|unknown|not-run`) rendered as a single muted text line (`StateNote`).
8. **Accessibility seeds**: skip link, `aria-current`, `role="status"`/`aria-live`, table captions, focus-visible outline, `tabIndex=-1` main target.

## 4. Data dependency → source map (every frontend data dependency)

| UI data | Source endpoint | Channel | Module |
|---|---|---|---|
| session bearers | `POST /local/session` ×2 | bootstrap | `api.ts` |
| daemon liveness | `GET /personal/health` | none | `App.tsx` Home |
| status/readiness/doctor | `GET /personal/{status,readiness,doctor}` | mgmt | Home |
| runtime inventory | `GET /management/resource/v1/list?family=runtime` | mgmt | Agents |
| runtime inspect | `GET /management/resource/v1/inspect?family=runtime&id=` | mgmt | Agent detail |
| bindings | `GET /management/agent-bindings` | mgmt | Agents, Bindings |
| dsh runtime | `GET /personal/dsh/runtime` | mgmt | Agents, Bindings |
| dsh selected model | `GET /provider/v1/dsh/selected-model` | mgmt | Bindings |
| provider accounts | `GET /management/providers/accounts` + `/inspect` | mgmt | Providers |
| provider mutations | `POST /management/providers/accounts{,/update,/delete,/key}` | mgmt | Providers |
| models | `GET /management/providers/models?account_id=`; `POST …/refresh`, `…/add` | mgmt | Providers, Bindings |
| binding mutations | `POST /management/agent-bindings{,/remove}` | mgmt | Bindings |
| dsh apply | `POST /personal/dsh/runtime {op:apply}` | mgmt | Bindings |
| usage/budgets/alerts/audit | `GET /management/{usage,budgets,alerts,audit}` | mgmt | Activity |
| task chain | `POST /task/{intent.record,intent.interpret,preview,admit}` | task | Tasks |
| task projections | `GET /task/{effects,observation,evidence}?task_ref=` | task | Tasks |
| task watch | `GET /task/watch?resume_from=` (manual poll) | task | Tasks |
| resource families | `GET /management/resource/v1/list?family=…` | mgmt | Resources |

## 5. Design-token / theme reality

No tokens file, no light theme, no density scale, no motion, no elevation, no badges/chips. Palette: `--bg #10141c`, `--panel #1a2130`, `--ink #e8edf5`, `--muted #9aa6b8`, `--accent #7eb6ff`, focus `#ffd166` + warn/bad/ok accents (`styles.css:1-14`). Font `"Segoe UI", system-ui, sans-serif`. **No "Apple theme" code exists in this repo** — the accepted D10 refinement lives only in an unpublished Git bundle (SHA-256 `02a0216f…641e`); current branches contain no trace of it.

## 6. Typed-client question (resolved by TS-layer audit, 2026-08-24)

- `pc/web` does **not** consume `packages/contracts-ts` or `packages/sdk-ts`; every import is `react`/`react-dom`/`react-router-dom`/relative (verified by full import scan). The clients repo has **no code outside `pc/web/`** — `shared/` is docs-only (no package.json anywhere else in that repo).
- Kernel-side TS layer (4 packages): `contracts-ts` (54 generated schema bindings from `specs/schemas/` + error registry; no provider-* modules), `sdk-ts` (typed **AKP-envelope** clients for the M5 surface — management `<op>`, shell verbs, 4 task routes, `/task/watch` SSE; **does not cover** the Personal HTTP surface the SPA uses: `/personal/*`, `/management/providers/*`, `/management/resource/v1/*`, `/management/{usage,budgets,alerts,audit}`), `pi-cognitiveos` (contains the only typed Personal-daemon client — `PersonalDaemonClient`, read-only, 5 endpoints), `dsh-akp-adapter`.
- Documented canonical SDK home for clients: `packages/sdk-ts` (clients-repo README §4; dependency direction `agent-shell → sdk-ts → contracts-ts`). ADR-0053 §4 mandates **route-inventory reuse**, not sdk-ts adoption.
- **Consequence (recorded for `36`/`39`):** the SPA's typed-envelope need is real and currently unmet; the smallest honest path is a `pc/web`-local typed client module generated/hand-mapped from the frozen route inventory + `contracts-ts` envelope types where they apply — not an sdk-ts adoption (wrong surface), not a shared/ package (no infrastructure there).

## 7. Test reality

9 test files, all unit/DOM-level: channel classification, session memory-only guard, policy gates (CAS derivation, dispatch/apply gates, cost display, escaping), watch state machine, SSE parser, task draft builder, identities merge, and 4 DOM tests around the session gate/sidebar. **The network layer (`readJson`/`daemonFetch` against a mock fetch) is never tested; no page-level interaction tests; no e2e.**

## 8. In-flight branch deltas (audit basis)

- `main` @ `db56374` = merged D01–D09 state (PR #2).
- `personal/P7-T05-web-ui-sidebar-fix` @ `adea0b5`: inline session gate (no redirect), SessionForm extraction, SessionTick context, skip-link focus, NAV export.
- `personal/P7-T05-dsh-binding-cas` @ `0320c1a` (contains sidebar-fix): CAS-from-active-binding derivation, "Apply to running dsh" section with fail-closed gate, copy corrections, active-only bindings table + Dispatch column, tests.
- Unpublished: D10 Apple-theme bundle (SHA-256 `02a0216f…641e`), blocked on client-repo write access (owner remediation pending).

---

*Per-page audit (routes, component trees, problems, reusable/replaceable parts) is `34-webui-current-state-audit.md`. Subsystem keep/refactor/replace decisions are `36-refactor-vs-rewrite.md`.*
