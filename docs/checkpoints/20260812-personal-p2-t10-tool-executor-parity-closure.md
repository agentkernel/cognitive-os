# P2-T10 closure — registered Tool family executor parity

- Task: `P2-T10` registered Tool family executor parity (optimization-proposal
  INV-TASK-2)
- Slices: `P2-T10/D01`–`D04`
- Branch: `personal/P2-T10-tool-executor-parity`
- PR: [#207](https://github.com/agentkernel/cognitive-os/pull/207)
- Lease: `lease/personal/P2-T10/tool-executor-parity`
- Trigger: the `PERSONAL-PERF-EVAL-002` assessment, §14 capability truth and
  §16 Priority 6, under the owner's 2026-08-12 delivery instruction to remove
  the structural blockers that make the `C1`/`C2` capability classes
  unreachable.

## 1. What was unreachable, and what changed

The campaign measured four of the six registered Tool families as
`registered_only`: registered and enabled, with no assembled executor. An Agent
could not search a workspace, write or patch a file, or fetch a URL at all.
This task assembles those four sinks. It closes **gap 3 only** of the four
wiring gaps recorded on `handbook/*/developer/execution-chain-status.md`.

| Family | Before | After |
|---|---|---|
| `workspace_read` | assembled sink | unchanged |
| `process_check` | assembled sink | unchanged |
| `workspace_search` | **no sink** | assembled (D01) |
| `workspace_write` | **no sink** | assembled (D02) |
| `workspace_patch` | **no sink** | assembled (D02) |
| `http_fetch_read_only` | **no sink** | assembled (D03) |

## 2. Acceptance mapping

The formal acceptance is: assemble real executors for the registered families
in the order `WorkspaceSearch` → `WorkspaceWrite`/`WorkspacePatch` →
`HttpFetchReadOnly`, each reusing daemon authority, descriptor digest binding,
persist-before-dispatch, idempotency, fencing and reconcile; mutation
additionally requires expected-preimage, atomic publish, partial write,
symlink/path race, duplicate dispatch and `OUTCOME_UNKNOWN` negatives; no
unregistered family may be added and no "local tool" may bypass Effect
semantics.

| Acceptance item | Where it is satisfied | Focused evidence |
|---|---|---|
| Executors in the registered order | D01, D02, D03 | `search.rs`, `mutate.rs`, `http_fetch.rs` |
| Reuses daemon authority | every sink is driven by `EffectProtocol::authorize_effect` → `dispatch_effect` → `record_outcome` with the caller's `AuthorizationGrant`, `GovernanceCurrency` and `WriterLease` | `dispatch_staged_workspace_search_effect`, `dispatch_staged_workspace_mutation_effect`, `dispatch_staged_http_fetch_effect` |
| Descriptor digest binding | staging validates the descriptor through the unchanged `validate_native_tool_request`; the catalog and every descriptor digest are unchanged | `readiness_follows_the_assembled_set_without_touching_any_descriptor_digest` |
| Persist-before-dispatch | the Effect protocol records `EXECUTING` before the sink touches the filesystem, proven over real SQLite by an in-dispatch hook that reads durable state | `durable_workspace_search_dispatch_records_executing_before_io_without_advancing_task`; `durable_workspace_write_records_executing_before_mutation_and_reconciles_once` |
| Idempotency under the original key | each sink absorbs a duplicate dispatch and returns the first receipt | `workspace_search_requires_a_staged_digest_bound_request_before_io`; `duplicate_workspace_write_dispatch_publishes_exactly_once`; `duplicate_http_fetch_dispatch_performs_exactly_one_request` |
| Fencing | a stale epoch is refused before any I/O or egress | `workspace_search_sink_rejects_stale_fencing_before_io`; `workspace_mutation_sink_rejects_stale_fencing_before_any_write`; `http_fetch_sink_rejects_stale_fencing_before_egress` |
| Reconcile | `OUTCOME_UNKNOWN` resolves through the original key without repeating the work | `unknown_native_workspace_search_reconciles_original_key_without_second_scan`; `durable_workspace_write_records_executing_before_mutation_and_reconciles_once` |
| Mutation: expected preimage | validation refuses a mutation that does not declare its preimage; the sink verifies it before building the postimage and again before the rename | `workspace_mutation_cannot_be_validated_without_an_expected_preimage`; `workspace_write_refuses_a_preimage_mismatch_without_touching_the_target` |
| Mutation: atomic publish | an in-dispatch hook observes the postimage fully staged while the target still holds the preimage | `workspace_write_publishes_atomically_and_leaves_no_staging_residue` |
| Mutation: partial write | no partial target is ever observable and no staging residue survives a published, refused or raced mutation | same test, plus the directory-entry assertions in the mismatch and race tests |
| Mutation: symlink / path race | a symlinked target and an escaping parent are refused; a concurrent writer that wins the pre-rename race is not clobbered | `workspace_mutation_refuses_a_symlinked_target_and_an_escaping_parent`; `workspace_mutation_refuses_a_target_that_changed_before_publication` |
| Mutation: duplicate dispatch | one original key publishes exactly once | `duplicate_workspace_write_dispatch_publishes_exactly_once` |
| Mutation: `OUTCOME_UNKNOWN` | reconciled from durable filesystem state, including by a fresh executor with an empty ledger | `workspace_mutation_query_outcome_reconciles_from_durable_target_state` |
| No unregistered family added | `BUILTIN_TOOL_CATALOG` is unchanged at six entries with unchanged digests | `catalog_contains_every_required_native_operation_family`; the digest-stability assertion above |
| No Effect bypass | every sink implements `EffectExecutor` and is reachable only through the Effect protocol; none has a direct-call path | module structure; `dispatch_staged_*_effect` are the only adapter entry points |

## 3. Validation record (`TEST-REPORT-INCREMENTAL-01`)

Each unit was appended to `PROGRESS.md` Layer 2 as it finished. Consolidated:

| Unit | Revision | Result |
|---|---|---|
| `cargo fmt --all -- --check` | every checkpoint | **pass** |
| `cargo test -p cognitive-kernel tool_registry` | `ed0bea9`, `be4aaa4`, `1193235`, `8979749` | **11/11 pass** |
| `cargo test -p kernel-server tool_executor` (D01) | `ed0bea98792fa85a46183d26a22959b83127cde1` | **35/35 pass** (27 → 35) |
| `cargo test -p kernel-server tool_executor` (D02) | `be4aaa471338a5c07ed1de7c28d8747e2c0569a9` | **45/45 pass** |
| `cargo test -p kernel-server tool_executor` (D03) | `11932359911e822b5ebf95fed1c7c68994f4ce88` | **52/52 pass** |
| `cargo test -p kernel-server tool_executor` (D04) | `8979749e9efa2fcb9bef8f6be47f0ea76e5d1751` | **54/54 pass** |
| `cargo test -p kernel-server` full package | `8979749…` | **pass** |
| `cargo test -p cognitive-provider-transport` | `8979749…` | **10/10 pass**, including four new real loopback-TLS negatives |
| `cargo clippy -p kernel-server --all-targets` | `be4aaa4` | one `type_complexity` warning on a test hook; fixed by a type alias |
| `cargo clippy` for `kernel-server`, `cognitive-provider-transport`, `cognitive-kernel` | `1193235`, `8979749…` | **clean** |
| `pnpm run check:consistency` | local | **pass** |
| `check-handbook` + `generate-handbook --check` + `docs-sync-gate --staged` | local, every checkpoint | **pass**, never via `DOCS_IMPACT_NONE` |
| Required Ubuntu/Windows CI | closure head `effae9a4365db9d538942ec5dc3b1535af37ccc3` | **both pass**, run `31617885666` |

All Rust evidence is exact-revision native Linux on `DEV-LINUX-NATIVE-01`
consuming pushed revisions only. The local Windows GNU host cannot link Rust
(`RUST-LINK-DEV-WIN-GNU-01`) and ran no Rust build, test or Clippy.

**One CI failure was observed and is recorded rather than hidden.** The first
`verify (windows-latest)` attempt on `effae9a` failed after 8m20s in
`p1_t05_personal_readiness::status_and_doctor_require_management_channel_and_report_blocked`
with `bootstrap secret not found`. All 151 unit tests, including every test
added by this task, passed in that same run; the failing assertion is a
pre-existing integration test that allows the daemon only 100 × 20 ms = 2 s to
publish its bootstrap secret, and the runner's daemon start took ≈2.2 s. It was
diagnosed as a timing flake and confirmed by re-running the identical revision,
which passed. Nothing in this task's change set touches daemon startup, the
bootstrap secret path or that test. The thin wait budget is a real latent
fragility on slow Windows runners but is outside this task's scope and lease,
so it is reported here rather than modified.

## 3a. Task closure

- PR [#207](https://github.com/agentkernel/cognitive-os/pull/207) merged at
  `main@3f766020c4d822556887ff8af59d41ed0cb92d75`.
- Lease `lease/personal/P2-T10/tool-executor-parity` moved to the closed table.
- Local and remote task branches deleted; local `main` fast-forwarded to the
  merge with a clean worktree.
- Formal plan, phase summary, totals and `PROGRESS.md` Layer 1/Layer 2
  reconciled; the two remaining structural blockers registered as `P2-T12` and
  `P2-T13`.

## 4. What this task deliberately did not do

1. **No production caller.** `execution_ready` now means *this binary contains
   a sink for the family*. It is not a claim that an Agent can reach one. No
   `dispatch_staged_*_effect` has a production caller, so gaps 1, 2 and 4 —
   admission inserts no scheduler row, there is no periodic tick, and the
   verifier has no production caller — are unchanged. Until those close, no
   `C1` or `C2` campaign cell can execute end to end.
2. **`HEAD` is not implemented at the HTTP sink.** The registered validator
   admits `GET` and `HEAD` and the transport supports both, but there is no
   registered parameter channel for choosing a verb and no caller needs one, so
   the MVP issues `GET` only rather than inventing an unregistered
   micro-contract.
3. **`WorkspacePatch` reconciliation after a restart is `Indeterminate`, by
   design.** A whole-file write knows its postimage digest at staging, so a
   fresh executor resolves the original key from the target alone. A patch's
   postimage depends on the preimage bytes, which a digest does not carry, so a
   restarted daemon that finds a patched target no longer matching the preimage
   reports `Indeterminate` rather than guessing. That fails closed. There is no
   dedicated negative for this specific restart path; the behaviour is stated
   here rather than claimed as tested.
4. **`ProcessRun` and Git-specific surfaces are out of scope** (INV-TASK-3) and
   still need an owner product-semantic decision.
5. **No Gate, release or Profile claim.** No existing negative, contract,
   registered error or transition was weakened, and the descriptor catalog and
   its digests are byte-identical.

## 5. Unowned concurrent change

Commit `f7d0950` ("Add the CognitiveOS Personal optimization proposal as an
informative review document") was made by another party directly onto this task
branch while P2-T10 was in flight. It is not part of this task. Its history was
not rewritten and its content was not modified. It was found staged in this
session's index and immediately unstaged so it could not be mixed into a task
commit (axiom A8); it was then discovered to have been committed independently.
Because HB009 fails closed on any tracked file that no `source-coverage.json`
rule classifies, it blocked the branch from pushing; commit `8979749`
classifies it exactly like its three sibling root reviews (`legacy-docs`,
informative, not a handbook source). This is recorded here so the change is
attributable rather than silently absorbed.
