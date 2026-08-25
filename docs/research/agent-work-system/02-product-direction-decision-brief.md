# CognitiveOS Agent Work System — product direction decision brief

Date: 2026-08-25
Status: **rounds 1–5 confirmed; detailed candidate documents generated**

This brief proposes a coherent product direction before detailed Personal and
Enterprise product, interaction, visual, and architecture specifications are
written. Rounds 1–5 record thirty-four owner-confirmed selections made on
2026-08-25. The owner supplied custom intent text only for the Round 4
Enterprise wedge; no other written rationale was supplied. Original
recommendations remain visible and are marked where a confirmed choice
superseded them. The detailed documents `03`–`10` remain candidate,
non-canonical discovery and provide no implementation authorization.

## 1. Recommended product thesis

### 1.1 Umbrella positioning

**[CONFIRMED — ROUND 1]** Use **CognitiveOS — AI Workforce OS** as the umbrella
positioning.

The owner selected this language without supplying a written rationale. The
original recommendation—**CognitiveOS — Governed Agent Work System** as the
testable product category—is preserved here but **superseded for umbrella
positioning**.

The original proposed product promise remains a useful candidate:

> Turn intent into Agent work that is assigned, bounded, observable,
> recoverable, and accepted through evidence.

**[CONFIRMED — ROUND 2]** “AI Workforce OS” is vision/category language only.
Personal uses literal Agent, Provider, entitlement, binding, usage, cost, and
Task terms, without an employee/company metaphor.

The original concern remains relevant context: employee replacement, company
simulation, HR administration, and organizational breadth create expectations
that have not been validated.

### 1.2 Personal positioning

**[CONFIRMED — ROUND 2]**

> CognitiveOS Personal is the local-first operations workspace where one person
> assigns meaningful work to their Agents, supervises by exception, controls
> Provider access and cost, and accepts results only when evidence supports
> completion.

Personal is not:

- a general-purpose chat client;
- a gallery of Agent personalities;
- a one-person company simulator;
- a Provider billing portal;
- a complete project-management or knowledge-management product.

### 1.3 Enterprise positioning

**[CONFIRMED — ROUNDS 4–5]**

> CognitiveOS Enterprise is a governance and execution-assurance plane that
> connects organizational intent, identity, policy, scoped resources, local
> authority daemons, evidence, and accountability without replacing enterprise
> systems of record.

Enterprise is a future adjacent product, not a feature toggle inside the
Personal MVP. Its confirmed initial wedge is execution assurance and governed
work, extended with portable continuation across supported Agent tools.
Provider/subscription management is the second priority.

## 2. Recommended product form

### 2.1 Personal form

**[CONFIRMED — ROUND 1]** Personal's first product form is a **native desktop
application**.

**[CONFIRMED — ROUND 2]** Use a native desktop shell around the existing Web
client. Validate Windows first, then decide macOS/Linux priority from evidence.
The owner supplied no written rationale.

The original Round 1 recommendation—deliver the desktop-grade control plane
through the responsive Web client with a CLI companion—remains superseded as
the product form, while its Web-client reuse strategy is now confirmed inside
the native shell. Fully platform-native UI and a new cross-platform
desktop-toolkit UI were not selected.

**[CONFIRMED — ROUND 3]** Do not select the shell technology yet. Compare at
least two candidates through equivalent security and packaging spikes. A
Tauri-like Rust-native shell, Electron, or another implementation remains an
ADR/spike candidate—not an accepted architecture choice.

The comparison must cover process isolation, WebView/runtime provenance,
update signing, permission surface, narrow IPC, secret handling,
accessibility, packaging, and existing repository fit.

Recommended surface roles:

| Surface | Role |
|---|---|
| Native Personal desktop application | Primary Agent/Provider inventory, readiness, entitlement, binding, usage, and cost cockpit |
| Formal Personal Web client | Reused presentation layer inside the native desktop shell |
| Local daemon | Sole authority writer and execution coordinator |
| `cognitive` CLI | Expert automation, diagnostics, and recovery companion |
| Native dsh panel | Runtime-specific operational surface; not the Personal control plane |
| Provider first-party pages | External billing, plan, and unsupported entitlement management |

### 2.2 Enterprise form

**[CONFIRMED — ROUND 4]** Model Enterprise as a **central Web governance plane
and integration service** paired with customer/node authority daemons.

- Central plane: registry, sponsor, policy source, allocation, approval,
  projections, incident links, and connectors.
- Node daemon: Task, lease, Intent, Effect, dispatch, verification, local
  evidence, and safe recovery.
- Synchronization: signed/versioned requests and projections, never direct
  SQLite access.

The owner selected a desktop fleet application as the primary Enterprise UI.
The central Web governance backend and desktop-primary UI can coexist. Round 5
must decide whether a Web operator UI remains a supported fallback/deep-admin
surface or is not a product surface.

## 3. Recommended primary user and first scenario

### 3.1 Personal primary user

**[CONFIRMED — ROUND 1]** Start with a technical individual operator who uses
multiple coding or research Agents. The owner supplied no written rationale.
The previously stated user profile is retained as a provisional elaboration:

- uses at least two coding or research Agent tools every week;
- already uses Git, issues, CI, checklists, or human review;
- has at least one Provider API account or Agent subscription;
- delegates bounded but non-trivial work;
- cares about credential isolation, cost, evidence, and safe continuation.

Do not optimize the first version for a casual consumer seeking instant
answers. The authority and evidence model would add friction without solving
that user's primary job.

### 3.2 First high-value scenario

**[CONFIRMED — ROUND 1]** Start with **Agent/Provider inventory, entitlement,
and cost visibility**.

The original recommendation—assign one bounded technical Task to one installed
Agent and supervise it to an evidence-backed result—is preserved below but
**superseded as the first value path**. It remains the recommended next
expansion after the confirmed P0 activation boundary.

```text
Discover
→ Review
→ Register
→ Link approved Provider access
→ Verify readiness
→ Bind Agent/Profile/Instance to account/model
→ Show entitlement/usage/cost source and freshness
→ Ready
```

Recommended activation outcome:

> A first user is activated only after one Agent is registered, one supported
> Provider access path is linked through approved authentication, readiness is
> verified, an explicit Agent/Profile/Instance ↔ account/model binding exists,
> and entitlement/usage/cost status is displayed with source and freshness.

This prevents a passive-dashboard dead end: the cockpit must end in a verified,
usable binding rather than a collection of disconnected inventory rows.

Keep consumer plan, API account, authentication method, `SecretRef`,
entitlement, budget, usage observation, and cost observation conceptually
separate. A value may be user-declared, Provider-reported, locally observed,
estimated, stale, unavailable, or unknown; the UI must identify the source and
freshness rather than collapse these facts into one “subscription” state.

P0 may invoke an already approved authentication/SecretStore path to link
Provider access, but it does not implement credential acquisition, secret
management, or a general authentication lifecycle. Credential scraping, cookie
import, password storage, token copying into ordinary configuration, secret
display, and unsupported consumer-plan or subscription mutation remain outside
P0.

The superseded task-first flow remains the candidate next expansion:

```text
Capture intent
→ clarify outcome and acceptance
→ preview scope, Agent, Provider, resources, and budget
→ admit and assign
→ observe wakeup and governed execution
→ handle one blocker or decision if needed
→ inspect verification and Effects
→ accept the result or follow the durable recovery action
```

## 4. Recommended core object model

### 4.1 User-facing objects

| Object | User meaning | Recommended MVP treatment |
|---|---|---|
| Personal Workspace | The owner's local operating scope | One implicit workspace; no switcher |
| Goal-lite | Desired longer-term outcome or external intent anchor | Optional label/reference; no independent authority state |
| Task | Bounded work with scope, budget, resources, and acceptance | Primary authority and product object |
| Assignment | The Agent selected to handle a Task | Typed, versioned, distinct from lease |
| ExecutionAttempt | One wakeup-to-terminal execution presentation | Derived projection before a new persisted Run domain |
| Result | Human-readable accepted outcome | Projection backed by acceptance evidence |
| Evidence | Verification, Effects, artifacts, and source facts | First-class product surface |
| Agent Profile | Purpose, capability, supported Task families, and compatibility | Distinct user-facing concept with identity, purpose, capability source, compatibility, and bindings |
| Agent Instance | Discovered, registered, installed, or runnable deployment with health | Distinct from Profile; shows health and concrete binding participation |
| Provider API account | External account/reference used for supported API access | Separate from consumer plan and authentication |
| Authentication path | Approved mechanism used to obtain Provider access | Secret material remains in an approved Secret Store |
| `SecretRef` | Non-secret reference to approved stored secret material | Never a credential value or scrape result |
| Consumer plan | User-known or supported read-only plan fact | Not assumed to expose an API or mutable subscription lifecycle |
| Entitlement | Source-typed right or limit for models/features/usage | Separate from plan, budget, and observed consumption |
| Agent ↔ Provider/model binding | Explicit selection of the access path and model an Agent may use | Required activation outcome; not inferred from inventory |
| Budget | Owner-controlled local spending/usage boundary | Not Provider billing truth |
| Usage observation | Source-typed consumption fact | Provider-reported, locally observed, estimated, stale, unavailable, or unknown |
| Cost observation | Source-typed amount, estimate, or status | Separate from budget and entitlement |
| Resource | Context, Memory, Skill, Tool, Artifact, or external Knowledge source | Separate families with honest capabilities |
| Activity | Cross-domain timeline of authority events and observations | Read projection; not chat |

These are discovery-level product distinctions, not final schema or API
contracts.

**[CONFIRMED — ROUND 3]** After P0, Task is the first authority object. Goal is
a lightweight outcome/reference, and Workstream remains deferred until
repeated cross-Task demand is validated.

### 4.2 Objects deliberately deferred

Under the confirmed metaphor and P0 defaults:

- **[CONFIRMED — ROUND 3]** full Workstream lifecycle until repeated cross-Task
  coordination demand is validated;
- organization hierarchy;
- persistent generic Run entity;
- recurring Routine editor;
- generic KnowledgeBase;
- multi-Agent planning and negotiation;
- universal Subscription lifecycle.

### 4.3 Assignment versus execution ownership

**[CONFIRMED — ROUND 3]** Keep three questions separate:

1. **Assignment:** which ready Agent/Profile/Instance binding did the owner
   explicitly choose?
2. **Eligibility:** is that Agent currently healthy, compatible, authorized,
   and correctly bound?
3. **Execution ownership:** which attempt currently holds the scheduler's
   fenced lease?

This prevents an assigned Agent from acquiring permanent authority over a Task.
For the next vertical slice, the system previews scope, Provider/model,
resources, budget, and acceptance before admission; after admission it
executes within admitted authority and interrupts only on exception.

## 5. Recommended Personal information architecture

The first native desktop surface is an **inventory and readiness cockpit**.
**[CONFIRMED — ROUND 3]** Personal uses a persistent sidebar with Home, Agents,
Providers, Work, Activity, and System. First use enters a resumable activation
wizard; after activation, users land on a card-led Home.

