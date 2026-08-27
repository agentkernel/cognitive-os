# CognitiveOS Personal 2.0 scope

- Status: full product-version commitment; implementation remains
  capability-gated
- Change class: product-semantic
- Date: 2026-08-27
- Product intent and acceptance: [Product design](product-design.md)
- Ordered behavior: [User journeys](user-journeys.md)
- Current-status owner: [PROGRESS.md](../../../../docs/plan/PROGRESS.md)
- Preserved release boundary: [Linux 1.0 scope](../linux-1.0-scope.md)

This document owns exact Personal 2.0 product inclusion, platform and Agent
scope, capability status, and release-level exclusions. It does not own current
task or Gate status, public machine contracts, implementation architecture, or
human-market evidence.

## 1. Full-version and evidence boundary

Personal 2.0 is a complete product-version commitment, not a directional
milestone. All included capabilities in §4 are release blockers. No delivery
date or effort appetite is adopted by this scope.

The commitment is product-semantic only. Every missing capability remains
**Requires-backend**. No target wording means that implementation, Gate,
release, Profile, performance, containment, Agent-benefit, usability,
adoption, willingness-to-pay, or problem-solution-fit evidence exists.

The product has no external human research evidence. AI-window results are
simulated product evaluation only and use the fixed acceptance boundary owned
by [Product design §12](product-design.md#12-measurement-simulated-acceptance-and-non-claims).

## 2. Reality ledger

| Boundary | Exact scope truth |
|---|---|
| **Current implementation (Now)** | Linux 1.0 is six-family and Pi-qualified. Its daemon-served Control Plane has the current seven spaces, Activity is a bounded composition, and the native dsh panel remains separate. Bounded Provider, resource, Task-evidence, readiness, backup/recovery, Pi, and dsh capabilities exist, but they do not compose the full Personal 2.0 product. |
| **Personal 2.0 full-version target** | A cross-platform local product for independently qualified Windows, macOS, and Linux paths; exact Pi, DeepSeek Harness Developer Preview, and supported-platform Codex desktop paths; unified Agent/account/resource/governed-work stewardship; embedded conversations; Goal -> Plan revision -> Task -> Attempt; multi-Agent supervision; Account Hub; federated resources; unified Activity; and MCP as the seventh family. |
| **Requires-backend** | The missing product capabilities in §5 are target-only until implemented and independently validated on each claimed platform and Agent path. |
| **Requires-core (conditional)** | Only a new or changed public machine surface requires the separately governed contract decision. A Personal-private product projection may not. This document defines no schema, route, API, database, or transition. |

The Rust daemon is the sole authority writer on every claimed platform.
Native applications, Agents, adapters, sidecars, MCP servers, UI, CLI, and the
Global Agent Shell remain clients, observers, candidates, or explainers.

## 3. Exact platform and Agent inclusion

### 3.1 Cross-platform local boundary

Windows, macOS, and Linux are adopted Personal 2.0 local product platforms.
Each platform requires an independent capability, installation, SecretStore,
Agent, lifecycle, recovery, negative, and release qualification statement.
One platform's evidence transfers to none of the others.

Cross-platform local means that every supported installation retains one
owner-local daemon authority. It does not adopt distributed authority, public
remote administration, cloud authority, enterprise tenancy, HA, or
multi-principal RBAC.

The current release remains Linux 1.0. Windows and macOS Personal 2.0 support
are targets and **Requires-backend** until separately qualified.

### 3.2 Initial Agent set

| Agent product | Exact included identity | Platform boundary | Qualification boundary |
|---|---|---|---|
| **Pi** | exact acquired Pi package and Personal sidecar path | only platforms explicitly qualified for that exact path | Linux 1.0 Pi evidence remains exact and does not automatically qualify Personal 2.0 capabilities or another platform |
| **DeepSeek Harness** | [`deepseek-ai/deepseek-harness`](https://github.com/deepseek-ai/deepseek-harness), labelled **Developer Preview** | each Personal platform is independently claimed | exact source revision, profile, native session, capability, lifecycle, recovery, and negatives; no DeepSeek model or Provider evidence transfer |
| **Codex desktop** | the Codex experience in the current official ChatGPT desktop app | only platforms officially supported by that desktop product and explicitly qualified by Personal; **no Linux Codex desktop is implied** | exact official product/build, project/session, lifecycle, recovery, permission, and platform; Codex CLI, web, IDE, ChatGPT account, OpenAI Provider, model, or bridge evidence does not qualify this path |

Connecting an Agent, authenticating an account, selecting a Provider, invoking
a CLI, or projecting an MCP configuration is not qualification. Every Agent
and platform must declare exact support or explicit unavailable status.

### 3.3 Non-normative product-behavior references

Official product identity sources:

- DeepSeek Harness:
  [`deepseek-ai/deepseek-harness`](https://github.com/deepseek-ai/deepseek-harness);
- current Codex desktop identity:
  [OpenAI Codex app announcement](https://openai.com/index/introducing-the-codex-app/)
  and the current Codex experience in the official ChatGPT desktop app.

[`jlcodes99/cockpit-tools`](https://github.com/jlcodes99/cockpit-tools) is a
non-normative product-behavior reference only. Its published product behavior
includes a cross-platform local account roster, account switching, quota and
reset visibility, explicit application paths, and isolated multi-account
application instances. Personal does not inherit its requirements,
implementation, credential storage, security posture, license, support,
authority model, or qualification evidence.

## 4. Exact full-release inclusion

The five P0 outcomes and acceptance signals are owned by
[Product design §5](product-design.md#5-p0-release-outcomes-and-product-acceptance).
The exact included product capabilities are:

1. **qualified Agent and account stewardship**
   - exact Pi, DeepSeek Harness Developer Preview, and Codex desktop paths;
   - signed catalog or Connect existing onboarding;
   - first real native response;
   - Agent health, recovery, disconnect, and uninstall;
   - Account Hub presets, custom OpenAI-compatible endpoints,
     subscription/OAuth, API key, approved credential import, routing scopes,
     usage, quota, and cost;
2. **conversation-to-governed work**
   - adapter-backed embedded native conversations and history;
   - Manage with Personal;
   - Goal -> Plan revision -> Task -> Attempt;
   - durable attempts and current independent verification;
3. **multi-Agent supervision and unified Activity**
   - admitted multi-Agent graph and explicit handoffs;
   - Native / Observed / Governed / Verified source labelling;
   - declared Activity coverage;
   - Global Agent Shell explain/propose behavior;
4. **seven-family and federated resource stewardship**
   - Memory, Skill, Tool, Context, Task, Runtime/Process, and MCP;
   - authority-backed Context and Runtime inventory;
   - origin-owned native content;
   - federated change detection, bounded synchronization, guarded write-back,
     and fail-closed conflict handling;
5. **recovery, controls, and continuity**
   - interrupt, pause/resume request, cancel, detach, retry/fork,
     runtime restart/recover, and defined compensation only;
   - Standard Workspace, bounded Extended Home, cross-session continuity,
     backup, recovery, stale/disconnected truth, and redacted support facts.

The target product organization remains:

**Home / Agents / Work / Library / Activity / Settings**

Providers and System belong to Settings; Memory, Skills, Tools, and MCP belong
to Library; Context belongs to Work; Runtime/Process belongs to Agents. The
Global Agent Shell is cross-cutting and never authority.

## 5. Capability-status ledger

Every target below is a Personal 2.0 release blocker.

| Capability | Current product truth | Personal 2.0 treatment |
|---|---|---|
| Cross-platform Personal | the current release claim is Linux 1.0 | independently qualified Windows, macOS, and Linux local product paths — **Requires-backend** |
| Three-Agent initial set | Linux 1.0 qualifies only exact Pi; bounded dsh and Codex-related implementation facts do not establish full 2.0 support | independently qualified exact Pi, DeepSeek Harness Developer Preview, and supported-platform Codex desktop paths — **Requires-backend** |
| Embedded conversations/history | no common embedded Control Plane conversation/history path; native dsh remains separate | adapter-backed native conversations with explicit observation/admission — **Requires-backend** |
| Goal -> Plan revision -> Task -> Attempt | current governed Task capability does not provide the complete hierarchy or attempt controls | persistent Goal, daemon-owned Plan revisions, bounded Tasks, preserved attempts — **Requires-backend** |
| Task controls | current detach is bounded observation and does not cancel work; full controls are absent | interrupt, pause/resume request, cancel, detach, retry/fork — **Requires-backend** |
| Agent lifecycle | current library/CLI/runtime facts do not compose full cross-platform lifecycle stewardship | onboarding, health, restart/recover, disconnect/uninstall — **Requires-backend** |
| Context/Runtime inventory | current projections cover bounded facets | authority-backed Work and Agent stewardship inventory — **Requires-backend** |
| Multi-Agent orchestration | adopted design target, not a complete current product capability | daemon-admitted graph and explicit handoffs — **Requires-backend** |
| Unified Activity | current Activity is a labelled bounded composition with partial source coverage | cross-domain source-labelled Activity with declared coverage — **Requires-backend** |
| Federated synchronization | no general bidirectional native-resource synchronization | scoped change detection, guarded write-back, conflict handling — **Requires-backend** |
| Account Hub expansion | custom OpenAI-compatible accounts/endpoints and bounded Provider management exist; broader methods and overrides do not | subscription/OAuth, approved import, broader presets, three-level routing scopes — **Requires-backend** |
| MCP seventh family | bounded MCP Tool transport is Tool-family implementation; no MCP server-family manager exists | install, connection, health, permission, update, quarantine, client projection — **Requires-backend** |
| Global Agent Shell | no cross-Agent Control Plane Shell exists | persistent explainer and proposal layer, never authority — **Requires-backend** |
| Frozen AI-window acceptance | no result is claimed by this scope | eight fixed simulated product scenarios, target 8/8; not technical release evidence |

Existing capabilities may be composed only as declared facts. Composition
cannot imply that a missing target capability exists.

## 6. Authority, secret, origin, and evidence boundaries

- Only the daemon resolves authority, persists Intent/Effect, dispatches,
  reconciles, and accepts completion.
- A native or observed conversation, plan, resource, Tool result, Provider
  response, Agent output, native harness success, or process exit is not
  governed or complete by default.
- Agent connection establishes one explicit observation scope. There is no
  speculative global scan or surprise per-session enrollment.
- Secret material stays in approved Secret Stores and daemon-mediated proxy
  profiles. User-directed import does not place secret material in Agents,
  ordinary config, SQLite, argv, environment, logs, browser storage, Context,
  Memory, evidence, or chat.
- Origin owns vendor-native content and lifecycle. Personal owns admitted
  bindings, permissions, synchronization intent, policy, and authority
  records.
- Every write-back is a daemon-owned Intent/Effect. New, broader, destructive,
  or conflicted scope receives an exact preview and confirmation.
- MCP connection grants no Tool, Context, workspace, model, or host-session
  authority and transfers no Agent or platform qualification.
- Counts, rates, percentages, quotas, costs, and ETAs require a declared
  source, denominator, basis, and evidence boundary.

## 7. Product dependency and acceptance boundary

The product dependency order is:

1. freeze exact platform, Agent, account, and source identities;
2. establish lifecycle, observation, permission, and secret-safe account
   stewardship;
3. complete embedded conversation and single-Agent governed-work continuity;
4. complete seven-family inventory, controls, and preserved attempts;
5. complete federated synchronization, multi-Agent handoffs, unified Activity,
   and Global Agent Shell;
6. execute the frozen AI-window product scenarios.

This is not a RICE ranking, schedule, delivery estimate, or permission to omit
later rows. RICE remains **N/A** because no real Reach window, human evidence,
or person-week effort is available.

The fixed simulated product-acceptance denominator is eight scenarios. A full
pass requires 8/8. A failed, partial, or not-run scenario remains in the
denominator. S3 (Codex desktop) is platform-conditional per
[product design §12](product-design.md#12-measurement-simulated-acceptance-and-non-claims)
(owner decision 2026-08-27): while no supported Codex desktop platform is in
the active execution scope it is recorded `not-run (platform-conditional)`,
Linux-mainline acceptance closes at seven platform-eligible passes plus that
disposition, and full 8/8 remains the full-version requirement. The exact
scenarios live in
[User journeys §13](user-journeys.md#13-frozen-ai-window-simulated-product-acceptance).

## 8. Explicit exclusions and non-claims

- Linux 1.0 remains six-family and Pi-qualified. Personal 2.0 platform, Agent,
  conversation, MCP, federation, Account Hub, Activity, control, or multi-Agent
  evidence cannot be back-projected into its release claim.
- DeepSeek Harness remains explicitly **Developer Preview** until its exact
  upstream product status changes and Personal separately adopts that change.
- Codex desktop enters scope only on officially supported and independently
  qualified platforms. This scope claims no Linux Codex desktop.
- Personal 2.0 remains owner-local and single-principal. Multi-user/RBAC,
  enterprise tenancy, HA, distributed authority, public remote administration,
  and cloud authority are excluded.
- IoT/embodied and enterprise bridges remain unadopted headroom.
- AI-window results do not prove human desirability, usability, adoption,
  retention, willingness to pay, problem-solution fit, performance,
  containment, release readiness, Profile conformance, or Agent benefit.
- No target in this document is a statement that implementation, a Gate, or a
  release has completed.
