# LLM Provider Control Plane

## Status and authority

- Status: current Provider authority plus adopted Account Hub evolution
- Product: `cognitiveos-personal`
- Current-status owner:
  [PROGRESS.md](../../../../docs/plan/PROGRESS.md)
- Target experience: [Account Hub](account-hub.md)
- Credential-import boundary:
  [ADR-0055](../../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md)

The Rust daemon remains the sole authority writer and the only component that
may resolve Provider credentials or perform Personal-managed Provider egress.
The browser, Agents, adapters, native panels, and global Agent Shell never
receive raw secret material.

## Reality ledger

| Boundary | Provider truth |
|---|---|
| **Current implementation (Now)** | Named OpenAI, Anthropic, and custom OpenAI-compatible accounts; API-key SecretStore handoff; model discovery/manual models; fixed Agent binding; usage, cost, soft budgets/alerts, audit; current Providers UI. |
| **Adopted Personal 2.0 target** | Settings → Account Hub with tiered presets, subscription/OAuth, API key, ADR-0055 import, custom endpoint, daemon proxy profiles, and global/Agent/conversation routing scopes. |
| **Requires-backend** | Google/DeepSeek and additional preset adapters, subscription/OAuth, credential import, profile scope hierarchy, explicit current-session rebind/restart, and broader quota integration. |
| **Requires-core (conditional)** | Only a new or changed public account/profile/override machine contract requires P10-T02/Lane-CTR. A Personal-private projection may not require core changes. |

## Product outcome

The current Provider Control Plane is the authority foundation for Personal
2.0 Account Hub. It securely stores account metadata and opaque secret
references, binds Agents to models, performs bounded discovery/egress, and
reports usage/cost honestly. Account Hub adds a beginner-first selection and
routing experience without moving secret custody or policy into the UI.

Current operator usage is documented in
[`personal/handbook/en/user/provider-control-plane.md`](../../../handbook/en/user/provider-control-plane.md)
(zh-CN:
[`personal/handbook/zh-CN/user/provider-control-plane.md`](../../../handbook/zh-CN/user/provider-control-plane.md)).
This page remains product design and does not copy current task or Gate status.

## Current scope

| Surface | Current implementation (Now) |
|---|---|
| Official providers | OpenAI and Anthropic |
| Custom providers | OpenAI-compatible endpoints only |
| Qualification | Pi remains the Linux 1.0 qualified path; no Provider preset transfers Agent qualification |
| Credential | API key only |
| Accounts | Multiple named accounts per provider |
| Binding | One fixed account + provider + model per agent instance |
| Fallback | None; failures are returned and audited |
| Cost control | Estimate and soft alert; no blocking |
| UI | current daemon-served Providers and Agent-binding surfaces |

There is no current subscription/OAuth import, conversation override,
automatic fallback, routing/load balancing, hard budget blocking, or
multi-user administration.

## Account and endpoint trust

### Current implementation (Now)

An account stores non-secret identity, Provider kind, redacted endpoint/trust,
status, catalog revision, and an opaque SecretStore reference. It never stores
or returns the API key. Key rotation preserves account and historical usage
identity.

Creation validates endpoint policy, persists the governed operation, stores the
key through the daemon, discovers models, and verifies the resulting account.
Discovery failure is degraded and auditable without invalidating a prior
catalog or binding. Missing/removed key material makes the account non-callable.
Active bindings prevent deletion.

Official endpoints are fixed. A custom OpenAI-compatible endpoint may be
public, loopback, LAN, private-network, or plain HTTP only after explicit,
durable account-level confirmation (`--allow-private-network` and/or
`--allow-insecure-http`). Confirmation is renewed when authority changes,
network scope broadens, or HTTPS becomes HTTP. DNS results are checked again at
request time.

Embedded credentials, redirects, caller-supplied paths, arbitrary headers, and
implicit URL rewriting are rejected. Custom authentication is always
`Authorization: Bearer <API_KEY>`; clients cannot supply header templates.
Official adapters use their fixed provider-native wire requirement.

### Adopted Personal 2.0 target

Account Hub treats every Personal-managed credential method—subscription/OAuth,
API key, ADR-0055 import, or custom endpoint—as input to the same daemon-owned
SecretStore and proxy-profile boundary. Native-only Agent use may remain
outside Personal, but it is labelled Native/Observed and is never represented
as governed routing.

The target presets and methods are described in [Account Hub](account-hub.md).
They are **Requires-backend** except where the current Provider authority
already supports the exact account/API-key behavior.

## Models and pricing

Account creation performs one foreground model discovery using the Secret Store
key. Refresh is explicit and never background. A failed refresh is audited and
preserves the account, last catalog, and binding. Compatible discovery is
`GET /v1/models`.

