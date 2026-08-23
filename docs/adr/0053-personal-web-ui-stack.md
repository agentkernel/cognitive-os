# ADR-0053: Personal Web UI Stack, Serving, Session, and Threat Binding (P7-T05)

- Status: Accepted for P7-T05/D01 readiness. SPA implementation remains blocked
  on an approved `cognitiveos-clients` checkout under `pc/web/`.
- Date: 2026-08-23
- Decision owners: CognitiveOS Personal maintainers
- Classification: Personal product local-UI decision. This ADR is not a
  registry REQ, schema, transition, vector, Gate, release, or Profile claim.
  P7-T05 stays post-1.0 and non-blocking for Linux 1.0 / `GMVP-LINUX`.

## Context

P7-T05 must deliver a localhost-only, single-owner Web UI that is a daemon
client only. The formal plan names React + TypeScript + Vite as a candidate
and places the client in the external repository
[cognitiveos-clients](https://github.com/agentkernel/cognitiveos-clients)
at `pc/web/`. ADR-0001 already selects TypeScript for clients. ADR-0003
selects HTTP JSON + SSE. ADR-0019/0022 freeze loopback listen, channel-scoped
bearers, no cookie auth, and the local-daemon threat model, and they defer a
stricter browser Origin allowlist to a future local-UI ADR. This is that ADR.

This repository must not recreate `clients/**` and must not implement the SPA
inside `apps/cognitiveos-console` (stub only). A local checkout of
`cognitiveos-clients` was not present on the delivery host when this ADR was
accepted; D02–D07 SPA work is blocked on that checkout.

## Decision

### 1. Client repository and stack

1. The only implementation path is `cognitiveos-clients/pc/web/`.
2. The approved stack is **React + TypeScript + Vite**, producing a
   deterministic static bundle with pinned exact dependency versions.
3. Runtime dependencies must be OSI-permissive (MIT/Apache-2.0/BSD). AGPL or
   other copyleft runtime libraries are rejected for this SPA (the Agent Hub
   AGPL gate remains a separate, out-of-scope product).
4. No runtime CDN, no package download at browse time, no service worker that
   caches credentials.
5. This ADR does not approve a second in-repo SPA, a Console rewrite, or a
   new public contract.

### 2. Serving path, origin, CORS, CSRF

1. **Product serving path:** the Personal daemon serves the pinned static
   bundle same-origin from its existing loopback listener
   (`127.0.0.1` / `[::1]`, product port `48181` when started by
   `cognitive daemon start`) under `/ui/` and `/ui/*`. There is no second
   public listener and no LAN bind.
2. **Rejected as the product path:** a separately launched Vite preview or
   static server on another port. That would require CORS and a second origin.
   Local Vite is not an accepted validation substitute for the daemon-served
   bundle.
3. **CORS:** none. Same-origin needs no `Access-Control-Allow-Origin`. Foreign
   origins receive no credentialed API access.
4. **CSRF:** cookies remain forbidden (ADR-0022). The SPA sends
   `Authorization: Bearer <channel-token>` only. A browser `Origin` or
   `Referer`, when present, must match the daemon's own loopback origin
   (`http://127.0.0.1:<port>`, `http://localhost:<port>`, or
   `http://[::1]:<port>`). Missing Origin remains allowed for CLI/curl.
   Non-loopback Origin/Referer is rejected. The daemon does not yet enforce
   the Origin/Referer allowlist; that in-repo control is a follow-on of this
   decision and is not a reason to invent cookie auth.
5. **Host:** existing `LOCAL_HOST_HEADER_REJECTED` loopback Host check stays.

### 3. Session and secret handling

1. Bootstrap remains `POST /local/session` with the daemon bootstrap secret
   (ADR-0022). The SPA holds management and Task bearers **in process memory
   only**.
2. Forbidden persistence: localStorage, sessionStorage, IndexedDB, URL query,
   hash that is recorded in history, exported support bundles, DOM text, and
   telemetry.
3. The browser never reads `local-bootstrap.secret`, SQLite, SecretStore, or
   the filesystem. The owner pastes the bootstrap secret once into a
   non-echoing, memory-only field; the value is discarded after session issue.
4. Management and Task tokens, caches, retry state, watch cursors, and
   operation sets stay disjoint. Cross-presentation is
   `SHELL_CHANNEL_BINDING_MISMATCH`.
5. Provider API keys travel only in the management `POST /management/providers/accounts/key`
   body to the daemon, then into the approved SecretStore. The SPA must not
   put key material or resolvable SecretRefs in DOM, URL, storage, logs,
   errors, or support output. `SecretRef` is an opaque handle, not a
   credential.

### 4. Contract reuse

The SPA consumes existing management and Task routes inventoried in
[web-ui-route-inventory.json](../architecture/personal/web-ui-route-inventory.json).
If a typed daemon operation is missing (Task cancel HTTP; Agent
pause/resume/stop/restart/quarantine HTTP), the UI renders
unavailable/not-run and records the missing dependency. P7-T05 must not add a
generic lifecycle route or a second API writer.

### 5. Threat model binding (browser client)

This ADR binds ADR-0019 rows to the SPA:

| Threat | SPA control |
|---|---|
| CSRF | no cookies; explicit bearer; loopback Origin/Referer allowlist |
| DNS rebinding | loopback Host check; same-origin `/ui/`; no CORS `*` |
| Token theft | memory-only tokens; no URL/storage; short idle/absolute expiry; restart revoke |
| Channel confusion | disjoint clients and credentials; fail closed on mismatch |
| XSS / untrusted output | daemon-redacted text escaped; CSP `default-src 'self'`; no `eval`; Agent/Provider/Event markup never executed |
| Secret leak | key/SecretRef negatives on DOM/URL/storage/history/logs/telemetry/errors/support |
| Confused deputy | same-user malware remains inside the Personal v1 trust boundary (ADR-0019) |
| Remote exposure | loopback only; no product LAN/public Console |

### 6. Validation route

| Check | Environment | Maximum claim |
|---|---|---|
| Inventory/static Node tests | `DEV-WIN-GNU-01` or CI | implementation evidence |
| Daemon Origin/CSP/static `/ui/` | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` / exact-revision `DEV-LINUX-NATIVE-01` | implementation evidence |
| SPA unit/component/a11y | client checkout + Node | implementation evidence |
| Browser journeys | `DEV-LINUX-NATIVE-01` with a local browser against the daemon-served bundle | `tested-local`; not Gate/release/Profile |
| B01 guest | not a P7-T05 target | — |

Ordinary CI and local smoke tests do not promote Gate, release, Profile, or
Agent-benefit claims.

## Alternatives considered

1. **Vue/Svelte/Solid instead of React.** Workable, but rejected: ADR-0001
   already standardizes TypeScript clients; React is the plan candidate; a
   third UI runtime would add licensing and contributor cost without an
   authority benefit.
2. **Vite preview as the product origin.** Rejected: a second origin forces
   CORS and weakens the Host/Origin story ADR-0019 reserved for this ADR.
3. **Cookie session for "simple" login.** Rejected: CSRF on loopback is the
   reason ADR-0019 forbids cookies.
4. **Implement the SPA in `apps/cognitiveos-console` or `clients/` inside this
   repo.** Rejected: formal plan and ADR-0007 successor place the client in
   `cognitiveos-clients/pc/web/`.
5. **Browser talks to Provider or SecretStore directly.** Rejected: axioms A1
   and A5; P8-T13 already owns daemon-side Provider control.

## Consequences

- D01 can land in this repository (ADR, inventory, focused negatives, serving
  and session decision) without a client checkout.
- D02–D07 SPA slices stay blocked until the owner provides an approved
  `cognitiveos-clients` checkout. Missing checkout is `blocked` /
  `not_available`, not a pass and not a reason to invent `clients/**`.
- A later in-repo daemon change may enforce the Origin/Referer allowlist and
  serve `/ui/` from a configured asset directory. That change is still P7-T05
  and still must not invent public Task/lifecycle contracts.
- Completing this ADR does not implement a Web UI, pass a Gate, or change
  Linux 1.0 scope.

## Compliance checks

- `tools/test/p7_t05_web_ui_inventory.test.mjs` against
  `docs/architecture/personal/web-ui-route-inventory.json`.
- `pnpm run check:consistency` and handbook sync for mapped pages.
- No `clients/` directory in this repository; `apps/cognitiveos-console`
  remains a documentation stub.
