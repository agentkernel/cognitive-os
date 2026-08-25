# CognitiveOS Personal 2.0 Scope

- Status: product-semantic design baseline (documentation)
- Date: 2026-08-10
- Decision carriers: [ADR-0041](../../../docs/adr/0041-personal-axiom-system-revision.md)–[ADR-0045](../../../docs/adr/0045-personal-os-positioning.md)
- Does not own: task status, Gate results, release claims, Profile conformance

## 1. Positioning

Personal is a local **operating system for cognitive resources**: one owner-local
substrate that governs Memory, Skill, Tool, Context, Task, and Runtime for
mainstream Agents. Linux `1.0.0` / `GMVP-LINUX` remains the first ship Gate;
Phase 8/9 define the post-1.0 design and engineering train without rewriting
the 1.0 claim composition.

## 2. In scope for the 2.0 design baseline (docs now; implement later)

| Area | Design owner | First implementation tasks |
|---|---|---|
| Single axiom system + three pillars | AXIOMS, ADR-0041/0042 | P8-T01 |
| Universal Agent Adapter Contract | architecture + ADR-0043 | P8-T02 |
| First non-Pi agent qualification | product + ADR-0043/0044 | P8-T03 |
| Deterministic harness hooks | architecture + ADR-0042 | P8-T04 |
| Context compaction / adaptive budget | architecture + ADR-0042 | P8-T05 |
| Cross-episode learning loop | architecture + ADR-0042 | P8-T06 |
| Async / structure / store evolution | performance architecture | P9-T01..T03 |

## 3. Explicit non-claims

- Linux 1.0 still qualifies **only Pi**; multi-agent runtime is design-mainline
  but not a 1.0 claim (ADR-0044).
- IoT/embodied and enterprise multi-tenant bridges are architecture **headroom**
  only; they are not Phase 8/9 formal tasks (ADR-0045).
- This document does not change core/specs/conformance machines, Gate denominators,
  or Profile matrices.

## 4. Relationship to Linux 1.0

Keep `GMVP-LINUX = B01+B02+B03+B04+B05+B08+B09+B12`. Phase 8 implementation
tasks start after their typed dependencies (often after P2-T08 / P5 / P7
convergence) and never inflate 1.0 evidence by reusing Pi campaigns for other
agents.
