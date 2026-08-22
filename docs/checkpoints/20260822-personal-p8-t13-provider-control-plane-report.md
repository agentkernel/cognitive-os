# P8-T13 LLM Provider Control Plane — running validation report

- Task: `P8-T13`
- Branch: `personal/P8-T13-provider-control-plane`
- Draft PR: [#259](https://github.com/agentkernel/cognitive-os/pull/259)
- Lease: `lease/personal/P8-T13/provider-control-plane`
- Product evidence revision: `9427a1b3f411321dbe7626735aa062f460b566c6` (first push; Linux compile/test fail retained)
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

## Blocked paths

- none currently recorded for the private-candidate bound path (wired in this
  change set; Linux/CI still `not-run`). Custom Anthropic-compatible endpoints
  remain refused by design. Real provider calls, Gate, Profile, release, and B01
  remain out of scope.

## Unique next action

Push the 409 Conflict reason-phrase fix, then exact-revision Linux focused tests at the new HEAD. Keep PR #259 Draft until Linux + required CI pass.