Catalog sources are `provider_discovered` or `manually_configured`. Manual
models are selectable but visibly less certain. Official models use a versioned
built-in price table. Custom/manual models accept input, output, cache-read,
and cache-write prices per million tokens. Missing prices yield
`cost_unavailable`; usage is not hidden or priced as zero.

## Agent binding

### Current implementation (Now)

Each Agent has at most one active, revision-guarded account/provider/model
binding. Requests cannot select another route. There is no fallback. Errors are
stable and audited. Agents do not read the SecretStore; Personal-managed
Provider traffic uses the daemon proxy/session boundary. Agent adapters are
qualified independently.

### Adopted Personal 2.0 target

Profile selection has three explicit scopes:

1. global default;
2. Agent override;
3. conversation override.

The narrower scope wins only where the daemon has admitted it. This is not
automatic failover, load balancing, or caller-selected arbitrary headers. A
change states whether it applies to new conversations only or whether a current
session must be explicitly rebound/restarted. Existing sessions never switch
silently.

The scope hierarchy and current-session handling are **Requires-backend**. Any
new or changed public binding contract conditionally requires
P10-T02/Lane-CTR; a Personal-private projection may not.

## Usage, privacy, and alerts

No prompt, completion, key, request header, or reversible payload is retained.
Per-call events are retained 30 days; queryable aggregates 90 days.

Token fields are nullable/unknown when unavailable; unknown is not zero.
`metering_source` is `provider_reported`, `locally_estimated`, or `unavailable`.
Estimation records its method. Monthly token and monetary budgets may target an
account or agent. A period emits one deduplicated `warning` at 80% and one
`exceeded` at 100%; alerts are queryable/audited and never block or reroute.
Usage queries support time range, account, provider, model, agent, and outcome
filters. Cache hit is represented by `cache_read_tokens`; a hit rate is shown
only when the provider denominator semantics are known, otherwise raw counters
and an `unknown` rate are returned.

Personal 2.0 keeps three readings separate:

- **quota:** Provider account/subscription allowance and reset facts, only when
  the Provider supplies them;
- **usage:** measured or estimated consumption with source and period;
- **cost:** priced usage with price version, currency/basis, and
  unavailable/estimated status.

Quota is not inferred from usage, usage is not inferred from cost, and missing
data is not rendered as zero. A percentage or rate appears only when the
Provider denominator is known and shown. Broader subscription quota ingestion
is **Requires-backend**.

## Current deterministic CLI

```text
cognitive provider account create|list|show|update|delete
cognitive provider key set|rotate|remove
cognitive provider models refresh|list|add|set-price
cognitive agent binding set|show|list|remove
cognitive usage query
cognitive budget set|list|remove
cognitive alerts list|acknowledge
cognitive audit query
```

Shipped `cognitive usage query` and `cognitive audit query` take no filters.
Exact flags live with the CLI implementation and the handbook usage page.

The CLI projects the authenticated daemon service and never opens SQLite or
resolves secrets. Responses contain IDs, redacted endpoint metadata, usage
source, cost state, and stable errors only.

## Adopted Account Hub provider order

The first Account Hub screen presents:

1. OpenAI;
2. Anthropic;
3. Google;
4. DeepSeek.

The expanded list presents Qwen/Bailian, Kimi, Zhipu, SiliconFlow,
Volcengine-Doubao, MiniMax, and OpenRouter. Custom OpenAI-compatible is a
first-class choice rather than a hidden "other" form.

A visible preset is a product choice, not a claim that its adapter,
subscription method, quota API, or Agent path is implemented or qualified.

## Capability gaps and fixed boundaries

### Backend absent

- subscription/OAuth and refresh lifecycle;
- ADR-0055 existing-credential import implementations;
- target Provider presets beyond current adapters;
- global/Agent/conversation profile hierarchy;
- explicit current-session rebind/restart coordination;
- broader Provider quota ingestion.

### API exists, UI-dark or partial

Current account, key, catalog, binding, usage, budget, alert, and audit
authority already backs the current UI. Account Hub regrouping can reuse those
facts. Native Agent account/session state may exist at the vendor surface but
is not automatically a Personal account fact.

### Contract/core gap

Only a new or changed public account/profile/override or subscription machine
contract conditionally requires P10-T02/Lane-CTR. Product prose does not define
its shape, and a Personal-private projection may not require core changes.

### Never weakened

- no raw credential retrieval by browser, Agent, adapter, Shell, or MCP server;
- no ambient Provider fallback, load balancing, arbitrary auth headers, or
  silent current-session switching;
- no prompt/completion retention in the usage ledger;
- no Provider success, quota state, or process result as Task completion;
- no multi-user administration or remote public control plane in Personal.

This design makes no Provider-quality, Gate, release, Profile, performance,
containment, or Agent-benefit claim.
