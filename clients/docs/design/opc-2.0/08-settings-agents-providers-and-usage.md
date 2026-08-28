# 08 — Settings, Model Connections, capabilities, and diagnostics

- Requirements:
  [OPC requirements analysis](../../../../personal/docs/product/personal-2.0-opc-requirements-analysis.md)
- Product sources:
  [Model Connections](../../../../personal/docs/product/account-hub.md) and
  [MCP capability governance](../../../../personal/docs/product/mcp-resource-family.md)
- Status: Owner-accepted V2 interaction baseline (2026-08-28 competitive-informed
  overwrite; not a v3; not overlay-conversation / stacked-column V2)
- Interaction baseline:
  [**Owner-approved interaction baseline (2026-08-28)**](personal-20-ai-ceo-e2e-optimized-v2.canvas.tsx)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

## Settings information architecture

Settings is fixed at the bottom and grouped as:

1. Personal Home;
2. Model Connections;
3. Cost & Alerts;
4. Notifications;
5. Privacy & Recovery;
6. Advanced Diagnostics.

Project capability acquisition usually starts from a setup or work need, not
from Settings. Settings can inspect acquired artifacts, versions, grants, and
review status without becoming a broad marketplace.

## Model Connections

Mainstream Providers use quick templates: choose Provider, enter key through a
one-way SecretStore takeover (operator-style; the key never appears in chat or
the canvas), discover/select model, verify redacted facts, and
save a receipt. Advanced setup accepts explicit custom URL, compatibility
mode, key, and model.

Endpoint trust, compatibility, credential custody, reachability, model
availability, Provider quota, usage, and cost are separate facts. Consumer
subscription, invoice, plan, and product billing management are outside 2.0.

Every Project Member requires an explicit Provider/model selection. The
Assistant may recommend but cannot bind silently. A Role Runtime Template
declares model capabilities but contains no concrete credential or connection.
A later route change is a versioned Member/Task revision with affected work and
restart consequence. There is no hidden fallback, ambient load balancing, or
silent rebind.

## Secret boundary

Raw secret material enters only an approved SecretStore through a non-logging
daemon path. Chat never collects or displays a key. It never appears in DOM, URL, browser storage, Agent/Member
configuration, DSH/Pi environment, MCP metadata, Context, Memory, Conversation,
Vault, ordinary config, SQLite, argv, logs, evidence, or export. Connection
save does not prove reachability, entitlement, model availability, or quota.

## Cost, usage, and quota

Cost is labelled:

- **actual** — Provider-reported or directly metered with source/period;
- **estimated** — pricing/model/method/version/scope declared;
- **unknown** — no conclusion; never displayed as zero;
- **warning** — threshold or variance requiring attention.

Personal 2.0 does not automatically stop work at a product budget threshold.
Provider quota, credential failure, or unavailability may still fail an
external call. Current advisory-budget behavior is a factual implementation
foundation, not the target policy.

## Skill and MCP acquisition

The Assistant may discover a capability from a Project need:

1. Skill review records source, exact version, digest, license, maintainer,
   hidden instructions, prompt injection, and file/network/command intent.
2. MCP adds dependencies, executable code, supply chain, destinations,
   Secret/model/Tool permissions, compatibility, removal, and rollback.
3. A Skill may install automatically only after passing review.
4. First MCP installation or any permission expansion requires Owner
   confirmation of exact version and permissions.
5. Acquisition creates a reusable pinned artifact; each Project/Member receives
   an independent least-privilege grant.
6. Updates repeat review and compatibility tests and retain rollback.

Installation or connection grants no Tool, Context, workspace, network,
command, model, secret, Memory, or authority by implication.

## Hidden engines and advanced diagnostics

DSH and Pi are absent from everyday navigation. Advanced diagnostics may show
exact artifact/version/digest/license, broker compatibility, health, Windows
sandbox qualification, bounded capabilities, affected Members/Tasks,
update/rollback, and retention only when needed for recovery.

There is no engine store, alternate Harness selector, native DSH/Pi UI, or
native conversation synchronization. DSH native MCP/base tools, HMR, home
patch, ambient credentials, and direct Provider traffic remain forbidden.

## Personal Home, recovery, forms, and states

Personal Home shows conceptual `app/`/`data/` separation, Project directories,
storage permission, local restore points, manual export, archive, and
diagnostics opt-in. Same-disk restore points are not disaster backup; export
excludes secrets.

Forms preserve non-secret values, show exact endpoint/credential/model/
permission failures, list affected Projects/Members/Tasks, and use
daemon-issued previews for consequential version, grant, update, rollback,
restore, archive, or deletion changes.

States include empty, loading, partial, stale, permission, SecretStore locked,
credential revoked, endpoint unavailable, compatibility/model/quota unknown,
cost warning, review required, grant required, artifact drift, sandbox
unqualified, compatibility failed, rollback available, restore conflict,
diagnostics off, receipt, `Requires-backend`, and `Requires-environment`.
Missing connection, install, or confirmation paths are labelled gaps; the
prototype draws no Connect / Install / Confirm fake buttons.

Current Provider and Linux-era UI surfaces are factual foundations only. The
target grouping, Member routing, capability review/acquisition, Windows engine
diagnostics, quota/cost composition, Personal Home, and recovery are not
implemented by this design.
