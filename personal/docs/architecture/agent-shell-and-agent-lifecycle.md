# Personal Agent Shell and Agent Lifecycle

- Status: informative current/target alignment
- Change class: owner-approved `product-semantic + structural` documentation
- Linux 1.0 decisions:
  [ADR-0035](../../../docs/adr/0035-personal-pi-shell-and-managed-agent-role-separation.md) and
  [ADR-0036](../../../docs/adr/0036-personal-linux-1-0-and-official-pi-acquisition.md)
- Adapter decision:
  [ADR-0043](../../../docs/adr/0043-personal-universal-agent-adapter.md)
- Personal 2.0 companion:
  [Agent adapter architecture](agent-adapter-contract.md)

## 1. Current boundary

### Now

The Agent Shell is a client and candidate producer, not an authority service.
Linux 1.0 uses Pi both as the Shell host and as the only qualified Agent
adapter. Those are independent roles even when one process hosts both.

The delivered P8 adapter registration/lifecycle boundary generalizes exact
identity, digest, channel, and candidate-only checks. P8's Codex work is a
fixture qualification, not a live production qualification and not transferred
Pi evidence. The delivered dsh integration supplies a separate native Agent
path and native web surface; it does not merge dsh sessions with Control Plane,
Shell, Task, or Agent authority identities.

The current Control Plane can observe and govern the services it actually has,
but it does not have a common native-conversation projection service or typed
browser controls for Task and Agent lifecycle. Existing Core
Conversation/ConversationBinding contracts do not make that product surface
implemented. Architecture must not imply otherwise.

## 2. Global Agent Shell role

### 2.0 target

The Shell is a global assistant available across Personal experiences. It may:

- interpret owner language into candidates;
- explain daemon projections, provenance, policy, blockers, and evidence;
- navigate to exact Personal objects and native conversations;
- prepare a Goal -> Plan revision -> Task -> Attempt admission candidate;
- suggest resource bindings, assignments, handoffs, and recovery choices; and
- invoke deterministic read operations available to its authenticated channel.

The Shell may not:

- mutate authority from conversation text;
- claim a native conversation operation that the vendor adapter does not
  support;
- turn a native plan into the daemon Plan without explicit admission;
- authorize Tool/MCP use, external writeback, or Provider selection;
- receive raw Provider/user secrets; or
- present Agent/native success as Effect closure or Task acceptance.

Natural language and deterministic commands converge at the daemon policy
boundary. If the Shell host or model is unavailable, deterministic inspection
and recovery remain available.

## 3. Strict identities

Personal keeps these identities separate even when a vendor runtime collapses
them internally:

| Identity | Owner | Must not be treated as |
|---|---|---|
| Agent package | acquisition source plus verified immutable identity | installation, permission, or running process |
| Agent installation | Personal installation authority | registration, qualification, or capability |
| Agent registration | Personal policy plus exact adapter binding | active instance or native login |
| Agent instance | Personal Runtime authority | process, conversation, Task, or completion |
| Adapter/sidecar session | daemon-supervised integration boundary | native conversation, authority service, or Agent instance |
| Native runtime | vendor process/service identity | Personal instance or admitted work |
| Native account/login | origin-owned authentication state referenced by an opaque handle | Provider secret, Personal principal, or capability |
| Vendor-native conversation ID | opaque origin identity and lineage bound by the adapter | a new public Conversation, Shell session, Goal, Plan, Task, or execution |
| Core Conversation / ConversationBinding | existing governed interaction scope and fixed history/working-scope binding, reused or referenced where applicable | vendor-native session ID or Personal-private extension state |
| Native turn | origin-owned unit of conversation activity | Personal Task step or Effect |
| Native plan | origin-owned plan observation | daemon-owned revisioned Plan |
| Runtime attachment | current adapter link to a native runtime/conversation | durable registration or authorization |
| Agent execution | daemon scheduling binding between governed work and an Agent instance | native turn, OS process, or Task identity |
| OS process attempt | bounded host observation | stable runtime identity or success |
| Shell session | assistant interaction and client channel | native conversation, management authority, or execution |
| Control Plane session | browser client authentication/channel state | Shell/native session or daemon authority |
| Goal | daemon-owned durable outcome | Plan, Task, Attempt, Agent, or conversation |
| Plan revision | daemon-owned decomposition under one Goal | Agent-native plan or Task |
| Task | daemon-owned bounded work under one Plan revision | Attempt or process |
| Governed attempt | preserved execution/recovery branch under one Task | OS process attempt, native conversation fork, or rewritten prior evidence |

