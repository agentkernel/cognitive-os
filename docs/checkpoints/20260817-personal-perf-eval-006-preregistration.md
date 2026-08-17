# PERSONAL-PERF-EVAL-006 freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-006`
- Lease: `lease/personal/EVAL-006/c1-c2-paired-freeze`
- Date: 2026-08-17
- Frozen product source: `origin/main@103fe776eb7a3aca4d1281aefbda34fdaa445e0b`
  (merge of P2-T30 closure PR #234). Product enablement merged as PR #233 at
  `b13655b9`.
- Target: `B01-Desktop-Linux-002` via `wuz@192.168.1.2` (libvirt host
  `hal9000`) ProxyJump `hal9001@192.168.123.160`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reuse EVAL-004 campaign roots
`/home/hal9001/perfeval004` or `/home/hal9001/perfeval004-20260816`, EVAL-005
root `/home/hal9001/perfeval005-20260817`, loopback ports
`48286`/`48288`/`48386`/`48388`, SecretStore items `/12`/`/13`/`/14`, broker,
runner, corpus, oracle, or evidence denominator. `PERSONAL-PERF-EVAL-005`,
`PERSONAL-PERF-EVAL-004`, and `PERSONAL-PERF-EVAL-002` remain closed.

## Owner authorization

Owner 2026-08-17: close EVAL-005, deliver the scheduler-lease product mutex,
then re-measure C1/C2 with a new freeze. P2-T30 merged PR #233; lease closed
PR #234 at `main@103fe776`.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval006-20260817` mode `0700` | `perfeval004`, `perfeval004-20260816`, `perfeval005-20260817`, `~/perfeval002`, `~/p9t04`, `cos-current` |
| Loopback port | `127.0.0.1:48290` daemon; broker `127.0.0.1:48390` | `48181`, `48282`, `48284`, `48286`, `48288`, `48383`, `48386`, `48388` |
| SecretStore entry | new item via product stdin (`cognitive init --api-key-file -`) | `/11`, `/12`, `/13`, `/14`; never `secret-tool search`/`lookup` |
| Source pin | `main@103fe776` (P2-T30 closed) | EVAL-005 pin `b16d2955`; EVAL-004 pin `1e71344a` |

`B01-Clean-Linux-001` stays out of bounds. Snapshot revert/delete, P9-T04
residue, and the owner plaintext key file are not in this freeze's allowlist.
**Rotate the previously leaked Provider key** (EVAL-004 `secret-tool search`
incident) if that item is still in use.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| P2-T30 merged and lease closed | **pass** | PR #233 product; PR #234 closure at `main@103fe776` |
| Evaluation lease claimed | **pass** | this document + Current snapshot row |
| Product source pin | **pass** | `103fe776eb7a3aca4d1281aefbda34fdaa445e0b` |
| Source archive + SHA-256 | **pass** | `git archive --format=tar --prefix=cognitiveos-personal-103fe776/` of exact `103fe776`; 14,571,520 bytes; 1532 entries; 0 `.git/` members; SHA-256 `d322be1555f987a096b7e4815b433950f84565579aa49facf54583821d797bf1`. See § Source archive |
| New campaign root/port | **pass** | `/home/hal9001/perfeval006-20260817` mode `0700`; daemon `127.0.0.1:48290`. Listeners `48181`/`48284`/`48383` untouched |
| Exact-source daemon/CLI binaries | **pass** | `DEV-LINUX-NATIVE-01` `cargo build --release --locked -p kernel-server -p admin-cli -p pi-agent-adapter` from extracted archive; Rust 1.97.1. See § Exact-source binaries |
| Campaign daemon on `48290` | **pass** | public `cognitive daemon start --bind 127.0.0.1:48290`; pid `273829`. See § Campaign daemon start |
| New SecretStore entry | **pass** | product stdin import into **new** item `/org/freedesktop/secrets/collection/login/15` (not `/12`/`/13`/`/14`). D-Bus `SearchItems` paths only; never `secret-tool search`/`lookup`. See § SecretStore import |
| Local Pi `0.81.1` pin | **pass** | guest-local npm pack+install under the new root; `--extension` absolute path only. Doctor: package/pinned/observed `0.81.1`, `first_conversation_ready: true`. See § Pi pin |
| Exact-source `pi-agent-adapter` | **pass** | same extracted `103fe776` archive; host release build copied to guest. See § Adapter freeze |
| C1/C2 paired B0 | `not-run` | freeze complete; next cell |
| C1/C2 paired B1/B2 | `not-run` | after B0 |
| Cleanup / campaign close | `not-run` | stop 48290/48390; clear the new SecretStore item without search/lookup; leave 48181/48284/48383 and EVAL-004/005 roots untouched |

## Source archive (2026-08-17)

`git archive --format=tar --prefix=cognitiveos-personal-103fe776/` of exact
`103fe776eb7a3aca4d1281aefbda34fdaa445e0b` on `DEV-LINUX-NATIVE-01`. Guest
copy lives at
`/home/hal9001/perfeval006-20260817/cognitiveos-personal-103fe776.tar`.
PowerShell SSH piping corrupts the digest; the verified copy used `scp`.

| Asset | Value |
|---|---|
| Bytes | 14,571,520 |
| Entries | 1532; 0 `.git/` members |
| SHA-256 | `d322be1555f987a096b7e4815b433950f84565579aa49facf54583821d797bf1` |

Windows GNU Rust build remains `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).

## Exact-source binaries (2026-08-17)

`CARGO_NET_OFFLINE=true cargo build --release --locked -p kernel-server -p
admin-cli -p pi-agent-adapter` on `DEV-LINUX-NATIVE-01` (Rust 1.97.1) from
the extracted `103fe776` archive. Guest `ldd` on `kernel-server` resolves
only glibc/`libgcc`/`libm`.

| Binary | Bytes | SHA-256 |
|---|---:|---|
| `kernel-server` | 16,536,080 | `47513386ae53c59a750e15c52c19735873c58368e46a6dcd2870b6524c1ec53c` |
| `cognitive` | 10,313,952 | `760ad2c7f3cbd90906b15f3ccf2344e8b0fa82baefc0ee1486f24fa5aa15afe5` |
| `pi-agent-adapter` | 1,126,192 | `816856b49674d06f025f535fe2bf5219dd9744ab899250a489538ea687aa3167` |

Pi Extension `tsc` from the same extracted archive:

| Asset | SHA-256 |
|---|---|
| `dist/index.js` (barrel) | `d27f97764e55b9a9b22bbf7e22e48c0ef2a017924ed13684b143b196991c1a57` |
| `dist/extension.js` | `d5ba4e47d2e05a260f9c5e3850572edf228628ab02c78e7acd75c98f2278d880` |
| `dist/workspace-tools.js` | `233d77268519992453293ea9bde463ad548db6e720c22e3478b0322301336c5a` |
| `dist/tool-policy.js` | `4ce7dc2f4c6f2381805ed5c0ba66d4cd1f5ccdff712d6ae9c2a845601cb2916c` |
| dist archive (full `tsc` output) | `99c553e4767b2f86f149c1fc692d756c6639cfe613f84901f405880a7fe71dd4` |

`dist/extension.js` registers WorkspaceSearch/Write/Patch via `pi.registerTool`;
`tool-policy.js` lets those names through and still blocks bash/edit/write.
WorkspaceRead is **not** advertised as a Pi tool at this revision.

## Campaign daemon start (2026-08-17)

Public caller:

`/home/hal9001/perfeval006-20260817/cognitive daemon start --runtime-root /home/hal9001/perfeval006-20260817/runtime --bind 127.0.0.1:48290 --kernel-server /home/hal9001/perfeval006-20260817/kernel-server`

Result: `action=started`, pid `273829`, endpoint `127.0.0.1:48290`, lock
`…/runtime/cognitiveos/daemon.lock`. Pre-credential `cognitive status`:
system/database/secret/daemon `ready`, provider `blocked`
(`provider_config_missing`), pi `not_configured`,
`first_conversation_ready: false`, `authority_side_effects: false`.
Listeners `48181`/`48284`/`48383` still present and untouched.

## SecretStore import (2026-08-17) — pass

Operating Model §2.3 plus standing operator authorization: import the
owner-designated local test Provider key through the product stdin path into
an approved Secret Store. A **new** campaign-only item was required; `/12`,
`/13`, and `/14` must not be reused.

Owner file `~/下载/deepseek.txt` was located by name and inspected by shape
only (line 8 length 35, `sk-` prefix). Bytes were never printed. Import
caller (key never in argv/env):

`sed -n '8p' /home/hal9001/下载/deepseek.txt | /home/hal9001/perfeval006-20260817/cognitive init --runtime-root /home/hal9001/perfeval006-20260817/runtime --provider deepseek --base-url https://api.deepseek.com/v1 --model-id deepseek-v4-flash --api-key-file -`

with `XDG_RUNTIME_DIR=/run/user/1000` and
`DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus`.

Product report (redacted): `status=ok`, `action=configured`,
`secret_backend=linux-secret-tool`, `secret_material_written=true`,
`secret_ref_redacted=true`, `provider_id=deepseek`,
`selected_model=deepseek-v4-flash`,
`snapshot_digest=fnv1a64:c58ce6f2f7521544`,
`profile_claim=not-claimed`, `gate_claim=not-claimed`.

Post-import D-Bus `SearchItems` (Python `dbus`, attributes/paths only):
**new** path `/org/freedesktop/secrets/collection/login/15` (1 unlocked, 0
locked). `login` collection `Items` contains only `/15`. This is not `/12`,
`/13`, or `/14`. `secret-tool search` / `lookup` were not used.

Public `cognitive status` after import: provider `ready` (84 ms), secret
`ready`, pi still `not_configured`.

## Pi 0.81.1 pin (2026-08-17) — pass

Guest-local, no global install, no reuse of EVAL-004/005 roots.

| Asset | Digest / version |
|---|---|
| Pi tarball `@earendil-works/pi-coding-agent@0.81.1` | 4,967,228 bytes; `sha256:420113c0282160e6181656fd16cf18742f76bf9040ee3dfb9cb67e3e6ad5641c`; npm integrity `sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==` |
| Pi runtime `package-lock.json` | `sha256:8a6ef5b2b0ed1127785989e3b6d15af4cd4913124453aa0085c848d0cc9857f7` |
| Pi executable | `/home/hal9001/perfeval006-20260817/pi-runtime/node_modules/@earendil-works/pi-coding-agent/dist/cli.js`; `pi --version` → `0.81.1` |
| `pi.json` after candidate-path configure | `sha256:40bf84219496d820f484e283e153d1932880f476c870f61f3a08dca05f416e41` |

`pi --version` with `--extension <absolute-path>` also reports `0.81.1`.

## Adapter freeze (2026-08-17) — pass

Exact-source `pi-agent-adapter` was built on `DEV-LINUX-NATIVE-01` from the
extracted `103fe776` archive together with `kernel-server`/`admin-cli`.
Guest `ldd` resolves only glibc/`libgcc`. EVAL-004/005 roots and listeners
were not read or changed.

| Asset | Bytes | SHA-256 |
|---|---:|---|
| `pi-agent-adapter` (host and guest) | 1,126,192 | `816856b49674d06f025f535fe2bf5219dd9744ab899250a489538ea687aa3167` |
| `private_candidate_provider.mjs` | — | `2b7e52a6afe205e5997c58fe59b096fc7666dfd8733e196777e915d3a0bc245b` |
| campaign `o-arm-candidate.mjs` (quoted ESM) | — | `29870821488451b5728f88c4612e1616fd65681adaf23011dd898d459428e573` |

Public caller:

`/home/hal9001/perfeval006-20260817/cognitive pi configure --runtime-root /home/hal9001/perfeval006-20260817/runtime --executable /home/hal9001/perfeval006-20260817/pi-runtime/node_modules/@earendil-works/pi-coding-agent/dist/cli.js --extension-entry /home/hal9001/perfeval006-20260817/pi-cognitiveos/dist/index.js --candidate-adapter /home/hal9001/perfeval006-20260817/pi-agent-adapter --candidate-extension /home/hal9001/perfeval006-20260817/o-arm-candidate.mjs`

`pi.json` contains only non-secret absolute paths (`schema_version` 1,
surface `personal-pi-config`). Campaign wrapper `o-arm-candidate.mjs` is a
guest-root instrument: it loads the frozen private-candidate Provider and
registers the frozen `daemonGovernedWorkspaceTools()` list. `node --check`
passed. Product code was not modified.

Public `cognitive status` / `doctor` after configure: all required
components `ready`, `first_conversation_ready: true`, Pi `0.81.1`
(`package_status=ready`, `pinned_version=0.81.1`,
`observed_version=0.81.1`), provider `secret_ref_resolves=true` (redacted),
`secret_material_exposed=false`. Daemon pid 273829 still bound to
`127.0.0.1:48290`. Listeners `48181` / `48284` / `48383` untouched;
`48286` / `48386` / `48388` / `48390` absent. This is readiness evidence
only — no conversation, Task, Tool, Effect, or sample until B0.

## Unique next action

Run C1/C2 paired **B0** against this freeze (`retry=0`). Do not patch
product code. Do not reuse EVAL-004/005 roots/ports or SecretStore
`/12`/`/13`/`/14`. Never `secret-tool search`/`lookup`. Rotate the
Provider key exposed earlier by EVAL-004 `secret-tool search`.

## Non-claims

This campaign creates no Gate, release, Profile, B01, or Agent-benefit
promotion. `retry=0` for Provider cells. Every started sample is retained.
Each finished cell is appended to the running report immediately.
