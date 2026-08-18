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
| D01 adapter protocol focused tests | exact `a9555325` on `DEV-LINUX-NATIVE-01` | `pass` | `cargo test -p pi-agent-adapter --test daemon_candidate_protocol --locked`: **20/20**. The omitted digest recomputes from WorkspaceSearch parameters; missing both remains rejected; unknown fields remain rejected. |
| D02 kernel-server focused unit tests | exact `a9555325` on `DEV-LINUX-NATIVE-01` | `pass` | `cargo test -p kernel-server --bin kernel-server --locked`: **341/341**. Tail-preserving redaction, `sk-` masking, and adapter exit-class separation all pass in the kernel-server unit suite. |
| D02 Clippy | exact `a9555325` on `DEV-LINUX-NATIVE-01` | `pass` | `cargo clippy -p pi-agent-adapter -p kernel-server --all-targets --locked -- -D warnings` completed without warnings. |
| D03 formatting | exact `a9555325` on `DEV-LINUX-NATIVE-01` | `pass` | `cargo fmt --all -- --check` passed. |
| Required CI `32106757917` | Ubuntu + Windows | `fail` | Both platforms reached TypeScript tools tests, where `check-consistency` found the initial P2-T35 registration's mismatched task counts and missing D01-D03 Current snapshot statuses. No Rust test failure occurred; this report and plan/snapshot are being corrected before rerunning CI. |
| Required CI `32107175113` | Ubuntu + Windows | `pass` | Corrected exact branch head `ecc89fd1`: Ubuntu verify **pass** (3m24s), Windows verify **pass** (11m1s), and `required-ci` **pass**. |
| Required CI `32108875933` | Ubuntu + Windows | `pass` | Exact branch head `aa51b49de60cac3704b05ed38cfd22e4824eebe6`: Ubuntu verify **pass** (3m25s), Windows verify **pass** (21m48s), `resolve validation route` **pass**, and `required-ci` **pass**. |
| Required CI `32113063107` | Ubuntu + Windows | `pass` | Final PR head `2623fcd265b86888292a7a5d6db1d036a17e0546`: Ubuntu verify **pass** (3m19s), Windows verify **pass** (13m44s), `resolve validation route` **pass**, and `required-ci` **pass**. |

## Remaining

All required CI checks passed for the recorded branch head. Windows local Rust
validation remains `not-run by owner-directed Linux-only route`; the Windows
CI result above is independent supported validation. A new live Pi evaluation
requires a separately preregistered EVAL-012 campaign and must not reuse
EVAL-011 runtime state.
