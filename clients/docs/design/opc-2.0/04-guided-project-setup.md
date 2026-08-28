# 04 — Research-first guided Project setup

- Requirements:
  [OPC requirements analysis](../../../../personal/docs/product/personal-2.0-opc-requirements-analysis.md)
- Status: current interaction prototype is post journey-subtraction; archived V2 is historical chrome only
- Current interaction prototype:
  [**personal-20-opc-e2e (post journey-subtraction)**](personal-20-opc-e2e.canvas.tsx)
- Archived historical V2 (not current chrome):
  [pre-subtraction history](history/2026-08-28-pre-subtraction/README.md)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

## Pattern and first value

The pattern is a resumable Assistant conversation synchronized with structured,
editable sections. First value is an active Project canvas and group
conversation with a confirmed charter, goal/output contracts, manager/team,
first work cycle, permissions, triggers, and receipt—not a tour or fabricated
celebration.

## Setup sequence

```text
business understanding
  -> broad automatic web research
  -> charter
  -> main/period/day goals + output/evidence contracts
  -> Project Manager + researched Member Roles
  -> plan and work cycle
  -> explicit Provider/model + required capabilities
  -> permissions, autonomy envelope, HITL/external-action policy
  -> triggers and no-overlap/queue-latest behavior
  -> one-cycle simulation
  -> structured launch preview
  -> Owner edit / narrow / reject / confirm
  -> active Project receipt
```

Research seeks sufficiently broad, high-quality coverage without asking for
each ordinary web read. Sources, freshness, rights, conflicts, gaps, and
coverage remain visible. Non-secret Project context may be used; raw
credentials and third-party data the Owner cannot disclose may not.

## Goal and output contract

Every goal records expected result, openable deliverables, due/cadence,
responsible Member, success criteria, evidence, and uncontrollable-outcome
limits. Goal depth is main -> phase/quarter when useful -> month -> week ->
day/Task. The manager owns planning and acceptance preparation, not guaranteed
followers, revenue, quality, or other uncontrollable results.

## Role, model, and capability setup

Only the base Project Manager Role is built in. The Assistant researches and
proposes each additional Role Runtime Template with purpose, responsibility,
prohibited work, input/output/handoff/success contracts, instructions, work
cycle/reflection, Context/Memory, model needs, Skills, Tools, MCP, permissions,
and escalation.

Each Project Member adds Project-specific responsibility, pinned Template
revision, explicit Provider/model choice, grants, Memory scope, and
permissions. Members are not shared across Projects. Recommendations cannot
bind silently.

Skill candidates may install automatically only after source, exact version,
license, hidden-instruction, prompt-injection, and file/network/command-intent
review. MCP additionally reviews dependency, executable code, network, Secret,
Tool permission, and supply chain; first installation or permission expansion
requires exact Owner confirmation. Acquisition and Project/Member grants are
separate.

## Simulation and structured launch review

The one-cycle simulation exposes:

- planned delegation and handoffs;
- Task/output/evidence contracts;
- Context sources and likely model-window compression;
- permissions and human-intervention points;
- external actions plus Intent/Effect/receipt flow;
- trigger overlap, queue-latest, missed/offline behavior;
- actual/estimated/unknown cost basis and warnings;
- missing backend/environment capabilities.

The confirmable review is daemon-issued and digest-bound. It includes editable
charter, goals, outputs, team, plan, models, capabilities, autonomy/HITL,
triggers, simulation findings, risks, omissions, and recovery. Launch preview
is the full AUTONOMY packet on the canvas; there is no Confirm in chat.
Assistant prose remains a Candidate.

## State model

| State | UI behavior |
|---|---|
| `local-draft` | client custody and safe-leave basis shown |
| `daemon-draft` | daemon identity/version exists; Project inactive |
| `researching` | exact sources, progress, partial results, conflicts, cancellation |
| `waiting-owner` | exact input, account, credential, permission, or decision |
| `partial` | usable sections and missing facets remain distinct |
| `simulation-failed` | retained draft, failed step, edit/retry route |
| `review-ready` | validated structured revision; still inactive |
| `re-preview` | stale preview, changed facts, preserved edits |
| `creating` | durable operation, safe-leave behavior, no fake success |
| `failed` | failed stage, retained work, safe recovery |
| `active-receipt` | Project/revision, team, plan, triggers, first output, next action |

## Form, recovery, and capability honesty

Fields use visible labels, pre-input constraints, connected errors, async
status, preserved non-secret values, error summaries, and predictable focus.
External text is untrusted and cannot execute, install, change schema, or
expand permission. A stale preview returns to review. Offline/research/model/
permission failure preserves completed work and exact custody.

Project draft authority, research orchestration, Role/Member creation,
capability review/acquisition, simulation, launch preview, activation, and
receipt are **Requires-backend**; Windows, DSH, Provider, MCP, and X execution
also require applicable environments. The prototype may demonstrate only
labelled example states, never a real Project id or acceptance receipt. There
are no Connect / Install / Confirm fake buttons.