| Area | Primary question | Recommended content |
|---|---|---|
| Home | What is ready, what needs attention, and where should I continue? | Card-led status and milestone summaries with deep links to durable object state |
| Readiness | What can I use now, what is missing, and what should I do next? | Activation progress, Agent and Provider/model readiness, explicit bindings, source-typed entitlement/usage/cost status |
| Agents | Which Agents are known and runnable? | Discovery/registration source, Profile/Instance distinction, compatibility, health, and bindings |
| Providers | Which access paths and economic constraints exist? | Separate plan/API account/auth/`SecretRef`/entitlement/model/budget/usage/cost facts |
| Work | What governed work exists and what is its truth? | Next expansion: Task master/detail, Assignment, attempt timeline, evidence, blockers, recovery |
| Resources | What Context, Memory, Skill, Tool, Artifact, and Knowledge refs are available? | Family-specific inventory, provenance, scope, and use |
| Activity | What changed, who/what caused it, and is it authority or observation? | Filterable cross-domain event projection |
| System | Is the local authority environment healthy and recoverable? | Daemon, storage, sessions, diagnostics, backups, version, support evidence |

Recommended global shell:

- persistent left navigation for Home, Agents, Providers, Work, Activity, and
  System;
- visible current scope and daemon health;
- global command/search for objects and supported actions;
- one contextual activation action until a usable binding exists, such as
  “Register Agent,” “Link Provider,” “Verify readiness,” or “Create binding”;
- “New Task” becomes the primary action only if governed Task execution is
  explicitly included in P0 or enters the next expansion;
- attention badge based on actionable items, not raw event count;
- route and selected-object state preserved through desktop navigation and,
  when a Web view is reused, in the URL;
- unsupported capability shown as unavailable with a reason, never as a fake
  disabled lifecycle.

### 5.1 Readiness cockpit

**[CONFIRMED — ROUND 3]** During first run, the launch surface is a resumable
**activation and readiness wizard**, not analytics. Activated returning users
land on card-led Home.

First viewport order:

1. Activation progress toward one verified usable binding.
2. Next required action and why it is needed.
3. Agent readiness and discovery/registration source.
4. Provider/model access, entitlement, and binding status.
5. Usage/cost source, freshness, and unknowns.
6. Blocked, failed, unsupported, or stale setup facts.

Counts or charts may appear only when they change an owner decision. Avoid
large decorative charts, generic “productivity” scores, vanity Agent counts,
or totals that hide source and freshness.

### 5.2 Work

**[ORIGINAL RECOMMENDATION — NEXT EXPANSION]** Work uses a **master/detail
operational workbench** after P0 activation.

Desktop:

- stable filterable Task list;
- selected Task remains visible;
- central detail with outcome, status, Assignment, and current next action;
- timeline/evidence inspector available without losing list position.

Task list columns should prioritize:

- Task outcome/title;
- authority status;
- assigned Agent;
- blocker/attention reason;
- last durable event;
- cost status;
- next action.

Do not use a card grid for dense comparison.

### 5.3 Task creation — next expansion

**[RECOMMENDATION]** Use a two-speed flow:

1. Quick capture: outcome in plain language.
2. Structured preview: scope, acceptance, Agent, Provider binding, resources,
   budget, and consequences.

The preview is a review step, not a long setup wizard. Returning users may use
templates or recent settings, but the exact authority summary remains visible
before admission.

## 6. Recommended primary interaction pattern

For the selected first value path, the primary pattern is a **guided readiness
workbench** combining:

- dense, inspectable Agent and Provider inventory;
- an explicit activation checklist with one recommended next action;
- review before importing discovered facts or linking access;
- approved authentication with only non-secret `SecretRef` presentation;
- explicit binding rather than inferred compatibility;
- source and freshness labels for entitlement, usage, and cost;
- honest unavailable, stale, unsupported, and unknown states.

### 6.1 P0 activation hierarchy

| Moment | Primary action | Secondary actions |
|---|---|---|
| Empty | Discover Agents | Explain supported discovery; enter manually |
| Discovered | Review proposed facts | Inspect provenance; reject, correct, or continue manually |
| Reviewed | Register one Agent/Profile/Instance | Confirm identity and purpose; preserve provenance |
| No Provider access | Link one supported access path | Review supported auth; keep unsupported paths read-only/unavailable |
| Access linked | Verify Provider/model readiness | Refresh entitlement; inspect auth source without revealing secret material |
| Ready but unbound | Bind Agent/Profile/Instance to account/model | Change model/access path; review local budget |
| Bound | Inspect entitlement/usage/cost source and freshness | Refresh supported sources; inspect unavailable or stale facts |
| Ready | Continue from the usable binding | Governed Task execution remains the next expansion |

Discovery must not silently create trusted inventory, authentication, or
bindings. **[CONFIRMED — ROUND 2]** Discovery is user-triggered, every proposed
fact is reviewed before import, provenance is retained per fact, and manual
registration remains available.

### 6.2 Governed Work pattern — next expansion

The original task-first recommendation is preserved as the next expansion:
an **exception-first operational workbench** combining:

- priority queue for attention;
- master/detail for Work;
- plan/authority preview before consequential execution;
- progress + artifacts + attempt timeline for long-running work;
- evidence receipt after verification;
- revision-bound decision cards;
- durable recovery cards after interruption or failure.

#### Task interaction hierarchy

| Moment | Primary action | Secondary actions |
|---|---|---|
| Capture | Create Task | Use template, link external intent |
| Preview | Admit and assign | Edit scope, change Agent, adjust budget, cancel |
| Queued | Inspect readiness | Cancel if supported, change Assignment before lease |
| Running | Inspect current authoritative phase | Stop/pause only if backend truthfully supports it; inspect actions/artifacts |
| Waiting | Resolve one decision | Narrow scope, deny, provide non-secret input |
| Failed | Follow recommended recovery | Inspect evidence, retry if safe, reassign |
| Accepted | Review result and evidence | Open artifacts, export receipt, create follow-up |

### 6.3 Control honesty

The UI must not expose pause, stop, restart, quarantine, undo, or resume merely
because the concept is desirable. Each control requires an actual daemon route,
defined state transition, failure behavior, and verification path.

If an action cannot interrupt an in-flight external mutation, say so before the
run starts and distinguish “stop future steps” from “undo completed effects.”

## 7. Recommended supervision, evidence, and recovery UX — next expansion

This section preserves the original governed Task recommendation. It is not
part of the confirmed P0 boundary.

### 7.1 Supervision

**[RECOMMENDATION]** Show the current **authority phase**, not simulated
thinking:

```text
Proposed → Previewed → Admitted → Assigned → Queued
→ Lease acquired → Candidate running → Effect pending
→ Verification → Accepted / Failed / Blocked
```

The activity timeline should visually distinguish:

- authority transition;
- Agent candidate statement;
- process/runtime observation;
- Tool Intent;
- external Effect;
- verifier result;
- owner decision;
- cost/usage observation.

Do not turn this into a chat transcript. Human-readable summaries may expand
into technical detail.

### 7.2 Evidence layering

**[CONFIRMED — ROUND 3]** Use three depths:

1. **Card summary:** accepted/not accepted, verifier, key Effects, artifacts,
   cost status, and remaining limitations.
2. **Readable detail:** acceptance criteria, passed/failed checks, before/after
   facts, sources, and excluded claims.
3. **Audit grade:** immutable source/event references, sequence, digests,
   contract epoch, exact revisions, raw bounded reports, and export.

Never display a generic numerical confidence score unless it has a calibrated,
task-specific meaning. Prefer evidence basis, scope, freshness, and missing
facts. Authority versus observation, source, freshness, and missing facts
remain explicit at every layer.

### 7.3 Decision cards

Every owner decision should show:

- what is requested;
- why the system stopped;
- affected Task/resources;
- exact revision or action digest;
- risk and reversibility;
- estimated/known cost state;
- recommended choice;
- alternatives and consequences;
- expiry/staleness behavior.

Approval does not complete the downstream action; it permits reevaluation.

### 7.4 Recovery cards

Every failure or interruption should answer:

1. What failed?
2. Which durable facts were committed?
3. Which effects definitely occurred, may have occurred, or did not occur?
4. Is retry safe?
5. What single next action is recommended?
6. Who owns that action?

Recommended recovery states:

- retry same attempt from a safe checkpoint;
- reassign after revoking stale Assignment;
- narrow scope and create a new contract epoch;
- resolve missing Provider/Resource access;
- wait for an external dependency;
- stop and require owner action for unknown worktree or irreversible ambiguity.

## 8. Recommended visual and layout direction

Status: **Rounds 2–3 visual direction confirmed; detailed values still require
rendered validation**.

### 8.1 Desired character

**[CONFIRMED — ROUND 2]** Lead with **consumer-style spacious cards and
wizard-led setup**. The owner selected this alternative without supplying a
written rationale.

The original recommendation—calm, dense, precise, professional, and
trustworthy desktop operations—was **superseded as the leading visual and
first-run direction**, but its scan-speed concern remains relevant for
high-volume inventory and evidence work.

Avoid:

- glassmorphism as a theme;
- giant gradients or hero layouts;
- playful virtual-employee avatars;
- color-only status;
- terminal-log aesthetics for ordinary users.

### 8.2 Layout concept

**[CONFIRMED — ROUND 3]** Use a coherent hybrid:

- card-led Home, onboarding, readiness milestones, and compact status
  summaries;
- list/table + master/detail views for high-volume Agent/Provider inventory,
  bindings, Activity, and evidence;
- a returning-user speed path that bypasses completed onboarding;
- the same domain objects, source labels, and status semantics in both
  presentations.

The original operational desktop structure remains a candidate for those
high-volume views:

```text
┌──────────────┬─────────────────────────────────────────────────────┐
│ Global nav   │ Page title / scope / primary action                 │
│ 216–240 px   ├────────────────┬────────────────────────────────────┤
│              │ Master list    │ Detail / timeline / evidence       │
│              │ 340–420 px     │ flexible                           │
└──────────────┴────────────────┴────────────────────────────────────┘
```

- Cards may lead Home and onboarding, but should not become the only
  orientation or navigation model.
- Master/detail preserves filters, selected Task, and scroll position.
- Evidence uses a tab, inspector, or expandable rail depending on width; avoid
  four simultaneously competing columns.
- At medium widths, list and detail become sibling routes or a two-pane view.
- Mobile is initially a monitoring/decision companion: priority stack → detail
  route → sticky safe action. It is not a compressed desktop administration
  surface.

### 8.3 Density and spacing

Candidate starting principles for the confirmed hybrid:

- system UI font with optical sizing;
- body text around 14–15 px on desktop, compact but not below accessible
  legibility;
- clear weight hierarchy before large size changes;
- an 8 px macro spacing rhythm with 4 px internal increments;
- row heights around 40–48 px for dense desktop lists;
- 44 px minimum touch targets where touch is expected;
- bounded content widths for prose, full-width space for operational tables;
- thin separators and surface contrast instead of card-on-card nesting.

