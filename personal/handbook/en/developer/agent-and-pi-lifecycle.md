---
doc_id: dev.agent-pi-lifecycle
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: personal/crates/cognitive-runtime/src/installer.rs
    symbols: ["install_package", "acquire_official_pi_durable"]
  - path: personal/crates/cognitive-runtime/src/agent_registration.rs
    symbols: ["register_official_pi_agent_durable", "activate_official_pi_agent_durable"]
  - path: personal/crates/cognitive-runtime/src/pi_launcher.rs
    symbols: ["admit_pi_launch"]
  - path: personal/packages/pi-cognitiveos/src/pi-route-observation.ts
  - path: personal/packages/pi-cognitiveos/src/extension.ts
  - path: personal/apps/pi-agent-adapter/src/lib.rs
  - path: personal/apps/pi-agent-adapter/src/main.rs
  - path: personal/apps/kernel-server/src/personal/pi_runtime.rs
  - path: personal/crates/cognitive-runtime/src/agent_adapter_manifest.rs
    symbols: ["register_agent_adapter"]
  - path: personal/crates/cognitive-runtime/src/non_pi_agent.rs
  - path: personal/crates/cognitive-runtime/src/dsh_agent.rs
    symbols: ["register_dsh_adapter"]
  - path: core/crates/cognitive-akp/src/deepseek_harness.rs
    symbols: ["DeepSeekHarnessAdapter"]
  - path: core/crates/cognitive-akp/src/bin/dsh-akp-bridge.rs
  - path: personal/packages/dsh-akp-adapter/src/index.ts
  - path: personal/packages/dsh-akp-adapter/src/plugin.ts
    symbols: ["apply", "applyDshAkpCordisPlugin"]
  - path: personal/packages/dsh-akp-adapter/src/index.test.ts
  - path: personal/apps/admin-cli/src/personal_cli/dsh.rs
    symbols: ["configure", "launch", "status"]
  - path: personal/packages/dsh-akp-adapter/scripts/dsh-real-process.mjs
  - path: personal/packages/dsh-akp-adapter/scripts/dsh-web-preflight.mjs
  - path: personal/packages/dsh-akp-adapter/scripts/paired-path.mjs
  - path: personal/docs/product/agent-integration-and-conversations.md
  - path: personal/docs/product/agent-integration-and-conversations.zh-CN.md
  - path: personal/docs/architecture/agent-shell-and-agent-lifecycle.md
  - path: personal/docs/architecture/agent-adapter-contract.md
  - path: personal/docs/architecture/multi-agent-orchestration.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: personal/docs/architecture/windows-host-background.md
tests:
  - personal/crates/cognitive-runtime/tests/p5_t01_pi_acquisition.rs
  - personal/crates/cognitive-runtime/tests/p5_t02_agent_registration.rs
  - personal/crates/cognitive-runtime/tests/p5_t05_identity_recover.rs
  - personal/crates/cognitive-runtime/tests/p5_t05_upgrade_fencing.rs
  - personal/apps/admin-cli/tests/p2_t27_pi_lifecycle.rs
  - personal/packages/pi-cognitiveos/src/pi-route-observation.test.ts
  - personal/apps/pi-agent-adapter/tests/daemon_candidate_protocol.rs
  - personal/apps/kernel-server/tests/p2_t31_live_daemon_scheduler.rs
  - personal/apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - personal/apps/admin-cli/tests/p2_t33_private_candidate_host_path.rs
  - personal/packages/dsh-akp-adapter/src/index.test.ts
fingerprint: "sha256:26db874d43c3ce1821d76ee098e49a04eb2e713768550eeb4bb3c988ef1c068f"
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
Pi's runtime registry, then activates only those two names. Before each agent
turn, it repeats that activation after the runtime registry is available;
unknown names are ignored, so CognitiveOS fails closed if either name is absent
from Pi's actual registry. The CLI's explicit `--tools` list is the full Pi
registry allowlist, so this cannot activate Pi-native filesystem, shell, or
mutating tools.

The shell-host Provider route has an opt-in, non-authority campaign observer.
One opaque id correlates each concurrent Pi request with two daemon-measured
stages; Node and Rust monotonic durations remain separate clock domains.
Completed, cancelled and failed attempts have explicit terminal records, while a
disabled session emits none. Pi conversations stay unary (`stream:false`). The
public management Provider proxy may forward `stream:true` as SSE. Provider usage is never estimated or
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

### Personal 2.0 OPC managed-Agent target

`Requires-backend + Requires-environment`: DSH is the preinstalled managed
Installed Agent and default digital-employee runtime. Product form is an exact
audited official artifact in an immutable installation slot, run as a
Personal-managed isolated child through a bounded stdio broker and daemon
Provider proxy. Settings exposes source/version/digest, health, Windows
sandbox qualification, employee/Task bindings, update and rollback.

DSH is not linked in-process or vendored. It owns no Personal Conversation,
archive, Memory, Task, Effect or completion. Its native UI/conversation is not
embedded or synchronized. Environment/plaintext credentials, native MCP/base
tools, HMR and home patch remain denied.

