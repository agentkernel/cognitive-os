# LLM Provider Control Plane

## Status and authority

This is an owner-approved product design proposal for `cognitiveos-personal`.
It is planning material, not an implementation, release, Gate, Profile,
provider-quality, or agent-benefit claim. Formal task IDs and current status
remain owned by [the Personal plan](../../plan/PERSONAL-DEVELOPMENT-PLAN.md) and
[PROGRESS.md](../../plan/PROGRESS.md). This design extends existing
Provider/Secret Store/daemon/Pi work; it creates no second authority writer.

## Product outcome

The owner-local control plane lets one user manage named OpenAI and Anthropic
accounts, rotate API keys without exposing key material, discover or manually
register models, bind an installed agent to one fixed account/model, inspect
input/output/cache token usage, and receive cost/soft-budget alerts. The first
delivery is daemon API, durable metadata, audit, usage queries, and CLI. A web
or desktop panel is deferred. Operator usage of the shipped CLI is in
[`handbook/en/user/provider-control-plane.md`](../../../handbook/en/user/provider-control-plane.md)
(zh-CN:
[`handbook/zh-CN/user/provider-control-plane.md`](../../../handbook/zh-CN/user/provider-control-plane.md)).
This page remains the product-design record and does not copy current task or Gate
status.

## Scope

| Surface | First-delivery decision |
|---|---|
| Official providers | OpenAI and Anthropic |
| Custom providers | OpenAI-compatible endpoints only |
| Named agents | Pi agent and DeepSeek harness |
| Qualification | Pi remains the current qualified path; DeepSeek requires independent adapter validation and does not inherit Pi evidence |
| Credential | API key only |
| Accounts | Multiple named accounts per provider |
| Binding | One fixed account + provider + model per agent instance |
| Fallback | None; failures are returned and audited |
| Cost control | Estimate and soft alert; no blocking |
| UI | CLI/API first |

OAuth, browser login, refresh tokens, multi-user RBAC, automatic routing/load
balancing, hard budget blocking, and arbitrary Anthropic-compatible endpoints
are out of scope.

## Account and endpoint trust

```text
ProviderAccount
  id, display_name
  provider_kind       # openai_official | anthropic_official | openai_compatible
  endpoint, secret_ref
  endpoint_trust, status, catalog_revision
  created_at, updated_at
```

The account stores only an opaque Secret Store reference, never the API key.
Names are unique within the owner-local instance. Key rotation changes the
Secret Store item, not the account identity or historical usage identity.

Creation is ordered as `validate -> persist intent -> store key -> discover
models -> verify`. A discovery failure leaves the account `degraded` and
auditable; it does not invalidate an existing binding. A missing or removed key
makes the account `revoked` and non-callable until repaired. An account with
active bindings cannot be deleted.

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

```text
AgentBinding
  agent_instance_id, provider_account_id, provider_kind, model_id
  binding_revision, status
```

Each agent has at most one active binding. Requests cannot select another
provider, model, or account. Errors are stable, returned, and audited; there is
no fallback. Agents do not read the Secret Store. The preferred path is the
existing daemon provider proxy/session boundary; any adapter token is short-
lived, binding-scoped, and contains no provider key. Pi and DeepSeek adapters
are validated independently.

## Usage, privacy, and alerts

No prompt, completion, key, request header, or reversible payload is retained.
Per-call events are retained 30 days; queryable aggregates 90 days.

```text
event_id, timestamp, agent_instance_id, provider_account_id, provider_kind
model_id, input_tokens, output_tokens, cache_read_tokens, cache_write_tokens
duration_ms, outcome, metering_source, estimation_method, pricing_version
cost_status
```

Token fields are nullable/unknown when unavailable; unknown is not zero.
`metering_source` is `provider_reported`, `locally_estimated`, or `unavailable`.
Estimation records its method. Monthly token and monetary budgets may target an
account or agent. A period emits one deduplicated `warning` at 80% and one
`exceeded` at 100%; alerts are queryable/audited and never block or reroute.
Usage queries support time range, account, provider, model, agent, and outcome
filters. Cache hit is represented by `cache_read_tokens`; a hit rate is shown
only when the provider denominator semantics are known, otherwise raw counters
and an `unknown` rate are returned.

## Management CLI

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

## Open-source reference decision

Cockpit is an interaction reference for local status/forms/tables; CC Switch is
a reference for named-provider UX. Their browser/session, proxy, privilege,
and credential implementations are not imported. Any source reuse requires a
separate license, provenance, dependency, and security review.

## Non-goals

Web/desktop implementation, OAuth, multi-user administration, approvals,
fallback/routing/load balancing, hard limits, arbitrary auth headers,
Anthropic-compatible custom endpoints, background discovery, and
prompt/completion retention.
