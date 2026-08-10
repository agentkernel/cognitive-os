# ADR-0042: Personal Three-Pillar Engineering

- Status: Accepted
- Date: 2026-08-10
- Decision owner: CognitiveOS Personal product owner
- Classification: product-semantic documentation decision
- Related: P8-T01, P8-T04, P8-T05, P8-T06, AXIOMS.md P1–P3

## Context

Industry agent systems succeed largely through deterministic scaffolding
(context assembly, harness controls, and loop termination) around a small
probabilistic core. Personal already encodes pieces of this (Context Builder,
WIA, independent verifier, budget STOP) but lacked a named product vocabulary
binding those pieces to planned Phase 8 work.

## Decision

Personal adopts three engineering pillars beneath the axioms:

1. **Context engineering (P1):** authorize-before-rank, explicit loss, digest
   binding, stable-prefix/delta reuse, and planned compaction/adaptive budgets.
2. **Harness engineering (P2):** WIA, fencing, budget, independent verifier,
   Tool pre-validators, and planned graded hooks/extension primitives that
   cannot relax A1–A8.
3. **Loop engineering (P3):** ACT→VERIFY→CONTINUE→OBSERVE, layered termination,
   externally grounded verification, and cross-episode learning only via
   candidate→admission.

## Consequences

- Whitepaper and architecture docs use this vocabulary consistently.
- P8-T04/T05/T06 implement harness hooks, context compaction, and learning
  loop under these pillars without inventing a fourth authority path.
- Pillars may evolve via product-semantic ADR without rewriting A1–A8.

## Non-goals and non-claims

No implementation, Gate, release, or Profile claim is created. Pillars do not
weaken existing Context/Memory/Skill contracts.
