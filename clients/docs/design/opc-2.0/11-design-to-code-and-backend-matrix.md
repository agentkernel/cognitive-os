# 11 — Design-to-code trace and Requires-backend matrix

## Evidence classes

| Class | Meaning |
|---|---|
| Current | verified repository implementation fact within its recorded platform/scope |
| Reusable foundation | existing primitive that can support a target but does not complete it |
| Requires-backend | target needs new/changed daemon/client/adapter/data behavior |
| Requires-environment | acceptance needs a qualified Windows-native or campaign environment |
| Deferred | explicitly outside the 2.0 success path |
| Forbidden | would violate product/safety boundary |

## Current implementation sources

- `clients/pc/web/`: delivered daemon-served Control Plane, not OPC IA.
- `personal/apps/kernel-server/`: current daemon authority and HTTP surfaces.
- `personal/crates/cognitive-store/`: current authority/derived storage.
- `personal/crates/cognitive-runtime/`: current scheduler, Agent, Provider, and
  lifecycle primitives.
- `personal/packages/pi-cognitiveos/`: current Pi client/Shell integration.
- `personal/packages/dsh-akp-adapter/`: current bounded dsh adapter facts.
- Frozen audit:
  [`legacy-control-plane-20260827`](../legacy-control-plane-20260827/README.md).

The design does not prescribe code paths where the formal Phase 11 task has not
made the interface decision.

## Target matrix

| Surface/capability | Current/reusable truth | Required successor | UI treatment now |
|---|---|---|---|
| Windows host/tray/background | ordinary Windows CI and installer fragments; no qualified OPC host | `P11-T02` | Requires-backend + Requires-environment |
| Project/Charter/Goal/Plan/Attempt | Task/preview/Effect/evidence primitives | `P11-T03` | design/prototype only |
| Role/Assignment/Employee | adapter/Agent identities are not employee authority | `P11-T04` | design/prototype only |
| Personal Conversation archive/index/retrieval | ADR-0058 private decision; no OPC shape | `P11-T05` | design/prototype only; do not reinterpret envelope `0.1` |
| Pi Personal Assistant | Pi Shell/client primitives | `P11-T06` | candidate-only target |
| managed DSH Installed Agent | post-1.0 dsh Path B and adapter | `P11-T07` | advanced dossier; unqualified |
| Routine/Trigger/missed/queue-latest | scheduler primitives | `P11-T08` | design/prototype only |
| HITL Inbox/approval/recovery | preview, Effect, alert, recovery primitives | `P11-T09` | design/prototype only |
| Knowledge/Vault ingestion/sync/conflict | Memory/Skill/Context primitives | `P11-T10` | design/prototype only |
| Memory admission/privacy/forget | current Memory admission/forget foundation | `P11-T11` | target archive integration absent |
| Provider routing/budgets/usage | fixed binding, usage, advisory budgets | `P11-T12` | current facts + target gaps |
| OPC Today/Projects/Team/Knowledge/Inbox | current Linux-era UI | `P11-T13` | Canvas/spec only |
| X qualified connector | bounded Tool/HTTP/Effect foundations | `P11-T14` | unavailable; no active publish button |
| Windows OPC fixed denominator | no executed target matrix | `P11-T15` | not-run |
| Native mobile/E2E relay remote | no 2.0 capability | Personal 2.1 | Deferred |
| MCP family manager | Tool transport only | post-2.0 advanced successor | Deferred |
| Hermes/Codex/Cursor adapters | research/candidate only | future independent qualification | Deferred |

Task numbering is owned by the formal plan. This matrix follows its final
registered IDs; if the plan changes before implementation, update this
informative map in the same product-semantic delivery.

## Forbidden implementation shortcuts

- UI/client/Agent directly writes authority.
- DSH or Pi receives raw secrets or direct Provider credentials.
- DSH runs in-process or as a vendored fork.
- DSH native MCP/base tools, HMR, or home patch are enabled by default.
- Project/Role/Employee/Routine/Attempt becomes a generic Resource family.
- Agent/manager self-report or engine checkpoint marks completion.
- Temporal or LangGraph becomes a second scheduler/authority.
- Letta/Mem0 writes durable Memory without Personal admission.
- Browser connector evades fingerprint, CAPTCHA, or anti-abuse controls.

## Design-to-code handoff

Each implementation task must:

1. start from its formal acceptance and typed dependencies;
2. choose CI-WINDOWS-MSVC and a future qualified native Windows route;
3. add failure-first authority/secret/recovery negatives;
4. expose only real backend capabilities;
5. preserve current Linux 1.0 and accepted contract behavior;
6. synchronize product/architecture/handbook/status in the same task;
7. report not-run environment evidence honestly.

This document authorizes none of those implementations.
