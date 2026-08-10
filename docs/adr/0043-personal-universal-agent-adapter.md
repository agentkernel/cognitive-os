# ADR-0043: Personal Universal Agent Adapter

- Status: Accepted
- Date: 2026-08-10
- Decision owner: CognitiveOS Personal product owner
- Classification: product-semantic documentation decision
- Related: P8-T01, P8-T02, P8-T03, P5-T01/T02, ADR-0035/0036/0038

## Context

Linux 1.0 product-qualifies only Pi while delivering a general adapter
framework for later independent agent qualification. Owner direction expands
the product statement to a unified cognitive-resource substrate for mainstream
agents. A2A Agent Card discovery semantics and MCP tool ecosystems are useful
interop references, but Personal must keep AKP and daemon authority as the
sole adaptation and authority path.

## Decision

1. Document a Universal Agent Adapter Contract: capability declaration,
   registration, lifecycle (acquire/install/activate/rollback/uninstall),
   candidate-only I/O, and channel isolation.
2. AKP remains the only adaptation protocol into Personal authority. Align with
   A2A discovery *semantics* without introducing a public network listener by
   default.
3. Each new agent is independently qualified (B09-style campaign); Pi evidence
   does not transfer.
4. Implementation belongs to P8-T02/T03 after P8-T01 design baseline; Lane-CTR
   registers any new machine contract such as `agent-adapter-manifest`.

## Consequences

- Product and architecture indexes describe multi-agent *readiness* without
  claiming multi-agent runtime for Linux 1.0.
- Sidecar-first integration remains the default boundary.
- MCP/dynamic Tool ecosystems stay post-1.0 capability trains unless a later
  ADR expands Linux 1.0 scope.

## Non-goals and non-claims

This ADR does not ship a second agent, open a public A2A port, or alter
specs/conformance. No Gate/release/Profile claim.
