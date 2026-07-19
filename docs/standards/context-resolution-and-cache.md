# Context Resolution and Cache Standard

- Standard ID: `cognitiveos.standard.context-resolution-cache/0.1`
- Version: v0.1 Draft
- Machine version: `0.1.0-draft.1`
- Status: Draft Normative Standard
- Date: 2026-07-20
- Machine assets: `specs/schemas/context-request.schema.json`,
  `context-request-admission.schema.json`, `context-view.schema.json`,
  `context-view-delta.schema.json`, `activity-context.schema.json`
- Normative sources: RFC-0001 sections 10-11, 15; Core companion section 6

## 1. Scope and normative language

RFC 2119/8174 language applies. This standard fixes the deterministic
implementation contract for Context Resolution and its caches. Owning
requirements: [REQ-CTX-001] through [REQ-CTX-012]. It registers no new
requirement.

## 2. Deterministic nine-stage pipeline

Context Resolution MUST run as an ordered, deterministic pipeline: admission
of the ContextRequest → governance pre-filter (tenant/scope) → candidate
retrieval → per-object authorization re-validation → semantic
ranking/selection → budget fitting → loss declaration → deterministic
rendering → ContextView emission with provenance. Probabilistic components
(retrieval, embedding, ranking) act strictly between the governance
pre-filter and budget fitting and only reorder or shrink the candidate set;
they MUST NOT add objects that failed a governance stage.

Two orderings are load-bearing and MUST NOT be swapped:

1. Tenant/scope filtering happens before any retrieval or ranking touches
   content (vector `context-rank-before-auth.json`, [REQ-CTX-002]).
2. Per-object body authorization re-validation happens before ranker and
   renderer see the body ([REQ-CTX-006]).

## 3. Required set and budgets

A `required` context item that cannot be authorized, retrieved, or fitted
within the hard budget MUST fail the resolution closed with
`CONTEXT_INCOMPLETE` or `CONTEXT_BUDGET_EXCEEDED` (vector
`context-required-over-budget.json`, [REQ-CTX-004]). Optional items degrade
with an explicit loss declaration in the ContextView; silent omission is
forbidden ([REQ-CTX-005]). Repeated non-converging resolution MUST surface
`CONTEXT_RESOLUTION_STAGNATED` rather than looping (vector
`context-resolution-stagnation.json`, [REQ-DISC-STAGNATION-001]).

## 4. Cache keys bind governance dimensions

Every cache on the resolution path (candidate sets, rendered fragments,
embeddings, ContextViews) MUST key on at least: tenant, principal/actor-chain
digest, capability set version, revocation epoch, purpose, schema digest,
and encoding profile. A hit that ignores any of these dimensions is a
correctness defect, not an optimization (vector
`context-revocation-cache-reuse.json`, [REQ-PROFILE-CVM-001], [REQ-CAP-005]).
Revocation or membership change
advances the epoch component, which MUST invalidate by key mismatch rather
than by best-effort scanning.

## 5. Deterministic rendering and prefix stability

Rendering the same ContextView with the same renderer version MUST be
byte-stable, and item ordering MUST follow the declared deterministic order
([REQ-CTX-012], IMP-02, vector `context-render-stability.json`). Renderers
SHOULD maximize shared-prefix stability across successive views in one
Conversation so provider prompt caches survive; content-affecting changes
MUST invalidate the affected suffix, never be masked. Delta consumption
follows `context-view-delta.schema.json`: a delta never widens scope
(vector `context-delta-scope.json`).

## 6. Non-authority projection

A ContextView is evidence for reasoning, never an authority: authorization,
CAS, transitions and commits MUST re-check authoritative state and MUST NOT
trust ContextView content ([REQ-CTX-005], `00-architecture-invariants`
rule). Untrusted content inside a view MUST NOT be promoted to control
(vector `prompt-injection-isolation.json`, [REQ-CTX-008], [REQ-SEC-002]).

## 7. Compliance checks

Vectors `ctx-schema-001.json`, `ctx-view-001.json`,
`context-rank-before-auth.json`, `context-required-over-budget.json`,
`context-revocation-cache-reuse.json`, `context-render-stability.json`,
`context-resolution-stagnation.json` and `prompt-injection-isolation.json`
must execute with evidence. Cache-key completeness is reviewed against
section 4 for every cache added on this path.
