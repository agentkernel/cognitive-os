# CognitiveOS Agent Work System — discovery workspace

Status: **candidate / owner-confirmed scope expansion / non-canonical / Personal Desktop 1.0 OSS + UX refinement complete**

This folder contains the owner-requested product-discovery package for a
Paperclip-informed CognitiveOS Agent Work System. It records research,
recommendations, alternatives, and questions for interactive review. It does
not authorize implementation and does not register a formal Personal task,
accepted PRD, ADR, contract change, Enterprise product, release claim, or Gate
result.

## Authority and evidence boundaries

- Repository facts follow `specs/` for machine shape, normative standards for
  behavior, accepted ADRs for decisions, the formal Personal plan for task
  acceptance, `PROGRESS.md` Current snapshot for current status, and actual
  code for implementation reality.
- `docs/design/01–41` is treated as a dated design baseline. Its current
  untracked state means it is not promoted here into canonical product or
  architecture authority.
- Public product documentation and selected public issue reports are research
  inputs. Marketing claims are not treated as implementation proof, and
  anecdotes are not treated as market validation.
- Important conclusions use `FACT`, `INFERENCE`, `RECOMMENDATION`,
  `PRODUCT HYPOTHESIS`, or `OPEN QUESTION`.
- Capability claims distinguish specified, implemented, HTTP-accessible,
  CLI/library-only, tested, Gate/release proven, designed, deferred, absent,
  and unknown.
- CognitiveOS axioms A1–A8, daemon-only authority, persist-before-dispatch
  Intent/Effect, fencing, budgets, SecretStore isolation, independent
  verification, evidence claim ceilings, and unknown-worktree protection
  remain unchanged.
- **Canonical conflict:** Accepted ADR-0036 and the formal Personal plan still
  reserve `1.0.0` for Linux x86_64 and place Web UI/Windows desktop post-1.0.
  This package may say **Personal Desktop 1.0 candidate**, but does not
  supersede Linux `1.0.0`, `GMVP-LINUX`, or any formal task.
- **Scope expansion:** on 2026-08-25 the owner requested the Desktop candidate
  to prioritize Provider/subscription, Agent↔Provider Binding, Knowledge,
  Memory, Skills, Tools, token usage, Context, and Conversation history for
  office workers, programmers, and researchers. P0/P1/P2 below represent
  depth inside that candidate release, not canonical release scope.

## Current documents

1. [Research and development readiness](./01-research-and-development-readiness.md)
   Dated repository, market, Paperclip, Provider, Personal, and Enterprise
   research plus the pre-development readiness package.
2. [Product direction decision brief](./02-product-direction-decision-brief.md)
   Rounds 1–5 owner decisions, preserved recommendations/tradeoffs, and final
   requirement closure.
3. [Personal product design](./03-personal-product-design.md)
   Positioning, scope, activation, IA, object UX, screens, states, metrics, and
   P0 acceptance.
4. [Personal interaction and visual specification](./04-personal-interaction-and-visual-spec.md)
   Shared brand, spacious cards, operational views, candidate tokens,
   components, accessibility, notifications, and usability scenarios.
5. [Personal architecture](./05-personal-architecture.md)
   Native-shell/Web/daemon boundaries, fixed framework spike, P0 data flows,
   security, private/public seams, and validation route.
6. [Enterprise product design](./06-enterprise-product-design.md)
   Governed work, Continuation Package, qualified completion, Provider
   same-release track, managed Knowledge, Desktop Fleet, and workflows.
7. [Enterprise interaction and visual specification](./07-enterprise-interaction-and-visual-spec.md)
   Higher-density fleet operations, queues, evidence, approvals, Knowledge,
   role views, Desktop/Web parity, and accessibility.
8. [Enterprise architecture](./08-enterprise-architecture.md)
   Central/node topology, federation, policy contract, evidence projections,
   continuation/sync, managed Knowledge index, Provider track, and threats.
9. [Shared domain and contract boundaries](./09-shared-domain-and-contract-boundaries.md)
   Shared/not-shared matrix, domain dispositions, SoR/authority, contract gaps,
   Candidate ADRs, compatibility, and repository strategy.
