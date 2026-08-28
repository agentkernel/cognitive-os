# 11 — Design-to-code trace and capability matrix

- Requirements:
  [OPC requirements analysis](../../../../personal/docs/product/personal-2.0-opc-requirements-analysis.md)
- Status: design trace only; architecture/formal-plan/handbook reconciliation
  pending; Owner-accepted V2 interaction baseline (2026-08-28
  competitive-informed overwrite; not a v3; not overlay-conversation /
  stacked-column V2)
- Interaction baseline:
  [**Owner-approved interaction baseline (2026-08-28)**](personal-20-ai-ceo-e2e-optimized-v2.canvas.tsx)
- Not-run validation: Canvas runtime/render, NVDA, host-theme contrast, and
  200% real layout
- Evidence boundary: Owner approval is not usability, accessibility, backend,
  Gate, release, qualification, or acceptance evidence

## Evidence classes

| Class | Meaning |
|---|---|
| Current | repository-established implementation fact within its exact recorded platform/scope |
| Reusable foundation | existing primitive that may support a target but does not complete it |
| Requires-backend | target needs new/changed daemon/client/adapter/data behavior |
| Requires-environment | target needs unexecuted qualified Windows-native or external evidence |
| Deferred | explicitly outside the 2.0 success path |
| Forbidden | violates product or A1–A8 safety boundaries |

## Current implementation sources

- `clients/pc/web/`: delivered daemon-served Linux-era Control Plane, not the
  OPC target IA.
- `personal/apps/kernel-server/`: current daemon authority and HTTP surfaces.
- `personal/crates/cognitive-store/`: current authority/derived storage.
- `personal/crates/cognitive-runtime/`: current Task, scheduler, Agent,
  Provider, Effect, and lifecycle primitives.
- `personal/packages/pi-cognitiveos/`: current Pi client/Shell integration.
- `personal/packages/dsh-akp-adapter/`: bounded post-1.0 DSH adapter facts.
- [Linux 1.0 scope](../../../../personal/docs/product/linux-1.0-scope.md),
  [Provider Control Plane](../../../../personal/docs/product/provider-control-plane.md),
  and [Resource Manager](../../../../personal/docs/product/resource-manager-design.md):
  preserved factual product foundations.
- [Frozen 2026-08-27 audit](../legacy-control-plane-20260827/README.md):
  superseded target vocabulary, retained only as history.

Composition of these primitives does not make a 2.0 target current.

## Target matrix

| Requirement family | Current/reusable truth | Required successor | Design treatment |
|---|---|---|---|
| Windows host/tray/background | ordinary Windows CI/install fragments; no qualified OPC host | packaged local host, service/tray, safe background/close behavior | Requires-backend + Requires-environment |
| Project/Charter/Goal/Plan/Attempt | Task/preview/Effect/evidence primitives | long-lived Project aggregate, goal/output contracts, versioned plan | Requires-backend |
| Role Runtime Template/Member Runtime | Agent/adapter identities are not Member authority | reusable Template, Project-specific Member, Task/Attempt process launch | Requires-backend |
| Conversation and canvas | private conversation decision and current UI fragments | Personal-owned global/group/work archives plus typed source-linked canvas | Requires-backend |
| Personal Assistant/Pi | Pi Shell/client primitives | global Assistant composition; hidden candidate-only Pi | Requires-backend; Pi qualification does not transfer |
| hidden managed DSH | post-1.0 Path B and adapter | audited Windows child, broker, sandbox, version/rollback diagnostics | Requires-backend + Requires-environment |
| Routine/Trigger/queue-latest | scheduler primitives | six trigger classes, no-overlap, missed/coalesced, cross-Project ordering | Requires-backend |
| contextual HITL/recovery | previews, Effects, alerts, recovery primitives | Today/Project attention, exact approval, unknown reconciliation | Requires-backend |
| Knowledge/Vault/Context | Memory/Skill/Context foundations | import/index/archive plus model-window-aware bounded Context | Requires-backend |
| Memory/feedback | admission/forget primitives | candidate lineage, correction, promotion, feedback-to-version proposal | Requires-backend |
| Model Connections/cost | current accounts, fixed Agent binding, usage, advisory budgets | quick/custom connection, explicit Member model, honest cost warning | Requires-backend |
| Skill/MCP capability path | Skill and Tool/MCP transport foundations | reviewed acquisition, exact version, scoped grants, compatibility/rollback | Requires-backend; external use may require environment |
| Today/Projects/Knowledge shell | current Linux-era UI | three anchors, bottom Settings, Assistant/group canvas workbench | Requires-backend |
| X/Twitter loop | bounded Tool/HTTP/Intent/Effect foundations | qualified publish/readback connector and independent oracle | Requires-backend + Requires-environment |
| formal acceptance | old denominator/task records exist | reconciled requirement-family denominator and exact evidence route | pending plan reconciliation; not-run |
| native mobile/E2E relay | no 2.0 capability | Personal 2.1 | Deferred |
| alternate Agent/Harness engines | research candidates only | future independent product/qualification decision | Deferred |

No legacy task ID, route, or scene count in existing plans is a current design
authority. Formal task ownership and acceptance must be reconciled before
implementation; this document does not invent replacement IDs.

## Forbidden implementation shortcuts

- UI, Assistant, manager, Member, engine, connector, or MCP writes authority.
- DSH/Pi/MCP receives raw secrets or direct Provider credentials.
- External mutation bypasses persisted Intent/Effect, fencing, or unknown
  reconciliation.
- Agent/manager self-report, Provider/Tool success, or process exit completes a
  Task.
- Generated canvas code, `eval`, hidden fetches, or invented data.
- DSH runs in-process/vendored, exposes native UI, or enables native MCP/base
  tools, HMR, or home patch.
- Project/Role/Member/Routine/Attempt becomes a generic Resource family.
- Installed capability implies a Project/Member grant.
- Temporal/LangGraph becomes a second scheduler; Letta/Mem0 becomes Memory
  authority.
- Browser/API connector evades fingerprint, CAPTCHA, anti-abuse, or rights
  controls.

## Future implementation handoff

After architecture and plan reconciliation, each implementation task must start
from formal acceptance, add authority/secret/recovery negatives, expose only
real capabilities, preserve Linux 1.0 and accepted contracts, use supported
exact-revision Windows/native validation, synchronize product/architecture/
handbook/status, and record pass/fail/partial/not-run honestly.

This document changes no architecture, contract, route, code, task status, or
authorization.