These are design hypotheses, not implementation tokens. Spacious cards do not
require excessive empty space, low information clarity, or card-only
presentation for repeated operational tasks.

### 8.4 Color and status

- Use neutral surfaces for structure.
- Reserve semantic colors for state and risk.
- Pair every color with text/icon/shape.
- Distinguish authority state from observation source through labels and
  structure, not a rainbow timeline.
- Green means independently accepted or healthy within a named scope, not
  merely “process exited.”
- Amber means attention/partial/stale; red means failed/denied/critical risk;
  gray means unknown/unavailable, not healthy.

### 8.5 Motion

- Motion provides orientation and causality, not decoration.
- Use short, critically damped transitions by default.
- List/detail selection and inspectors should feel spatially anchored.
- Every transition is interruptible where direct manipulation exists.
- Reduced-motion mode keeps orientation through instant state changes or
  cross-fades.
- Long-running progress should update from durable facts, never fake movement.

## 9. Recommended Enterprise governance-plane form

Status: **Rounds 4–5 direction and boundaries confirmed**.

### 9.1 Primary Enterprise surfaces

| Surface | Job |
|---|---|
| Desktop Fleet | Primary Enterprise operator UI for governed work, continuation, fleet status, decisions, and evidence drilldown |
| Agent Registry | Discover, classify, sponsor, approve, suspend, and retire logical Agents and versions |
| Governance Work | Carry work toward independently accepted completion or an honest durable terminal disposition |
| Continuation | Build and transfer portable Continuation Packages across supported Agent tools or use official native-session continuation |
| Identity & Scope | Show human, Agent, workload, sponsor, delegation, and organization references |
| Policy | Author, review, version, sign, distribute, and explain policy bundles |
| Entitlements & Cost | Project Provider contracts, pools, allocation, usage, and external invoices |
| Knowledge Governance | Enroll and govern managed-index sources, copied content/embeddings, authorization, provenance, retention, revocation, and retrieval evidence |
| Approvals | Risk-ranked, separation-of-duty decision queue |
| Incidents | Link deviations, revocations, quarantine, evidence, and external SIEM/case systems |
| Fleet | Project node health, policy version, revocation watermark, evidence backlog, and attestation |
| Integrations | Configure references/connectors to existing enterprise SoRs |

### 9.2 Enterprise interaction thesis

Enterprise should use a **governance queue + scoped registry + evidence
drilldown** in its desktop fleet UI, not a remote operations console that
directly edits node state.

Central actions should read as requests:

- request suspend;
- distribute policy version;
- revoke capability;
- require approval;
- reserve allocation;
- request evidence export.

The node response should show accepted, rejected, pending, expired, or
unreachable with reason and evidence.

### 9.3 Execution assurance and continuation

**[CONFIRMED — ROUND 4]** The initial wedge builds on execution assurance and
governed work, additionally supporting conversation/Task/context continuation
within the same Agent tool or across different Agent tools, with the owner's
stated objective of carrying work through to completion.

“Complete context migration” cannot honestly mean every private or hidden
state. It cannot reliably include hidden model chain-of-thought,
Provider-private state, credentials, unsupported native sessions, or content
the user lacks authority to transfer.

**[OWNER DECISION — ROUND 5]** Define a portable **Continuation Package**
containing:

- `TaskContract` or objective and acceptance;
- decisions and owner instructions;
- approved transcript excerpts or summaries;
- `ContextView` and authorized source references;
- artifacts and their provenance;
- Effects and independent evidence;
- Provider/model binding and budget state without secret material;
- blockers, completed/remaining work, and the durable next action.

Use native session continuation only when a Provider officially supports it and
the user is authorized to continue that session. Portability must not treat an
Agent's self-report as completion or weaken independent acceptance.

### 9.4 Honest completion semantics

“Ensure completion” cannot promise successful acceptance when resources are
unavailable, authority is denied, budget or deadline is exhausted, or
acceptance is unsatisfiable.

**[OWNER DECISION — ROUND 5]** Offer a qualified completion guarantee only for
qualified Task classes while declared authority/resource/budget/deadline
preconditions remain satisfied. Otherwise use terminal accountability: drive
work, within explicit retry/reassignment/budget/time bounds, to either:

1. independently accepted completion; or
2. an honest durable `blocked`/`failed` disposition with evidence, owner, and
   one next action.

The exact transfer trigger and retry/reassignment bounds remain Round 5
decisions.

### 9.5 Managed central Knowledge index

**[CONFIRMED — ROUND 4; HIGH-IMPACT BOUNDARY]** Enterprise uses a managed
central Knowledge index that copies content and embeddings and authorizes
retrieval. This does not make CognitiveOS the source system of record.

The index requires, before implementation:

- explicit source enrollment/opt-in and authority to copy;
- authorization before indexing, search, or body exposure;
- classification, residency, retention, legal-hold, deletion, and purge rules;
- revocation propagation and measurable ACL freshness;
- tenant partitioning and encrypted transport/storage;
- source/content/chunk/embedding provenance;
- prompt-injection and retrieval-poisoning controls;
- bounded indexing scope and auditable retrieval/use evidence.

The enrollment, content, and retention boundary remains a Round 5 decision.

### 9.6 Enterprise priority order

**[CONFIRMED — ROUND 4]** Execution assurance, governed work, and continuation
are first priority. Provider/subscription management is second priority.
Round 5 must decide whether second priority means P1 after the first release or
part of the same initial release.

### 9.7 Enterprise entry boundary

After Round 5, detailed Enterprise discovery documents may be generated.
Do not promote them into implementation authority, final validated screens, or
accepted architecture contracts until:

- at least three target organizations validate governance jobs;
- a design partner supplies a real IAM/HRIS/SIEM/knowledge topology;
- the system-of-record matrix is accepted;
- revocation and offline-authority SLOs are decided;
- policy and Knowledge authorization pass representative tests.

## 10. Explicit non-goals

- Chat as the primary navigation or Task authority.
- Employee avatars, salaries, promotions, company simulation, HR workflows, or
  other employee/company metaphors in Personal; Round 2 confirmed literal
  domain language.
- Agent self-reported completion.
- Process exit as outcome verification.
- Claiming portable continuation includes hidden model chain-of-thought,
  Provider-private state, credentials, unsupported native sessions, or content
  the user lacks authority to transfer.
- Promising successful completion when authority, resources, budget, deadline,
  or acceptance feasibility prevents it.
- Universal Agent, Subscription, Resource, Run, or KnowledgeBase records.
- Consumer credential brokerage, cookie scraping, password storage, or token
  copying into ordinary configuration.
- Unsupported consumer-plan or subscription mutation.
- Full project-management, OKR, HRIS, IAM, SIEM, DLP, FinOps, billing, or DMS
  replacement.
- Central direct writes to node SQLite.
- Treating the managed central Knowledge index as the source system of record,
  or indexing/searching/exposing content without current authorization.
- Multi-Agent autonomous delegation in the Personal MVP.
- Treating Electron, a Tauri-like shell, or another shell technology as
  accepted before an ADR-quality comparison spike applies equivalent security
  and packaging criteria.
- Visual polish that invents unavailable backend capability.

## 11. Key tradeoffs

| Tradeoff | Recommended bias | Cost |
|---|---|---|
| Work-first vs chat-first | Work-first; chat may clarify intent | Less familiar to casual AI users |
| Local-first vs cloud-first | Local-first Personal | Remote/mobile access is narrower |
| Full domain model vs projection-first | Projection-first for Run/Goal; typed Assignment where authority needs it | Some later migration |
| Activation vs passive inventory | Require one verified Agent ↔ Provider/model binding | More setup work than a read-only dashboard |
| Inventory first vs governed Task first | Inventory/readiness in P0; governed Task confirmed next | Differentiated execution value arrives later |
| Native product form vs toolkit commitment | Native desktop confirmed; toolkit remains open | Architecture and platform estimates remain provisional |
| Automation vs owner control | Exception-based autonomy within fixed authority | Preview/decision friction |
| Consumer cards vs operational comparison | Confirmed hybrid: card-led orientation plus operational comparison views | Two presentation modes require shared semantics and careful continuity |
| One product vs Personal/Enterprise split | Shared substrate, distinct products | More explicit boundary work |
| Provider management vs observation | Manage only supported APIs; otherwise link/observe | Less “all-in-one” appearance |
| Generic extensibility vs one qualified path | One path end-to-end first | Narrow initial compatibility |
| Central consistency vs node authority | Node authority with eventual projections | Central UI must represent stale/offline states |
| “Complete” migration vs portable truth | Authorized Continuation Package plus official native-session continuation where supported | Hidden/private state cannot be recreated or transferred |
| Successful completion vs accountability | Accepted completion or durable blocked/failed disposition within explicit bounds | Cannot promise success under denied or unsatisfiable conditions |
| Source-native Knowledge vs central index | Managed central index selected with explicit enrollment, authorization, tenancy, retention, and purge boundaries still open | Major security, residency, deletion, and poisoning burden |
| Desktop fleet vs Web operator UI | Desktop fleet confirmed primary; fallback/deep-admin Web relationship open | Potential dual-surface parity cost |

## 12. Interactive owner decision rounds

The decisions are ordered so later answers depend on earlier ones. Detailed
specifications should not be written as final until the relevant round is
settled.

### Round 1 — Product identity and first value — confirmed 2026-08-25

#### Decision 1: umbrella positioning

- **Status:** `confirmed-round-1`.
- **Owner selection:** “AI Workforce OS.”
- **Original recommended default — superseded:** “CognitiveOS — Governed Agent
  Work System.”
- Alternative A: “Personal Agent Operating System.”
- Consequences:
  - Governed Agent Work System is precise and testable but less aspirational.
  - Personal Agent OS is approachable but understates Enterprise and evidence.
  - AI Workforce OS is memorable but creates company/HR/replacement
    expectations.
- Affects: all product documents, navigation copy, marketing language, and
  Enterprise boundary.

#### Decision 2: Personal primary user

- **Status:** `confirmed-round-1`.
- **Owner selection:** technical individual operator using multiple coding or
  research Agents.
- **Original recommended default — confirmed:** the same selection.
- Alternative A: broad knowledge worker using general-purpose assistants.
- Alternative B: independent developer only.
- Consequences:
  - Technical operator gives realistic verification and Provider workflows.
  - Broad knowledge worker expands market but requires non-code verifiers and
    simpler setup.
  - Developer-only sharpens MVP but may trap the product in coding.
- Affects: Personal PRD, onboarding, vocabulary, first integrations, metrics,
  and validation participants.

#### Decision 3: first high-value scenario

- **Status:** `confirmed-round-1`.
- **Owner selection:** Agent/Provider inventory, entitlement, and cost
  visibility first.
- **Original recommended default — superseded:** one bounded technical Task
  assigned to one Agent and carried to independent acceptance.
- Alternative B: long-running research Task with citation/evidence acceptance.
- Consequences:
  - Assigned Task proves the full product loop but requires Assignment work.
  - Inventory is faster but risks becoming administration without outcome.
  - Research broadens beyond coding but needs a credible verification oracle.
