# Personal authority, data, checkpoint, and recovery

- Status: current invariants plus Personal 2.0 target composition
- Normative behavior:
  [Task/Loop/Verification](../../../docs/standards/task-loop-verification.md),
  [Intent/Effect](../../../docs/standards/intent-effect-idempotency.md),
  [authorization](../../../docs/standards/authn-authz-capability.md), and
  [event/watch](../../../docs/standards/event-audit-watch.md)
- Current decision:
  [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)

## 1. Sole authority

The Rust daemon is the only writer of Project, Goal, Plan, Role/Assignment/
Employee, Routine/Trigger, Task/Attempt, Handoff, budget, Intent/Effect,
evidence, verification, and acceptance facts. UI, Assistant, Pi, DSH,
employees, workflow engines, connectors, Provider systems, and host services
submit requests/candidates/observations only.

Every mutation checks principal/channel, exact object/scope/purpose, current
version/epoch, capability, binding, budget, idempotency, and domain rules.

## 2. Data ownership

| Plane | Examples | Treatment |
|---|---|---|
| authority | Project/work/execution/binding/budget/Effect facts | daemon-owned, versioned |
| candidate | Assistant/manager/employee/reflection/MCP/source proposal | no authority until deterministic admission |
| observation | process/DSH/Provider/connector/index result | source-labelled, bounded, never authority by shape |
| verified | independent evidence and daemon acceptance | completion only under current authority |
| content | Conversation archive, Vault source, artifacts | scoped ownership and provenance; not authority by file presence |
| secret | Provider/native credentials | approved SecretStore/non-logging daemon path only |

Project folders, indexes, engine checkpoints, UI caches, and native receipts are
not authority stores.

## 3. Persist-before-dispatch

Every external or irreversible mutation:

1. resolves exact target/source/revision;
2. validates policy, scope, capability, budget, binding, and epoch;
3. captures recoverable preimage or records why rollback is impossible;
4. persists Intent/Effect and stable identity before I/O;
5. dispatches the admitted operation;
6. records receipt or outcome unknown;
7. reconciles with the original identity;
8. verifies post-state independently;
9. closes, compensates, blocks, or quarantines under daemon authority.

DSH/connector/Provider success is not Effect closure. Unknown is never blindly
redispatched.

## 4. Engine checkpoint is not authority

An Attempt engine checkpoint may contain execution position, pending internal
writes, and deterministic resume data. It cannot:

- commit an external side effect;
- replace Task/Attempt identity;
- bypass current Context/policy/budget;
- erase prior Attempt/evidence;
- prove completion.

On resume, the daemon reloads current authority and reconciles Effects before
accepting the checkpoint. LangGraph remains behind this port if a future spike
passes strict serialization and side-effect negatives.

## 5. Ordered recovery

```text
reload authority
  -> fresh recovery epoch/fence
  -> reconcile pending/unknown Effects
  -> observe Windows/DSH/Pi/Provider/Vault/connectors
  -> resolve conflicts and missed occurrences
  -> reauthorize policy/bindings/budgets/secrets
  -> rebuild Context
  -> reattach or replace runtime
  -> resume/pause/block/quarantine
```

A responsive process, Conversation, connector, or checkpoint is not
automatically current. Offline/missed work is dispositioned by risk; publishing,
communication, spend, deletion, and expansion return to Owner review.

## 6. Conflict and revisions

Stale Project/Plan/Blueprint/Assignment/Vault/origin versions fail closed.
Timestamp or model judgment cannot choose a winner. The daemon preserves
versions, allows safe read-only observation, and requests deterministic merge,
compensation, or Owner confirmation.

Manager within-envelope changes remain versioned authority. Cross-boundary
changes require a structured preview.

## 7. Backup, restore, archive, delete

Authority backup/restore, content export, and same-disk local restore points are
distinct. Secrets are excluded from backup/export. Local restore points do not
protect against disk failure.

Project archive stops triggers and preserves read/export/restore. Permanent
deletion previews all authority, Conversation, Memory, Vault, artifacts,
bindings, pending Effects, and restore points and requires a separate
confirmation.

## 8. Current/target and non-claims

Current Task/Effect/fencing/verification authority remains **Now**. OPC Project/
Role/Employee, Conversation archive, Routine/missed, Windows/DSH recovery,
binding hierarchy, and HITL canvas composition (not Inbox L1) are **Requires-backend**; Windows
validation **Requires-environment**.

This architecture does not implement or qualify those targets and creates no
support, Gate, release, Profile, disaster-backup, 24/7, or Agent-benefit claim.
