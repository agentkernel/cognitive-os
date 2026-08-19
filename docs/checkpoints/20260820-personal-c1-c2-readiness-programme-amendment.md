# C1/C2 readiness programme amendment (2026-08-20)

- Date: 2026-08-20
- Change class: `product-semantic` (formal task P9-T08 registered; readiness
  definition of done raised) plus `corrective` (packages 6–8 were closed as
  assessment-only)
- Task / slice: `P9-T08/D01`
- Lease: `lease/personal/P9-T08/c1-c2-paired-readiness`
- Branch: `personal/P9-T08-c1-c2-paired-readiness`
- Plan: [PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md](../plan/PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md)
- Claim ceiling: `hypothesis` / non-claim
- This is not an EVAL ID, Gate, release, Profile, B01, or Agent-benefit report

Owner instruction 2026-08-20: after this programme executes, an owner must be
able to activate a **new** preregistered EVAL and immediately start **B0
qualification on `B01-Desktop-Linux-002`** for paired C1+C2, with no remaining
product/adapter/asset/fairness/guest/secret/denominator gap that would force
`not-run` of the C1/C2 arms.

The prior assessment
[20260820-personal-c1-c2-readiness-delivery-assessment.md](20260820-personal-c1-c2-readiness-delivery-assessment.md)
remains historical evidence of packages 1–5. It is **not** a completion claim
for the amended programme.

## What was missing

| Gap | Prior disposition | Amended requirement |
|---|---|---|
| P-arm | assessment only / `not-run` | Deliver secret-safe pure-Pi broker + equivalent Workspace* fixture adapter (package 6 / P9-T08/D02) |
| Frozen assets | assessment only | Freeze runner/corpus/oracle/redactor/seeds/`retry=0`/digest ledger before any sample (package 7) |
| B0 fairness | assessment only | Encode execution-plan §2.3 and prove the checker on non-B01 (package 8) |
| B01 guest | implied by “may request B0” | Checkable procedure on `B01-Desktop-Linux-002` only; new ID/root/ports/SecretStore (package 9) |
| C1/C2 cells | parent §10 still “OS arm unreachable” | Overlay: O-arm product gap closed; remaining blockers are P-arm/assets/fairness/guest (package 10) |
| Secret bind | lesson in P2-T37 report only | `--reuse-existing-secret-binding` runbook as start gate; no keyfile copy (package 11) |
| Environment checklist | leftover `[ ]` after “complete” | Exit table E1–E12 (package 12) |
| Housekeeping | noted, not done | Delete leftover remote `personal/P2-T37-c2a-public-mutation-path` (package 13) |
| Definition of done | “a B0 may be requested” | Packages 1–14 delivered; B0 is package 15 (measurement), then B1/B2 C1+C2 |

## Package table

| Order | Package | Class | Status after D01 |
|---:|---|---|---|
| 1–5 | C1/C2a–d public O-arm | historical product | **done** (unchanged evidence) |
| 6 | Pure-Pi P arm | readiness delivery | not started |
| 7 | Frozen paired assets | readiness delivery | not started |
| 8 | B0 fairness contract | readiness delivery | not started |
| 9 | B01 guest readiness | readiness delivery | not started (isolation reserved) |
| 10 | Paired C1+C2 cell definitions | readiness delivery | gates rewritten in the plan; freezeable manifests remain |
| 11 | Secret / doctor bind path | readiness delivery | not started |
| 12 | Environment checklist | readiness delivery | now an exit table; rows `not-run` until later slices |
| 13 | Housekeeping | readiness delivery | not started |
| 14 | EVAL preregistration scaffolding | readiness delivery | `PERSONAL-PERF-EVAL-012` reserved, **not active** |
| 15 | B0 on B01 | measurement | not started; forbidden until 6–14 |
| 16 | B1 C1/C2 pilot | measurement | not started |
| 17 | B2 C1/C2 confirmatory | measurement | not started |

## Isolation reserved (not bound)

- Campaign ID: `PERSONAL-PERF-EVAL-012` (reserved; evaluation routing OFF)
- Guest: `B01-Desktop-Linux-002` only; `B01-Clean-Linux-001` forbidden
- Route: `wuz@192.168.1.2` → ProxyJump `hal9001@192.168.123.160`
- Planned daemon `127.0.0.1:48300`, broker `127.0.0.1:48400`, SecretStore
  item not `/12`–`/19`
- Do not reuse closed EVAL roots/ports or P2-T37 roots
  `p2-t37-c2a-write-20260820` / `p2-t37-c2a-patch-20260820`

## Product gap

No new P2 product task is identified on current `main` after P2-T36/P2-T37.
P-arm is campaign instrumentation (P9-T08). If D02 cannot clone Workspace*
schemas without daemon authority, fail closed and register a P2 task; never
daemon-proxy as pure Pi.

## Unique next action

Execute `P9-T08/D02` (package 6) on `DEV-LINUX-NATIVE-01`. Do not activate
EVAL-012. Do not start B01 samples.

## Non-claims

This amendment does not run B0, B1, or B2; does not promote Gate, release,
Profile, B01, paired benchmark, performance, or Agent-benefit; and does not
resume closed EVAL-002 or EVAL-004 through EVAL-011.
