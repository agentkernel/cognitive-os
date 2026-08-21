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
- HEAD at last Path B Flash samples: `bd8baf96919886c2e4ad13e6a06214a20efed2a1`
- Guest kernel-server/cognitive binaries: exact `9e239b7512e706d56d3359e5ab30a6c3469c35f8`
  (Rust mapping unchanged since that pin). Adapter/scripts at `bd8baf96` for the
  Flash samples; Workspace* real-dsh `startupEvents` land in the next push.

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
| TypeScript source build/test | **pass** (12/12) | `DEV-WIN-GNU-01` `pnpm --filter @cognitiveos/dsh-akp-adapter` after Workspace* startupEvent test |
| Jump-host Rust `cognitive-akp` | **pass** (8/8) | `DEV-LINUX-NATIVE-01` at `9e239b75` `--locked` |
| Jump-host `p8_t09_dsh_akp_live` | **pass** (1/1) | live `POST /task/akp/dsh` WorkspaceRead admission left DRAFT |
| Jump-host `dsh_akp_tests` | **pass** (2/2) | inactive/malformed/oversized/wrong-version; restart INACTIVE |
| Jump-host fmt / Clippy `-D warnings` | **pass** | `cognitive-akp` + `cognitive-runtime` + `kernel-server` at `9e239b75` |
| linux-002 identity | **pass** | domain/UUID/hostname as above |
| SecretStore DeepSeek Flash bind | **pass** (redacted) | `cognitive init --api-key-file -`; `secret_material_written: true`; `secret_ref_redacted: true`; `selected_model: deepseek-v4-flash`; doctor `secret_ref_resolves: true` |
| Path B shim E2E Read | **pass** | `attachDshCordisPlugin` → HTTP → daemon; lifecycle `COMPLETED` |
| Path B shim E2E Search | **pass** | admitted then `COMPLETED` |
| Path B shim E2E Write | **pass** | disposable write file; lifecycle `COMPLETED`; dsh response ≠ Task completion |
| Fail-closed version/secret | **pass** | `DSH_VERSION_MISMATCH`; `SECRET_SHAPED_PAYLOAD` |
| Fail-closed malformed/oversized | **pass** / **pass-closed** | `MALFORMED_JSON`; oversized hits front-door `REQUEST_BODY_TOO_LARGE` (400) before AKP `FRAME_TOO_LARGE` |
| Jump-host Path A Flash | **pass** | `dsh --profile headless` `pong` in 9.66 s at pin `528c682e` after `build:lib:host`. tested-local / jump-host, not linux-002 |
| Path B real dsh → linux-002 Flash | **pass** | two retained samples at `bd8baf96`: 9566 ms and 9463 ms, `assistant_is_pong: true`, `dsh_exit: 0`. dsh on jump host; SSE bridge on jump; tunneled loopback to guest daemon Provider proxy. n=2, not p50/p95 |
| Real-dsh Workspace* startupEvents | **not-run** | harness now admits Read/Search/Write and emits them from `dsh --patch`; needs the next pushed SHA on linux-002 |
| Paired Path A/B Provider timing | **observation** | Path A 9.66 s (n=1, jump-host API). Path B ~9.5 s (n=2, guest API). Different hosts; not a lossless claim |
| Secret residue cleanup | **pass** | daemon at `/home/hal9001/p8t09-11830baf` stopped; `secret-tool clear` product triple exit 0; tight `sk-[A-Za-z0-9]{16,}` / PEM scan 18 files, 2 skipped large, 0 hits; disposable jump pubkey removed; jump tunnel/key/bootstrap shredded; EVAL listeners 48181/48284/48383 untouched |
| Required CI | **partial** | Ubuntu verify **pass** on `bd8baf96`; Windows still in progress at last check |

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

Push this harness revision, confirm linux-002 identity, re-bind SecretStore
Flash, and run `scripts/dsh-real-process.mjs` plus `scripts/linux002-e2e.mjs`
at that exact SHA. Then required CI and D04 docs/merge. Do not auto-claim P6/P7.
