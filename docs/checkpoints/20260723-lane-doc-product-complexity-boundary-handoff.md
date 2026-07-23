# Lane-DOC product-complexity boundary handoff

- Date: 2026-07-23
- Base: `origin/main@8683930`
- Branch: `lane/doc-product-complexity-boundary`
- Type: product-priority decision; no machine-contract or implementation change.

## Completed

- Added ADR-0015: Ordinary Core is the default product range for ordinary users and
  general enterprises; High-Assurance audit/signature/verifier/retention/export and
  complex approval/configuration work is deferred/tracking.
- Updated the post-v0.1 plan and PROGRESS Lane-DOC status.

## Boundary

- Basic deterministic authorization, isolation, revocation, stop, recovery,
  fail-closed behavior, and minimum readable audit remain in scope.
- D-016/D-022 facts, machine-registration gates, external-evidence requirements
  for High-Assurance claims, and Profile status were not weakened or rewritten.

## Next entry

- Continue the smallest Ordinary Core usability/security slice with a real
  implementation and behavior test.
- Reopen a High-Assurance item only with a named customer/regulatory need, risk
  model, budget, and externally verifiable evidence plan.
