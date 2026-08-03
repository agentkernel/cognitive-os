# P2-T02 authenticated intent contract checkpoint

- Task / slice: `P2-T02/D01`, `lease/personal/P2-T02/task-intent-contracts` (`in-progress`)
- Change class: `normative-semantic` public-contract prerequisite
- Branch: `lane/ctr-p2-t02-task-intent-contracts`
- Status: validated contract batch; PR #139 is ready for merge

## Implemented scope

The full correct P2-T02 path requires a durable interpretation and daemon-owned
authority context before `task.admit` can succeed. This checkpoint introduces
the narrow wire contracts for authenticated Task-channel `intent.record` and
`intent.interpret`:

- callers submit raw expression or an interpretation candidate only;
- the daemon, rather than the caller, owns identity, writer lease, actor chain,
  correlation, intent authority, and governance-context derivation;
- neither result asserts acceptance, admission, dispatch, verification, or Task
  completion; and
- `TASK-INTENT-API-009` rejects client-supplied governance and writer-lease
  facts before authority mutation.

## Deliberate non-claims

No Personal daemon route, authority-context resolver, durable store composition,
SDK transport mapping, authenticated watch implementation, admission success,
Gate, release, Profile, provider, secret, service-manager, B01, or remote
state change is claimed in this checkpoint.

## Validation

- Immutable final checkpoint: `9e910bf` on
  `lane/ctr-p2-t02-task-intent-contracts`.
- Local checks passed: `pnpm run check:consistency`,
  `node tools/src/gen-matrix.mjs --check`, `git diff --check`, and
  `pnpm --filter @cognitiveos/contracts-ts build/test` (39/39).
- Exact Linux Git worktree (source checkpoint `d28bc76246d3c845e8b54c821cc3d39cc431d053`):
  generated 50 schemas, then passed `generated_types` (9/9) and
  `runner_execution` (13/13). Windows GNU compilation was not run, per
  `RUST-LINK-DEV-WIN-GNU-01`.
- Required PR #139 CI passed: Ubuntu twice and Windows twice, including the
  final pushed revision.

## Next action

Merge PR #139 and claim a distinct, narrow Lane-RUN lease. The implementation
must compose durable daemon-owned governance context, authenticated principal
binding, server writer lease, the intent-to-preview-to-admit route sequence,
and authenticated process-lifetime watch without claiming cross-restart event
durability.
