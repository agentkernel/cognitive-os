# Project, Role, Assignment, and Digital Employee architecture

- Status: Personal 2.0 target; `Requires-backend`
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Product model: [OPC product model](../product/opc-product-model.md)

## 1. Bounded context and ownership

The Project context owns Project, Charter revision, Goal, Metric, Plan revision,
Role Blueprint, Project Role Assignment, Digital Employee, and policy links.
The daemon is the only writer. Runtime, Conversation, Knowledge, Provider, and
Task contexts reference Project identities through private application ports;
they do not mutate Project records directly.

These objects are not a generic Resource family and are not retrofits of
existing Task rows.

## 2. Conceptual aggregates

| Aggregate/root | Owns | References |
|---|---|---|
| Project | lifecycle, current Charter/Plan, current manager Assignment, policy/budget/trigger links | Goals, Assignments, Routines, Tasks, Knowledge |
| Role Blueprint | immutable revision, purpose, responsibilities, capability requirements, governance obligations | no concrete Project/Provider/runtime |
| Project Role Assignment | Project, Blueprint revision, responsibility/subgoal, capability/budget envelope, employee | Provider/runtime effective bindings |
| Digital Employee | stable Project identity, current Assignment, Conversation/Memory/work links | disposable Runtime executions |

A Project becomes active only after an exact Charter/initial Plan/team/policy
preview is confirmed. Each active Project has exactly one current manager
Assignment. Replacement preserves prior manager and handoff history.

## 3. Revisions and invariants

- Charter, Plan, Blueprint, and Assignment updates create revisions.
- Project Manager governance/safety duties cannot be removed by specialization.
- A Blueprint declares capability needs but no Provider binding or authority.
- Employee identity survives runtime restart/update/replacement.
- Manager may admit within-envelope subgoal/Task/order/frequency/responsibility
  changes only under current daemon policy.
- Primary goal, team, budget, Provider, Tool, permission, or external rule
  changes require Owner-confirmed revision.
- Agent/manager text is a candidate; it cannot set current revision.

## 4. Collaboration

The execution context owns Tasks/Attempts and Handoffs. A handoff binds source/
target Assignments, bounded work, current authority, artifacts/Context,
open/unknown Effects, budget, and readiness. The receiving employee gets only
reauthorized references and capabilities. Free group chat and Agent agreement
are observations.

## 5. Failure and recovery

Project activation is resumable through local/daemon drafts. Stale preview,
missing Project directory, unavailable runtime, or Provider failure does not
delete authority. Recovery reloads current revisions and fences stale manager/
employee work before reassignment.

Archive stops triggers and keeps read/export/restore access. Permanent deletion
requires a separate impact preview across Project, Tasks, Conversations,
Memory, Vault, artifacts, bindings, and restore points.

## 6. Contract boundary and non-claims

Concrete persistence, routes, private envelopes, errors, and transitions are
Phase 11 work. A public surface requires Lane-CTR. This chapter does not
implement Project/Role/Employee authority or claim Windows support, DSH
qualification, Gate, release, Profile, or multi-employee benefit.
