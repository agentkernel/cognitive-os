# 05 — Roles, Project Members, and governed conversations

- Requirements:
  [OPC requirements analysis](../../../../personal/docs/product/personal-2.0-opc-requirements-analysis.md)
- Product model:
  [OPC product model](../../../../personal/docs/product/opc-product-model.md)
- Status: current interaction prototype is owner-approved v9 (2026-08-30); v8 is the prior approved baseline (not overwritten); v5–v7 and archived pre-v5 / V2 are historical chrome only
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v9**](personal-20-opc-e2e-optimized-v9.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](history/2026-08-28-pre-subtraction/README.md)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

## Project-scoped roster

Members are opened from a Project; there is no first-level roster destination
or human-account team model. Every active Project has exactly one current
Project Manager Member. Only the base Project Manager Role is built in. The
Personal Assistant researches and proposes all other Roles.

The current chain is:

`Role Runtime Template -> Project-specific Member Runtime definition -> Task
-> Attempt -> disposable Agent process`

There is no additional employee object. Chrome object name is **Member
Runtime**. “Digital staff” remains marketing positioning only. Members are
not shared across Projects; only Role Runtime Templates may be reused.

## Role Runtime Template and Member creation

Every Template version defines purpose, responsibilities, prohibited work,
input/output/success/handoff contracts, instructions, Skills/Tools/MCP needs,
work cycle, collaboration/reflection, model capability, Context/Memory policy,
permissions, safety, and escalation.

A Project Member pins one Template revision and adds name, Project-specific
responsibility/subgoal, explicit Provider/model, capability grants, Memory
scope, cost basis, permission, and Runtime recipe. The Owner reviews the exact
Template and Member revisions. A global Template update never silently changes
existing Members.

## Member cards and detail

Current chrome (v9) member management is **select-then-configure**:

- Choose a person in the roster before any configuration appears.
- Unselected is an empty state. Do not default-select the first row.
- Switching Project clears the selection.
- Tabs: 职责 / 输入 / 输出 / 技能 / 工具 / 工作说明 / 周期与触发 / 连接与权限.
- Identity (model, seating, responsible stage) stays in the detail header;
  there is no overview tab that restates those six facts.
- Input is the read-only process contract (previous-stage handoff or project
  materials). Do not invent member-private input fields.
- Output is the editable 「交出什么」.
- Adding a member uses the current Project’s real roster, not a global sample
  list.

Cards still lead with:

- goal, responsibility, and expected deliverable;
- current work and next action;
- Working / Queued / Waiting activity; queued is not running;
- latest accepted result with evidence/source/freshness;
- block, missed work, or Owner decision;
- actual/estimated/unknown cost basis;
- Member Runtime version and rollback availability.

Work, group participation, Member work Conversation, Memory, grants, versions,
Attempts, artifacts, evidence, and advanced diagnostics are contextual tabs or
inspectors. Engine health is not the card's primary status.

## Project Manager loop and versioned improvement

The manager operates:

`observe -> plan -> delegate -> execute -> independently verify -> summarize
-> reflect -> adjust`

Reflection occurs per Task, day, cycle/week, and incident. A one-off Task
strategy adjustment may apply within the current boundary. A persistent Member
Runtime change creates a new version, runs replay/simulation/comparison where
available, records evidence, and retains rollback. Activation is allowed only
inside the approved autonomy envelope.

Primary-goal, team, Provider/model, Tool/MCP, permission, global Role, and
external-action-rule changes require a daemon preview and Owner confirmation.
Cost pressure produces source-labelled warnings, not an automatic product stop.

## Project group conversation

Inside a Project, the primary conversation contains Owner, manager, and
Members:

- manager speaks by default;
- Members speak proactively only when `@` mentioned, delivering, handing off,
  blocked, or requesting a decision;
- `@manager` requests a briefing or delegation;
- `@member` asks or temporarily redirects bounded goal/path; that message
  creates a formal Task revision, not a shadow plan;
- ordinary execution traces stay folded behind Tasks/Attempts;
- full group and Member-work archives remain inspectable under their scopes.
  A Member work conversation is visible to the Owner, the manager, and that
  Member.

Every message is conversation until a work-changing instruction becomes a
formal Task or revision. Group discussion, agreement, and routing never become
authority or independent verification. HITL is announced in the right-column
chat and linked to the center-canvas preview. Chat has no Approve control and
no “Don’t ask again” grant. Chat may narrate “Observed now”; self-report is
not completion. Why is layered (candidate reason, then kernel fact). `@`
inserts only into the unsent draft.

Outside a Project the global Personal Assistant is the visible conversation
in the same right column. Assistant/Project contexts retain independent
drafts. Switching context cannot merge, clear, or send. A consequential
suggestion opens a central daemon-issued structured preview.

## Process and engine separation

A Task starts a disposable Agent process/Attempt from an exact Member Runtime
revision. Process stop, failure, retry, DSH update, or quarantine preserves the
Member, conversations, Memory, artifacts, attempts, and evidence. Process death
does not delete the Member.

DSH and Pi are hidden managed engines. Their exact version, provenance, health,
qualification, update, and rollback appear only for fault recovery or advanced
diagnostics. No native engine UI, native session sync, alternate Harness
selector, or everyday engine-installation surface is designed.

## States and capability honesty

Roster and Conversation cover empty, researching Role, loading, partial,
stale, permission, offline, blocked, missed, unknown, failed-preserved-draft,
version-validating, rollback-available, archived, and receipt states.

Role/Member authority, manager policy, Project/group archives, Runtime
versioning, managed engines, and Assistant composition are
**Requires-backend**; Windows/DSH/Provider execution is also
**Requires-environment**. No prototype control may simulate these mutations.
