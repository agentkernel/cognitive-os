# LLM Provider Control Plane

## Status and authority

- Status: current Provider authority plus adopted OPC evolution
- Product: `cognitiveos-personal`
- Current-status owner: [PROGRESS.md](../../../docs/plan/PROGRESS.md)
- Target experience: [Account Hub](account-hub.md)
- Credential boundary:
  [ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md)

The Rust daemon remains the only component allowed to resolve Provider
credentials or perform Personal-managed Provider egress. UI, Personal
Assistant, digital employees, DSH, Pi, adapters, MCP servers, and Vault tools
never receive raw secret material.

## 1. Current implementation (Now)

The current Provider Control Plane supports:

- named OpenAI, Anthropic, and custom OpenAI-compatible accounts;
- API-key one-way handoff into approved SecretStore custody;
- fixed official endpoints and explicitly reviewed custom endpoint trust;
- model discovery and manual models;
- one revision-guarded account/provider/model binding per Agent instance;
- no fallback;
- source-labelled usage/cost, advisory budgets/alerts, and audit;
- daemon-served Provider and binding UI.

Current account creation, key rotation/removal, model refresh, binding changes,
usage, budget, alert, and audit operations remain daemon-owned. A failed model
refresh preserves the last catalog and active binding. Missing price yields
`cost_unavailable`; unknown tokens are not zero.

Exact CLI/API behavior belongs to implementation and generated handbook
references. This product document does not redefine it.

## 2. Adopted Personal 2.0 target

The OPC target groups Provider facts in Settings > Providers and Account Hub:

```text
account/authentication
  + endpoint/trust
  + model catalog
  + effective binding
  + Project/member/Task budget
  + Provider quota
  + actual usage/cost
```

Effective binding becomes global -> Project -> employee -> Task. Subscription,
OAuth/API account, API billing/quota, and consumer-product entitlement remain
separate. DSH and Pi use only a Task-scoped effective route through the daemon
proxy.

The hierarchy, subscription/OAuth observations, concrete credential import,
additional adapters, quota integration, and hard budget enforcement are
**Requires-backend**.

## 3. Endpoint and credential controls

- Official Provider endpoints are fixed by the qualified adapter.
- Custom OpenAI-compatible endpoints receive explicit trust review for LAN,
  private-network, loopback, or insecure HTTP scope.
- Embedded credentials, redirects to disallowed origins, arbitrary headers,
  caller-supplied credential paths, and implicit URL rewriting fail closed.
- ADR-0055 import is user-initiated, per-source consented, daemon-owned,
  non-logging, and defaults to retaining the source.
- Secret material never enters Agent config, DSH/Pi environment, ordinary
  config, SQLite, argv, logs, Context, Memory, Conversation, evidence, or chat.

Account import success is not Provider reachability, model availability,
entitlement, or Agent readiness.

## 4. Routing and budget behavior

The narrower admitted binding wins; no caller chooses an arbitrary route.
Changing a binding states whether it affects new Tasks or requires current
runtime rebind/restart. Existing work never switches silently.

Budget enforcement must:

1. evaluate the Project, member, and Task envelopes before new dispatch;
2. preserve in-flight and unknown Effect reconciliation;
3. stop and create an Inbox item at the declared boundary;
4. require an admitted adjustment rather than silently rerouting;
5. retain the actual metering source and period.

Current budgets are advisory and must not be presented as enforced.

## 5. Usage and quota honesty

Quota, usage, and cost are three readings:

- quota: Provider-reported allowance/reset/source when available;
- usage: Provider-reported or locally estimated consumption with method;
- cost: pricing version, currency/basis, and estimated/reported/unavailable.

No prompt, completion, key, header, or reversible payload belongs in the usage
ledger. Counts, percentages, and remaining fractions appear only with a
declared denominator. Missing quota or price stays unavailable.

## 6. Required states and non-claims

The product distinguishes usable, degraded, credential missing/revoked,
SecretStore locked, endpoint trust required, model unavailable, quota unknown,
budget warning/stopped, stale catalog, rebind required, and outcome unknown.
Errors preserve non-secret input and state whether retry is safe.

This evolution does not implement Provider adapters, DSH/Pi proxy changes,
binding hierarchy, budget enforcement, quota, Windows support, or qualification.
It makes no Provider-quality, cost-accuracy, Gate, release, Profile, business,
or Agent-benefit claim.
