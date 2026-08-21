# Pi Integration Map for CognitiveOS Personal

**Status:** Informative implementation map

**Last updated:** 2026-08-02

**Decisions:** [ADR-0035](../adr/0035-personal-pi-shell-and-managed-agent-role-separation.md),
[ADR-0036](../adr/0036-personal-linux-1-0-and-official-pi-acquisition.md),
[ADR-0037](../adr/0037-personal-unified-cognitive-resource-substrate.md),
[ADR-0038](../adr/0038-personal-agent-sidecar-linux-evolution-boundary.md)

This document explains how the two Pi integration tracks compose. It is not a task backlog,
current-status ledger, release Gate, or evidence source. Task definitions come only from
[PERSONAL-DEVELOPMENT-PLAN.md](./PERSONAL-DEVELOPMENT-PLAN.md), current facts come only from the
`Current snapshot` in [PROGRESS.md](./PROGRESS.md), and writable paths come only from active leases in
[PARALLEL-LANES.md](./PARALLEL-LANES.md).

## 1. Two independent roles

Pi can participate in Personal in two roles without merging their identities or authority.

| Track | Pi role | Product purpose | Formal task ownership |
|---|---|---|---|
| A | Pi-hosted Agent Shell through Pi sidecar | Natural-language entry point for inspecting and proposing changes to the six cognitive resources | P2-T02, P2-T04 |
| B | Managed Pi Agent + per-Agent sidecar | Product-acquired, installed, registered, supervised, upgraded, rolled back, and uninstalled Agent integration | P5-T01, P5-T02, P5-T05, P7-T08 |

Track A does not prove that Pi is installed as a managed Agent. Track B does not grant its Pi
process Shell authority. One runtime may participate in both tracks, but shared bytes or a shared
process never merge credentials, sessions, capabilities, lifecycle state, or completion semantics.

## 2. Track A: Pi-hosted Agent Shell

### 2.1 Composition

```text
Pi-hosted Agent Shell (Interactive CLI; Extension is presentation-only)
  -> pinned per-Agent Pi sidecar
  -> daemon Resource/Task application services
  -> private versioned six-resource projection
  -> deterministic authority
```

The Shell, Extension and sidecar are non-authority clients. They may interpret natural language,
render daemon projections, collect confirmation, carry watch cursors, and submit typed proposals.
They may not write authority state, commit Effects, infer completion, or bypass daemon
authorization. In particular, the Extension must not hold daemon bootstrap, service-management,
installation-management or lifecycle authority; those operations remain daemon-owned application
services reached through the separately authorized management channel.

### 2.2 Channel separation

The implementation must keep these channels distinct:

| Channel | Examples | Required boundary |
|---|---|---|
| Task channel | create, inspect, watch, pause, resume, stop a Task; inspect six-resource projection | Task application services and Task-scoped capability |
| Management channel | install, activate, upgrade, roll back, uninstall an Agent or sidecar | Resource-management services, server-issued preview, stronger lifecycle capability |

Pi built-in mutating tools remain default-deny. Natural-language and deterministic CLI commands
must converge before authorization so the Shell cannot create a second policy path.

### 2.3 Reused implementation

- `packages/pi-cognitiveos`: presentation-only Pi Extension; no daemon bootstrap/management authority.
- `apps/agent-shell`: reusable task-channel/session/watch client core used behind the sidecar boundary.
- pinned Pi sidecar: versioned client transport and adapter/session/process identity boundary.
- daemon local API, `TaskApplicationService` and resource services: sole application-service boundary.
- approved `SecretStore` backend (desktop Secret Service or headless encrypted
  vault) and daemon Provider proxy: sole approved provider-secret path.

### 2.4 Acceptance mapping

P2-T02 owns Shell composition and must demonstrate:

- real Resource + Task API use rather than fixture-only state;
- private versioned six-resource projection and watch/recovery behavior;
- parity with deterministic CLI application services;
- sidecar Task/management bearer, cache, retry, cursor and projection isolation;
- no authority side effects from Pi output, Provider success, process exit, or `agent_end`.

## 3. Track B: managed Pi Agent

### 3.1 Acquisition and installation

P5-T01 owns the Linux 1.0 acquisition transaction:

