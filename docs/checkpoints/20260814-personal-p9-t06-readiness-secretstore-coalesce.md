# P9-T06 readiness SecretStore coalesce — running validation

- Task: `P9-T06/D01`
- Lease: `lease/personal/P9-T06/readiness-secretstore-coalesce`
- Branch: `personal/P9-T06-readiness-secretstore-coalesce`
- Claim ceiling: non-claim (no Gate/release/Profile)

## Why this task exists

The primary CI flake (`p1_t05_personal_readiness` 2 s startup wait) is already
leased on Draft PR #213 with both required jobs green. This slice is the
preregistered fallback: stop `status`/`doctor` paying a duplicated SecretStore
round-trip (~1.8 s post-P2-T11) without caching secret material or introducing
a stale-ready TTL.

## Change

`evaluate_personal_readiness` binds one SecretStore per evaluation:

- secret probe and provider `secret_ref` resolve share that backend;
- `get` is skipped when the probe already proved the backend cannot answer;
- resolved material is dropped immediately and never enters a fact;
- a later request is evaluated again (no cross-request ready cache).

Existing injected overrides keep their previous precedence, so locked/dangling
and config-snapshot negatives are unchanged.

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

| Unit | Environment | Outcome | Notes |
|---|---|---|---|
| `cargo fmt --all -- --check` | `DEV-WIN-GNU-01` | **pass** | format only; no linking |
| focused `personal::readiness` | required CI / native Linux | pending | local GNU linking is `not-run` per `RUST-LINK-DEV-WIN-GNU-01` |
| `check:consistency` | `DEV-WIN-GNU-01` | **pass** | 275 requirements, leases verified |
| `check-handbook` | `DEV-WIN-GNU-01` | **pass** | 54 docs × 2 locales |
| `generate-handbook --check` | `DEV-WIN-GNU-01` | **pass** | 18 pages byte-identical |
| docs-sync gate | `DEV-WIN-GNU-01` | pending commit | mapped `daemon-http` pages updated bilingually; fingerprints refreshed |

## Non-claims

This is not a Gate, release, Profile, B01, or measured latency claim. The
counting-store negatives prove call coalescing, not wall-clock improvement on
a production Secret Service.
