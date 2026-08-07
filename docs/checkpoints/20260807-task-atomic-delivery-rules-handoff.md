# Whole-task delivery governance refactor handoff

- Date: 2026-08-07
- Lease: `lease/personal/GOV/task-atomic-delivery-rules`
- Change class: repository-governance refactor; product and normative surfaces unchanged
- Branch: `lane/governance-task-atomic-delivery-rules` (isolated from the parallel window's
  legacy shared branch `lane/ctr-p3-t01-context-request-binding`)
- Base: `origin/main@d3d9d29` (PR #153)
- Upstream: to be created on push
- PR: to be created after push
- Worktree: isolated at `d:\cos-governance-worktree`; contains only the governance files, so the
  parallel window's 237 unmerged commits and its CRLF-only worktree changes are untouched

## Completed governance outcome

- Added `TASK-ATOMIC-DELIVERY-01`: one formal task is the default delivery boundary and uses one
  task branch, one evolving Draft PR and one task-scoped lease through full acceptance and closure.
- Made Delivery Slices internal execution checkpoints rather than separate branches, PRs, leases,
  handoffs or routine report points.
- Restricted checkpoints to immutable revisions needed by remote CI, exact-revision Linux validation
  or abnormal recovery; checkpoint, push, CI and recoverable failures no longer authorize stopping.
- Added MVP-first authorization: use the existing owner-local, single-principal, task-scoped,
  daemon-issued path before full RBAC, approval chains, generic capability administration or future
  extension frameworks, without weakening daemon authority, SecretStore, Intent/Effect, fencing,
  audit or independent verification.
- Added deterministic task closure: exact acceptance mapping, required validation, final evidence,
  ready/merge, lease closure, remote branch cleanup, local `main` fast-forward and clean
  HEAD/upstream verification are one uninterrupted final step.
- Added machine-enforced consistency guards and destructive override tests so removal of the new
  workflow rules fails CI.
- Recorded that the active P2-T04 and P2-T05 leases are legacy tasks sharing one branch; each must
  split to its own task branch/PR at the next safe continuation boundary, and no new task may join
  the shared branch.

## Validation

- `pnpm run check:consistency`: pass; output includes `task-atomic delivery` verification.
- `node --test tools/test/check.test.mjs`: pass, 5/5, including governance failure injection.
- `git diff --check`: pass; only pre-existing CRLF normalization warnings were emitted.

## Evidence boundary and non-claims

This delivery changes repository development workflow only. It changes no Personal task acceptance,
Gate threshold, product behavior, public contract, schema, transition, vector, release or Profile
claim. Existing task/Gate status remains unchanged.

## Remaining Git closure

Commit only the declared governance paths on `lane/governance-task-atomic-delivery-rules`, run the
required checks, push, open a PR against `main`, merge after CI passes, close the governance lease
and remove the worktree. The parallel window's branch and worktree remain untouched throughout.
