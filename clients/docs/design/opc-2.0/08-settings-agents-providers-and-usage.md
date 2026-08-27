# 08 — Settings, Installed Agents, Providers, and Usage

## Settings information architecture

Settings is a searchable grouped destination:

1. Personal Home;
2. Installed Agents;
3. Providers and Accounts;
4. Binding, Budgets, and Usage;
5. Notifications;
6. Privacy and Recovery;
7. Diagnostics;
8. Advanced capabilities.

Each group has unsaved/saved/stale/conflict/permission/error states and an
audit link for consequential changes.

## Installed Agents

DSH appears as:

```text
DeepSeek Harness
Preinstalled · Managed by Personal · Default employee runtime
```

Detail shows official artifact source, exact version/digest/license,
adapter/broker compatibility, health, Windows sandbox qualification, bounded
capabilities, employee/Task usage, update, active/rollback slot, and retention.

There is no native DSH UI or conversation link. Employee conversations belong
to Personal. DSH native MCP/base tools, HMR, home patch, env/plaintext
credentials, and direct Provider traffic are forbidden.

Pi does not appear in the ordinary list. Advanced Personal Assistant
diagnostics may show the exact Pi engine, health, candidate-only boundary, and
current product pin.

Hermes, Codex, Cursor, and other entries appear only in a **Future adapter
candidates** explanation, never as installable/supported controls.

## Providers and accounts

The surface separates subscription, authentication account, API billing/quota,
model catalog, binding, budget, and usage. Secret inputs are one-way daemon
handoffs and never echoed or stored in the browser.

Effective binding:

`global -> Project -> employee -> Task`

The view explains which route wins, affected running/future work, and whether a
runtime restart is required. A Role Blueprint cannot hold a Provider binding.
No silent fallback or load balancing is implied.

## Budgets and usage

Budget sections show Project total, employee allocation, Task cap, warning/stop
policy, actual usage, and Provider quota. Current advisory budgets remain
labelled advisory. Enforcement is `Requires-backend`.

Quota, usage, and cost carry source, period, pricing version/method, and
unknown/unavailable state. Unknown is not zero; percentages require a
denominator.

## Personal Home and recovery

Shows selected root and conceptual `app/`/`data/` separation, Project
directories, storage permission, local restore points, manual export, archive,
and diagnostics opt-in. Same-disk restore points are visibly **not disaster
backup**. Export excludes secrets by default.

## Forms and consequences

- visible labels and constraints;
- non-secret values retained after failure;
- exact SecretStore/endpoint/account/model failure class;
- scope and affected Projects/employees/Tasks before save;
- structured review for binding, permission, budget, update, rollback, import,
  archive, restore, or deletion;
- daemon-issued confirmable preview only;
- keyboard focus to errors and status announcement;
- no color-only or hover-only state.

## States

Settings scenes include loading, partial, stale, permission, SecretStore
locked, credential revoked, endpoint unavailable, model unknown, quota
unavailable, budget stopped, DSH artifact drift, sandbox unqualified, update
failed/rollback available, restore conflict, diagnostics off, saved receipt,
and Requires-backend.

## Evidence boundary

Current Provider and P7-T05 surfaces are factual foundations. This design does
not implement the OPC grouping, DSH Windows supply chain, Pi Assistant
diagnostics, binding hierarchy, hard budgets, quota, Personal Home, restore,
or future adapters.
