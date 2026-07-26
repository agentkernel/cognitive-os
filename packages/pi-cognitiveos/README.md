# `@cognitiveos/pi-cognitiveos`

CognitiveOS Pi Extension for the Personal product (task **P1-T07**).

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

## Pi is not a dependency

ADR-0025 forbids vendoring or redistributing Pi; the user installs a compliant
Pi locally. This package therefore declares a structural mirror of the pinned
Pi Extension API in `src/pi-api.ts` instead of importing `@earendil-works/*`,
and nothing Pi-related enters `pnpm-lock.yaml`. The compatibility pin in
`src/pin.ts` is drift-checked against the authoritative Rust
`PiCompatibilityPin` in `apps/pi-agent-adapter/src/lib.rs`.

## Daemon discovery

Exactly two local files are read, the same ones `cognitive` reads:

- `$XDG_STATE_HOME/cognitiveos/daemon-endpoint.json` — the loopback address
  published by `cognitive daemon start`; a non-loopback address is refused;
- `$XDG_RUNTIME_DIR/cognitiveos/local-bootstrap.secret` — the 0600 local auth
  bootstrap, used once to mint a `management`-channel bearer and never logged,
  displayed or stored.

`XDG_RUNTIME_DIR` is required and fails closed, matching the Rust layout.

## Status and non-claims

P1-T07 is **not complete**. This package delivers the Extension half. The
daemon-owned Provider proxy and the readiness `pi` component flip are the
remaining halves and are tracked in
[`docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`](../../docs/plan/PERSONAL-DEVELOPMENT-PLAN.md).

Nothing here is a G0, B01-B12, C0/C1, Profile or release claim. The Extension
has not been loaded by a real Pi process in this repository: that evidence
belongs to P0-T06's `extension-load` verb, which requires a Linux-native host
and remains `not-run`.
