<!--
Task: P3-T06
Classification: implementation-only
Status: blocked
-->

# P3-T06 B03 campaign blocker

## Task state

`P3-T06/D01` is complete on branch `personal/P3-T06-context-correctness`.
The parent task remains blocked because the formal B03 campaign has not been
preregistered or executed.

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
- Final required Ubuntu/Windows CI run `31343238674` passed for branch
  revision `c03106b`.
- Exact-revision native Linux validation passed at
  `96f616fb3d337b6321cc818961bc48d69f94fda8` after routing GitHub access
  through the existing enabled user-level Mihomo service. The tools test suite
  passed 11/11, the tools syntax build passed, and consistency passed.
- Earlier direct acquisition attempts failed before checkout due to GitHub
  transfer/throughput errors. No uncommitted source was copied to the host;
  the successful validation used a clean disposable clone.

## Blocker

`blocked_paths`: supported B03 campaign preregistration and execution evidence.
D01 native validation is complete; local tooling and the non-claim evaluator
still cannot substitute for a formal B03 campaign.

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
disposition; retain `B03` as `not-run` until then.
