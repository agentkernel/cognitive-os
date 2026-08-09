<!--
Task: P3-T06
Classification: implementation-only
Status: blocked
-->

# P3-T06 B03 campaign blocker

## Task state

`P3-T06` is blocked after completing `P3-T06/D01` on branch
`personal/P3-T06-context-correctness`.

The implementation commit is
`afffb24072c78dc2f93958bc14e164f2681aea95`. It adds a deterministic,
non-claim B03 observation evaluator. The evaluator requires these explicit
facts, each set to `true`:

- authorized Context only;
- current source versions only;
- required source present; and
- no false completion.

It rejects missing or false observations, malformed Context-view digests, and
authority-shaped fields. Its report remains `claim_scope: non-claim`; it does
not set B03, a Gate, release, Profile, or Task-completion state.

## Validation

- Local Node build: passed.
- Local Node test suite: passed (11 tests).
- Local consistency check: passed.
- `git diff --check`: passed before the implementation commit.
- Required Ubuntu CI: passed.
- Required Windows CI: passed.
- Exact-revision native Linux validation: not-run. Two disposable worktree
  acquisition attempts failed before checkout: first with an unexpected Git
  transfer EOF and then with a GitHub HTTPS connection timeout. No uncommitted
  source was copied to the host, and both disposable roots were cleaned up.

## Blocker

`blocked_paths`: supported B03 campaign preregistration and execution evidence.

`blocked_task_ids`: `P3-T06`.

`blocked_gate_ids`: `B03`.

Owner: the campaign operator and independent verifier.

The formal campaign minimum has not been registered: qualified environment and
reset, exact source/artifact pins, operator opt-in, workload denominator and
threshold, complete failure accounting, evidence collector/redaction/cleanup,
and independent verifier disposition. Without those preconditions, no local
tooling report can be a B03 Gate decision.

## Recovery action

Preregister and execute a supported B03 Context-correctness campaign. Bind the
result to the required campaign evidence and obtain independent-verifier
disposition. Claim a new, exact-path lease for that execution slice; retain
`B03` as `not-run` until then.

