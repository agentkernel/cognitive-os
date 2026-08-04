# P2-T03/D05 WIA recovery CI checkpoint

- Date: 2026-08-04
- Task / slice: `P2-T03/D05` daemon-only WIA handoff and startup recovery
- Lease: `lease/personal/P2-T03/worker-input-contract` (active)
- Branch: `lane/ctr-p2-t03-worker-input-contract`
- Code checkpoint: `64015d284af8750f3ec027f6ab6820ea31d1ed58`
- PR: [#149](https://github.com/agentkernel/cognitive-os/pull/149) (Draft)
- Change class: `implementation-only`
- Normative surface: unchanged

## Checkpointed implementation

The private D05 path now retains exact scheduler lease identity at WIA
consumption and revalidates sealed WIA evidence at both consumption and
recovery boundaries. The bounded scheduler composition has the explicit
`WorkerAuthorizationStore` dependency required to load and consume its WIA.

Focused SQLite regressions prove that cancelled scheduler work and a scheduler
task binding different from the WIA task both cannot consume a WIA or persist a
lease binding. Existing exact-match, replacement-owner/epoch, legacy-unbound,
pending-effect and successor-release fences remain in force. Startup recovery
runs before endpoint publication and only reconciles already consumed,
exact-lease-bound handoffs.

The sealed-WIA boundary additionally rejects persistent-row disagreements in
the canonical budget charge or expected Loop version before either live WIA
consumption or restart recovery can use the row. A closed legacy unbound
handoff has no exact persisted lease identity and therefore retains scheduler
work rather than releasing a matching-looking lease.

The exact bound-handoff regression now also attempts a second consumption for
the same WIA with a distinct worker attempt ID. The duplicate conflicts, while
recovery still observes only the original consumption and exact lease binding.

The bound-handoff match also has focused coverage for a WIA from a successor
contract epoch against a lease for the same task at the current epoch. The
epoch mismatch conflicts before the consumption insert, so no handoff or lease
binding is persisted.

A matching WIA also cannot be consumed while the corresponding scheduler work
is merely `runnable`: the work must be in the exact active `leased` state with
the requested owner, epoch, and no cancellation. The runnable-state rejection
leaves no consumption or scheduler lease binding.

A terminal `succeeded` scheduler row likewise cannot reauthorize a historical
WIA, even if the row retains an earlier lease epoch. The terminal-state
rejection creates neither a consumption nor a lease binding.

Advancing the daemon fencing epoch after an exact WIA and scheduler lease are
prepared rejects the stale handoff before persistence. A test-only SQLite
trigger also injects failure at the lease-binding insert after consumption is
attempted: the enclosing transaction rolls back, and a reopened authority
database contains no modern unbound consumption or lease binding while the
scheduler lease remains intact.

## Validation

Local eligible non-linking checks passed at the checkpoint:

```text
cargo fmt --check
git diff --check
```

Required CI passed for the exact immutable revision:

```text
PR #149
verify (ubuntu-latest): pass
verify (windows-latest): pass
```

One Windows workflow initially failed outside this D05 change in the existing
P2-T02 bootstrap-secret timing test; the failure was retried without source
changes and its full Ubuntu/Windows workflow passed. The current head has two
passing Ubuntu and two passing Windows workflows.

The initial `89bbfed` runnable-state test revision failed because its fixture
violated the scheduler schema by inserting a null `lease_epoch` into a
non-null column. `e9026dc` corrects the non-leased fence to `0`, and all
required Ubuntu/Windows CI checks pass for that corrected revision. The failed
fixture revision is not claimed as D05 validation evidence.

The earlier `eb11d74` revision failed CI because the bounded scheduler generic
bound omitted `WorkerAuthorizationStore`; `ecda78e` fixed that compiler error.
The intermediate `fc1562c` and `11b4e36` revisions failed Clippy on pre-existing
test assertion style, and `be2948f` resolves those lint failures. Those failed
revisions are not claimed as validation evidence.

No exact-revision native Linux worker-recovery test was run for `64015d2`.
An attempt to run the focused `d329293` test from a clean `/tmp` Git worktree
was `not-run`: the host could reach SSH, but the GitHub clone stopped at its
low-throughput timeout before checkout or Cargo execution. The cleanup path
removed the temporary worktree. This is a validation-environment network
limitation, not a test or product failure.
Prior Linux candidate-persistence and conformance evidence remains scoped to
the earlier immutable revisions documented in the D05 candidate checkpoint.

## Remaining work and non-claims

D05 remains `in-progress`. Remaining exit work includes the specified
failure-injection matrix, exact-revision Linux worker integration validation,
and a daemon-owned external-execution/reconciliation protocol. The repository
still has no production `EffectExecutor` adapter; external dispatch remains
fail-closed/reconciliation-only.

No Provider, secret, production campaign, B01 guest workflow, external
mutating Effect, progress fact, evidence, verification, Task acceptance, or
Task completion was created. B02/B04/B05/B12, release and Profile remain
`not-run` or incomplete.