- Affects: first vertical slice, P0 scope, acceptance, UI emphasis, and
  architecture deltas.

#### Decision 4: Personal delivery form

- **Status:** `confirmed-round-1`.
- **Owner selection:** native desktop application first.
- **Original recommended default — superseded:** local daemon + formal
  responsive Web client + CLI companion.
- Alternative B: cloud-hosted Personal service.
- Consequences:
  - Web preserves the current repository/product boundary.
  - Native packaging improves launch/notifications but adds a new delivery
    surface before product proof.
  - Cloud improves remote access but changes identity, secret, tenancy, and
    authority assumptions.
- Affects: Personal architecture, distribution, navigation/responsiveness,
  notification design, and security model.

### Round 2 — P0 product model and desktop interaction — confirmed 2026-08-25

#### Decision 5: AI Workforce OS metaphor depth

- **Status:** `confirmed-round-2`.
- **Owner selection:** use “AI Workforce OS” at vision/category level;
  Personal uses literal Agent, Provider, entitlement, binding, usage, cost, and
  Task language.
- **Original recommended default — confirmed:** the same selection.
- Alternative A: add light role/team/workforce metaphors without employee or
  HR lifecycle concepts.
- Alternative B: make digital employees and a one-person AI company the
  explicit Personal model.
- Consequences:
  - Vision-level language preserves the confirmed umbrella while minimizing
    unvalidated HR/replacement expectations.
  - Light metaphor may improve narrative coherence but creates ambiguous
    boundaries around roles and teams.
  - Explicit company/employee framing is memorable but expands object model,
    onboarding, ethics, and user-expectation scope.
- Affects future documents: Personal product design, interaction/copy,
  validation plan, Enterprise product boundary, and shared terminology.

#### Decision 6: desktop implementation and platform priority

- **Status:** `confirmed-round-2`.
- **Owner selection:** package the existing formal Web client in a native
  shell, validate Windows first, then decide macOS/Linux order from evidence.
- **Original recommended default — confirmed:** the same selection.
- Alternative A: build a fully platform-native UI for one primary desktop OS.
- Alternative B: build a new UI with a cross-platform desktop toolkit and
  target Windows/macOS parity from the first release.
- Consequences:
  - A shell maximizes current-client reuse and reaches native packaging sooner,
    but may constrain platform-native interaction quality.
  - Fully platform-native UI offers the strongest OS fit but creates a
    separate presentation implementation and narrows initial reach.
  - A cross-platform toolkit centralizes desktop UI code but adds toolkit
    runtime, accessibility, packaging, and migration choices.
- Affects future documents: Personal architecture, interaction/visual spec,
  distribution/update model, platform support policy, security review, and
  delivery readiness.

#### Decision 7: exact P0 activation boundary

- **Status:** `confirmed-round-2`.
- **Owner selection:** P0 ends after one Agent is registered, one supported
  Provider access path is linked, readiness is verified, an explicit
  Agent ↔ Provider/model binding is created, and source-typed usage/cost status
  is shown; governed Task execution is the next expansion.
- **Original recommended default — confirmed:** the same boundary.
- Alternative A: include one bounded governed Task through independent
  acceptance in P0 after activation.
- Alternative B: stop at inventory visibility without requiring verified
  readiness or an explicit binding.
- Consequences:
  - The recommended boundary produces a usable setup outcome with bounded
    scope, but delays proof of governed execution value.
  - Including a Task proves the larger thesis but substantially expands P0.
  - Inventory-only ships sooner but risks a passive-dashboard dead end.
- Affects future documents: Personal P0 requirements, activation flow,
  acceptance criteria, Work scope, metrics, architecture deltas, and delivery
  plan.

#### Decision 8: discovery and import behavior

- **Status:** `confirmed-round-2`.
- **Owner selection:** user-triggered local discovery with a review screen,
  explicit confirmation before registration/import, provenance on every fact,
  and manual registration fallback.
- **Original recommended default — confirmed:** the same behavior.
- Alternative A: manual registration only; no discovery.
- Alternative B: background discovery that continuously proposes changes for
  review but never silently creates trust, auth, or bindings.
- Consequences:
  - User-triggered discovery balances setup speed, consent, and provenance.
  - Manual-only is easiest to reason about but increases setup friction and
    stale inventory.
  - Background proposals improve freshness but add notifications, lifecycle,
    privacy, and reconciliation complexity.
- Affects future documents: onboarding, permissions, privacy/threat model,
  Agent integrations, import UX, Activity provenance, and test scenarios.

#### Decision 9: Provider and entitlement scope

- **Status:** `confirmed-round-2`.
- **Owner selection:** qualify one supported Provider API access path;
  represent consumer plan, API account, auth, `SecretRef`, entitlement, budget,
  usage, and cost separately; include consumer-plan facts only through
  supported read-only sources or explicit user declaration.
- **Original recommended default — confirmed:** the same scope.
- Alternative A: support Provider API accounts only and omit consumer-plan
  facts from P0.
- Alternative B: launch with read-only inventory across multiple Providers and
  entitlement sources before any single path is fully qualified.
- Consequences:
  - One qualified path enables honest readiness and binding while preserving
    the user's broader entitlement mental model.
  - API-only is technically cleaner but hides relevant consumer access.
  - Broad read-only coverage improves inventory breadth but increases source
    inconsistency and may postpone a usable binding.
- Affects future documents: Provider product model, integrations, auth and
  SecretStore boundaries, cost UX, source taxonomy, threat model, and P0
  acceptance.

#### Decision 10: Agent object presentation

- **Status:** `confirmed-round-2`.
- **Owner selection:** show Agent Profile and Agent Instance as distinct
  inspectable concepts, presented with tool identity, purpose, capability
  source, health, compatibility, and bindings—not personality.
- **Original recommended default — confirmed:** the same presentation.
- Alternative A: present one simplified Agent row and reveal Profile/Instance
  distinctions only in detail.
- Alternative B: present Agents as named workforce roles or digital employees
  with operational facts beneath the metaphor.
- Consequences:
  - Explicit separation is precise for technical operators but requires clear
    onboarding.
  - A unified row is easier to scan but can hide version/deployment truth.
  - Role/employee presentation reinforces the umbrella metaphor but risks
    persona theater and conflating identity with runnable instances.
- Affects future documents: Personal object model, Agents IA, terminology,
  onboarding, visual identity, accessibility labels, and validation scenarios.

#### Decision 11: visual and interaction density

- **Status:** `confirmed-round-2`.
- **Owner selection — original Alternative A:** spacious consumer-style cards
  and a wizard-led experience.
- **Original recommended default — superseded as the leading direction:** calm,
  dense desktop operations using compact tables
  or lists, master/detail inspection, progressive disclosure, and a guided
  first-run activation path.
- Alternative B: terminal/IDE-inspired expert density with logs and commands
  prominent.
- Consequences:
  - Calm density supports repeated comparison while guided activation protects
    first-run usability.
  - Consumer cards are approachable but reduce scan speed and comparison
    capacity.
  - Expert density is efficient for developers but narrows the audience and
    can overexpose implementation detail.
- Affects future documents: interaction/visual spec, navigation, component
  system, accessibility, usability tests, and responsive/secondary-screen
  policy.

- Round 3 resolution: card-led Home/onboarding/status summaries coexist with
  list/table + master/detail inventory and evidence views.

### Round 3 — Personal product model and operating UX — confirmed 2026-08-25

#### Decision 12: IA, navigation, and default landing surface

- **Status:** `confirmed-round-3`.
- **Owner selection:** persistent sidebar with Home, Agents, Providers,
  Work, Activity, and System; first-run opens the resumable activation wizard,
  while activated returning users land on card-led Home.
- **Original recommended default — confirmed:** the same selection.
- Alternative A: land every session on Readiness until the user manually
  changes the default.
- Alternative B: restore the last-used area and make Home optional.
- Consequences:
  - Card-led Home supports orientation and status summaries while keeping
    product areas explicit.
  - Readiness-first keeps setup gaps visible but becomes repetitive after
    activation.
  - Last-used restoration speeds repeated work but weakens a consistent
    attention and status entry point.
- Affects future documents: Personal IA, navigation and route model, onboarding,
  desktop shell behavior, deep-link/state restoration, and usability tests.

#### Decision 13: card-led versus operational view split

- **Status:** `confirmed-round-3`.
- **Owner selection:** use spacious cards for Home, onboarding, readiness
  milestones, and compact summaries; use list/table + master/detail views for
  high-volume inventory, bindings, Activity, and evidence.
- **Original recommended default — confirmed:** the same hybrid.
- Alternative A: use cards throughout P0 and revisit operational views only
  after measured failure.
- Alternative B: use cards only during onboarding; switch the entire activated
  product to dense operational views.
- Consequences:
  - The hybrid preserves the confirmed approachable direction without turning
    comparison-heavy work into an oversized-card grid.
  - Card-everywhere maximizes visual consistency but reduces scan speed and
    evidence comparison as volume grows.
  - Onboarding-only cards optimize repeated work but create a stronger visual
    mode change after activation.
- Affects future documents: interaction/visual spec, component system,
  inventory layouts, evidence UX, responsive behavior, accessibility, and
  scenario tests.

#### Decision 14: post-P0 Goal, Workstream, and Task hierarchy

- **Status:** `confirmed-round-3`.
- **Owner selection:** Task is the first authority object; Goal remains a
  lightweight outcome/reference and Workstream is deferred until repeated
  cross-Task coordination is validated.
- **Original recommended default — confirmed:** the same hierarchy.
- Alternative A: introduce first-class Goal → Workstream → Task hierarchy with
  the first governed Task slice.
- Alternative B: remain Task-only with optional external links and no Goal
  object.
- Consequences:
  - Goal-lite preserves intent without making the next slice a project
    management product.
  - Full hierarchy improves planning context but expands lifecycle, IA,
    storage, and migration scope.
  - Task-only is smallest but weakens outcome grouping and future portfolio
    context.
- Affects future documents: Personal product model, Work IA, Task contracts,
  storage/API deltas, migration plan, and Enterprise integration boundary.

#### Decision 15: assignment and autonomy default for the next vertical slice

- **Status:** `confirmed-round-3`.
- **Owner selection:** the owner explicitly assigns one ready
  Agent/Profile/Instance binding; the system previews scope, Provider/model,
  resources, budget, and acceptance, then executes within admitted authority
  and interrupts on exception.
- **Original recommended default — confirmed:** the same autonomy default.
- Alternative A: system recommends an eligible binding, but the owner confirms
  every assignment and consequential action.
- Alternative B: system automatically selects among eligible bindings and acts
  within a standing budget after one initial authorization.
- Consequences:
  - Explicit assignment plus exception-based supervision balances control and
    sustained execution.
  - Per-action confirmation reduces perceived surprise but creates approval
    fatigue.
  - Automatic selection improves throughput but requires stronger selection
    policy, explanation, revocation, and blast-radius controls.
