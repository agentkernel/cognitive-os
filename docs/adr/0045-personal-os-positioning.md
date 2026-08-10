# ADR-0045: Personal OS Positioning

- Status: Accepted
- Date: 2026-08-10
- Decision owner: CognitiveOS Personal product owner
- Classification: product-semantic documentation decision
- Related: P8-T01, AXIOMS.md §3, CognitiveOS-Architecture.md, AIOS literature

## Context

Calling Personal an "agent runtime" understates durable authority, scheduling,
isolation, and unified agent-facing interfaces. AIOS and related agent-OS
research provide vocabulary (scheduling, context/memory/tool managers) that
maps cleanly onto Personal's six families and AKP envelopes. Owner direction
requires an explicit OS-positioning statement with clear non-claims for Linux
kernel replacement, IoT firmware, and enterprise multi-tenant control planes.

## Decision

1. Personal is positioned as an **operating system for cognitive resources**
   covering resource abstraction, scheduling semantics, isolation/protection,
   unified agent interface (AKP + typed ops), and durable state management.
2. Whitepaper and product vision adopt this wording; architecture headroom
   chapters reserve IoT/embodied and enterprise multi-tenant bridges without
   adding formal plan tasks.
3. Personal does **not** claim to replace the host OS kernel, provide a device
   firmware ABI, or ship an enterprise multi-tenant SaaS control plane in
   Linux 1.0 or Phase 8/9 task scope.

## Consequences

- Marketing and design language converge on cognitive-resource OS without
  overclaiming host-OS or IoT readiness.
- Extension domains (embodied safety, RFC-0001 multi-tenant) remain design
  headroom referenced from architecture docs.

## Non-goals and non-claims

No implementation change, no new Gate, no IoT or multi-tenant delivery
commitment.
