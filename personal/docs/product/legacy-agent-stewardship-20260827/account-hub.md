# Personal Account Hub

- Status: adopted Personal 2.0 product target
- Canonical language: English
- Current authority foundation:
  [Provider Control Plane](provider-control-plane.md)
- Credential-import boundary:
  [ADR-0055](../../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md)
- Chinese translation: [account-hub.zh-CN.md](account-hub.zh-CN.md)

Account Hub is the Settings surface for Provider accounts, credentials, custom
endpoints, models, daemon proxy profiles, routing scope, quota, usage, cost,
and account recovery. It is beginner-first without hiding governance detail.

## 1. Reality ledger

| Boundary | Account Hub truth |
|---|---|
| **Current implementation (Now)** | The Provider Control Plane supports named OpenAI, Anthropic, and custom OpenAI-compatible accounts; API-key SecretStore handoff; model discovery/manual models; fixed Agent bindings; usage, cost, soft budgets/alerts, audit; and current Providers UI. |
| **Adopted Personal 2.0 target** | A tiered Account Hub with broader presets, subscription/OAuth, API key, ADR-0055 import, custom endpoint, daemon proxy profiles, and global/Agent/conversation routing scopes. |
| **Requires-backend** | Additional Provider adapters, subscription/OAuth lifecycle, existing-credential import implementations, profile hierarchy, explicit current-session rebind/restart, and broader quota ingestion. |
| **Requires-core (conditional)** | Only a new or changed public account/profile/override machine contract requires P10-T02/Lane-CTR. A Personal-private projection may not require core changes. |

## 2. First screen and provider order

The first screen prioritizes the most common choices:

1. **OpenAI**
2. **Anthropic**
3. **Google**
4. **DeepSeek**

**More providers** expands to:

- Qwen/Bailian;
- Kimi;
- Zhipu;
- SiliconFlow;
- Volcengine-Doubao;
- MiniMax;
- OpenRouter.

**Custom OpenAI-compatible** is a first-class choice on the same surface, not a
buried generic form.

A visible preset is an adopted product choice, not an implementation,
qualification, availability, or Provider-quality claim. Each preset shows the
credential methods and capabilities the daemon actually supports.

## 3. Credential and endpoint methods

| Method | Product behavior | Current status |
|---|---|---|
| Subscription/OAuth | User authorizes an account through the supported Provider flow; refresh/revocation stays daemon-owned and non-logging. | **Requires-backend** |
| API key | Hidden one-way handoff to the daemon; the browser never reads it back. | **Current implementation (Now)** for current Provider kinds |
| Import existing credential | User selects one exact existing source; daemon reads and stores it under ADR-0055. | Boundary adopted; implementation **Requires-backend** |
| Custom endpoint | User configures a first-class custom OpenAI-compatible endpoint with explicit trust review where needed. | **Current implementation (Now)** for the existing compatible path |

Every Personal-managed method ends in:

1. an approved daemon SecretStore;
2. non-secret account metadata;
3. a daemon-mediated proxy profile;
4. redacted readiness, model, usage, and audit projections.

Raw credential material never enters Agent configuration, browser storage,
URLs, ordinary configuration, SQLite, argv, environment, logs, evidence, or
chat. Agents, adapters, MCP servers, and the global Agent Shell never receive
it.

## 4. Existing-credential import

ADR-0055 fixes the import boundary:

- user-initiated and separately consented for each exact source;
- no speculative, background, or bulk credential scanning;
- daemon-owned read and SecretStore write;
- secret material exists only in daemon process memory between source and
  target;
- audit/evidence stores redacted source kind, target store, time, and outcome;
- source retention is the default;
- secure source deletion is a separate per-import choice;
- no new plaintext copy outside source and target SecretStore.

An import success means only that material reached the target SecretStore. It
does not prove Provider reachability, entitlement, model availability, quota,
or Agent readiness.

## 5. Daemon proxy profiles and scope hierarchy

The target presents routing as a visible scope hierarchy:

1. **Global default** — the ordinary default for new Personal-managed use;
2. **Agent override** — a selected Agent uses a different profile;
3. **Conversation override** — one conversation uses a different profile.

