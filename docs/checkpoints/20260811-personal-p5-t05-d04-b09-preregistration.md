# P5-T05/D04 B09 campaign preregistration

- Task: `P5-T05`
- Slice: `P5-T05/D04`
- Lease: `lease/personal/P5-T05/b09-managed-pi`
- Branch: `personal/P5-T05-b09-managed-pi`
- Draft PR: https://github.com/agentkernel/cognitive-os/pull/183
- Status: registered under ADR-0047 MVP fixed denominator
- Date: 2026-08-11

## Campaign identity

| Field | Value |
|---|---|
| Campaign id | `B09-managed-pi-sidecar/1` |
| Target Gate | B09 only |
| Policy | ADR-0047 |
| Environment | `DEV-LINUX-NATIVE-01` exact revision + required Ubuntu/Windows CI |
| Claim scope | `non-claim` until owner disposition |

## Fixed denominator

Eleven authority-path observations listed in ADR-0047 plus the non-claim
`tools` harness (`b09-managed-pi-gate`). Live Provider/Pi statistical campaigns
are deferred and are not an MVP mutex.

## Non-claims

This preregistration does not set Gate state, qualify non-Pi adapters, claim
GMVP-LINUX/release/Profile, or claim live production process supervision.
