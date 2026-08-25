# CognitiveOS Personal Product Design

- Status: canonical stable product intent
- Initial release target: Linux x86_64 `1.0.0` through `GMVP-LINUX`
- Architecture: [Personal architecture](../architecture/README.md)
- Decisions: [ADR-0037](../../../docs/adr/0037-personal-unified-cognitive-resource-substrate.md),
  [ADR-0038](../../../docs/adr/0038-personal-agent-sidecar-linux-evolution-boundary.md)

## 1. Product statement

CognitiveOS Personal is a local **operating system for cognitive resources** for
one owner and their Agents. It makes Memory, Skill, Tool, Context, Task and
Runtime/Process visible and governable through one local product while keeping
their domain-specific identities, stores and lifecycles separate. See
[personal-2.0-scope.md](personal-2.0-scope.md) for the post-1.0 design baseline.

Users can work through a conversational Agent Shell or deterministic commands.
The Rust daemon is the sole authority writer: it resolves identity and policy,
issues previews, admits exact requests, enforces CAS/epoch/budget guards,
schedules work, commits and reconciles Effects, and decides final acceptance
from independent evidence.

Personal is an operating layer above Linux. It is not a replacement kernel, a
driver framework, a distributed control plane or a launcher that trusts every
Agent independently.

## 2. Target user and jobs

### Primary user

A technically capable individual who uses coding or general-purpose Agents but
wants one local place to understand what the Agents know, which Skills and
Tools they may use, what Context a Task received, what is running, what changed
and whether a result is actually complete.

### Jobs to be done

1. Install Personal and reach a useful first conversation without distributing
   Provider credentials into Agent runtimes.
2. Import, inspect, pin and disable Skills without treating instructions or
   scripts as permission.
3. Explicitly remember useful knowledge, review Agent Memory proposals, search
   admitted Memory and forget it with a visible tombstone.
4. Create a bounded Task whose real Context and Tool availability can be
   inspected and reproduced.
5. Work in a low-friction Standard Workspace and deliberately extend access to
   selected home paths without granting ambient home-directory access.
6. Supervise, pause, resume, recover, upgrade or remove an Agent while package,
   installation, registration, sidecar, instance, execution and process remain
   distinguishable.
7. Diagnose and restore service even when the model, Provider, Agent or sidecar
   is unavailable.

## 3. Resource and authority model

### Six resource families

The user-visible resource families are:

| Family | Product responsibility |
|---|---|
| Memory | admitted durable knowledge with scope, provenance, versions, conflicts, expiry, forget and tombstone |
| Skill | immutable locally imported instructions/resources/scripts package with revision and enablement policy |
| Tool | registered workspace, process/check and read-only fetch operations with explicit availability |
| Context | the authorized, budgeted Task input request, resolved view and explicit losses/deltas |
| Task | raw intent, preview, admission, bounded execution, checkpoint, Effect and verification |
| Runtime/Process | Agent package through execution identities plus daemon-owned process observation and supervision |

Budget, Permission, Model, Artifact, Intent/Effect, Evidence and Event cross the
families. They appear in family and Activity views, but are not additional
families. The product does not create a universal `Resource` record or one
state machine for all six families.

`CognitiveResourceManifest` remains only the discovery manifest filtered for a
specific `ActivityContext`. It does not become this product taxonomy and does
not grant read or dispatch permission.

### Daemon authority

Pi, every Agent sidecar, the CLI, SDK and future UI are clients. They may submit
candidate interpretations, Memory proposals, Context requests, Tool requests
and runtime observations. Only deterministic daemon services may:

- authorize and issue capability decisions;
- resolve versions and apply CAS/epoch guards;
- admit Memory and Context;
- register and dispatch Tools;
- advance Task, Effect or verification state;
- supervise runtime identities and accept completion;
- write authority SQLite.

## 4. Product principles

### Shell first, deterministic fallback always

The Pi-hosted Agent Shell is the primary Linux 1.0 experience. The `cognitive`
CLI calls the same daemon application services for exact commands, automation,
recovery and model-free operation.

### Preview before authority mutation

Natural language is compiled into a server-issued canonical preview containing
exact targets, versions, permissions, budget impact, external mutations and
rollback expectations. Admission binds to that digest. Stale or changed facts
require a new preview.

### One default approval, hard safety rails

Tier 0 operations are silent, Tier 1 uses a bounded first-use capability lease,
and Tier 2 always requires explicit confirmation. Budgets, epochs, static Tool
descriptors and state guards enforce policy without an approval chain.

### Content never implies permission

An installed Agent, enabled Skill, selected Model, discovered resource or Tool
summary grants no runtime capability. Skill scripts run only through registered
Tools. Workspace, process, network, Memory and model scopes remain independent
and revocable.

### Filter before ranking

Context and Memory retrieval apply authorization and policy filters before
deterministic ranking. Hidden or denied sources cannot influence ranking or be
revealed through a side channel. Required unavailable inputs fail closed, and
losses remain explicit.

### Results must be explainable

Users see authority state, Context digest/loss, Tool availability,
pending/unknown Effects, evidence and verifier results. Process exit, sidecar
success, Provider response or fluent Agent output is never presented as Task
completion by itself.

### Local by default, secrets stay native

The daemon binds loopback only. Provider/user secret material stays in an
approved `SecretStore` backend and never enters a service unit/credential,
environment, Agent configuration, SQLite, argv, logs, Context, Memory or
support evidence.

## 5. Workspace model

### Standard Workspace

