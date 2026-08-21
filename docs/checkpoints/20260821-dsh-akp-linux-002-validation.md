# DeepSeek Harness AKP adapter validation (2026-08-21)

- Change class: product-semantic Personal task `P8-T09` (candidate-only dsh AKP
  adapter and daemon front door).
- Target: `B01-Desktop-Linux-002` (`linux-002`) through
  `wuz@192.168.1.2` -> ProxyJump -> `hal9001@192.168.123.160`.
- Claim ceiling: implementation evidence / tested-local / linux-002 observation
  only; no Gate, release, Profile, B01, EVAL, or Agent-benefit claim.

## Task and lease

- Task: `P8-T09` (`in-progress`)
- Slice: `P8-T09/D03` in-progress; `D01`/`D02` implemented and Linux-validated
- Branch: `personal/P8-T09-dsh-akp-adapter`
- Draft PR: https://github.com/agentkernel/cognitive-os/pull/254
- Lease: `lease/personal/P8-T09/dsh-akp-adapter`
- HEAD: `40384e4b620d2fbc5d69c768134b052af6fd3751` (docs/shim checkpoint); Cordis plugin is uncommitted until the next push.
- Guest kernel-server/cognitive binaries: exact `9e239b7512e706d56d3359e5ab30a6c3469c35f8`
  (Rust mapping fix). Harness JS: `a33e58ad`.

## Pins

- dsh git revision: `528c682e061696f5a160f363f236ecbf53cbd006`
- AKP request-envelope schema digest:
  `sha256:feeaeb0942ce2796d0155b4b9c316a87cca94eccbf7b0fd7b031a2135dd7ee7b`
- bridge protocol: `cognitiveos.dsh-akp/0.1`

## Guest identity (HARD STOP if this fails)

- libvirt domain `B01-Desktop-Linux-002`
- UUID `f7bb6a52-2a0b-4ecb-8e8f-f4c60ca472a0`
- hostname `hal9001-Standard-PC-Q35-ICH9-2009`
- Ubuntu 24.04.4 LTS; glibc 2.39; Node v22.23.2
- Using this guest is not a B01/Gate/release/Profile pass

## Validation ledger

| Check | Result | Evidence / limitation |
|---|---|---|
| Formal task/lease/branch | **pass** | `P8-T09`; lease active; Draft PR 254 |
| TypeScript source build/test | **pass** (8/8) | `DEV-WIN-GNU-01` `pnpm --filter @cognitiveos/dsh-akp-adapter` |
| Jump-host Rust `cognitive-akp` | **pass** (8/8) | `DEV-LINUX-NATIVE-01` at `9e239b75` `--locked` |
| Jump-host `p8_t09_dsh_akp_live` | **pass** (1/1) | live `POST /task/akp/dsh` WorkspaceRead admission left DRAFT |
| Jump-host `dsh_akp_tests` | **pass** (2/2) | inactive/malformed/oversized/wrong-version; restart INACTIVE |
| Jump-host fmt / Clippy `-D warnings` | **pass** | `cognitive-akp` + `cognitive-runtime` + `kernel-server` at `9e239b75` |
| linux-002 identity | **pass** | domain/UUID/hostname as above |
| SecretStore DeepSeek Flash bind | **pass** (redacted) | `cognitive init --api-key-file -`; `secret_material_written: true`; `secret_ref_redacted: true`; `selected_model: deepseek-v4-flash`; doctor `secret_ref_resolves: true` |
| Path B shim E2E Read | **pass** | `attachDshCordisPlugin` → HTTP → daemon; lifecycle `COMPLETED` |
| Path B shim E2E Search | **pass** | admitted then `COMPLETED` (harness first sample saw `ACTIVE`; later evidence `COMPLETED`) |
| Path B shim E2E Write | **pass** | disposable `p8-t09-write.txt` 24 bytes; lifecycle `COMPLETED`; dsh response ≠ Task completion |
| Fail-closed version/secret | **pass** | `DSH_VERSION_MISMATCH`; `SECRET_SHAPED_PAYLOAD` |
| Fail-closed malformed/oversized | **pass** / **pass-closed** | `MALFORMED_JSON`; oversized hits front-door `REQUEST_BODY_TOO_LARGE` (400) before AKP `FRAME_TOO_LARGE` |
| Real dsh process → Flash | **partial** | Jump-host Path A `dsh --profile headless` **pass** (`pong`, 9.66 s) at pin `528c682e` after `build:lib:host`. linux-002 real dsh→daemon Flash still **not-run**. |
| Paired Path A/B Provider timing | **partial** | Path A one-shot 9.66 s on `DEV-LINUX-NATIVE-01` (not p50/p95). Path B daemon proxy **not-run**. Not a lossless claim. |
| Secret residue cleanup | **pass** | daemon 420890 stopped; `secret-tool clear` product triple exit 0; tight `sk-[A-Za-z0-9]{16,}` / PEM scan 48 files, 1 skipped large, 0 hits; EVAL listeners untouched |
| Required CI | **partial** | Ubuntu verify pass on `a33e58ad` run; Windows pending at last check |

## Path B shim timings (observation, not a claim)

Values are nanoseconds from `@cognitiveos/dsh-akp-adapter` on one linux-002
warm-ish run. They are not p50/p95, not Path A, and not Gate evidence.

| Family | serialization | transport | total |
|---|---:|---:|---:|
| Read | 220977 | 83046261 | 83267238 |
| Search | 73360 | 59540163 | 59613523 |
| Write | 46867 | 96912401 | 96959268 |

## Secret handling

Local key file existed (length 35). Material entered linux-002 only through
`cognitive init --api-key-file -` stdin. This checkpoint records no key bytes
and no key digest. Cleanup cleared the product SecretStore attribute triple
(`application=cognitiveos-personal`, `provider=deepseek`,
`purpose=provider-api-key`) without `secret-tool search`/`lookup`.

## Unique next action

Run `scripts/dsh-real-process.mjs` on identity-confirmed linux-002 (or jump-host
dsh against the guest daemon) at a pushed exact revision for Path B Flash
through `POST /provider/v1/chat/completions`; keep fail-closed
restart/timeout coverage; wait for required CI; then D04 docs/CI/merge. Do not
auto-claim P6/P7.
