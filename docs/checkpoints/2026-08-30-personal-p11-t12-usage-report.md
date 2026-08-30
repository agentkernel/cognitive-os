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

Honest usage unknown≠0: store + management `GET /management/usage` returns source-labelled `actual | estimated | unknown` costs; binding explanation global→Project→employee→Task as durable facts (Project/employee/Task unbound today); silent rebind rejected; secrets never in usage HTTP. Required CI retry after `clippy::redundant_guards` at `453ffc37`. `DEV-WIN-GNU-01` cargo test **not-run**.

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
| store + in-process HTTP focused tests | **pass** (6/6 + 1/1 + 1/1) | `DEV-LINUX-NATIVE-01` | `eb27cb8625cd2cde2494b451547a2984f126feee` |
| `check-consistency` / handbook / generate `--check` / docs-sync-gate | pending commit | `DEV-WIN-GNU-01` | this slice |
| SecretStore/Provider host E2E | **not-run** | unqualified | this slice |
| Settings chrome / member budget stop | **not-run** | 2.1 / T13 / Deferred | this slice |
| required CI | pending push | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | this slice |

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-08-30 | `verify (ubuntu-latest)` Clippy (deny warnings) | **fail** (`clippy::collapsible_if`) | `CI-UBUNTU-01` | `eb27cb8625cd2cde2494b451547a2984f126feee` | [job 99264539781](https://github.com/agentkernel/cognitive-os/actions/runs/33314217245/job/99264539781) run [33314217245](https://github.com/agentkernel/cognitive-os/actions/runs/33314217245), ~2m43s. Build + Test Rust **pass**. Clippy `-D warnings` failed at `personal/crates/cognitive-store/src/provider_control_plane.rs:688` nested `if let` + `status == "active"` silent-rebind guard. rustfmt/handbook/codegen **not-run** (clippy failed first). Windows still in progress at log time. |
| 2026-08-30 | clippy `collapsible_if` let-chain collapse | recorded | `DEV-WIN-GNU-01` | this commit | Collapse nested `if` in `write_binding`. Same silent-rebind reject/accept semantics. `p11_t12_honest_usage` and HTTP unknown≠0 / silent-rebind tests unchanged. `cargo clippy`/`test` **not-run** locally (`RUST-LINK-DEV-WIN-GNU-01`). |
| 2026-08-30 | `cargo fmt --all` | pass | `DEV-WIN-GNU-01` | this commit | formatting only; no link |
| 2026-08-30 | `fill-handbook-fingerprints` `dev.store-migrations` + `user.provider-control-plane` | pass | local Node | this commit | Fingerprint-only (en + zh-CN, 4 pages) after `provider_control_plane.rs` let-chain; handbook prose unchanged. |
| 2026-08-30 | `cargo test -p cognitive-store --test p11_t12_honest_usage` | **pass** 6/6 | `DEV-LINUX-NATIVE-01` | `eb27cb8625cd2cde2494b451547a2984f126feee` | Recorded; not re-run this turn. Focused store negatives + labelled read. |
| 2026-08-30 | kernel-server `http_usage_unknown_cost_never_zero` | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `eb27cb8625cd2cde2494b451547a2984f126feee` | Recorded; not re-run this turn. Unknown≠0 HTTP. |
| 2026-08-30 | kernel-server `http_silent_rebind_is_rejected` | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `eb27cb8625cd2cde2494b451547a2984f126feee` | Recorded; not re-run this turn. Silent rebind → `PROVIDER_SILENT_REBIND_REJECTED`. |
| 2026-08-30 | `check-handbook` / `generate-handbook --check` | pass | local Node | this commit | 58×2 locales; 18 generated pages byte-identical. |
| 2026-08-30 | `verify (ubuntu-latest)` Clippy (deny warnings) | **fail** (`clippy::redundant_guards`) | `CI-UBUNTU-01` | `453ffc371d498cb17be91a71cc395983db497403` | [job 99265864818](https://github.com/agentkernel/cognitive-os/actions/runs/33314703597/job/99265864818) run [33314703597](https://github.com/agentkernel/cognitive-os/actions/runs/33314703597), 2m58s. Build + Test Rust **pass**. Clippy `-D warnings` failed at `personal/apps/kernel-server/src/personal/provider_control_plane.rs:1340` `store_error` match guard `if detail == "silent rebind rejected"`. rustfmt/handbook/codegen **not-run** (clippy failed first). Windows still in progress at log time. Scanned remaining usage handlers: `write_binding` let-chain already collapsed; no other `==` match guards; no `dead_code` on labelled usage helpers. Unknown≠0 negatives unchanged. |
| 2026-08-30 | clippy `redundant_guards` pattern match | recorded | `DEV-WIN-GNU-01` | this commit | `store_error` matches `Conflict { detail: "silent rebind rejected" }`. Same 409 `PROVIDER_SILENT_REBIND_REJECTED`. Store test asserts the exact detail literal (stronger than `contains`). `cargo clippy`/`test` **not-run** locally (`RUST-LINK-DEV-WIN-GNU-01`). |
| 2026-08-30 | `cargo fmt --all -- --check` | pass | `DEV-WIN-GNU-01` | this commit | formatting only; no link |
| 2026-08-30 | `fill-handbook-fingerprints` + `generate-handbook --check` + `check-handbook` | pass | local Node | this commit | Fingerprint-only (en + zh-CN: daemon-and-http, provider-control-plane, provider-and-secrets, generated http-api). Handbook prose unchanged. 58×2 locales; 18 generated pages byte-identical. |
| 2026-08-30 | `cargo clippy -p kernel-server --all-targets -- -D warnings` | **fail** (`clippy::redundant_guards`) | `DEV-LINUX-NATIVE-01` | `453ffc371d498cb17be91a71cc395983db497403` | Same lint as Ubuntu job 99265864818: `store_error` match guard `if detail == "silent rebind rejected"`. cognitive-store clippy **pass**; `p11_t12_honest_usage` **6/6 pass** — not re-run this turn. |
| 2026-08-30 | `cargo clippy -p cognitive-store` / `p11_t12_honest_usage` | **pass** (recorded; not re-run) | `DEV-LINUX-NATIVE-01` | `453ffc371d498cb17be91a71cc395983db497403` | Owner-directed skip of re-run. Unknown≠0 negatives unchanged. |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC, Agent-benefit. Not T07 hosted DSH. Not T09 HITL rewrite. Not T13 Settings chrome. Not member-level budget hard-stop. Not inventing Project/employee/Task Provider bindings or quota numbers. `locally_estimated` is not pretended. Evaluation routing OFF.
