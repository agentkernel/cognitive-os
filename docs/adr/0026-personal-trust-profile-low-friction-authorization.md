# ADR-0026: Personal Trust Profile — Low-Friction Authorization Model

- Status: **Accepted** (owner directive 2026-07-26, interactive session)
- Date: 2026-07-26
- Decision owners: CognitiveOS repository owner (interactive decision session)
- Classification: Personal product interaction-policy decision. Not a
  CognitiveOS specification requirement, registry REQ, schema, transition,
  vector, Profile claim, G0 claim, or B01-B12 claim. It changes **when a human
  is asked**, not **what the system records or enforces**.
- Related: plan.md DEC-P-20, risk R-22, §17 Deferred Backlog (企业审批),
  REQ-CAP-001/002/003 (capability bind/attenuation/lease),
  REQ-RUN-003/005/008 + REQ-RES-001 (budgets/bounds), REQ-EFF-001..006
  (Intent/Effect), REQ-AUDIT-001/002, PERS-PR-022
- Planning IDs affected: P1-T09, P2-T01, P2-T02, P2-T08, P5-T01, P5-T02,
  P7-T02 (all not-started; no done-task acceptance is modified)

## Context

Owner directive: CognitiveOS Personal must be production-usable by a single
owner, and **must not impose cumbersome permission management**. The plan
already contains scattered low-friction signals — risk R-22 (企业模块阻塞个人版,
mitigated by "lightweight policy profile"), the Deferred Backlog rule that
企业审批链 is out of Personal scope with only destructive confirmation retained,
single-owner tenant bypass, and uninstall/purge/secret-delete double
confirmation — but no single binding decision defines the interaction policy.
Without it, P2/P5 implementers could default to per-action approval prompts,
reproducing the exact failure R-22 warns about.

Two constraints must both hold:

1. **Governance layer is untouchable.** Intent/Effect records, single authority
   writer, independent verifier, audit, admission, capability model, and all
   registry REQs remain fully in force. Low friction is achieved by changing
   the *interaction layer* (when the human is interrupted), never by skipping
   records or verification.
2. **Friction must concentrate at one moment.** The task admission preview
   (P2-T01 preview digest) is the natural single approval point: the owner sees
   scope, criteria, budgets, and planned effects once, then the system runs
   within those rails without further prompts on the default path.

## Decision

### 1. Single-owner trust model

Personal runs with exactly one human principal (the owner). There are no
approval chains, no multi-party sign-off, and no role hierarchies on the
default path. 企业审批 remains in the Deferred Backlog for the Enterprise
Profile and must not leak into Personal's critical path (PERS-PR-020).

### 2. Three-tier interaction policy

Every action an Agent/Tool performs is classified into exactly one tier:

| Tier | Definition | Interaction | Record |
|---|---|---|---|
| **Tier 0** | Read-only operations, and workspace-local reversible writes inside the admitted task scope | **Silent auto-authorization.** No prompt, ever | Full Intent/Effect + audit as always |
| **Tier 1** | External mutating operations that are idempotent or reconcilable (send/create/update via catalog-bound Tools) | **Ask-once standing grant** per Tool×scope on first use, with "remember" as the default; subsequent uses are silent | Grant persisted as a capability **lease** (REQ-CAP-003) with attenuated scope (REQ-CAP-002); revocable any time via `cognitive grants` CLI |
| **Tier 2** | Irreversible / destructive / costly: data purge, secret deletion, uninstall with data removal, actions exceeding admitted budgets | **Always explicit confirmation.** Never remembered, never batched away | Confirmation itself is audited |

Classification is a property of the catalog-bound operation (Operation
Catalog metadata), not of free-text Agent claims. Unknown or unclassifiable
operations default to Tier 2.

### 3. One default approval moment per task

The task admission preview (P2-T01: raw intent, preview digest, epoch fencing)
is the **single default human authorization point**. Admission approval covers
all Tier 0 actions and all Tier 1 actions under existing grants for that
task's scope. Target metric: **default-path approvals ≤ 1 per task**
(Tier 2 excluded; first-use Tier 1 grants excluded because they amortize to
zero).

### 4. Budgets are the rails that replace prompts

Per-task budgets and bounds (REQ-RUN-003/005/008, REQ-RES-001) are hard
enforcement: token, cost, tool-call, retry, and iteration limits are set at
admission and enforced by the daemon. Exceeding a budget stops the task and
surfaces a decision — it does not silently continue and does not pre-emptively
nag. Budgets are why per-action prompts are unnecessary, not a substitute for
Tier 2 confirmation.

### 5. What this ADR does NOT change

1. Intent/Effect discipline (REQ-EFF-*), persist-before-dispatch, verifier
   acceptance, fencing/reconcile, OUTCOME_UNKNOWN handling — unchanged.
2. Audit completeness (REQ-AUDIT-001/002) — unchanged; Tier 0 silence means
   "no prompt", never "no record".
3. Capability/tenant type system — retained in full; Personal binds a
   single-owner profile onto it rather than deleting types.
4. Install ≠ permission (REQ-AGENT-INSTALL-001/002, PERS-PR-013) — retained;
   installation still grants nothing until first-use grant or admission.
5. Secret boundary (ADR-0018/0020) — unchanged; secret deletion stays Tier 2.

## Consequences

- P1-T09 (init flow): beyond API key entry and model selection, init has no
  mandatory interactive step.
- P2-T01/P2-T02: admission preview implements the single approval moment;
  trust profile applied at daemon/CLI/Pi surfaces alike (PERS-PR-014).
- P2-T08 (B04 evidence): must record the count of human confirmations on the
  default path (≤1/task) and include a Tier-2 negative test (purge without
  explicit confirm must fail).
- P5-T01/P5-T02: Tool/Agent first-use produces the one-tap
  "grant and remember" lease; capability default-deny and instance isolation
  are unchanged underneath.
- P7-T02: user-facing `cognitive backup` / `cognitive restore` commands
  (excluding secrets) so that trust in automation is recoverable — a
  production-readiness gap identified alongside this decision.
- B02/B04/B05/B12 evidence rows gain a 确认次数 (confirmation-count) field.

## Rejected alternatives

1. **Per-action approval prompts** — safest-looking but destroys the product
   (R-22); users train themselves to click "yes", eroding real security.
2. **Enterprise approval chains scaled down** — wrong principal model for a
   single owner; stays in Deferred Backlog for Enterprise Profile.
3. **No confirmation at all (full YOLO)** — violates Tier 2 irreversibility
   protection and PERS-PR-013/018 boundaries; rejected.
4. **Session-global "trust everything" toggle** — coarser than Tool×scope
   leases; makes revocation and audit attribution meaningless.

## Non-claims

Accepting this ADR does **not** mean Profile `implemented`, any Gate passed,
B01-B12 executed, RC ready, or that the trust profile is implemented. All
affected tasks remain not-started; evidence_status remains not-run
(PERS-PR-022).
