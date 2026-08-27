# CognitiveOS Personal user journeys

- Status: current journeys plus full Personal 2.0 product-version target
- Change class: product-semantic
- Product intent and acceptance: [Product design](product-design.md)
- Current release boundary: [Linux 1.0 scope](linux-1.0-scope.md)
- Exact target inclusion and capability status:
  [Personal 2.0 scope](personal-2.0-scope.md)
- Resource semantics: [Cognitive resource model](cognitive-resource-model.md)

Every journey separates visible interaction from authority facts. Agents,
adapters, native panels, MCP servers, and the global Agent Shell may explain,
translate, propose, or observe. The Rust daemon alone resolves identity,
authorizes, persists Intent/Effect, dispatches, reconciles, and accepts.

These journeys serve the unified resource-stewardship outcome owned by
[Product design §2](product-design.md#2-product-definition-icp-trigger-and-outcome).
They are product hypotheses because there is no external human research. Their
ordered behavior and AI-window scenarios do not establish human desirability,
usability, adoption, willingness to pay, or problem-solution fit.

## 0. Reality ledger

| Boundary | Journey truth |
|---|---|
| **Current implementation (Now)** | Linux 1.0 provides the Pi-qualified six-family path. `/ui/` provides Home, Work, Agents, Providers, Resources, Activity, and System. The native dsh panel is separate. |
| **Personal 2.0 full-version target** | Cross-platform local stewardship of exact Pi, DeepSeek Harness Developer Preview, supported-platform Codex desktop, accounts, seven resource families, native conversations, Goal -> Plan revision -> Task -> Attempt work, multi-Agent handoffs, unified Activity, controls, and recovery. |
| **Requires-backend** | Cross-platform and Agent qualification, embedded conversations, onboarding/catalog, Goal -> Plan revision -> Task -> Attempt, controls, multi-Agent orchestration, federated sync, account import/runtime methods, Global Agent Shell, unified Activity, and MCP management. |
| **Requires-core (conditional)** | Existing Core Conversation/ConversationBinding is reused. Only a new/changed public MCP, conversation extension, Goal, Plan, Run, Harness, or attempt machine surface requires P10-T02/Lane-CTR. |

## 1. Connect an exact Agent and reach first chat

**Personal 2.0 full-version target**

1. The user opens Home or Agents and chooses **Add Agent**.
2. Step 1 offers the exact qualified Pi path, DeepSeek Harness Developer
   Preview from
   [`deepseek-ai/deepseek-harness`](https://github.com/deepseek-ai/deepseek-harness),
   or the Codex experience in the current official ChatGPT desktop app on an
   officially supported and independently qualified platform. The user may
   choose a signed upstream record or **Connect existing**.
3. The product shows source, product identity, version, platform, signature or
   trust facts, license, adapter compatibility, capability coverage, and
   qualification boundary before installation or connection.
4. The next step is one review for Provider/profile, Standard Workspace,
   observation scope, and requested permissions. Optional configuration is
   deferred.
5. That review establishes the exact observation scope; no speculative/global
   session scan or surprise per-session enrollment occurs.
6. The final step opens the Agent's embedded native conversation. The setup is
   not labelled ready until a real model response arrives.
7. The conversation remains Native. The next milestone is
   **Manage with Personal** and one governed and verified Task.

Agent and platform evidence stays independent:

- Pi evidence qualifies only the exact Pi path and platform named by that
  evidence;
- DeepSeek Harness remains labelled **Developer Preview** and is not a
  DeepSeek Provider or model qualification;
- Codex desktop is distinct from Codex CLI, web, IDE, ChatGPT account,
  Provider, and model evidence; no Linux Codex desktop is implied.

If time-to-first-response or onboarding-step results are reported, the record
declares the start event, prerequisites, eligible denominator, platform,
Agent, environment, and not-run cases.

**Failure and recovery**

- Missing or locked SecretStore links to Account Hub without discarding Agent
  selection.
- Provider/model failure retains the review and names whether reachability,
  credential, model, or native adapter is at fault.
- Permission denial preserves a narrower or native-only path when safe.
- Installation or connection failure preserves source and review state and
  offers retry only when the daemon says retry is safe.
- An unsupported Agent/platform pair remains explicitly unavailable. It is not
  substituted with CLI, Provider, model, bridge, or another platform's
  qualification.

**Current implementation (Now)**

The current Control Plane can inspect bounded Agent facts and manage current
Provider bindings, but cannot run this onboarding or embed a conversation. The
qualified Pi conversation path and separate native dsh panel remain available.

**Dependency:** cross-platform and Agent qualification, onboarding/catalog,
embedded conversation, and full lifecycle stewardship are
**Requires-backend**. This journey invents no route or schema.

## 2. First governed and verified success

**Personal 2.0 full-version target**

1. In a useful Native Conversation, the user chooses **Manage with Personal**.
2. The global Agent Shell explains that native conversation and Agent plan are
   not yet governed and proposes the smallest durable outcome.
3. The daemon previews admission of a persistent Goal, initial daemon-owned
   Plan revision, bounded Task, Agent, workspace, Context sources, permissions,
   budget, external effects, and acceptance criteria.
4. The user confirms the consequential preview once.
5. The daemon admits the Goal, initial Plan revision, and Task, then starts
   attempt 1 under that Task.
6. Work shows one execution flow with Native, Observed, and Governed events
   clearly separated.
7. Agent text, Tool output, and process exit remain observations. Effects are
   reconciled and an independent verifier evaluates the fixed criteria.
8. The milestone is reached only when the Task has current Verified evidence
   and daemon acceptance. The success state links to the Goal, durable receipt,
   affected resources, and next action.

**Failure and recovery**

- Changed scope or stale versions require a new daemon preview.
- Missing Context or permission fails closed and preserves the Goal draft.
- Unknown external outcome moves to reconciliation; it is never silently
  retried.
- Failed verification preserves attempt 1 and offers a bounded correction,
  retry, or checkpoint fork only when supported.

**Current implementation (Now)**

Current `/ui/` implements the governed intent/interpret/preview/admit chain and
Task evidence reading. It has no persistent Goal or daemon-owned Plan API and
no common native-conversation projection or retry/fork control. Existing Core
Conversation/ConversationBinding contracts are reused but are not a Control
Plane implementation. The current Task path can demonstrate governed,
independently verified success without claiming the Personal 2.0 wrapper.

**Dependency:** Goal -> Plan revision -> Task -> Attempt and target controls are
**Requires-backend**; only new public machine semantics conditionally require
P10-T02/Lane-CTR.

## 3. Daily Goal → Plan → Tasks execution

**Personal 2.0 full-version target**

1. Home resumes the most relevant active Goal with current Plan revision,
   blockers, last verified outcome, and next action.
2. In Work, the user reviews a plain-language Plan. The inspector shows exact
   revision, authority, Context, budgets, and dependencies.
3. The hierarchy is Goal -> Plan revision -> Task -> Attempt. The daemon
   decomposes the admitted Plan into one or more Tasks. If multiple
   Agents are useful, it owns the graph and issues explicit handoffs; Agents do
   not transfer authority to each other.
4. Each Task starts an attempt bound to its Agent runtime engine, Context, and
   epoch. Native Agent plans and conversations remain source-labelled.
5. The execution flow shows what is Native, Observed, Governed, and Verified.
   Counts appear only with a declared denominator; no fake percentage or ETA is
   inferred from model narration.
6. A Plan revision preserves prior revisions and explains why Tasks changed.
   Consequential scope changes receive a fresh daemon preview and one
   confirmation.
7. Completion rolls up only from independently verified Tasks with reconciled
   Effects. A Goal can remain open even after one Task completes.

**Recovery**

- A failed Agent handoff blocks downstream governed work rather than cascading.
- A stale Plan or Context revision requires reload and review.
- Returning tomorrow resumes from durable Goal/Task facts, not browser memory.
- Switching Agent or Provider for current work is explicit and preserves the
  previous attempt.

**Current implementation (Now)**

The current UI has Task inventory/detail and a composed execution reading. It
has no Goal, Plan, first-class Run, Harness, Conversation, multi-Agent graph, or
attempt API.

**Dependency:** the target journey is **Requires-backend**. New public concepts
conditionally require P10-T02/Lane-CTR; Personal-private projections may not.

## 4. Use the Library without collapsing resource boundaries

**Current implementation (Now)**

1. The user may remember or forget admitted Memory. Candidate, admission,
   provenance, version, expiry, conflict, and tombstone remain distinct.
2. The user may import, inspect, bind, and revoke immutable Skill revisions.
   Skill content grants no execution permission.
3. The user may inspect and govern registered Tools. Unknown, drifted,
   disabled, quarantined, or revoked operations cannot dispatch.
4. Context is inspected with its Task in Work; Runtime is inspected with its
   Agent. Neither belongs in the Library navigation.

**Personal 2.0 full-version target**

Library contains Memory, Skills, Tools, and MCP. Vendor-native resources are
mapped through adapters with origin and sync state. Personal owns governance
and bindings, not the native content. A Skill script still executes only
through an independently registered and authorized Tool. An MCP server
connection still grants neither Tool nor Context permission.

**Dependency:** federated mapping and bidirectional sync are
**Requires-backend**. MCP implementation is **Requires-backend**; only a
new/changed public MCP machine surface conditionally requires P10-T02/Lane-CTR.

## 5. Import an existing account credential

**Personal 2.0 full-version target**

1. In Settings → Account Hub, the user chooses a Provider preset or custom
   OpenAI-compatible endpoint and selects **Import existing credential**.
2. The daemon names the exact source and target SecretStore before reading.
   The user consents per source; there is no background scan.
3. The user reviews redacted source kind, target profile, Provider endpoint,
   and source-retention choice. Retention is the default; secure deletion is a
   separate per-import choice.
4. The daemon reads the owner-designated source through the ADR-0055
   non-logging boundary and writes directly to the approved SecretStore. No new
   plaintext copy is created.
5. A bounded probe reports reachability, credential, model, and capability
   facts separately. Failure preserves the source and entered configuration.
6. The user selects whether the profile is a global default, Agent override, or
   conversation override. Any current session rebind/restart is explicit.
7. The receipt contains redacted metadata and audit identity only.

**Failure and permission**

- A locked source or SecretStore does not expose material and offers unlock or
  another account method.
- Unsupported source format stays unavailable; it is not guessed.
- Import success does not imply Provider reachability or model availability.
- Secret material never enters browser storage, Agent config, logs, evidence,
  SQLite, argv, environment, or chat.

**Current implementation (Now)**

Current Provider management supports API-key handoff to SecretStore, model
catalog, custom OpenAI-compatible accounts/endpoints, fixed Agent binding,
usage, budgets, alerts, and audit. ADR-0055 authorizes the import boundary but
explicitly does not implement it.

**Dependency:** credential import, subscription/OAuth, expanded presets, and
override hierarchy are **Requires-backend**.

## 6. Install and project an MCP server

**Personal 2.0 full-version target**

1. In Library → MCP, the user chooses a server source and reviews identity,
   version, trust/provenance facts, requested permissions, compatible Agent
   clients, and update behavior.
2. The daemon previews installation and any external mutation. Connection alone
   grants no Tool, Context, workspace, model, or host-session authority.
3. After confirmation, the daemon installs/registers the server through the
   family-specific workflow and reports health separately from permission.
4. The user selects compatible Agent clients. Vendor-native session APIs are
   preferred for projection.
5. Where no native API exists, MCP plus vendor rules may cooperatively update
   configuration. It cannot control the host Agent session.
6. After the first explicit authorization, an admin-preauthorized configuration
   may be applied automatically only within that exact scope. Permission
   expansion always receives a new preview and confirmation.
7. Library shows server health, permissions, update state, projected clients,
   and conflicts. Exposed capabilities become eligible Tools or Context inputs
   only through their own mapping and authorization.

**Failure and recovery**

- Unhealthy server and denied permission are separate states.
- Partial client projection lists each successful and failed target.
- A host Agent may require explicit restart/reload; Personal does not claim to
  control it through MCP.
- Update failure preserves the last known usable version when the underlying
  lifecycle supports that outcome; otherwise the state remains explicit.

**Current implementation (Now)**

MCP is outside the six-family Linux 1.0 model and no Personal MCP family manager
exists.

**Dependency:** MCP family implementation is **Requires-backend**. A new or
changed public machine surface conditionally requires P10-T02/Lane-CTR; a
Personal-private projection may not.

## 7. Detect and resolve a federated-resource conflict

**Personal 2.0 full-version target**

1. An authorized adapter detects that a vendor-native Skill, Memory, Tool
   description, MCP configuration, or related binding changed at its origin.
2. Personal records the observation and compares it with the last admitted
   binding/sync fact. Read and change detection may be automatic only inside
   the explicit observation scope established when the Agent was connected.
3. If there is no conflict, the origin-owned content remains native and
   Personal refreshes its governed projection.
4. If native and Personal-side changes conflict, synchronization fails closed.
   No side wins by timestamp or model judgment alone.
5. The global Agent Shell explains the origin, changed facts, affected Agents
   and work, and the family-specific resolution choices available from the
   daemon.
6. Every write-back retains daemon Intent/Effect. Because this journey is
   conflicted, the daemon previews the exact target/consequence and the user
   confirms once. Unconflicted writes may run automatically only inside an
   unchanged exact daemon grant/risk policy.
7. Activity records Native/Observed/Governed facts separately. Verification is
   attached only when an independent check exists.

**Current implementation (Now)**

Current Personal resource operations do not provide general bidirectional
vendor-resource synchronization.

**Dependency:** adapter change detection, sync state, guarded write-back, and
conflict resolution are **Requires-backend**. Public sync contracts, if needed,
conditionally require P10-T02/Lane-CTR.

## 8. Disconnect or uninstall an Agent

**Personal 2.0 full-version target**

1. From an Agent inspector, the user chooses Remove.
2. The product asks a required first question:
   - **Disconnect** Personal management and preserve the native installation;
   - **Uninstall** the Personal-managed installation after an impact preview.
3. The daemon shows affected conversations, Goals, Plan revisions, Tasks,
   Task-owned attempts, bindings, runtime engines, pending Effects, and retained
   data without conflating them.
4. Disconnect revokes Personal bindings and observation according to the exact
   preview; it does not claim to delete vendor-native data.
5. Uninstall follows the daemon-owned lifecycle, fences new work, reconciles or
   exposes pending Effects, and preserves governed history unless a separately
   confirmed retention action applies.
6. The receipt states what was removed, retained, unknown, or incomplete.

**Current implementation (Now)**

The Control Plane has no full Agent lifecycle HTTP surface. Current lifecycle
operations remain outside this UI and must not be represented as working
buttons.

**Dependency:** disconnect/uninstall projection and typed controls are
**Requires-backend**.

## 9. Recover work without erasing attempts

**Personal 2.0 full-version target**

1. Work states exactly what failed: conversation transport, Agent runtime
   engine, Task, Effect, Context source, Provider, MCP server, or watch.
2. The global Agent Shell explains available controls without exercising them.
3. The user chooses among the controls the daemon genuinely supports:
   - **Interrupt** the current interaction;
   - **Pause/Resume request** for governed work;
   - **Cancel** the Task;
   - **Detach** observation only;
   - **Retry** or **Fork from checkpoint** into a new attempt;
   - **Restart/Recover** the Agent runtime engine;
   - **Compensate** an external effect only when a defined compensation exists.
4. The daemon previews consequential scope, pending/unknown Effects, checkpoint
   compatibility, and what cannot be undone.
5. Recovery reloads durable facts, fences stale work, reconciles unknown
   Effects, reauthorizes current policy, and rebuilds Context before dispatch.
6. A retry/fork preserves the failed attempt and its evidence. Current and
   prior attempts remain inspectable under their Task in the same Goal
   timeline.
7. The outcome is Verified only after current independent verification.

**Current implementation (Now)**

Current `/ui/` can attach/detach bounded Task observation; detach does not
cancel or stop anything. It has no Task pause/cancel/retry controls and no full
Agent restart/recover API. Killing a process is not pause, cancellation,
recovery, or completion.

**Dependency:** the target controls and attempt model are
**Requires-backend**. New public control/attempt semantics conditionally require
P10-T02/Lane-CTR.

## 10. Current deterministic stewardship

These are **Current implementation (Now)** Linux/daemon journeys and remain
available as the Personal 2.0 UI evolves:

- Agent/package upgrade uses exact source and digest, stages immutable bytes,
  validates compatibility, preserves the prior complete binding for rollback,
  and exposes incomplete rollback honestly. Pi evidence qualifies only Pi.
- Product upgrade, rollback, and uninstall use the same signed artifact,
  daemon, and application services across desktop, headless, and foreground
  modes. No mode creates a second authority writer.
- Backup/restore excludes secrets, verifies integrity and compatibility, and
  requires secrets to be rebound through an approved SecretStore.
- Deleting user data, changing external state, or removing a managed
  installation receives a daemon preview and retains durable receipts.

The current Control Plane exposes bounded system stewardship such as
backup/restore, but full Agent lifecycle remains outside its HTTP surface.
Personal 2.0 may regroup these journeys under Settings; regrouping does not
change their authority semantics.

## 11. Empty, loading, error, permission, and stale journeys

These states apply to Home, Agents, Work, Library, Activity, Settings, the
global Agent Shell, and every onboarding/import flow.

| State | User sees | Recovery |
|---|---|---|
| **Empty** | why no Agent, conversation, Goal, resource, account, MCP server, or activity exists | one concrete create/connect/import action; filter-empty also offers clear filters |
| **Loading** | the exact source being read or mutation awaiting authority; stable content remains in place | leave/detach when safe; no fake progress bar |
| **Partial** | available facets plus the missing source and coverage boundary | continue with safe facts or repair the missing source |
| **Error** | what failed, where, whether input/work was preserved, and whether retry is safe | retry, edit, choose alternate path, copy redacted details, or open support |
| **Permission** | exact requested scope, reason, consequence, and current narrower capability | deny, grant the bounded scope, or select a narrower/native-only path |
| **Stale** | last known fact, age/freshness, and actions unsafe until refresh | refresh/re-authenticate; never infer current progress or completion |
| **Disconnected** | which watch, adapter, Agent, server, or daemon link was lost | reconnect or remain detached; work state stays unknown unless authority says otherwise |
| **Success** | durable receipt, affected object, source/authority badge, and next valuable action | continue conversation, open Goal, inspect evidence, or return to the previous list |

Long-running setup, import, installation, synchronization, and execution must
also define cancel/detach/retry/resume honestly. If the backend lacks a control,
the state explains that absence instead of drawing a fake control.

## 12. Diagnosis and fixed boundaries

`cognitive doctor --bundle` and System gather redacted platform, service,
database, SecretStore state, Provider, workspace, Agent, Tool, Context, Task,
Process, Effect, and recovery facts. Support output distinguishes known,
unknown, stale, and not-run without prompts, raw Provider traffic, key
material, resolvable SecretRefs, or sensitive content.

Personal 2.0 does not install a kernel module, eBPF control plane, device
scheduler, or distributed authority. It remains owner-local. Hardware or
native integration cannot bypass daemon authorization, CAS/epoch, budget,
Intent/Effect, reconciliation, or independent acceptance.

No journey here is a Gate, release, Profile, performance, containment, or
Agent-benefit claim.

## 13. Frozen AI-window simulated product acceptance

The product-acceptance manifest is fixed at eight scenarios. Each scenario has
`N=1`. A scenario enters the numerator only when every expected product-visible
result is present and no forbidden claim appears. Full pass requires **8/8**.
Failed, partial, and not-run scenarios remain in the denominator and cannot be
replaced. S3 is platform-conditional under
[product design §12](product-design.md#12-measurement-simulated-acceptance-and-non-claims)
(owner decision 2026-08-27): while no officially supported Codex desktop
platform is inside the active execution scope, S3 is recorded as
`not-run (platform-conditional)`; Linux-mainline acceptance closes at seven
platform-eligible passes plus that explicit S3 disposition, and the full 8/8
result remains the full-version requirement.

The AI window receives product-visible facts only. A human may review its
record, but the result remains simulated product evaluation. It is not human
research, usability evidence, adoption evidence, willingness-to-pay evidence,
problem-solution-fit evidence, or release/Gate technical validation.

### S1 — Pi stewardship (`N=1`)

- **Setup:** one exact Pi package/sidecar/platform fixture whose qualification
  boundary is declared; one valid account and Standard Workspace.
- **Action:** connect or inspect Pi, receive one real native response, inspect
  current health, then choose Disconnect.
- **Expected visible result:** exact Pi source, version, platform, capability
  coverage, health, Native conversation state, observation scope, and retained
  native installation after Disconnect.
- **Forbidden claim:** Pi evidence qualifies DeepSeek Harness, Codex desktop,
  another platform, or the complete Personal 2.0 release.

### S2 — DeepSeek Harness Developer Preview stewardship (`N=1`)

- **Setup:** one exact revision and profile from
  [`deepseek-ai/deepseek-harness`](https://github.com/deepseek-ai/deepseek-harness)
  on one declared Personal platform.
- **Action:** connect the native Harness session, receive a real response,
  inspect session/trajectory state, and exercise one supported resume or fork.
- **Expected visible result:** exact upstream identity, revision, profile,
  platform, capability coverage, current session state, preserved prior
  trajectory, and explicit **Developer Preview** status.
- **Forbidden claim:** a DeepSeek model or Provider is the Agent product;
  Developer Preview is stable release qualification; or evidence transfers to
  Pi, Codex desktop, or another platform.

### S3 — Codex desktop stewardship (`N=1`)

- **Setup:** one exact official ChatGPT desktop app build with the Codex
  experience on an officially supported and independently declared platform.
- **Action:** connect an existing Codex project/session, receive a real
  response, inspect current project/session work, and exercise one supported
  continuation or recovery action.
- **Expected visible result:** exact official product/build, platform,
  project/session identity, capability coverage, current state, and explicit
  distinction from Codex CLI, web, IDE, ChatGPT account, Provider, and model.
- **Forbidden claim:** Codex desktop exists on Linux; CLI or Provider evidence
  qualifies desktop; or one supported desktop platform qualifies another.

### S4 — Native conversation to governed work (`N=1`)

- **Setup:** one useful Native Conversation on an independently declared Agent
  path, with bounded workspace, Context, permission, budget, and acceptance
  criteria.
- **Action:** choose **Manage with Personal**, review the daemon preview, and
  confirm once.
- **Expected visible result:** the native conversation remains source-labelled;
  the daemon admits one persistent Goal, current Plan revision, bounded Task,
  and attempt 1; completion appears only after current independent verification
  and daemon acceptance.
- **Forbidden claim:** observation, fluent Agent text, Provider response, Tool
  result, harness success, or process exit creates authority or completion.

### S5 — Multi-Agent handoff and unified Activity (`N=1`)

- **Setup:** one admitted Goal with at least two independently declared Agent
  products, bounded Tasks, explicit handoff criteria, and declared Activity
  sources.
- **Action:** complete one source Task, issue one daemon-owned handoff, and
  inspect the downstream Task and merged Activity.
- **Expected visible result:** source, target, bounded work, authority,
  readiness/blocker, and Native/Observed/Governed/Verified facts remain
  distinct; missing source coverage is explicit.
- **Forbidden claim:** Agents transfer authority to each other; Activity is a
  confidence/progress percentage; or the scenario proves multi-Agent benefit.

### S6 — Recovery and attempt preservation (`N=1`)

- **Setup:** one governed Task whose first attempt has a declared failure or
  unknown Effect and whose current daemon capability names the supported
  controls.
- **Action:** inspect the failure, then detach, retry/fork, or recover through
  one genuinely supported control.
- **Expected visible result:** detach changes observation only; the first
  attempt and evidence remain; an unknown Effect enters reconciliation; a new
  attempt receives a distinct identity; current verification is required
  before completion.
- **Forbidden claim:** detach is cancel; process exit is recovery; unknown work
  may be blindly redispatched; or compensation is rollback.

### S7 — Account, resource, and secret boundaries (`N=1`)

- **Setup:** one declared account/profile and at least one object from each
  currently available family, with source, permission, and observation scope.
- **Action:** connect or select the account, inspect resource state, and change
  or revoke one bounded binding.
- **Expected visible result:** Agent, account, Provider, model, resource,
  installation, and session identities stay separate; source, authority,
  current/unknown state, and next action are explicit; no secret material is
  product-visible.
- **Forbidden claim:** account import proves Provider reachability; connection
  grants permission; or secret material may enter an Agent, browser storage,
  ordinary config, SQLite, argv, environment, logs, evidence, or chat.

### S8 — MCP and federated-resource conflict (`N=1`)

- **Setup:** one exact MCP server target and one origin-owned native resource
  with a staged Personal/native conflict inside a declared observation scope.
- **Action:** connect/project the MCP server, inspect health and permission
  separately, detect the conflict, and request a conflicting write-back.
- **Expected visible result:** MCP remains the seventh family; connection grants
  no Tool/Context/workspace/model/host-session authority; origin and sync
  coverage remain explicit; conflict fails closed; the daemon issues the exact
  preview before any write-back.
- **Forbidden claim:** MCP connectivity qualifies an Agent, Tool, server, or
  host session; timestamp or model judgment resolves conflict; or the scenario
  proves human adoption or broad MCP ecosystem support.

### Aggregate disposition

```text
simulated product acceptance
  = passed frozen scenarios / 8
  = 8 / 8 required
```

The report records each scenario as `pass`, `fail`, `partial`, or `not-run`.
An S3 `not-run` caused by the platform-conditional rule records the reason
`platform-conditional` and follows product design §12. No count, percentage,
timing, or ETA is published without its declared denominator, setup, and
evidence boundary.
