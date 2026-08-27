# CognitiveOS Personal product design

- Status: canonical Personal 2.0 product intent and product acceptance
- Change class: product-semantic
- Current release boundary: [Linux 1.0 scope](linux-1.0-scope.md)
- Exact Personal 2.0 inclusion and capability status:
  [Personal 2.0 scope](personal-2.0-scope.md)
- Ordered behavior and simulated acceptance:
  [User journeys](user-journeys.md)

## 1. Document authority and evidence boundary

This document owns Personal product intent, concepts, outcomes, information
architecture, and product acceptance. The Personal 2.0 scope owns exact
release inclusion and capability status. User journeys own ordered interaction,
failure, and recovery behavior. These documents link to one another instead of
maintaining parallel reality ledgers.

Personal 2.0 is a **full product-version commitment**. That commitment is not
implementation, Gate, release, Profile, performance, containment, market,
usability, adoption, or Agent-benefit evidence. Every adopted target remains
capability-gated and **Requires-backend** until its exact capability exists.
This document defines no schema, route, API, database, or implementation
architecture.

The product has no external human research evidence for frequency, intensity,
existing workarounds, willingness to pay, adoption, retention, or usability.
The jobs, JTBD, Forces of Progress, and Opportunity Solution Tree in this
document are hypotheses. No validation score or numerical RICE score is
claimed.

AI-window evaluation may establish simulated product understanding and
scenario acceptance for visible state, recovery, provenance, authority, and
non-claims. It cannot establish human desirability, usability, adoption,
willingness to pay, problem-solution fit, or release/Gate technical evidence.

The Rust daemon remains the sole authority writer. Agents, adapters, the global
Agent Shell, UI, CLI, sidecars, native applications, and MCP servers may
propose, explain, or observe; they do not authorize, commit Effects, reconcile
outcomes, or accept completion.

## 2. Product definition, ICP, trigger, and outcome

CognitiveOS Personal 2.0 is the cross-platform local stewardship layer for one
owner's Agents, accounts, cognitive resources, and governed work. It targets
Windows, macOS, and Linux as independently qualified local product platforms.
It is not a distributed authority, remote public administration service,
multi-user system, or launcher that trusts every Agent independently.

The initial user hypothesis is one owner who operates or is onboarding multiple
local Agent products, Provider accounts, reusable cognitive resources, and
consequential governed work. The initial Agent set is:

