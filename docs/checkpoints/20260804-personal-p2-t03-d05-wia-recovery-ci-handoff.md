# P2-T03/D05 WIA recovery CI checkpoint

- Date: 2026-08-04
- Task / slice: `P2-T03/D05` daemon-only WIA handoff and startup recovery
- Lease: `lease/personal/P2-T03/worker-input-contract` (active)
- Branch: `lane/ctr-p2-t03-worker-input-contract`
- Code checkpoint: `a1d1cc79bd3d9a517ad0e7b5e6bbaa5e6d3b2bdf`
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

The earlier `eb11d74` revision failed CI because the bounded scheduler generic
bound omitted `WorkerAuthorizationStore`; `ecda78e` fixed that compiler error.
The intermediate `fc1562c` and `11b4e36` revisions failed Clippy on pre-existing
test assertion style, and `be2948f` resolves those lint failures. Those failed
revisions are not claimed as validation evidence.

No exact-revision native Linux worker-recovery test was run for `a1d1cc7`.
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
