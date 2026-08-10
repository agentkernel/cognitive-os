# ADR-0041: Personal Axiom System Revision

- Status: Accepted
- Date: 2026-08-10
- Decision owner: CognitiveOS Personal product owner
- Classification: product-semantic + structural documentation decision
- Related: P8-T01, AXIOMS.md, DEVELOPMENT-OPERATING-MODEL.md §8, AGENTS.md
- Supersedes: divergent numbered invariant lists previously maintained in
  `AGENTS.md` and Operating Model §8 as independent owners

## Context

`AGENTS.md` and the Development Operating Model each carried a shortened
invariant list. Wording and cardinality drifted (six vs seven items; different
emphasis on secrets vs unknown worktree changes). Owner direction for the
Personal 2.0 design baseline required a single axiom document with research
justification and an explicit engineering-principle layer for context,
harness, and loop engineering.

## Decision

1. Create `docs/governance/AXIOMS.md` as the sole owner of immutable axioms
   A1–A8 and principles P1–P3.
2. Operating Model §8 and `AGENTS.md` defer to AXIOMS.md and keep only brief
   operational restatements.
3. Axiom change requires an owner-approved ADR that revises AXIOMS.md in the
   same delivery.
4. Research citations in AXIOMS.md are informative justification, not machine
   contracts and not Profile claims.

## Consequences

- Agents and humans have one canonical list; consistency checks and reviews
  treat AXIOMS.md as authoritative for axiom wording.
- Future harness/hook/context/learning designs must cite A1–A8 rather than
  invent parallel safety slogans.
- No specs, conformance vectors, registry entries, Gate results, release, or
  Profile claims change with this ADR.

## Non-goals and non-claims

This ADR does not implement adapters, hooks, compaction, multi-agent runtime,
or performance rewrites. It creates no Gate, release, or Profile evidence.
