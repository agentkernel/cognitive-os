# CognitiveOS Personal 2.0 OPC product design

- Status: canonical owner-approved product intent
- Change class: `product-semantic`
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Exact scope: [Personal 2.0 scope](personal-2.0-scope.md)
- Ordered behavior: [User journeys](user-journeys.md)
- Current-status owner: [PROGRESS.md](../../../docs/plan/PROGRESS.md)

## 1. Problem and evidence

The Owner wants to run long-lived business or development Projects through
digital employees without having to translate every decision into Agent,
Prompt, Tool, MCP, Loop, or Harness mechanics. The unresolved job is to know:
what is being attempted, who is responsible, what changed, what is blocked,
which decision needs the Owner, what it cost, and whether the result is
independently supported.

This is an **owner-directed requirements baseline**, not a validated market
finding. Evidence consists of the Owner's stated needs, the existing product
audit, and informative OSS/product research. There are no five-or-more ICP
interviews, behavior/frequency data, observed workarounds, retention data, or
willingness-to-pay evidence. Demand, adoption, usability, and monetization
remain hypotheses.

## 2. Target user and JTBD

The 2.0 user is one local human Owner: an OPC operator or individual developer
who understands their business but may not understand Agent infrastructure.
Projects, roles, and digital employees belong directly to the Owner. Optional
business/brand details reduce repeated setup but do not create a Company or
Business Space aggregate.

> When I delegate long-running work to digital employees, I want one local
> business console to define Projects, supervise progress and cost, approve
> risky changes, recover failures, and verify outcomes, so I can operate a
> one-person company without treating Agent self-report as truth.

Personal 2.0 is a Windows-local product while the host is online. Native mobile
and relay-based remote control begin in 2.1.

## 3. Goals

1. Make the next Owner decision and its consequence visible without a KPI wall.
2. Keep Project goals, team responsibility, work, artifacts, handoffs, cost,
   and verification in one governed local loop.
3. Let the Personal Assistant translate business intent into candidates and
   daemon-issued previews without becoming authority.
4. Provide a default DSH-backed digital-employee runtime with inspectable
   provenance, isolation, updates, and rollback.
5. Preserve A1–A8, secret isolation, Intent/Effect ordering, and independent
   completion verification across every surface.

## 4. P0 requirements

### P0-1 — Create an active governed Project

The Owner can complete a resumable conversational setup covering research,
charter, goals, metrics, team, plan, permissions, budgets, and triggers; review
one structured diff; confirm it; and receive a durable receipt. A Project is
inactive until its charter revision is confirmed.

### P0-2 — Operate Projects and digital employees daily

Today, Projects, Team, Knowledge, and Inbox expose the current goal,
responsibility, state, next action, latest verified evidence, and cost basis.
Each Project has exactly one current manager and manager-led
Task/artifact/handoff coordination.

### P0-3 — Converse and remember under Personal ownership

The Owner can converse with the Personal Assistant, manager, or employee using
one active composer at a time. Personal archives and indexes conversations by
scope, retrieves only bounded/redacted/provenance-bearing observations, and
admits semantic Memory only through policy. The Owner can inspect, correct, and
forget it.

### P0-4 — Run and recover on Windows through qualified managed DSH

DSH is the preinstalled managed Installed Agent and default employee runtime:
an exact audited artifact in an isolated child process behind a stdio broker and
daemon Provider proxy. Routines support manual, schedule, and qualified event
triggers; no overlap, queue-latest, offline/missed visibility, risk-based
resume, and an explicit close-window background/pause choice.

### P0-5 — Complete one controlled external-work scenario

An X/Twitter content operation can progress from research through a reviewed
publication package, qualified connector dispatch, receipt, feedback readback,
and independent acceptance. No fingerprint evasion, CAPTCHA bypass,
anti-abuse avoidance, blind retry, or unlicensed copying is permitted.

## 5. Product organization

The primary IA is **Today / Projects / Team / Knowledge / Inbox**. Settings is
fixed at the bottom. A global right-side **Personal Assistant** explains,
navigates, researches, and proposes.

- **Today:** a priority narrative of today's plan, Owner decisions, Project
  health, employee state, missed work, and latest evidence—not a metrics wall.
- **Projects:** governed workspaces and Project briefings; details descend into
  goals, plan revisions, Routines, Tasks, Attempts, Effects, artifacts, and
  evidence.
- **Team:** Role Blueprints, Project Role Assignments, digital employees,
  responsibility, state, next work, conversations, memory, and runtime.
- **Knowledge:** Owner-shared knowledge, project Markdown Vaults, imports,
  provenance, indexing, conflicts, and admitted Memory.
- **Inbox:** approvals, requested input, permission blocks, failures, unknown
  outcomes, missed runs, and budget warnings.
- **Settings:** Personal Home, Installed Agents, Providers/accounts, binding,
  usage, budgets, notifications, privacy, diagnostics, restore points, and
  advanced capabilities.

Default terminology is Project, Team, Digital Employee, Role Blueprint, Work
Plan, and Execution Record. Prompt means work instruction; Skill, work method;
Tool, executable action; MCP, connected application/capability; Loop, work
cycle; Harness, execution engine. Technical terms are progressively disclosed.

## 6. Success measures and counter-measures

These are **future fixed-denominator product acceptance measures**, not current
results:

| Measure | Target | Counter-measure |
|---|---:|---|
| Frozen Windows OPC scenarios passing | 15/15 | zero critical A1–A8, secret, external-effect, or false-completion failure |
| Consequential scenario previews bound to the applied revision | 100% | no stale preview accepted and every rejection preserves the draft |
| Started external actions with a terminal or explicit unknown/reconcile record | 100% | no unknown action blindly redispatched |
| Completed Tasks with current independent evidence | 100% | Agent/manager self-report alone contributes 0 completions |
| Required surface-state cases represented in design acceptance | 10/10 state classes per surface | unsupported actions remain non-interactive `Requires-backend`, not simulated success |

Human usability, time-to-value, adoption, retention, and willingness to pay
need separate human research before numerical product claims.

## 7. Out of scope

- native mobile, pairing, E2E relay, or remote control in 2.0;
- human-team accounts, Company/Business Space, RBAC, multi-tenancy, cloud
  authority, or offline-host execution;
- native DSH UI/conversation synchronization, an in-process DSH daemon, or a
  vendored fork;
- Hermes, Codex, Cursor, or another runtime as supported 2.0 adapters;
- MCP as a 2.0 success-path requirement or DSH native/base-tool grant;
- guaranteed business outcomes, full autonomy, all-platform publishing,
  browser reliability equivalent to an API, or multi-Agent benefit;
- disaster backup: same-disk versions are local restore points only.

## 8. Non-claims

This document changes product semantics and planning only. It does not implement
or qualify Windows, DSH, Pi, Project, Conversation, Vault, Trigger, UI,
Provider routing, budget enforcement, or X/Twitter connectivity. It creates no
Gate, release, Profile, market, usability, performance, containment, or
Agent-benefit evidence.
