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
  - path: apps/pi-agent-adapter/src/lib.rs
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
fingerprint: "sha256:dedfe44799e541e0f41bcc15f198f7e5cc5117a8bc6ba2d0a602cbfd84289ce5"
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
HTTPS proxy endpoint. It passes only `--extension <absolute-path>`.

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
WorkspaceSearch/Write/Patch; the adapter maps one such tool call onto the
P2-T21 candidate path. The daemon treats the output as a candidate for
admission — nothing more. A test stub adapter may emit that untrusted candidate
on stdout without connecting to the Provider completion socket; the daemon still
validates descriptor, digest, and authorization.

## Beyond Pi

The Universal Agent Adapter Contract (`agent_adapter_manifest`) registers
AKP-speaking adapters (public listeners and authority writers rejected;
candidate-only capabilities), with epoch-fenced lifecycle. The first non-Pi
qualification (OpenAI Codex CLI) is a fixture-scoped identity/lifecycle matrix
proving independence from Pi evidence — explicitly not a network or binary
integration.
