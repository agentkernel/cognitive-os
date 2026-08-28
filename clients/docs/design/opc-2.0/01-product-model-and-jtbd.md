# 01 — Product model and JTBD

- Requirements:
  [OPC requirements analysis](../../../../personal/docs/product/personal-2.0-opc-requirements-analysis.md)
- Product model:
  [OPC product model](../../../../personal/docs/product/opc-product-model.md)
- Status: Owner-accepted V2 interaction baseline (2026-08-28 competitive-informed
  overwrite; not a v3; not overlay-conversation / stacked-column V2)
- Interaction baseline:
  [**Owner-approved interaction baseline (2026-08-28)**](personal-20-ai-ceo-e2e-optimized-v2.canvas.tsx)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

## Evidence boundary

This direction is grounded in the Owner's confirmed `/grill-me` requirements,
a current-product audit, and informative source research. It lacks multiple ICP
interviews, observed workaround/frequency data, analytics, retention,
willingness-to-pay evidence, and executed usability studies. JTBD,
prioritization, and success measures are owner-approved hypotheses, not market
validation.

## Positioning and primary job

Personal 2.0 is an **AI-native digital-staff console** for one local OPC
Owner-operator or individual developer. “Digital staff” is positioning
language; it does not add an object between Role and Runtime. The accepted V2
interaction chrome makes the job visible as Ingest → Decide → Authorize →
Execute → Verify → Report, with Today as one decision packet plus four
exception swimlanes. That chrome is UX, not a new domain object.

> When I run a governed Project, I want to describe goals and acceptable
> outputs in business language, let a manager organize and improve bounded
> work, talk to the team in one Project group, and receive source-linked,
> independently verified deliverables, so I can operate without becoming an
> Agent-infrastructure administrator.

Supporting jobs:

1. turn a business situation into a researched, safely activated Project;
2. see expected outputs, accepted results, exceptions, and decisions due now;
3. delegate or redirect bounded work through the manager and Members;
4. inspect, accept, reject, or revise openable deliverables and evidence;
5. connect a model and acquire reviewed capabilities without leaking secrets;
6. preserve identity, conversations, Context sources, Memory, and attempts
   across disposable processes;
7. recover missed, failed, stale, partial, or unknown work without false
   completion or blind retry.

## Product object chain

```text
Owner
  -> global Personal Assistant
  -> Project
       -> Charter / goal hierarchy / output contracts / Plan revisions
       -> Project group Conversation / operating canvas
       -> Project Manager Member + other Project Members
            -> pinned Role Runtime Template revision
            -> Project-specific Member Runtime definition
            -> explicit Provider/model + scoped capability grants
       -> Routine / Trigger / occurrence
            -> Task -> Attempt -> disposable Agent process
                 -> bounded internal subagents
                 -> Artifact / Intent / Effect / Evidence
       -> Project Vault / conversation archive / admitted Memory
```

- A Project is a long-lived governed workspace, not a directory, chat, Loop,
  Harness, process, or Task bucket.
- A Role Runtime Template is a reusable versioned operating recipe, not a
  process, credential, Member, or authority grant.
- A Project Member Runtime definition persists responsibility, configuration,
  grants, Conversation, Memory, and history for one Project.
- A Task Attempt starts a disposable Agent process from an exact Member
  revision. Process exit never deletes the Member or proves completion.
- The Project group is the primary interaction surface. A message remains a
  candidate until translated into a daemon-owned Task or revision.
- Only the base Project Manager Role is built in. The Assistant researches and
  proposes other Roles.

## Manager responsibility and autonomy

The manager operates:

`observe -> plan -> delegate -> execute -> independently verify -> summarize
-> reflect -> adjust`

Inside the approved envelope it may change subgoals, Task decomposition/order,
frequency, and bounded Member responsibility. Primary-goal, team,
Provider/model, Tool/MCP, permission, global Role, and external-action-rule
changes require an exact preview and Owner confirmation. Cost warnings inform
decisions but do not automatically stop Personal work.

## Default language

| Advanced term | Default product language |
|---|---|
| Prompt | work instruction |
| Skill | work method |
| Tool | executable action |
| MCP | connected application/capability |
| Loop | work cycle |
| Harness | execution engine, disclosed only in diagnostics |
| Agent process | one disposable Task Attempt |

## Desired outcome and non-claims

The desired outcome is a Windows-local, host-online Project loop in which every
goal has expected results, deliverables, success criteria, evidence, state, and
a recovery path. This design does not promise a shipped backend, qualified
Windows/DSH/Pi/Provider/MCP/X path, guaranteed business result, full autonomy,
offline-host execution, browser/API equivalence, all-platform publishing,
market demand, or multi-Agent benefit. Formal acceptance remains pending plan
reconciliation.