Pi separately powers the user-visible Personal Assistant as a hidden,
candidate-only, default-deny engine. Pi is not an ordinary Installed Agent in
the OPC target and owns no authority, Secret, archive or Memory.

Project Manager and employee collaboration remains daemon-owned through
Project/Plan/Task/Attempt/artifact/handoff facts. Hermes, Codex, Cursor and
other products are future independently qualified candidates; DSH/Pi/Linux
evidence does not transfer.

DeepSeek Harness is explicitly a **Developer Preview** Agent product, not a
DeepSeek model or Provider. The current bridge is a candidate-only adapter. The Rust side pins
the exact dsh git revision and the AKP request-envelope schema digest, fences
a process-local session, enforces monotonic sequences, and rejects
authority-shaped and secret-shaped payloads. `POST /task/akp/dsh` must be
activated after daemon start; a restart empties the session table and fails
closed. Workspace* candidates map onto the existing public candidate
admission path using the native catalog: WorkspaceRead is parameter-free
(digest still covers `{"family":"WorkspaceRead"}`); WorkspaceSearch requires
a query; WorkspaceWrite/Patch require canonical `input_b64` and `preimage`.
The TypeScript shim sends snake_case JSONL or HTTP frames over
a long-lived, length-bounded transport. It never receives Provider
credentials, writes authority state, or treats a dsh response as Task
completion. Live linux-002 runs are implementation evidence, not a Gate,
release, Profile, B01, or Agent-benefit result. Timing fields are measurement
hooks, not a zero-overhead claim. `personal/packages/dsh-akp-adapter/scripts/linux002-e2e.mjs`
drives `attachDshCordisPlugin` over HTTP on an identity-confirmed linux-002
runtime and waits for Task `COMPLETED`. `personal/packages/dsh-akp-adapter/src/plugin.ts`
is the Cordis `apply` entry for `dsh --patch`; `scripts/dsh-real-process.mjs`
starts pinned dsh with compiled `apps/cli/lib/bin.js` when host `build:lib`
outputs exist, otherwise `node --import tsx/esm apps/cli/src/bin.ts` (not
`pnpm dsh`), loads `plugin.bundle.cjs` because Node 22.23 rejects `require()` of
the ESM `plugin.js`, admits disposable WorkspaceRead/Search/Write Tasks, submits those candidates
as plugin `startupEvents` from the real dsh process, and routes Flash through
the daemon Provider SSE proxy (`POST /provider/v1/dsh/chat/completions` with
`stream:true`). The interactive native panel pins
`llm-deepseek.maxTokens` to the bound LongCat route's 131,072-token maximum:
a smaller 256-token budget can be consumed before a reasoning-capable model
emits assistant content, while dsh's 256,000-token default is rejected
upstream. Bounded one-shot probes use a separate 4,096-token budget. The daemon normalizes only null-valued continuation fields
(`id`, `type`, `function.name`, and `function.arguments`) in streaming
OpenAI-compatible `tool_calls` frames before they reach dsh; this preserves the
opening frame's identity instead of allowing an upstream `null` to overwrite
the accumulated tool name. It does not invent or authorize a tool call.
Product install
is `cognitive dsh configure` then `cognitive dsh launch` (Path B). `cognitive dsh web`
starts the native panel (`dsh --profile web --no-open`, default
`http://127.0.0.1:3080`) after `pnpm run build` has produced `apps/web/dist`; it
is not Personal `/ui/`. Web Path B writes `$DSH_HOME/settings.yaml` so
`llm-deepseek` stays on `POST /provider/v1/dsh/chat/completions` and aliases the
official Models catalog ref to the daemon management bearer — not a SecretStore
copy and not a dsh `.env` key. Native Models is the current dsh-bound account
catalog (not leftover Cos/DeepSeek ids). Binding set/remove and `op: apply`
rewrite that overlay; Cos-installed web reloads it. `cognitive dsh status`
reads `GET /personal/dsh/runtime`. `POST /personal/dsh/runtime` `op: apply`
publishes the Cos `agent://personal/dsh` binding as Path B selected-model and
reloads native Models from that catalog. `op: clear`
drops the bound pid and in-memory sessions so the projection is `INACTIVE`.
`op: apply` is accepted only for an already-`ACTIVE` runtime and is limited to
supported binding/model overlay synchronization. It is not a session-refresh
path after daemon restart: the new daemon has no prior runtime registration,
projects dsh as `INACTIVE`, and rejects `apply`.
Direct Flash
(`--path a`) is measurement-only via `scripts/paired-path.mjs`. Adapter
registration digest in `dsh.json` is not SQLite-durable daemon adapter state.
Both remain implementation evidence only.

After a daemon restart, the current dsh Path B web process can retain a stale
management session and surface the resulting 401 as "API key invalid." The
required recovery is to restart `cognitive dsh web`, then inspect
`cognitive dsh status`; `apply` is rejected while the new daemon reports
`INACTIVE`. No
approved non-logging direct-bearer probe exists: never extract the credential
or pass it on process argv. Persisted Provider account `active` is not a live
SecretStore-resolution result; discovery/proxy use performs live resolution,
so locked or changed store state remains a separate possible cause. See the
[tracked defect](../../../../docs/bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md).
