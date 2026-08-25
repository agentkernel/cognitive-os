---
doc_id: ai.source-of-truth
locale: en
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: docs/governance/PROJECT-IDENTITY.md
  - path: docs/governance/DEVELOPMENT-OPERATING-MODEL.md
    symbols: ["TASK-ATOMIC-DELIVERY-01", "Sources of truth"]
  - path: docs/standards/normative-source-and-versioning.md
fingerprint: "sha256:e93c9f7bb7d8a3b9532fe76f71918c5df1288643b77eb50c75547acf2f396298"
non_claims:
  - This page routes to canonical owners; it never replaces or restates their current content.
---

# Source-of-truth order

When two sources disagree, resolve in this order (from
[`PROJECT-IDENTITY.md`](../../../../docs/governance/PROJECT-IDENTITY.md) §4):

1. [`docs/governance/PROJECT-IDENTITY.md`](../../../../docs/governance/PROJECT-IDENTITY.md) — repository identity; `cognitiveos-personal` is the only active project.
2. [`docs/governance/DEVELOPMENT-OPERATING-MODEL.md`](../../../../docs/governance/DEVELOPMENT-OPERATING-MODEL.md) — workflow, evidence, lease, and closure semantics.
3. [`docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`](../../../../docs/plan/PERSONAL-DEVELOPMENT-PLAN.md) — formal tasks, acceptance, Delivery Slices, Gates.
4. [`docs/plan/PROGRESS.md`](../../../../docs/plan/PROGRESS.md) `Current snapshot` — the only current task/Slice/Gate/claim facts. Read it fresh every session; never quote from memory or from this handbook. Its `Owner-directed campaign` row, when active, routes continuation to that evaluation campaign and suspends development-task claiming (Operating Model §2.5).
5. [`docs/plan/PARALLEL-LANES.md`](../../../../docs/plan/PARALLEL-LANES.md) active lease table — current writable paths.
6. [`personal/docs/product/`](../../../docs/product/README.md) and [`personal/docs/architecture/`](../../../docs/architecture/README.md) — stable design intent (never current status).
7. Latest matching handoff under [`docs/checkpoints/`](../../../../docs/checkpoints/TEMPLATE.md) — operational continuity only.
8. [`docs/plan/plan.md`](../../../../docs/plan/plan.md) — research detail; never a status source.

For contract semantics: exact machine assets (`core/specs/registry/`, `core/specs/schemas/`,
`core/specs/transitions/`, `core/conformance/vectors/`) outrank normative companions
(`core/specs/*/README.md`, [`RFC-0001`](../../../../core/docs/architecture/RFC-0001-cognitiveos-governance-context-access.md),
[`docs/standards/`](../../../../docs/standards/normative-source-and-versioning.md)), which outrank the informative whitepaper
([`core/docs/architecture/CognitiveOS-Architecture.md`](../../../../core/docs/architecture/CognitiveOS-Architecture.md)). Immutable axioms live only in
[`docs/governance/AXIOMS.md`](../../../../docs/governance/AXIOMS.md).

This handbook sits below all of the above: it is generated/maintained documentation
about the implementation. If a handbook page disagrees with a canonical source, the
canonical source wins and the handbook page must be corrected in the same delivery.

Never read or cite `History/` (frozen archive). Owner analysis under
[`docs/research/`](../../../../docs/research/README.md) is informative only — never a
task, Gate, or status source. Never treat old prompts (`docs/prompts/`), dated
handoffs, or chat context as current facts.
