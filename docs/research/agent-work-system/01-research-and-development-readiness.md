# COGNITIVEOS AGENT WORK SYSTEM — RESEARCH AND DEVELOPMENT READINESS

Date: 2026-08-25
Status: **candidate research package / non-canonical / no implementation authorization**

This document is a dated synthesis of repository reality, owner intent,
Paperclip research, Provider and subscription research, market and standards
research, product modeling, architecture deltas, and pre-development shaping.
It does not register a task, accept an ADR or PRD, modify a machine contract,
or authorize Personal or Enterprise implementation.

## 1. Executive Decision

1. **[RECOMMENDATION]** Position CognitiveOS as a **Governed Agent Work
   System**, not a chat aggregator or a simulated company of virtual employees.
2. **[RECOMMENDATION]** The Personal core loop should be:
   `Intent/Goal → Task → Assignment → Governed Execution → Verification →
   Accepted Result → Safe Continuation`.
3. **[RECOMMENDATION]** Absorb Paperclip's work-centered orchestration and
   supervision UX; reject its company metaphor, agent self-completion,
   process-exit success semantics, second scheduler authority, and runtime.
4. **[FACT]** The repository already has strong Task, Scheduler,
   Intent/Effect, Evidence, Verifier, Provider, Memory, and Skill foundations.
   First-class Assignment, Goal/Project, rich Work projection, unified
   Activity, and durable Run identity are absent or partial.
5. **[RECOMMENDATION]** Model commercial access as
   `Provider → AccessAccount → Auth/SecretRef → Entitlement → Binding →
   UsageObservation → Budget → CostObservation`; do not add a universal
   `Subscription` kernel entity now.
6. **[RECOMMENDATION]** Keep Personal owner-local and single-principal.
   Enterprise may later add a governance plane, but the node daemon remains
   the only writer of local Task, Intent, Effect, and execution facts.
7. **[RECOMMENDATION]** Do not build a copied-content KnowledgeBase now. Start
   with external source registration, Context governance, pre-retrieval
   authorization, provenance, and usage evidence.
8. **[RECOMMENDATION]** Deliver one complete Personal path with at most five P0
   capabilities. Keep Enterprise at discovery and architecture-headroom stage.
9. **[RECOMMENDATION]** Do not create a new repository or monorepo now. Prove
   the product boundary in the current kernel/private-service and formal client
   topology.
10. **[RECOMMENDATION]** Overall readiness is **PARTIALLY READY**: discovery
    and shaping can proceed, but primary user evidence is missing and the
    active P7-T05/D12 lease overlaps the future client surface.

## 2. Owner Intent Interpretation

| Conclusion | Classification | Interpretation |
|---|---|---|
| Personal should center durable real work rather than one-response chat | PRODUCT HYPOTHESIS | Strong fit with the current authority kernel, but not yet supported by first-party user research |
| Multi-Agent, Provider access, cost, task, and evidence need one supervision surface | PRODUCT HYPOTHESIS | Public pain signals exist; frequency, intensity, and willingness to pay remain unknown |
| Agent results require independent verification | FACT | Required by A1–A8 and the current Task acceptance architecture; see `docs/governance/AXIOMS.md` |
| Enterprise should reuse the execution substrate and add governance | RECOMMENDATION | Avoids a second authority writer and preserves local execution correctness |
| Personal should precede Enterprise | FACT / RECOMMENDATION | `cognitiveos-personal` is the sole active implementation project; see `docs/governance/PROJECT-IDENTITY.md` |
| “AI Workforce OS” is a validated category | OPEN QUESTION | Current evidence does not establish customer language, buying behavior, or acceptable employee-replacement framing |

Non-goals:

- **[RECOMMENDATION]** Do not design Personal as a one-person company with CEO,
  hiring, salary, or reporting-line metaphors.
- **[RECOMMENDATION]** Do not make Enterprise a replacement for HRIS, IAM,
  Secret Manager, SIEM, DLP, FinOps, DMS, OKR, or project-management systems.
- **[FACT]** Do not let an Agent, Provider, process, or central governance plane
  become an authority writer.
- **[RECOMMENDATION]** Do not claim management of consumer plans, remaining
  allowance, or invoices when a Provider exposes no supported interface.
- **[RECOMMENDATION]** Do not force Personal to implement multitenancy, SCIM,
  ReBAC, SPIFFE, or centralized policy before a concrete Enterprise need.
- **[FACT]** `docs/design/01–41` is a dated and currently untracked baseline,
  not an Accepted product or architecture decision.

## 3. Repository Reality and Existing Assets

### 3.1 Authority and topology

- **[FACT]** Repository facts are ordered as: `specs/` machine shape →
  normative standards → Accepted ADRs → Personal formal plan → `PROGRESS.md`
  Current snapshot → active leases → product and design material.
- **[FACT]** The Rust daemon is the sole authority writer. Agents, Providers,
  and third-party runtimes produce candidates and observations.
- **[FACT]** Contracts, daemon, runtime, SQLite store, management, SecretStore,
  and Provider transport live in this repository. The formal Personal Web
  client lives in `cognitiveos-clients/pc/web`.
- **[FACT]** `apps/cognitiveos-console` is a deprecated documentation stub, not
  the Personal SPA.

### 3.2 Capability reality

| Capability | State | Evidence and delta |
|---|---|---|
| Task / TaskContract | specified; implemented; HTTP-accessible; tested; bounded Gate-proven in parts; not release/Profile-proven | `specs/schemas/task-contract.schema.json`; `crates/cognitive-management/src/task_application.rs`; KEEP and add a Work projection |
| Assignment | absent as a product work relation | `AgentExecutionBinding` may reference Task, but there is no typed assignee, priority, or assignment API; NEW |
| Scheduler / lease / fencing | implemented; CLI/library-only; tested | Durable CAS lease, epoch, expiry, reclaim in `crates/cognitive-store/src/scheduler.rs`; KEEP |
| Task budget | implemented; hard-enforced; tested | `crates/cognitive-runtime/src/scheduler_service.rs`; KEEP |
| Provider budget | implemented; HTTP-accessible; tested; observe-only | Add explicit enforcement class; never label as hard limit |
| Intent / Effect | specified; implemented; tested; partly HTTP-readable | `crates/cognitive-kernel/src/ports.rs`; `specs/transitions/effect.transitions.json`; KEEP |
| Evidence / Verification / Acceptance | implemented; HTTP-accessible; tested; bounded Gate-proven | Production fixed-effect and registered-check verifiers; KEEP as product differentiator |
| Agent installation and runtime | implemented; mostly CLI/library-only; partly tested; HTTP projection incomplete | Installation, registration, instance, sidecar, and attempts are durable; runtime resource projection remains empty |
| Provider access / model / binding / usage | implemented; HTTP/CLI-accessible; tested; not Provider-Control-Plane Gate-proven | Add entitlement and cost provenance; strengthen mutation atomicity and audit |
| HTTP session | implemented; HTTP-accessible; tested; owner-local | Useful for Personal only; not Enterprise IAM |
| ContextSource / ContextRequest / ContextView | specified; internally implemented; partly HTTP-readable; B03 MVP-proven | Extend authorized read projection |
| Memory | specified; implemented; HTTP lifecycle; tested; B08 MVP-proven | FTS search and proposal review are not HTTP-accessible |
| Skill | privately implemented; HTTP-accessible; tested; public contract deferred | Do not invent a public Skill schema for a UI |
| Artifact | implemented; internal CAS; tested; no generic HTTP | Expose bounded metadata through evidence projection |
| Knowledge | designed/specification-level; product implementation absent | No Knowledge store, product service, or HTTP lifecycle |
| Approval | specified; library-tested; product integration absent | Existing approval gate is not wired into Personal HTTP; preview/admit is the current real owner decision |
| Activity / audit | partially implemented; partly HTTP-accessible; unified domain absent | Task transitions, Effects, and Provider audit are fragmented |
| Goal / Project / Workstream | absent | Task objective text and a `project` scope enum do not establish these domains |
| Run | designed/partial observations; durable first-class identity absent | Start with a derived ExecutionAttempt projection |

### 3.3 Current Git and ownership collision

