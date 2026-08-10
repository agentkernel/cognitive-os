<!--
Task: P2-T08
Slice: D04
Gates: B02, B04, B05, B12
Campaign: runtime-spine-gates/1
Status: owner affirmed under ADR-0046; MVP Gate pass recorded
-->

# P2-T08/D04 Runtime Spine execution evidence — native Linux

## Bound execution

- Campaign: `runtime-spine-gates/1`
- Policy: ADR-0046 MVP fixed denominator
- Exact native checkout revision: `be7febb490fcbdf9970a700b6b6975ae49aadffe`
- Environment: `DEV-LINUX-NATIVE-01` / `personal-linux-native-01` (`hal9000`)
- Environment attestation: Ubuntu 22.04 x86_64, kernel `6.8.0-83-generic`;
  Rust `1.97.1`; Node `v22.19.0`; pnpm `10.33.2`
- Operator: Cursor agent
- Secret and network boundary: no Provider credential, no Secret Store mutation,
  no live Pi Provider traffic; evidence is redacted
- Disposable checkout: `/tmp/cognitiveos-p2-t08-d04-be7febb` removed after
  validation (`CLEANUP_OK`)

## Fixed denominator matrix

| Observation | Result | Evidence command |
|---|---:|---|
| `six_family_projection_isolated` | pass | `cargo test -p kernel-server --test p2_t02_resource_projection resource_projection_is_private_versioned_and_management_channel_bound -- --exact` |
| `task_management_channel_isolated` | pass | `cargo test -p admin-cli --test p2_t02_cli_parity cognitive_uses_isolated_task_and_resource_daemon_channels -- --exact` |
| `default_path_confirmation_recorded` | pass | `cargo test -p cognitive-runtime --test p2_t01_task_application_service preview_digest_mismatch_is_refused_before_any_kernel_mutation -- --exact` |
| `tier2_purge_requires_explicit_confirmation` | pass | `cargo test -p cognitive-management --test m5_session_approval runtime_spine_tier2_purge_requires_explicit_confirmation -- --exact` |
| `shell_close_preserved_authority` | pass | `cargo test -p cognitive-runtime --lib runtime_spine_shell_close_preserves_authority_without_cancelling_task` |
| `daemon_close_recoverable` | pass | `cargo test -p kernel-server --bin kernel-server runtime_spine_daemon_close_recoverable_without_duplicate_dispatch` |
| `outcome_unknown_reconciled_by_original_key` | pass | `cargo test -p kernel-server --bin kernel-server runtime_spine_outcome_unknown_reconciles_original_key_and_rejects_blind_retry` |
| `no_blind_retry_without_key_change` | pass | same OUTCOME_UNKNOWN original-key command |
| `no_false_completion` | pass | `cargo test -p kernel-server --bin kernel-server runtime_spine_false_completion_self_check_rejects_passed_report_as_task_completion` |
| `adr0018_local_native_exception_absent_or_replaced` | pass | `cargo test -p pi-agent-adapter --bin pi-agent-adapter expired_local_native_provider_exception_is_rejected` |
| Non-claim harness / tooling | 26/26 | `pnpm --filter @cognitiveos/repo-tools test` |

Primary matrix result: **all required observations pass** at
`be7febb490fcbdf9970a700b6b6975ae49aadffe`.

## Supported checks

- `pnpm run check:consistency`: passed on the exact Linux checkout.
- Focused Clippy `-D warnings` for `cognitive-runtime`, `cognitive-management`,
  `kernel-server`, `admin-cli`, and `pi-agent-adapter`: passed.
- Required CI for `be7febb490fcbdf9970a700b6b6975ae49aadffe`: run
  `31407542786` — Ubuntu `success`, Windows `success`.

## Non-claim suite report

```json
{
  "schema_version": "cognitiveos.runtime-spine-gate-report/0.1",
  "campaign_id": "runtime-spine-gates/1",
  "claim_scope": "non-claim",
  "target_gates": ["B02", "B04", "B05", "B12"],
  "suite_digest": "sha256:ccdc97adbb66ebfcc1fa62e38843ee17aaf2e4a0c13561c3ef91875a954b5899",
  "trace_digest": "sha256:6695aa04cdca33c2d91215e6f941ab8b840519fb087381913cb301c9deea72cd",
  "report_digest": "sha256:8a0103284a8f51bf44ee3863a0ac026c06f1404315346591a0fc48dc2e8a989e",
  "default_path_confirmation_count": 1
}
```

The evaluator did not mutate Gate state.

## Product-owner MVP disposition

Owner session reply `affirm all` (2026-08-11) affirms B02, B04, B05, and B12
against the ADR-0046 fixed matrix, native Linux/Clippy, required CI
`31407542786`, cleanup/redaction, and the non-claim report digest
`sha256:8a0103284a8f51bf44ee3863a0ac026c06f1404315346591a0fc48dc2e8a989e`.
Under ADR-0046 the bounded MVP Gate results are **pass** for each named Gate.
This does not authorize GMVP-LINUX, B08, B09, release, or Profile claims.

## Disposition boundary

Gate state is owned by this documented product decision, not by the evaluator.
Normal PR, lease, branch, and main reconciliation remain delivery closure work.

## Non-claims

This evidence does not claim GMVP-LINUX, release, Profile conformance, B08,
B09, or Task completion beyond the P2-T08 acceptance boundary.
