# CognitiveOS Personal Linux 1.0 Scope

- Product version: `1.0.0`
- Release Gate: `GMVP-LINUX`
- Platform: Linux x86_64
- Decision: [ADR-0036](../../adr/0036-personal-linux-1-0-and-official-pi-acquisition.md)

This document defines the stable release boundary. Current readiness and Gate
status remain in [PROGRESS.md](../../plan/PROGRESS.md).

## 1. Included and required

| Capability | Linux 1.0 requirement |
|---|---|
| Product topology | one `cognitiveos-personal.service` user unit, loopback `127.0.0.1:48181` |
| Installation trust | production-signed Linux bundle, safe extraction, SBOM and attestation |
| Secret boundary | native Secret Service; no plaintext fallback |
| Provider/model | daemon-owned Provider egress, active capability probe and selected-model snapshot |
| Agent Shell | Pi-hosted natural-language Shell plus deterministic `cognitive` fallback |
| Managed Agent | exact official Pi npm package, verified acquisition lock, registry/instance health and lifecycle |
| Task | one governed single-Agent Task/Loop with preview, admission, watch, control and recovery |
| Tool | at least one safe catalog-bound operation; no generic ambient shell |
| Budget/permission | deadline/retry/step/cost enforcement and Tier 0/1/2 capability policy |
| Effect/recovery | persist-before-dispatch, idempotency, epoch fencing, unknown-outcome reconciliation |
| Completion | criterion evidence and independent verifier; false-completion negatives |
| Product lifecycle | update, rollback and uninstall with durable receipts/compensation |
| Data operations | backup/restore excluding secret material |
| Supportability | redacted doctor and support bundle with stable error guidance |

Passing one component Gate cannot replace another. `GMVP-LINUX` composes the
formal B01, P2, managed-Pi B09 and P7 production-operability evidence.

## 2. Framework-ready but not multi-adapter support

Linux 1.0 must include reusable package acquisition, adapter identity,
installation, registry/instance lifecycle and qualification test seams. Pi is
the only adapter allowed in the product support claim.

The framework is successful when a future adapter can declare and test its own
package/protocol identity, capabilities, sandbox, lifecycle, recovery and
negative boundaries without changing daemon authority rules. It is not
successful merely because an adapter interface or manifest file exists.

## 3. Explicitly deferred

- OpenClaw, Hermes, Codex, WorkBuddy and all non-Pi Agent qualification;
- general MCP server/Tool ecosystem;
- durable governed Memory, FTS and embedding;
- broad Context source management and optimization benchmarks;
- Multi-Agent delegation/orchestration;
- Web UI and independent Console product;
- Windows installer, service and credential-store parity;
- Linux aarch64, macOS, mobile and WSL2 as product platforms;
- enterprise approval chains, multi-tenancy, HA and cloud sync.

These may be developed in isolated tracks after their implementation
requirements are met, but they cannot expand a 1.0 release statement.

## 4. Unsupported or forbidden in 1.0

- non-loopback daemon binding;
- Provider/user keys in Pi, argv, ordinary config, SQLite, logs or evidence;
- Pi built-in tools as an authority bypass;
- unpinned/latest Agent acquisition;
- treating npm SRI as publisher signature;
- installing an Agent and automatically granting runtime capability;
- blind redispatch after an unknown external outcome;
- marking a Task complete from Provider response, Agent output or process exit;
- claiming Windows install parity, containment or Core Profile implementation.

## 5. Release evidence composition

The release campaign must identify exact:

- Linux image/environment and reset procedure;
- product source revision, artifact digest, signing key and attestation;
- Node version, Pi package/version/SRI/digest and adapter digest;
- native Secret Service behavior and cleanup;
- B01 attempt denominator and statistics;
- P2 workload, Tool, failure injection and verifier;
- B09 acquisition/lifecycle/rollback/uninstall cases;
- upgrade, backup/restore, doctor and support-bundle checks;
- independent verifier identity and evidence collector version.

Ordinary CI, WSL, fixtures and experimental native hosts remain implementation
evidence unless the preregistered campaign explicitly includes them.

## 6. Release statement template

A valid release statement is bounded:

> CognitiveOS Personal 1.0 supports Linux x86_64 with the pinned, qualified Pi
> Agent and Pi-hosted Shell. It provides the executed Task, Tool, recovery and
> product-lifecycle capabilities listed in the release manifest. Other Agents,
> Memory, MCP, Multi-Agent, Web UI, Windows installation and Profile
> conformance are not included.

Before `GMVP-LINUX` passes, the same wording must use “target” or “planned” and
must not say “supports” or “released.”