1. the exact qualified Pi path;
2. DeepSeek Harness Developer Preview from
   [`deepseek-ai/deepseek-harness`](https://github.com/deepseek-ai/deepseek-harness);
3. the Codex experience in the current official ChatGPT desktop app, only on
   its officially supported platforms.

The trigger hypothesis is the point at which native Agent work, accounts,
resources, permissions, or recovery state can no longer be managed honestly as
one isolated conversation or application session.

### Primary outcome — unified resource stewardship

For one declared owner-local inventory, Personal lets the owner identify,
connect, inspect, govern, recover, or explicitly disposition every eligible
Agent, account, resource binding, and governed-work item without confusing
native output with daemon authority.

```text
unified stewardship completion rate
  = passed eligible stewardship cases
  / all eligible stewardship cases in the frozen evaluation manifest
```

A case passes only when the product-visible result states the exact object
identity, current or explicitly unknown/stale state, provenance and authority
boundary, and one valid next action or explicit unavailable reason.
Consequential history and prior attempts remain preserved. The fixed simulated
product-acceptance denominator is eight scenarios; the target is **8/8**. A
partial, failed, or not-run scenario does not enter the numerator.

## 3. Reality and release boundary

| Boundary | Product truth |
|---|---|
| **Current implementation (Now)** | Linux 1.0 is six-family and Pi-qualified. Its daemon-served Control Plane is additive; Activity is a bounded composition and the native dsh panel remains separate. |
| **Personal 2.0 full-version target** | Cross-platform local stewardship on independently qualified Windows, macOS, and Linux product paths; exact Pi, DeepSeek Harness Developer Preview, and supported-platform Codex desktop paths; embedded conversations; Goal -> Plan revision -> Task -> Attempt; multi-Agent work; Account Hub; federated resources; unified Activity; and seven families including MCP. |
| **Requires-backend** | Agent/account lifecycle, common conversation/history, Goal/Plan/Attempt orchestration, controls, multi-Agent handoffs, authority-backed Context/Runtime inventory, unified Activity, federated synchronization, broader Account Hub methods, Global Agent Shell, and MCP family management are not complete Personal capabilities today. |
| **Requires-core (conditional)** | A new or changed public machine surface requires the separately governed contract decision. Personal-private product projections may not. No public shape is implied here. |

Each platform and Agent has an independent capability and qualification
statement. Connection, an adapter, a Provider account, a model, a CLI, an MCP
bridge, or evidence from another platform transfers no qualification.

The exact inclusion matrix, platform boundary, and capability truth live in
[Personal 2.0 scope](personal-2.0-scope.md). Current task, Gate, release, and
environment status remain owned by `PROGRESS.md` and the formal plan.

## 4. JTBD and working-mode hypotheses

### Primary JTBD hypothesis

> When I operate several native Agents, accounts, reusable resources, and
> consequential outcomes, I want one owner-local stewardship layer to show
> what exists, what is authoritative, what changed, and what I can safely do
> next, so I can continue, supervise, recover, and accept work without losing
> native ownership or mistaking Agent output for completion.

The owner moves among three working modes without changing authority:

- **conversation:** create or continue native work and request governance when
  the outcome must become durable or consequential;
- **supervision:** follow Goals, Plan revisions, Tasks, attempts, handoffs,
  Activity, evidence, cost, conflicts, and next actions;
- **stewardship:** manage Agent identity and lifecycle, accounts, permissions,
  resources, MCP, synchronization, workspace, backup, recovery, and updates.

Conversation and supervision are working modes inside the unified stewardship
outcome. Stewardship owns continuity across them; none creates a second
authority writer or a separate product personality.

The candidate task outcomes remain hypotheses:

1. reach one real native Agent response with declared prerequisites and timing
   basis;
2. connect or install an exact Agent through a bounded onboarding path;
3. move useful native work into governed and independently verified work;
4. supervise one Goal across sessions and Agents;
5. explain change and recover without erasing attempts;
6. steward native and Personal resources without losing origin ownership;
7. manage accounts, routing, usage, and cost without exposing secrets;
8. diagnose and restore service when an Agent, Provider, resource, MCP server,
   native surface, or observation source is unavailable.

No frequency, intensity, workaround, or willingness-to-pay score exists for
these hypotheses. The hypothesized adoption forces are fragmented native state
and uncertain completion as Push; unified local stewardship as Pull; permission,
secret, migration, and reliability risk as Anxiety; and independently usable
native applications as Habit.

## 5. P0 release outcomes and product acceptance

All five outcomes are Personal 2.0 release blockers. Dependency order does not
make any outcome optional.

### P0-1 — Qualified Agent and account stewardship

Personal stewards the exact initial Agent set and its accounts without merging
Agent, Provider, account, model, installation, instance, session, or process
identity.

Acceptance signals:

- Pi, DeepSeek Harness, and Codex desktop each show exact source, product,
  version, platform, capability coverage, health, and qualification boundary;
- each supported Agent can complete its declared connect/install, first real
  response, inspect, recovery, and lifecycle disposition path;
- Account Hub separates subscription/OAuth, API key, approved credential
  import, custom endpoint, Provider reachability, model availability, usage,
  quota, and cost;
- global, Agent, and conversation routing scopes are explicit; a current
  session changes only through an explicit rebind or restart;
- removal distinguishes **Disconnect** from **Uninstall** and states what is
  retained, removed, unknown, or incomplete.

### P0-2 — Conversation-to-governed work

Personal embeds adapter-backed native conversations and admits selected work
into the durable hierarchy:

```text
Native Conversation -> Goal -> Plan revision -> Task -> Attempt
```

Acceptance signals:

- a native conversation remains Native until the owner chooses
  **Manage with Personal** and confirms the exact daemon preview;
- admission creates one persistent Goal, a daemon-owned Plan revision, one or
  more bounded Tasks, and a first attempt under each started Task;
- Agent-authored plans remain Native candidates until admitted;
- retry or fork creates a new attempt and never erases prior failure or
  evidence;
- only current independent verification and daemon acceptance produce Verified
  completion.

### P0-3 — Multi-Agent supervision and unified Activity

Personal supervises daemon-admitted multi-Agent work through explicit handoffs
and one source-labelled Activity reading.

Acceptance signals:

- one Goal may use multiple independently qualified Agents without allowing an
  Agent to transfer authority to another;
- each handoff states source, target, bounded work, current authority, and
  blocked or ready disposition;
- Activity separates **Native / Observed / Governed / Verified** facts and
  declares source coverage;
- missing or disconnected sources produce partial, stale, unknown, or not-run
  facts rather than inferred progress;
- the Global Agent Shell explains and proposes but never holds authority,
  dispatches ambient tools, or accepts completion.

### P0-4 — Seven-family and federated resource stewardship

Personal 2.0 stewards seven independent families: Memory, Skill, Tool, Context,
Task, Runtime/Process, and MCP.

Acceptance signals:

- each family retains its own identity, lifecycle, retention, permission, and
  failure semantics;
- MCP manages server source, installation, health, permission, update, and
  compatible-client projection without becoming a Tool alias;
- MCP connection grants no Tool, Context, workspace, model, or host-session
  authority;
- vendor-native content remains origin-owned while Personal owns admitted
  bindings, permissions, synchronization intent, and authority records;
- synchronization declares origin and coverage, and every conflict fails closed
  until the owner confirms an exact daemon preview.

### P0-5 — Recovery, controls, and continuity

Personal preserves work and authority truth across interruption, failure,
restart, stale observation, and recovery.

Acceptance signals:

- the product exposes only controls the current daemon capability genuinely
  supports: interrupt, pause/resume request, cancel, detach, retry/fork,
  runtime restart/recover, or a defined compensation;
- detach changes observation only; process exit is not cancellation, recovery,
  or completion;
- an unknown Effect enters reconciliation and is never blindly redispatched;
- retry/fork preserves every started attempt, evidence, and failure;
- irreversible work never promises rollback, and a compensation is shown only
  when the daemon defines one.

## 6. Agent and account scope

### Initial Agent claim set

| Agent product | Exact Personal 2.0 target identity | Independent claim boundary |
|---|---|---|
| **Pi** | the exact acquired Pi package and Personal sidecar path | Linux 1.0 evidence qualifies only its exact Pi path; Personal 2.0 capabilities and every additional platform remain independently qualified |
| **DeepSeek Harness** | [`deepseek-ai/deepseek-harness`](https://github.com/deepseek-ai/deepseek-harness), explicitly labelled **Developer Preview** | exact source revision, profile, session, capability, lifecycle, recovery, negatives, and platform; not a DeepSeek model or Provider qualification |
| **Codex desktop** | the Codex experience in the current official ChatGPT desktop app | only officially supported desktop platforms enter exact scope; Codex CLI, web, IDE, account, model, or Provider evidence does not qualify the desktop product; no Linux Codex desktop is implied |

Windows, macOS, and Linux are Personal 2.0 target platforms, but support is
declared per platform and per Agent. Cross-platform local means each supported
installation keeps local owner authority; it does not imply cloud authority,
public remote administration, or one platform's evidence covering another.

Agent connection establishes one explicit observation scope. There is no
speculative global scan or surprise per-session enrollment. Native applications
remain independently usable. Onboarding retains the target path:

1. choose an exact signed upstream record or **Connect existing**;
2. review Provider, workspace, observation, and permission scope once;
3. open the native conversation; ready means one real response arrived.

Any timing, count, or onboarding-step result reports its start event,
prerequisites, eligible denominator, environment, and not-run cases.

### Account Hub

Account Hub is the Personal 2.0 target for Provider presets, custom
OpenAI-compatible endpoints, subscriptions/OAuth, API keys, approved
user-directed credential import, routing scopes, current-session rebind,
usage, quota, cost, and recovery. Current custom OpenAI-compatible
account/endpoint support does not imply the broader target is complete.

Secret material stays in approved Secret Stores and daemon-mediated proxy
profiles. It never appears in Agent configuration, ordinary config, SQLite,
argv, environment, logs, Context, Memory, evidence, browser storage, or chat.

Non-normative product-behavior reference:
[`jlcodes99/cockpit-tools`](https://github.com/jlcodes99/cockpit-tools) is a
cross-platform local AI-application account manager whose published behavior
includes account rosters, account switching, quota/reset visibility, explicit
application paths, and isolated multi-account application instances. It is not
a requirements source, implementation dependency, authority model, credential
boundary, or qualification source for Personal.

## 7. Resource and authority model

### Family model

| Family | Product responsibility |
|---|---|
| Memory | admitted durable knowledge with scope, provenance, versions, conflicts, expiry, forget, and tombstone |
| Skill | immutable instruction/resource/script packages with revision and enablement policy |
| Tool | registered governed operations with explicit availability |
| Context | authorized and budgeted Task input with explicit omissions, losses, and deltas |
| Task | raw intent, preview, admission, bounded execution, checkpoint, Effect, and verification |
| Runtime/Process | separate package-through-execution identities and daemon-owned process observation |
| MCP | server source, installation, connection, health, permission, update, quarantine, and compatible-client projection |

Linux 1.0 remains six-family; MCP is a Personal 2.0 target and
**Requires-backend**. Budget, Permission, Model, Artifact, Intent/Effect,
Evidence, and Event remain cross-cutting objects rather than additional
families.

Content and connection never imply permission. Installing an Agent, enabling a
Skill, selecting a model, discovering a native resource, or connecting an MCP
server grants no runtime capability. Workspace, process, network, Memory,
model, MCP, and write-back scopes remain separate and revocable.

Origin owns native content; Personal owns admitted governance. Automatic
observation is limited to the exact connection scope. Every write-back is a
daemon-owned Intent/Effect mutation. New, broader, destructive, or conflicted
scope requires preview and confirmation. A conflict never resolves by
timestamp or model judgment alone.

Standard Workspace and bounded Extended Home remain established boundaries. A
Goal may span workspaces only through explicit bounded entries. Federated
resources never widen filesystem or network scope.

## 8. Conversation and governed-work model

Native behavior survives integration. Vendor-specific adapters preserve the
native harness, expose capability coverage, and retain vendor extension facts.
Vendor-native conversation identifiers remain opaque origin bindings.

**Manage with Personal** is the consequential boundary. The daemon previews the
exact Goal, Plan revision, Tasks, Agent assignments, Context, workspace,
permissions, budget, external Effects, and acceptance criteria. The owner
confirms that preview once; broader or changed scope receives a new preview.

The daemon owns Plan revisions, Task authority, attempts, and multi-Agent
handoffs. A Goal may span sessions, Agents, and Tasks. A Task is the bounded
authority unit. Each attempt belongs to one Task. A composed **execution flow**
is a product reading and does not by itself create a new authority object.

Agent final text, Tool result, Provider response, native harness success, or
process exit remains an observation. Fluent text never becomes authority by
presentation.

## 9. Activity, controls, and recovery

Unified Activity is the target source-labelled reading:

| Label | Meaning |
|---|---|
| **Native** | originated in a native Agent application or session |
| **Observed** | seen by an adapter or daemon but not admitted as authority |
| **Governed** | daemon admission, authorization, mutation, and Effect reconciliation |
| **Verified** | current independent verification and daemon acceptance only |

These labels describe provenance and authority, not confidence or linear
progress. Counts, percentages, rates, and ETAs appear only with a declared
denominator and basis.

The Global Agent Shell may explain current state, conflicts, missing
capability, and recovery choices; propose one next action; and request the
authoritative daemon preview. It never holds authority or silently widens
scope.

Empty, loading, partial, stale, permission, disconnected, error, success, and
long-running states are first-class product truths. If a control is absent, the
product states that absence rather than presenting a fake capability.

Recovery reloads durable facts, fences stale work, reconciles unknown Effects,
reauthorizes current policy, and rebuilds current Context before dispatch.
Support facts are redacted and distinguish known, unknown, stale, and not-run.

## 10. Hypothesis Opportunity Solution Tree and IA trace

The desired outcome is the unified stewardship completion rate defined in §2.
The tree is a hypothesis tree because there is no external human research.

| Hypothesized opportunity | Adopted solution branches | Simulated evidence only |
|---|---|---|
| **O1 — fragmented Agent, account, and lifecycle identity** | qualified Agent roster; Account Hub; readiness and recovery disposition | Agent and account scenarios |
| **O2 — native work loses authority and recovery continuity** | embedded conversation; Goal -> Plan revision -> Task -> Attempt; multi-Agent handoffs; unified Activity; controls | governed-work and recovery scenarios |
| **O3 — reusable resources lose origin, permission, or conflict truth** | seven-family inventory; MCP management; federated synchronization; fail-closed conflict resolution | resource, secret, MCP, and conflict scenarios |

The target information architecture traces to those opportunities:

| Space | Outcome responsibility | Opportunity trace |
|---|---|---|
| **Home** | readiness, attention, active stewardship cases, blockers, and next valid action | O1, O2 |
| **Agents** | Agent identity, conversation, Runtime/Process, health, permissions, handoffs, and lifecycle | O1, O2 |
| **Work** | Goal, Plan revision, Task, Attempt, Context, Effects, execution flows, and evidence | O2 |
| **Library** | Memory, Skills, Tools, MCP, origin, permission, and synchronization state | O3 |
| **Activity** | merged provenance, declared coverage, conflict, and recovery evidence | O2, O3 |
| **Settings** | Account Hub, Provider routes, workspace, permissions, system, backup, and recovery | O1, O3 |

Providers and System are nested in Settings. Context belongs to Work;
Runtime/Process belongs to Agents. The Global Agent Shell is a cross-space
explainer, not a seventh space and never an authority writer.

## 11. Capability and dependency ledger

| Release-blocking target | Current product truth | Dependency order |
|---|---|---:|
| exact cross-platform and three-Agent qualification | Linux 1.0 qualifies only its exact Pi path; other platform/Agent facts are bounded or target-only | D0 |
| Agent lifecycle and Account Hub | current capabilities cover bounded Pi/dsh facts, custom OpenAI-compatible accounts/endpoints, and fixed bindings; the full target is absent | D1 |
| embedded conversation/history | native dsh remains separate; there is no common embedded product path | D2 |
| Goal -> Plan revision -> Task -> Attempt | current governed Task capability does not provide the full hierarchy or attempt controls | D2 |
| recovery controls and preserved attempts | current detach is observation-only; the full control vocabulary is absent | D2 |
| authority-backed Context/Runtime inventory | current projections cover bounded facets only | D2 |
| MCP seventh-family management | current MCP Tool transport does not provide server-family lifecycle | D2 |
| federated synchronization and conflict handling | no general bidirectional native-resource synchronization exists | D3 |
| multi-Agent graph and handoffs | adopted target, not a complete current runtime capability | D3 |
| unified Activity | current Activity is a bounded composition with partial watch coverage | D3 |
| Global Agent Shell | no cross-Agent Control Plane Shell exists | D3 |
| frozen AI-window product acceptance | no execution result is claimed by this document | D4 |

All rows are Personal 2.0 release blockers. The order is a product dependency
sequence, not a RICE ranking or delivery estimate. RICE is **N/A** because no
real Reach window or person-week effort exists. Evidence confidence is high for
the documented authority and current/target boundary, but low for human demand,
adoption, and usability.

## 12. Measurement, simulated acceptance, and non-claims

The frozen AI-window set contains eight scenarios, each with `N=1`: Pi,
DeepSeek Harness, Codex desktop, conversation-to-governance, multi-Agent
handoff, recovery/attempt preservation, account/resource/secret boundaries,
and MCP/federated conflict. The exact setup, action, expected visible result,
forbidden claim, and denominator live in
[User journeys §13](user-journeys.md#13-frozen-ai-window-simulated-product-acceptance).

Passing **8/8** establishes simulated product acceptance only. It does not
establish human desirability, usability, adoption, retention, willingness to
pay, problem-solution fit, performance, containment, release readiness,
Profile conformance, or Agent benefit. Every started or required scenario is
retained as pass, fail, partial, or not-run; no scenario is replaced to improve
the denominator.

Linux 1.0 remains a six-family, Pi-qualified product with Standard Workspace,
bounded Extended Home, one canonical local service, and the exact qualified Pi
path. Its current Control Plane is additive and the native dsh panel remains
separate. DeepSeek Harness, Codex desktop, Windows/macOS Personal 2.0 platform
claims, MCP, embedded conversations, federated synchronization, Account Hub
expansion, and multi-Agent orchestration do not enter or revise the Linux 1.0
claim.

Personal does not include a kernel module, eBPF control plane, device
scheduler, custom kernel, distributed authority, enterprise tenancy, HA, cloud
authority, or public remote administration. Formal thresholds, current
task/Gate status, release evidence, and environment qualification remain owned
by the formal plan, preregistered campaigns, and `PROGRESS.md`.
