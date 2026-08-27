# Personal Account Hub

- Status: adopted Personal 2.0 target over a current Provider foundation
- Canonical language: English
- Current authority foundation:
  [Provider Control Plane](provider-control-plane.md)
- Credential boundary:
  [ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md)
- Chinese mirror: [account-hub.zh-CN.md](account-hub.zh-CN.md)

## 1. Separate facts

Account Hub must not collapse these concepts:

| Fact | Meaning |
|---|---|
| Consumer subscription | entitlement in a vendor consumer product |
| Provider account/auth | API key, OAuth, or approved imported credential identity |
| API billing/quota | allowance, reset, limits, invoice/pricing basis |
| Model catalog | models available to an account at an observed time |
| Binding | admitted account/provider/model route for a scope |
| Budget | Personal's Project/member/Task ceiling |
| Usage/cost | source-labelled actual or estimated consumption |

Subscription does not prove API entitlement. Credential presence does not
prove reachability. Quota is not inferred from usage. Unknown usage/cost is not
zero.

## 2. Reality ledger

| Boundary | Account truth |
|---|---|
| **Current implementation (Now)** | Named OpenAI, Anthropic, and custom OpenAI-compatible accounts; API-key SecretStore handoff; model discovery/manual models; fixed Agent binding; usage/cost; advisory budgets/alerts; audit and current Provider UI. |
| **Adopted Personal 2.0 target** | Settings Account Hub with account/subscription/billing separation, global/Project/employee/Task binding, Project/member/Task budgets, DSH/Pi daemon proxy, and honest quota/usage. |
| **Requires-backend** | Additional adapters, OAuth/subscription observation, concrete ADR-0055 import readers, binding hierarchy, budget enforcement, broader quota, and runtime rebind/restart. |

## 3. Effective binding

The admitted route resolves in this order:

```text
global default
  -> Project default
  -> digital employee override
  -> Task temporary override
```

The narrowest admitted value wins. A Role Blueprint declares capability needs
but never stores a concrete Provider binding. A change states which future and
current Tasks are affected and whether a DSH/Pi runtime restart is required.
There is no silent fallback, ambient load balancing, caller-supplied credential,
or arbitrary auth header.

## 4. Secret and proxy boundary

All Personal-managed methods terminate in an approved SecretStore and daemon
proxy:

- user-initiated API-key handoff;
- OAuth/subscription token lifecycle when implemented;
- per-source ADR-0055 credential import;
- custom endpoint with explicit trust review.

Raw material never enters UI storage, URL, Agent/employee configuration, DSH,
Pi, MCP, Vault, Conversation archive, ordinary config, SQLite, argv,
environment, logs, evidence, or chat. Import success means only SecretStore
custody; Provider/model/quota checks remain separate.

## 5. Budgets, quota, and usage

| Control | Product target |
|---|---|
| Project budget | total approved envelope and warning/stop policy |
| Member budget | employee allocation inside the Project envelope |
| Task budget | temporary maximum for one governed Task |
| Provider quota | provider-reported allowance/reset/source, if available |
| Actual usage | Task/member/Project/account/model attribution and metering source |

Budget enforcement stops new dispatch at the defined boundary and sends the
adjustment request to Inbox. It does not erase an in-flight unknown Effect or
invent a cheaper route. Current advisory budgets remain labelled advisory
until enforcement is implemented and validated.

## 6. Setup and recovery

Account setup:

1. select a supported Provider/custom endpoint and method;
2. complete the non-logging secret path;
3. inspect redacted endpoint, account, model, quota, and trust facts;
4. choose global/Project/employee/Task scope;
5. run bounded reachability/credential/model checks separately;
6. save and return to the originating Project/employee/Task.

Input is preserved after recoverable failure. Locked SecretStore, expired
credential, unreachable endpoint, model missing, quota unknown, stale catalog,
budget exceeded, and rebind-required are distinct states.

## 7. Required states and non-claims

Account Hub covers empty, loading, partial, stale, permission, error, unknown,
offline, budget-warning/stopped, success, and archived-account states. Every
reading carries source and period; every percentage carries a denominator.

The expanded Account Hub, binding hierarchy, budget enforcement, DSH/Pi proxy
composition, and quota integration are **Requires-backend**. This document
makes no Provider quality, entitlement, cost-accuracy, support, Gate, release,
Profile, or business-outcome claim.
