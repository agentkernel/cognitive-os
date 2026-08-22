# ADR-0027: Personal Pi Extension Surface and Pi Runtime Observation (P1-T07)

- Status: Accepted for P1-T07 implementation
- Date: 2026-07-26
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: Personal product decision. Not a CognitiveOS specification
  requirement, registry REQ, schema, transition, vector, Profile claim, G0
  claim, B01-B12 claim, containment claim, or supply-chain provenance claim.
- Extended by: ADR-0035 separates Pi's Shell-host role from its managed-Agent
  role; ADR-0036 replaces the Linux 1.0 default user-installed Pi path with
  product-owned official npm acquisition. This ADR's non-authority Extension,
  secret, readiness and tool-denial boundaries remain in force.
- Amended 2026-08-22 (P8-T11): the public management `POST /provider/v1/chat/completions`
  route may forward `stream:true` as SSE. Pi conversations and the private-candidate
  proxy remain unary. Local-disconnect-to-upstream-abort is still not a cancellation
  contract.

## Context

P1-T07 reuses Pi as the terminal UI for the first governed conversation. Three
facts constrain how that can be done.

1. **Pi has no permission system.** An extension holds the full permission of
   the Pi process (source PI-01). Anything CognitiveOS wants to forbid inside
   Pi must be forbidden by the extension itself, at the hook that Pi actually
   calls, not by convention.
2. **Pi can own credentials.** Pi resolves provider keys from environment
   variables, `auth.json`, or even a command it executes (source PI-07). That
   is incompatible with ADR-0018's daemon-owned secret boundary.
3. **ADR-0023 pinned the `pi` readiness component to `not_configured`** until
   P1-T07, and made `first_conversation_ready` require it. The component is
   therefore structurally false today, and the flip has to happen without
   changing the aggregation rules the ADR froze.

ADR-0025 additionally decided not to vendor or redistribute Pi or Node: the
user installs a compliant Pi locally.

## Decision

### 1. The Extension is a package in this repository, and Pi is not a dependency

`packages/pi-cognitiveos/` carries the CognitiveOS Pi surface. It declares a
structural mirror of the pinned Pi Extension API subset in `src/pi-api.ts`
rather than importing `@earendil-works/pi-coding-agent`. Nothing Pi-related
enters `pnpm-lock.yaml`, so a Pi dependency cannot be pulled into every
workspace install or CI job, and ADR-0025's non-vendoring decision is preserved
mechanically rather than by review.

The Pi compatibility pin stays single-sourced in the Rust
`PiCompatibilityPin::expected()` (`apps/pi-agent-adapter/src/lib.rs`). Both
mirrors — the TypeScript `src/pin.ts` and the Rust `PINNED_PI_VERSION` in the
daemon — are drift-checked against that constant by tests.

### 2. Tool calls are default-deny, not "block the three mutating built-ins"

The Extension refuses **every** `tool_call`, with a specific reason for
`bash`/`edit`/`write` and a generic ungoverned-execution reason otherwise.

The reasoning is not defense in depth, it is authority: the Extension has no
catalog, no capability, no Intent and no Effect protocol, so there is nothing it
could use to authorize a tool. ADR-0026 reaches the same place from the other
direction — tier classification is a property of a catalog-bound operation, and
unknown or unclassifiable operations default to Tier 2. Every Pi tool is
unclassifiable from inside the Extension.

`READ_ONLY_TOOL_ALLOWLIST` exists and is empty. It is the single reviewed place
where a future batch may admit a tool, so admitting one is an explicit edit
rather than an accidental gap. Governed tool execution belongs to the Tool
Registry and process supervisor (P2-T05/P2-T06) and runs in the daemon.

`project_trust` is likewise always denied: Pi's own trust prompt would grant Pi
ambient project permission, which is precisely the decision CognitiveOS must own.

### 3. The Extension holds no Provider credential and reads only daemon facts

The Extension reads exactly two local files, the same ones `cognitive` reads:
the published loopback endpoint (`daemon-endpoint.json`) and the 0600 local auth
bootstrap (`local-bootstrap.secret`). It mints a `management`-channel bearer per
ADR-0022 and reads `GET /personal/status`.

It never reads the Provider configuration file, never resolves a `SecretRef`,
never reads a Provider key from the environment, never opens a database, never
spawns a subprocess, and never writes to the filesystem. These are enforced by a
source scan over the runtime modules, not only by review.

A missing `XDG_RUNTIME_DIR` fails closed rather than falling back to another
location: a 0600 secret has no acceptable alternate home.

### 4. Failure is explicit; readiness is never synthesized

An unreachable daemon, a refused bearer and a malformed projection each map to a
stable error code and are surfaced in both the status bar and a notification. A
daemon restart that invalidates the bearer is recovered from exactly once; a
second refusal fails. No failure path renders as `ready`, and a projection
reporting `authority_side_effects: true` is refused rather than displayed.

This is the client-side counterpart of the B01 failure condition "daemon
synthetic ready".

### 5. The `pi` readiness component becomes a real observation

`apps/kernel-server/src/personal/pi_runtime.rs` observes the Pi runtime from a
new, non-secret `pi.json` in the Personal config directory:

```json
{
  "schema_version": 1,
  "surface": "personal-pi-config",
  "executable_path": "<absolute path to the user's Pi executable>",
  "extension_entry_path": "<absolute path to the built CognitiveOS Extension>"
}
```

`pi.json` mirrors ADR-0020's `provider.json` shape: schema version, surface
marker, non-secret fields only. Relative paths are rejected, because a path
resolved against whatever directory the daemon happened to start in is not a
reproducible fact.