The narrower scope wins only when the daemon has admitted it. The hierarchy is
not automatic fallback, load balancing, arbitrary per-request Provider
selection, or permission expansion.

Every selected profile keeps secret resolution and Provider egress in the
daemon. Native-only Agent use may continue outside Personal, but it remains
Native/Observed and is not represented as governed proxy use.

### Current-session behavior

A profile change states its effect before confirmation:

- applies to new conversations;
- can rebind a current session through a supported native/adapter path; or
- requires an explicit session/runtime restart.

No current session switches Provider, account, model, endpoint, or credential
silently. Rebind/restart preserves the prior conversation and governed-attempt
history.

The hierarchy and current-session coordination are **Requires-backend**.

## 6. Account setup flow

Account setup can be entered directly or inside Agent onboarding:

1. choose Provider/custom endpoint and credential method;
2. enter or authorize material through the approved daemon path;
3. review redacted endpoint trust, model/profile choice, and routing scope;
4. run bounded checks that keep reachability, credential, model discovery, and
   capability outcomes separate;
5. save the profile and return to the originating Agent or Settings context.

The form preserves non-secret input after recoverable errors. Optional pricing,
quota, and advanced endpoint detail do not block first chat when they are not
required.

## 7. Account states and recovery

The UI renders exact daemon facts into plain-language groupings without
creating new authority states:

- usable;
- degraded;
- credential missing/revoked;
- SecretStore locked/unresolvable;
- endpoint trust required;
- model unavailable;
- unknown or stale.

A successful network connection alone never means usable. A failed model
refresh preserves the last known catalog and current binding. Deleting an
account with active bindings remains blocked until impact is resolved.

Rotation, revocation, endpoint change, and account removal show affected
Agents/conversations and exact current-session consequences. The global Agent
Shell may explain recovery; only the daemon previews and executes it.

## 8. Quota, usage, and cost

These are separate readings:

| Reading | Required honesty |
|---|---|
| **Quota** | Provider-reported allowance, remaining amount, reset period, and source when available; otherwise unavailable |
| **Usage** | measured or estimated consumption, period, account/model/Agent scope, and metering source |
| **Cost** | pricing version, currency/basis, estimated/reported status, and unavailable state |

Quota is not inferred from usage. Usage is not inferred from cost. Unknown is
not zero. A percentage, cache-hit rate, or remaining fraction appears only
with a declared denominator. Soft budgets and alerts do not silently block or
reroute requests.

## 9. Required product states

| State | Account Hub behavior |
|---|---|
| Empty | explain why no account exists; offer the four primary presets and custom endpoint |
| Loading | name whether SecretStore, Provider, model catalog, quota, usage, or audit is loading |
| Partial | keep usable account/profile facts and label the unavailable source |
| Permission | explain exact import, endpoint trust, or routing scope; allow deny/narrow path |
| Error | preserve non-secret input; name failure class and safe retry/edit path |
| Stale | show last known catalog/quota/usage time and require refresh before consequential change |
| Success | show redacted receipt, selected scope, affected Agent/conversation, and next action |

## 10. Backend Capability Gaps

### Backend absent

- subscription/OAuth capture, refresh, and revocation;
- concrete ADR-0055 import readers;
- Provider adapters and entitlement/quota readers beyond current support;
- global/Agent/conversation profile hierarchy;
- explicit native-session rebind/restart coordination.

### API exists, UI-dark or reusable

Current account, API-key, endpoint trust, model, binding, usage, budget, alert,
and audit capabilities already back current Provider UI. Account Hub can
regroup those capabilities under Settings without claiming the missing target
methods.

### Contract/core gap

Only new or changed public account, proxy-profile, subscription, or override
machine semantics conditionally require P10-T02/Lane-CTR. Personal-private
projection work may not require core changes.

## 11. Fixed boundaries and non-claims

- No browser, Agent, adapter, MCP server, or Shell secret custody.
- No ambient fallback, load balancing, arbitrary auth headers, or silent
  current-session switching.
- No prompt/completion retention in usage accounting.
- No Provider success, quota state, model response, or process exit as Task
  completion.
- No multi-user/RBAC or remote public administration.
- Preset presence and product adoption make no implementation, Provider
  quality, Gate, release, Profile, performance, or Agent-benefit claim.
