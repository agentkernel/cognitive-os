---
doc_id: user.pi-shell
locale: en
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: packages/pi-cognitiveos/src/extension.ts
    symbols: ["registerCognitiveOsExtension"]
  - path: packages/pi-cognitiveos/src/daemon-provider.ts
  - path: packages/pi-cognitiveos/src/pi-route-observation.ts
  - path: packages/pi-cognitiveos/src/tool-policy.ts
  - path: apps/kernel-server/src/personal/pi_runtime.rs
  - path: apps/admin-cli/src/personal_cli/pi.rs
tests:
  - packages/pi-cognitiveos/src/extension.test.ts
  - packages/pi-cognitiveos/src/daemon-provider.test.ts
  - packages/pi-cognitiveos/src/pi-route-observation.test.ts
  - packages/pi-cognitiveos/src/safety.test.ts
  - apps/kernel-server/tests/p2_t31_live_daemon_scheduler.rs
  - apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - apps/admin-cli/tests/p2_t33_private_candidate_host_path.rs
fingerprint: "sha256:add07d67904390629e85cd7f69f5a37107ccacd033edd071915c74a40be9deca"
non_claims:
  - Pi remains a candidate-producing client; nothing in the shell can advance authority state, and conversation quality/benefit is not claimed.
---

# The Pi shell

`partial`: daemon-proxied conversation, readiness display, a status command, and
advertised daemon-governed WorkspaceRead/Search/Write/Patch are implemented; Pi
native filesystem/shell tools stay refused. Resource/task browsing surfaces are
deliberately not available in the shell yet.

## What works today

Launch Pi through `cognitive pi launch`. It loads only the configured Extension,
uses Pi's explicit tool allowlist to exclude Pi-native tools, and leaves only daemon-governed Workspace tools available
to the conversation. Optional `--append-system-prompt <absolute-path>` forwards an
existing non-empty UTF-8 file to Pi; relative, missing, and empty files fail closed,
and the file bytes are not printed. Its fixed Pi allowlist contains only `WorkspaceRead`,
`WorkspaceSearch`, `WorkspaceWrite`, and `WorkspacePatch`; it never activates
Pi's native file or shell tools. With
`--runtime-root`, the CLI maps its hermetic layout roots into Pi's XDG environment
so the Extension resolves that runtime's endpoint and local bootstrap secret. The
CognitiveOS extension uses Pi's required runtime tool schemas before it exposes
those tools, then:

- discovers the daemon via `daemon-endpoint.json` and authenticates with the
  per-boot bootstrap secret (management + task bearers, kept separate);
- registers the `cognitiveos` model provider: your prompts go Pi → daemon Provider
  proxy → your Provider. The Pi process never sees the API key;
- shows daemon readiness at session start and warns when the first conversation is
  blocked;
- answers `/cognitive-status` with daemon facts only.

Responses are one-shot: the daemon requests a non-streaming completion. It emits
bounded text or one structured daemon-governed Workspace tool call; images and
unsupported tool-call shapes are rejected. When Pi supplies the corresponding
tool result, the next daemon request preserves the bounded assistant tool call
and result using the Provider's standard tool-continuation message roles.

## What is deliberately locked

- `project_trust` is always denied. Pi's native bash/write/edit (and other
  built-ins) stay refused. The Extension advertises daemon-governed
  WorkspaceRead/WorkspaceSearch/WorkspaceWrite/WorkspacePatch whose execute path does not
  touch the filesystem; the daemon admits those arguments as candidates on the
  Intent/Effect path. A JSON-only candidate from the model is not trusted as a
  digest: the adapter recomputes `parameters_digest` from `parameters` when
  present, including an omitted, empty, or otherwise invalid model-supplied digest. Adapter diagnostics preserve a redacted tail error and distinguish usage failures (exit 2) from runtime failures (exit 3). Pi native filesystem tools are not the C1/C2 arm.
- No resource browsing, task submission, or watch UI inside Pi yet: those client
  methods exist in `PersonalDaemonClient` and the CLI (`cognitive resource|task`),
  but are not wired into shell UX.
- Model parameters are fixed by the daemon's selected model. Token usage is shown
  only when the Provider returned complete, internally consistent counters;
  otherwise it is left unavailable rather than estimated. Cost is never shown,
  because no priced source is bound to the shell.

## Campaign measurement is off unless you turn it on

An ordinary session measures nothing. Set both
`COGNITIVEOS_PI_ROUTE_OBSERVATION=enabled` and
`COGNITIVEOS_PI_ROUTE_OBSERVATION_CAMPAIGN=<campaign id>` before launching Pi.
The daemon process must see the same enable variable, otherwise the two nested
daemon stages stay `not_available` rather than being joined. Each request then
publishes one in-memory observation. A completed request contains all seven route
stages (request preparation, extension dispatch, loopback wait, daemon preflight,
Provider network, response parse, event delivery); a cancelled or failed request
retains the measured Pi-stage prefix instead of disappearing from the sample
denominator. Every record states `requestMode=non_streaming`, a terminal outcome,
the last measured Pi stage, and a content-free failure class such as
`provider_unavailable` or `protocol_error`. The product rejects an upstream
`stream:true` request before secret resolution, so it is not represented as a
successful streaming observation.

Pi stages use Node's monotonic clock and daemon stages use Rust `Instant`.
Wall-clock timestamps are not part of the observation, and stages from the two
clock domains must never be subtracted. The opaque correlation id only joins the
same request; concurrent requests receive different ids. Token counters become
`measured` only when the authenticated daemon-response parser saw a complete,
internally consistent Provider usage object. A caller cannot publish hand-built
counters as measured evidence; missing or inconsistent usage remains
`not_available`. This is provenance inside the product path, not a
cryptographic attestation that an upstream Provider counted correctly.

An observation carries durations and counters only — never a prompt, a response,
a header, a bearer or a Provider key — and the shell writes nothing to disk for
it. `COGNITIVEOS_PI_ROUTE_OBSERVATION_SINK` names an absolute `.ndjson` path for
a campaign harness that embeds the extension; the shell itself never opens it,
and a path inside a CognitiveOS state, runtime or config directory is refused.
Stage timings are measurement, not a performance result: they support no benefit,
Gate, release or Profile claim.

## The other Pi role

Separately from the shell, the daemon manages Pi as a governed agent (acquisition,
registration, sidecar sessions) and can launch a locked-down Pi child process to
produce **candidates** over a private one-shot socket — that path disables tools,
skills, sessions, and extensions except the pinned candidate extension. A test
stub may emit the same untrusted candidate on stdout without using the Provider
completion socket. The completion socket itself is bound under a short host
path (`$XDG_RUNTIME_DIR/cognitiveos/pc-<pid>-<seq>.sock`, then the process
temp directory, then `/tmp/cognitiveos`) so a long `--runtime-root` cannot
exceed Linux `UNIX_PATH_MAX`; Pi `--work-dir`/`--config-dir` may still live
under the runtime root. Adapter and Pi stderr is a redacted `daemon.log` fact
when the adapter rejects. See
[Agent and Pi lifecycle](../developer/agent-and-pi-lifecycle.md).
