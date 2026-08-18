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
| D02 empty digest with parameters | **pass** | linux-002 extracted zip of `a60ceed5` at `/home/wuz/agent-kernel-worktrees/p2-t34-a60ceed5` with dedicated `CARGO_TARGET_DIR`: `--test daemon_candidate_protocol` **17/17** including `recomputes_digest_when_parameters_digest_is_empty`. Full `-p pi-agent-adapter --locked` **pass** (protocol 17, launch_policy 5, p0_t06 7, lib units 7). Clippy `-p pi-agent-adapter --all-targets -D warnings` **pass**. `cargo fmt --all -- --check` **pass** |
| D03 required CI | `not-run` | Ubuntu `verify` SUCCESS on PR [#241](https://github.com/agentkernel/cognitive-os/pull/241) run `32087503082` at `a60ceed5`; Windows was IN_PROGRESS at record time; retarget to `main` after T32/T33 merge |
| Local `cargo fmt --all -- --check` | **pass** | this window, Windows GNU eligible |
| P2-T32 merge | **pass** | PR [#239](https://github.com/agentkernel/cognitive-os/pull/239) merged at `main@c7ce4e5f` |
| P2-T33 merge | **pass** | PR [#240](https://github.com/agentkernel/cognitive-os/pull/240) merged at `main@09a44455` |

No Gate, release, Profile, B01, or Agent-benefit claim. Stub ≠ C1/C2 真机.
