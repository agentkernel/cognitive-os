# PERSONAL-PERF-EVAL-012 assessment (running)

- Campaign: `PERSONAL-PERF-EVAL-012`
- Frozen source target: `370b26fcc05976c7c1c97e5510a99ed3ebc23f2c` (P9-T08
  merged; docs-head after PR [#247](https://github.com/agentkernel/cognitive-os/pull/247))
- Lease: `lease/personal/EVAL-012/c1-c2-paired-b0` (active 2026-08-20)
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: campaign **active**. Measurement-only. Evaluation routing ON.

This is the campaign's single report (`TEST-REPORT-INCREMENTAL-01`). Append
each finished cell immediately. Do not hold conclusions until the end of a
batch.

Owner 2026-08-20 activated this EVAL. Closed EVAL-002 and EVAL-004 through
EVAL-011 are not resumed. Packages 1–14 remain readiness evidence, not B0.

## Cells

| Cell | Status | Note |
|---|---|---|
| Closed EVALs remain closed (coordination) | **pass** | do not reuse `48286`–`48298` / `48386`–`48398` / `48383` / `/12`–`/19` |
| Owner activation | **pass** | Current snapshot `PERSONAL-PERF-EVAL-012` **active** |
| Evaluation lease claimed | **pass** | `lease/personal/EVAL-012/c1-c2-paired-b0` on `evaluation/EVAL-012-freeze` |
| Guest identity | **pass** | `B01-Desktop-Linux-002` running; MAC `52:54:00:33:27:c1`; guest `192.168.123.160`; `B01-Clean-Linux-001` shut off, not contacted |
| Freeze (archive/binaries/root/port) | **pass** | pin `370b26fc`; root `/home/hal9001/perfeval012-20260820` mode `0700`; `48300`/`48400` free; leftover `48181`/`48284`/`48383` untouched |
| Secret bind | **pass** | E9: product stdin import into new item suffix `/24` (≠ `/12`–`/19`; keyring allocator skipped retired ids). `secret_material_written: true`, `secret_ref_redacted: true`, `selected_model=deepseek-v4-flash`. Guest temp shredded. No search/lookup, no `provider.json` copy |
| Pi 0.81.1 pin | **pass** | in-campaign `@earendil-works/pi-coding-agent@0.81.1`; `cli.js --version` `0.81.1`; extension `index.js` digest matches host freeze |
| `cognitive doctor` | **pass** | overall `ready`; `first_conversation_ready: true`; `secret_ref_resolves: true`; `selected_model_digest_matches: true`; Pi `0.81.1`; daemon `127.0.0.1:48300` pid 326605. Conversation readiness is **not** C1/C2 |
| B0 C1 / C2a / C2b / C2c / C2d | `not-run` | E9 passed; one qualification seed per class; three warmups per arm |
| B0 P-arm / broker `48400` | `not-run` | after O-arm bind and fairness check |
| B1/B2 C1/C2 paired | `not-run` | B0 not started |
| Cleanup | `not-run` | stop `48300`/`48400`; clear only the campaign SecretStore item |

## Activation (2026-08-20) — pass

Owner instruction “激活” set the Current snapshot row. Isolation reserved in
P9-T08 is now bound in the preregistration: root
`/home/hal9001/perfeval012-20260820`, daemon `127.0.0.1:48300`, broker
`127.0.0.1:48400`, SecretStore planned `/20`. Provider budget ceiling **1010**
counted C1/C2 arm-runs (B0 sub-ceiling 10). Freeze mutation on the new
campaign root is recorded below. No counted B0 sample has started.

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or Agent-benefit
claim.

## Guest identity (2026-08-20) — pass

Registered route: `wuz@192.168.1.2` (`hal9000`) `virsh -c qemu:///system`,
then ProxyJump `hal9001@192.168.123.160`. Domain `B01-Desktop-Linux-002`
(uuid `f7bb6a52-2a0b-4ecb-8e8f-f4c60ca472a0`) is **running**. Guest NIC
`enp1s0` MAC `52:54:00:33:27:c1` matches the domain XML; address
`192.168.123.160/24`. Hostname `hal9001-Standard-PC-Q35-ICH9-2009`. Ubuntu
24.04.4 LTS. User `hal9001` uid `1000`. Session bus
`/run/user/1000/bus` exists. `B01-Clean-Linux-001` is **shut off** and was
not contacted. Snapshot restore/delete was not performed.

SSH sessions must export `XDG_RUNTIME_DIR=/run/user/1000` and
`DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus` for Secret Service.

## Freeze (2026-08-20) — pass

Campaign root `/home/hal9001/perfeval012-20260820` mode `0700`. Closed EVAL
roots `perfeval002` / `perfeval004` / `perfeval004-20260816` /
`perfeval005-20260817` / `perfeval006-20260817` / `perfeval007-20260817` /
`perfeval008-20260818` / `perfeval010-20260818` / `e009` / `p9t04` remain
in `$HOME` and were not reused. Listeners `127.0.0.1:48181` (`cos-current`),
`127.0.0.1:48284` (EVAL-004 residue), and `127.0.0.1:48383` were left
untouched. Campaign ports `48300` and `48400` were free. Runtime tree is
empty (no `provider.json`). Copies used `scp` (PowerShell SSH pipes corrupt
tar digests). Windows GNU Rust build remains `not-run`
(`RUST-LINK-DEV-WIN-GNU-01`).

Host `DEV-LINUX-NATIVE-01` built release binaries from exact `370b26fc` with
`CARGO_NET_OFFLINE=true`, a dedicated `CARGO_TARGET_DIR`, rustc 1.97.1.
Guest `ldd` on `kernel-server` shows only glibc / `libgcc_s` / `libm` /
`ld-linux`. Archive: 15,073,280 bytes; 1590 entries; 0 `.git/` members.

| Asset | SHA-256 |
|---|---|
| `cognitiveos-personal-370b26fc.tar` | `1b41aeb31b70cdd59e60a598174eca00cc3f7f2ad1d51d1a005c370b0b9c1cdd` |
| `kernel-server` | `cfcfdaa2315657511445742352bb5a2820964c429bdbebab108b04e0f300c3a8` |
| `cognitive` | `f02931df5b17f40ee1705443c042ca3c81d342fe39172f81eb7f0f7dd71ca802` |
| `pi-agent-adapter` | `54ce9eaa0e61febeff53d8e96b43f0d30570fcfb5fdd95e455715fe061991fce` |
| `pi-cognitiveos-dist.tar` | `51295727f721880767639ae4e0ba706e072b63591d2bc8d48e5239ba81808615` |
| extension `pi-cognitiveos/dist/index.js` | `d27f97764e55b9a9b22bbf7e22e48c0ef2a017924ed13684b143b196991c1a57` |

Guest-extracted instruments at
`src/cognitiveos-personal-370b26fc/tools/personal/c1-c2-paired/` match the
preregistration freeze ledger (fixtures, broker, fairness checker, runner,
`cells.json`, secret helper).

## Pi 0.81.1 pin (2026-08-20) — pass

In-campaign install under
`/home/hal9001/perfeval012-20260820/pi-runtime/node_modules/@earendil-works/pi-coding-agent/`
(`npm install --omit=dev @earendil-works/pi-coding-agent@0.81.1`).
`node …/dist/cli.js --version` → `0.81.1`. Guest `node` v22.23.2, npm 10.9.8.
This is the pin, not doctor readiness and not a C1/C2 Task. Closed-EVAL Pi
runtimes were not reused.

## Secret bind / E9 (2026-08-20) — pass

Earlier the same day, D-Bus `SearchItems` on the product attribute triple
found 0 items (EVAL-010 had cleared `/19`). `--reuse-existing-secret-binding`
was not executed.

Owner standing authorization (§2.3 designated local test Provider key) plus
the 2026-08-20 “最高授权请继续自主持续推进” instruction authorized product
stdin import into a **new** item. `cognitive init --api-key-file -` on the
campaign runtime root reported `action: configured`,
`secret_backend: linux-secret-tool`, `secret_material_written: true`,
`secret_ref_redacted: true`, `selected_model: deepseek-v4-flash`,
`snapshot_digest: fnv1a64:c58ce6f2f7521544`. Guest temp was shredded
(`KEY_GONE`). No `secret-tool search`/`lookup`, no `provider.json` copy, no
material on argv/env/chat/Git.

Post-import `SearchItems` (paths-only): `item_count_unlocked=1`,
`item_suffixes=["24"]`. Item `/24` is not `/12`–`/19`. The keyring allocator
did not reuse retired suffixes; planned `/20` was the reservation name, not
a required path number.

## Pi configure and daemon (2026-08-20) — pass

Public `cognitive pi configure` wrote non-secret `pi.json` with pinned
`cli.js` `0.81.1`, Extension `pi-cognitiveos/dist/index.js`,
`pi-agent-adapter`, and in-archive
`apps/pi-agent-adapter/fixtures/private_candidate_provider.mjs`.

Public `cognitive daemon start --bind 127.0.0.1:48300` reported
`action: started`, pid **326605**, `log_path` mode-managed under the
campaign runtime. `daemon status`: `process_alive: true`,
`bootstrap_present: true` (value not read; file mode `600`, 71 bytes).
Listeners `48181` / `48284` / `48383` remain untouched. Broker `48400` is
not started.

## Doctor (2026-08-20) — pass

Redacted `cognitive doctor --runtime-root …/runtime` (secret-shaped scan
clean): overall `ready`, `first_conversation_ready: true`,
`authority_side_effects: false`. Provider `secret_ref_resolves: true`,
`selected_model_digest_matches: true`, `secret_material_exposed: false`.
Pi `package_status=ready` / `pinned_version=0.81.1` /
`observed_version=0.81.1`. This is conversation-shell readiness, not a
C1/C2 Task.

## Unique next action

Run B0 qualification cells on this freeze (`retry=0`; three non-counted
warmups per arm; one counted seed per class C1/C2a/C2b/C2c/C2d; retain
every started sample). Start C1 O-arm. Do not open B1/B2. Claim ceiling
`hypothesis`.