Goal/Plan/Attempt and the native/common projection are **2.0 target** and
**Requires-backend**. The projection reuses existing Core
Conversation/ConversationBinding. Only a new or changed public extension
conditionally requires P10-T02/Lane-CTR; Personal-private projection state may
not require core changes.

## 4. Pi and other native Agents

Pi retains two independent current roles:

1. Linux 1.0 Shell host; and
2. Linux 1.0 qualified managed Agent adapter.

No later adapter inherits either role. A vendor-specific adapter may use a
native application server, RPC protocol, host integration, or another
vendor-supported control surface. ACP is optional and is not a Personal
qualification prerequisite.

MCP plus Agent rules/instructions may provide cooperative candidate and tool
exchange when no stronger native interface exists. That fallback is not
session control: it cannot prove login state, list complete conversations,
preserve lineage, steer or interrupt a turn, fork or close a conversation,
retrieve full history, or attach to the correct runtime unless the native
integration independently supports those facts.

## 5. Adapter and sidecar boundary

### Now

The delivered adapter manifest and private AKP path bind exact package,
adapter, protocol, channel, and candidate-only declarations. A daemon-supervised
sidecar can translate Agent-native behavior into bounded candidates and
observations. It cannot authorize itself, broaden capability, write authority
storage, commit an Effect, reconcile a mutation, or accept work.

### 2.0 target

The vendor adapter additionally projects:

- initialization and exact adapter/native identity;
- capability conditions;
- auth status and an opaque login handle;
- conversation list/create/load/resume/fork/close capability;
- turn submit/steer/interrupt capability;
- sequenced native events;
- tool approval, native plan, history, attachment, and MCP-binding capability;
- runtime launch/attach capability; and
- bounded vendor-specific render slots.

This is a conceptual product contract, not a public machine shape. Its detailed
semantics live in
[Agent adapter architecture](agent-adapter-contract.md).

No raw secret crosses the adapter conversation wire. Native login may occur
through a vendor-owned secure flow or a daemon-mediated approved boundary, but
the common projection carries only redacted status and an opaque handle.

## 6. Native conversation and explicit admission

### 2.0 target

A native conversation is useful before it becomes governed work. Connecting an
Agent establishes an explicit observation scope. Personal may observe
automatically only inside that scope; there is no speculative/global session
scan or surprise per-session enrollment. Observation does not:

- copy the conversation into authority by default;
- create a Goal, Plan, or Task;
- grant Context, Tool, MCP, workspace, network, or Provider capability; or
- make Personal responsible for origin-native completion claims.

Owner request/confirmation followed by daemon admission creates the governance
bridge:

1. the adapter supplies exact native identity, lineage, bounded content
   references, native-plan observation, capability conditions, and sequence
   coverage;
2. the daemon records provenance and unresolved gaps;
3. the Shell or Control Plane prepares an exact candidate Goal and Plan;
4. the daemon issues the exact preview and the owner confirms it; and
5. the daemon admits the Goal and Plan revision, creates governed Tasks and
   Task-owned attempts, then owns assignments, handoffs, Effects,
   reconciliation, verification, and acceptance.

Later native events remain observations linked to the admitted work. They do
not silently revise the daemon Plan.

## 7. Lifecycle responsibility

Agent package, installation, registration, instance, adapter session, native
runtime, native conversation, execution, and process each have their own
owner. Personal may expose lifecycle actions only through a typed capability
owned by the relevant layer.

| Operation concept | Owning meaning |
|---|---|
| acquire/install/register | establish verified Personal package and policy identities; grants no runtime capability |
| launch/attach | start or connect to a vendor runtime under exact adapter identity |
| create/load/resume/fork/close conversation | change origin-owned native conversation state |
| submit/steer/interrupt turn | request native turn behavior when the adapter supports it |
| assign/handoff | change daemon-owned governed work graph |
| pause/resume execution | reach a safe governed point, fence stale work, and reauthorize before continuation |
| stop/restart runtime | quiesce or replace runtime machinery; not Task cancellation or completion |
| cancel Task | request governed Task closure; not process kill or conversation close |
| upgrade/rollback/uninstall | change Personal package/registration binding while preserving recovery and audit |

