# P7-T08 Public Linux 1.0 Gate (GMVP-LINUX) closure

- Task: `P7-T08`
- Slices: `D01-D04`
- Branch: `personal/P7-T08-gmvp-linux`
- Lease: `lease/personal/P7-T08/gmvp-linux`
- Draft/Ready PR: https://github.com/agentkernel/cognitive-os/pull/194
- Classification: MVP task closure
- Date: 2026-08-11

## Acceptance mapping

| Acceptance item | Evidence |
|---|---|
| B08 Memory+Skill Gate disposition | ADR-0048 matrix at `65a736c`; Linux 14/14+1/1+Clippy; CI `31479512940`; disposition checkpoint |
| Promotion composition B01+B02+B03+B04+B05+B08+B09+B12 | ADR-0049 binder; prior MVP Gate dispositions + B08 |
| UCR-01 fixed-scenario assertions | Bound as non-claim composition observations to existing authority-path/runner evidence |
| Six-resource / release / doctor / backup / SecretStore / Pi operability | Bound to P7-T01..T03 and B09 evidence in D03 checkpoint |
| Non-claim evaluators | `b08-memory-skill-gate` + `gmvp-linux-gate` Node harnesses |
| Required CI | `31479512940` (B08 harness) and `31480604511` (composition binder `b3f4b88`) |
| Docs / lease / PR closure | this checkpoint + plan/PROGRESS sync |

## Gate dispositions

| Gate | Disposition |
|---|---|
| B08 | pass (MVP, ADR-0048) |
| GMVP-LINUX | pass (MVP, ADR-0049) |

## Non-claims

No Profile conformance, Windows B01-W parity, B06/B07 benefit, or B10/B11
enablement claim.

## Remaining delivery actions

Mark PR #194 ready, merge, close lease, delete task branch, reconcile local
`main`, then continue the campaign on the next ready Personal task.
