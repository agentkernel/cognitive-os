# ADR-0037: Personal Unified Cognitive Resource Substrate

- Status: Accepted
- Date: 2026-08-02
- Decision owners: CognitiveOS Personal product owner
- Classification: product-semantic and structural documentation decision
- Related: ADR-0026, ADR-0035, ADR-0036, P2-T02, P3, P4, P5, P7-T08
- Partially supersedes: ADR-0036 only where it defers all durable Memory and
  general Context delivery from Linux 1.0

## Context

Personal already governs Tasks, Tools, Context inputs, Agent runtimes,
artifacts and evidence through daemon-owned authority boundaries. The prior
Linux 1.0 decision narrowed the release around a governed Task and managed Pi,
but deferred durable Memory and broad Context work. It also described many
objects as peers in one resource list without distinguishing user-visible
resource families from policy, execution and evidence objects that cut across
those families.

That shape is too narrow for the approved product identity. CognitiveOS
Personal is a single-user local substrate for Agent cognitive resources, not
only an Agent launcher. Linux 1.0 needs a small real slice of each resource
family while retaining deterministic authority and a bounded release.

## Decision

### 1. Six user-visible resource families

Personal has exactly six user-visible resource families for the Linux 1.0
baseline:

1. Memory;
2. Skill;
3. Tool;
4. Context;
5. Task;
6. Runtime/Process.

These families organize user intent, product navigation and release scope.
They do not introduce a universal public `Resource` DTO, a giant database
`Resource` table or one lifecycle/state machine shared by all families. Each
family keeps the identity, storage, transition and retention rules appropriate
to its semantics.

Budget, Permission, Model, Artifact, Intent/Effect, Evidence and Event are
cross-cutting objects. They can bind to multiple families and appear in their
views, but are not additional top-level resource families. An Agent is a user
navigation and actor concept projected from Runtime identities; it is not a
seventh storage model.

### 2. Discovery manifest remains narrow

`CognitiveResourceManifest` retains its existing normative meaning: a
discovery manifest filtered for an `ActivityContext`, purpose, scope, policy,
revocation version, budget and expiry. It may list discoverable domains,
expandable references and query capabilities. It is not the six-family
catalog, a dump of every object, a Tool capability or permission to read any
discovered content.

This ADR does not change the manifest schema or create a parallel manifest.

### 3. Minimum real Memory slice

Memory uses the deterministic flow
`MemoryCandidate -> MemoryAdmissionDecision -> MemoryObject`. Both an explicit
user `remember` request and an Agent proposal create candidates. Explicit user
intent raises priority but does not bypass scope, permission, conflict or
retention policy.

An admitted object records scope, purpose, provenance, version, conflict
disposition and retention facts. The service supports explicit expiry,
forgetting and durable tombstones. SQLite is the source of truth. FTS5 and
metadata filters are derived, rebuildable retrieval indexes.

Embeddings, a vector database, graph memory and automatic extraction of every
conversation are deferred. An Agent may propose Memory; only the daemon admits,
versions, expires, forgets or tombstones it.

### 4. Skill is an independent first-class family

A Skill is an immutable package and revision, not a Tool alias or a capability.
Linux 1.0 supports local import compatible with `SKILL.md` plus bounded
`resources/` and `scripts/`, and supports install, list, inspect, pin, enable,
disable and remove.

Skill instructions and resources may become authorized Context inputs. A Skill
script can run only through a separately registered and authorized Tool. Skill
installation or enablement grants no workspace, process, network, model or
secret permission, and a Skill never directly dispatches execution.

Marketplaces, automatic download, Skill chaining and autonomous dependency
resolution are deferred.

### 5. Minimum Tool slice

The Linux 1.0 Tool family contains:

- workspace read, search, write and patch;
- bounded process and check execution;
- read-only HTTP fetch.

Tools come from a static daemon registry with immutable descriptors and an
availability projection. Unknown, descriptor-drifted, disabled or quarantined
Tools have dispatch count zero. Reversible writes inside an admitted Standard
Workspace use a low-friction recovery journal. External or irreversible
mutations use persist-before-dispatch Intent/Effect, idempotency, fencing and
reconciliation.

### 6. A real ContextRequest and ContextView per Task

