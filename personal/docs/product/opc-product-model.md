# OPC product model: Projects, Role Runtimes, and Member Runtimes

- Status: adopted Personal 2.0 product semantics
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Scope: [Personal 2.0](personal-2.0-scope.md)
- Requirements:
  [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Current interaction prototype:
  [**personal-20-opc-e2e (post journey-subtraction)**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e.canvas.tsx)
- Archived historical V2 (not current chrome):
  [pre-subtraction history](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Prototype identity: current chrome is the post-workshop canvas. Archived V2
  is not current chrome. Canvas-only HITL and daemon authority path remain.
- Existing architecture input (pending reconciliation):
  [Project, Role, and Employee](../architecture/project-role-employee.md)

## 1. Product object chain

```text
Owner
  -> Project
       -> Charter / Goals / Metrics / Plan revisions
       -> operating-report template / temporary ad-hoc canvases
       -> reusable Role Runtime Template
            -> Project Member Runtime definition
                 -> Provider / model / capability grants
                 -> Project group + Member Conversation
       -> Routine / Trigger
            -> Occurrence -> Task
                 -> disposable Agent process / Attempt
                 -> bounded internal subagents
                 -> Effect / Artifact / Evidence
       -> Project Vault / Conversation archive / feedback evidence
```

These are product/domain concepts. They do not add a generic Core `Resource`
family, public DTO, or universal state machine. The visible CEO loop
(Ingest → Decide → Authorize → Execute → Verify → Report) is Control Plane
chrome over this chain, not a new domain object.

## 2. Project

A Project is a governed long-term workspace. It answers:

- why the work exists (charter and primary goal);
- how main -> phase/quarter when useful -> month -> week -> day/Task goals
  relate;
- which expected result, deliverable, cadence/due, owner, success criterion,
  evidence, and process measures count;
- who is responsible;
- which plan revision is current;
- which permissions, Provider bindings, cost/limit policies, and triggers are
  approved;
- which Tasks, Attempts, artifacts, handoffs, Effects, and evidence exist;
- what needs the Owner next.

A Project is not a directory, group conversation, Agent process, workflow,
Loop, or Harness. It owns the goal hierarchy, output contracts, current plan,
team, work cycle, triggers, deliverables, operating evidence, and decisions.
Its data directory is a managed human-readable companion. Moving or losing it
cannot silently delete daemon authority.

Project states include draft, researching, waiting-for-input, preview-ready,
confirmation-required, creating, active, attention, paused, archived,
restore-ready, and deletion-preview. Only confirmation of the exact charter
revision permits activation.

## 3. Role Runtime Template

A Role Runtime Template is a versioned reusable operating recipe:

- business purpose, responsibility, non-responsibility, and success contract;
- prohibited work plus input, deliverable, and handoff contracts;
- work instructions, Skills, Tools, and MCP capability requirements;
- work cycle, reflection, escalation, and collaboration behavior;
- required model capabilities, Context policy, and Memory policy;
- permission and safety boundaries.

It is not a process, conversation, Project Member, Provider credential, or
authority grant.

Only the base **Project Manager** Role is built in. Project setup
specializes it to the current charter. Governance, safety, reporting, and
Owner-escalation obligations cannot be removed. The Personal Assistant proposes
other Roles from Project needs after source-backed research.

A Project instantiates a Member Runtime by binding one Template revision to a
responsibility, subgoal, Provider/model, capability grant, permission, cost
policy, Context policy, and named Member. The Provider/model choice is explicit
during Member creation; an Assistant recommendation cannot bind silently. The
same Template can instantiate several Members, and Projects may pin different
Template revisions. A Member Runtime belongs to exactly one Project; Members
are not shared across Projects. Template reuse is the only cross-Project
reuse. A global Template upgrade never silently changes existing Members.

## 4. Project Member Runtime

A Project Member is a Project-specific long-lived Runtime definition with:

- name, Project, Role revision, responsibility, and assigned subgoal;
- current goal/work, state, next action, and latest verified result;
- group-chat identity, Member work Conversation, and private Memory;
- Task/Attempt/artifact/handoff history;
- explicit Provider/model selection, capability grants, cost basis, and
  permissions;
- the exact Runtime recipe used to start disposable Agent processes.

A Member Runtime is not an always-running process. It belongs to exactly one
Project and is not shared into another Project. Every Task execution starts
a separate Agent process/Attempt pinned to an exact Member Runtime revision.
Process exit, retry, engine update, or quarantine does not erase the Member,
conversation, Memory, deliverables, or evidence. Cards answer goal,
responsibility, current work, next run, latest accepted deliverable, block,
cost basis, and any Owner action.

## 5. Project Manager and collaboration

Every active Project has exactly one current Project Manager Member. The
manager:

- maintains the current plan, standard operating-report canvas, and daily
  briefing;
- decomposes approved goals into bounded Tasks;
- assigns responsibility and records explicit handoffs;
- surfaces blockers, missed work, budget pressure, and Owner decisions;
- performs Task, daily, cycle, and incident reflections;
- may validate and activate a new Member Runtime revision inside the approved
  envelope, with comparison evidence and rollback.

Its operating loop is observe -> plan -> delegate -> execute -> independently
verify -> summarize -> reflect -> adjust. It is responsible for planning,
assignment, verification, acceptance preparation, and escalation, not for
guaranteeing uncontrollable commercial results.

Inside the approved envelope, the manager may adjust subgoals, Tasks, order,
frequency, and member responsibility. Changes to the primary goal, team,
Provider/model, Tool/MCP grants, permissions, global Role Template, or
external-action rules require a revision preview and Owner confirmation. Cost
tracking and warnings do not automatically stop work in the 2.0 product.

The Project group conversation is the primary interaction surface:

- `@manager` requests a briefing or manager-owned Task assignment;
- `@member` asks or temporarily redirects goal/path inside the approved Task
  boundary; that message creates a formal Task revision, not a shadow plan;
- the manager speaks by default;
- Members speak proactively only when mentioned, submitting a deliverable,
  handing off, blocked, or requesting a decision;
- ordinary process traces remain collapsed behind the relevant work object.

Messages remain candidates/observations until translated into daemon-owned
Project revision, Task, Handoff, Effect, or verification facts.

## 6. Canvas and deliverable projection

Every Project starts from the system-default routine operating-report template.
The manager may version that Project's template without creating a global
template. An
ad-hoc Owner question lets the Assistant/manager compose a temporary canvas
from approved typed components and real goals, artifacts, evidence, decisions,
timeline, organization, and cost readings.

Temporary canvases are not saved unless pinned or converted into a Project
template. A canvas cannot execute generated code or `eval`, invent values, or
hide goal/acceptance state, failed/not-run work, Owner decisions, source, or
freshness. It is a projection, never authority.

## 7. Agent process, Conversation, and Context

DSH is the hidden default engine used to start a Member's Agent process. It is
not an Installed Agent product concept in 2.0. Exact version, provenance,
health, qualification, update, and rollback appear only for recovery or in
advanced diagnostics.

Conversation is owned by Personal. The Project group is shared by the Owner,
manager, and Members; a Member work conversation is visible to Owner, manager,
and that Member. Full local archives remain inspectable, while each Agent
process receives only a model-window-aware Context package:

`current Task contract -> fixed decisions -> relevant source/artifact excerpts
-> provenance-linked summaries -> older narrative`

Compression never deletes the source archive, replaces authority, proves
completion, or automatically admits Memory.

A Task process may create bounded internal subagents with explicit count, time,
cost, and permission limits. They have no Project-member identity or long-term
Memory; results return to the current Member.

The Personal Assistant is a global product identity, not a Project Member. Pi
may support it internally but remains hidden and owns no Project, Task, durable
Memory, secret, or authority.

## 8. Governed change and receipts

Consequential changes follow:

```text
business-language request
  -> candidate interpretation
  -> daemon-issued structured diff
  -> consequence / permission / cost preview
  -> Owner confirm or reject
  -> persisted Intent/Effect where applicable
  -> verified receipt
```

Stale revisions force a re-preview. Rejection preserves the draft. A receipt
states applied, rejected, partial, unknown/reconciling, or failed; unknown is
never success.

New instructions create versions and apply at a safe point through continue,
pause, or restart. They are never injected silently into a running prompt.

## 9. Non-claims

The model is **Requires-backend**. It does not assert that Project, Role
Template, Member Runtime, group Conversation, Routine, Trigger, or Attempt
authority and UI exist, that DSH is qualified on Windows, or that multi-Member
work improves results.