`unsupported`, `unavailable`, and `unknown` are distinct:

- **unsupported** means the adapter declares no such operation;
- **unavailable** means the operation exists but current auth, runtime,
  connection, policy, or dependency blocks it;
- **unknown** means Personal lacks enough current observation to decide.

## 8. Recovery verb distinctions

- **detach** stops one client or adapter observation attachment. Native work
  may continue.
- **interrupt** asks the current native turn to yield. It does not close the
  conversation, cancel a Task, or reconcile an Effect.
- **cancel** is a daemon-owned Task decision and must account for open Effects.
- **pause** fences new governed dispatch and seeks a safe checkpoint.
- **restart** replaces a runtime or adapter session under a fresh epoch; it is
  not resume by itself.
- **native fork** creates a new origin-native conversation lineage. The fork is
  not a new Goal -> Plan revision -> Task -> Attempt branch until the owner
  confirms and the daemon admits it.
- **retry/fork from checkpoint** creates a new daemon-owned governed attempt and
  preserves the prior attempt, failure, evidence, and Effect facts.
- **close** ends an origin-native conversation according to the adapter. It
  does not erase Personal history.
- **undo** is a compensating daemon-governed mutation with new
  Intent/Effect/evidence. It never deletes the original fact.

The current browser does not expose typed Task or Agent controls. Those
controls and common native conversation operations are **Requires-backend**.

## 9. Provider, Tool, and MCP boundaries

An Agent receives a daemon-mediated Provider proxy binding, never raw Provider
credentials. Global, Agent, or conversation-level Provider selection becomes
effective for current governed work only after explicit daemon rebind.

A native tool request or MCP-advertised tool is a candidate. The daemon still
validates exact Tool identity, current capability, scope, budget, and epoch and
persists Intent/Effect before external mutation. Native approval is not
Personal authorization. A receipt is not reconciliation or verification.

MCP installation and connection grant no Tool capability. Personal MCP policy,
bindings, and external config projection are described in
[Resource Manager architecture](resource-manager-architecture.md).

## 10. Agent onboarding and removal

### 2.0 target

Agent onboarding preserves native behavior while establishing exact Personal
identity:

1. choose a signed upstream catalog record or **Connect existing**;
2. review Provider/proxy profile, Standard Workspace, and requested
   permissions together; and
3. open the native conversation.

A catalog record exposes source, version, digest, signature, license, and
adapter compatibility. Listing or installing it grants no permission and
transfers no qualification.

Activation has two separate milestones:

- **first chat** — the first real native response; and
- **first governed and verified Task** — daemon-admitted work whose Effects are
  Governed/reconciled and whose outcome has current independent verification
  and daemon acceptance.

Removal distinguishes:

- **disconnect** — remove Personal observation/management bindings while
  preserving the native installation and native data; and
- **uninstall** — remove a Personal-managed installation only after impact,
  pending Effect, retained-data, and recovery review.

Catalog onboarding, connect-existing composition, embedded conversation, and
the target disconnect/uninstall experience are **Requires-backend**.

## 11. Current/target boundary

| Capability | Status |
|---|---|
| Pi Shell host and Pi-only Linux 1.0 qualification | **Now** |
| Delivered P8 adapter registration/lifecycle boundary | **Now** |
| Codex fixture qualification and dsh integration path | **Now**, with their recorded non-claims |
| Vendor-native common conversation/capability projection | **Requires-backend** |
| Goal, revisioned Plan, and Task-owned attempt admission | **Requires-backend**; P10-T02/Lane-CTR only for new public semantics |
| Typed browser Task/Agent lifecycle controls | **Requires-backend** |
| MCP as seventh family | **Requires-backend**; P10-T02/Lane-CTR only for a new/changed public machine surface |
| Non-Pi Linux 1.0 qualification transfer | **Not permitted** |

Current facts remain in
[PROGRESS.md](../../../docs/plan/PROGRESS.md). This architecture creates no
Gate, release, Profile, or Agent-benefit claim.