- **[FACT]** The Current snapshot now records `P7-T05/D12` as in progress on
  `lease/personal/P7-T05/control-plane-foundation`.
- **[FACT]** The kernel worktree is on
  `personal/P7-T05-control-plane-foundation` at
  `593a5adf7f9b24f3e2e635ee3ce38be16b5d6c02`.
- **[FACT]** `docs/plan/PROGRESS.md` is modified by the active work and is not
  owned by this discovery package.
- **[FACT]** Existing untracked `.cursor/skills/`, `docs/design/`, and temporary
  scripts are protected and were not modified.
- **[FACT]** Future Work, shared shell, client data/state, and visual paths can
  overlap the active Control Plane work.
- **[RECOMMENDATION]** Any implementation task must wait for the active task to
  close or transfer its paths and must begin from a clean, explicit lease.
- **[FACT]** This research package did not rerun product tests, inspect live
  daemon data, or promote recorded evidence.

## 4. User Problems and JTBD

### 4.1 Personal jobs

1. **[PRODUCT HYPOTHESIS]** When I use multiple Agent tools, I want one place
   to see capability, health, and task fit, so I can select the right Agent.
2. **[PRODUCT HYPOTHESIS]** When I hold multiple AI plans or API accounts, I
   want to see entitlement, cost status, and bindings, so I can avoid account
   confusion and uncontrolled spend.
3. **[PRODUCT HYPOTHESIS]** When an outcome requires hours or multiple
   sessions, I want to turn it into a tracked Task and assign an Agent, so I do
   not repeatedly explain context.
4. **[PRODUCT HYPOTHESIS]** When an Agent is working, I want to see why it
   started, what it is doing, why it is blocked, and whether it needs me, so I
   can supervise by exception.
5. **[PRODUCT HYPOTHESIS]** When an Agent claims completion, I want independent
   verifier, Effect, and acceptance evidence, so I do not mistake plausible
   output for a completed task.
6. **[PRODUCT HYPOTHESIS]** When work is interrupted, I want to resume from a
   durable next action and committed facts, so I avoid duplicate effects and
   lost progress.

| Force | Personal assessment |
|---|---|
| Push | Fragmented IDE/CLI tabs, sessions, issues, Provider dashboards, manual verification, and context restatement |
| Pull | One Work-centered surface, exception-first supervision, independent acceptance, explicit cost state, and durable continuation |
| Anxiety | Expanded authority, credential custody, false completion, and another management dashboard |
| Habit | Terminal multiplexers, IDE tabs, issues, README/checklists, spreadsheets, CI, and human review |
| Workaround | Copy context manually, paste Agent output into issues, use Git/PR/CI as partial evidence, inspect Provider usage separately |
| Existing evidence | **[FACT]** Selected Cursor, OpenHands, Claude Code, LangSmith, and Cline reports show supervision, resume, missing-usage, and premature-completion friction; these are weak anecdotes |
| Missing research | Frequency, time loss, willingness to pay, non-coding applicability, and acceptable preview/approval friction |

### 4.2 Enterprise jobs

1. **[PRODUCT HYPOTHESIS]** When Agent use grows across the organization, I
   want to discover, register, constrain, and audit every Agent, so shadow and
   ownerless agents do not persist.
2. **[PRODUCT HYPOTHESIS]** When a business sponsor funds Agent work, I want to
   link Agent, Task, outcome, evidence, and cost to organizational intent, so
   value and accountability are visible.
3. **[PRODUCT HYPOTHESIS]** When administrators allocate Provider access and
   budget, I want scope-aware assignment, so credentials are not shared and
   spend is attributable.
4. **[PRODUCT HYPOTHESIS]** When a Knowledge owner exposes sources to Agents, I
   want authorization before retrieval based on identity, purpose, and Task,
   so cross-scope leakage is prevented and evidenced.
5. **[PRODUCT HYPOTHESIS]** When an Agent proposes a high-risk action, I want
   policy, risk, approval, and verification to constrain it, so automation can
   scale without abandoning governance.
6. **[PRODUCT HYPOTHESIS]** When identity, organization, or runtime state
   changes, I want authority to narrow quickly and in-flight work to be handled
   safely, so revoked access cannot continue unnoticed.

| Force | Enterprise assessment |
|---|---|
| Push | Shared service accounts, ownerless agents, fragmented audit, opaque spend, data leakage, and delayed revocation |
| Pull | Unified accountability, Task-scoped authority, source-native access, evidence, cost attribution, and containment |
| Anxiety | Central outage, policy complexity, false blocking, sensitive-data copying, latency, and lock-in |
| Habit | Existing Entra/Okta, HRIS, Vault, SIEM, DLP, Jira/ServiceNow, DMS/catalog, and FinOps systems |
| Workaround | Treat Agents as service accounts, track inventory in CMDB/spreadsheets, correlate logs in SIEM, and use manual approvals |
| Existing evidence | **[FACT]** Agent 365, Entra Agent ID, ServiceNow, and Google/AWS registries show a supplier category; they do not prove CognitiveOS demand |
| Missing research | Economic buyer, revocation SLO, inventory scale, offline authority, sponsor succession, source mix, and cost-dispute workflow |

### 4.3 Validation plan

- **[RECOMMENDATION]** Interview at least five, preferably eight, Personal
  recent switchers/evaluators and the same number of Enterprise evaluators from
  the last 90 days, including non-buyers.
- **[RECOMMENDATION]** Reconstruct the trigger timeline:
  first thought → search → evaluation → first run → first failure →
  recovery/abandonment.
- **[RECOMMENDATION]** Run a 14-day Personal diary and a four-week,
  three-team Enterprise shadow study without collecting prompt bodies or
  secrets.
- **[RECOMMENDATION]** Use a concierge Task/Agent ledger, exception alerts, and
  recovery packet before automating the solution.
- **[RECOMMENDATION]** Require a paid continuation, budget commitment, or
  design-partner deposit for commercial validation.

## 5. Paperclip Adopt / Adapt / Reject Matrix

