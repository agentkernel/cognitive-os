# DeepSeek Harness AKP adapter validation (2026-08-21)

- Change class: product-semantic Personal task `P8-T09` (candidate-only dsh AKP
  adapter and daemon front door).
- Intended target: `B01-Desktop-Linux-002` (`linux-002`) through the registered
  `wuz@192.168.1.2` -> ProxyJump -> `hal9001@192.168.123.160` route.
- Claim ceiling: implementation evidence / tested-local / linux-002 observation
  only; no Gate, release, Profile, B01, EVAL, or Agent-benefit claim.

## Task and lease

- Task: `P8-T09` (`in-progress`)
- Slice: `P8-T09/D01` in-progress; `D02`–`D04` ready (D02 implementation is in the same delivery)
- Branch: `personal/P8-T09-dsh-akp-adapter`
- Lease: `lease/personal/P8-T09/dsh-akp-adapter`

## Pins

- dsh git revision: `528c682e061696f5a160f363f236ecbf53cbd006`
- AKP request-envelope schema digest:
  `sha256:feeaeb0942ce2796d0155b4b9c316a87cca94eccbf7b0fd7b031a2135dd7ee7b`
- bridge protocol: `cognitiveos.dsh-akp/0.1`

## Validation ledger

| Check | Result | Evidence / limitation |
|---|---|---|
| Formal task/lease/branch | **pass** | `P8-T09` registered; active lease claimed; branch from `origin/main` |
| TypeScript source build/test | **pass** (8/8) | local `pnpm --filter @cognitiveos/dsh-akp-adapter` build then test on `DEV-WIN-GNU-01`; not linux-002 |
| Rust focused tests / fmt / Clippy | **not-run** locally | `RUST-LINK-DEV-WIN-GNU-01`; route to linux-002 / CI |
| Daemon `POST /task/akp/dsh` protocol negatives | written | kernel-server `dsh_akp_tests`; execute on supported Linux |
| Linux-002 guest identity / E2E | **not-run** | requires a pushed exact Git revision |
| Real dsh → AKP → daemon → DeepSeek Flash | **not-run** | after immutable push |
| Paired Path A/B timing | **not-run** | no performance claim |
| Secret residue cleanup | **not-run** | no SecretStore bind yet |

## Secret handling

The local DeepSeek key file is used only as the owner-authorized SecretStore
source on linux-002. This checkpoint records neither key content nor a
redacted digest of the key. No Provider request has been made yet.

## Unique next action

Commit and push an immutable secret-free task-owned revision, open or update the
Draft PR, then identity-confirm linux-002 and run D03 on that exact SHA.
