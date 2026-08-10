# P7-T04 governance A/B non-inferiority preregistration

- Status: owner-authorized task preregistration for `P7-T04/D05`
- Classification: experimental-local-only / fixed-native campaign contract
- Date: 2026-08-10
- Task: `P7-T04`
- Lease: `lease/personal/P7-T04/performance-governance`
- Branch / Draft PR: `personal/P7-T04-performance-governance` / #179

## Fixed campaign contract

| Field | Value |
|---|---|
| Campaign id | `P7-T04-governance-ab-001` |
| Claim level | `non_inferiority` only (governance overhead; no significant-benefit) |
| Environment | `DEV-LINUX-NATIVE-01` (`personal-linux-native-01`), exact Git revision |
| Arms | A=`native_baseline` (lightweight cache-hit path); B=`governance_only` (authorize→Context→cache→Intent path) |
| Denominator | fixed started attempts = retained attempts; incomplete retention fails closed |
| Safety | critical safety failures = 0; false completions = 0 |
| Metric | `governed_latency_ms` with p50/p95/p99 and confidence interval |
| Breach action | release-gating threshold must `block_release` |
| Non-claims | not Gate/Profile/GMVP-LINUX; B06/B07 remain observations; floating CI is not this environment |

## Independent review

Required Ubuntu/Windows CI plus Draft PR review of this preregistration and the
digest-bound campaign report are the independent review boundary for task
closure. This campaign does not create a product Gate pass.
