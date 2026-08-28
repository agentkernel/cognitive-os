# Personal Model Connections

- Status: adopted Personal 2.0 target over a current Provider foundation
- Canonical language: English
- Current authority foundation:
  [Provider Control Plane](provider-control-plane.md)
- Requirements:
  [OPC requirements analysis](personal-2.0-opc-requirements-analysis.md)
- Current interaction prototype:
  [**personal-20-opc-e2e (post journey-subtraction)**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e.canvas.tsx)
- Archived historical V2 (not current chrome):
  [pre-subtraction history](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- Prototype identity: current chrome is the post-workshop canvas. Archived V2
  is not current chrome. Canvas-only HITL and daemon authority path remain.
- Credential boundary:
  [ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md)
- Chinese mirror: [account-hub.zh-CN.md](account-hub.zh-CN.md)

## 1. Product boundary

Model Connections keeps these facts separate:

| Fact | Meaning |
|---|---|
| Connection | admitted Provider endpoint, compatibility mode, and account identity |
| Secret custody | opaque SecretRef in an approved SecretStore |
| Model catalog | models available to an account at an observed time |
| Member selection | explicit admitted Provider/model route for one Project Member |
| Provider quota | externally reported allowance/reset/availability when available |
| Usage/cost | source-labelled actual or estimated consumption |

Consumer subscription, invoice, plan, and product billing management are not
Personal 2.0 features. Credential presence does not prove reachability. Quota
is not inferred from usage. Unknown usage/cost is not zero.

## 2. Reality ledger

| Boundary | Connection truth |
|---|---|
| **Current implementation (Now)** | Named OpenAI, Anthropic, and custom OpenAI-compatible accounts; API-key SecretStore handoff; model discovery/manual models; fixed Agent binding; usage/cost; advisory budgets/alerts; audit and current Provider UI. |
| **Adopted Personal 2.0 target** | Settings > Model Connections with mainstream quick templates, advanced custom connection, explicit Provider/model selection for every Member, DSH/Pi daemon proxy, and honest quota/usage/cost. |
| **Requires-backend** | Additional adapters, custom compatibility modes, concrete SecretStore readers, Member selection/revision, broader quota, source-labelled cost composition, and runtime rebind/restart. |

Current fixed Agent bindings and advisory budgets remain factual foundations;
they do not define the 2.0 product organization or an automatic stop policy.

## 3. Connection creation

The everyday flow is:

```text
choose mainstream Provider template
  -> enter key through one-way SecretStore handoff
  -> discover/select model
  -> verify redacted endpoint/account/model facts
  -> save connection receipt
```

Advanced setup accepts an explicit custom URL, compatibility mode, key, and
model. Endpoint trust, compatibility, credential, reachability, and model
availability are separate checks. There is no consumer-subscription or invoice
flow.

## 4. Explicit Member selection

Creating every Project Member requires the Owner to choose a Provider/model.
The Assistant may explain requirements and recommend a route but cannot bind
silently. A Role Runtime Template declares model capabilities, not a concrete
connection or credential.

A later change is a versioned Member/Task revision that states affected work
and whether a process restart is required. Existing work never switches
silently. There is no ambient load balancing, hidden fallback, caller-supplied
credential, or arbitrary auth header.

## 5. Secret and proxy boundary

All Personal-managed methods terminate in an approved SecretStore and daemon
proxy:

- user-initiated API-key SecretStore takeover (the key never appears in chat
  or the canvas);
- per-source ADR-0055 credential import;
- custom endpoint with explicit trust review.

Raw material never enters UI storage, DOM, URL, Agent/Member configuration, DSH,
Pi, MCP, Vault, Conversation archive, ordinary config, SQLite, argv,
environment, logs, evidence, or chat. Import success means only SecretStore
custody; Provider/model/quota checks remain separate.

## 6. Cost, quota, and usage

| Control | Product target |
|---|---|
| Provider quota | provider-reported allowance/reset/source, if available |
| Actual cost | Provider-reported or directly metered value with source/period |
| Estimated cost | declared model/pricing/method/version and scope |
| Unknown cost | unavailable conclusion that must never render as zero |
| Cost warning | visible threshold/variance signal for Owner/manager attention |

Personal 2.0 does not automatically stop work at a product budget threshold.
Warnings do not erase an in-flight unknown Effect or invent a cheaper route.
Provider quota, credential failure, or Provider unavailability may still cause
an external failure. Current advisory-budget facts remain labelled as current
implementation behavior, not the 2.0 target policy.

## 7. Setup and recovery

Model Connection setup:

1. select a mainstream Provider template or advanced custom connection;
2. complete the non-logging secret path;
3. inspect redacted endpoint, account, model, quota, and trust facts;
4. run bounded reachability/credential/model checks separately;
5. save and return to the originating Member creation or Settings surface;
6. explicitly select the connection/model for the Member.

Input is preserved after recoverable failure. Locked SecretStore, expired
credential, unreachable endpoint, model missing, quota unknown, stale catalog,
cost warning, and rebind-required are distinct states.

## 8. Required states and non-claims

Model Connections covers empty, loading, partial, stale, permission, error,
unknown, offline, cost-warning, quota-unavailable, success, and archived
states. Every reading carries source and period; every percentage carries a
denominator.

The expanded Model Connections, Member selection/revision, DSH/Pi proxy
composition, and quota/cost integration are **Requires-backend**. This document
makes no Provider quality, entitlement, cost-accuracy, support, Gate, release,
Profile, or business-outcome claim.
