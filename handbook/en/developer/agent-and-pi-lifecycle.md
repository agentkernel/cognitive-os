---
doc_id: dev.agent-pi-lifecycle
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-runtime/src/installer.rs
    symbols: ["install_package", "acquire_official_pi_durable"]
  - path: crates/cognitive-runtime/src/agent_registration.rs
    symbols: ["register_official_pi_agent_durable", "activate_official_pi_agent_durable"]
  - path: crates/cognitive-runtime/src/pi_launcher.rs
    symbols: ["admit_pi_launch"]
  - path: packages/pi-cognitiveos/src/pi-route-observation.ts
  - path: packages/pi-cognitiveos/src/extension.ts
  - path: apps/pi-agent-adapter/src/lib.rs
  - path: apps/pi-agent-adapter/src/main.rs
  - path: apps/kernel-server/src/personal/pi_runtime.rs
  - path: crates/cognitive-runtime/src/agent_adapter_manifest.rs
    symbols: ["register_agent_adapter"]
  - path: crates/cognitive-runtime/src/non_pi_agent.rs
tests:
  - crates/cognitive-runtime/tests/p5_t01_pi_acquisition.rs
  - crates/cognitive-runtime/tests/p5_t02_agent_registration.rs
  - crates/cognitive-runtime/tests/p5_t05_identity_recover.rs
  - crates/cognitive-runtime/tests/p5_t05_upgrade_fencing.rs
  - apps/admin-cli/tests/p2_t27_pi_lifecycle.rs
  - packages/pi-cognitiveos/src/pi-route-observation.test.ts
  - apps/pi-agent-adapter/tests/daemon_candidate_protocol.rs
  - apps/kernel-server/tests/p2_t31_live_daemon_scheduler.rs
  - apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - apps/admin-cli/tests/p2_t33_private_candidate_host_path.rs
fingerprint: "sha256:c2bd7577c1da6bd71a0c97945f2fc2d0229579598392fef6f35ac7fd98730257"
non_claims:
  - Pi qualification evidence transfers to no other agent; Codex qualification is a fixture-identity matrix with no network/binary claim. B09-class Gate accounting is owned by the formal plan.
---

# Agent and Pi lifecycle

Three separated stages — **install ≠ register ≠ activate** — each a durable
daemon-side authority record, all epoch-fenced.

## Acquisition and installation

`acquire_official_pi_durable` pins `@mariozechner/pi@0.81.1` by exact
`sha512-…` integrity: normalizes/validates the npm metadata URL against an
allowlist, verifies the tarball hash, repackages deterministically, and emits an
acquisition report whose failure classes are typed. `install_package` verifies
digest + signature ports and commits immutable installation evidence with **zero
capability grants**. A custom-project verifier path exists for local-operator
packages (path-safety + digest + local policy id).

## Registration and sidecar sessions

`register_official_pi_agent_durable` requires durable installation evidence and
binds the exact package digest; activation flips a single active pointer under
epoch CAS; `SidecarSession` binds a live process identity (`process_bound`),
enforces pause/resume/stop/recover transitions with fencing, and reports redacted
health. Upgrade/uninstall fence old epochs; recover/orphan negatives are tested.
`admin-cli` (`install/register/activate/activate-root/rollback/agent-*`) is the deterministic caller.

## Launch admission (shell host role)

`admit_pi_launch` fail-closes unless: Linux native (not WSL2/Windows), doctor
components all ready, sandbox adapter present, `pi.json` paths absolute and
existing, version exactly `0.81.1`, and model egress bound to the registered
HTTPS proxy endpoint. It passes the configured Extension, disables Pi-native
tools, and explicitly permits only `WorkspaceRead` and `WorkspaceSearch`.
Those Extension tools use the pinned Pi runtime's TypeBox schemas; JSON-shaped
lookalikes are rejected during live registration. Once Pi binds its session,
the Extension re-registers the same daemon-governed definitions to refresh
Pi's runtime registry, then activates only those two names; unknown names are
ignored. This post-bind step cannot activate Pi-native filesystem, shell, or
mutating tools.

The shell-host Provider route has an opt-in, non-authority campaign observer.
One opaque id correlates each concurrent Pi request with two daemon-measured
stages; Node and Rust monotonic durations remain separate clock domains.
Completed, cancelled and failed attempts have explicit terminal records, while a
disabled session emits none. The route is non-streaming; `stream:true` remains a
stable pre-secret-resolution refusal. Provider usage is never estimated or
accepted from a runner-built object.

## Candidate production role

`pi-agent-adapter` (pinned adapter, `daemon-candidate` capability only) runs the
locked-down Pi child: built-in filesystem/shell tools, skills, sessions and
extension discovery disabled (`--no-builtin-tools`), env allowlist, one-shot
private socketpair with byte caps and deadlines, structured `AdapterOutcome`
(never authority state). The CognitiveOS Extension advertises daemon-governed
WorkspaceRead/Search/Write/Patch; their I/O-free Extension handlers emit only
an untrusted candidate, and the adapter maps exactly one such tool call onto
the daemon candidate path. WorkspaceRead carries only its workspace target;
the parameterized families retain their bounded parameter handling. A
JSON-fallback candidate still has its
`parameters_digest` recomputed from `parameters` when present, including an
omitted, empty, or otherwise invalid model-supplied digest; otherwise the
digest must be `sha256:` plus 64 lowercase hex. The daemon treats the output as a candidate for
admission — nothing more. A test stub adapter may emit that untrusted candidate
on stdout without connecting to the Provider completion socket; the daemon still
validates descriptor, digest, and authorization. The completion socket is bound
under `$XDG_RUNTIME_DIR/cognitiveos/` (then the process temp directory, then
`/tmp/cognitiveos`) so the path fits Linux `UNIX_PATH_MAX`; this is independent
of daemon layout fail-closed behaviour when `XDG_RUNTIME_DIR` is absent. Linux
candidate spawn forwards a host allowlist (`HOME`, locale, `XDG_RUNTIME_DIR`,
TLS trust files) after `env_clear()` and never copies `DBUS_SESSION_BUS_ADDRESS`
or Provider keys. Adapter/Pi stderr retains its redacted tail error on
`daemon.log`; exit code 2 denotes usage errors and exit code 3 denotes runtime
failures, so a public skip stays attributable. The private-candidate Provider proxy
strips `tools`/`tool_choice` before forward, accepts one text choice that may
include `role=assistant`, and refuses `tool_calls`.

## Beyond Pi

The Universal Agent Adapter Contract (`agent_adapter_manifest`) registers
AKP-speaking adapters (public listeners and authority writers rejected;
candidate-only capabilities), with epoch-fenced lifecycle. The first non-Pi
qualification (OpenAI Codex CLI) is a fixture-scoped identity/lifecycle matrix
proving independence from Pi evidence — explicitly not a network or binary
integration.
