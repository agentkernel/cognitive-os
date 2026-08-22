# P8-T11 Provider streaming, dsh runtime inspect, real-task A/B (2026-08-22)

- Change class: product-semantic Personal task `P8-T11`.
- Target: `B01-Desktop-Linux-002` (`linux-002`) through
  `wuz@192.168.1.2` -> ProxyJump -> `hal9001@192.168.123.160`.
- Claim ceiling: implementation evidence / tested-local / linux-002 /
  performance observation only; no Gate, release, Profile, B01, EVAL, or
  Agent-benefit claim.

## Task and lease

- Task: `P8-T11` (`done`)
- Slice: `P8-T11/D04`
- Branch: `personal/P8-T11-provider-stream-dsh-runtime`
- PR: [#257](https://github.com/agentkernel/cognitive-os/pull/257)
- Lease: closed `lease/personal/P8-T11/provider-stream-dsh-runtime`
- P8-T10 remains `done` (PR [#256](https://github.com/agentkernel/cognitive-os/pull/256))
- P8-T09 remains `done` (PR [#254](https://github.com/agentkernel/cognitive-os/pull/254))

## Pins

- Product evidence revision (linux-002 D03):
  `4b191740c817d6075e9a93c883a93d94f65c350b`
- dsh git revision: `528c682e061696f5a160f363f236ecbf53cbd006`
- AKP request-envelope schema digest:
  `sha256:feeaeb0942ce2796d0155b4b9c316a87cca94eccbf7b0fd7b031a2135dd7ee7b`
- Model: `deepseek-v4-flash`
- Path A base URL: `https://api.deepseek.com`
- Disposable daemon: `127.0.0.1:48521` (stopped at cleanup)
- Guest install root: `/home/hal9001/p8t11-e48517cb` (product `cognitive`
  + `kernel-server`, adapter tree, pinned dsh with `build:lib` overlay)
- Runtime used: `/home/hal9001/p8t11-e48517cb/runtime-clean`
- Dirty runtime `/home/hal9001/p8t11-e48517cb/runtime` restart still fails
  with scheduler lease-conflict (retained P8-T09 limitation; not used)

## Latency root cause

P8-T10 Path A/B ~10.5 s was **not** Provider TTFB and **not** the 172 ms
adapter overhead. Same-host raw Flash streaming at this SHA is ~1.1 s elapsed
/ ~270 ms TTFB. Diagnostic breakdown:

1. **tsx-from-source on the 2 vCPU / 4 GiB guest** loads the dsh monorepo
   through `node --import tsx/esm`. Cold/warm tsx stayed ~10.5–13 s even with
   `NODE_COMPILE_CACHE` and `thinking: disabled`. That matched P8-T10 Path A
   **and** Path B because both spawned tsx dsh.
2. **SSE-to-unary bridge** (P8-T10 Path B) was a real product defect relative
   to streaming, but it was not the 10.5 s. D01 now forwards public
   `stream:true` as HTTP/1.1 SSE (`crates/cognitive-provider-transport/src/stream_http.rs`).
   UnexpectedEof after TLS close is treated as end-of-stream. Pi and
   private-candidate stay unary. Path B no longer uses `provider-sse-bridge.mjs`.
3. **Compiled CLI** (`apps/cli/lib/bin.js` + `packages/api/gateway/lib/index.js`
   after `pnpm build:lib` on the pin) drops Path A/B into the ~3.2–3.7 s band.
   Remaining wall time is dsh process spawn plus Flash; raw Provider stays ~1 s.

Fix that shipped: product `dsh-real-process.mjs` / `cognitive dsh launch`
prefer compiled-lib; pin Flash thinking off; durable Node compile cache;
seed disposable `README.md` so WorkspaceRead is not stuck `ACTIVE`; public
SSE pass-through; runtime `op: clear` drops in-memory sessions.

## Validation ledger

| Check | Result | Evidence / limitation |
|---|---|---|
| linux-002 identity | **pass** | libvirt `B01-Desktop-Linux-002`, UUID `f7bb6a52-2a0b-4ecb-8e8f-f4c60ca472a0`; guest hostname `hal9001-Standard-PC-Q35-ICH9-2009`; Ubuntu 24.04.4; Node v22.23.2 |
| D01 public SSE | **pass** implementation | dedicated rustls HTTP/1.1 stream client; first-byte flush; private-candidate stream refuse; Pi unary |
| D02 runtime inspect | **pass** linux-002 | `GET /personal/dsh/runtime` + `cognitive dsh status`: live `ACTIVE` `process_alive: true`; kill **status** pid (dsh child, not helper) → `CRASHED` `process_alive: false`; `op: clear` → `INACTIVE` sessions 0. Liveness is `/proc/{pid}` directory only |
| Installed Path B product launch | **pass** linux-002 at `4b191740` | `cognitive dsh configure` exact pin; `cognitive dsh launch --print --path b`; `cli_mode: compiled-lib`; assistant_ok (non-`pong` one-sentence summarize); WorkspaceRead/Search/Write `COMPLETED`; elapsed 4079 ms; TTFT 3939 ms |
| Fail-closed spot-check | **pass** | product `--path a` rc 1 (`direct Flash path is measurement-only`); wrong revision configure rc 1 (exact pin `528c682e…`) |
| Raw vs Path A vs Path B n=5 | **pass** linux-002 at `4b191740` | see tables; discarded 0/5 both paths; Path B retained implies Workspace* `COMPLETED` + assistant_ok (helper exit 0) |
| Secret residue cleanup | **pass** | disposable daemon 48521 stopped; EVAL 48181/48284/48383 untouched; `secret-tool clear` product triple rc 0; bootstrap shredded; jump ed25519 `/tmp/p8t11-jump` absent; `# p8t11-temp` authorized_keys lines removed (2). Residue scan 19574 files / 11 skipped / 7 pattern hits: 5 in pinned dsh **vendor test fixtures**, 1 skipped pre-existing `/tmp/eval006-pi-cognitiveos-dist.tar` (untouched), 1 `/tmp/node-compile-cache` V8 cache (not opened; likely vendor `sk-` literals). Key file existence/length only: 35 bytes |
| Jump-host Rust | **pass** | `DEV-LINUX-NATIVE-01` at `4b191740`: `cognitive-provider-transport` unit+integration including delayed-sse first-chunk-before-last and UnexpectedEof-as-EOS; `p8_t11_dsh_runtime` 1/1; admin-cli `dsh` 7/7; Clippy `-D warnings` for transport + kernel-server `--all-targets`; `cargo fmt --all -- --check`. Local Windows GNU Rust `not-run` |
| Required CI | **pass** | Ubuntu, Windows, and required-ci `32551050984` at `849c01a5`. Earlier `32549721560` failed unique-slice consistency (D01+D02 both in-progress). `32550651605` Windows failed delayed-SSE first-byte budget that included ~2 s loopback TLS handshake; follow-on clocks first-byte from HTTP status |

## D03 installed Path B (linux-002)

Guest has no cargo; binaries built on the jump host. dsh tree is the P8-T10
copy plus jump `build:lib` overlay (no `.git`; pin file present). Product SHA
file `PRODUCT_SHA` = `4b191740c817d6075e9a93c883a93d94f65c350b`.

1. `cognitive init --api-key-file -` (SecretStore `linux-secret-tool`; later
   `--reuse-existing-secret-binding`; `secret_material_written` false on reuse)
2. `cognitive daemon start --bind 127.0.0.1:48521`
3. `cognitive doctor` overall `ready`; Pi `not_configured`
4. `cognitive dsh configure --dsh-root … --adapter-root … --revision 528c682e061696f5a160f363f236ecbf53cbd006`
5. `cognitive dsh launch --print --path b` with the non-`pong` summarize task

Control plane: before launch `INACTIVE` sessions 0; during launch `ACTIVE` and
bound pid alive; after helper `clear` / status `INACTIVE`. Crash probe killed
the pid from `cognitive dsh status` (dsh child). Launch JSON `process_id` is
the helper and must not be used as the kill target.

## D03 same-host paired observation

Host: `B01-Desktop-Linux-002` only. Sequential Path A then Path B. `n=5`
retained per path. `lossless_preset: false`. Path A key via `--api-key-file -`
(stdin); never argv/env/logs. Path B uses SecretStore-bound Flash on the
already-running disposable daemon. LLM task is one-sentence summarize of
CognitiveOS Personal (not `pong`). Path B also runs WorkspaceRead (`README.md`),
WorkspaceSearch (`needle`), and disposable WorkspaceWrite.

| Probe / path | retained / started | discarded | elapsed p50 (min / p95 / max) ms | first-byte p50 ms | cold elapsed ms |
|---|---|---|---|---|---|
| Raw Provider stream (no dsh) | 1 / 1 | 0 | 1056 (TTFB 270; body 20083 B; HTTP 200) | 270 (TTFB) | 1056 |
| A compiled dsh → Flash direct | 5 / 5 | 0 | 3181 (2903 / 3264 / 3264) | 3000 | 3118 |
| B compiled dsh → AKP → daemon SSE → Flash | 5 / 5 | 0 | 3654 (3211 / 3935 / 3935) | 3534 | 3683 |

- `overhead_b_minus_a_p50_ms`: **473**
- Warm A elapsed_ms: 2903, 3181, 3264, 3233
- Warm B elapsed_ms: 3459, 3211, 3654, 3935
- Timeouts/errors: 0 retained failures
- Compared with P8-T10 tsx Path A p50 **10628** / Path B **10800**: compiled-lib
  removes ~7 s of guest JIT. Remaining A−raw ≈ 2.1 s is dsh spawn/runtime, not
  Provider TTFB. Path B−A p50 473 ms includes Workspace* admits plus daemon SSE
  proxy; do not hide that governance cost. Not lossless.
- Limitation: TTFT is first **dsh stdout** byte, not a streaming token
  timestamp. Path B samples ran after Path A. Dirty-runtime restart still fails
  scheduler lease-conflict.

## Non-claims

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, EVAL, or
Agent-benefit promotion. Do not preset lossless. dsh response ≠ Task
completion. `dsh.json` digest ≠ durable daemon adapter SQLite. Compiled-lib
~3.2–3.7 s is still dsh process spawn + Flash, not a zero-overhead claim.
Dirty-runtime restart remains a known P8-T09 limitation.
