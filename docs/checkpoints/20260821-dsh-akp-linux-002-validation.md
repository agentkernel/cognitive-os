# DeepSeek Harness AKP adapter validation (2026-08-21)

- Change class: product-semantic Personal task `P8-T09` (candidate-only dsh AKP
  adapter and daemon front door).
- Target: `B01-Desktop-Linux-002` (`linux-002`) through
  `wuz@192.168.1.2` -> ProxyJump -> `hal9001@192.168.123.160`.
- Claim ceiling: implementation evidence / tested-local / linux-002 observation
  only; no Gate, release, Profile, B01, EVAL, or Agent-benefit claim.

## Task and lease

- Task: `P8-T09` (`in-progress`; D03 evidence complete)
- Slice: `P8-T09/D03` done; `P8-T09/D04` docs/CI/merge remaining
- Branch: `personal/P8-T09-dsh-akp-adapter`
- Draft PR: https://github.com/agentkernel/cognitive-os/pull/254
- Lease: `lease/personal/P8-T09/dsh-akp-adapter`
- Exact revision for combined real-dsh E2E: `5b1c22790820690035b1700711a1e1eed5d19657`
- Guest kernel-server/cognitive binaries: exact `9e239b7512e706d56d3359e5ab30a6c3469c35f8`
  (Rust mapping unchanged since that pin). Adapter/scripts at `5b1c2279`.

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
| TypeScript source build/test | **pass** (12/12) | `DEV-WIN-GNU-01` `pnpm --filter @cognitiveos/dsh-akp-adapter` |
| Jump-host Rust `cognitive-akp` | **pass** (8/8) | `DEV-LINUX-NATIVE-01` at `9e239b75` `--locked` |
| Jump-host `p8_t09_dsh_akp_live` | **pass** (1/1) | live `POST /task/akp/dsh` WorkspaceRead admission left DRAFT |
| Jump-host `dsh_akp_tests` | **pass** (2/2) | inactive/malformed/oversized/wrong-version; restart INACTIVE |
| Jump-host fmt / Clippy `-D warnings` | **pass** | `cognitive-akp` + `cognitive-runtime` + `kernel-server` at `9e239b75` |
| linux-002 identity | **pass** | domain/UUID/hostname as above |
| SecretStore DeepSeek Flash bind | **pass** (redacted) | `cognitive init --api-key-file -`; `secret_material_written: true`; `secret_ref_redacted: true`; `selected_model: deepseek-v4-flash` |
| Path B shim E2E Read/Search/Write | **pass** | at `5b1c2279`: all three `accepted`/`admitted`/`COMPLETED`; dsh response ≠ Task completion |
| Real-dsh Workspace* + Flash | **pass** | two retained `dsh --patch` samples at `5b1c2279`: Read/Search/Write `COMPLETED` and `assistant_is_pong: true` (10488 ms, 10515 ms) |
| Fail-closed live matrix | **pass** | `DSH_VERSION_MISMATCH`; `SCHEMA_DIGEST_MISMATCH`; `BRIDGE_PROTOCOL_MISMATCH`; `STALE_FENCING_EPOCH`; `SEQUENCE_NOT_MONOTONIC`; `MALFORMED_JSON`; `SECRET_SHAPED_PAYLOAD`; unknown session `INACTIVE` |
| Fail-closed oversized/timeout/authority | **pass** (unit / earlier live) | oversized front-door `REQUEST_BODY_TOO_LARGE`; adapter `TIMEOUT` / `AUTHORITY_CLAIM_FORBIDDEN` / `FORBIDDEN_PAYLOAD_FIELD` in TypeScript tests |
| Clean-runtime daemon restart | **pass** | empty root `/home/hal9001/p8t09-5b1c2279-restart`: activate then stop/start; post-restart event `INACTIVE` `candidate_only: true` |
| Dirty-runtime daemon restart | **fail** / **not-run** for AKP INACTIVE on that root | `/home/hal9001/p8t09-5b1c2279` start after Workspace* runs failed: `scheduler lease conflict: lease owner or epoch mismatch on release`. Daemon refused to serve; not an AKP session leak. Scheduler recovery is outside this adapter slice. |
| Jump-host Path A Flash | **pass** | `dsh --profile headless` `pong` in 9.66 s at pin `528c682e`. tested-local / jump-host, not linux-002 |
| Earlier Path B Flash-only | **pass** | two samples at `bd8baf96`: 9566 ms and 9463 ms `pong` (no Workspace* startupEvents yet) |
| Paired Path A/B timing | **observation** | Path A 9.66 s n=1 jump-host API. Path B 10.49 s / 10.52 s n=2 at `5b1c2279` (guest API, includes Workspace* admits). Different hosts; n is tiny; not a lossless claim |
| Secret residue cleanup | **pass** | disposable daemons stopped; `secret-tool clear` product triple (exit 1 empty stderr = no matching item); tight `sk-[A-Za-z0-9]{16,}` / PEM scan 106+13 files, 2+2 skipped large, 0 hits; jump tunnel/key/bootstrap shredded; temp pubkey absent; EVAL listeners 48181/48284/48383 untouched |
| Required CI | **pass** | Ubuntu, Windows, and `required-ci` SUCCESS on `5b1c2279` (run `32495521633`) |

## Path B shim timings at `5b1c2279` (observation, not a claim)

Nanoseconds from `@cognitiveos/dsh-akp-adapter`. Not p50/p95, not Path A, not Gate evidence.

| Family | serialization | transport | total |
|---|---:|---:|---:|
| Read | 227314 | 73168221 | 73395535 |
| Search | 89956 | 90086510 | 90176466 |
| Write | 71234 | 71027987 | 71099221 |

## Secret handling

Local key file existed (length 35). Material entered linux-002 only through
`cognitive init --api-key-file -` stdin. This checkpoint records no key bytes
and no key digest. Cleanup targeted the product SecretStore attribute triple
(`application=cognitiveos-personal`, `provider=deepseek`,
`purpose=provider-api-key`) without `secret-tool search`/`lookup`.

## Unique next action

D04: docs-sync already in this checkpoint changeset; wait for required CI on
the docs HEAD, then ready/merge/lease/branch/main. Do not auto-claim P6/P7.
No Gate / release / Profile / B01 / Agent-benefit claim.