Research baseline: Paperclip
[v2026.817.0](https://github.com/paperclipai/paperclip/releases/tag/v2026.817.0)
and the 2026-08-24 master revision.

| Paperclip concept | Decision | CognitiveOS mapping |
|---|---|---|
| Company | ADAPT | Personal Workspace / Enterprise Organization; reject company theater in Personal |
| Agent / Employee | ADAPT; REJECT term | Separate Profile, Instance, runtime, workload identity, and execution |
| Role / reporting | DEFER Personal; ADAPT Enterprise | Personal purpose/capability; Enterprise sponsor/owner relationships |
| Goal | ADAPT | Outcome anchor; progress cannot be Agent self-report |
| Project | ADAPT later | Workstream/external-project link; no MVP lifecycle |
| Issue / Task | ADAPT heavily | CognitiveOS Task + immutable TaskContract |
| Parent-child Task | ADOPT concept | Decomposition only; no automatic parent completion |
| Blocker / dependency | ADOPT | Explicit directed readiness relation and durable reason |
| Assignment | ADAPT | Separate user assignment from current execution authority |
| Atomic checkout | ADOPT semantics | Single execution owner plus CognitiveOS lease epoch/fence |
| Heartbeat / wakeup | ADAPT | Scheduler Wakeup / Execution Trigger only |
| Run | ADAPT | ExecutionAttempt projection, never a Task verdict |
| Adapter | ADAPT boundary | Candidate/observation, never authority |
| Workspace | ADAPT selectively | Keep identity/coherence/recovery UX; reject unknown-change mutation and default concurrent writers |
| Routine / schedule | ADOPT pattern; DEFER MVP | Materialize a governed Task; never bypass admission |
| Budget | ADAPT | Admission/scheduling/dispatch; hard monetary claims require reservation |
| Cost event | ADAPT as observation | Source-typed usage/cost; not invoice truth |
| Approval | ADOPT UX; ADAPT authority | Revision-bound, exact-once; reevaluate after approval |
| Interaction / confirmation | ADOPT | Questions, human-only resolution, stale-revision failure |
| Activity / audit | ADOPT projection | Couple atomically to authority changes; logs are not verification |
| Orphan / stale-lock recovery | ADOPT mechanics | Live/dead classification, bounded retry, lock adoption, and sink fencing |

Explicit rejections:

- **[RECOMMENDATION]** Personal CEO, hiring, salary, and organization chart.
- **[RECOMMENDATION]** Agent-authored `done` or other self-completion.
- **[RECOMMENDATION]** Process exit, adapter report, comment, transcript, or
  activity row as proof of Task completion.
- **[RECOMMENDATION]** Secret injection through argv, environment, ordinary
  configuration, prompts, or browser-readable material.
- **[RECOMMENDATION]** Arbitrary Process adapters as trusted Agents.
- **[RECOMMENDATION]** Paperclip scheduler/runtime beside the CognitiveOS
  daemon.
- **[RECOMMENDATION]** Default shared-workspace concurrent writers or automatic
  recovery that changes an ownership-unknown worktree.

| Layer | Final disposition |
|---|---|
| Product operating model | Absorb work-centered long-running supervision, not autonomous-company framing |
| Work orchestration | Strongly adapt Assignment, blocker, checkout, wakeup, attempt, routine, and recovery |
| UX patterns | Strongly adopt trigger reason, attempt timeline, confirmation, budget, and recovery surfaces |
| Execution semantics | Reject; retain CognitiveOS Intent/Effect, fencing, verification, and acceptance |
| Runtime | Do not embed, fork, or treat as a compatibility target |

## 6. Subscription and Provider Access Research

| Product | Subscription/API and authentication | Visibility and management boundary | CognitiveOS posture |
|---|---|---|---|
| Claude Pro/Max + Claude Code | Subscription may fund Claude Code; API billing is separate; OAuth/API key/cloud identity | First-party usage UI; no public consumer-quota API | Native link/deep-link; observe API account; never broker consumer OAuth |
| ChatGPT Plus/Pro + Codex | ChatGPT entitlement and API billing are separate; OAuth/API key | First-party usage; API organizations have usage/cost APIs; no consumer-plan API | Link/deep-link; observe API account |
| Gemini CLI | Individual Free/AI Pro/Ultra OAuth was retired in 2026; enterprise/API paths remain | `/stats` is local/session data, not remaining entitlement | Enterprise link/observe; individual path unavailable |
| Google Antigravity | Consumer CLI entitlement; OAuth/API key/Enterprise SSO/WIF | First-party `/usage` and `/credits`; no general consumer-plan management API | Link/deep-link; Enterprise observe |
| Cursor | Entitlement covers Agent/CLI; BYOK is separate Provider billing | Dashboard and Enterprise Admin API; consumer plan remains first-party | Consumer link/observe; Enterprise delegated controls |
| OpenCode | Open-source runtime; Go allowance plan; Zen PAYG; OAuth/API key | Local stats are estimates; first-party console owns allowance/balance | API binding; linked Go/Zen plan |
| CognitiveOS dsh | dsh is not a subscription; binds a Provider API account | Provider-reported usage and locally versioned price; may be unavailable | Manage local API account/binding/usage, not Provider subscription |
| GitHub Copilot | Consumer/Business/Enterprise entitlement | Enterprise seat and metrics APIs exist | Consumer link; managed Enterprise seat only through explicit delegated API |
| Kiro CLI | Credit-based plan distinct from general model API access | First-party usage and subscription portal | Link/observe/deep-link |

Representative official sources:

- [Claude Code with Pro/Max](https://support.claude.com/en/articles/11145838-use-claude-code-with-your-pro-or-max-plan)
- [Anthropic Claude Code authentication](https://docs.anthropic.com/en/docs/claude-code/iam)
- [OpenAI Codex authentication](https://developers.openai.com/codex/auth)
- [OpenAI Codex pricing and limits](https://developers.openai.com/codex/pricing)
- [Gemini CLI transition](https://developers.googleblog.com/en/an-important-update-transitioning-gemini-cli-to-antigravity-cli/)
- [Antigravity plans](https://antigravity.google/docs/plans/)
- [Cursor models and pricing](https://cursor.com/docs/models-and-pricing)
- [OpenCode providers](https://opencode.ai/docs/providers/)

Candidate model:

```text
Provider
 └─ AccessAccount
    ├─ AuthenticationMethod → SecretRef | NativeRuntimeSessionRef
    ├─ Entitlement → AllowanceWindow*
    ├─ ModelCatalogSnapshot*
    ├─ UsageObservation* → CostObservation*
    └─ ExternalBillingLink / InvoiceRef

AgentBinding → AccessAccount + AuthenticationMethod + ModelRef
BudgetPolicy → Account | Binding | Agent | Task
```

- **[RECOMMENDATION]** Do not create a universal kernel `Subscription` entity.
- **[RECOMMENDATION]** Distinguish `managed subscription`,
  `observable entitlement`, `linked external subscription`,
  `manually declared plan`, and `unknown/unavailable`.
- **[FACT]** Subscription OAuth is not an API credential.
- **[FACT]** Shared quotas may span Web, CLI, and multiple Agents; local usage
  cannot reliably calculate remaining allowance.
- **[RECOMMENDATION]** Cost observations require
  `provider_reported_accrual`, `local_estimate`, `invoice`, or `unavailable`.

## 7. Personal Product Model

```text
Personal Workspace
├── Intent / Goal-lite
├── Work
│   ├── Tasks
│   ├── Assignments
│   ├── Blockers
│   └── Execution attempts / evidence projection
├── Agents
│   ├── Profiles
│   ├── Installations / Instances
│   └── Bindings
├── Provider Access
├── Resources
│   ├── Context / Memory / Skill / Tool
│   └── Artifact / external Knowledge refs
├── Activity
└── System
```

- **[RECOMMENDATION]** Work/Task is the primary product object, not Agent org
  structure or chat.
- **[RECOMMENDATION]** Personal Workspace is a single-owner namespace.
- **[RECOMMENDATION]** Goal begins as an outcome anchor or external reference;
  TaskContract remains execution authority.
- **[RECOMMENDATION]** Add Project/Workstream only after repeated multi-Task
  aggregation is observed.
- **[RECOMMENDATION]** Activity distinguishes authority transition,
  observation, cost observation, and user interaction.

```text
Capture intent
→ Clarify scope and acceptance
→ Preview authority/cost/resources
→ Admit Task
→ Assign eligible Agent binding
→ Wake scheduler
→ Acquire fenced lease
→ Produce candidate and governed Effects
→ Independent verification
→ Daemon acceptance
→ Evidence receipt
→ Goal progress projection / durable next action
```

The product must represent Empty, Loading, Partial, Stale, Permission, Blocked,
Long-running, Failed, Accepted, and Recovery states without conflating them.

## 8. Enterprise Product Model

```text
Enterprise Governance Plane
├── Agent Registry and Sponsorship
├── Identity / Organization references
├── Policy authoring and distribution
├── Entitlement / Budget allocation
├── Knowledge governance
├── Goal / Project links
├── Approval and Risk
├── Audit / Incident integration
└── Fleet projections
        ↓ signed requests, bundles, revocations
Node / Workspace Authority Daemons
        ↓
Agent runtimes and governed Tool execution
```

- **[RECOMMENDATION]** The governance plane owns registry, sponsor, policy
  source, allocation, approval, fleet projection, and connectors.
- **[FACT]** Node daemons own Task, workspace, Intent, Effect, lease, dispatch,
  verification, and local evidence.
- **[RECOMMENDATION]** A central command is a signed mutation request. The node
  reauthorizes, persists Intent, executes, verifies, and emits Effect.
- **[RECOMMENDATION]** The central plane never mounts or writes node SQLite.
- **[RECOMMENDATION]** During partition, high-risk mutation, new enrollment,
  and fresh approval fail closed.
- **[RECOMMENDATION]** Reconnect order is revocation → policy/identity/ACL →
  queued-work reevaluation → evidence/usage drain.

| Fact | System of record | Governance plane | Node daemon |
|---|---|---|---|
| People and organization | HRIS | Minimal projection | Consume signed principal context |
| Human authentication/groups | IdP | Federation/reference | Validate session/delegation |
| Agent registry/sponsor | External registry or CognitiveOS governance | Authoritative governance record | Cache operational subset |
| Workload identity | SPIFFE/cloud IAM | Trust configuration | Local attestation |
| Policy | Policy repository | Author/review/sign/distribute | Verify/evaluate/enforce |
| Task/workspace | Node daemon | Read projection | Sole writer |
| Intent/Effect/Evidence | Node daemon | Aggregate/index | Persist/reconcile |
| Approval | Governance workflow | Authoritative approval | Bind digest and reevaluate |
| Secret value | Secret Manager | Reference only | Just-in-time resolve |
| Knowledge body/ACL | DMS/catalog/source | Metadata projection | Source-native recheck |
| Usage | Node/Provider | Aggregate/export | Deterministic emit |
| Invoice/chargeback | Provider/finance | Reference/report | No authority |

## 9. Personal / Enterprise Shared Boundary

| Capability | Shared substrate | Personal | Enterprise |
|---|---|---|---|
| Contracts | Shared protocol | Owner-local | Realm-aware extension |
| Task / Effect / Evidence | Shared code and protocol | Local authority | Node-local authority |
| Scheduler | Shared code | Local scheduler | Node scheduler; central plane does not dispatch |
| Agent adapter | Shared interface | Local registry | Attested/fleet-managed |
| Tool | Shared code and protocol | Owner-approved | Policy-scoped |
| Memory | Shared concept/partial code | Personal/Task scope | Policy-bound scope |
| Context | Shared protocol/code | Local ContextView | Purpose/ACL-aware ContextView |
| Provider access | Shared concept/connector interface | Owner account | Organization account/seat pool |
| SecretRef | Shared protocol | Local approved SecretStore | Enterprise Secret Manager |
| Usage | Shared event envelope | Task/account observation | Fleet allocation |
| Budget | Shared concept/partial code | Hard Task, advisory Provider | Allocation/reservation |
| Agent registry | Shared concept, not necessarily implementation | Local Profile/Instance | Central lifecycle/sponsor |
| Goal | Shared concept/external reference | Personal outcome | Organizational objective link |
| Organization | Not shared implementation | Absent | Enterprise-only |
| Identity | Shared seam only | Single owner | OIDC/SAML/SCIM/SPIFFE adapters |
| Policy | Shared decision interface | Fixed local evaluator | Signed policy bundles |
| Knowledge governance | Shared principles/references | Owner-authorized sources | Classification/residency/DLP |
| Approval | Shared protocol principles | Preview/admit | Multi-party/separation of duty |
| Audit | Shared envelope | Local projection | SIEM/incident integration |
| Fleet management | Not shared implementation | Absent | Enterprise-only |

## 10. Agent, Goal, Task, and Execution Model

### 10.1 Agent objects

| Object | Meaning | State |
|---|---|---|
| Agent Package | Code, manifest, publisher, version, provenance | Partly implemented |
| Installation | Package installed in an environment | Implemented; CLI/library-only |
| Adapter | Candidate/observation protocol to a runtime | Implemented |
| Registration | Daemon record accepting installation/capability declaration | Implemented/partial |
| Agent Profile | Purpose, capability, Task family, default binding | NEW product projection |
| Agent Instance | A deployed or schedulable instance | Partly implemented |
| Runtime Process | OS process/workload identity | Observation |
| Sidecar Session | Daemon/tool-boundary session | Implemented/partial |
| Agent Session | Runtime conversation/context continuation | Adapter-owned |
| Agent Execution | Controlled Agent-to-Task attempt relation | Specified/partial; production use unknown |

- **[RECOMMENDATION]** Do not collapse these into one universal Agent row.
- **[RECOMMENDATION]** Capability is a claim. Eligibility also requires
  registration evidence, health, policy, and Task compatibility.
- **[RECOMMENDATION]** Assignment answers who was selected; lease/execution
  ownership answers who currently holds fenced execution authority.

### 10.2 Goal, Project, Task, and Run

| Concept | Decision |
|---|---|
| Goal | EXTEND as Goal-lite; defer an independent lifecycle |
| Project/Workstream | DEFER; use external reference or saved view first |
| Task | KEEP as authority unit |
| Parent-child | NEW narrow decomposition relation |
| Blocker/dependency | NEW directed readiness relation |
| Recurring Task/Routine | DEFER; a Routine may only create a new admitted Task |
| Run | Start with derived ExecutionAttempt projection |
| Goal progress | Derive from accepted Task evidence, external SoR, or human-approved metrics |

### 10.3 Execution chain

```text
Goal or external intent
→ Task proposal
→ Clarification
→ Preview
→ Admission
→ Versioned assignment
→ Wakeup trigger
→ Scheduler eligibility
→ Fenced lease
→ Agent candidate
→ Governed Tool request
→ Persisted Intent
→ External dispatch
→ Effect reconciliation
→ Independent verification
→ Daemon acceptance
→ Evidence-backed result
→ Goal progress projection
→ Durable next action
```

## 11. Permission and Identity Model

Personal default:

```text
realm = local
human = owner = sponsor = initiator = approver
agent = durable local Agent ID
workload = local process/daemon identity
policy = fixed owner-local policy
SCIM/SPIRE/ReBAC/central approval = absent
```

Independent verification remains distinct where safety requires it even when
the owner fills all business roles.

Enterprise authorization composition:

```text
effective authority =
entitlement ceiling
∩ resource policy
∩ relationships
∩ contextual ABAC
∩ task capability
∩ fresh identity/attestation
∩ budget/subscription ceiling
```

| Model | Good fit | Main limitation |
|---|---|---|
| RBAC | Stable administrative duties | Role explosion and weak Task context |
| ABAC | Purpose, classification, risk, time | Attribute freshness and provenance |
| ReBAC | Owner, sponsor, delegation, hierarchy | Consistent relationship graph required |
| Capability | Task-scoped delegation and bounded offline use | Theft, replay, and revocation |
| Resource policy | Source-native ACL | Vendor-specific semantics |
| Risk-adaptive | Dynamic reduction of authority | Cannot be a grant source |
| Approval | High-risk exception | Fatigue, replay, and self-approval |

Decision input:

```text
Human + Agent + Workload + Sponsor + Initiator + Delegation chain
+ Organization scope + Resource + Action + Purpose + Task/Goal
+ Environment + Risk + Time + Policy version
+ Attestation + Classification/residency
+ Entitlement/budget + Approval + Capability + Revocation watermark
```

Decision output:

```text
Permit | Deny | RequireApproval
+ reason_codes + obligations + policy_version
+ input_digest + expires_at + evidence_reference
```

- **[RECOMMENDATION]** Missing, stale, or unverifiable required input becomes
  Deny/indeterminate.
- **[RECOMMENDATION]** RequireApproval is not temporary Permit; submit a new
  decision after approval.
- **[FACT]** OIDC/SAML federate humans, SCIM provisions identities, SPIFFE
  identifies workloads, and SSF/CAEP carries security signals. None is a
  complete authorization engine.

## 12. Subscription, Budget, and Cost Model

Personal:

```text
Owner
→ AccessAccount
→ Entitlement / AllowanceWindow
→ AgentBinding
→ UsageObservation
→ BudgetPolicy
→ CostObservation | Unknown
```

Enterprise:

```text
Organization contract
→ Provider tenant/account
→ Seat or entitlement pool
→ Allocation
→ Department/team/project/agent/task
→ Usage attribution
→ Quota and budget
→ Showback/chargeback reference
```

| Type | Product label | Authority |
|---|---|---|
| Plan price | List price | Provider catalog |
| Included allowance | Observed entitlement | Provider first-party source |
| Provider usage | Provider-reported usage | Observation |
| Local usage | Locally measured usage | Observation |
| List-price arithmetic | Estimated cost | Not an invoice |
| Provider accrual | Provider-reported accrual | May be delayed or corrected |
| Invoice/export | Actual invoiced cost | Provider/finance SoR |
| Internal allocation | Showback/chargeback | Enterprise policy |
| No data | `cost_unavailable` | Honest unknown |

| Constraint | CognitiveOS role | Provider role |
|---|---|---|
| Task tool/domain/time/token budget | Hard enforceable | None |
| Local concurrency/workspace/binding | Enforceable | None |
| Provider rolling quota | Observe only | Final enforcement |
| Consumer eligibility | Link/observe | Final enforcement |
| Enterprise seat | Delegated mixed | Final enforcement |
| Spend alert | Advisory until proven otherwise | May expose Provider control |
| Invoice | Reference only | Authoritative |

**[RECOMMENDATION]** Any future claim of a hard monetary budget requires
pre-dispatch reservation and proof against concurrency, retry, cancellation,
and Provider reporting lag.

## 13. Knowledge Governance Model

```text
KnowledgeSource
KnowledgeObject / ExternalRef
KnowledgeVersion
SourceAclSnapshot
Classification / PurposeGrant
Provenance
EligibilityDecision
RetrievalGrant
Derived Chunk / Embedding / Summary
Citation
RevocationTombstone
```

Mandatory flow:

```text
Human/Agent/Workload identity
→ Task purpose and scope
→ policy authorization
→ source eligibility
→ authorized metadata filtering
→ source ACL recheck
→ short-lived retrieval capability
→ body retrieval
→ ranking/context construction
→ DLP/output controls
→ usage evidence
```

- **[RECOMMENDATION]** Never retrieve all bodies and filter afterward.
- **[RECOMMENDATION]** Titles, paths, hit counts, snippets, embeddings, and
  citations may themselves be sensitive.
- **[RECOMMENDATION]** Cache keys include principal, Agent, Task purpose,
  policy version, ACL revision, and source version.
- **[RECOMMENDATION]** Retrieved content is untrusted data, not policy or
  authority.
- **[FACT]** RAG and fine-tuning do not eliminate prompt injection; see
  [NIST AI 100-2e2025](https://csrc.nist.gov/pubs/ai/100/2/e2025/final) and
  [OWASP LLM01:2025](https://genai.owasp.org/llmrisk/llm01-prompt-injection/).

| Existing domain | Knowledge relationship |
|---|---|
| ContextSource | External source registration and version entry |
| ContextRequest | Minimum context need for one Task |
| ContextView | Authorized, filtered, version-pinned consumable view |
| Memory | Approved derived fact, not a source content repository |
| Skill | Executable method/capability package |
| Artifact | Immutable output, evidence, or retrieval product |
| External knowledge system | Body, ACL, retention, and legal-hold SoR |

**[RECOMMENDATION]** Use external source registry + Context governance now.
Do not build a universal copied-content KnowledgeBase.

## 14. Market and Standards Findings

| Category | Representative | Demonstrated capability | Remaining gap |
|---|---|---|---|
| Coding runtime | [OpenHands](https://docs.openhands.dev/openhands/usage/architecture/runtime) | Isolated execution, resource control, workspace | Does not prove Goal, Assignment, or independent acceptance |
| Orchestration | [LangGraph](https://docs.langchain.com/oss/python/langgraph/overview) | Durable graph, persistence, human intervention | Adopter defines authority, policy, and evidence |
| Observability | [LangSmith](https://docs.langchain.com/langsmith/evaluation-concepts) | Traces, evaluation, cost tracking | No Task assignment/completion authority |
| Personal work/AI hub | [Notion Custom Agents](https://www.notion.com/help/custom-agents) | Workspace context, triggers, run log | No cross-runtime Effect governance |
| Enterprise control tower | [ServiceNow AI Control Tower](https://www.servicenow.com/products/ai-control-tower.html) | Inventory, CMDB context, governance | Marketing does not prove a complete Task authority chain |
| Project integration | [Linear Agents](https://linear.app/docs/agents-in-linear) | Issue delegation, human owner, session activity | External Agent still owns execution, cost, and recovery |

- **[INFERENCE]** The market gap is not merely an Agent dashboard. It is a
  durable authority chain across heterogeneous runtimes.
- **[FACT]** Selected public issues about multitasking, resume, missing usage,
  and premature completion are anecdotes, not evidence of frequency or payment.
- **[RECOMMENDATION]** Position Personal as “Agent Work System for one
  operator” and Enterprise as an execution-assurance and governance plane.
- **[RECOMMENDATION]** Keep “AI Workforce OS” as long-range vision language.
- **[FACT]** No final cross-vendor standard defines a complete Agent registry,
  sponsor model, and lifecycle.
- **[RECOMMENDATION]** Distinguish logical Agent, version/deployment, runtime
  workload, and Task delegation identities.
- **[RECOMMENDATION]** Compare OPA and Cedar before an Enterprise policy-engine
  decision. Add OpenFGA only for validated relationship-scale needs.
- **[FACT]** Receiving SSF/CAEP signals does not itself enforce revocation.

## 15. Repository Strategy

| Option | Current disposition | Reason |
|---|---|---|
| Keep current topology | RECOMMEND | Lowest migration cost and clear authority boundary |
| Add explicit boundaries in current crates/packages | RECOMMEND | Supports narrow Work projection, Profile, and Entitlement seams |
| Extract stable protocol/SDK package | DEFER | Wait for a second consumer and stable public API |
| Add Enterprise service repository | DEFER | Wait for independent deployment, release, security ownership, and validation |
| Product monorepo | REJECT now | High migration cost and blurred ownership |
| Separate connector packages | DEFER | Wait for stable interface and real consumers |

Recommendation now:

- Keep `cognitive-os` as contracts, daemon, runtime, CLI, and Personal
  authority.
- Keep `cognitiveos-clients` as the formal Personal client.
- Add narrow application projections/services in the existing topology.
- Do not create an Enterprise repository, kernel repository, or monorepo.

Topology-change triggers:

- independent deployment/scaling boundary;
- independent release cycle or security ownership;
- at least two real external consumers;
- stable public API;
- different license/commercial boundary; or
- measured delivery blockage caused by the current topology.

Versioning:

- Public contracts use additive versioning, capability negotiation, and
  explicit deprecation.
- Private projections may evolve faster but require route whitelists, stub
  detection, and compatibility tests.
- **[FACT]** Some unknown `/management/*` and `/task/*` POST paths can return a
  200 stub; clients must not infer capability from HTTP success alone.

## 16. Opportunity Solution Trees

All opportunities below are **PRODUCT HYPOTHESES**.

### Personal OST

Desired outcome: in a four-week pilot, independently accepted Agent-assisted
Tasks reach at least 90% while median supervision minutes per Task fall at
least 25% from each participant's first-week baseline.

| Opportunity | Solution | Falsifiable experiment |
|---|---|---|
| Multiple runs make owner/state/spend/next action hard to see | Unified Work board | Six users × ten runs; answer key status questions within 60 seconds for ≥90% of samples |
| Same | Exception-only inbox | 50 runs; surface ≥80% of actionable exceptions first; false critical alert ≤1/10 |
| Same | Existing-tracker overlay | 30 eligible Tasks; ≥60% delegated there; duplicate entry <10% |
| Interruption or self-reported completion makes recovery and proof unreliable | Durable resume packet | Interrupt 12 Tasks; ≥80% resume without decision restatement; recovery improves ≥30% |
| Same | Predeclared acceptance + isolated verifier | 20 outputs with ten seeded faults; catch ≥9; false block ≤10% |
| Same | Recovery reconciler | 15 sandbox faults; ≥12 become accepted or honestly blocked; duplicate irreversible Effect = 0 |

### Enterprise OST

Desired outcome: across three design-partner teams, at least 90% of Agent work
is traceable to owner, policy, budget, and evidence while accepted-Task cycle
time increases by no more than 10%.

| Opportunity | Solution | Falsifiable experiment |
|---|---|---|
| Agent work is not linked to outcome, owner, policy, and cost | Goal→Task→Agent ledger | Shadow 50 work items; ≥90% have complete trace |
| Same | Bidirectional project connector | 30 sandbox Tasks; ≥80% sync within five minutes; unauthorized writes = 0 |
| Same | Risk/budget templates | 30 proposals; ≥90% agreement with blinded governance decisions; critical false allow = 0 |
| Risk/Ops lacks runtime-neutral proof and containment | Independent verification service | Replay 40 runs; detect ≥90% of high-severity faults |
| Same | Normalized evidence envelope | Three runtimes; ≥95% event attribution; missing required evidence fails closed |
| Same | Exception control room | 12 chaos drills; all critical deviations contained; execution after authoritative stop = 0 |

## 17. MVP Scope

### Personal P0

1. Agent inventory and eligible Profile.
2. Goal-lite Task creation and explicit Assignment.
3. Wakeup and governed execution.
4. Evidence-backed completion.
5. Recovery and cost-status visibility.

These are one complete path, not five independent administration modules.

### P1

- Parent-child Task and directed blockers.
- Provider entitlement/allowance observation.
- Exception-only supervisor inbox.
- Capability filtering and saved Work views.
- Unified Activity projection.

### P2

- External Goal/Project links.
- Routine-generated Tasks.
- Additional adapter families.
- Provider billing reconciliation.
- Memory/Skill provenance drill-down.

### Deferred

- Independent Project/Workstream lifecycle.
- Generic persisted Run entity.
- Knowledge domain.
- Multi-Agent orchestration.
- Enterprise policy engine, SCIM, and SPIFFE fleet.

### Explicitly out of scope

- Company/CEO simulation.
- Consumer subscription brokerage or cookie capture.
- Full PM/OKR product.
- New repository/monorepo.
- Enterprise v1.
- Agent self-completion.
- Any reduction in verification, recovery, or security quality.

## 18. Personal Candidate PRD

Status: **candidate / owner-driven / non-canonical**.

### Problem

**[PRODUCT HYPOTHESIS]** A technical individual uses several IDE/CLI Agents,
API accounts, and consumer plans for work that spans hours or sessions. Task
state is scattered across chat, terminals, issues, Git, CI, and Provider
dashboards. The user cannot reliably answer who owns the work, why it was
assigned, which effects occurred, what the cost source is, whether acceptance
was met, or how to continue safely.

### Evidence

**[FACT]** CognitiveOS already implements TaskContract, preview/admission,
fenced scheduler lease, hard Task budget, Intent/Effect, independent verifier,
acceptance evidence, Provider binding, and SecretStore. **[FACT]** Assignment,
rich Work inventory, complete Agent projection, and unified Activity are
missing. Public reports from several Agent products show supervision, resume,
usage, and premature-completion friction, but are weak signals. **[OPEN
QUESTION]** No target-user interviews, diary study, paid commitment, or
baseline analytics exist.

### Goals

- Create, preview, admit, and assign one bounded Task from user intent.
- Let the user identify Assignment, trigger, authority phase, blocker, cost
  state, and next action within 60 seconds.
- Never let Agent, process, or Provider success bypass verification.
- Resume from durable state instead of restating chat context.
- Prove value with one owner, workspace, and qualified Agent path.

### Target user

**[PRODUCT HYPOTHESIS]** A technical individual using at least two AI
coding/research tools weekly, already relying on Git, issues, CI, or human
review, and sensitive to credentials, cost, and recovery quality.

### Requirements

P0:

1. Goal-lite intent reaches exact preview before daemon admission.
2. The user selects a real eligible Agent; Assignment and lease remain
   separate.
3. Wakeup enters the existing scheduler and requires a fenced lease.
4. Work detail separates candidate, process observation, Intent/Effect,
   verifier, and acceptance.
5. Interruption, budget stop, unavailable secret, verification failure, and
   stale Assignment retain a durable reason, next action, and cost provenance.

P1: blockers, parent-child decomposition, entitlement observation, exception
inbox, and Activity aggregation.

P2: external Goal/Project links, routines, more adapters, and invoice
reconciliation.

### Success metrics and counter-metrics

| Metric | Counter-metric |
|---|---|
| ≥60% of eligible pilot work creates Task/Assignment before execution | Manual administration <2 minutes/Task |
| ≥90% of terminal attempts are accepted, failed, or honestly blocked | Unverified Tasks labeled done = 0 |
| ≥80% of interrupted Tasks resume without decision restatement | Duplicate irreversible Effects = 0 |
| Median supervision time falls ≥25% | Accepted-Task cycle time rises ≤10% |
| ≥95% of variable spend is attributable | Estimates presented as invoices = 0 |

These are proposed pilot thresholds, not observed outcomes.

### Out of scope and open questions

No multi-user IAM, company metaphor, consumer-plan purchase/custody, complete
Project/OKR, generic KnowledgeBase, multi-Agent negotiation, new runtime,
Paperclip embedding, or Gate promotion.

Open questions include whether users prefer a unified Work board or tracker
overlay, accept preview/admission friction, need Assignment as a public
contract, and will pay for supervision, recovery, or trustworthy acceptance.

## 19. Enterprise Discovery Brief

Target users: IAM/platform/security operators, AI governance/risk, business
sponsors, Knowledge owners, FinOps, and incident operators.

Governance jobs:

- Agent discovery, owner/sponsor, lifecycle, and access review.
- Human, Agent, workload, and Task-delegation identity.
- Goal linkage, policy, approval, evidence, and accountability.
- Provider entitlement, allocation, and usage attribution.
- Knowledge authorization, DLP, residency, and retention.
- Incident containment and fleet observability.

| System | CognitiveOS may consume or emit | Must not reimplement |
|---|---|---|
| IdP/HRIS | Issuer-subject, groups, active state, organization/sponsor refs | Password, MFA, payroll, employee master |
| Secret Manager | SecretRef, version, lease | Secret value and rotation engine |
| SIEM/DLP | Structured events and policy/risk signals | Correlation, case management, classifier |
| FinOps/billing | Usage, allocation, invoice refs | Tax, GL, commercial pricing contract |
| Jira/Linear/GitHub/Asana | Object links, webhook cursor, evidence backlink | Boards, sprints, full PM |
| DMS/catalog | Source/version/ACL/classification refs | Canonical body and legal hold |

Entry criteria:

1. Three target enterprises validate the JTBD/accountability model.
2. One design partner supplies a real system topology.
3. System-of-record and field ownership are approved.
4. Permission contract, hard deny, and revocation SLO are frozen.
5. Node single-writer protocol is proven.
6. Policy-engine bakeoff passes a real policy corpus.
7. Knowledge stale-ACL, metadata-leakage, prompt-injection, and revocation
   tests pass.
8. Entitlement unit and cost attribution are understood.
9. Personal has no mandatory Enterprise dependency.

Enterprise should remain shadow-mode discovery until those conditions hold.

## 20. Domain Delta Map

| Domain | Decision | Reason |
|---|---|---|
| Task / TaskContract | KEEP | Authority core |
| Intent / Effect / Evidence / Verification | KEEP | Non-negotiable correctness |
| Scheduler lease/fencing/Task budget | KEEP | Execution ownership |
| Work projection | NEW | Aggregate facts without becoming authority |
| Assignment | NEW | Typed/versioned Task↔Agent relation |
| Agent Profile | NEW projection | User-facing purpose/capability |
| Agent Instance/health | EXTEND | Derive from existing records |
| Run | NEW projection; persisted entity DEFER | Validate semantics first |
| Goal | EXTEND Goal-lite; lifecycle DEFER | Progress comes from accepted evidence |
| Project/Workstream | DEFER | Insufficient need evidence |
| Parent-child/blocker | NEW narrow relations | Separate decomposition and dependency |
| Provider Access | KEEP/EXTEND | Separate account/auth/entitlement |
| Provider mutation/audit | REFACTOR | Durable idempotency and atomic audit |
| Universal Subscription | REJECT | Cannot truthfully unify consumer/API/seat/PAYG |
| Usage/CostObservation | EXTEND | Source, freshness, estimate, invoice |
| Context/Memory/Skill/Artifact | KEEP/EXTEND | Governed resource/evidence view |
| KnowledgeBase | DEFER / REJECT generic copy | External registry first |
| Unified Activity | EXTEND | Separate authority and observation |
| Enterprise Organization/Policy/Fleet | DEFER | Discovery only |
| Lease as Assignment | REJECT | Responsibility differs from current execution right |
| Agent self-completion | REJECT | Conflicts with A1/A4 |

## 21. ADR Candidate Register

All entries are **Candidate**, not Accepted.

| Candidate | Decision question | Options | Recommendation | Consequence | Reversibility |
|---|---|---|---|---|---|
| Paperclip-inspired work model | Adopt overall model? | Embed / copy / selective / reject | Selective adaptation | Reuse orchestration concepts without runtime | High |
| Goal/Project/Task | Independent Goal and Project now? | Full / Goal-lite / external-only | Goal-lite; defer Project | Narrow MVP | Medium |
| Wakeup semantics | Can heartbeat execute work? | Second scheduler / trigger / direct adapter | Trigger only | Preserves single authority | High |
| Profile vs Instance | One Agent row? | Unified / split | Conceptual split | More accurate projections | Medium |
| Provider Access | Universal Subscription? | Universal / Entitlement / provider-only | Entitlement + AllowanceWindow | Honest commercial semantics | Medium |
| Run entity | Persist now? | Entity / projection / none | Projection first | Lower schema risk | High |
| Enterprise boundary | May central plane write local state? | Central / shared DB / node writer | Signed request + node writer | Availability tradeoffs under partition | Low |
| Knowledge scope | New KnowledgeBase? | Copy / registry / external-only | Registry + Context governance | Less content duplication | Medium |
| Repository boundary | Split now? | Keep / package / new repo / monorepo | Keep | Delay migration | High |
| Assignment authority | Who writes; public contract? | Private / public / lease-derived | Daemon-written typed relation | Requires CAS/epoch/audit | Medium-low |
| Cost truth taxonomy | One cost field? | Single / source-typed | Estimate/accrual/invoice/unavailable | Honest UI/contracts | Medium |

## 22. API / Contract Gap Register

| Current capability | Product need | Projection/service | Mutation | Lane-CTR | Candidate acceptance |
|---|---|---|---|---|---|
| Task list is a thin envelope | Rich Work list/detail | NEW read projection | None | Only for public semantics | Objective/state/Assignment/evidence query |
| Task propose/preview/admit | Goal-lite launch | Reuse | Existing route | Usually no | Preview digest matches admitted epoch |
| No Assignment API | Typed Assignment | NEW service | Daemon CAS | Likely | Exact epoch, eligible binding, conflict fails |
| Scheduler library-only | Wakeup visibility | Projection + narrow command | Daemon enqueue | Public trigger may require | Durable reason, duplicate idempotent |
| Attempt facts fragmented | ExecutionAttempt view | Derived projection | None | Not initially | Process/candidate/verifier separated |
| Evidence/Effects HTTP | Unified receipt | Aggregate | None | No | Missing data is unknown, not success |
| Runtime projection empty | Profile/Instance/health | EXTEND | Lifecycle deferred | Public lifecycle only | Source and freshness visible |
| Provider APIs | Entitlement/cost provenance | Connector projection | Delegated only | New public shape may | No secret; source/freshness explicit |
| Provider direct mutation | Durable idempotency/audit | REFACTOR | Intent/Effect where external | Semantic change | State and audit atomic |
| Context pins | Resource-use explanation | Context summary | None | Usually no | Only authorized refs/version |
| Memory/Skill lifecycle | Usage projection | Aggregate | Existing | Public Skill deferred | Task-bound provenance |
| Fragmented audit | Unified Activity | Append/query projection | Emitter refactor | Public event may | Transition atomically traceable |
| Approval library-only | Decision inbox | Future service | Exact-once | Public approval requires | Digest/version/expiry bound |
| Unknown 200 stubs | Capability honesty | Whitelist | None | No | Unsupported is explicit |

## 23. Threat Model Delta

| Threat | Primary control | Residual risk |
|---|---|---|
| Subscription credential leakage | Native auth, approved SecretStore, no cookie/password/token import | Same-user runtime session files |
| Account/session confusion | Pin account/auth/model and display billed identity | Provider may not expose immediate proof |
| Capability overclaim | Separate declaration, evidence, health, and policy | Version behavior drift |
| Duplicate execution | CAS lease, epoch/fence, idempotency, sink fence | External sink may lack fencing |
| Stale Assignment | Bind Task epoch and Instance/Binding version | More fail-closed results |
| Runaway scheduled work | Routine→Task, catch-up cap, coalescing, budget | Already-incurred Provider spend |
| Budget bypass | Hard Task budget, reservation, retry accounting | Unknown pricing prevents monetary guarantee |
| Knowledge cross-scope leakage | Pre-retrieval auth, partition, ACL recheck, DLP | Embedding revocation complexity |
| Prompt injection | Untrusted data, deterministic PEP, allowlist, purpose binding | Legitimate-looking harmful action |
| Agent self-completion | Candidate-only, independent verifier, daemon acceptance | Verifier defects |
| Forged usage/cost | Source-typed observation and Provider reconciliation | Delayed/corrected reports |
| Central/local authority conflict | Signed bundles, node evaluation, monotonic expiry | Partition reduces availability |
| Revoked identity continues | Durable signal, short TTL, watermark, quarantine | Propagation window remains |
| Approval replay/self-approval | Digest, expiry, single-use, separation | Approver compromise |
| Unknown worktree overwrite | A8 fail closed and explicit ownership | Human intervention required |
| Evidence omission | Atomic outbox, sequence, signed export | Node compromise needs independent source |

## 24. Candidate Development Task

Candidate ID: **`CAND-AWS-PERSONAL-001`** — unregistered and not a formal
`P*-T*` task.

| Field | Candidate content |
|---|---|
| User outcome | Assign one bounded Task to a real Agent and inspect admission-to-acceptance facts in one Work detail |
| Problem | Strong Task authority exists, but Assignment, rich Work projection, and Agent/attempt visibility do not |
| Dependencies | Active P7-T05 closure/transfer; clean ownership; qualified Agent path; ADR dispositions; Lane-CTR assessment |
| In scope | Goal-lite Task, one Agent projection, typed Assignment, wakeup, existing scheduler, one Agent path, evidence projection, cost status |
| Out of scope | Project, routine, multi-Agent, Enterprise, consumer subscription management, generic Run schema, KnowledgeBase, new repository |
| Acceptance | Exact preview/admit, Assignment CAS, stale binding failure, fenced execution, state separation, accepted receipt, durable recovery |
| Failure-first negatives | Duplicate Assignment, stale epoch, unhealthy Agent, wrong account, duplicate wakeup, expired lease, self-completion, missing verifier, unknown cost/worktree |
| Validation | Local Node/TS and Rust fmt; CI Ubuntu/Windows; exact Linux revision; daemon-served browser journey |
| Candidate paths | `crates/cognitive-store/`, `crates/cognitive-kernel/`, `apps/kernel-server/src/personal/`, focused tests, conditional contracts/bindings, client Work/data paths |
| Docs-sync | Source-map routing, bilingual handbook, generated fingerprints when implementation exists |
| Risks | Lease collision, over-generalized Assignment, projection cost, capability overclaim, private API drift |
| Non-claims | No market, Enterprise, release/Profile, quota, invoice, or multi-Agent claim |

## 25. First Vertical Slice

### Single-Agent Assigned Task to Verified Result

```text
User enters Goal-lite intent
→ existing propose/clarify/preview/admit creates a real Task
→ user selects one registered, healthy, bound Agent
→ daemon persists Assignment against Task epoch
→ Assignment creates durable wakeup
→ scheduler acquires fenced lease
→ existing Agent path produces candidate and governed Tool request
→ Intent/Effect persists and reconciles
→ independent verifier evaluates acceptance
→ Work detail shows accepted result, evidence, Effect, and cost status
```

Why first:

- **[FACT]** Reuses existing Task, scheduler, adapter, Tool, Effect, and
  verifier rather than building a runtime.
- **[RECOMMENDATION]** Closes the user loop: who owns it, what happened, was it
  accepted, and what proves it.
- **[RECOMMENDATION]** Avoids Project, routine, multi-Agent, Enterprise,
  Knowledge, and consumer OAuth.
- **[RECOMMENDATION]** Tests whether Assignment deserves a first-class domain
  while keeping Run as a projection.
- **[FACT]** Candidate, process exit, and Provider response cannot complete the
  Task.

Implementation prerequisites:

- Close or formally transfer the active overlapping lease.
- Confirm a clean client ownership state.
- Select a production-proven Agent path; do not substitute a fixture and claim
  production capability.

## 26. Acceptance and Validation Plan

| Acceptance | Failure-first negative | Validation route |
|---|---|---|
| Preview matches admitted contract/epoch | Stale preview rejected | Focused Rust tests; CI Ubuntu/Windows |
| Assignment binds only eligible Agent | Unhealthy/revoked/wrong account fails | Store/service tests; exact Linux |
| Assignment differs from lease | Assignment grants no direct state write | Authority negatives |
| Duplicate assign/wakeup is idempotent | Competing trigger does not double-run | Concurrency tests |
| Scheduler enforces epoch/fence | Stale worker Effect rejected | Integration tests |
| Intent precedes dispatch | No mutation without Intent | Failure-first integration |
| Agent output is candidate | Self-reported done does not complete | Completion-authority tests |
| Verifier is independent/required | Missing verifier fails or blocks | Verifier tests |
| Work detail is semantically honest | Process/candidate/accepted separated | Frontend component/browser |
| Cost source is honest | Estimate not shown as invoice | API/UI tests |
| Recovery is durable | Restart preserves next action | Exact-revision Linux |
| Secrets stay isolated | No secret in browser/API/log | Redaction/network tests |
| Capability is honest | 200 stub not treated as supported | Frontend network negatives |
| A8 protection holds | Unknown dirty workspace fails closed | Controlled integration |

Environment constraints:

- **[FACT]** `DEV-WIN-GNU-01` must not run Rust build/test/Clippy/link. It may
  run Rust fmt, Node/TS, static, and documentation checks.
- Rust validation routes through `CI-UBUNTU-01`, `CI-WINDOWS-MSVC-01`, or
  pushed exact revision on `DEV-LINUX-NATIVE-01`.
- Frontend validation requires unit, network normalization, component/a11y,
  deterministic build, and daemon-served `/ui/` browser journey.
- **[FACT]** No candidate acceptance above was executed in this research.
- Ordinary CI/native evidence cannot be promoted to release, Profile, or Gate.

## 27. Development Readiness Gate

Overall result: **PARTIALLY READY**

| Gate item | Result | Reason |
|---|---|---|
| Problem sufficiently defined | PARTIAL | Structure is clear; frequency, loss, and payment evidence are missing |
| User and JTBD defined | PARTIAL | Users/jobs are defined as hypotheses |
| Current capability known | PASS | Specified/implemented/HTTP/test/Gate states are separated |
| Product boundary decided | PASS as candidate | Personal, Enterprise, and non-goals are clear |
| Architecture boundary decided | PASS as candidate | Node sole writer; no second authority |
| Domain conflicts resolved | PARTIAL | Assignment, Goal-lite, Run, and Entitlement await disposition |
| Contract changes identified | PASS | Private projection and Lane-CTR boundaries are listed |
| Threat model complete enough | PASS for first slice | Required threats have controls and residual risk |
| First slice bounded | PASS | One owner, Task, and Agent path |
| Acceptance testable | PASS | Negatives and supported routes are mapped |
| Validation environment available | PASS | CI, exact Linux, and frontend routes exist |
| Repository ownership clear | PARTIAL | Repository boundary is clear; active overlapping work remains |
| No collision with active task | FAIL | P7-T05/D12 overlaps future client surfaces |

Readiness layers:

- Product discovery: **READY**.
- Pre-development shaping: **READY WITH CONDITIONS**.
- Immediate implementation: **NOT READY**.
- Overall: **PARTIALLY READY**.

Minimum blockers:

1. Product owner completes at least five recent Personal switcher/evaluator
   interviews, preferably eight.
2. The active P7-T05 owner closes or transfers overlapping work and confirms
   clean ownership.
3. Product/architecture and Lane-CTR owners disposition Assignment authority,
   Goal-lite, Run projection, and Entitlement/cost taxonomy.

Enterprise validation is not a blocker for the Personal first slice;
Enterprise implementation remains deferred.

## 28. Final Decision Card

1. **Personal core loop?**
   **[RECOMMENDATION]** Intent/Goal-lite → Task → Assignment → governed
   execution → independent verification → daemon acceptance → evidence →
   continuation.
2. **What to absorb from Paperclip?**
   **[RECOMMENDATION]** Work-centered operation, Goal/Task traceability,
   blockers, checkout, wakeup reasons, attempt timeline, routine→Task, budget
   admission, revision-bound confirmation, and orphan recovery.
3. **What does not fit Personal?**
   **[RECOMMENDATION]** Company/employee/CEO/hiring/reporting-line metaphor,
   self-completion, exit=success, second scheduler, insecure secret delivery,
   and arbitrary-process trust.
4. **Personal Agent objects?**
   **[RECOMMENDATION]** Package, Installation, Adapter, Registration, Profile,
   Instance, Runtime Process, Sidecar Session, Agent Session, and Execution.
5. **What does “subscription management” manage?**
   **[RECOMMENDATION]** Provider, AccessAccount, Auth/SecretRef, Entitlement,
   AllowanceWindow, Model snapshot, Binding, Usage, Budget, CostObservation,
   and billing link.
6. **Which plans are only linked/observed?**
   **[FACT]** Most Claude, ChatGPT/Codex, Antigravity, Cursor consumer,
   OpenCode Go, and Kiro plans. Only explicit delegated APIs may be managed.
7. **Goal, Project, Task, Run?**
   **[RECOMMENDATION]** Goal is initially an outcome anchor; Project deferred;
   TaskContract remains authority; Run begins as ExecutionAttempt projection;
   progress comes from accepted evidence or external SoR.
8. **Enterprise versus Personal?**
   **[RECOMMENDATION]** Personal is single-owner local operations. Enterprise
   adds federation, sponsor, policy, allocation, knowledge governance,
   approval, incident, and fleet projection.
9. **How does Enterprise integrate existing systems?**
   **[RECOMMENDATION]** Through references, signed projections, events, and
   connectors while external systems retain authoritative facts.
10. **How is implicit Knowledge escalation prevented?**
    **[RECOMMENDATION]** Authorize identity, purpose, source ACL,
    classification, and scope before body retrieval; recheck ACL and propagate
    controls into cache/embedding.
11. **Central versus node responsibility?**
    **[RECOMMENDATION]** Central authors/distributes policy and requests; node
    owns Task, Intent, Effect, lease, dispatch, verification, and evidence.
12. **New repository now?**
    **[RECOMMENDATION]** No. Wait for independent deployment, release,
    consumers, security ownership, or measured delivery blockage.
13. **Five Personal P0 items?**
    **[RECOMMENDATION]** Agent inventory/Profile; Goal-lite Task+Assignment;
    wakeup/governed execution; evidence-backed completion; recovery+cost state.
14. **Next candidate task?**
    **[RECOMMENDATION]** Unregistered `CAND-AWS-PERSONAL-001`:
    Single-Agent Assigned Work to Evidence-backed Result.
15. **First vertical slice?**
    **[RECOMMENDATION]** Create/admit a Task, assign a qualified Agent, execute
    through scheduler/lease/Intent/Effect/verifier, and show accepted evidence
    and recovery in Work detail.
16. **Ready for development?**
    **[RECOMMENDATION]** PARTIALLY READY; discovery/shaping is ready, immediate
    implementation is not.
17. **What must not be done now?**
    **[RECOMMENDATION]** Do not overlap active P7 work; create a new
    repo/monorepo; implement Enterprise; embed Paperclip; capture consumer
    credentials; create universal Agent/Subscription/KnowledgeBase entities;
    accept self-completion; or weaken A1–A8, Intent/Effect, fencing, budget,
    SecretStore, verification, and A8.

## Source and claim note

**[FACT]** This package combines a read-only repository audit, official
Paperclip and Provider documentation, Enterprise standards/vendor material,
and a bounded market survey. Public issues are weak anecdotal evidence;
marketing pages are not implementation proof. No service, migration,
dependency, secret, artifact, branch, commit, PR, or product code was created
or changed by the research.
