# PERSONAL-PERF-EVAL-004 re-freeze preregistration

- Campaign: `PERSONAL-PERF-EVAL-004`
- Lease: `lease/personal/EVAL-20260816/full-os-only-refreeze`
- Date: 2026-08-16
- Frozen product source: `origin/main@1e71344a7b2c4a443fd0581e7fd33f21e970efbd`
  (merge of P2-T28 / PR #227; BR-01..BR-08 are on `main`)
- Campaign registration: PR #228 merged at
  `main@f1fa00a1a9698e8059a594f047ab2d6676854e32` (docs-only; product tree unchanged)
- Target: `B01-DESKTOP-002` / `B01-Desktop-Linux-002`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted (measurement-only)

This is a **new freeze**. It does not reuse the 2026-08-15 campaign root
`/home/hal9001/perfeval004`, loopback port `48284`, SecretStore entry, broker,
runner, corpus, oracle, evidence denominator, or any prior EVAL-004 asset.
`PERSONAL-PERF-EVAL-002` remains closed and is not resumed.

## Owner authorization

Owner standing instruction: after BR-01..BR-08 merge, re-freeze EVAL-004 and
continue measurement. BR-08 closed via PR
[#227](https://github.com/agentkernel/cognitive-os/pull/227) at
`main@1e71344a7b2c4a443fd0581e7fd33f21e970efbd`.

## Isolation (must not collide with prior campaigns)

| Item | This freeze | Explicitly not reused |
|---|---|---|
| Campaign root | `/home/hal9001/perfeval004-20260816` mode `0700` | `/home/hal9001/perfeval004`, `~/perfeval002`, `~/p9t04`, `cos-current` |
| Loopback port | `127.0.0.1:48286` daemon pid 199172; broker `127.0.0.1:48386` pid 201300 | `48181`, `48282`, `48284`, residual `48383` |
| SecretStore entry | new item `/12` via product stdin (`cognitive init --api-key-file -`) | 2026-08-15 item `/11` (cleared by product `put` of the same attribute triple; old `provider.json` not copied) |
| Source archive | `sha256:a871f5d32f2cdc818a696b7908d1fce2bc4bb63ebf47a4d36185c570146be7e8` (14,407,680-byte `git archive` of `1e71344a`) | archive digest `sha256:3578b4fa…` from `93dde21` |

`B01-Clean-Linux-001` stays out of bounds. Snapshot revert/delete, P9-T04
residue, and the owner plaintext key file are not in this freeze's allowlist.

## Freeze checklist (append-only)

| Step | Status | Note |
|---|---|---|
| BR-01..BR-08 merged | **pass** | P2-T21..P2-T28 on `main@1e71344a` |
| Evaluation lease claimed | **pass** | this document + Current snapshot row |
| Product source pin | **pass** | `1e71344a7b2c4a443fd0581e7fd33f21e970efbd` |
| Source archive + SHA-256 | **pass** | `git archive --format=tar --prefix=cognitiveos-personal-1e71344a/` of `1e71344a`; 14,407,680 bytes; `sha256:a871f5d32f2cdc818a696b7908d1fce2bc4bb63ebf47a4d36185c570146be7e8` verified on Windows, `DEV-LINUX-NATIVE-01`, and the guest |
| New campaign root/port | **pass** | `/home/hal9001/perfeval004-20260816` mode `0700`; port `127.0.0.1:48286` unused. Prior root `/home/hal9001/perfeval004` and listeners `48181`/`48284` left untouched |
| Exact-source daemon/CLI binaries | **pass** | `DEV-LINUX-NATIVE-01` `cargo build --release --locked -p kernel-server -p admin-cli` from extracted archive, 1m40s, Rust 1.97.1. `kernel-server` 16,456,928 bytes `sha256:ecc1bf395d0d4368dfd4d32666cecaa2bb1bc5350f26fa2f52a7829aa1ce1e3e`; `cognitive` 10,313,952 bytes `sha256:760ad2c7f3cbd90906b15f3ccf2344e8b0fa82baefc0ee1486f24fa5aa15afe5`. Digests matched host `/home/wuz/eval004-refreeze-20260816/` and guest campaign root |
| Campaign daemon on `48286` | **pass** | public `cognitive daemon start --runtime-root …/runtime --bind 127.0.0.1:48286 --kernel-server …/kernel-server`; pid 199172; lock live; bootstrap present (value not read). Public status: system/database/secret/daemon `ready`, provider `blocked` (`provider_config_missing`), pi `not_configured`, `first_conversation_ready: false`, `authority_side_effects: false`. Listeners `48181`/`48284` still present and untouched |
| New SecretStore entry | **pass** | product stdin import into a new Secret Service item `/12` (created 2026-08-16 12:24:11 UTC). Owner file `~/下载/deepseek.txt` used by shape only; key travelled `sed -n '8p'` → pipe → `--api-key-file -`. Never argv/env/config/log/evidence/chat/Git. See § SecretStore import |
| Local Pi `0.81.1` pin | **pass** | guest-local npm pack+install under the new root; `--extension` absolute path only. Doctor: package/pinned/observed `0.81.1`, `first_conversation_ready: true`. See § Pi pin |
| Pure-Pi broker freeze | **pass** (listen/health only) | `pure-pi-broker.py` `sha256:88a0d5cd2509fe28fcebffd49ad9f3a4617f0ab963c40ec40676cef8a6caba8c` on `127.0.0.1:48386` pid 201300; key loaded once into memory; health `ok` with 0 upstream forwards. Residual `48383` pid 167900 untouched |
| Equivalent fixture/oracle/runner | **pass** for C0; C1/C2 `not-run` | C0 corpus `sha256:38e282d4e3ceba0d62768073cf64e27a0e910832ad2ef4bfcca3f2460c919ab1` (byte-identical to closed EVAL-002); runner `sha256:b6f1946b922054850a854ef29785943b18e19eedadc1e0053305fafd45b7b106` (new root/port/extension/seed `20260816`); analyze `sha256:6575f912a21c9b3563c883682cddc26d1facac7054ea92d408e79aa0d991906b`. C1/C2 equivalent workspace adapters are not frozen |
| Redactor/sampler/cleanup digests | **pass** (scanner freeze) | `redactor.py` `sha256:665ae17713c6816b20b871778daca47dff0e9e0c9648e9ef102a30861dec6010`; evidence/runtime-config/arm homes 0 key-shaped hits. Sampler not-run until B4 |
| Independent reviewer before B1 | not-run | `not_reviewed`; B0 may continue; B1 is forbidden |

No B0/B1/B2/B3/B4 **sample** has started under this freeze (no Provider qualification run, no Task/Tool/Effect). No Gate, release, Profile, B01, or Agent-benefit claim is created by this preregistration.

## Archive and guest-root pin (2026-08-16)

Host `wuz@192.168.1.2` (`DEV-LINUX-NATIVE-01` / libvirt host `hal9000`) and
guest `hal9001@192.168.123.160` (`B01-Desktop-Linux-002`, running; identity
`hal9001-Standard-PC-Q35-ICH9-2009`) were contacted only on the registered
route. `B01-Clean-Linux-001` remained shut off and was not contacted.
`virsh -c qemu:///system` was read-only.

The source archive is a clean `git archive --format=tar
--prefix=cognitiveos-personal-1e71344a/` of
`1e71344a7b2c4a443fd0581e7fd33f21e970efbd` (1525 entries, 0 `.git` members).
SHA-256 `a871f5d32f2cdc818a696b7908d1fce2bc4bb63ebf47a4d36185c570146be7e8`
matched on the Windows operator host, on `DEV-LINUX-NATIVE-01` at
`/home/wuz/eval004-refreeze-20260816/cognitiveos-personal-1e71344a.tar`, and
on the guest at
`/home/hal9001/perfeval004-20260816/cognitiveos-personal-1e71344a.tar`.
The guest root was created `0700`. Prior campaign root
`/home/hal9001/perfeval004` and pre-existing loopback listeners `127.0.0.1:48181`
and `127.0.0.1:48284` were left untouched. Port `48286` was not listening.

Windows GNU Rust build remains `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).
Host disk at pin time: 32G free (93%). Guest disk: 42G free (26%).

## Exact-source binaries (2026-08-16)

`CARGO_NET_OFFLINE=true cargo build --release --locked -p kernel-server -p
admin-cli` finished in 1m 40s on `DEV-LINUX-NATIVE-01` (Rust 1.97.1) from the
extracted `1e71344a` archive. Guest `ldd` resolves only glibc/`libgcc`/`libm`
for both binaries (no missing libraries). Files were copied into the new
campaign root only; `/home/hal9001/perfeval004` was not read or modified.

| Binary | Bytes | SHA-256 |
|---|---|---|
| `kernel-server` | 16,456,928 | `ecc1bf395d0d4368dfd4d32666cecaa2bb1bc5350f26fa2f52a7829aa1ce1e3e` |
| `cognitive` | 10,313,952 | `760ad2c7f3cbd90906b15f3ccf2344e8b0fa82baefc0ee1486f24fa5aa15afe5` |

## Campaign daemon start (2026-08-16)

Public caller:

`/home/hal9001/perfeval004-20260816/cognitive daemon start --runtime-root /home/hal9001/perfeval004-20260816/runtime --bind 127.0.0.1:48286 --kernel-server /home/hal9001/perfeval004-20260816/kernel-server`

Result: `action=started`, pid `199172`, endpoint `127.0.0.1:48286`, lock
`…/runtime/cognitiveos/daemon.lock`, `bootstrap_present=true` (value not read).
`cognitive status` reports system/database/secret/daemon `ready`, provider
`blocked` (`provider_config_missing`), pi `not_configured`,
`first_conversation_ready: false`, `authority_side_effects: false`. Guest
listeners are `48181`, `48284`, and `48286`; the first two were not stopped.

No Provider sample, Task, Tool, or Effect occurred at daemon start. SecretStore
import and Pi pin are recorded below.

## SecretStore import (2026-08-16) — pass

Operating Model §2.3 plus standing operator authorization: import the
owner-designated local test Provider key through the product stdin/hidden
path into an approved Secret Store. A **new** campaign-only item was
required; the 2026-08-15 item and any prior EVAL entry must not be reused
as this freeze's config.

Owner file `~/下载/deepseek.txt` was located by name and inspected by shape
only (line count, line lengths, character-class mask). The file is the
owner's saved `cognitive init` invocation. Non-secret lines 2–6:

- `--provider deepseek`
- `--base-url https://api.deepseek.com/v1`
- `--model-id deepseek-v4-flash`

Line 8 is 35 bytes and `sk-`-prefix shaped; its bytes were never printed.
`wc -l` reports 7 because line 8 has no trailing newline. Owner-file mtime
remained `2026-08-12 12:51:02 +0800` after import.

Pre-import Secret Service search (attributes/paths only, no `lookup` of
values) showed one item `[/11]` label `cognitiveos-personal-provider-api-key`
created/modified `2026-08-15 09:08:52`. The new runtime had no
`provider.json`. The 2026-08-15 root file
`/home/hal9001/perfeval004/runtime/config/cognitiveos/provider.json` was not
copied.

Import caller (key never in argv/env):

`sed -n '8p' /home/hal9001/下载/deepseek.txt | /home/hal9001/perfeval004-20260816/cognitive init --runtime-root /home/hal9001/perfeval004-20260816/runtime --provider deepseek --base-url https://api.deepseek.com/v1 --model-id deepseek-v4-flash --api-key-file -`

with `XDG_RUNTIME_DIR=/run/user/1000` and
`DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus`.

Product report (redacted): `status=ok`, `action=configured`,
`secret_backend=linux-secret-tool`, `secret_material_written=true`,
`secret_ref_redacted=true`, `provider_id=deepseek`,
`selected_model=deepseek-v4-flash`,
`snapshot_digest=fnv1a64:c58ce6f2f7521544`,
`profile_claim=not-claimed`, `gate_claim=not-claimed`,
`authority_side_effects=false`.

Post-import Secret Service search shows **new** item `[/12]` with the same
stable product label, created/modified `2026-08-16 12:24:11` UTC.
Product `put()` clears the attribute triple
(`application=cognitiveos-personal`, `provider=deepseek`,
`purpose=provider-api-key`) then stores; `[/11]` is therefore absent from
search. This freeze did not copy the old runtime config. The 2026-08-15
`provider.json` mtime is unchanged (`2026-08-15 17:08:54 +0800`). Old
listeners `48181`/`48284` were not stopped.

Public `cognitive status` after import: provider `ready` (77–103 ms),
secret `ready`, pi still `not_configured`, `first_conversation_ready: false`.
Public `cognitive doctor`: `secret_ref_present=true`,
`secret_ref_resolves=true`, `secret_ref_redacted=true`,
`secret_material_exposed=false`, `backend_class=native`,
`selected_model_present=true`, `selected_model_digest_matches=true`.
Daemon remained pid `199172` on `127.0.0.1:48286`.

## Pi 0.81.1 pin (2026-08-16) — pass

Guest-local, no global install, no reuse of `/home/hal9001/perfeval004`.

| Asset | Digest / version |
|---|---|
| Pi tarball `@earendil-works/pi-coding-agent@0.81.1` | 4,967,228 bytes; `sha256:420113c0282160e6181656fd16cf18742f76bf9040ee3dfb9cb67e3e6ad5641c`; npm integrity `sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==` (byte-identical to the prior EVAL pin) |
| Pi runtime `package-lock.json` | `sha256:8a6ef5b2b0ed1127785989e3b6d15af4cd4913124453aa0085c848d0cc9857f7` |
| Pi executable | `/home/hal9001/perfeval004-20260816/pi-runtime/node_modules/@earendil-works/pi-coding-agent/dist/cli.js`; `pi --version` → `0.81.1` |
| Extension dist archive (from extracted `1e71344a` `packages/pi-cognitiveos`, host `tsc`) | `sha256:6fd3426f3c06aff9c1fd67542e5828e6096b6893816604c62bb5773e326dbfb3` |
| Extension entry `dist/index.js` | `sha256:d27f97764e55b9a9b22bbf7e22e48c0ef2a017924ed13684b143b196991c1a57` |
| `pi.json` | `sha256:6c2648e1d3fa57a9bfdc5eaf66258eaacd8dfca1aede8f0ee3bf73b609c6581e` |

`pi --version` with `--extension <absolute-path>` also reports `0.81.1`.
Public caller:

`/home/hal9001/perfeval004-20260816/cognitive pi configure --runtime-root /home/hal9001/perfeval004-20260816/runtime --executable /home/hal9001/perfeval004-20260816/pi-runtime/node_modules/@earendil-works/pi-coding-agent/dist/cli.js --extension-entry /home/hal9001/perfeval004-20260816/pi-cognitiveos/dist/index.js`

`pi.json` contains only non-secret absolute paths (`schema_version` 1,
surface `personal-pi-config`). Public status after configure: all six
components `ready`, `first_conversation_ready: true`, pi probe ~1.7 s.
Doctor: `package_status=ready`, `pinned_version=0.81.1`,
`observed_version=0.81.1`. This is readiness evidence only — no
conversation, Task, Tool, Effect, or sample.

Guest facts for this freeze (read-only): identity
`hal9001-Standard-PC-Q35-ICH9-2009`, Linux `7.0.0-28-generic`, glibc
`2.39`, 2 vCPU, 4 GiB (`virsh -c qemu:///system` domain
`B01-Desktop-Linux-002` UUID `f7bb6a52-2a0b-4ecb-8e8f-f4c60ca472a0`),
guest `MemTotal` 4005432 kB / `MemAvailable` 2307300 kB, disk 42G free
(26%). `B01-Clean-Linux-001` remained shut off. CPU model
`Intel(R) Xeon(R) CPU E5-2686 v4 @ 2.30GHz`.

## Pure-Pi broker listen/health (2026-08-16) — pass as freeze, not a sample

New campaign-only instrument, Python 3 standard library, guest path
`/home/hal9001/perfeval004-20260816/pure-pi-broker.py` mode `0700`,
`sha256:88a0d5cd2509fe28fcebffd49ad9f3a4617f0ab963c40ec40676cef8a6caba8c`.
Bind `127.0.0.1:48386` (not `48383`/`48284`/`48181`/`48286`). Pid 201300.
Pi-facing token is the fixed non-secret string
`campaign-broker-nonsecret-token`. Key is read once via `secret-tool lookup`
into process memory; metrics JSONL contains the listen event only
(107 bytes, 0 `sk-` hits). `GET /health` returns `key_loaded: true` with
`accepted=0` / `upstream_ok=0` / `upstream_err=0`. No Provider forward has
been issued. Residual `127.0.0.1:48383` python pid 167900 was left
untouched.

## C0 corpus/runner/redactor freeze (2026-08-16) — pass for C0; C1/C2 not-run

Closed EVAL-002 C0 instruments were copied into the **new** root and
re-pathed; `/home/hal9001/perfeval002` is not this freeze's runtime.
`/home/hal9001/perfeval004` was not read.

| Asset | SHA-256 |
|---|---|
| `paired_corpus.py` (C0 v1, 9 families, mechanical oracle) | `38e282d4e3ceba0d62768073cf64e27a0e910832ad2ef4bfcca3f2460c919ab1` |
| `paired_runner.py` (ROOT/port/extension/seed adapted) | `b6f1946b922054850a854ef29785943b18e19eedadc1e0053305fafd45b7b106` |
| `analyze_paired.py` | `6575f912a21c9b3563c883682cddc26d1facac7054ea92d408e79aa0d991906b` |
| P-arm `models.json` (broker `127.0.0.1:48386`, placeholder token only) | `a953dad522adf97a516effb0527eb5d268777e8cecccbd58a3d7c43cf62a35d5` |
| O-arm `settings.json` (`cognitiveos` / `deepseek-v4-flash`) | `0f1f200a4f98c9d7f1edd84d729a207b6f4e5e2d253904311f3850857e44b82a` |
| `redactor.py` | `665ae17713c6816b20b871778daca47dff0e9e0c9648e9ef102a30861dec6010` |

Fairness for C0: same Pi `0.81.1` binary, same `--no-tools` policy, same
model id, `retry=0`, 180 s timeout, mechanical `ANSWER:` oracle, arm order
randomized from seed `20260816`. Declared arm differences remain broker vs
Extension/daemon proxy. C1/C2 equivalent workspace tool adapters, hidden
oracles, and reset digests are **not** frozen; those classes stay
`not-run`/`not_available` until a later B0 asset freeze. No Provider
qualification sample has started.

Redactor over `evidence/`, `runtime/config/`, and both arm homes:
`key_shaped_hits=0`.

## Unique next action

Run B0 C0 non-counted warmups (3 per arm) then one qualification sample
per C0 family with `retry=0`; retain every started sample. C1/C2 remain
`not-run` (equivalent workspace fixtures absent). Independent reviewer
remains `not_reviewed` (B1 forbidden). Do not bind or stop
`48181`/`48284`/`48383`.
