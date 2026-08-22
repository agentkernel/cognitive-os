# P8-T13 LLM Provider Control Plane — running validation report

- Task: `P8-T13`
- Branch: `personal/P8-T13-provider-control-plane`
- Draft PR: [#259](https://github.com/agentkernel/cognitive-os/pull/259) **merged** at `main@0e8ccad9`
- Lease: closed `lease/personal/P8-T13/provider-control-plane`
- Product evidence revision: `6256f7232d458a14836b1d49d43b1465686fb0b9`
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, provider-quality, or Agent-benefit promotion

Per `TEST-REPORT-INCREMENTAL-01`, each finished unit is appended below.

## Units

| Unit | Environment | Revision | Result | Notes |
|---|---|---|---|---|
| Local `cargo fmt --all -- --check` | `DEV-WIN-GNU-01` | working tree | **pass** | no GNU linking |
| `generate-handbook.mjs --check` | local Windows Node | working tree | **pass** | 18 pages byte-identical |
| `check-handbook` | local Windows Node | working tree | **pass** | 54 docs × 2 locales |
| `check:consistency` | local Windows Node | working tree | **pass** | 275 requirements; P8-T13 table pipe escaped |
| `docs-sync-gate --staged` | local Windows Node | staged set | **pass** | no `DOCS_IMPACT_NONE` |
| `git diff --check --cached` | local Windows | staged set | **pass** | |
| Focused `p8_t13_provider_control_plane` | `DEV-LINUX-NATIVE-01` | pending push | **not-run** | never infer pass |
| admin-cli parse `provider_control_plane_verbs_parse_and_refuse_api_key_flag` | `DEV-LINUX-NATIVE-01` | pending push | **not-run** | |
| `p8_t13_endpoint_trust` / `p8_t13_provider_store` | `DEV-LINUX-NATIVE-01` | pending push | **not-run** | |
| Clippy `-D warnings` kernel-server+admin-cli | `DEV-LINUX-NATIVE-01` | pending push | **not-run** | Windows GNU linking forbidden |
| Required Ubuntu/Windows CI | GitHub Actions | pending Draft PR | **not-run** | |
| `p8_t13_endpoint_trust` | `DEV-LINUX-NATIVE-01` | `9427a1b3f411321dbe7626735aa062f460b566c6` | **fail** | public test injected `Authorization: Bearer`; daemon wire auth allows that header. Retained. Follow-up asserts `X-Api-Key` injection. |
| `cognitive-store` compile (`p8_t13_provider_store`) | `DEV-LINUX-NATIVE-01` | `9427a1b3f411321dbe7626735aa062f460b566c6` | **fail** | E0618 `unavailable` local shadows helper; E0596 `Connection::transaction` needs `&mut self`. Blocks kernel-server/admin-cli tests. Retained. |
| `p8_t13_endpoint_trust` | `DEV-LINUX-NATIVE-01` | `f825d17fe4ae8799065cbf23ac404c32adab9f94` | **pass** | 1/1 after X-Api-Key negative. Worktree `/home/wuz/agent-kernel-worktrees/p8-t13-f825d17f`. rustc 1.97.1. |
| `p8_t13_provider_store` | `DEV-LINUX-NATIVE-01` | `f825d17fe4ae8799065cbf23ac404c32adab9f94` | **pass** | 8/8 |
| Focused `p8_t13_provider_control_plane` | `DEV-LINUX-NATIVE-01` | `f825d17fe4ae8799065cbf23ac404c32adab9f94` | **fail** | kernel-server compile: unresolved `provider_control_plane` in `server.rs`; `store_error`/`trust_error` param shadows `fn error`; E0502 key-op document borrow. Retained. |
| Focused `p8_t13_provider_control_plane` | `DEV-LINUX-NATIVE-01` | `77cb1410324658f8df46fcba2f4e690d99a47f13` | **fail** | E0282 bound-stream callback `Ok(())` missing error type after `ProviderTransport` import cleanup. Retained. |
| Required Ubuntu/Windows CI | GitHub Actions | `19cc7945f5227ec61c967534cdbb95df6e6afaf2` | **fail** | `RUSTFLAGS=-D warnings`: unused `mut` on bound-stream callbacks; unused `SqliteAuthorityStore` import on Windows; unused unix-only `forward_private_candidate_completion` wrapper. Retained. |
| `p8_t13_endpoint_trust` | `DEV-LINUX-NATIVE-01` | `f0fee96f9ede29bf1f7d75e1d8434c7250bf85ac` | **pass** | 1/1 public + 7/7 lib endpoint_trust. Worktree `/home/wuz/agent-kernel-worktrees/p8-t13-f0fee96f`. rustc 1.97.1. |
| `p8_t13_provider_store` | `DEV-LINUX-NATIVE-01` | `f0fee96f9ede29bf1f7d75e1d8434c7250bf85ac` | **pass** | 8/8 including 90-day aggregate, 100% alert, period rollover |
| Focused `p8_t13_provider_control_plane` | `DEV-LINUX-NATIVE-01` | `f0fee96f9ede29bf1f7d75e1d8434c7250bf85ac` | **fail** | 2/3. Isolation test expected `409 Conflict`; proxy error writer emitted `409 Error`. Retained. Follow-up maps 409 to Conflict. |
| admin-cli parse / Clippy | `DEV-LINUX-NATIVE-01` | `f0fee96f9ede29bf1f7d75e1d8434c7250bf85ac` | **not-run** | script stopped after kernel-server test failure |
| `p8_t13_endpoint_trust` | `DEV-LINUX-NATIVE-01` | `deb60d12ffa87181dcc595e96b0ba34145282954` | **pass** | 1/1. Worktree `/home/wuz/agent-kernel-worktrees/p8-t13-deb60d12`. rustc 1.97.1. |
| `p8_t13_provider_store` | `DEV-LINUX-NATIVE-01` | `deb60d12ffa87181dcc595e96b0ba34145282954` | **pass** | 8/8 |
| Focused `p8_t13_provider_control_plane` | `DEV-LINUX-NATIVE-01` | `deb60d12ffa87181dcc595e96b0ba34145282954` | **pass** | 3/3 including Pi/dsh isolation |
| admin-cli parse `provider_control_plane_verbs_parse_and_refuse_api_key_flag` | `DEV-LINUX-NATIVE-01` | `deb60d12ffa87181dcc595e96b0ba34145282954` | **pass** | 1/1 |
| Clippy `-D warnings` kernel-server+admin-cli | `DEV-LINUX-NATIVE-01` | `deb60d12ffa87181dcc595e96b0ba34145282954` | **fail** | cognitive-store: too_many_arguments, type_complexity, manual_unwrap_or_default, match_like_matches_macro. Retained. |
| `p8_t13_provider_store` | `DEV-LINUX-NATIVE-01` | `10fb686952e763217e60c5d82bc1c97f31f4c469` | **pass** | 8/8 after store Clippy allows. Worktree `/home/wuz/agent-kernel-worktrees/p8-t13-10fb6869`. rustc 1.97.1. |
| Clippy `-D warnings` kernel-server+admin-cli | `DEV-LINUX-NATIVE-01` | `10fb686952e763217e60c5d82bc1c97f31f4c469` | **fail** | admin-cli lib test `clippy::panic`; kernel-server collapsible_if on key rotate; needless_borrow in `store_error`; `too_many_arguments` on `handle_resource_manager_route`. Retained. |
| Required Ubuntu CI | GitHub Actions | `10fb686952e763217e60c5d82bc1c97f31f4c469` | **fail** | run `32581900056` verify (ubuntu-latest): same four Clippy classes. Windows was still in progress at this row. Retained. |
| `p8_t13_endpoint_trust` | `DEV-LINUX-NATIVE-01` | `6256f7232d458a14836b1d49d43b1465686fb0b9` | **pass** | 1/1. Worktree `/home/wuz/agent-kernel-worktrees/p8-t13-6256f723`. rustc 1.97.1. Shared `CARGO_TARGET_DIR=/home/wuz/agent-kernel-worktrees/p8-t13-f0fee96f-target`. |
| `p8_t13_provider_store` | `DEV-LINUX-NATIVE-01` | `6256f7232d458a14836b1d49d43b1465686fb0b9` | **pass** | 8/8 |
| Focused `p8_t13_provider_control_plane` | `DEV-LINUX-NATIVE-01` | `6256f7232d458a14836b1d49d43b1465686fb0b9` | **pass** | 3/3 including Pi/dsh isolation |
| admin-cli parse `provider_control_plane_verbs_parse_and_refuse_api_key_flag` | `DEV-LINUX-NATIVE-01` | `6256f7232d458a14836b1d49d43b1465686fb0b9` | **pass** | 1/1 |
| Clippy `-D warnings` workspace `--all-targets` | `DEV-LINUX-NATIVE-01` | `6256f7232d458a14836b1d49d43b1465686fb0b9` | **pass** | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| `cargo fmt --all -- --check` | `DEV-LINUX-NATIVE-01` | `6256f7232d458a14836b1d49d43b1465686fb0b9` | **pass** | |
| Required Ubuntu/Windows CI | GitHub Actions | `6256f7232d458a14836b1d49d43b1465686fb0b9` | **pass** | run `32582360429`: resolve-validation-route, verify (ubuntu-latest), verify (windows-latest), and `required-ci` all SUCCESS |
| Independent re-read of Linux log | `DEV-LINUX-NATIVE-01` | `6256f7232d458a14836b1d49d43b1465686fb0b9` | **pass** | `/tmp/p8-t13-6256f723.log` ends `=== ALL_PASS ===`; HEAD and rustc 1.97.1 match. Not a second execution. |

## Acceptance mapping

| Formal acceptance | Evidence |
|---|---|
| D01 endpoint trust / SSRF / wrong-channel negatives | Linux `p8_t13_endpoint_trust` 1/1 + kernel-server test `control_plane_refuses_unauth_task_channel_and_untrusted_endpoints` at `6256f723` (embedded credentials, HTTP/private without grants, Anthropic-compatible, header injection, 401, task-channel 403) |
| D02 named accounts, opaque Secret Store refs, delete blocked by binding | Linux store `secret_material_never_lands_in_sqlite_and_audit_rejects_key_shaped_detail` + `active_binding_blocks_account_delete_and_discovery_failure_preserves_catalog`; kernel-server `create_without_key_preserves_manual_catalog_and_blocks_delete_with_binding` |
| D03 catalog, manual models, discovery failure preserves catalog/binding | Same kernel-server catalog test + store discovery-failure preserve |
| D04 usage/cost/audit/retention | Linux store 8/8: unknown not zero, cache unknown, duplicate/historical cost, 30-day events / 90-day aggregates, `cost_unavailable` |
| D05 Pi vs dsh isolation; no fallback | Linux kernel-server `pi_and_dsh_bindings_are_isolated_before_secret_store`  + store `pi_and_dsh_bindings_are_independent` |
| D06 observe-only 80%/100% alerts; CLI callers | Linux store `budget_alerts_dedupe_at_80_and_100_and_ignore_unavailable_cost_as_zero`; admin-cli parse refuses `--api-key` |
| D07 docs-sync, Linux, required CI | Local handbook/consistency/docs-sync **pass**; Linux Clippy+fmt **pass**; required CI `32582360429` **pass** at `6256f723` |
| Key set/rotate/remove live Secret Store | **not-run** (no approved Secret Store in this campaign; daemon implements set/rotate/remove against opaque refs) |
| Real provider / live Pi / live dsh qualification | **not-run** |
| Redirect / proxy-env / oversized body / timeout transport | Reused P1-T09 fixture coverage; not re-run as P8-T13 cells |

## Blocked paths

- none for the private-candidate bound path. Custom Anthropic-compatible
  endpoints remain refused by design. Real provider calls, live Pi/dsh
  qualification, Gate, Profile, release, and B01 remain `not-run`.

## Unique next action

Merged PR [#259](https://github.com/agentkernel/cognitive-os/pull/259) at `main@0e8ccad9`. Lease and task branch closed. Do not auto-claim P6 / P7-T05 / P7-T06 / P7-T07.
