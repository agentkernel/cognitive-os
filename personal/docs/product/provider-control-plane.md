# LLM Provider Control Plane

## Status and authority

- Status: current Provider authority plus adopted OPC evolution
- Product: `cognitiveos-personal`
- Current-status owner: [PROGRESS.md](../../../docs/plan/PROGRESS.md)
- Target experience: [Model Connections](account-hub.md)
- Requirements:
  [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Current interaction prototype:
  [**personal-20-opc-e2e-optimized-v5**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e-optimized-v5.canvas.tsx)
- Archived (not current chrome):
  [pre-v5-approval](../../../clients/docs/design/opc-2.0/history/2026-08-29-pre-v5-approval/README.md);
  [pre-subtraction V2](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Credential boundary:
  [ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md)

The Rust daemon remains the only component allowed to resolve Provider
credentials or perform Personal-managed Provider egress. UI, Personal
Assistant, Project Members, Agent processes, DSH, Pi, adapters, MCP servers,
and Vault tools
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

The OPC target groups Provider facts in Settings > Model Connections:

```text
connection/account
  + endpoint/trust
  + compatibility mode
  + model catalog
  + explicit Project Member selection
  + Provider quota
  + source-labelled actual/estimated/unknown usage and cost
```

Mainstream Providers use quick templates where the Owner enters a key.
Advanced setup accepts custom URL, compatibility mode, key, and model. Every
Member creation requires an explicit Provider/model choice. The Assistant may
recommend but cannot bind or rebind silently. DSH and Pi use only an admitted
Task-scoped route through the daemon proxy.

Consumer subscription, plan, invoice, and product billing management are
outside 2.0. Custom compatibility setup, concrete credential import,
additional adapters, Member revision, quota integration, and cost attribution
are **Requires-backend**.

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

## 4. Member routing and cost behavior

The Owner explicitly selects a Provider/model when creating each Member. A
Role Runtime Template declares model capabilities but stores no concrete
connection or secret. Changing a Member or Task route states the affected work
and whether a process restart is required. Existing work never switches
silently, and no caller chooses an arbitrary route.

Personal shows cost as actual, estimated, or unknown with source and period.
It may warn on a threshold or variance, but Personal 2.0 does not automatically
stop work at a product budget threshold. Provider quota, credential failure, or
unavailability may still block the external call. In-flight and unknown Effects
continue reconciliation; cost pressure never authorizes silent rerouting.

Current advisory-budget behavior remains a factual implementation foundation
and must not be presented as the 2.0 product policy.

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
SecretStore locked, endpoint trust required, compatibility unknown, model
unavailable, quota unknown, cost warning, stale catalog, rebind required, and
outcome unknown.
Errors preserve non-secret input and state whether retry is safe.

This evolution does not implement Provider adapters, DSH/Pi proxy changes,
Member routing, cost composition, quota, Windows support, or qualification.
It makes no Provider-quality, cost-accuracy, Gate, release, Profile, business,
or Agent-benefit claim.
