# Personal Cognitive Resource Model

- Status: canonical product concept model
- Architecture mapping: [System architecture](../../architecture/personal/system-architecture.md)

## 1. Definition

A cognitive resource is anything the user can ask Personal to inspect, bind,
budget, authorize, execute, retain or remove while pursuing a goal. Resources
have stable identities, authority-owned state, provenance and explicit
lifecycle boundaries. The Shell provides names and natural-language access;
the preview resolves those names to exact identities and versions.

## 2. Resource families

| Resource | User questions and actions | Source of truth | Linux 1.0 state |
|---|---|---|---|
| Agent package/installation | install, inspect source/version, upgrade, rollback, uninstall | daemon installation authority and acquisition lock | Pi only; generic framework ready |
| Agent instance/execution | connect, activate, supervise, pause, resume, stop, recover | daemon lifecycle and scheduler projections | Pi only |
| Model/Provider | discover, select, inspect capabilities and readiness | daemon Provider configuration and capability snapshot | included, one selected model path |
| Tool/operation | list, inspect risk, grant, execute, disable, reconcile | daemon Tool Registry and Effect state | one safe catalog-bound operation |
| Task/Loop | create, preview, admit, watch, correct, cancel, verify | Task/Loop authority objects and events | included |
| Budget | inspect or set deadline, retry, step, token and cost bounds | TaskContract and durable budget records | included |
| Permission | list, grant, revoke and inspect scope/expiry | capability authority and policy | included Tier 0/1/2 |
| Context | inspect selected inputs, provenance, budget and losses | governed Context resolution | minimum Task inputs only; optimization deferred |
| Memory | remember, inspect provenance, forget and resolve conflict | governed Memory service | deferred |
| Artifact | inspect generated files, reports and content digests | governed artifact/evidence store | minimum Task artifacts included |
| Effect | inspect pending, unknown, reconciled or compensated external mutations | Intent/Effect authority | included |
| Evidence/Verification | explain progress, criteria and completion | immutable evidence refs and verifier state | included |

## 3. Common resource properties

Every product-visible resource should expose, when applicable:

- stable ID and human-readable label;
- type, owner/scope and current version;
- lifecycle and health state;
- provenance/source and immutable digest;
- current bindings to Task, Agent, model, Tool or workspace;
- capability requirements and active leases;
- budget use and remaining ceilings;
- last authority event and watch cursor;
- pending/unknown Effects;
- evidence and verifier disposition;
- supported next actions and reasons blocked.

Absent data is shown as unknown/not-run rather than guessed from process or
client state.

## 4. Action semantics

| Action class | Examples | Default interaction |
|---|---|---|
| Tier 0 read/reversible local | list Agents, inspect Task, attach/detach watch | silent after authenticated request |
| Tier 1 accountable mutation | acquire Pi, activate an instance, invoke an idempotent external operation | preview plus first-use capability lease |
| Tier 2 irreversible/high-risk | purge data, destructive uninstall, exceed budget, broad permission | explicit confirmation every time |

Risk is a daemon catalog/policy fact, never inferred by Pi from a verb or tool
name. Unknown operations default to Tier 2 and cannot dispatch without a
registered implementation.

## 5. Relationship rules

1. An installed Agent has no runtime permission by default.
2. An Agent instance may serve multiple Tasks over time, but each
   `AgentExecution` is bound to one admitted Task/Loop epoch.
3. A model selection does not grant Tool, filesystem or network capability.
4. Context is Task input; Memory is durable governed knowledge. Conversation
   history is neither automatically.
5. An artifact is output/data; evidence is the immutable fact used to verify a
   criterion. One object may be referenced in both roles under explicit policy.
6. An Effect is not complete because an external process exited; reconciliation
   establishes its authoritative outcome.
7. Verification is independent of the Agent/executor that produced the result.

## 6. Shell presentation

Examples of intended natural-language requests include:

- “Show installed Agents and why any are unhealthy.”
- “Install the approved Pi version and show the source and permissions before
  activation.”
- “Pause the current Pi execution after it reaches a safe checkpoint.”
- “What budget remains for this Task and which Effect is unresolved?”
- “Remove Pi but retain my Tasks, evidence and Provider configuration.”

The Shell compiles each request into a structured operation. It never answers a
state-changing question by directly calling Pi tools or mutating local files.

## 7. Deferred resource capabilities

The model intentionally includes durable Memory, broad Context management,
multiple Agent adapters and MCP so their eventual designs fit the same
authority namespace. Their conceptual presence does not place them in Linux
1.0. See [Linux 1.0 scope](linux-1.0-scope.md).
