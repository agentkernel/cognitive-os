# OPC product model: Projects, roles, and digital employees

- Status: adopted Personal 2.0 product semantics
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Scope: [Personal 2.0](personal-2.0-scope.md)
- Architecture pair: [Project, Role, and Employee](../architecture/project-role-employee.md)

## 1. Product object chain

```text
Owner
  -> Project
       -> Charter / Goals / Metrics / Plan revisions
       -> Project Role Assignment
            -> Digital Employee Instance
                 -> Agent Runtime
                 -> Personal-owned Conversation
       -> Routine / Trigger
            -> Task -> Attempt -> Effect / Artifact / Evidence
```

These are product/domain concepts. They do not add a generic Core `Resource`
family, public DTO, or universal state machine.

## 2. Project

A Project is a governed long-term workspace. It answers:

- why the work exists (charter and primary goal);
- which outcomes and process measures count;
- who is responsible;
- which plan revision is current;
- which permissions, Provider bindings, budgets, and triggers are approved;
- which Tasks, Attempts, artifacts, handoffs, Effects, and evidence exist;
- what needs the Owner next.

A Project is not a directory, conversation, Agent team, workflow, Loop, or
Harness. Its data directory is a managed human-readable companion. Moving or
losing it cannot silently delete daemon authority.

Project states include draft, researching, waiting-for-input, preview-ready,
confirmation-required, creating, active, attention, paused, archived,
restore-ready, and deletion-preview. Only confirmation of the exact charter
revision permits activation.

## 3. Role Blueprint and Assignment

A Role Blueprint is a versioned reusable description of business purpose,
responsibilities, capability requirements, working methods, collaboration
expectations, and governance obligations. It does not own a Provider, runtime,
conversation, permission, or employee history.

Only the base **Project Manager** blueprint is built in. Project setup
specializes it to the current charter. Governance, safety, reporting, and
Owner-escalation obligations cannot be removed. The Personal Assistant proposes
other roles from Project needs.

A Project Role Assignment binds one Blueprint revision to one Project,
responsibility set, subgoal, capability/budget envelope, and named employee.
The same Blueprint can create several employees, and projects may pin different
Blueprint revisions. Upgrades show a diff and are opt-in per Project.

## 4. Digital Employee Instance

A digital employee is a long-lived Project identity with:

- name, Project, Assignment, and responsibility;
- current goal/work, state, next action, and latest verified result;
- employee-scoped Conversation and private Memory;
- Task/Attempt/artifact/handoff history;
- effective Provider/budget/permission facts;
- one or more disposable runtime executions.

An employee is not a process. Runtime restart, replacement, or quarantine does
not erase the employee, its conversations, or evidence. Cards always answer:
goal, responsibility, state, next action, last verified fact, and cost basis.

## 5. Project Manager and collaboration

Every active Project has exactly one current manager. The manager:

- maintains the current plan candidate and daily briefing;
- decomposes approved goals into bounded Tasks;
- assigns responsibility and records explicit handoffs;
- surfaces blockers, missed work, budget pressure, and Owner decisions;
- proposes reflections and revisions.

Inside the approved envelope, the manager may adjust subgoals, Tasks, order,
frequency, and member responsibility. Changes to the primary goal, team,
budget, Provider, tools, permissions, or external-action rules require a
revision preview and Owner confirmation.

Agent messages, free group chat, self-critique, or manager agreement are
candidates/observations. Collaboration authority lives in daemon-owned Task,
Assignment, Handoff, Effect, and verification facts.

## 6. Runtime and Conversation

Agent Runtime is how an employee executes. DSH is the preinstalled managed
Installed Agent and default runtime for 2.0; exact version and qualification
facts remain visible in advanced Settings. Runtime bytes/process/session are
separate identities from the employee.

Conversation is owned by Personal and scoped to the Owner, Project, and
employee. DSH receives bounded Context and returns candidate output. There is
no native DSH UI or conversation synchronization in the 2.0 product.

The Personal Assistant is a global product identity, not a Project employee.
Pi may support it internally, but Pi is hidden from the ordinary Installed
Agents list and owns no long-term Conversation/Memory or authority.

## 7. Governed change and receipts

Consequential changes follow:

```text
business-language request
  -> candidate interpretation
  -> daemon-issued structured diff
  -> consequence / permission / budget preview
  -> Owner confirm or reject
  -> persisted Intent/Effect where applicable
  -> verified receipt
```

Stale revisions force a re-preview. Rejection preserves the draft. A receipt
states applied, rejected, partial, unknown/reconciling, or failed; unknown is
never success.

## 8. Non-claims

The model is **Requires-backend**. It does not assert that Project, Blueprint,
Assignment, Employee, Conversation, Routine, or Attempt authority and UI exist,
that DSH is qualified on Windows, or that multi-employee work improves results.
