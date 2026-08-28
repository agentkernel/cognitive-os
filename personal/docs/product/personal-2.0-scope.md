# CognitiveOS Personal 2.0 scope

- Status: adopted full-version target; capability-gated
- Change class: `product-semantic`
- Date: 2026-08-28
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Product-direction amendment: owner-confirmed `/grill-me` design tree,
  2026-08-28; architecture and formal-plan reconciliation are deferred.
- Product intent: [Product design](product-design.md)
- Requirements:
  [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Interaction baseline:
  [**Owner-approved interaction baseline (2026-08-28)**](../../../clients/docs/design/opc-2.0/personal-20-ai-ceo-e2e-optimized-v2.canvas.tsx)
- Baseline identity: same V2 files (not a v3). Owner accepted the 2026-08-28
  competitive-informed overwrite: visible CEO loop (Ingest → Decide →
  Authorize → Execute → Verify → Report), Today decision packet plus four
  exception swimlanes, canvas-only HITL, and daemon authority path. This is
  not the pre-overwrite overlay-conversation / stacked-column V2.
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence
- Current-status owner: [PROGRESS.md](../../../docs/plan/PROGRESS.md)
- Preserved release boundary: [Linux 1.0 scope](linux-1.0-scope.md)

## 1. Formal version boundary

Personal 2.0 is the Windows-first digital-staff console for a single human
Owner to research, activate, operate, and recover governed local Projects. The
formal 2.0 boundary is one **Windows-local Project loop while the host is
online**, operated primarily through a Personal Assistant/Project group
conversation and an evidence-backed flexible canvas.
Native mobile, device pairing, and end-to-end encrypted relay access are
Personal 2.1.

This is a product commitment, not a capability claim. `Requires-backend`,
`Requires-environment`, `unknown`, and `not-run` remain explicit until their
formal tasks and validation routes close. Linux Personal 1.0 remains finalized
and is not superseded as a historical support/release fact.

## 2. Principal and ownership

Included:

- one local human Owner;
- Projects, reusable Role Runtime Templates, and Project-specific Member
  Runtime definitions belonging directly to the Owner;
- optional business/brand profile information;
- daemon-owned authority, local data, and Owner-controlled exports.

Excluded:

- human team accounts, organization membership, `Company` or `Business Space`
  as a current aggregate;
- multi-tenant RBAC, cloud authority, public administration, or HA;
- any assumption that a Project Member is a human identity or an always-running
  OS process.

## 3. Included product capabilities

### 3.1 Conversation-and-canvas control plane

- stable anchors **Today / Projects / Knowledge**, with Settings at the bottom;
  Team and Inbox are not first-level destinations;
- locked left / center / right shell; conversation always the third column;
  a narrow canvas scrolls horizontally and does not stack those columns;
  there is no overlay “open conversation” control;
- global Personal Assistant outside Projects and a Project group conversation
  for Owner, manager, and Members;
- HITL announced in chat and confirmed on the center-canvas preview; chat has
  no Approve control and no “Don’t ask again” grant;
- `@manager` and `@member` routing with daemon-owned Task/revision effects;
- manager-default speech; Member proactive speech only when mentioned,
  delivering, handing off, blocked, or requesting a decision;
- Project operating-report template as the default Project surface, then the
  X loop when that Project needs it;
- stable routine-report templates plus temporary ad-hoc canvas composition from
  typed, source-linked components and real Project results;
- temporary canvases are not saved unless pinned or made a template; generated
  code/`eval`, invented values, and hidden failure/freshness are excluded;
- publication packages show thread preview plus acceptance; planned is not
  published; publish preview is the full AUTONOMY packet on the canvas, with
  no Confirm in chat;
- Team and attention/approval surfaces opened contextually, not as permanent
  first-level navigation;
- visible CEO loop (Ingest → Decide → Authorize → Execute → Verify → Report);
- Today decision packet plus four exception swimlanes (Needs you / Can
  continue / Unknown / Missed); cost estimated or actual, with actual unknown
  never shown as zero; Member activity Working / Queued / Waiting table, where
  queued is not running;
- Operations default working view: Candidate → Intent persisted → Fence →
  Execute → Independent verify → Receipt;
- Knowledge Context shows why each fragment was selected; Memory is not silent
  auto-ingest;
- Secrets use SecretStore takeover and never appear in chat;
- `@` inserts only into the unsent draft;
- Role Template → Member → Task → disposable process; process death does not
  delete the Member; Operations Working is not completion;
- Knowledge Vault is Markdown files; Obsidian is an optional companion and is
  not an embedded app;
- absent capabilities are `Requires-backend` / `Requires-environment`; there
  are no Connect / Install / Confirm fake buttons;
- native mobile, pairing, and cloud 24/7 chrome are 2.1 and are not drawn as
  current product chrome;
- business language first and advanced Runtime terms one disclosure deeper.

### 3.2 Project, Role, and Member Runtime model

- Project charter, main/phase-or-quarter/month/week/day goal hierarchy,
  deliverable/evidence contracts, revisioned plan, permissions, cost policy, and
  triggers;
- reusable Role Runtime Template -> Project-specific Member Runtime definition
  -> Task -> disposable Agent process/Attempt;
- one current manager per Project;
- only the base Project Manager Role built in;
- project-specialized manager and Personal-Assistant-researched Member Roles;
- each Member explicitly binds an admitted Provider/model before activation;
- Member cards expose business responsibility/result first and Runtime recipe
  one disclosure deeper;
- manager-led Task, artifact, and handoff coordination;
- bounded manager autonomy and Owner-confirmed boundary revisions;
- bounded internal subagents per Member Task process; no Project identity,
  long-term Memory, or direct inter-Member authority.

### 3.3 Execution and continuity

- Task/Attempt identity, artifacts, Effects, evidence, and independent
  completion verification;
- Routine revisions and manual/schedule/accepted-artifact/Project-state/
  qualified-external-event/testable-data-condition triggers;
- no-overlap plus queue-latest;
- offline, missed, skipped/coalesced, and risk-based resume facts;
- close-window choice between eligible background work and pause;
- key-result and daily/weekly reflection candidates;
- Task, daily, cycle, and incident reflection; evidence-backed, versioned
  Member Runtime improvement inside the approved envelope with rollback;
- safe-point continue/pause/restart for new instruction revisions; no silent
  prompt injection into a running process;
- archive-first Project lifecycle and local restore points.

### 3.4 Assistant and hidden managed engines

- Personal Assistant supported internally by candidate-only Pi;
- DSH supplied as the hidden default Member execution engine;
- exact audited DSH artifact, Personal-managed isolated child process, bounded
  stdio broker, daemon Provider proxy, health, update, and rollback;
- engine identity shown only in fault-resolution or advanced diagnostics;
- no Installed Agent store, alternative engine installation, or Harness
  switching product;
- no native DSH UI or native conversation synchronization;
- Personal-owned group/member Conversation, Memory, Task, and archive.

### 3.5 Knowledge and memory

- Personal Home with separate `app/` and `data/`;
- automatic per-Project data directories;
- Owner-shared knowledge, Project Markdown Vault, and Member-private memory;
- Obsidian-compatible files and optional companion only;
- provenance-preserving import, indexing, reindex, conflict, exclusion, and
  failure handling;
- scoped episodic conversation archive participating in bounded retrieval;
- model-window-aware Context assembly that preserves the current Task contract
  and fixed decisions before sourced summaries;
- full raw conversations retained; ordinary chat not admitted automatically;
  “remember”/stable verified facts create candidates; feedback remains Project
  evidence before any versioned Member/global Role proposal;
- redaction, provenance, untrusted-observation labels, semantic admission,
  inspect/correct/promote/forget, with Owner confirmation for cross-Project
  promotion.

### 3.6 Model connections, capabilities, cost, and external work

- Settings **Model Connections** with mainstream Provider quick templates where
  the Owner enters a key, plus advanced custom URL/compatibility-mode/key/model
  input;
- no consumer subscription, invoice, or plan-management product;
- explicit Provider/model selection for every Member; Task temporary override
  only through an admitted revision;
- source-labelled actual/estimated/unknown usage and cost with warning-only
  product policy, not a Personal-managed automatic budget stop;
- daemon-proxied DSH/Pi Provider traffic and approved SecretStore custody;
- Assistant-led Skill discovery and automatic installation only after source,
  license, hidden-instruction, prompt-injection, and file/network/command-intent
  review;
- Assistant-led MCP discovery with those checks plus dependency,
  executable-code, network, Secret, tool-permission, and supply-chain review;
  exact Owner confirmation before first install or permission expansion;
- globally reusable, version-pinned capability artifacts with separate
  Project/Member grants, update review, compatibility test, and rollback;
- first important X/Twitter content-operation acceptance scenario;
- individually qualified browser/API connectors, rights-safe source handling,
  preview/approval/receipt, and feedback readback.

## 4. Capability truth

| Capability | Current product truth | 2.0 treatment |
|---|---|---|
| Windows host/install/background | existing Windows fragments and ordinary MSVC CI do not constitute a qualified host product | **Requires-backend + Requires-environment** |
| Project/Charter/Goal/Plan/Attempt | current Task authority is reusable but the complete Project aggregate and UI projection are absent | **Requires-backend** |
| Role/Member Runtime definition | no complete current authority/projection | **Requires-backend** |
| Personal-owned Conversation archive | ADR-0058 private envelope exists as a decision; no OPC archive/index/retrieval product | **Requires-backend**; new shape must not reinterpret `0.1` |
| Personal Assistant | existing Pi Shell primitives are reusable; global OPC assistant does not exist | **Requires-backend**; Pi remains hidden/candidate-only |
| Hidden managed DSH engine | dsh Path B exists post-1.0 but is not the Windows packaged/isolated/supply-chain-qualified product | **Requires-backend + Requires-environment** |
| Routine/Trigger/missed-run | existing scheduler primitives do not provide the full product lifecycle | **Requires-backend** |
| Contextual approval/recovery | existing previews, Effects, alerts, and recovery facts are partial inputs | **Requires-backend** |
| Knowledge/Vault ingestion | current Memory/Skill/Context operations are not an OPC Vault/import/index product | **Requires-backend** |
| Memory privacy/forget | existing admitted Memory/forget is reusable but conversation extraction/retrieval policy is absent | **Requires-backend** |
| Provider/model connection and cost visibility | current fixed Agent binding, usage, and advisory budgets exist | Member binding hierarchy, custom compatibility setup, and honest attribution **Requires-backend** |
| OPC UI | current `/ui/` is a delivered non-blocking Linux-era surface | target IA and Windows host integration **Requires-backend** |
| X connector | no qualified X/Twitter connector is claimed | **Requires-backend + Requires-environment** |
| Project/Member Skill/MCP acquisition and grant | current Skill/Tool/MCP transport facts are insufficient; no reviewed discovery/grant flow exists | security-reviewed acquisition and exact per-scope grant **Requires-backend**; broad marketplace/family console remains out of scope |

Composition of current primitives does not turn a target row into current
support.

## 5. DSH and Pi boundary

DSH and Pi are hidden managed engines during ordinary work. Their identity,
exact version, health, update/rollback, qualification, and failure facts appear
only when the Owner needs to resolve a problem or opens advanced diagnostics.
There is no Installed Agents product surface.

Both receive only task-scoped, model-window-bounded Context and opaque Provider
results through the daemon. They receive no raw secret, ambient environment
credential, unmanaged native MCP/base-tool grant, HMR, home patch, authority
write, Memory ownership, or completion authority.

Personal 2.0 qualifies only DSH as the Member execution engine and Pi as the
Assistant engine. Hermes, Codex, Cursor, and other engines remain outside 2.0;
the product has no user-facing adapter marketplace or switching promise.

The 2026-08-27 architecture, ADR, formal-plan, and handbook descriptions that
still expose Installed Agents, top-level Team/Inbox, the prior Role/Employee
chain, or a different MCP/budget policy are **pending architecture/plan/
handbook reconciliation**. They remain dated facts and are not rewritten by
this product-only scope update.

## 6. Local data and recovery

Product and business data remain local. Diagnostics are opt-in. Same-disk
automatic versions are named **local restore points** and explicitly do not
protect against disk loss. Manual export remains available; secrets are
excluded by default. Project archive stops triggers before permanent deletion,
and deletion requires impact preview and a second confirmation.

## 7. Explicit 2.1 boundary

2.1 may add native mobile, device pairing, and an E2E relay, but the Windows
host remains online and authoritative. The Owner decided against per-action
biometric reauthentication after pairing. Future minimum controls remain
device-bound keys, revocation, short sessions, action preview, receipt/audit,
and no secret downlink.

## 8. Exclusions and non-claims

2.0 does not promise offline-host 24/7 execution, a guaranteed business
outcome, full autonomy, browser/API equivalence, all-platform publication,
anti-abuse evasion, CAPTCHA bypass, unlicensed copying, multi-Agent benefit,
external Agent/Harness support, consumer subscription management, arbitrary
generated-code UI, a general no-code workflow builder, native mobile, disaster
backup, or cloud takeover.

This scope implements and qualifies nothing. Windows, DSH, Pi, Project,
Conversation, Vault, Provider, connector, UI, and acceptance tasks remain
unclaimed until the formal plan says otherwise.
