# CognitiveOS Personal product design

- Status: canonical stable product-design index
- Current product direction:
  [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- Current-status owner:
  [PROGRESS.md](../../../docs/plan/PROGRESS.md) `Current snapshot`
- Task/Gate owner:
  [PERSONAL-DEVELOPMENT-PLAN.md](../../../docs/plan/PERSONAL-DEVELOPMENT-PLAN.md)
- Preserved release record: [Personal Linux 1.0](linux-1.0-scope.md)

This directory owns stable Personal product intent, user concepts, release
boundaries, information architecture, and user journeys. It does not own
implementation status, leases, campaign evidence, support claims, or Gate
results.

## Current direction

Personal 2.0 is a **Windows-first, owner-local operating console for one-person
companies and individual developers**. One human Owner creates governed
Projects and long-lived digital employees in business language, supervises
work and cost, approves consequential actions, and verifies outcomes. The
primary IA is:

**Today / Projects / Team / Knowledge / Inbox**, with **Settings** at the
bottom and a global right-side **Personal Assistant**.

The direction is approved product semantics, not shipped capability. Most OPC
surfaces are **Requires-backend** and Windows remains unqualified. The delivered
Linux 1.0 product, current daemon-served `/ui/`, Provider Control Plane,
Resource Manager, Pi qualification, and dsh Path B are preserved current facts
only within their recorded boundaries.

## Status vocabulary

| Label | Meaning |
|---|---|
| **Current implementation (Now)** | A repository-established capability. Exact current status still comes from `PROGRESS.md`. |
| **Adopted Personal 2.0 target** | Owner-approved Windows OPC product intent; never an implementation or support claim. |
| **Requires-backend** | A daemon, client, adapter, host, archive, or workflow capability is absent or insufficient. The UI must not fake an action. |
| **Requires-environment** | Acceptance needs a qualified Windows-native or campaign environment that does not yet exist. `not-run` is not pass. |
| **Deferred** | Explicitly outside the 2.0 success path, such as native mobile remote control or another Agent adapter. |

English product documents are canonical where a bilingual pair exists.
`*.zh-CN.md` files are faithful mirrors and link to the English source.

## Product corpus

### Core product and scope

| Document | Responsibility |
|---|---|
| [Product design / PRD](product-design.md) | problem evidence, target user, outcome, P0 requirements, success measures, exclusions, and non-claims |
| [Personal 2.0 scope](personal-2.0-scope.md) | exact Windows-local inclusion, capability ledger, version boundary, and 2.1 deferral |
| [OPC product model](opc-product-model.md) | Project, Role Blueprint, Assignment, Digital Employee, Runtime, Conversation, authority, and terminology |
| [User journeys](user-journeys.md) | first Project, daily operation, approvals, recovery, knowledge, archive, and X scenario |
| [Long-running operations](long-running-operations.md) | Routine/Trigger, no-overlap, queue-latest, offline/missed, background choice, and receipts |

### Experience surfaces

| Document | Responsibility |
|---|---|
| [Web UI product design](web-ui-design.md) | OPC IA, app shell, page responsibilities, state matrix, single-composer rule, and Requires-backend behavior |
| [Agent integration and conversations](agent-integration-and-conversations.md) ([中文](agent-integration-and-conversations.zh-CN.md)) | Personal Assistant/Pi, preinstalled managed DSH, Installed Agents, employee conversations, and future adapters |
| [Knowledge, Memory, and Vault](knowledge-memory-vault.md) | Personal Home, app/data split, archive/index/retrieval, Vault, memory admission, correction, and forgetting |
| [Account Hub](account-hub.md) ([中文](account-hub.zh-CN.md)) | account/subscription/billing separation, binding precedence, budgets, quota, and usage |
| [Provider Control Plane](provider-control-plane.md) | current Provider authority and OPC daemon-proxy evolution |

### Resources and external references

| Document | Responsibility |
|---|---|
| [Cognitive resource model](cognitive-resource-model.md) | preserved six-family 1.0, advanced MCP target, and why OPC domain objects are not a generic Resource family |
| [Resource Manager](resource-manager-design.md) | factual common projections, family-native actions, knowledge indexing boundaries, and conflict behavior |
| [MCP resource family](mcp-resource-family.md) ([中文](mcp-resource-family.zh-CN.md)) | advanced/deferred MCP identities, admission, health, quarantine, and DSH base-tool prohibition |
| [Informative OSS matrix](oss-reference-matrix.md) | exact research HEAD, license, verdict, allowed learning, and rejected inference |

## Preserved and frozen material

- [Linux 1.0 scope](linux-1.0-scope.md) remains the finalized six-family,
  Pi-qualified 1.0 product boundary. The OPC rebaseline does not revise it.
- [Frozen 2026-08-27 agent-stewardship corpus](legacy-agent-stewardship-20260827/README.md)
  preserves the superseded cross-platform/external-Agent 2.0 target and its
  original index. It is not current semantics.
- Accepted ADR-0056/0058 remain historical decisions. ADR-0059 records their
  exact partial supersession; no accepted history is rewritten.

## Fixed safety and claim boundaries

- The Rust daemon remains the sole authority writer. The Personal Assistant,
  Pi, DSH, digital employees, adapters, UI, MCP servers, and connectors are
  clients, candidate producers, or bounded executors.
- External mutation remains persist-before-dispatch Intent/Effect work under
  fencing and reconciliation. Independent verification remains required for
  completion.
- Secrets enter only approved Secret Stores through non-logging daemon paths.
  DSH/Pi Provider traffic is daemon-proxied.
- Project, Role, Employee, Routine, Trigger, Attempt, Conversation, Vault,
  Provider account, and budget are product/domain concepts, not new generic
  Cognitive Resource families or public Core schemas.
- Product adoption does not imply Windows support, DSH qualification,
  connector reliability, a Gate, release, Profile, business outcome, market
  validation, usability validation, 24/7 operation, or multi-Agent benefit.