A Task selects a Standard Workspace as its default file boundary. Within that
boundary, policy may allow low-friction read/search and reversible write/patch
through registered Tools. Writes retain a recoverable journal and explicit
change projection.

### Bounded Extended Home

Extended Home is an explicit set of additional document/project roots,
purposes and allowed operations. It may also enable ordinary outbound network
access. It is previewed, remembered only by explicit choice and revocable.
Selecting one path does not grant its siblings or the full home directory. A
sidecar sees only the resolved paths and network policy admitted for its
current execution.

Extended Home hard-denies Secret Store contents, SSH/GPG keys, browser
credential/profile stores, CognitiveOS authority/bootstrap data, Docker and
system sockets, system directories, privilege elevation, service management
and package management. Publication, repository push, irreversible deletion
and other remote mutations remain exact typed operations with confirmation
where required.

## 6. Agent and sidecar model

The per-Agent sidecar is the primary Agent integration boundary. Each managed
Agent registration binds an exact sidecar identity and digest. The sidecar
translates Agent-native protocol, Context delivery, Tool requests, cancellation
and observations into daemon application-service ports; it never becomes an
authority writer.

Package, installation, registration, instance, sidecar, execution and process
are distinct identities. Sidecar may be co-located with an Agent process, but
co-location does not merge permissions, lifecycle, epochs or completion.

Pi is the only Linux 1.0 qualified Agent/sidecar combination. Pi may also host
the Shell, but Shell session and managed runtime remain separate roles and
channel credentials.

## 7. Primary surfaces and local modes

| Surface | Purpose | Authority boundary |
|---|---|---|
| Pi-hosted Agent Shell | natural-language goals, inspection, preview and watch | client only; facts come from daemon projections |
| `cognitive` CLI | deterministic resource management and recovery | client only; same application services |
| daemon local API | separate Task and management sessions | authenticated loopback front door |
| doctor/support bundle | explain readiness, drift and failure | redacted facts and digests only |

Desktop, headless and foreground modes use the same production artifact, Rust
daemon authority and application services. UI attachment and supervision may
differ, but there is no mode-specific database writer or alternate backend.
Linux 1.0 requires no Web UI.

Desktop mode uses FreeDesktop Secret Service. A headless daemon may start
locked against an approved encrypted vault, use SSH TTY unlock as the baseline,
and optionally obtain only vault-unlock material through a systemd encrypted
credential. Provider/user secrets never enter the service unit, credential,
environment, argv, ordinary configuration, SQLite, logs or evidence.

## 8. Information architecture

The product presents five top-level spaces:

1. **Home**: readiness, current work, health, blocked actions and budget alerts;
2. **Agents**: package, installation, registration, instance, sidecar, health,
   version and permissions;
3. **Tasks**: raw intent, preview, contract/bindings, Context, progress, budget,
   checkpoints and verification;
4. **Resources**:
   - **Memory**;
   - **Skills**;
   - **Tools**;
   - **Context**;
5. **Activity**:
   - **Run**;
   - **Process**;
   - **Effect**;
   - **Evidence**.

Model, Permission, Budget, Artifact and Event are shown in the views they
explain. They do not create parallel top-level spaces. Natural-language names
are conveniences; every mutation preview shows stable identity and version.

## 9. Interaction states

Every long-running operation exposes, as applicable:

- proposed or awaiting clarification;
- awaiting exact admission;
- accepted/queued;
- running or waiting;
- suspended/blocked;
- reconciling unknown outcome;
- verifying;
- completed, failed, cancelled or quarantined.

The Shell may acknowledge request receipt but cannot fabricate later states.
Detaching stops observation; it does not cancel work.

## 10. Linux and hardware evolution boundary

Personal stabilizes software ports for user service, filesystem and process
observation, native secrets, package acquisition, network egress and future
hardware capability/observation. Linux and hardware adapters remain below the
daemon authority boundary.

Linux 1.0 does not implement a kernel module, eBPF control plane, device
scheduler, custom kernel or distributed authority. Future acceleration cannot
move authorization, CAS, budgets, Intent/Effect commit or acceptance out of
the daemon without a new architecture decision.

## 11. Success measures

Release campaigns measure, without converting local samples into claims:

- clean install-to-first-conversation success and time;
- successful minimum Memory, Skill, Tool and Context workflows;
- default-path confirmation count in Standard Workspace;
- unauthorized Extended Home and unknown/drifted Tool dispatch count;
- false-completion and duplicate external mutation rates;
- recovery time across daemon, sidecar, Agent and process failure;
- Pi install/upgrade/rollback/uninstall correctness;
- secret/redaction critical failures;
- actionable doctor/support outcomes.

Formal thresholds, Gate status and environment requirements remain owned by
the formal plan, preregistered campaigns and `PROGRESS.md`.

## 12. Linux 1.0 product shape

Linux 1.0 targets a minimum real slice of all six families, the Standard
Workspace plus bounded Extended Home, one canonical local service and the exact
official Pi package with a qualified Pi sidecar. The reusable sidecar framework
does not qualify other Agents.

The target release composition is
`B01 + B02 + B03 + B04 + B05 + B08 + B09 + B12 + P7 operability`. B01 uses
the successor campaign policy in ADR-0039; that policy does not itself claim
that B01 or the release gate has passed.
`B06`, `B07`, `B10` and `B11` do not block Linux 1.0. This statement changes no
current Gate status and claims no implementation or evidence.

See [Linux 1.0 scope](linux-1.0-scope.md) for the exact target and non-claim
boundary.
