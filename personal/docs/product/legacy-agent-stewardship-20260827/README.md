# Frozen Personal 2.0 agent-stewardship product corpus

- Status: frozen superseded product-design snapshot
- Frozen from: `main@e9d56186f5173e89de4dfac8a955e04aa041d89d`
- Frozen date: 2026-08-27
- Superseded by:
  [ADR-0059](../../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
  and the current [product index](../README.md)
- Change policy: no current product semantics; preserve content for audit and
  migration only

This directory preserves the former Personal 2.0 target corpus that centered
cross-platform stewardship of several external Agent products, native
conversation projection, the Home/Agents/Work/Library/Activity/Settings IA,
and P10-T03..T18. It is not a current source of product intent, support,
implementation status, or task priority.

The snapshot does **not** invalidate any factual description of the delivered
Linux 1.0 product, P7-T05 Control Plane, Provider authority, Resource Manager,
Pi qualification, or dsh Path B implementation as they existed at the frozen
revision. Current facts still come from `PROGRESS.md`; finalized Linux 1.0
facts remain in [linux-1.0-scope.md](../linux-1.0-scope.md).

## Snapshot contents

| Frozen file | Former responsibility |
|---|---|
| [canonical-index-before-opc.md](canonical-index-before-opc.md) | former product index and status vocabulary |
| [product-design.md](product-design.md) | cross-platform multi-Agent stewardship PRD |
| [personal-2.0-scope.md](personal-2.0-scope.md) | former full-version inclusion and eight-scenario boundary |
| [user-journeys.md](user-journeys.md) | external Agent conversation and stewardship journeys |
| [web-ui-design.md](web-ui-design.md) | former six-space desktop target |
| [agent-integration-and-conversations.md](agent-integration-and-conversations.md) | vendor-native conversation projection model |
| [agent-integration-and-conversations.zh-CN.md](agent-integration-and-conversations.zh-CN.md) | former Chinese mirror |
| [account-hub.md](account-hub.md) / [中文](account-hub.zh-CN.md) | former global/Agent/conversation routing target |
| [provider-control-plane.md](provider-control-plane.md) | factual Provider foundation plus former target |
| [cognitive-resource-model.md](cognitive-resource-model.md) | six-family current and seventh-family target model |
| [resource-manager-design.md](resource-manager-design.md) | common projection and federation target |
| [mcp-resource-family.md](mcp-resource-family.md) / [中文](mcp-resource-family.zh-CN.md) | MCP family target |

Relative links inside the frozen files resolve within this snapshot where the
linked file was also frozen. Links back to current architecture, plans, and
ADRs are retained as dated provenance and must not be read as current target
ownership.

## Supersession summary

The current OPC model changes the formal 2.0 platform to Windows-local; moves
the IA to Today/Projects/Team/Knowledge/Inbox; makes Project, Role Assignment,
and Digital Employee the primary business concepts; makes Personal own
conversations and memory; keeps Pi hidden behind the Personal Assistant; and
keeps DSH visible as a preinstalled managed Installed Agent without embedding
its native UI or conversation store. MCP and multi-external-Agent work remain
advanced/future rather than 2.0 success-path promises.

No file in this directory is an implementation, support, Gate, release,
Profile, market-validation, usability, or Agent-benefit claim.
