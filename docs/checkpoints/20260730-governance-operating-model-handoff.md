# Governance operating-model refactor handoff

- Date: 2026-07-30
- Task: repository governance correction for CognitiveOS Personal progress
- Classification: corrective + structural governance documentation
- Implementation commit: `075806a1efe886f5842643729c269f4a903bb2bf`
- Branch: `lane/personal-p1-t08-mvp-single-service`
- Remote visibility: pushed to the matching lane branch

## Completed

1. Added the tracked, tool-neutral [Development Operating Model](../governance/DEVELOPMENT-OPERATING-MODEL.md).
2. Decoupled task status, implementation evidence, development track, formal Gate
   status, and release/Profile claim scope.
3. Changed the formal Personal ledger so P1-T09 is truthfully `in-progress`,
   `experimental-local-only`, `tested-local`, and non-claim while B01 remains
   `not-run`; Phase 1 and total counts now reconcile.
4. Typed implementation, acceptance, and promotion dependencies and clarified
   that Gate prerequisites do not block isolated implementation.
5. Normalized B01 campaign semantics: independent attempts, retry accounting,
   explicit threshold, zero-tolerance security failures, complete reporting, and
   disposable-test-root cleanup instead of product uninstall.
6. Added staged validation and documentation-closure rules, including the
   implementation-only docs-sync category.
7. Replaced static lane ownership assumptions with active ownership leases and
   retained the old table as historical context.
8. Added a reconciled `PROGRESS.md` current snapshot and made the autopilot prompt
   dynamic rather than a stale task-specific continuation script.
9. Updated the controlling Cursor plan with the same independent status model and
   current P1-T09/B01 facts.

## Verification

- `pnpm run check:consistency` — passed: 273 requirements, 55 error codes, 63
  schemas, 85 vectors, links and traceability verified.
- `git diff --check` — passed before commit.
- `git diff --cached --check` — passed before commit.
- No code, schema, registry, transition, or conformance vector was changed.
- `apps/kernel-server/src/personal/server.rs` remains an uncommitted,
  user-owned change and was not read, staged, committed, or modified by this
  delivery.

## Explicit non-claims

- P1-T09 is not complete.
- B01 and `GMVP-LINUX` are `not-run`.
- No native first-conversation, release, or Profile claim was created.
- Profile `implemented` remains 0.
- The governance change does not weaken secret storage, daemon authority,
  deterministic transitions, Effect/recovery, fencing, or contract boundaries.

## Next entry

Claim a fresh P1-T09 Lane-RUN ownership lease with a task-correct branch and
declared paths. The next implementation slice is a deterministic binary
Provider fixture, followed by the current route's real pinned Pi Extension load.
Keep both slices experimental/local until their evidence is actually executed;
do not run or claim B01 until its pre-registration and acceptance prerequisites
are complete.