- Affects future documents: governed Task slice, Assignment model, preview,
  policy, decision cards, notifications, Activity, budgets, and acceptance.

#### Decision 16: Activity and evidence explanation depth

- **Status:** `confirmed-round-3`.
- **Owner selection:** three layers—card summary, human-readable detail,
  and audit-depth source/event evidence—with source type, freshness, authority
  versus observation, and missing facts visible.
- **Original recommended default — confirmed:** the same three layers.
- Alternative A: two layers—simple summary and downloadable technical report.
- Alternative B: one expert timeline showing all events and evidence by
  default.
- Consequences:
  - Three layers support both approachable entry and technical verification
    without hiding audit depth.
  - Two layers simplify UI but make the jump from summary to raw evidence
    abrupt.
  - One expert timeline maximizes immediacy but conflicts with the confirmed
    consumer-led presentation and increases cognitive load.
- Affects future documents: Activity IA, evidence UX, source taxonomy,
  disclosure components, export, accessibility, and verification tests.

#### Decision 17: desktop shell technology and security posture

- **Status:** `confirmed-round-3`.
- **Owner selection — original Alternative B:** defer framework selection until
  equivalent security and packaging spikes compare at least two candidates.
- **Original recommended default — superseded as a selection:** prefer a
  Tauri-like Rust-native shell if a focused
  spike proves Web-client compatibility, Windows packaging/update signing,
  accessibility, process isolation, narrow IPC allowlisting, SecretStore use,
  and no broad filesystem/network bridge.
- Alternative A: Electron with hardened context isolation, sandboxing, narrow
  preload/IPC, signed updates, and explicit runtime provenance.
- Consequences:
  - A Tauri-like shell may align with the Rust daemon and reduce bundled
    runtime footprint, but compatibility and Windows WebView behavior need
    evidence.
  - Electron has mature packaging and predictable rendering but increases
    runtime footprint and requires rigorous browser-process hardening.
  - A comparison spike reduces premature commitment but delays implementation
    estimates.
- Affects future documents: Personal architecture, threat model, IPC contract,
  packaging/update design, platform qualification, dependency policy, and
  delivery readiness.

#### Decision 18: notifications and system-tray behavior

- **Status:** `confirmed-round-3`.
- **Owner selection:** optional system-tray presence after explicit
  opt-in; notify only for activation completion/failure, stale or lost
  readiness that needs action, and future Task decisions/failures; every
  notification deep-links to a durable state.
- **Original recommended default — confirmed:** the same behavior.
- Alternative A: no tray or OS notifications in P0; show status only when the
  app is open.
- Alternative B: tray starts by default and reports routine readiness, usage,
  cost, and Agent changes.
- Consequences:
  - Opt-in actionable notifications support desktop value without creating
    ambient noise or hidden background behavior.
  - No notifications simplify lifecycle and permissions but weaken recovery
    and background presence.
  - Default-on broad notifications maximize visibility but risk fatigue,
    privacy surprises, and unclear process lifecycle.
- Affects future documents: desktop lifecycle, onboarding/permissions,
  notification taxonomy, tray commands, background process behavior, privacy,
  accessibility, and Windows validation.

### Round 4 — Enterprise product and boundary — confirmed 2026-08-25

Two facts are not owner options: the node/workspace daemon remains the sole
authority writer, and the central plane never writes remote node SQLite
directly. All deployment, federation, registry, policy, evidence, knowledge,
and UI alternatives below must preserve those boundaries.

#### Decision 19: Enterprise initial wedge

- **Status:** `confirmed-round-4`.
- **Owner selection:** execution assurance and governed work—connect intent,
  assignment, admitted authority, node execution, independent verification,
  evidence, and accountability across existing systems—plus complete
  conversation/Task/context migration within or across Agent tools to carry
  work through to completion. Provider/subscription management is second
  priority.
- **Original recommended default — extended by owner:** execution assurance and
  governed work without the additional migration objective.
- Alternative A: Agent registry/control tower first—inventory, sponsorship,
  status, compatibility, and fleet visibility before governed work.
- Alternative B: entitlement and cost governance first—Provider contracts,
  pools, allocation, usage, and cost projections.
- Consequences:
  - Execution assurance differentiates through the full authority/evidence
    loop but requires a qualified end-to-end organizational workflow.
  - Registry/control tower has a simpler observational entry but risks becoming
    inventory without outcome assurance.
  - Entitlement/cost governance has a clear economic buyer but competes with
    FinOps/IAM products and may underuse CognitiveOS execution strengths.
  - The migration objective requires a portable, authorized continuation unit;
    hidden/private Provider state and unauthorized content are not portable.
  - Completion must mean accepted completion or an honest durable terminal
    disposition, not guaranteed success under impossible constraints.
- Affects future documents: Enterprise product design, first user/buyer, wedge
  validation, IA, integration priorities, success metrics, and delivery
  readiness.
- Round 5 resolution: portable package, bounded same-binding retry,
  owner-confirmed cross-tool transfer, qualified guarantee with terminal
  fallback, and same-release Provider/subscription track.

#### Decision 20: Enterprise product form and deployment

- **Status:** `confirmed-round-4`.
- **Owner selection:** central Web governance plane and integration service
  paired with customer/node authority daemons; central actions are signed,
  versioned requests and projections, never direct node-state writes.
- **Original recommended default — confirmed:** the same deployment split.
- Alternative A: self-hosted-only central plane plus customer/node daemons.
- Alternative B: SaaS-heavy central service that also coordinates most
  execution, while node daemons retain sole local authority writes.
- Consequences:
  - The recommended split supports cross-node governance while preserving local
    authority, but requires offline/stale-state and trust-boundary design.
  - Self-hosted-only simplifies data-residency positioning but increases
    customer operations and slows service iteration.
  - SaaS-heavy coordination improves managed operations but expands tenancy,
    availability, data-transfer, and central-compromise impact.
- Affects future documents: Enterprise architecture, deployment topology,
  tenancy, identity, synchronization, SLOs, threat model, and operations.

#### Decision 21: systems-of-record and federation posture

- **Status:** `confirmed-round-4`.
- **Owner selection:** federate IAM, HRIS, Secret Store, SIEM, project/work
  management, and knowledge systems through stable external references and
  scoped connectors; CognitiveOS owns execution-specific facts only.
- **Original recommended default — confirmed:** the same federation posture.
- Alternative A: copy selected external identities, organization, work, secret
  metadata, incidents, and knowledge metadata into CognitiveOS as canonical
  enterprise records.
- Alternative B: avoid formal federation in the first Enterprise wedge and use
  manual references/imports only.
- Consequences:
  - Federation preserves source ownership and freshness semantics but requires
    connector contracts, reconciliation, and unavailable/stale states.
  - Canonical copies simplify local queries but create duplication, deletion,
    residency, and conflict obligations.
  - Manual references reduce integration scope but weaken authorization,
    automation, audit continuity, and adoption.
- Affects future documents: system-of-record matrix, connector model, identity,
  secret isolation, incident links, work/knowledge references, retention, and
  integration validation.

#### Decision 22: central evidence and data-retention depth

- **Status:** `confirmed-round-4`.
- **Owner selection:** retain minimized signed projections, receipts,
  digests, source references, policy/decision facts, and bounded summaries
  centrally; keep raw logs and artifacts at customer/node or source systems.
- **Original recommended default — confirmed:** the same minimized retention.
- Alternative A: copy bounded normalized evidence reports and selected
  artifacts centrally under explicit retention/classification policy.
- Alternative B: centralize broad execution logs, traces, and artifacts for
  search and investigation.
- Consequences:
  - Minimized projections reduce breach, residency, and deletion scope but make
    deep investigations depend on source availability.
  - Bounded copies improve investigation continuity but require classification,
    retention, encryption, deletion, and access-control policy.
  - Broad centralization maximizes search but creates the largest privacy,
    secret-leak, storage, legal-hold, and blast-radius burden.
- Affects future documents: evidence architecture, data classification,
  retention/deletion, export, incident response, encryption, residency, and
  verification UX.

#### Decision 23: Agent registry ownership

- **Status:** `confirmed-round-4`.
- **Owner selection:** federate external Agent/workload registries and add a
  CognitiveOS execution overlay for sponsor, version, capability source,
  eligibility, bindings, policy, node presence, and evidence.
- **Original recommended default — confirmed:** the same registry overlay.
- Alternative A: make CognitiveOS the primary native Agent registry and
  lifecycle authority, exporting to external systems where needed.
- Alternative B: keep registries node-local and provide only aggregate central
  fleet projections.
- Consequences:
  - A federated overlay respects existing ownership while adding execution
    semantics, but identity matching and freshness are harder.
  - A native primary registry gives coherent lifecycle control but duplicates
    IAM/CMDB/platform registries and increases migration scope.
  - Node-local registry minimizes central identity scope but weakens global
    sponsorship, policy targeting, and cross-node traceability.
- Affects future documents: Agent domain model, registry connectors,
  sponsorship, identity mapping, fleet IA, policy targeting, and migration.

#### Decision 24: policy engine posture

- **Status:** `confirmed-round-4`.
- **Owner selection:** stabilize a versioned policy-decision contract and
  evidence format; keep the engine pluggable, with the built-in Personal
  evaluator as the initial implementation rather than selecting OPA or Cedar
  now.
- **Original recommended default — confirmed:** the same contract-first posture.
- Alternative A: standardize on OPA/Rego now as the Enterprise policy engine.
- Alternative B: standardize on Cedar now as the Enterprise policy engine.
- Consequences:
  - A stable pluggable contract preserves semantics and postpones engine lock-in
    until representative policy tests exist.
  - OPA offers broad ecosystem and flexible policy but adds Rego operations,
    distribution, and explanation design.
  - Cedar offers a focused authorization model and analyzability but may not
    express all execution/governance policies without adjacent mechanisms.
- Affects future documents: policy ADR, decision API, bundle/version model,
  explanation UX, distribution/cache, conformance, and migration strategy.

#### Decision 25: Enterprise knowledge retrieval posture

- **Status:** `confirmed-round-4`.
- **Owner selection — original Alternative A:** build a managed central
  Knowledge index with copied content and embeddings and CognitiveOS-authorized
  retrieval.
- **Original recommended default — superseded:** preserve source-native
  authorization and retrieval;
  CognitiveOS stores scoped references, classification, purpose, ACL freshness,
  and usage evidence rather than a universal central index.
- Alternative B: hybrid—centralize approved metadata and selected derived
  indexes while fetching protected content from source at use time.
- Consequences:
  - Source-native retrieval preserves authoritative ACLs and residency but
    depends on source uptime, latency, and connector correctness.
  - A managed index improves search consistency but creates duplication,
    deletion, poisoning, ACL synchronization, and residency obligations.
  - Hybrid indexing can improve discovery while limiting copied content, but
    split authorization and freshness become more complex.
- Affects future documents: knowledge architecture, connector contracts,
  authorization, indexing, retention, provenance, retrieval evidence, and
  threat model.
