# CognitiveOS Personal Linux 1.0 Scope

- Product version target: `1.0.0`
- Release Gate: `GMVP-LINUX`
- Platform target: Linux x86_64
- Decisions: [ADR-0036](../../adr/0036-personal-linux-1-0-and-official-pi-acquisition.md),
  [ADR-0037](../../adr/0037-personal-unified-cognitive-resource-substrate.md),
  [ADR-0038](../../adr/0038-personal-agent-sidecar-linux-evolution-boundary.md)

This document defines the stable release target. Current readiness, task state
and every Gate status remain exclusively in
[PROGRESS.md](../../plan/PROGRESS.md).

## 1. Release identity and authority

Linux 1.0 is the first public realization of CognitiveOS Personal as a
single-user local Agent cognitive-resource substrate. It delivers a minimum
real slice of Memory, Skill, Tool, Context, Task and Runtime/Process. These are
six user-visible families, not rows in a universal Resource table.

The Rust daemon is the only authority writer. Pi, the Pi sidecar, the
Pi-hosted Shell, CLI, SDK and future UI are clients. They cannot authorize,
advance state, commit Effects, reconcile or decide Task completion.

Budget, Permission, Model, Artifact, Intent/Effect, Evidence and Event are
cross-cutting objects included where the six families require them.

## 2. Required product foundation

| Capability | Linux 1.0 target requirement |
|---|---|
| Product topology | one `cognitiveos-personal.service` user unit and numeric loopback `127.0.0.1:48181` |
| Local modes | desktop, headless and foreground use the same production-signed artifact, daemon and application services |
| Installation trust | safe extraction, production signing, SBOM, attestation and immutable release manifest |
| Secret boundary | desktop Secret Service or approved headless encrypted vault; locked start + SSH TTY unlock, with optional systemd encrypted-credential unlock material; no plaintext fallback or Provider/user secret in units/credentials/SQLite/config/argv/logs/evidence |
| Provider/model | daemon-owned Provider egress, active capability probe and selected-model snapshot |
| Workspace | low-friction Standard Workspace plus Extended Home selected document/project roots and optional ordinary outbound network; credential/authority/system/privilege paths stay hard-denied |
| Agent Shell | Pi-hosted natural-language Shell plus deterministic `cognitive` fallback on the same daemon services |
| Agent integration | versioned per-Agent sidecar boundary; sidecar remains a non-authority client |
| Managed Agent | exact official Pi npm package, verified acquisition lock, registration/instance/sidecar health and lifecycle |
| Product lifecycle | update, rollback and uninstall with durable receipts or explicit incomplete outcome |
| Data operations | backup/restore excluding secret material |
| Supportability | redacted doctor/support bundle with stable error guidance and explicit unknown/not-run facts |

## 3. Minimum real resource slices

### 3.1 Memory

Linux 1.0 must exercise `MemoryCandidate -> MemoryAdmissionDecision ->
MemoryObject` from both explicit user `remember` and an Agent proposal. The
slice includes scope, purpose, provenance, version, conflict handling, expiry,
forget and tombstone. Daemon-owned SQLite is the source of truth; FTS5 and
metadata filters are derived retrieval indexes.

Embeddings, vector storage, graph Memory and automatic full-conversation
extraction are not required.

### 3.2 Skill

Skill is an independent first-class family. Linux 1.0 supports immutable local
package/revision import compatible with `SKILL.md` plus bounded `resources/`
and `scripts/`, and install, list, inspect, pin, enable, disable and remove.

Authorized instructions/resources may enter Context. Scripts run only through
a separately registered Tool. A Skill grants no permission and cannot execute
directly. Marketplace, chaining and automatic download are not required.

### 3.3 Tool

The static Tool registry must cover:

- Standard Workspace read and search;
- Standard Workspace write and patch with a recoverable journal;
- bounded process and check execution;
- read-only HTTP fetch.

Descriptors and availability are daemon facts. Unknown, descriptor-drifted,
disabled or quarantined Tools have dispatch count zero. External or
irreversible mutations use persisted Intent/Effect; a reversible local
workspace mutation uses the bounded journal rather than ambient Agent writes.

### 3.4 Context

Every admitted Task has a real `ContextRequest` and `ContextView` that can
combine Task/current state, Memory, Skill instructions/resources, Tool
summaries, artifacts/evidence, workspace inputs and explicit Task inputs.

Authorization and filtering precede ranking. Required unavailable input fails
closed. Omission, truncation, conflict, staleness and budget loss are explicit.
Selection uses deterministic priority, metadata filters and FTS with stable
tie-breaking. Views preserve a stable prefix, canonical digest and bounded
delta. Complex learned ranking is not required.

### 3.5 Task

The Task slice retains durable raw intent, a server-issued digest-bound
preview, exact admission under CAS/epoch guards, budgets, scheduler/watch,
checkpoint, Intent/Effect and independent verification. Provider response,
Agent output, sidecar success and process exit do not complete a Task.

A future `TaskContract` is intended to fix exact resource refs/constraints,
adapter identity and Context policy. Those future fields are not claimed as
implemented or required by this document without a separate contract change.

### 3.6 Runtime/Process

The Runtime slice keeps package, installation, registration, instance,
sidecar, execution and process identities separate. Process is daemon-owned
observation/supervision data and does not create a new domain or completion
authority.

Pi is the only Agent/sidecar combination qualified for Linux 1.0. The generic
sidecar framework is reusable but transfers no Pi evidence to another Agent.

## 4. Information architecture requirement

The Linux 1.0 Shell and deterministic projection model use:

