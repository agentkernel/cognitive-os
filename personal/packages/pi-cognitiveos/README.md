# `@cognitiveos/pi-cognitiveos`

CognitiveOS Pi-hosted Agent Shell adapter for Personal. The initial Provider
proxy and first-conversation integration was delivered by task **P1-T07**;
composition with governed Task and resource-management application services is
owned by **P2-T02**.

Pi (`@earendil-works/pi-coding-agent`) is reused as a terminal UI. This package
is the CognitiveOS surface that runs *inside* Pi and keeps it a **non-authority
client**: it writes no authority state, mints no capability, creates no Effect
and advances no Task.

## What it does

| Pi hook | Behaviour |
|---|---|
| `project_trust` | always `{ trusted: "no" }` — Pi's own trust prompt would grant ambient project permission; in governed mode only CognitiveOS authorizes anything |
| `tool_call` | **default-deny.** `bash`, `edit` and `write` are refused with a mutating-tool reason; every other tool is refused as ungoverned |
| `session_start` | reads `GET /personal/status` from the Personal daemon and shows the real projection; warns when the first conversation is blocked |
| `/cognitive-status` | prints the same daemon facts on demand |
| Provider adapter | registers the daemon-selected model and sends Provider traffic through the daemon-owned proxy without exposing its secret to Pi |
| Private sidecar client | reads the versioned Resource projection/watch over a management session and Task watch over a separate Task session; both streams require a snapshot-first response |

## Why default-deny on tools

The Extension has no catalog, no capability and no Effect protocol, so it cannot
authorize a tool. ADR-0026 reaches the same conclusion from the other side:
tier classification is a property of a catalog-bound operation, and unknown or
unclassifiable operations default to Tier 2. Governed tool execution arrives
with the Tool Registry and process supervisor (P2-T05/P2-T06) and runs in the
daemon.

`READ_ONLY_TOOL_ALLOWLIST` is deliberately empty and is the single reviewed
place where a future batch may admit a tool.

## What it never does

- never holds, reads or forwards a Provider API key — no key from the
  environment, from a Provider configuration file, or from a resolved secret
  reference;
- never opens a database or a subprocess, and never writes to the filesystem;
- never invents readiness: an unreachable daemon, a refused bearer or a
  malformed projection all fail explicitly with a stable code, and none of them
  render as `ready`.

`src/safety.test.ts` enforces all of the above by scanning the runtime sources.

## Pi acquisition is separate from this package

ADR-0025 and ADR-0036 forbid vendoring or redistributing Pi in a CognitiveOS
release. The planned Linux 1.0 installation path instead acquires the exact
approved Pi package from the fixed official npm origin and commits a verified,
immutable managed installation. Pi is therefore not a workspace dependency of
this Extension: `src/pi-api.ts` declares the reviewed structural API mirror and
nothing Pi-related enters `pnpm-lock.yaml`. The compatibility pin in
`src/pin.ts` is drift-checked against the authoritative Rust
`PiCompatibilityPin` in `apps/pi-agent-adapter/src/lib.rs`.

## Daemon discovery

Exactly two local files are read, the same ones `cognitive` reads:

- `$XDG_STATE_HOME/cognitiveos/daemon-endpoint.json` — the loopback address
  published by `cognitive daemon start`; a non-loopback address is refused;
- `$XDG_RUNTIME_DIR/cognitiveos/local-bootstrap.secret` — the 0600 local auth
  bootstrap, used once to mint a `management`-channel bearer and never logged,
  displayed or stored.

`XDG_RUNTIME_DIR` is required and fails closed, matching the Rust layout. The
sidecar keeps management and Task bearers separately, never moves a cursor
between those paths, remints a refused read bearer at most once, and makes no
mutation request. Task completion, Effect progression, verification, dispatch,
and SQLite authority writes remain daemon-only operations.

## Status and non-claims

P1-T07 is complete in the formal task ledger, including daemon-owned Provider
proxy integration and recorded real-Pi load/first-response implementation
evidence. That evidence does not deliver P2-T02's governed Task/resource
composition and does not install, register, supervise, upgrade, roll back, or
uninstall Pi as a managed Agent; those are the separate P5-T01/P5-T02 track.

Nothing in this package alone is a B01-B12, `GMVP-LINUX`, C0/C1, Profile, or
release claim. Current status and evidence scope are owned by
[`PROGRESS.md`](../../docs/plan/PROGRESS.md), not by this README.