- Round 5 resolution: source opt-in, pre-index authorization, approved content,
  bounded retention/residency, tenant encryption/partition, provenance, ACL
  freshness, revocation/deletion, and verified purge.

#### Decision 26: Enterprise UI interaction form

- **Status:** `confirmed-round-4`.
- **Owner selection — original Alternative A:** desktop fleet application as
  the primary Enterprise surface.
- **Original recommended default — superseded as primary UI:** Web
  admin/operations control plane using governance
  queues, scoped registries, fleet/status projections, and evidence drilldown.
- Alternative B: embedded experiences inside existing ITSM/SIEM/PM portals,
  with CognitiveOS Web UI limited to deep administration.
- Consequences:
  - A Web plane fits multi-user administration and broad deployment but
    requires strong tenancy, session, permission, and stale-state UX.
  - A desktop fleet app may improve operator integration but complicates
    distribution, updates, multi-user access, and remote administration.
  - Embedded experiences reduce context switching but fragment interaction,
    depend on host capabilities, and need a durable deep-link/admin fallback.
- Affects future documents: Enterprise IA, interaction/visual spec, session and
  permission model, integrations, responsive behavior, accessibility, and
  usability validation.
- Round 5 resolution: Web UI remains a supported fallback/deep-admin surface
  sharing backend, permissions, and durable routes with Desktop Fleet.

### Round 5 — final product and boundary clarifications — confirmed 2026-08-25

#### Decision 27: exact migration unit

- **Status:** `confirmed-round-5`.
- **Owner selection:** a portable, versioned Continuation Package containing
  the Task contract/objective, acceptance, decisions, approved transcript
  excerpts or summaries, authorized ContextView/source refs, artifacts,
  Effects/evidence, non-secret binding and budget state, blockers, and durable
  next action.
- **Original recommended default — confirmed:** the same package.
- Alternative A: raw/full transcript and session export wherever a Provider or
  Agent tool exposes it, with redaction and authority checks.
- Alternative B: Provider-native session continuation only; no general
  cross-tool package.
- Consequences:
  - A typed package is portable, reviewable, and authority-bounded but cannot
    recreate hidden/private model state exactly.
  - Raw export preserves more visible conversation but increases secret,
    irrelevant-data, retention, format, and authorization risk.
  - Native continuation preserves tool-specific state where supported but does
    not satisfy broad cross-tool portability.
- Affects future documents: continuation product model, package contract,
  import/export, redaction, provenance, compatibility matrix, security, and
  conformance tests.

#### Decision 28: transfer trigger and control

- **Status:** `confirmed-round-5`.
- **Owner selection:** allow bounded automatic retry on the same approved
  binding; require owner confirmation before cross-tool transfer or
  reassignment, with package preview and consequence summary.
- **Original recommended default — confirmed:** the same control split.
- Alternative A: require owner confirmation before every retry, transfer, and
  reassignment.
- Alternative B: automatically retry and reassign across eligible tools within
  explicit budget/time/attempt/policy bounds, notifying on exception.
- Consequences:
  - The recommended split avoids unnecessary retry friction while keeping
    context/authority transfer explicit.
  - Full confirmation maximizes control but creates approval fatigue and slows
    recovery.
  - Bounded automation improves continuity but requires strong eligibility,
    leakage prevention, loop detection, revocation, and audit controls.
- Affects future documents: continuation flow, retry/reassignment policy,
  preview/approval UX, budgets, notifications, audit events, and failure tests.

#### Decision 29: honest completion guarantee

- **Status:** `confirmed-round-5`.
- **Owner selection — original Alternative A:** promise successful completion
  only for qualified Task classes while declared
  authority/resource/budget/deadline preconditions remain satisfied; use
  terminal accountability otherwise.
- **Original recommended default — superseded as the universal default:**
  terminal accountability—within explicit bounds,
  reach independently accepted completion or a durable `blocked`/`failed`
  disposition with evidence, owner, and next action.
- Alternative B: make no completion guarantee; expose best-effort Agent
  attempts and current status only.
- Consequences:
  - Terminal accountability is honest across unavoidable blockers while still
    preventing abandoned work.
  - A narrow success guarantee may be testable but requires strict admission,
    SLOs, exclusions, and remediation terms.
  - Best effort is easy to claim but weakens the product's core execution
    assurance differentiation.
- Affects future documents: Enterprise promise, Task terminal semantics,
  admission, retry/reassignment bounds, SLOs, evidence, recovery UX, and
  validation.

#### Decision 30: managed Knowledge index enrollment and retention

- **Status:** `confirmed-round-5`.
- **Owner selection:** per-source opt-in; authorize before indexing; copy
  only approved content/classes; configure residency and bounded retention;
  enforce tenant partitioning, encryption, provenance, ACL freshness,
  revocation, deletion, and verified purge.
- **Original recommended default — confirmed:** the same boundary.
- Alternative A: organization-wide default enrollment with administrator
  exclusions and policy-driven retention.
- Alternative B: initially index metadata and embeddings only, fetching source
  bodies at retrieval time despite the Round 4 central-index direction.
- Consequences:
  - Per-source opt-in minimizes unauthorized copying and makes retention
    accountable, but increases setup and coverage gaps.
  - Default enrollment improves coverage but creates the highest consent,
    residency, deletion, and overcollection risk.
  - Metadata/embedding-only indexing reduces copied body content but weakens
    offline retrieval and complicates source-time authorization.
- Affects future documents: Knowledge product model, ingestion authorization,
  classification/residency, retention/purge, tenancy, encryption, provenance,
  ACL SLOs, and threat tests.

#### Decision 31: desktop fleet and Web UI relationship

- **Status:** `confirmed-round-5`.
- **Owner selection:** desktop fleet is the primary operator experience;
  Web UI remains a supported fallback and deep-admin surface over the same
  governance backend, permissions, and durable routes.
- **Original recommended default — confirmed:** the same relationship.
- Alternative A: desktop fleet is the only product UI; the central Web plane
  exposes APIs/services but no operator interface.
- Alternative B: Web UI remains the primary admin surface; desktop is a
  specialized fleet/notification companion.
- Consequences:
  - A supported Web fallback improves remote/admin access but creates two
    surfaces requiring semantic and authorization parity.
  - Desktop-only sharpens product focus but weakens browser-based operations,
    support, and multi-user administration.
  - Web-primary reduces desktop scope but contradicts the selected primary UI
    and may underuse native fleet integration.
- Affects future documents: Enterprise IA, surface ownership, route/deep-link
  model, session/permission parity, distribution, accessibility, and UI tests.

#### Decision 32: conversation and context storage locality

- **Status:** `confirmed-round-5`.
- **Owner selection:** node/local storage is canonical for conversations
  and Continuation Packages; optional encrypted, tenant-scoped cloud sync is
  explicitly enabled by policy and user authority.
- **Original recommended default — confirmed:** the same locality.
- Alternative A: central cloud storage is canonical, with node caches for
  offline execution.
- Alternative B: local/node-only storage with no cloud synchronization.
- Consequences:
  - Local canonical plus opt-in sync balances portability and residency but
    requires conflict, key, deletion, and partial-sync semantics.
  - Cloud canonical simplifies cross-device/tool access but expands central
    breach, tenancy, retention, and availability impact.
  - Local-only minimizes central exposure but limits cross-node continuation,
    recovery, and fleet visibility.
- Affects future documents: data architecture, sync protocol, encryption/key
  ownership, tenancy, offline behavior, retention/deletion, backup, and threat
  model.

#### Decision 33: detailed visual and brand relationship

- **Status:** `confirmed-round-5`.
- **Owner selection:** one restrained CognitiveOS brand and semantic state
  system; Personal leads with spacious consumer cards plus confirmed hybrid
  operational views, while Enterprise desktop fleet uses greater operational
  density for queues, fleet, Activity, and evidence.
- **Original recommended default — confirmed:** the same visual relationship.
- Alternative A: use the same spacious card language and density across
  Personal and Enterprise.
- Alternative B: create visibly separate Personal and Enterprise brands and
  component systems.
- Consequences:
  - A shared brand with density adaptation preserves recognition while fitting
    different task frequency and scale.
  - One consumer density maximizes consistency but can degrade Enterprise scan
    speed and comparison.
  - Separate brands permit stronger audience fit but increase design-system,
    learning, documentation, and maintenance cost.
- Affects future documents: Personal and Enterprise visual specs, tokens,
  typography, color/status, component variants, motion, accessibility, and
  rendered usability validation.

#### Decision 34: Provider/subscription second-priority release boundary

- **Status:** `confirmed-round-5`.
- **Owner selection — original Alternative A:** include Provider/subscription
  management in the same first Enterprise release as execution assurance and
  migration, with separate acceptance and capability honesty.
- **Original recommended default — superseded for release timing:**
  Provider/subscription management is Enterprise P1
  after the execution-assurance/continuation first release, with separate
  acceptance and release criteria.
- Alternative B: keep it as read-only discovery/reference in Enterprise and
  defer management indefinitely.
- Consequences:
  - P1 sequencing protects the wedge while preserving the owner's stated second
    priority.
  - One release creates a broader buyer story but substantially expands
    integrations, security, billing semantics, and delivery risk.
  - Read-only deferral minimizes scope but may fail economic-governance demand.
- Affects future documents: Enterprise scope/priorities, release plan,
  Provider integrations, entitlement/cost model, acceptance, metrics, and
  validation.

## 13. Decision log — 2026-08-25

The owner supplied all Round 1–5 selections below. Only Decision 19 included
custom intent text; no other written rationale was supplied. Consequences and
follow-ups are the brief author's analysis unless explicitly attributed.

### Decision log 1 — umbrella positioning

- Status: `confirmed-round-1`
- Owner answer: AI Workforce OS.
- Original recommendation: Governed Agent Work System.
- Supersession: original recommendation superseded for umbrella positioning.
- Rationale: not supplied by the owner.
- Open follow-up: none; Round 2 confirmed vision/category language only.
- Affected future documents: all product documents, navigation/copy, market
  language, validation, and Enterprise boundary.
- Recorded on: 2026-08-25.

### Decision log 2 — Personal primary user

- Status: `confirmed-round-1`
- Owner answer: technical individual operator using multiple coding or
  research Agents.
- Original recommendation: the same primary user.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Open follow-up: validate the narrower behavioral and integration assumptions.
- Affected future documents: Personal product design, onboarding,
  integrations, vocabulary, metrics, and research plan.
- Recorded on: 2026-08-25.

### Decision 3 — first value path

- Status: `confirmed-round-1`
- Owner answer: Agent/Provider inventory, entitlement, and cost visibility
  first.
- Original recommendation: one bounded technical Task assigned to one Agent
  and carried to independent acceptance.
- Supersession: original recommendation superseded for P0 sequencing; retained
  as the confirmed next expansion after P0 activation.
- Rationale: not supplied by the owner.
- Open follow-up: none for P0 sequencing; Round 2 confirmed both boundaries.
- Affected future documents: Personal P0, activation, IA, acceptance, metrics,
  Provider model, and architecture deltas.