10. [Validation and delivery readiness](./10-validation-and-delivery-readiness.md)
    Decision trace, acceptance/negative matrices, supported environments,
    candidate waves, readiness gates, blockers, and non-claims.
11. [Repository governance and topology recommendation](./11-repository-governance-and-topology-recommendation.md)
    Evidence-based recommendation to modularize before splitting, candidate
    Core boundaries, future four-repository topology, objective triggers,
    versioning, migration runbook, governance, validation, and rollback.
12. [Open-source reuse assessment](./12-open-source-reuse-assessment.md)
    Evidence-ranked project/license/security matrix, direct-dependency versus
    adapter/import/reference decisions, no-copy list, PoC gates, SBOM,
    provenance, and upgrade governance.
13. [Control Plane baseline → Personal Desktop 1.0 delta](./13-control-plane-baseline-to-personal-desktop-1.0-delta.md)
    Complete `docs/design/01–41` disposition, candidate IA/product/visual/
    architecture deltas, D13 protection, canonical conflict, and future
    canonicalization sequence.

## Reading order

1. Read `01` for dated repository/research reality.
2. Read `02` for owner decisions, including the Desktop 1.0 scope expansion.
3. Read `13` before editing or interpreting the protected Control Plane
   baseline.
4. Read Personal `03` → `04` → `05`.
5. Read `12` before selecting any upstream dependency, adapter, importer, or
   interaction reference.
6. Read Enterprise `06` → `07` → `08`; these remain discovery only.
7. Reconcile through shared boundaries `09`.
8. Use `10` for validation/readiness and implementation stop/go criteria.
9. Use `11` for repository topology, split triggers, and migration governance.

## Interactive workflow

1. Requirements confirmation is complete.
2. Keep all outputs candidate/non-canonical until separately promoted through
   repository governance.
3. Resolve contradictions against repository reality and A1–A8; never “fix”
   them by weakening contracts or negatives.
4. Before implementation, satisfy `10` stop/go gates, resolve active
   P7-T05/D13 ownership/canonical-scope collision, accept the required
   product-semantic ADR/formal-plan rebaseline, and obtain a formal task/lease.
5. Open-source status is **assessment only**. No upstream source, asset,
   dependency, state, credential, package, container, or history was imported.

## Implementation prohibition

This package does **not** authorize product code, schema, transition,
conformance, client, service, dependency, formal task, ADR, branch, PR, release,
Profile, or Gate work. The discovery lease cannot be reused as an implementation
lease. Node-daemon-only authority, Intent-before-Effect, fencing, hard Task
budget, SecretStore isolation, independent verification, evidence limits, and
unknown-worktree protection remain binding.

In particular, it does not authorize a Tauri/Electron spike, MCP activation,
ccusage import, OpenHands/LiteLLM/RAGFlow/Mem0/OpenLLMetry adapter, migration
from another client, or changes to `docs/design/**`.

## Repository and docs-sync status

- Product discovery lease
  `lease/personal/DISCOVERY-agent-work-system/owner-requirements` is closed.
- Repository-topology assessment lease
  `lease/personal/DISCOVERY-repository-topology/governance-recommendation`
  is closed after completing this `README.md`, document `11`, and static
  validation. Both discovery leases grant no implementation authorization.
- Owner-requested refinement lease
  `lease/personal/DISCOVERY-personal-1.0/oss-and-ux-refinement` is used only
  for the exact files listed in `PARALLEL-LANES.md` and is closed after
  document `12`/`13`, cross-link, lint, and whitespace validation complete.
- Active P7-T05 facts were refreshed before writing: evaluation routing is
  off; P7-T05/D13 is active. `docs/design/**` was intentionally left untouched.
- The current `handbook/_meta/source-map.json` has no rule matching
  `docs/agent-work-system/**`. **Docs-sync impact:** documentation-only,
  non-canonical discovery; no generated handbook route exists and no handbook
  file was edited. A future commit would require a concrete
  `DOCS_IMPACT_NONE` reason unless separately authorized work extends the
  source map.
- No generated handbook page should be edited manually.
