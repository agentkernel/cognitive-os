---
doc_id: dev.contributing-workflow
locale: en
kind: guide
audience: [developer]
status: implemented
generated: false
sources:
  - path: docs/governance/DEVELOPMENT-OPERATING-MODEL.md
    symbols: ["TASK-ATOMIC-DELIVERY-01", "CHECKPOINT-DELIVERY-01"]
  - path: docs/standards/docs-sync-contract.md
  - path: .github/workflows/ci.yml
fingerprint: "sha256:b2c50e6490fb250fc3572e0d139fd79e2fb6726bddd89ab9feb0a6b2dfc6ad57"
non_claims:
  - The Operating Model owns the binding workflow; this page is an oriented summary for contributors.
---

# Contributing workflow

The binding rules live in the
[Operating Model](../../../docs/governance/DEVELOPMENT-OPERATING-MODEL.md); this is
the practical shape:

1. **Select** a formal task from
   [`PERSONAL-DEVELOPMENT-PLAN.md`](../../../docs/plan/PERSONAL-DEVELOPMENT-PLAN.md)
   whose dependencies are met. Read
   [`PROGRESS.md`](../../../docs/plan/PROGRESS.md) Current snapshot and the active
   lease table fresh. Exception: an active `Owner-directed campaign` row in the
   snapshot suspends task selection — sessions execute that evaluation campaign
   instead (Operating Model §2.5).
2. **Claim** it whole: one task branch, one Draft PR, one exact-path lease in
   [`PARALLEL-LANES.md`](../../../docs/plan/PARALLEL-LANES.md); register Delivery
   Slices (`<task>/DNN`) in the plan. Slices are execution checkpoints, not
   separate branches/PRs.
3. **Deliver vertically**: smallest real slice first (real caller or durable
   authority outcome over helper stacks); focused failure-first tests; validation
   routed per [development environments](./development-environments.md).
4. **Checkpoint** by committing/pushing coherent secret-free progress to the same
   Draft PR — checkpoints are background persistence, not report boundaries or
   merge triggers. Every commit and push must first pass the docs-sync gate
   (`node tools/src/docs-sync-gate.mjs --staged|--push`; enable the repo hooks
   once with `pnpm run hooks:install`).
5. **Sync docs in the same change set, before commit/push/merge**: declare the
   change class and follow
   [`docs-sync-contract.md`](../../../docs/standards/docs-sync-contract.md) §2
   for legacy docs plus [`handbook/_meta/sync-policy.md`](../../_meta/sync-policy.md)
   for this handbook (source-map lookup, regenerate generated pages, refresh
   fingerprints). A genuinely documentation-neutral change passes the gate only
   with `DOCS_IMPACT_NONE="<concrete reason>"`, recorded in the commit/PR.
6. **Close deterministically**: map every acceptance item to implementation +
   negatives + executed evidence; run required CI at the exact merge-candidate
   head; flip the PR from Draft only then; merge; close the lease; delete the
   task branch; fast-forward local `main`; leave `git status` clean.

Never: merge with failing/pending checks, force-push shared history, use
`git add -A`, mix unknown worktree changes, or record unexecuted validation as
anything but `not-run`.