The component is `Ready` only when the configuration parses, both files exist,
**and** the executable reports exactly the pinned Pi version. It is
`NotConfigured` when there is no `pi.json` — a host that never configured Pi
reads exactly as it did before this ADR. Everything else is `Blocked` with its
own error class (`pi_config_unusable`, `pi_executable_missing`,
`pi_extension_missing`, `pi_probe_failed`, `pi_probe_timeout`,
`pi_version_mismatch`).

Writing a configuration file is explicitly not evidence that Pi is installed.

Two properties are preserved deliberately:

- **Aggregation is unchanged (ADR-0023).** `pi` stays `required: false`, so a
  broken Pi never rewrites the required-set aggregate; it only keeps
  `first_conversation_ready` false.
- **The probe hands the child no credential.** The version probe clears the
  environment and rebuilds it from an OS-essentials allowlist, so an ambient
  Provider key in the daemon's environment cannot reach a Pi process through a
  readiness check. It is bounded by a five-second deadline so a hung executable
  cannot hang a readiness request, which has no response-side timeout.

A `Ready` `pi` component carries a `containment_claim: not-claimed` fact. It
means "the pinned Pi and the built Extension are present", never "Pi is
contained", "the supply chain is verified", or "a Gate passed".

## Consequences

- The first-conversation path can now become ready on a host that has installed
  a compliant Pi, without any Gate or Profile claim changing.
- `pi.json` is a new Personal configuration surface. It holds no secret, and
  P1-T08's installer is the natural writer of it.
- The daemon now spawns a short-lived child process during readiness
  evaluation. This follows the existing precedent of the native SecretStore
  backend, and is bounded and environment-scrubbed.

## Explicit non-claims

- No G0, B01-B12, C0/C1, Profile, release, sandbox or containment claim.
- Pi remains uncontained and non-authority. A ready `pi` component does not
  make Pi an authority, does not grant it a capability, and does not create an
  Effect or advance a Task.
- The npm SRI and source commit in the compatibility pin are **not** trusted
  signature or provenance evidence; the supply-chain verifier remains Pi P2.
- P0-T06's `extension-load` verb was executed on the designated local
  Linux-native experimental host on 2026-07-27. Its redacted record is only
  PoC/non-claim evidence; it is not containment, Profile, Gate or release
  evidence.

## Remainder-of-P1-T07 Provider proxy decision (2026-07-27)

The daemon owns a non-streaming OpenAI-compatible provider proxy at
`POST /provider/v1/chat/completions`. The route requires a `management` channel
bearer, then resolves the Provider material through `ProviderKeyService` and
attaches the generated `Authorization: Bearer ...` header only to the outbound
request. The local caller never receives the Provider credential. The proxy is
an egress adapter only: it does not create an Intent or Effect, grant a
capability, write SQLite, or advance Task/Verification state.

### Production `ProviderTransport`: Rustls HTTP, not a subprocess

The production implementation is `RustlsProviderTransport` in the daemon
composition root. It uses blocking `reqwest` with the Rustls TLS backend;
`cognitive-secret` remains transport-injected and HTTP/TLS dependency-free, as
ADR-0021 requires. The transport requires HTTPS, rejects URL user-info and
header injection, disables redirects, bounds the response to 1 MiB, and applies
the configured request deadline.

A subprocess backend is rejected for this surface. Unlike the native Secret
Store adapter, Provider egress is ordinary daemon HTTPS and does not need an OS
service bridge. A subprocess would add credential-passing, lifecycle,
cancellation and output-redaction surfaces without improving the daemon-owned
secret boundary. In particular, command arguments and inherited environment
are forbidden credential paths.

### Streaming scope (P1 unary; P8-T11 public SSE)

P1 first conversations use a bounded non-streaming request/response exchange.
The private-candidate Provider proxy and the Pi Extension conversation client
still send `stream: false`. `PERSONAL_PROVIDER_STREAMING_UNSUPPORTED` remains
the fail-closed code when a private-candidate request asks for a stream.

P8-T11 adds the deferred public-proxy contract: a management-channel
`stream: true` request is forwarded to the selected model as SSE, and the
daemon flushes upstream bytes to the local caller without waiting for a complete
unary JSON body. Selected-model, SecretStore, and HTTPS-only pins are unchanged.
The daemon still does not implement local-disconnect-to-upstream-abort; a dropped
loopback client does not become a cancellation authority. Trailing
`X-CognitiveOS-Provider-Network-Nanos` is omitted on streaming success responses
because the total is unknown when headers are flushed.

This decision implements only the daemon egress boundary. The pinned Pi
Extension API mirror does not currently expose a supported completion-provider
registration or interception hook, so the package is not yet wired to consume
this route for a Pi conversation. It must not compensate by reading
`provider.json`, resolving a secret, or configuring an independent Provider.
That remaining compatibility integration keeps P1-T07 in progress.

## Alternatives rejected

1. **Depend on `@earendil-works/pi-coding-agent` for its types.** Rejected: it
   places Pi in the lockfile and therefore in every install and CI job, against
   ADR-0025. A drift-checked structural mirror gives the same type safety.
2. **Block only `bash`/`edit`/`write`.** Rejected: it treats the denylist as the
   security boundary. Any tool Pi adds later would be permitted by default, and
   the Extension has no authority to permit anything.
3. **Let the Extension read `provider.json` and proxy the key itself.**
   Rejected by ADR-0018 and ADR-0020: clients are non-authority and must not own
   secret persistence or resolution.
4. **Derive the `pi` component from repository presence** (e.g. "the package
   directory exists"). Rejected: that is static analysis rewritten as runtime
   readiness, which ADR-0023 exists to prevent.
5. **Make `pi` a required component.** Rejected: it would change ADR-0023's
   aggregation semantics and would turn an optional client surface into a
   blocker for unrelated required checks.
