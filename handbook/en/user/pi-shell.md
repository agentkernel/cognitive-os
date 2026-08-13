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
  - path: packages/pi-cognitiveos/src/tool-policy.ts
  - path: apps/admin-cli/src/personal_cli/pi.rs
tests:
  - packages/pi-cognitiveos/src/extension.test.ts
  - packages/pi-cognitiveos/src/daemon-provider.test.ts
  - packages/pi-cognitiveos/src/safety.test.ts
fingerprint: "sha256:eeb94b66886c6d42ae623c362e334abd8ec9563e7ffd9d001757f2f44f391213"
non_claims:
  - Pi remains a candidate-producing client; nothing in the shell can advance authority state, and conversation quality/benefit is not claimed.
---

# The Pi shell

`partial`: daemon-proxied conversation, readiness display, and a status command are
implemented; agent tool use and the resource/task browsing surfaces are deliberately
not available in the shell yet.

## What works today

Launch Pi through `cognitive pi launch`. The CognitiveOS extension then:

- discovers the daemon via `daemon-endpoint.json` and authenticates with the
  per-boot bootstrap secret (management + task bearers, kept separate);
- registers the `cognitiveos` model provider: your prompts go Pi → daemon Provider
  proxy → your Provider. The Pi process never sees the API key;
- shows daemon readiness at session start and warns when the first conversation is
  blocked;
- answers `/cognitive-status` with daemon facts only.

Responses are one-shot: the daemon requests a non-streaming completion and the
extension emits it as a single block (text only; images/tool-calls are rejected).

## What is deliberately locked

- `project_trust` is always denied and **every** Pi built-in tool (including
  read-only ones) is refused by the tool policy — the shell cannot touch your files
  or run commands.
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
`COGNITIVEOS_PI_ROUTE_OBSERVATION_CAMPAIGN=<campaign id>` before launching Pi and
each request additionally publishes one in-memory observation: the seven stages
of the route (request preparation, extension dispatch, loopback wait, daemon
preflight, Provider network, response parse, event delivery) as monotonic
durations, joined to the daemon side by one opaque correlation id, plus the same
Provider usage described above.

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
skills, sessions, and extensions except the pinned candidate extension. See
[Agent and Pi lifecycle](../developer/agent-and-pi-lifecycle.md).
