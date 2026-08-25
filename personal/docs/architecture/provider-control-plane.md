# Provider Control Plane Architecture

## Position and reuse

The control plane is a Rust daemon application service. It reuses the existing
Provider/Secret Store, bounded Personal daemon, local management session, CLI,
and Pi provider-proxy direction (P1-T02, P1-T03, P1-T04, P1-T06, P1-T07).
It is not a new daemon, authority writer, generic Resource DTO, or public
contract until formal task and contract review approve one.

```text
Owner-local CLI / management session
                |
        Personal management API
                |
 Provider account + binding service
       |                    |
 Secret Store          Runtime broker/proxy
       |                    |
 Discovery/transport   Pi / DeepSeek adapters
                |
       Usage normalizer + audit journal
                |
      Usage ledger -> aggregates -> budgets/alerts
```

The daemon alone writes account metadata, bindings, usage, budgets, alerts,
audit facts, and Secret Store references. This document creates no evidence,
Gate, Profile, release, or provider-quality claim. Shipped operator usage of the
CLI (no Web or desktop panel in this phase) is in
[`personal/handbook/en/user/provider-control-plane.md`](../../handbook/en/user/provider-control-plane.md)
(zh-CN:
[`personal/handbook/zh-CN/user/provider-control-plane.md`](../../handbook/zh-CN/user/provider-control-plane.md)).

## Mutation and authority sequence

For account/key/trust/catalog/binding/budget/retention mutations: authenticate
the owner-local session; validate and normalize; persist an Intent and
idempotency key; dispatch the minimum external effect; persist Effect and
independent verification; update the projection and append a redacted audit
fact. A caller cannot observe success before durable state and Effect outcome
commit. Failed discovery preserves the last catalog and binding.

## Logical records

These are logical records, not permission to add public schemas without the
contract lane:

```text
provider_accounts: account_id, name, kind, normalized_endpoint, secret_ref,
  trust_grant, status, catalog_revision, timestamps
provider_models: account_id, model_id, source, capability_snapshot,
  pricing_version, four token prices
agent_provider_bindings: agent_instance_id, account_id, model_id, revision, status
llm_usage_events: identity, four token categories, duration, outcome,
  metering_source, estimation_method, pricing_version, cost_status
llm_usage_aggregates: period, account/agent/model dimensions, sums, source summary
llm_budgets / llm_alerts: scope, period, limits, threshold state, dedupe key
```

Secret material is never a column. Approved Personal migration/application
services own placement; no ad hoc second database.

## Endpoint and SSRF policy

Before reading a key or dispatching a request, the evaluator rejects userinfo,
fragments, ambiguous authorities, unsupported schemes, redirects, proxy
environment inheritance, caller paths, and arbitrary headers. It requires the
account trust grant for HTTP/loopback/LAN/private ranges; resolves DNS and
validates every result; pins the validated destination for the request; bounds
DNS/connect/TLS/response/total time and response size; and records a redacted
policy decision. Official endpoints are immutable. Custom endpoints expose only
bounded OpenAI-compatible `/v1/models` and inference operations.

Custom auth is Bearer-only. Official adapters have fixed native auth where the
provider requires it; callers cannot inject headers.

## Adapters, discovery, and usage normalization

Adapters are OpenAI official, Anthropic official, and OpenAI-compatible custom.
Creation performs one explicit discovery; compatible discovery is
`GET /v1/models`; refresh is foreground-only. Results are size-bounded,
schema-validated, revisioned, and source-labelled. Manual models are allowed
after failure or omission.

Known provider fields map to `input_tokens`, `output_tokens`,
`cache_read_tokens`, and `cache_write_tokens`. Missing/ambiguous fields are
unknown, never fabricated as zero. Estimation records its method and is never
presented as provider-reported usage. Query projections support time range,
account, provider, model, agent, and outcome filters. A cache-hit rate is
derived only where the provider defines a valid denominator; otherwise the
projection exposes raw cache counters and an unknown rate.

## Secret and runtime boundary

The Secret Store adapter owns set/rotate/lookup/remove. Plaintext exists in
memory only for the minimum egress operation and never in SQLite, ordinary
config, argv, environment, service-unit material, CLI, logs, fixtures, or
evidence.

```text
agent -> scoped local session/proxy
      -> daemon resolves binding and Secret Store key
      -> daemon performs provider egress
      -> daemon normalizes usage and returns redacted result
```

Any runtime token is short-lived, scoped to agent/account/model/binding
revision, revocable or expiring, and contains no provider key. Pi and DeepSeek
harness adapters are independent qualification paths.

## Ledger, pricing, retention, and alerts

The ledger accepts one normalized event after a response or definitive failure;
event identity/idempotency prevents duplicate accounting. It stores no content.
Per-call events expire at 30 days; aggregates remain 90 days. Cleanup is
bounded, idempotent, and audited. Price version is captured at calculation
time, so later edits cannot rewrite historical cost. Missing price is
`cost_unavailable` and is not treated as zero.

Monthly account/agent budgets emit one deduplicated 80% `warning` and 100%
`exceeded` alert per period. Alerts are observability only and never authorize,
block, or reroute a request.

## Private management projection

Expected versioned routes behind the existing management channel are:

```text
/management/providers/accounts
/management/providers/accounts/{id}/key
/management/providers/accounts/{id}/models[/refresh]
/management/agent-bindings
/management/usage
/management/budgets
/management/alerts
/management/audit
```

Exact envelopes, errors, pagination, and versioning follow existing Personal
contracts and Lane-CTR. Responses expose IDs, redacted endpoint metadata,
usage source, cost state, and stable errors; never keys, bearers, prompts,
completions, or unredacted sensitive paths.

## Failure invariants

- Missing/removed/expired/mismatched credentials fail closed.
- Unbound agents are denied; active bindings block account deletion.
- Endpoint policy fails before key disclosure or provider dispatch.
- Provider errors are audited and returned without fallback.
- Failed refresh preserves catalog and binding.
- Unknown usage and unavailable pricing remain explicit.
- Provider success, Pi events, and process exits do not complete a Task;
  existing independent verification remains authoritative.