- Recorded on: 2026-08-25.

### Decision 4 — first product form

- Status: `confirmed-round-1`
- Owner answer: native desktop application first.
- Original recommendation: local daemon + responsive Web client + CLI
  companion.
- Supersession: original recommendation superseded as product form, not as an
  implementation-reuse option.
- Rationale: not supplied by the owner.
- Open follow-up: an ADR-quality comparison spike must evaluate at least two
  shell candidates; Round 2 confirmed Web-client reuse and Windows-first
  validation.
- Affected future documents: Personal architecture, interaction/visual spec,
  distribution, updates, platform support, and security model.
- Recorded on: 2026-08-25.

### Decision log 5 — AI Workforce OS metaphor depth

- Status: `confirmed-round-2`
- Owner answer: vision/category language only; Personal uses literal domain
  terms and no employee/company metaphor.
- Original recommendation: the same selection.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: Personal product/copy, validation, Enterprise
  boundary, and shared terminology.
- Recorded on: 2026-08-25.

### Decision log 6 — desktop implementation and platform priority

- Status: `confirmed-round-2`
- Owner answer: native shell around the existing Web client; validate Windows
  first, then decide macOS/Linux from evidence.
- Original recommendation: the same selection.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Open follow-up: compare at least two shell candidates through equivalent
  security/packaging spikes before an ADR selects one.
- Affected future documents: Personal architecture, distribution, platform
  support, security, and delivery readiness.
- Recorded on: 2026-08-25.

### Decision log 7 — exact P0 activation boundary

- Status: `confirmed-round-2`
- Owner answer: register one Agent, link one supported Provider access path,
  verify readiness, create an explicit Agent ↔ Provider/model binding, and show
  source-typed usage/cost status; governed Task execution follows P0.
- Original recommendation: the same boundary.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: P0 requirements, activation, acceptance, metrics,
  Work scope, architecture deltas, and delivery plan.
- Recorded on: 2026-08-25.

### Decision log 8 — discovery and import behavior

- Status: `confirmed-round-2`
- Owner answer: user-triggered discovery, review before import, provenance per
  fact, and manual fallback.
- Original recommendation: the same behavior.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: onboarding, permissions, privacy/threat model,
  Agent integrations, Activity, and tests.
- Recorded on: 2026-08-25.

### Decision log 9 — Provider and entitlement scope

- Status: `confirmed-round-2`
- Owner answer: one qualified Provider API access path plus supported read-only
  or user-declared consumer-plan facts, with plan/account/auth/`SecretRef`/
  entitlement/budget/usage/cost kept separate.
- Original recommendation: the same scope.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: Provider model, integrations, auth/SecretStore,
  cost UX, source taxonomy, threat model, and P0 acceptance.
- Recorded on: 2026-08-25.

### Decision log 10 — Agent object presentation

- Status: `confirmed-round-2`
- Owner answer: distinct Agent Profile and Agent Instance presentation with
  identity, purpose, capability source, health, compatibility, and bindings.
- Original recommendation: the same presentation.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: Personal object model, Agents IA, onboarding,
  terminology, visual identity, accessibility, and validation.
- Recorded on: 2026-08-25.

### Decision log 11 — visual and interaction density

- Status: `confirmed-round-2`
- Owner answer: consumer-style spacious cards and wizard-led setup.
- Original recommendation: calm dense operations with compact lists/tables,
  master/detail, progressive disclosure, and guided activation.
- Supersession: original recommendation superseded as the leading direction;
  its high-volume scan-speed concern is retained.
- Rationale: not supplied by the owner.
- Open follow-up: none; Round 3 confirmed the hybrid split.
- Affected future documents: interaction/visual spec, navigation, components,
  inventory/evidence UX, accessibility, and usability tests.
- Recorded on: 2026-08-25.

### Decision 12 — IA, navigation, and default landing

- Status: `confirmed-round-3`
- Owner answer: persistent sidebar with Home, Agents, Providers, Work,
  Activity, and System; resumable first-use activation wizard; card-led Home
  after activation.
- Original recommendation: the same selection.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: Personal IA, navigation/routes, onboarding,
  desktop state restoration, and usability tests.
- Recorded on: 2026-08-25.

### Decision 13 — card and operational view split

- Status: `confirmed-round-3`
- Owner answer: cards for Home/onboarding/milestones/summaries; list/table +
  master/detail for inventory, bindings, Activity, and evidence.
- Original recommendation: the same hybrid.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: interaction/visual spec, components, inventory,
  evidence UX, responsive behavior, accessibility, and tests.
- Recorded on: 2026-08-25.

### Decision 14 — post-P0 work hierarchy

- Status: `confirmed-round-3`
- Owner answer: Task first; Goal is a lightweight outcome/reference;
  Workstream deferred until cross-Task demand is validated.
- Original recommendation: the same hierarchy.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: Personal product model, Work IA, Task contracts,
  storage/API deltas, migration, and Enterprise integration.
- Recorded on: 2026-08-25.

### Decision 15 — next-slice assignment and autonomy

- Status: `confirmed-round-3`
- Owner answer: owner explicitly selects a ready Agent binding; the system
  previews scope, Provider/model, resources, budget, and acceptance, then
  executes after admission and interrupts only on exception.
- Original recommendation: the same autonomy default.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: governed Task slice, Assignment, preview, policy,
  notifications, Activity, budgets, and acceptance.
- Recorded on: 2026-08-25.

### Decision 16 — Activity and evidence depth

- Status: `confirmed-round-3`
- Owner answer: card summary, readable detail, and audit-grade sources/events;
  authority versus observation, source, freshness, and missing facts remain
  explicit.
- Original recommendation: the same three layers.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: Activity IA, evidence UX, source taxonomy,
  disclosures, export, accessibility, and verification tests.
- Recorded on: 2026-08-25.

### Decision 17 — desktop shell technology

- Status: `confirmed-round-3`
- Owner answer: do not select a framework yet; compare at least two candidates
  using equivalent security and packaging criteria.
- Original recommendation: conditionally prefer a Tauri-like Rust-native shell
  after a focused compatibility/security spike.
- Supersession: original recommendation superseded as a selection; Tauri-like,
  Electron, and other implementations remain ADR/spike candidates.
- Rationale: not supplied by the owner.
- Affected future documents: Personal architecture, threat model, IPC,
  packaging/updates, platform qualification, dependency policy, and delivery
  readiness.
- Recorded on: 2026-08-25.

### Decision 18 — system tray and notifications

- Status: `confirmed-round-3`
- Owner answer: opt-in tray; only actionable activation success/failure,
  readiness loss/expiry, and later Task decision/failure notifications; every
  notification deep-links to durable state.
- Original recommendation: the same behavior.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: desktop lifecycle, onboarding/permissions,
  notification taxonomy, background behavior, privacy, accessibility, and
  Windows validation.
- Recorded on: 2026-08-25.

### Decision 19 — Enterprise wedge

- Status: `confirmed-round-4`
- Owner answer: execution assurance/governed work plus complete
  conversation/Task/context migration within or across Agent tools, with the
  objective of carrying work through to completion; Provider/subscription
  management is second priority.
- Original recommendation: execution assurance and governed work.
- Supersession: original recommendation extended with migration/continuation
  and explicit second-priority Provider/subscription management.
- Owner-stated intent: carry work through to completion.
- Round 5 resolution: portability excludes hidden chain-of-thought,
  Provider-private state, credentials, unsupported sessions, and unauthorized
  content; qualified guarantee falls back to terminal accountability.
- Affected future documents: Enterprise product, continuation contract,
  execution/recovery, Provider priority, metrics, architecture, and validation.
- Recorded on: 2026-08-25.

### Decision 20 — Enterprise deployment

- Status: `confirmed-round-4`
- Owner answer: central Web governance plane/integration services plus
  customer/node authority daemons.
- Original recommendation: the same deployment split.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: Enterprise architecture, topology, tenancy,
  synchronization, identity, SLOs, threat model, and operations.
- Recorded on: 2026-08-25.

### Decision 21 — external-system federation

- Status: `confirmed-round-4`
- Owner answer: stable federated references/connectors; CognitiveOS owns only
  execution-specific facts.
- Original recommendation: the same posture.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: system-of-record matrix, connectors, identity,
  secret isolation, incidents, work/knowledge refs, and retention.
- Recorded on: 2026-08-25.

### Decision 22 — central evidence retention

- Status: `confirmed-round-4`
- Owner answer: minimized signed projections, receipts, digests, source refs,
  policy/decision facts, and bounded summaries; raw logs/artifacts remain at
  node/source.
- Original recommendation: the same minimized retention.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: evidence architecture, classification,
  retention/deletion, export, encryption, residency, incident response, and UX.
- Recorded on: 2026-08-25.

### Decision 23 — Enterprise Agent registry

- Status: `confirmed-round-4`
- Owner answer: federated external registry plus CognitiveOS
  execution/governance overlay.
- Original recommendation: the same overlay.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: Agent model, registry connectors, sponsorship,
  identity mapping, fleet IA, policy targeting, and migration.
- Recorded on: 2026-08-25.

### Decision 24 — policy engine

- Status: `confirmed-round-4`
- Owner answer: versioned decision/evidence contract first, pluggable engine;
  do not lock OPA or Cedar now.
- Original recommendation: the same contract-first posture.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Affected future documents: policy ADR, decision API, bundles, explanation,
  distribution/cache, conformance, and migration.
- Recorded on: 2026-08-25.

### Decision 25 — Enterprise Knowledge

- Status: `confirmed-round-4`
- Owner answer: managed central Knowledge index that copies content/embeddings
  and authorizes retrieval.
- Original recommendation: source-native authorization/retrieval with
  CognitiveOS storing references and governance metadata.
- Supersession: owner selected original Alternative A; original recommendation
  superseded.
- Rationale: not supplied by the owner.
- Round 5 resolution: source opt-in, pre-index authorization, approved
  classification, residency/retention, tenant partition, encryption,
  provenance, ACL freshness, revocation/deletion, verified purge, and injection
  controls are required.
- Affected future documents: Knowledge architecture, ingestion/retrieval auth,
  indexing, tenancy, retention, provenance, and threat model.
- Recorded on: 2026-08-25.

### Decision 26 — Enterprise primary UI

- Status: `confirmed-round-4`
- Owner answer: desktop fleet application.
- Original recommendation: Web admin/operations control plane.
- Supersession: owner selected original Alternative A; Web UI is superseded as
  primary UI, not necessarily as a fallback/deep-admin surface.
- Rationale: not supplied by the owner.
- Round 5 resolution: Desktop Fleet primary; Web UI supported fallback/
  deep-admin; shared backend, permissions, and durable routes.
- Affected future documents: Enterprise IA, surface ownership, sessions,
  permissions, distribution, accessibility, and usability validation.
- Recorded on: 2026-08-25.

### Decision 27 — migration unit