- **Home**;
- **Agents**;
- **Tasks**;
- **Resources**, with Memory, Skills, Tools and Context;
- **Activity**, with Run, Process, Effect and Evidence.

Model, Budget, Permission, Artifact and Event appear contextually rather than
as additional top-level spaces.

## 5. Gate composition target

`GMVP-LINUX` targets the composition:

`B01 + B02 + B03 + B04 + B05 + B08 + B09 + B12 + P7 operability`

For this composition, P7 operability means the production release operations
required by P7-T08, including signing/trust, update/rollback/uninstall,
backup/restore, doctor/support and release-manifest closure. It is not a new
parallel Gate.

`B06`, `B07`, `B10` and `B11` do not block Linux 1.0:

- B06/B07 advanced Context efficiency/optimization evidence is deferred beyond
  the deterministic Context slice and B03 target;
- B10 broad Tool/MCP ecosystem qualification is deferred beyond the static
  Linux 1.0 Tool family;
- B11 Multi-Agent value qualification is deferred and default-off.

Passing one component cannot replace another. This section sets target
composition only. Whether any Gate is `not-run`, `running`, `pass`, `fail` or
`blocked` is stated only by `PROGRESS.md` and preregistered campaign evidence.

## 6. Pi-only sidecar qualification

Linux 1.0 must bind exact Pi package, installation, registration, sidecar,
instance and execution identities and qualify drift, channel separation,
permission, lifecycle, recovery and out-of-band mutation negatives.

OpenClaw, Hermes, Codex, WorkBuddy and other Agents require independent
package/protocol/sidecar identity, capability, sandbox, lifecycle, recovery,
negative campaign and release inclusion decisions. MCP integration does not
inherit Tool or Agent support merely because a bridge can connect.

## 7. Explicitly deferred

- embedding/vector/graph Memory and automatic extraction of all conversations;
- Skill marketplace, chaining, automatic download and autonomous dependencies;
- learned/complex Context ranking and advanced B06/B07 optimization claims;
- broad dynamic Tool catalogs and general MCP ecosystem qualification;
- OpenClaw, Hermes, Codex, WorkBuddy and every non-Pi Agent qualification;
- Multi-Agent delegation/orchestration;
- Web UI and independent Console product;
- Windows installer, service and credential-store parity;
- Linux aarch64, macOS, mobile and WSL2 as product platforms;
- enterprise approval chains, multi-tenancy, HA and cloud sync.

These may be developed in isolated tracks after their implementation
requirements are met, but cannot expand a Linux 1.0 release statement.

## 8. Unsupported or forbidden in 1.0

- a universal Resource table or one state machine for all families;
- redefining `CognitiveResourceManifest` beyond ActivityContext discovery;
- non-loopback daemon binding or a second authority writer;
- Provider/user keys in Pi, sidecars, argv, ordinary config, SQLite, logs,
  Context, Memory or evidence;
- ambient full-home filesystem access;
- Secret Store contents, SSH/GPG keys, browser credential/profile stores,
  CognitiveOS authority/bootstrap data, Docker/system sockets, system
  directories or privilege management through Extended Home;
- Pi built-in tools or Skill scripts as an authority bypass;
- dispatch of unknown, drifted, disabled or quarantined Tools/sidecars;
- unpinned/latest Agent acquisition or treating npm SRI as publisher signature;
- installing an Agent or Skill and automatically granting runtime capability;
- blind redispatch after an unknown external outcome;
- marking a Task complete from Provider response, Agent/sidecar output, Tool
  result or process exit;
- separate desktop/headless/foreground authority implementations;
- kernel module, eBPF control plane, device scheduler or distributed authority;
- claims of Windows install parity, containment or CognitiveOS Profile
  implementation.

## 9. Release evidence composition

The release campaign must identify exact:

- Linux image/environment, mode and reset procedure;
- product source revision, artifact digest, signing key and attestation;
- Node version, Pi package/version/SRI/digest and sidecar digest;
- native Secret Service behavior and cleanup;
- headless vault locked start, TTY unlock and optional unattended unlock without
  Provider/user secret in service or credential material;
- Standard Workspace and Extended Home negative boundaries;
- B01 fixed six-outcome denominator, at-least-five-success threshold,
  statistics, and independent-verifier closure as defined by ADR-0039;
- Memory candidate/admission/object, retrieval and forget/tombstone cases;
- Skill import/revision/action/permission and script-through-Tool cases;
- Tool registry/availability/journal/Intent-Effect failure cases;
- per-Task ContextRequest/View, authorization-before-ranking, required-source
  failure, loss, stable-prefix/digest/delta cases;
- Task, scheduler, checkpoint, Effect and independent verifier cases;
- Pi acquisition/lifecycle/sidecar/recovery and identity-separation cases;
- update, backup/restore, doctor and support-bundle checks;
- independent verifier identity and evidence collector version.

Ordinary CI, WSL, fixtures and experimental native hosts remain implementation
evidence unless a preregistered campaign explicitly includes them.

## 10. Release statement template

A valid post-Gate release statement is bounded:

> CognitiveOS Personal 1.0 supports Linux x86_64 with the pinned, qualified Pi
> Agent, Pi sidecar and Pi-hosted Shell. It provides the executed minimum
> Memory, Skill, Tool, Context, Task and Runtime/Process capabilities listed in
> the release manifest through one daemon authority. Other Agents, advanced
> retrieval/ranking, Skill marketplaces, MCP, Multi-Agent, Web UI, Windows
> installation, kernel/hardware control and Profile conformance are not
> included.

Before `GMVP-LINUX` passes, the same wording must use "target" or "planned" and
must not say "supports" or "released".