Every admitted Task has a real `ContextRequest` and `ContextView`. Resolution
may combine Task and current authority state, Memory, Skill instructions and
resources, Tool summaries, artifacts and evidence, workspace inputs and
explicit Task inputs.

Authorization and policy filtering happen before ranking. A required source
that cannot be authorized or resolved fails closed. Every omission,
truncation, conflict, stale source or budget loss is explicit.

Linux 1.0 uses deterministic priority, metadata filters and FTS retrieval. It
preserves stable view prefixes, binds views to digests and emits bounded deltas
from a base view. Learned or complex semantic ranking is deferred.

### 7. Task authority remains deterministic

Task retains durable raw intent, a server-issued digest-bound preview,
admission under CAS and epoch guards, budgets, scheduler facts, watch,
checkpoints, Effects and independent verification. Provider success, Agent
output and process exit remain observations rather than completion.

A future `TaskContract` evolution is intended to fix exact resource references
and constraints, adapter identity, and Context policy. That future shape is not
declared implemented by this ADR. Linux 1.0 may bind resource selections
through existing daemon authority records while the contract change remains a
separate Lane-CTR decision.

### 8. Runtime and Process identities remain separate

The Runtime/Process family exposes distinct package, installation,
registration, instance, sidecar, execution and OS process identities. Sharing
bytes or an OS process does not merge those identities.

Process is daemon-owned observation and supervision data for a runtime. It is
not a new authority domain, Task identity or proof of execution success.

### 9. Workspace and information architecture

The default file boundary is a low-friction Standard Workspace chosen for the
Task. A bounded Extended Home may add explicit paths under current permission
and preview; it is not ambient access to the user's home directory.

The product navigation is:

- Home;
- Agents;
- Tasks;
- Resources, containing Memory, Skills, Tools and Context;
- Activity, containing Run, Process, Effect and Evidence.

Cross-cutting objects are shown where they explain a family or Activity item;
they do not require additional top-level spaces.

### 10. Linux 1.0 composition target

Linux 1.0 adds minimum real Memory and Context slices to the prior release
boundary and adds Skill as a first-class slice. The target Gate composition is
`B01 + B02 + B03 + B04 + B05 + B08 + B09 + B12 + P7 operability`.
`B06`, `B07`, `B10` and `B11` do not block Linux 1.0.

This is a target composition, not a status statement. Current Gate status is
owned only by `PROGRESS.md`.

## Supersession and migration

ADR-0036 remains authoritative for Linux x86_64, the canonical service,
official Pi acquisition, Pi-only qualification, signing and product lifecycle.
This ADR partially supersedes only its blanket deferral of durable Memory and
general Context from Linux 1.0. Advanced Memory retrieval and complex Context
optimization remain deferred as described above.

Existing family-specific tables and state machines remain in place. No data
migration into a universal Resource table is authorized. Product views should
compose family projections and cross-cutting references through stable daemon
application services.

## Consequences

- Personal's 1.0 product shape now demonstrates a bounded end-to-end slice of
  all six cognitive resource families.
- B03 and B08 become release composition targets, while their advanced
  benchmark and retrieval extensions remain independently scoped.
- Skill work must not be hidden inside Tool or Context implementation.
- Existing public contracts remain unchanged until a separately coordinated
  normative or structural contract decision is approved.

## Rejected alternatives

1. **Keep Memory and Context entirely post-1.0.** Rejected because the product
   would not prove its unified cognitive-resource identity.
2. **Create one generic Resource table and lifecycle.** Rejected because
   Memory retention, Skill revisioning, Tool dispatch, Context views, Task
   authority and runtime supervision have materially different invariants.
3. **Treat Skills as prompt files or executable Tools.** Rejected because it
   would collapse content, permission and execution boundaries.
4. **Require vector search for Memory 1.0.** Rejected because FTS5 and metadata
   filtering provide a deterministic, local and testable first slice.
5. **Let Agent ranking run before authorization.** Rejected because it leaks
   hidden candidates and allows probabilistic selection to widen scope.

## Non-claims

This ADR and its documentation batch implement no runtime behavior. They do not
run or pass any Gate, release Personal 1.0, produce release or Profile evidence,
or establish CognitiveOS Profile conformance. Gate states remain whatever
`PROGRESS.md` currently records.
