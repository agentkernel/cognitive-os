# P2-T35 running validation report

- Task: `P2-T35` — Private-candidate JSON-fallback digest default and adapter
  diagnostic separation
- Branch: `personal/P2-T35-candidate-digest-default`
- Lease: `lease/personal/P2-T35/candidate-digest-default`
- Classification: implementation-only; normative surface unchanged
- Claim ceiling: hypothesis/non-claim; no Gate, release, Profile, B01, or EVAL
  promotion.

## Running results

Results are appended immediately after each completed validation unit.

| Unit | Environment | Result | Evidence |
|---|---|---|---|
| D01 failure-first source assessment | `DEV-WIN-GNU-01` | `not-run` | Rust test execution is prohibited on the registered Windows GNU linker host. Before the default is applied, serde rejects an omitted `parameters_digest` with the EVAL-011-class `missing field parameters_digest` diagnostic; exact-revision Linux is required for executable proof. |

## Remaining

Run the focused D01/D02 Rust tests, Clippy, and fmt on an exact pushed revision
in `DEV-LINUX-NATIVE-01`, then confirm the required Ubuntu CI result. Windows
Rust validation remains `not-run by owner-directed Linux-only route`.
