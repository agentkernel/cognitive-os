# P8-T13 LLM Provider Control Plane — running validation report

- Task: `P8-T13`
- Branch: `personal/P8-T13-provider-control-plane`
- Draft PR: not opened yet
- Lease: `lease/personal/P8-T13/provider-control-plane`
- Product evidence revision: pending first push
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

## Blocked paths

- none currently recorded for the private-candidate bound path (wired in this
  change set; Linux/CI still `not-run`). Custom Anthropic-compatible endpoints
  remain refused by design. Real provider calls, Gate, Profile, release, and B01
  remain out of scope.

## Unique next action

Push this branch, open one Draft PR, then exact-revision Linux focused tests.
