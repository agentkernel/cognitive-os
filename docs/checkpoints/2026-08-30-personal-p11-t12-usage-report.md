# P11-T12 Provider honest usage — running report

- Task: `P11-T12` / slice `P11-T12/D01`
- Change class: `implementation-only` (labelled usage read on v25 Provider Control Plane; no `core/specs`, no Lane-CTR, no Settings chrome, no member-level budget hard-stop)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P11-T12/usage`
- Branch: `personal/P11-T12-usage` (worktree `D:\agent-kernel-wt-P11-T05`)
- Base: `origin/main@5374509e` (T09 merge PR [#285](https://github.com/agentkernel/cognitive-os/pull/285))
- Claim ceiling: `hypothesis` (A7: local/CI is not Gate/release/Profile)
- Evaluation routing: **OFF** (`PERSONAL-PERF-EVAL-015` closed)

## Unique next action

Honest usage unknown≠0: store + management `GET /management/usage` returns source-labelled `actual | estimated | unknown` costs; binding explanation global→Project→employee→Task as durable facts (Project/employee/Task unbound today); silent rebind rejected; secrets never in usage HTTP. `DEV-WIN-GNU-01` cargo test **not-run**.

## Closed predecessor

`P11-T09` **done**: merged PR [#285](https://github.com/agentkernel/cognitive-os/pull/285) at `main@5374509e`. Lease `lease/personal/P11-T09/hitl-canvas` closed into PARALLEL-LANES §3.1. HITL/ApprovalPreview is not a T12 rewrite. `P11-T14`/`P11-T15` stay parked. Member-level budget hard-stop is 2.1 / Deferred.

## Failure-first (this slice)

| ID | Test | Surface |
|---|---|---|
| N1 | unknown cost never serializes as JSON `0` or `"0"` | store `p11_t12_unknown_cost_never_serializes_as_zero`; HTTP `http_usage_unknown_cost_never_zero_and_omits_secrets`; T03 hook `p11_t03_unknown_cost_never_zero` |
| N2 | secret never in usage HTTP / SQLite scan | store `p11_t12_secret_never_lands_in_usage_read_model`; HTTP same |
| N3 | silent rebind rejected | store `p11_t12_silent_rebind_is_rejected`; HTTP `http_silent_rebind_is_rejected` → `PROVIDER_SILENT_REBIND_REJECTED` |
| N4 | labelled read `actual \| estimated \| unknown`; do not pretend `locally_estimated` | store `p11_t12_labelled_read_maps_existing_enums_honestly` |
| N5 | binding explanation four layers; missing layers unbound, no invented zeros | store `p11_t12_binding_explanation_is_durable_and_unbound_at_missing_layers` |
| N6 | account vs quota fields separated | store `p11_t12_account_and_quota_fields_are_separated` |

## Vertical slice

Reuse `ProviderControlPlaneStore` (v25), `record_usage` / `compute_cost` / `GET /management/usage`. T03 `unknown_cost_projection()` now delegates to `honest_unknown_cost("project")` (unbound project layer; never `0`). New `honest_usage_read_model()` is the GET usage body. No second scheduler. No new authority migration. Settings chrome waits (API is the caller). `locally_estimated` is mapped only when that metering_source was actually recorded.

## Validation

| Unit | Result | Env | Revision |
|---|---|---|---|
| store + in-process HTTP focused tests | **not-run** | `DEV-WIN-GNU-01` (`RUST-LINK-DEV-WIN-GNU-01`) | this slice |
| `check-consistency` / handbook / generate `--check` / docs-sync-gate | pending commit | `DEV-WIN-GNU-01` | this slice |
| SecretStore/Provider host E2E | **not-run** | unqualified | this slice |
| Settings chrome / member budget stop | **not-run** | 2.1 / T13 / Deferred | this slice |
| required CI | pending push | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | this slice |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC, Agent-benefit. Not T07 hosted DSH. Not T09 HITL rewrite. Not T13 Settings chrome. Not member-level budget hard-stop. Not inventing Project/employee/Task Provider bindings or quota numbers. `locally_estimated` is not pretended. Evaluation routing OFF.
