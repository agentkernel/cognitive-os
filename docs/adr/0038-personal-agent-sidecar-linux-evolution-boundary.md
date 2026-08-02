# ADR-0038: Personal Agent Sidecar and Linux Evolution Boundary

- Status: Accepted
- Date: 2026-08-02
- Decision owners: CognitiveOS Personal product owner
- Classification: product-semantic and structural documentation decision
- Related: ADR-0025, ADR-0034, ADR-0035, ADR-0036, ADR-0037, P2-T02,
  P5-T01, P5-T02, P5-T05, P7-T08
- Extends: ADR-0035 role separation and ADR-0036 Linux 1.0 deployment and
  Agent-support boundaries
- Partially supersedes: ADR-0036 only where native Secret Service was the sole
  Linux 1.0 secret backend; desktop Secret Service remains the default

## Context

ADR-0035 separated the Pi-hosted Shell from managed Pi identities. ADR-0036
selected Linux x86_64, official Pi acquisition and Pi-only 1.0 qualification.
Those decisions did not yet name the durable integration boundary between the
daemon and each Agent, nor did they bound how a local cognitive-resource
substrate may evolve toward desktop and hardware integration without becoming
a kernel or distributed authority system.

Directly embedding each Agent's protocol into the daemon would couple release
and authority code to Agent-specific sessions, tools and process behavior. A
global bridge shared by all Agents would make package, permission, health and
recovery identity ambiguous. Separate desktop, headless and foreground
backends would create drift in authority and service behavior.

## Decision

### 1. Per-Agent sidecar is the primary integration boundary

Every managed Agent registration binds an explicit, versioned sidecar adapter
identity and digest. The binding is per registered Agent rather than an
ambient global bridge. A sidecar translates Agent-native protocol, Context
delivery, Tool requests, runtime observations and cancellation into stable
daemon application-service ports.

The sidecar is not an authority writer. It may produce candidates, proposals
and observations. It cannot authorize, mint capabilities, change Task state,
commit an Effect, decide acceptance or write daemon-owned SQLite. Unknown or
drifted sidecar identity fails closed before execution dispatch.

Sidecar is a logical identity and service boundary, not necessarily one
dedicated OS process. It may be hosted beside or inside an Agent process when
qualification proves the boundary, but process co-location does not merge
credentials, epochs, lifecycle or authority.

### 2. Runtime identities are non-interchangeable

The following identities remain separate:

| Identity | Stable meaning | Authority boundary |
|---|---|---|
| Package | immutable upstream bytes and provenance | acquisition input only |
| Installation | verified private bytes and acquisition lock | daemon installation authority |
| Registration | policy and active installation/sidecar binding | daemon management authority |
| Instance | supervised logical runtime identity | daemon lifecycle authority |
| Sidecar | versioned Agent protocol adapter boundary | non-authority client/translator |
| Execution | Task/Loop/instance/epoch binding | daemon scheduler/runtime authority |
| Process | PID/handle and bounded host observations | daemon-owned supervision data |

A conversation session is also distinct from every row above. Process exit,
sidecar success and Agent completion events are observations only.

### 3. Pi is the only Linux 1.0 qualification

Pi is the only sidecar/Agent combination included in Linux 1.0 product
qualification. The Pi-hosted Shell remains a separate client role even when it
shares package bytes or a process with the managed Pi runtime.

The sidecar framework is reusable, but OpenClaw, Hermes, Codex, WorkBuddy, MCP
bridges and every other Agent require independent package, sidecar, protocol,
sandbox, lifecycle, recovery and negative qualification. Pi evidence does not
transfer.

### 4. Standard Workspace and bounded Extended Home

The default Agent file boundary is a user-selected Standard Workspace. Within
that boundary, registered read/search and reversible write/patch operations can
follow low-friction policy and a recoverable journal.

Extended Home access is an explicit set of additional document/project roots,
purposes and operations, plus ordinary outbound network access when the user
enables it. It requires daemon preview and permission, can be remembered and
revoked, and never means ambient access to the full home directory.

Extended Home still hard-denies native Secret Store contents, SSH/GPG keys,
browser credential/profile stores, CognitiveOS authority/bootstrap data,
Docker and system sockets, system directories, privilege elevation, service
management and package management. External publication, repository push,
irreversible deletion and other mutating remote operations remain typed
operations with exact confirmation where policy requires it. Sidecars receive
only the resolved paths, network policy and operations admitted for the current
execution.

