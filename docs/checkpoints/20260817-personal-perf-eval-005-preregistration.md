# PERSONAL-PERF-EVAL-005 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-005`
- Lease: `lease/personal/EVAL-005/c1-c2-paired-freeze`
- Date: 2026-08-17
- Frozen product source: `origin/main@b16d29556eb4113ead3661f186e615c3183962a9`
  (merge of P2-T29 closure PR #231). Product enablement merged as PR #230 at
  `98cb23d1`.
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reuse EVAL-004 campaign root
`/home/hal9001/perfeval004` or `/home/hal9001/perfeval004-20260816`, loopback
ports `48286`/`48386`, SecretStore items `/12`/`/13`, broker, runner, corpus,
oracle, or evidence denominator. `PERSONAL-PERF-EVAL-004` and
`PERSONAL-PERF-EVAL-002` remain closed.

## Owner authorization

Owner 2026-08-17: after EVAL-004 closure, deliver P2-T29 (C1/C2 product
mutexes) then measure with a new freeze. P2-T29 merged PR #230; lease closed
PR #231 at `main@b16d2955`.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval005-20260817` mode `0700` | `perfeval004`, `perfeval004-20260816`, `~/perfeval002`, `~/p9t04`, `cos-current` |
| Loopback port | `127.0.0.1:48288` daemon; broker `127.0.0.1:48388` | `48181`, `48282`, `48284`, `48286`, `48383`, `48386` |
| SecretStore entry | new item via product stdin (`cognitive init --api-key-file -`) | `/11`, `/12`, `/13`; never `secret-tool search`/`lookup` |
| Source pin | `main@b16d2955` (P2-T29 closed) | EVAL-004 pin `1e71344a` |

`B01-Clean-Linux-001` stays out of bounds. Snapshot revert/delete, P9-T04
residue, and the owner plaintext key file are not in this freeze's allowlist.
Rotate the previously leaked Provider key if that item is still in use.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| P2-T29 merged and lease closed | **pass** | PR #230 product; PR #231 closure at `main@b16d2955` |
| Evaluation lease claimed | **pass** | this document + Current snapshot row |
| Product source pin | **pass** | `b16d29556eb4113ead3661f186e615c3183962a9` |
| Source archive + SHA-256 | **pass** | `git archive --format=tar --prefix=cognitiveos-personal-b16d2955/` of `b16d2955`; 14,510,080 bytes; 1529 entries; 0 `.git/` members; `sha256:af2836ddd807f592110387e3e60eca5f2105b2464a22fcbc534ab08e98f6922a` matched on Windows, `DEV-LINUX-NATIVE-01`, and the guest |
| New campaign root/port | **pass** | `/home/hal9001/perfeval005-20260817` mode `0700`; `127.0.0.1:48288` unused then bound. EVAL-004 roots and listeners `48181`/`48284`/`48383` left untouched; `48286`/`48386` absent |
| Exact-source daemon/CLI binaries | **pass** | `DEV-LINUX-NATIVE-01` `cargo build --release --locked -p kernel-server -p admin-cli` from extracted archive, 40.68 s, Rust 1.97.1. `kernel-server` 16,477,528 bytes `sha256:00b8963ce991e782f180b315ba731dcac6001581201547b8dd5dc9b97916410a`; `cognitive` 10,313,952 bytes `sha256:760ad2c7f3cbd90906b15f3ccf2344e8b0fa82baefc0ee1486f24fa5aa15afe5`. Guest `ldd` resolves only glibc/`libgcc`/`libm` |
| Campaign daemon on `48288` | **pass** | public `cognitive daemon start --runtime-root …/runtime --bind 127.0.0.1:48288 --kernel-server …/kernel-server`; pid 267060; lock live; bootstrap present (value not read). Pre-credential status: provider `blocked` (`provider_config_missing`), pi `not_configured`, `first_conversation_ready: false` |
| New SecretStore entry | **pass** | product stdin import into **new** item `/org/freedesktop/secrets/collection/login/14` (not `/12`/`/13`). Owner file `~/下载/deepseek.txt` used by shape only; key travelled `sed -n '8p'` → pipe → `--api-key-file -`. D-Bus `SearchItems` paths only; never `secret-tool search`/`lookup`. See § SecretStore import |
| Local Pi `0.81.1` pin | **pass** | guest-local npm pack+install under the new root; `--extension` absolute path only. Doctor: package/pinned/observed `0.81.1`, `first_conversation_ready: true`. See § Pi pin |
| Exact-source `pi-agent-adapter` | **pass** | same extracted `b16d2955` archive; host `CARGO_NET_OFFLINE=true cargo build --release --locked -p pi-agent-adapter` 1 m 10 s. See § Adapter freeze |
| C1/C2 paired B0 | **partial** | two O-arm C1-search samples retained; skip class `scheduler_row_skip_before_lease`; P-arm not started. See § Private-candidate skip |
| C1/C2 paired B1/B2 | **not-run** | B0 never left `DRAFT`; no Provider spend |
| T8 / B3 stale | **not-run** | no public invocation / no mutation path |
| MS-AUTH Memory positives | **pass** | 10/10 unsealed lifecycle + caller-header 400. See running report |

## Non-claims

This campaign creates no Gate, release, Profile, B01, or Agent-benefit
promotion. `retry=0` for Provider cells. Every started sample is retained.
Each finished cell is appended to the running report immediately.

## Archive and guest-root pin (2026-08-17)

Host `wuz@192.168.1.2` (`DEV-LINUX-NATIVE-01` / libvirt host `hal9000`) and
guest `hal9001@192.168.123.160` (`B01-Desktop-Linux-002`, running; identity
`hal9001-Standard-PC-Q35-ICH9-2009`) were contacted only on the registered
route. `B01-Clean-Linux-001` remained shut off and was not contacted.
`virsh -c qemu:///system` was read-only (UUID
`f7bb6a52-2a0b-4ecb-8e8f-f4c60ca472a0`, 2 vCPU, 4 GiB). Guest kernel
`7.0.0-28-generic`, glibc `2.39`. Guest disk 41G free (28%).

The source archive is a clean `git archive --format=tar
--prefix=cognitiveos-personal-b16d2955/` of
`b16d29556eb4113ead3661f186e615c3183962a9` (1529 entries, 0 `.git/` members).
SHA-256 `af2836ddd807f592110387e3e60eca5f2105b2464a22fcbc534ab08e98f6922a`
matched on the Windows operator host, on `DEV-LINUX-NATIVE-01` at
`/home/wuz/eval005-freeze-20260817/cognitiveos-personal-b16d2955.tar`, and
on the guest at
`/home/hal9001/perfeval005-20260817/cognitiveos-personal-b16d2955.tar`.
The guest root was created `0700`. Prior campaign roots
`/home/hal9001/perfeval004` and `/home/hal9001/perfeval004-20260816` and
pre-existing loopback listeners `127.0.0.1:48181`, `127.0.0.1:48284`, and
`127.0.0.1:48383` were left untouched. Ports `48286`/`48386` were not
listening. Port `48288` was not listening before daemon start.

Windows GNU Rust build remains `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).

## Exact-source binaries (2026-08-17)

`CARGO_NET_OFFLINE=true cargo build --release --locked -p kernel-server -p
admin-cli` finished in 40.68 s on `DEV-LINUX-NATIVE-01` (Rust 1.97.1) from
the extracted `b16d2955` archive. Guest `ldd` resolves only
glibc/`libgcc`/`libm` for both binaries.

| Binary | Bytes | SHA-256 |
|---|---:|---|
| `kernel-server` | 16,477,528 | `00b8963ce991e782f180b315ba731dcac6001581201547b8dd5dc9b97916410a` |
| `cognitive` | 10,313,952 | `760ad2c7f3cbd90906b15f3ccf2344e8b0fa82baefc0ee1486f24fa5aa15afe5` |

Pi Extension `tsc` from the same extracted archive:

| Asset | SHA-256 |
|---|---|
| `dist/index.js` (barrel) | `d27f97764e55b9a9b22bbf7e22e48c0ef2a017924ed13684b143b196991c1a57` |
| `dist/extension.js` | `d5ba4e47d2e05a260f9c5e3850572edf228628ab02c78e7acd75c98f2278d880` |
| `dist/workspace-tools.js` | `233d77268519992453293ea9bde463ad548db6e720c22e3478b0322301336c5a` |
| `dist/tool-policy.js` | `4ce7dc2f4c6f2381805ed5c0ba66d4cd1f5ccdff712d6ae9c2a845601cb2916c` |
| dist archive | `332fb0d93293c61fd3be65554bb9a2439780589fb6ecd31211b665f82c3063b1` |

`dist/extension.js` registers WorkspaceSearch/Write/Patch via `pi.registerTool`;
`tool-policy.js` lets those names through and still blocks bash/edit/write.
WorkspaceRead is **not** advertised as a Pi tool at this revision.

## Campaign daemon start (2026-08-17)

Public caller:

`/home/hal9001/perfeval005-20260817/cognitive daemon start --runtime-root /home/hal9001/perfeval005-20260817/runtime --bind 127.0.0.1:48288 --kernel-server /home/hal9001/perfeval005-20260817/kernel-server`

Result: `action=started`, pid `267060`, endpoint `127.0.0.1:48288`, lock
`…/runtime/cognitiveos/daemon.lock`, `bootstrap_present=true` (value not read).
Pre-credential `cognitive status`: system/database/secret/daemon `ready`,
provider `blocked` (`provider_config_missing`), pi `not_configured`,
`first_conversation_ready: false`, `authority_side_effects: false`. Listeners
`48181`/`48284`/`48383` still present and untouched.

## SecretStore import (2026-08-17) — pass

Operating Model §2.3 plus standing operator authorization: import the
owner-designated local test Provider key through the product stdin path into
an approved Secret Store. A **new** campaign-only item was required; `/12`
and `/13` must not be reused.

Owner file `~/下载/deepseek.txt` was located by name and inspected by shape
only (line lengths, character-class mask, `sk-` prefix on line 8). Non-secret
flag lines: `--provider deepseek`, `--base-url https://api.deepseek.com/v1`,
`--model-id deepseek-v4-flash`. Line 8 is 35 bytes and `sk-`-prefix shaped;
its bytes were never printed. Owner-file mtime remained
`2026-08-12 12:51:02 +0800` after import.

Pre-import D-Bus `SearchItems` on the product attribute triple returned
`aoao 0 0` (zero unlocked, zero locked paths). `secret-tool search` /
`lookup` were not used.

Import caller (key never in argv/env):

`sed -n '8p' /home/hal9001/下载/deepseek.txt | /home/hal9001/perfeval005-20260817/cognitive init --runtime-root /home/hal9001/perfeval005-20260817/runtime --provider deepseek --base-url https://api.deepseek.com/v1 --model-id deepseek-v4-flash --api-key-file -`

with `XDG_RUNTIME_DIR=/run/user/1000` and
`DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus`.

Product report (redacted): `status=ok`, `action=configured`,
`secret_backend=linux-secret-tool`, `secret_material_written=true`,
`secret_ref_redacted=true`, `provider_id=deepseek`,
`selected_model=deepseek-v4-flash`,
`snapshot_digest=fnv1a64:c58ce6f2f7521544`,
`profile_claim=not-claimed`, `gate_claim=not-claimed`,
`authority_side_effects=false`.

Post-import D-Bus `SearchItems`: **new** path
`/org/freedesktop/secrets/collection/login/14` (1 unlocked, 0 locked). This
is not `/12` or `/13`. Old EVAL-004 `provider.json` files were not copied.

Public `cognitive status` after import: provider `ready` (79 ms), secret
`ready`, pi still `not_configured`.

## Pi 0.81.1 pin (2026-08-17) — pass

Guest-local, no global install, no reuse of EVAL-004 roots.

| Asset | Digest / version |
|---|---|
| Pi tarball `@earendil-works/pi-coding-agent@0.81.1` | 4,967,228 bytes; `sha256:420113c0282160e6181656fd16cf18742f76bf9040ee3dfb9cb67e3e6ad5641c`; npm integrity `sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==` |
| Pi runtime `package-lock.json` | `sha256:8a6ef5b2b0ed1127785989e3b6d15af4cd4913124453aa0085c848d0cc9857f7` |
| Pi executable | `/home/hal9001/perfeval005-20260817/pi-runtime/node_modules/@earendil-works/pi-coding-agent/dist/cli.js`; `pi --version` → `0.81.1` |
| `pi.json` | `sha256:611ee6756c5b2eacb61f3a032acfb482e3160dc05e02e0286089ac8fa6ce9806` |

`pi --version` with `--extension <absolute-path>` also reports `0.81.1`.
Public caller:

`/home/hal9001/perfeval005-20260817/cognitive pi configure --runtime-root /home/hal9001/perfeval005-20260817/runtime --executable /home/hal9001/perfeval005-20260817/pi-runtime/node_modules/@earendil-works/pi-coding-agent/dist/cli.js --extension-entry /home/hal9001/perfeval005-20260817/pi-cognitiveos/dist/index.js`

`pi.json` contains only non-secret absolute paths (`schema_version` 1,
surface `personal-pi-config`). Public status after configure: all six
components `ready`, `first_conversation_ready: true`, pi probe ~1.7 s.
Doctor: `package_status=ready`, `pinned_version=0.81.1`,
`observed_version=0.81.1`, `secret_ref_present=true`,
`secret_ref_resolves=true`, `secret_ref_redacted=true`,
`secret_material_exposed=false`. This is readiness evidence only — no
conversation, Task, Tool, Effect, or sample.

Private-candidate O-arm requires exact-source `pi-agent-adapter` plus
`--candidate-adapter` / `--candidate-extension`. The frozen product
`private_candidate_provider.mjs`
(`sha256:2b7e52a6afe205e5997c58fe59b096fc7666dfd8733e196777e915d3a0bc245b`)
registers a socket Provider only and does **not** advertise Workspace*.
Those paths are now configured (see § Adapter freeze).

## Adapter freeze (2026-08-17) — pass

Exact-source `pi-agent-adapter` was built on `DEV-LINUX-NATIVE-01` from the
extracted `b16d2955` archive (`CARGO_NET_OFFLINE=true cargo build --release
--locked -p pi-agent-adapter`, 1 m 10 s, Rust 1.97.1). Guest `ldd` resolves
only glibc/`libgcc`. EVAL-004 roots and listeners were not read or changed.

| Asset | Bytes | SHA-256 |
|---|---:|---|
| `pi-agent-adapter` (host and guest) | 1,125,592 | `5a082cb5ee5fac056c67632c729ab7fec0cabaccd9fc2db4389ebf58dc14ee49` |
| `private_candidate_provider.mjs` | 6,729 | `2b7e52a6afe205e5997c58fe59b096fc7666dfd8733e196777e915d3a0bc245b` |
| `adapter-bundle.tar` | 1,136,640 | `7a187ab6441036e4a4d9394013d0b73a115e2d078db710b82eaeef3763f6f74f` |
| `pi.json` after candidate-path configure | — | `9d97e8e1958d78c1946bfe13a3e426c4c946818f06203625e7027ae489bb3ebc` |
| campaign `o-arm-candidate.mjs` (quoted ESM; repaired) | — | `3d9c03dbac41dca93cc1704370e8c637557dbfef32d2fb91104d2d84a1889a53` |

Public caller:

`/home/hal9001/perfeval005-20260817/cognitive pi configure --runtime-root /home/hal9001/perfeval005-20260817/runtime --executable /home/hal9001/perfeval005-20260817/pi-runtime/node_modules/@earendil-works/pi-coding-agent/dist/cli.js --extension-entry /home/hal9001/perfeval005-20260817/pi-cognitiveos/dist/index.js --candidate-adapter /home/hal9001/perfeval005-20260817/pi-agent-adapter --candidate-extension /home/hal9001/perfeval005-20260817/o-arm-candidate.mjs`

`pi.json` contains only non-secret absolute paths. Campaign wrapper
`o-arm-candidate.mjs` is a guest-root instrument: it loads the frozen
private-candidate Provider and registers the frozen
`daemonGovernedWorkspaceTools()` list. The first copy used unquoted ESM
specifiers and `node --check` raised `SyntaxError`; it was replaced in
place with quoted specifiers (same two frozen modules). Product code was
not modified.

Public `cognitive status` / `doctor` after configure: all required
components `ready`, `first_conversation_ready: true`, Pi `0.81.1`. Daemon
pid 267060 still bound to `127.0.0.1:48288`. Listeners `48181` / `48284` /
`48383` untouched; `48286` / `48386` / `48388` absent.

## B0 C1-search O-arm (2026-08-17) — partial; samples retained

Two O-arm C1-search qualification Tasks were started with `retry=0` against
the public Task admit surface. Both are retained. Neither left `DRAFT`.
No Intent, Effect, verification, or acceptance row exists. WorkspaceRead
is still not advertised; this cell used WorkspaceSearch only.

| Seed | Task ref | Admit | Terminal at probe | Lifecycle now |
|---|---|---|---|---|
| `b0-0` (first) | `task://local/eval005-b0-C1-search-b0-0-757d5b66ffae` | 200 (observation `runnable_count`) | probe JSON overwritten by the second start | `DRAFT`; minted `2026-08-17T07:21:17.175Z` |
| `b0-0` (second) | `task://local/eval005-b0-C1-search-b0-0-ab6c3c389d2d` | 200 | probe wall 180825 ms; `acceptance_ref` absent | `DRAFT`; minted `2026-08-17T07:21:44.869Z` |

Guest evidence file:
`/home/hal9001/perfeval005-20260817/evidence/b0-oarm-C1-search-b0-0.json`
covers only the second start. Public `cognitive task evidence` confirms
both refs remain `DRAFT` with empty `intent_refs` / `effect_refs`. Bounded
observation plane records repeating O4 `runnable_count` / `queue_wait` for
both refs (ring ~256 samples; file still updating). No
`pi-agent-adapter` or Pi child was observed. Campaign daemon stdout/stderr
are `/dev/null` (public `cognitive daemon start`); skip errors are not in
the observation plane. `strace` attach was denied. EVAL-004 processes and
ports were not touched.

P-arm equivalent Workspace* adapter and broker `127.0.0.1:48388` were not
started. Paired B1/B2 must not begin while B0 path/fairness is incomplete.

This is not a performance result and creates no Agent-benefit claim.

## Private-candidate skip (2026-08-17)

Campaign-only observation. Skip class **`scheduler_row_skip_before_lease`**.
Public status/doctor remain ready (`first_conversation_ready: true`); that
is not C1/C2. Both retained Tasks stay `DRAFT` with empty Intent/Effect.
O4: `runnable_count` 32/32, `lease_acquired` 0/0. Campaign kernel-server
pid 267060 stderr is `/dev/null`, so the per-row skip string is not a
public fact. Adapter/`pi.json`/selected-model files exist; this is not a
missing-asset `not_available`. Remaining paired C1/C2, T8, and B3 stale
are `not-run`. MS-AUTH Memory positives **pass** (10/10). See the running
report.

## Unique next action

Keep the two started C1-search samples retained. Campaign stays **active**
until the owner closes it. Do not open B1/B2 Provider spend. Do not claim
`P*-T*` or patch product. Cleanup (daemon/broker/SecretStore `/14`) waits
for owner close. Do not reuse EVAL-004 roots/ports or SecretStore
`/12`/`/13`. Never `secret-tool search`/`lookup`.
