# P7-T04 performance governance closure

- Status: task closure
- Date: 2026-08-10
- Task: `P7-T04`
- Branch: `personal/P7-T04-performance-governance`
- Draft/Ready PR: https://github.com/agentkernel/cognitive-os/pull/179
- Lease: `lease/personal/P7-T04/performance-governance`

## Acceptance mapping

| Slice | Evidence |
|---|---|
| `P7-T04/D01` | Deterministic module benchmark covers Context/cache/CAS/scheduler/Memory FTS5/Intent-Effect/report serialization; exact native Linux + required CI |
| `P7-T04/D02` | `GovernedPathStageCollector` warm/cold/omitted-stage negatives; exact native Linux `perf::tests` 5/5 + required CI |
| `P7-T04/D03` | `buildB06B07ObservationReport` stable/changed vs full replay with complete denominator and safety accounting; required CI run `31376436215` |
| `P7-T04/D04` | `evaluateModuleRegressionFloor` rejects floating-CI release gates and records breaches as hypothesis-only |
| `P7-T04/D05` | Owner-preregistered fixed-native governance A/B non-inferiority on `DEV-LINUX-NATIVE-01` at measurement revision `d4c42e998d8d87c9b539193b6658a90a9ea4e748`; environment digest `sha256:8822e490dbeee6e77157cbd6813073a406912eaf6727fa84ca0858a493afbbfb`; report digest `sha256:b90b8452e5d7b833ada423fb6d9d8e6ae5db92830c22ebd2363d435e4fc4aad9` |

## Non-claims

- Not a Gate, Profile, or GMVP-LINUX pass
- B06/B07 remain observations and do not block Linux 1.0
- Floating CI is not release hardware evidence
- No generalized Agent-benefit or significant-benefit claim