### 5. One artifact and service across local modes

Desktop, headless and foreground operation use the same production-signed
Personal artifact, the same Rust daemon authority implementation and the same
application services. The supported launch or presentation mode may change
session lifetime, supervision and UI attachment; it must not create a second
database writer, alternate authority policy or mode-specific release backend.

Linux 1.0 retains the canonical `cognitiveos-personal.service` user unit and
numeric loopback endpoint. Foreground operation is an operational form of the
same service artifact, not a separate product.

### 6. Headless secret recovery

Desktop mode uses FreeDesktop Secret Service. A headless host may use an
approved encrypted vault behind the same `SecretStore` port. The daemon can
start in a locked, read-only diagnostic state; SSH TTY unlock is the baseline
recovery path. An optional unattended mode may obtain only vault-unlock
material through a systemd encrypted credential. Provider/user secret values
must never enter the unit, credential payload, environment, argv, ordinary
configuration, SQLite, logs or evidence.

The same production artifact and service implement desktop Secret Service,
headless vault and foreground recovery. A copied bootstrap secret, raw port
forward or second headless authority service is not a supported shortcut.

### 7. Linux and hardware evolution stabilize ports only

Personal may stabilize software ports for Linux user-service integration,
filesystem and process observation, Secret Store access, package acquisition,
network egress and future hardware capability/observation adapters. The daemon
continues to own all policy and durable authority above those ports.

Linux 1.0 does not implement or require a kernel module, eBPF control plane,
device scheduler, custom container kernel, distributed scheduler or
distributed authority. Future acceleration or device integration must remain
behind bounded ports and cannot move authorization, CAS, budgets, Effect commit
or acceptance out of the daemon without a new architecture decision.

### 8. Release composition remains explicit

Together with ADR-0037, the Linux 1.0 target composes
`B01 + B02 + B03 + B04 + B05 + B08 + B09 + B12 + P7 operability`.
`B06`, `B07`, `B10` and `B11` are non-blocking for this release target.

This composition does not set any Gate to running or pass. `PROGRESS.md`
remains the only owner of current Gate status.

## Extension and migration of prior decisions

ADR-0035 remains authoritative for Pi Shell versus managed Pi role separation,
channel isolation and non-authority completion. This ADR extends it by placing
the versioned per-Agent sidecar between each managed Agent and daemon service
ports.

ADR-0036 remains authoritative for official Pi acquisition, Linux x86_64,
single-service topology and Pi-only support. This ADR extends its deployment
boundary to all local presentation modes and partially replaces its
Secret-Service-only assumption with the approved headless vault path. It also
excludes kernel-level or distributed authority expansion.

The existing Pi Extension and adapter assets are migration inputs, not proof
of a qualified sidecar. Qualification must bind exact package, installation,
registration, sidecar, instance and execution identities and exercise drift,
permission, recovery and out-of-band mutation negatives.

## Consequences

- Agent-specific protocol change can be isolated from daemon authority rules.
- A sidecar can be restarted or replaced without changing the Task or Agent
  installation identity, subject to epoch fencing and recovery.
- Product packaging must not fork separate desktop and headless authority
  implementations.
- Hardware exploration cannot block Linux 1.0 and cannot create a competing
  scheduler or authority plane.

## Rejected alternatives

1. **Embed every Agent protocol directly in authority services.** Rejected
   because Agent drift would expand the trusted authority surface.
2. **Use one global adapter bridge for all Agents.** Rejected because it
   obscures per-Agent identity, health, permissions and qualification.
3. **Make sidecars authority writers.** Rejected because it creates multiple
   writers and inconsistent recovery.
4. **Ship separate desktop and headless backends.** Rejected because behavior,
   migration and evidence would drift by mode.
5. **Use kernel or eBPF enforcement as a 1.0 prerequisite.** Rejected because
   the product requires stable local ports and deterministic daemon policy,
   not a new operating-system kernel.

## Non-claims

This ADR and its documentation batch implement no sidecar, Runtime, Process,
desktop, hardware or release behavior. They run or pass no Gate, release no
artifact, produce no release or Profile evidence, and establish no Profile
conformance. Current task and Gate states remain owned by `PROGRESS.md`.
