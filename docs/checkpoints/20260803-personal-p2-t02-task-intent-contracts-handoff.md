# P2-T02 authenticated intent contract checkpoint

- Task / slice: `P2-T02/D01`, `lease/personal/P2-T02/task-intent-contracts` (`in-progress`)
- Change class: `normative-semantic` public-contract prerequisite
- Branch: `lane/ctr-p2-t02-task-intent-contracts`
- Status: source checkpoint pending generated bindings and supported validation

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

## Validation and recovery

- Local consistency: pending after source edits; Windows GNU Rust compilation
  is unsupported by `RUST-LINK-DEV-WIN-GNU-01`.
- Required generated-binding and Rust validation: run only on a disposable
  Linux Git worktree from a pushed immutable checkpoint revision.
- Recovery: generate bindings from the pushed source checkpoint, commit the
  generated artifacts and count pins, then run the focused Linux and CI suite.
