# P8-T10 dsh installed-agent report (2026-08-22)

- Change class: product-semantic Personal task `P8-T10` (install DeepSeek
  Harness onto cognitiveos-personal as a product agent path).
- Target: `B01-Desktop-Linux-002` (`linux-002`) through
  `wuz@192.168.1.2` -> ProxyJump -> `hal9001@192.168.123.160`.
- Claim ceiling: implementation evidence / tested-local / linux-002 /
  performance observation only; no Gate, release, Profile, B01, EVAL, or
  Agent-benefit claim.

## Task and lease

- Task: `P8-T10` (`done`)
- Slice: `P8-T10/D04`
- Branch: `personal/P8-T10-dsh-installed-agent`
- PR: [#256](https://github.com/agentkernel/cognitive-os/pull/256)
- Lease: closed `lease/personal/P8-T10/dsh-installed-agent`
- P8-T09 remains `done` (PR [#254](https://github.com/agentkernel/cognitive-os/pull/254))

## Pins

- Product evidence revision: `4fbffc24fb11ad2962060ce093f1cd62eb55cd7c`
  (clippy test fix may land as a follow-on SHA on the same branch)
- dsh git revision: `528c682e061696f5a160f363f236ecbf53cbd006`
- AKP request-envelope schema digest:
  `sha256:feeaeb0942ce2796d0155b4b9c316a87cca94eccbf7b0fd7b031a2135dd7ee7b`
- bridge protocol: `cognitiveos.dsh-akp/0.1`

## Validation ledger

| Check | Result | Evidence / limitation |
|---|---|---|
| Formal task/lease/branch | **pass** | `P8-T10`; lease closed; PR 256 |
| Jump-host admin-cli `dsh` tests | **pass** | 6/6 at `4fbffc24` on `DEV-LINUX-NATIVE-01` |
| linux-002 identity | **pass** | libvirt domain `B01-Desktop-Linux-002`, UUID `f7bb6a52-2a0b-4ecb-8e8f-f4c60ca472a0`; guest hostname `hal9001-Standard-PC-Q35-ICH9-2009`; Ubuntu 24.04.4 |
| `cognitive dsh configure` | **pass** linux-002 | pin `528c682e…`; `candidate_only: true`; `dsh.json` digest is not SQLite-durable daemon adapter state |
| Installed Path B `cognitive dsh launch --print` | **pass** linux-002 | product SHA `4fbffc24`; daemon `127.0.0.1:48513`; Flash `deepseek-v4-flash`; assistant `pong`; WorkspaceRead/Search/Write `COMPLETED` (`task://personal/p8-t10-dsh-*-432579`); elapsed 11685 ms; TTFT 11517 ms; write file 24 bytes. dsh response is not Task completion |
| Same-host Path A vs B n=5 | **pass** linux-002 | see paired table; discarded 0/5 both paths; `overhead_b_minus_a_p50_ms=172`; lossless not preset |
| Secret residue cleanup | **pass** | disposable daemon stopped (48513 gone); `secret-tool clear` product triple exit 0; bootstrap shredded; jump ed25519 key shredded; temp `authorized_keys` line removed; EVAL listeners 48181/48284/48383 untouched. Residue scan 1458 files / 11 skipped / 1 key-shaped hit in pre-existing `/tmp/eval006-pi-cognitiveos-dist.tar` (untouched; not a P8-T10 artifact) |
| Required CI | **pass** | Ubuntu clippy `-D clippy::unwrap-used` failed on test `unwrap_err` at `4fbffc24` (run `32521224272`). Follow-on `expect_err` + rustfmt at `a4b0ad09`. Required CI `32522937342` passed Ubuntu, Windows, and required-ci |

## D02 installed Path B (linux-002)

Guest install root `/home/hal9001/p8t10-a17edfad` (clean runtime; not the P8-T09 dirty root). Binaries built on the jump host at exact `4fbffc24`. dsh tree copied from pin `528c682e` without `.git` because the guest has no git; launch verifies `{dsh_root}/.cognitiveos-dsh-revision`.

Product path that passed:

1. `cognitive init --api-key-file -` (SecretStore `linux-secret-tool`; `secret_material_written: true`; `secret_ref_redacted: true`; model `deepseek-v4-flash`)
2. `cognitive daemon start --bind 127.0.0.1:48513`
3. `cognitive doctor` overall `ready`; Pi `not_configured`
4. `cognitive dsh configure --dsh-root … --adapter-root … --revision 528c682e061696f5a160f363f236ecbf53cbd006`
5. `cognitive dsh launch --print --path b`

Fixes required for guest Node v22.23.2 vs jump Node v22.19.0:

- `pnpm dsh` is not portable (pnpm 11 deps-status wants git). Helper boots `node --import tsx/esm apps/cli/src/bin.ts`.
- `require()` of ESM `dist/plugin.js` fails with `ERR_REQUIRE_CYCLE_MODULE`. Installed Path B loads committed `plugin.bundle.cjs`.

Injected `startupEvents` remain candidate events. Scheduler ticks on DRAFT rows still log `daemon-private Pi candidate transport is not configured` (Pi not configured); Workspace* completion used the AKP candidate path, not Pi.

## D03 same-host paired observation

Host: `B01-Desktop-Linux-002` only. n=5 sequential Path A then Path B. Path A key via `--api-key-file -` (stdin); never argv/env/logs. Path B uses SecretStore-bound Flash on the already-running disposable daemon.

| Path | retained / started | discarded | elapsed p50 (min / p95 / max) ms | TTFT p50 ms | cold elapsed ms |
|---|---|---|---|---|---|
| A `dsh → Flash` direct | 5 / 5 | 0 | 10628 (9655 / 11530 / 11530) | 10466 | 11530 |
| B `dsh → AKP → daemon → Flash` | 5 / 5 | 0 | 10800 (10080 / 11764 / 11764) | 10655 | 10080 |

- `overhead_b_minus_a_p50_ms`: **172**
- Warm A elapsed_ms: 11035, 10628, 10503, 9655
- Warm B elapsed_ms: 11405, 10800, 10735, 11764
- Timeouts/errors: 0 retained failures
- Provider network dominates (~10.5 s). 172 ms p50 is ~1.6% of Path A p50 and includes Path B Workspace* admits plus SSE-to-unary conversion. Not lossless. No further product change is justified at this n: the gap is smaller than Provider jitter (Path A range 9655–11530 ms).
- Limitation: Path B samples ran after Path A, so Path B "cold" is not a first-process cold of the daemon. TTFT is first dsh stdout byte, not a streaming token timestamp.

## Non-claims

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, EVAL, or
Agent-benefit promotion. Do not preset lossless. dsh response ≠ Task
completion. `dsh.json` digest ≠ durable daemon adapter SQLite.

## Unique next action

Ready/merge PR [#256](https://github.com/agentkernel/cognitive-os/pull/256),
delete the task branch, and reconcile local `main` to `origin/main`. Do not
auto-claim P6 / P7-T05 / P7-T06 / P7-T07.