```text
fixed official npm origin
  -> exact @earendil-works/pi-coding-agent@0.81.1
  -> package identity and version checks
  -> npm SRI and package/dependency digests
  -> exact sidecar package/protocol/adapter digests
  -> Node compatibility
  -> private immutable staging
  -> compatibility and health checks
  -> production-signed acquisition lock
  -> durable installation commit
```

Pi and Node are not bundled in the CognitiveOS release artifact. The admitted sidecar package and
its protocol/adapter pins are independently recorded in the acquisition lock and release manifest.
An incompatible or missing Node
fails closed; Pi acquisition must not silently download an unapproved Node runtime. npm SRI is an
integrity input, not a claim of publisher signature or provenance.

### 3.2 Registry, instance, and execution

P5-T02 owns these separate durable identities:

```text
AgentPackage
  -> AgentInstallation
  -> Agent definition and policy
  -> AgentInstance
  -> Task-bound AgentExecution
  -> SidecarPackage / SidecarInstallation
  -> SidecarSession
  -> supervised OS process
```

The following are also distinct: `ShellSession`, Pi session, sidecar session, sidecar process,
Agent process, Conversation, Task, Loop, and Effect.
Installation never implies activation or permission. A healthy process never implies Task
acceptance.

### 3.3 Lifecycle

Managed Pi lifecycle includes:

- register and inspect;
- activate and health-check;
- pause, resume, stop, and supervise;
- upgrade and rollback using immutable installations;
- uninstall while retaining or explicitly removing Personal-owned data according to policy;
- recover and reconcile interrupted lifecycle operations;
- emit redacted evidence for independent verification.

Every external mutation uses persist-before-dispatch Intent/Effect. `OUTCOME_UNKNOWN` is reconciled
with the original dispatch identity and idempotency key; it is never retried under a new key without
closure or quarantine.

### 3.4 Acceptance mapping

- P5-T01: Agent + sidecar acquisition, immutable installation, acquisition lock,
  upgrade/rollback/uninstall negatives.
- P5-T02: sidecar contract/registration, Agent/sidecar instance/session/process identity,
  health, supervision, lifecycle, and Pi foundation.
- P5-T05: managed-Pi + sidecar B09 evidence only. B10 belongs to the post-1.0 P5-T04
  Tool/MCP slice and is not coupled to P5-T05 completion.
- P7-T08: Linux 1.0 release composition and promotion through `GMVP-LINUX`.

## 4. Completion and authority invariant

Both tracks share the same deterministic completion path:

```text
Pi output or proposal
  -> daemon authorization and admission
  -> durable Task/Intent/Effect state
  -> supervised execution
  -> receipt or reconciliation
  -> evidence
  -> independent verification
  -> authority acceptance transition
```

None of these alone completes a Task:

- Pi `agent_end`;
- Provider response or success;
- process exit zero;
- Tool or Agent receipt;
- Shell-rendered success;
- AgentExecution terminal state.

## 5. Linux 1.0 and future adapters

Linux 1.0 product-qualifies Pi plus its exact sidecar only. The
package/installation/definition/instance/execution/sidecar model and adapter test harness must
remain Agent-neutral, but OpenClaw, Hermes, Codex, WorkBuddy, and other adapters remain deferred
until each has its own Agent + sidecar acquisition policy, protocol/adapter pins, lifecycle,
negative tests, benchmark evidence, and promotion decision. Pi evidence never qualifies another
adapter or sidecar.

Multiple installed Agents are not Multi-Agent orchestration. Multi-Agent planning, delegation, and
shared-budget coordination remain a separate post-1.0 capability and Gate decision.

## 6. Evidence and environment use

Environment capabilities and claim limits are registered in
[PERSONAL-TEST-ENVIRONMENTS.md](./PERSONAL-TEST-ENVIRONMENTS.md). Local, WSL, fixture, and ordinary CI
results are implementation evidence unless a formal campaign preregistration explicitly admits
them. The B01 first-install campaign and managed-Pi B09 campaign remain distinct evidence sets.

Historical handoffs and attempt records may explain a run but cannot redefine task acceptance,
campaign denominator, Gate threshold, current status, or release scope.
