# CognitiveOS Personal 2.0 scope

- Status: adopted full-version target; capability-gated
- Change class: `product-semantic`
- Date: 2026-08-27
- Decision: [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Product intent: [Product design](product-design.md)
- Current-status owner: [PROGRESS.md](../../../docs/plan/PROGRESS.md)
- Preserved release boundary: [Linux 1.0 scope](linux-1.0-scope.md)

## 1. Formal version boundary

Personal 2.0 is the Windows-first product for a single human Owner to create,
operate, and recover governed local Projects and digital employees. The formal
2.0 boundary is one **Windows-local project loop while the host is online**.
Native mobile, device pairing, and end-to-end encrypted relay access are
Personal 2.1.

This is a product commitment, not a capability claim. `Requires-backend`,
`Requires-environment`, `unknown`, and `not-run` remain explicit until their
formal tasks and validation routes close. Linux Personal 1.0 remains finalized
and is not superseded as a historical support/release fact.

## 2. Principal and ownership

Included:

- one local human Owner;
- Projects, Role Blueprints, Assignments, and digital employees belonging
  directly to the Owner;
- optional business/brand profile information;
- daemon-owned authority, local data, and Owner-controlled exports.

Excluded:

- human team accounts, organization membership, `Company` or `Business Space`
  as a current aggregate;
- multi-tenant RBAC, cloud authority, public administration, or HA;
- any assumption that a digital employee is a human identity.

## 3. Included product capabilities

### 3.1 Business control plane

- Today / Projects / Team / Knowledge / Inbox;
- Settings at the bottom;
- global right-side Personal Assistant;
- Project briefing as the Project default;
- business language first and advanced capability terms one disclosure deeper;
- one active assistant/employee composer, with preserved drafts on context
  switch.

### 3.2 Project and employee model

- Project charter, goals, metrics, revisioned plan, permissions, budgets, and
  triggers;
- Role Blueprint -> Project Role Assignment -> Digital Employee Instance ->
  Agent Runtime -> Personal-owned Conversation;
- one current manager per Project;
- only the base Project Manager blueprint built in;
- project-specialized manager and Personal-Assistant-generated member roles;
- manager-led Task, artifact, and handoff coordination;
- bounded manager autonomy and Owner-confirmed boundary revisions.

### 3.3 Execution and continuity

- Task/Attempt identity, artifacts, Effects, evidence, and independent
  completion verification;
- Routine revisions and manual/schedule/qualified-event triggers;
- no-overlap plus queue-latest;
- offline, missed, skipped/coalesced, and risk-based resume facts;
- close-window choice between eligible background work and pause;
- key-result and daily/weekly reflection candidates;
- archive-first Project lifecycle and local restore points.

### 3.4 Assistant and managed Agent

- Personal Assistant supported internally by candidate-only Pi;
- DSH supplied as the preinstalled managed Installed Agent and default digital
  employee runtime;
- exact audited DSH artifact, Personal-managed isolated child process, bounded
  stdio broker, daemon Provider proxy, health, update, and rollback;
- Installed Agents advanced settings with source/version/qualification truth;
- no native DSH UI or native conversation synchronization;
- Personal-owned Conversation, Memory, Task, and archive.

### 3.5 Knowledge and memory

- Personal Home with separate `app/` and `data/`;
- automatic per-Project data directories;
- Owner-shared knowledge, Project Markdown Vault, and employee-private memory;
- Obsidian-compatible files and optional companion only;
- provenance-preserving import, indexing, reindex, conflict, exclusion, and
  failure handling;
- scoped episodic conversation archive participating in bounded retrieval;
- redaction, provenance, untrusted-observation labels, semantic admission,
  inspect/correct/forget.

### 3.6 Providers, budgets, and external work

- subscription/account/billing/quota/model/binding/usage separation;
- effective binding `global -> project -> employee -> task`;
- Project, member, and Task budgets plus Provider quota and actual usage/cost;
- daemon-proxied DSH/Pi Provider traffic and approved SecretStore custody;
- first important X/Twitter content-operation acceptance scenario;
- individually qualified browser/API connectors, rights-safe source handling,
  preview/approval/receipt, and feedback readback.

## 4. Capability truth

| Capability | Current product truth | 2.0 treatment |
|---|---|---|
| Windows host/install/background | existing Windows fragments and ordinary MSVC CI do not constitute a qualified host product | **Requires-backend + Requires-environment** |
| Project/Charter/Goal/Plan/Attempt | current Task authority is reusable but the complete Project aggregate and UI projection are absent | **Requires-backend** |
| Role/Assignment/Employee | no complete current authority/projection | **Requires-backend** |
| Personal-owned Conversation archive | ADR-0058 private envelope exists as a decision; no OPC archive/index/retrieval product | **Requires-backend**; new shape must not reinterpret `0.1` |
| Personal Assistant | existing Pi Shell primitives are reusable; global OPC assistant does not exist | **Requires-backend**; Pi remains hidden/candidate-only |
| Managed DSH Installed Agent | dsh Path B exists post-1.0 but is not the Windows packaged/isolated/supply-chain-qualified product | **Requires-backend + Requires-environment** |
| Routine/Trigger/missed-run | existing scheduler primitives do not provide the full product lifecycle | **Requires-backend** |
| Inbox approval/recovery | existing previews, Effects, alerts, and recovery facts are partial inputs | **Requires-backend** |
| Knowledge/Vault ingestion | current Memory/Skill/Context operations are not an OPC Vault/import/index product | **Requires-backend** |
| Memory privacy/forget | existing admitted Memory/forget is reusable but conversation extraction/retrieval policy is absent | **Requires-backend** |
| Provider routing/budget enforcement | current fixed Agent binding, usage, and advisory budgets exist | hierarchy/enforcement **Requires-backend** |
| OPC UI | current `/ui/` is a delivered non-blocking Linux-era surface | target IA and Windows host integration **Requires-backend** |
| X connector | no qualified X/Twitter connector is claimed | **Requires-backend + Requires-environment** |
| MCP family | ADR-0057/0058 target remains valid; no family manager exists | **Deferred advanced track**, not a 2.0 P0 dependency |

Composition of current primitives does not turn a target row into current
support.

## 5. DSH and Pi boundary

DSH is visible in Settings > Installed Agents because the Owner must be able to
inspect supply chain, version, health, capability, update, and rollback. It is
not exposed as a separate everyday product or conversation authority.

Pi supports the Personal Assistant internally and is not an ordinary Installed
Agent. Both receive only bounded Context and opaque Provider results through
the daemon. They receive no raw secret, ambient environment credential, native
MCP/base-tool grant, HMR, home patch, authority write, Memory ownership, or
completion authority.

Personal 2.0 qualifies only DSH. Hermes, Codex, Cursor, and other adapter
candidates are future work with independent artifact, platform, capability,
security, and campaign evidence.

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
external Agent support, native mobile, disaster backup, or cloud takeover.

This scope implements and qualifies nothing. Windows, DSH, Pi, Project,
Conversation, Vault, Provider, connector, UI, and acceptance tasks remain
unclaimed until the formal plan says otherwise.