- Status: `confirmed-round-5`
- Owner answer: versioned Portable Continuation Package with Task contract/
  objective/acceptance, decisions, authorized transcript excerpts/summaries,
  ContextView/source refs, artifacts, Effects/evidence, non-secret
  binding/budget state, blocker, and durable next action.
- Original recommendation: the same package.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Recorded on: 2026-08-25.

### Decision 28 — transfer control

- Status: `confirmed-round-5`
- Owner answer: bounded automatic retry inside the same approved binding;
  cross-tool transfer/re-Assignment requires owner confirmation and preview.
- Original recommendation: the same split.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Recorded on: 2026-08-25.

### Decision 29 — completion semantics

- Status: `confirmed-round-5`
- Owner answer: qualified guarantee only for qualified Task classes while
  declared authority/resource/budget/deadline preconditions remain satisfied;
  otherwise terminal accountability.
- Original recommendation: terminal accountability for all cases.
- Supersession: owner selected original Alternative A; terminal accountability
  remains the required fallback.
- Rationale: not supplied by the owner.
- Recorded on: 2026-08-25.

### Decision 30 — central Knowledge boundary

- Status: `confirmed-round-5`
- Owner answer: source opt-in, pre-index authorization, approved
  content/classification only, configured residency/bounded retention, tenant
  partition, encryption, provenance, ACL freshness, revocation/deletion, and
  verified purge.
- Original recommendation: the same boundary.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Recorded on: 2026-08-25.

### Decision 31 — Enterprise UI relationship

- Status: `confirmed-round-5`
- Owner answer: Desktop Fleet primary; Web UI supported fallback/deep-admin;
  both share backend, permissions, and durable routes.
- Original recommendation: the same relationship.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Recorded on: 2026-08-25.

### Decision 32 — conversation/context locality

- Status: `confirmed-round-5`
- Owner answer: node/local canonical; encrypted tenant-scoped cloud sync only
  when policy and user authority explicitly enable it.
- Original recommendation: the same locality.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Recorded on: 2026-08-25.

### Decision 33 — visual and brand relationship

- Status: `confirmed-round-5`
- Owner answer: shared restrained CognitiveOS brand/semantic state system;
  Personal uses spacious cards plus hybrid operational views; Enterprise raises
  density for queue/fleet/Activity/evidence.
- Original recommendation: the same relationship.
- Supersession: none; original recommendation confirmed.
- Rationale: not supplied by the owner.
- Recorded on: 2026-08-25.

### Decision 34 — Provider/subscription release boundary

- Status: `confirmed-round-5`
- Owner answer: ships in the same first Enterprise release as execution
  assurance/migration, with separate acceptance and capability honesty.
- Original recommendation: P1 after the first release.
- Supersession: owner selected original Alternative A; separate acceptance
  remains required.
- Rationale: not supplied by the owner.
- Recorded on: 2026-08-25.

## 14. Final requirements round — confirmed

The exact Chinese Round 5 prompts are retained below as the decision record;
selected options are recorded in §12 and §13.

1. **跨 Agent 工具迁移的精确迁移单元应是什么？**
   - A — 推荐：版本化 Portable Continuation Package，包含 TaskContract/目标与验收、决策、获准的 transcript 摘录或摘要、ContextView/来源引用、artifact、Effects/evidence、非 secret 的 binding/budget 状态、blocker 和持久 next action。
   - B — 在 Provider/Agent 工具支持时导出 raw/full transcript 与 session，并执行 redaction 和 authority 检查。
   - C — 只使用 Provider 官方 native session continuation，不定义通用跨工具 package。
2. **重试、转移和重新 Assignment 应由谁触发？**
   - A — 推荐：同一已批准 binding 内允许有界自动重试；跨工具转移或重新 Assignment 必须由 owner 确认，并预览 Continuation Package 与后果。
   - B — 每次重试、转移和重新 Assignment 都要求 owner 确认。
   - C — 在明确 budget/time/attempt/policy 边界内，系统可自动重试并在合格工具间重新 Assignment，仅在异常时通知。
3. **“确保完成”应采用哪一种诚实语义？**
   - A — 推荐：terminal accountability——在明确边界内达到独立验收完成，或形成带 evidence、owner 和 next action 的持久 blocked/failed 终态。
   - B — 仅对已资格化 Task 类、且声明的 authority/resource/budget/deadline 前提持续满足时承诺成功；其他情况采用 terminal accountability。
   - C — 不提供完成保证，只展示 best-effort Agent attempts 和当前状态。
4. **Managed central Knowledge index 的 enrollment、内容和 retention 边界是什么？**
   - A — 推荐：按来源 opt-in；索引前授权；仅复制批准的内容/分类；配置 residency 和有界 retention；强制 tenant partition、encryption、provenance、ACL freshness、revocation、deletion 与 verified purge。
   - B — 组织范围默认 enrollment，由管理员排除来源并用 policy 设定 retention。
   - C — 初期只索引 metadata 和 embedding，检索时从来源获取正文。
5. **Desktop Fleet 与 Web governance UI 应是什么关系？**
   - A — 推荐：Desktop Fleet 是主要 operator experience；Web UI 保留为受支持的 fallback/deep-admin surface，并共享同一 governance backend、权限和持久 route。
   - B — Desktop Fleet 是唯一产品 UI；中央 Web plane 只提供 API/service，不提供 operator UI。
   - C — Web UI 仍是主要 admin surface；Desktop Fleet 是专用 fleet/notification companion。
6. **Conversation 与 Continuation Package 的存储本地性和 cloud sync 姿态是什么？**
   - A — 推荐：node/local storage 为 canonical；只有 policy 与用户 authority 显式启用时，才进行 encrypted、tenant-scoped cloud sync。
   - B — central cloud storage 为 canonical，node 仅保留离线 cache。
   - C — 仅 node/local storage，不提供 cloud sync。
7. **Personal 与 Enterprise 的详细视觉和品牌关系应是什么？**
   - A — 推荐：共享克制的 CognitiveOS brand 和 semantic state system；Personal 延续宽松 consumer cards + hybrid operational views，Enterprise Desktop Fleet 对 queue、fleet、Activity、evidence 使用更高运营密度。
   - B — Personal 与 Enterprise 使用相同的宽松卡片语言和密度。
   - C — Personal 与 Enterprise 使用明显独立的 brand 和 component system。
8. **Provider/subscription management 的第二优先级应落在哪个 release boundary？**
   - A — 推荐：作为 execution-assurance/continuation 首发之后的 Enterprise P1，使用独立 acceptance 和 release criteria。
   - B — 与 execution assurance 和 migration 一起进入首个 Enterprise release。
   - C — Enterprise 仅提供 read-only discovery/reference，management 无限期延后。

Requirements confirmation is complete. Detailed candidate documents `03`–`10`
have been generated; any contradiction found during implementation shaping
must be returned to owner review rather than silently resolved.

## 15. Candidate specification set

The generated set remains candidate/non-canonical and gives no implementation
authorization:

1. `03-personal-product-design.md`
2. `04-personal-interaction-and-visual-spec.md`
3. `05-personal-architecture.md`
4. `06-enterprise-product-design.md`
5. `07-enterprise-interaction-and-visual-spec.md`
6. `08-enterprise-architecture.md`
7. `09-shared-domain-and-contract-boundaries.md`
8. `10-validation-and-delivery-readiness.md`
9. `11-repository-governance-and-topology-recommendation.md`
10. `12-open-source-reuse-assessment.md`
11. `13-control-plane-baseline-to-personal-desktop-1.0-delta.md`

The Personal documents should be completed and reconciled before Enterprise is
promoted beyond discovery.

## 16. Owner scope expansion — confirmed 2026-08-25

Classification: **owner-confirmed-scope-expansion / non-canonical**.

### Decision 35 — Personal Desktop 1.0 candidate

The owner explicitly requests one highly usable Personal desktop product for:

1. daily office workers；
2. programmers；
3. researchers。

The same product uses saved views, presets, terminology help and progressively
disclosed detail; it does not fork three product editions.

The Personal Desktop 1.0 **candidate** prioritizes all of:

- Provider plan/account/auth/entitlement/model management；
- explicit Agent↔Provider/model Binding；
- Knowledge sources；
- durable Memory；
- Skills；
- Tools；
- source-typed token usage/cost；
- scoped Context；
- local-first Conversation history and continuation。

The candidate experience thesis is a local-first continuity workspace:

```text
Ready → Continue → Review → Work → Verify → Retain
```

The style direction is macOS-like in restraint, hierarchy, depth, feedback and
simplicity, while following Windows-first chrome, keyboard, accessibility and
high-contrast conventions. It explicitly rejects copied macOS chrome,
decorative glassmorphism, card walls and generic confidence scores.

### Reconciliation with the earlier inventory-only P0

The earlier activation/readiness P0 remains the **first milestone**, not the
whole Desktop 1.0 candidate:

| Depth | Candidate Desktop 1.0 meaning |
|---|---|
| P0 | activation + one supported Provider/Binding + one Conversation + one authorized input/source + Context inspection + explicit Memory retain/not-retain + visible Skill/Tool availability + source-typed token/cost status |
| P1 | governed Work/Task path, Knowledge source management, Conversation resume/import/export/delete, usage drill-down, continuation checkpoint, stronger Library lifecycle |
| P2 | additional qualified adapters, saved persona views, richer retrieval/continuation, delegated Provider controls only where officially supported |

All owner-priority domains belong to the candidate release, but their depth and
capability classification differ. `designed` or imported data is not
`implemented`, `HTTP-accessible`, `tested`, `Gate-proven` or provider-authoritative.

### Meaning of “manage”

“Manage” is not a blanket mutation claim. Every action must be rendered in one
of six modes:

1. **Managed here** — a daemon-owned, supported mutation exists.
2. **Link / reauthenticate** — approved Provider handoff exists.
3. **Observe read-only** — a supported source provides facts.
4. **Open Provider** — first-party deep link owns the action.
5. **Record manually** — user-declared fact, labeled as such.
6. **Unavailable** — capability is unknown, unsupported or inaccessible.

No UI may expose purchase/cancel/upgrade/quota/reset/remaining-allowance or
invoice controls without an official supported mutation/read capability.

### Canonical conflict

**FACT**：Accepted ADR-0036 and the formal Personal plan still define
`GMVP-LINUX` as Personal `1.0.0`, Linux x86_64 as the 1.0 platform, Pi as the
only qualified Agent, and Web UI/Windows installer as post-1.0.

Therefore:

- “Personal Desktop 1.0 candidate” is discovery language only；
- it does not supersede Linux `1.0.0`；
- it does not rename or modify `GMVP-LINUX`；
- it does not authorize product code, client, dependency, contract, task,
  branch, PR or release work；
- canonical adoption requires an accepted product-semantic ADR and formal-plan
  rebaseline after P7-T05/D13 ownership is resolved。

See [Personal product design](03-personal-product-design.md),
[open-source reuse](12-open-source-reuse-assessment.md), and
[baseline delta map](13-control-plane-baseline-to-personal-desktop-1.0-delta.md).
