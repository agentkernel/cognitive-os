# P2-T34 private-candidate parameters digest — running validation report

- Task: `P2-T34`
- Branch: `personal/P2-T34-candidate-parameter-digest`
- Lease: `lease/personal/P2-T34/candidate-parameter-digest`
- Claim ceiling: `hypothesis` / non-claim
- Failure-first: EVAL-010 skip class
  `candidate_has_missing_fields_or_an_invalid_parameters_digest`

Owner 2026-08-18 authorized product changes after EVAL-009, then continuing
after EVAL-010 close. EVAL-010 remains **closed**. This task does not reopen
that freeze.

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

| Unit | Status | Note |
|---|---|---|
| EVAL-010 remains closed | **pass** | do not reuse `48298` / `/19` / `perfeval010-20260818` runtime |
| D01 Adapter JSON-fallback digest tests | **pass** | linux-002 at `936333ab`: `cargo test -p pi-agent-adapter --locked` including `rejects_non_sha256` and `recomputes_digest`; Clippy `-p pi-agent-adapter --all-targets -D warnings` **pass**. fmt at `a7c7c2a5` |
| D01 required CI | **pass** | PR [#241](https://github.com/agentkernel/cognitive-os/pull/241) run `32067650868` at `a7c7c2a5`: Ubuntu `verify` SUCCESS, Windows `verify` SUCCESS, `required-ci` SUCCESS. Plan card still allows Windows `not-run by Linux-only route`; this cell actually passed |
| D02 empty digest with parameters | `not-run` | new focused test added; linux-002 after this push |
| Local `cargo fmt --all -- --check` | **pass** | this window, Windows GNU eligible |

No Gate, release, Profile, B01, or Agent-benefit claim. Stub ≠ C1/C2 真机.
