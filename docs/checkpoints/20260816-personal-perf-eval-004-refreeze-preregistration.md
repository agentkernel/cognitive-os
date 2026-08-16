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
| Equivalent fixture/oracle/runner | **pass** for C0; C1/C2 workspace bytes frozen, paired adapter `not-run` | C0 corpus `sha256:38e282d4e3ceba0d62768073cf64e27a0e910832ad2ef4bfcca3f2460c919ab1`; runner `sha256:b6f1946b922054850a854ef29785943b18e19eedadc1e0053305fafd45b7b106`; analyze `sha256:6575f912a21c9b3563c883682cddc26d1facac7054ea92d408e79aa0d991906b`. C1/C2 agent-visible/hidden/repaired trees digest-pinned under `fixtures/c1-c2/`; equivalent Pi Workspace* adapter remains `not-run` |
| Redactor/sampler/cleanup digests | **pass** (scanner freeze) | `redactor.py` `sha256:665ae17713c6816b20b871778daca47dff0e9e0c9648e9ef102a30861dec6010`; evidence/runtime/arm homes/fixtures 0 key-shaped hits. Sampler not-run until B4 |
| Independent reviewer | `not_reviewed` | Parent plan §3.2 and EVAL-002 execute B1/B2 at claim ceiling `hypothesis` with `verifier=not_reviewed`. Readiness-closure-plan `approved-for-B1` is a product-train go condition, not a measurement mutex after B0 C0 pass. Independent review remains required before any claim promotion |
| B0 C0 qualification samples | **pass** | 3 discarded warmups + 9/9 retained family samples; 7/9 oracle both arms; G6/G9 both-fail; retry=0; redactor 0 hits. Not a performance claim |
| B1 C0 pilot | **pass** | 90 paired blocks, 180/180 retained; see § B1 below. Hypothesis/non-claim only |

B0 C0 qualification and B1 C0 pilot are recorded below. C1/C2 paired B0/B1 remain `not-run` (no schema-equivalent Pi adapter). No Gate, release, Profile, B01, or Agent-benefit claim is created by this preregistration.

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
`not-run`/`not_available` until a later B0 asset freeze.

Redactor over `evidence/`, `runtime/config/`, and both arm homes:
`key_shaped_hits=0`.

## B0 C0 warmups (2026-08-16) — discarded, not in denominator

Three non-counted G1 pilot blocks (`start-index` 100, 3 replicas), 6 started
runs, 6 retained, all `completed` with oracle pass on both arms. Evidence
`sha256:6dd5ffc61b343c42433601b5196bf029af462ab7dd232015aeb30e1f68490cea`.
These runs are excluded from the qualification denominator.

## B0 C0 qualification (2026-08-16) — pass (path/fairness; not a performance claim)

One pilot seed per C0 family (`start-index` 0, 1 replica, seed `20260816`
in-block arm shuffle), `retry=0`, 180 s timeout. Evidence
`sha256:2c72ef63ae0a83189dbd20c3fbd485e77a205af570172909bb9ac113b0d79d58`.
9/9 blocks started and retained. Every arm `completed` (0 timeout, 0
process_error). Broker: 9 qualification P-arm forwards, all `upstream_ok`
(campaign total including warmups: accepted 12 / rejected 0 / upstream_ok 12
/ upstream_err 0).

| Family | Arm order | P oracle | O oracle | P wall ms | O wall ms |
|---|---|---|---|---:|---:|
| A5 | P,O | pass | pass | 10172 | 10666 |
| G2 | O,P | pass | pass | 3351 | 4823 |
| G6 | P,O | fail | fail | 12176 | 8483 |
| G1 | P,O | pass | pass | 3304 | 4842 |
| G3 | P,O | pass | pass | 7481 | 8973 |
| G4 | P,O | pass | pass | 3482 | 5416 |
| A1 | P,O | pass | pass | 3576 | 4967 |
| G9 | O,P | fail | fail | 4433 | 10968 |
| A4 | O,P | pass | pass | 4561 | 4941 |

Oracle completion 7/9 on each arm; G6 and G9 failed **both** arms (task
hardness, not an arm-specific instrument defect). This qualifies the C0
paired path. It is not a B1/B2 result and creates no Agent-benefit claim.

Redactor over `evidence/` after these cells: 4 files, `key_shaped_hits=0`.
Broker metrics JSONL: 0 `sk-` hits. Listeners `48181`/`48284`/`48383`
untouched. Daemon pid 199172 still live. Independent reviewer
`not_reviewed`.

C1/C2 B0 paired samples were not started at B0 time. Workspace fixture bytes
were frozen after B0 (see § C1/C2 fixture freeze); the equivalent Pi adapter
remains `not-run`.

## C1/C2 workspace fixture freeze (2026-08-16) — bytes frozen; paired adapter not-run

Campaign-only trees under
`/home/hal9001/perfeval004-20260816/fixtures/c1-c2/` (mode `0700`). Bytes are
the frozen `1e71344a` `registered_check` corpus constants (broken `left - right`
agent-visible sources, public tests, repaired oracles) plus a secret-free C1
read-only incident note. Hidden oracles are outside agent-visible trees.
`reset.py` restores golden agent-visible bytes.

| Tree | SHA-256 |
|---|---|
| agent-visible (and golden copy) | `75f527a4695be3735c9824bdc6ef5bf354c6493c5d23fd364e49cc64687da749` |
| hidden-oracle | `4ebf6c627ad9c92f714ce067cf45fa25f324e96b6f6d21b3692217fb190d39d1` |
| repaired-oracle | `401597ea74fedb7673577ab75f72ba4c1b87040ad1836f2b2408bd03acf550f7` |
| `reset.py` | `817d2162fa1cf1fa9a43334691281b6e99bd8c49e2e9a767750750626bca5f84` |
| freeze script | `045a78e22aad1923a87d14954413457c713721dc7780f26489887e509931b707` |

**Equivalent Pi Workspace\* adapter: `not-run`.** The CognitiveOS Extension
blocks all Pi built-in tools and does not advertise WorkspaceRead/Search/Write/Patch
as Pi tools; the O-arm uses daemon candidate → Intent. Substituting Pi native
`bash`/`edit`/`write` would break the equivalent-tool fairness contract;
reimplementing the candidate protocol as a campaign instrument would risk a
second authority writer. C1/C2 addendum §1 therefore keeps paired C1/C2
`not-run` rather than replacing the adapter with a daemon proxy. Fixture
redactor over `fixtures/`: 19 files, `key_shaped_hits=0`.

## B1 C0 pilot (2026-08-16) — pass (hypothesis / non-claim)

Parent plan §3.2: 9 families × 5 pilot seeds × 2 replicas = **90 paired
blocks, 180 started runs, 180 retained**. Stratum `pilot` (held-out
`confirmatory` seeds cannot overlap). Seed `20260816` in-block arm shuffle.
`retry=0`, 180 s timeout. Evidence
`sha256:45835e22aaa84d54cf7fb52b7f74eb7aaa8b240bc7d46e57b62349166e5cc667`.
Every arm `completed` (0 timeout, 0 process_error). Broker after B1:
accepted 102 / rejected 0 / upstream_ok 102 / upstream_err 0 (includes B0
warmups + qualification). Listeners `48181`/`48284`/`48383` untouched.
Daemon pid 199172 still live.

| Endpoint | `P` pure Pi | `O` OS Pi |
|---|---:|---:|
| oracle completion | 77 / 90 = **85.6 %** | 80 / 90 = **88.9 %** |
| wall time median | **4352.2 ms** | **6190.8 ms** |
| wall MAD | 864.4 ms | 925.3 ms |

- paired completion difference `O − P`: **+3.3 pp**, 95 % clustered bootstrap
  CI **[−1.11, +7.78] pp** (10 000 resamples, clustered on task-seed);
- McNemar exact on discordant pairs (P-only 2, O-only 5): **p = 0.4531**;
- paired wall delta `O − P`: median **+1790.0 ms**, 95 % CI
  **[1617.4, 2015.0] ms**; relative median **+43.9 %**;
- broker local overhead: **0.4 ms** median (0.2–1.1 ms);
- Provider calls per `P` task: median 1, max 1.

Per-family completion (n=10 blocks each; descriptive only):

| Family | `P` | `O` | delta |
|---|---:|---:|---:|
| `A1` | 10/10 | 10/10 | 0.0 pp |
| `A4` | 10/10 | 10/10 | 0.0 pp |
| `A5` | 7/10 | 8/10 | +10.0 pp |
| `G1` | 10/10 | 10/10 | 0.0 pp |
| `G2` | 10/10 | 10/10 | 0.0 pp |
| `G3` | 10/10 | 10/10 | 0.0 pp |
| `G4` | 10/10 | 10/10 | 0.0 pp |
| `G6` | 6/10 | 6/10 | 0.0 pp |
| `G9` | 4/10 | 6/10 | +20.0 pp |

Seven families saturate or nearly saturate; `G6`/`G9` remain discriminating
(task hardness, both arms). Oracle failure reasons were `completed/set` and
`completed/value` only.

**Power reading for `B2`.** The completion difference is already bounded
inside ±8 pp at N = 90 with seven discordant pairs, so confirmatory size stays
at the plan floor: 9 × 30 held-out `confirmatory` seeds, **1 replica**
(EVAL-002 realized confirmatory unit; statistical unit is the task-seed
cluster). No sample-size change was made because a result looked close to
significant.

Redactor: `evidence/` 7 files 0 hits; `runtime/` 8 files 0 hits; arm homes 0;
fixtures 0. Whole-root scan 114 files / 15 `sk-` shapes: 1 G9 corpus fake
`provider.api_key` example (and its `.pyc`), 11 `kernel-server` binary, 1
`cognitive` binary, 1 Extension test JS — not live Provider material, not
evidence.

Independent reviewer remains `not_reviewed`. Claim ceiling `hypothesis`.
This is not a B2 result and creates no Agent-benefit claim.

## UJ3 daily operations (2026-08-16) — pass

Instrument: campaign-only `local_surface.py`
`sha256:99499c023074cd6b15f87d8902fdde03a2de936c22dd5d778e7519b8479bbf25`.
Bootstrap read in-process; bearer never written to evidence. Evidence
`uj3.jsonl` `sha256:0de9e8650bc9698fb8f040ddda8a56a3483ef378ed212b00bd39703f3ed66c0e`.
Plan §5.3 counted samples retained. Three CLI warmups per verb discarded
from the summary denominators.

| Operation | N | p50 | MAD | min | max | p95 | Outcomes |
|---|---:|---:|---:|---:|---:|---:|---|
| `GET /personal/health` | 200 | 0.45 ms | 0.04 ms | 0.35 ms | 0.87 ms | 0.59 ms | 200 × 200 |
| `cognitive status` (CLI) | 100 | 1769.6 ms | 84.2 ms | 1541.6 ms | 2011.2 ms | 1950.8 ms | 100 × exit 0 |
| `cognitive doctor` (CLI) | 50 | 1749.4 ms | 63.0 ms | 1532.0 ms | 2003.8 ms | n/a | 50 × exit 0 |
| `cognitive daemon status` (CLI) | 50 | 2.91 ms | 0.03 ms | 2.81 ms | 4.53 ms | n/a | 50 × exit 0 |
| six-resource `GET` × 6 families | 50 each | 0.29–0.41 ms | ≤0.02 ms | 0.27 ms | 0.74 ms | n/a | 300 × 200 |
| bounded watch × 6 families | 10 each | 0.31–0.44 ms | ≤0.03 ms | 0.28 ms | 0.49 ms | n/a | 60 × 200 |

Channel isolation holds: projection 401 unauthenticated
(`LOCAL_SESSION_UNAUTHORIZED`), 403 task-channel
(`SHELL_CHANNEL_BINDING_MISMATCH`), 200 management. `/personal/status` and
`/personal/doctor` 401 unauthenticated and 200 with management bearer.

CLI `status`/`doctor` are ~1.75 s p50 here versus EVAL-002's ~71 ms: this
revision's readiness path performs a real SecretStore resolve and a Pi
probe. In-daemon health stays sub-millisecond. Hypothesis only; not a
regression Gate.

## T-GOV Tool projection (2026-08-16) — pass

One management-channel `GET /resource/v1/projection?family=tool&version=1`
(0.70 ms). Seven catalog families, all `enabled` / `execution_ready`,
descriptor digest present:

| Tool family | execution_readiness | risk |
|---|---|---|
| `workspace_read` | execution_ready | read_only |
| `workspace_search` | execution_ready | read_only |
| `workspace_write` | execution_ready | workspace_mutation |
| `workspace_patch` | execution_ready | workspace_mutation |
| `process_check` | execution_ready | process_execution |
| `http_fetch_read_only` | execution_ready | network_read |
| `registered_check_run` | execution_ready | process_execution |

This is the BR-01..BR-08 executor-parity projection on the campaign daemon
(EVAL-002 measured 2/6 execution-ready). Dynamic enable/disable/quarantine
lifecycle was not driven in this cell. Evidence
`sha256:4f4dd0ab3692a5ed1083cc8534d63c7c71df04818019b5f5930ce8e536f1556d`.
No live ecosystem claim.

## MS-AUTH Memory/Skill authority smoke (2026-08-16) — partial

Six public-surface negatives. Five matched the EVAL-002 expected
status/code pairs; `bind` of an unknown revision is now **400
`RESOURCE_SKILL_ID_INVALID`** rather than 409 `RESOURCE_SKILL_CONFLICT`
(tighter validation). Revoke is reachable (`RESOURCE_SKILL_BINDING_ID_INVALID`)
— the EVAL-002 `skill/bind` prefix-shadow is gone at this revision.

| Negative | Status | Registered code |
|---|---:|---|
| bind unknown revision | 400 | `RESOURCE_SKILL_ID_INVALID` |
| revoke unknown binding | 400 | `RESOURCE_SKILL_BINDING_ID_INVALID` |
| malformed object id | 400 | `RESOURCE_MEMORY_ID_INVALID` |
| task channel drives management mutation | 403 | `SHELL_CHANNEL_BINDING_MISMATCH` |
| unauthenticated management mutation | 401 | `LOCAL_SESSION_UNAUTHORIZED` |
| forget without a valid id/reason | 400 | `RESOURCE_MEMORY_ID_INVALID` |

Positive remember/import: 20/20 HTTP 400. The public contract requires
sealed governed headers; this campaign instrument does not compose those
digests (`sealed-header composer not-run`). That is an instrument gap, not
a product-admission result. Evidence
`sha256:015995f7ab5a1f722b28170a7b7dbcc95fd33d2998bc55fec51939a58bb4edce`.

## B4 local concurrency (2026-08-16) — pass (local profiles only)

932 started and retained local reads, **0 non-OK**. Mixed Agent/local
Provider profiles remain `not-run` until after B2 budget accounting.

| Profile | Concurrency | N | p50 | p95 | max | rps | non-200 |
|---|---:|---:|---:|---:|---:|---:|---:|
| health | 1 | 100 | 0.39 ms | 0.54 ms | 3.11 ms | 1837 | 0 |
| health | 8 | 100 | 3.98 ms | 7.78 ms | 11.53 ms | 1658 | 0 |
| health | 16 | 100 | 4.71 ms | 12.86 ms | 15.60 ms | 1931 | 0 |
| tool projection | 1 | 100 | 0.42 ms | 0.56 ms | 1.28 ms | 1799 | 0 |
| tool projection | 8 | 100 | 1.56 ms | 5.28 ms | 7.39 ms | 3004 | 0 |
| tool projection | 16 | 100 | 5.54 ms | 14.78 ms | 22.40 ms | 1706 | 0 |
| overload 17 in-flight | 17 | 100 | 6.08 ms | 14.00 ms | 15.92 ms | 1816 | 0 |
| overload 33 connections | 33 | 132 | 6.60 ms | 14.95 ms | 22.75 ms | 1592 | 0 |
| health after overload | 1 | 100 | 0.38 ms | 0.46 ms | 1.65 ms | 2203 | 0 |

Throughput stays in the 1.6–3.0 k rps band; p50 grows with concurrency
(queueing). Recovery is immediate. Evidence
`sha256:bb025851fbb177ea4ecf2a7f152399ef65c8e4c0f31df3271c072fb56c5f6e6e`.
Redactor over `evidence/` after these cells: 19 files, `key_shaped_hits=0`.

## B2 C0 confirmatory (2026-08-16) — pass (hypothesis / non-claim)

Held-out stratum `confirmatory`, 9 families × 30 seeds × 1 replica =
**270 paired blocks, 540 started runs, 540 retained**. No overlap with B1
pilot seeds. `retry=0`, 180 s timeout. Evidence
`sha256:ed9d05313b39c64310b04a66a6eda9e16b541ce20e9e70bf1dc3e6e11cc5fc42`.
Every arm `completed` (0 timeout, 0 process_error). Broker campaign totals
after B2: accepted 373 / rejected 1 / upstream_ok 372 / upstream_err 1
(includes B0+B1). Listeners `48181`/`48284`/`48383` untouched.

| Endpoint | `P` pure Pi | `O` OS Pi |
|---|---:|---:|
| oracle completion | 247 / 270 = **91.5 %** | 241 / 270 = **89.3 %** |
| wall time median | **4172.4 ms** | **5940.2 ms** |
| wall MAD | 698.1 ms | 719.6 ms |
| wall p95 (N>=100) | 16 298.8 ms | 17 106.5 ms |

- paired completion difference `O − P`: **−2.2 pp**, 95 % clustered bootstrap
  CI **[−5.19, +0.74] pp**;
- McNemar exact, discordant pairs P-only 12 / O-only 6: **p = 0.2379**;
- paired wall delta `O − P`: median **+1695.6 ms**, 95 % CI
  **[1595.7, 1833.4] ms**; relative median **+43.9 %**;
- broker local overhead: **0.4 ms** median, p95 1.0 ms;
- Provider calls per `P` task: median 1, max 2.

Per-family completion (descriptive; none claimed after Holm):

| Family | `P` | `O` | delta |
|---|---:|---:|---:|
| `A1` | 30/30 | 29/30 | −3.3 pp |
| `A4` | 30/30 | 30/30 | 0.0 pp |
| `A5` | 28/30 | 27/30 | −3.3 pp |
| `G1` | 30/30 | 30/30 | 0.0 pp |
| `G2` | 30/30 | 30/30 | 0.0 pp |
| `G3` | 30/30 | 30/30 | 0.0 pp |
| `G4` | 30/30 | 30/30 | 0.0 pp |
| `G6` | 20/30 | 21/30 | +3.3 pp |
| `G9` | 19/30 | 14/30 | −16.7 pp |

Six families remain saturated or near-saturated; `G6`/`G9` discriminate.
CI on the completion delta contains 0. Wall overhead is the same sign and
similar magnitude as B1 (+1790 ms / +43.9 %). Redactor `evidence/` after B2:
19 files, `key_shaped_hits=0`. Independent reviewer `not_reviewed`. Not a
Gate, release, Profile, B01, or Agent-benefit claim.

## B3 fault, restart, Pi kill (2026-08-16) — partial

Instrument `b3_faults.py`
`sha256:3e3c4435452d20b62e8e0ae06e0d89173809fd80de662175a15a17f2b7af8ea6`.
Evidence `sha256:2fae28caeba0ced6a378e84879de04afebc03085ae5e0395d579c71ff809b197`.
Campaign daemon was restarted as part of this cell (new pid 238755 on
`127.0.0.1:48286`). Broker pid 201300 and old listeners `48181`/`48284`/`48383`
were not stopped.

| Sub-cell | Started | Retained | Result |
|---|---:|---:|---|
| selected-model mismatch | 10 | 10 | **10/10 `PERSONAL_PROVIDER_SELECTED_MODEL_MISMATCH`**, HTTP 400, 30.7 ms p50 (25.0–37.0 ms), zero Provider dispatch |
| daemon stop/start cleanup | 10 | 10 | stop 10/10 exit 0 (11.0 ms p50); start 10/10 exit 0 (86.0 ms p50); health refused while down 10/10; **0 orphans, 0 stale locks** |
| Pi process kill | 10 | 10 | SIGKILL at 2 s, 10/10 returncode −9; stdout discarded |
| client deadline | 0 | 0 | **`not-run`** — no frozen short-deadline runner copy in this freeze |
| broker unavailable | 0 | 0 | **`not-run`** — frozen paired runner 180 s timeout would hold 10 P-arm samples ~30 min; EVAL-002 also saw uncontrolled O-arm timeouts |
| Provider timeout / rate-limit / response-size | 0 | 0 | **`not-run`** — no controlled upstream fixture |
| stale Task/epoch / `OUTCOME_UNKNOWN` | 0 | 0 | **`not-run`** — C1/C2 paired adapter absent |

Redactor `evidence/` after B3: 21 files, `key_shaped_hits=0`.

## B5 1 h soak (2026-08-16) — pass

Instrument `b5_soak.py`
`sha256:b38d12c2abcd836278a07efd899609d0774d092f2e385ffefc22160b486efd05`.
60 one-minute blocks, each 20 health + six resource projections + one
bounded watch: **1620 started, 1620 retained, 0 non-OK**. Daemon pid 238755.
RSS 10 172 kB → 10 280 kB (**+108 kB / hour**). Elapsed 3540 s. No extra
Provider Agent tasks. Evidence
`sha256:a1b14083e76328da340c08a53396f2cdad28553b3c618b5eefbad7eabb3015fe`.
Redactor `evidence/` 25 files, `key_shaped_hits=0`. Listeners
`48181`/`48284`/`48383` untouched. Plan trigger for 8 h is met (clean 1 h).

Paired soak blocks (plan every 5 min in 1 h) were **`not-run`**: B1/B2 already
consumed the confirmatory Provider denominator; this 1 h cell is local
leak/safety only.

## Remaining matrix dispositions (2026-08-17) — recorded while B5 8 h in progress

These cells do not need the live campaign daemon (or must not share it with
the started soak). Independent reviewer remains `not_reviewed` (claim
ceiling, not a remaining-cell mutex). No product code was added.

| Cell | Disposition | Note |
|---|---|---|
| UJ1 install→first response | **`not-run`** | Plan §5.1 reuses historical B01; this freeze does not reinstall or mutate the guest image |
| UJ2 cold/warm conversation | **queued after B5 8 h** | Cold stratum would stop the soak daemon. Daemon-warm/Pi-cold is already the B1/B2 condition (fresh Pi per task). Pi-warm process reuse remains `not_available` |
| UJ3 task-bound resource watch (plan N=20) | **queued with UJ4** | UJ3 recorded health/CLI/six-resource get/watch; the plan's 20 task-bound watches were not in that instrument |
| UJ4 / O1 Task admission | **queued after B5 8 h** | Frozen `p9-t04-l4-t1-scenario-runner.mjs` extracted from the `1e71344a` archive, `sha256:8fe3f3c936f1e30b2c72202749c51ef312e4bdc887174a73890b16ad8c9246f3` (matches in-tree). 30 unique read-only Tasks; would contaminate soak RSS/watch |
| UJ6 journey register | **queued at closure** | Final coverage matrix; not a live sample |
| O2 Context decision surface | **`not_available`** | Addendum: no public redacted decision surface; SQLite/test helpers forbidden |
| O3 cache/compaction | **`not_available`** | Addendum: no public cache observation surface |
| O4 scheduler/fairness telemetry | **`not_available`** | Capability partial; no compliant public fairness/queue/fence telemetry |
| O5 Effect history | **`not-run`** | C2a paired adapter absent; no public Effect-history campaign fixture |
| O6 verifier/acceptance | **`not-run`** | C1/C2 paired adapter absent; UJ4 will record admission ≠ completion |
| O10 management lifecycle | **covered by B3** | No independent O10 denominator; B3 already retained 10/10 public stop/start cycles |
| O11 six-resource projection | **covered by UJ3** | 300 × HTTP 200 family GET; 60 × bounded watch |
| O12 SecretStore fail-closed / redaction | **covered by B0** | SecretStore `/12` import + redactor 0 key-shaped hits on evidence/runtime/arms |
| O13 bounded public replay | **`partial`** (UJ3) | Public bounded watch only; full audit chain internal |
| O14 backup/restore | **`not_available`** | Addendum: no user CLI/API restore path |
| T2 enable/disable/quarantine | **queued after B5 8 h** | P2-T25 public management routes exist at this revision (EVAL-002's fall-through is historical). Instrument staged at guest `t2_lifecycle.py` `sha256:d654a9ca572aaf23e494daf3a48878f4543e56745f236ea5fa0dfc1f802aede7` — not started |
| T3 Tool selection pilot | **`not-run`** | Plan-optional; not in confirmatory |
| T4–T5, T9 positives | **`not-run`** | C1/C2 equivalent Pi Workspace* adapter absent |
| T6/T7 positives | **`not-run`** | Addendum: outside supported scope |
| T6/T7 fail-closed negatives | **`not-run`** | No public Tool-dispatch surface independent of the C1/C2 paired adapter; starting a dispatch sample would invent a second caller |
| T8 descriptor-drift deny | **`not-run`** | Same missing public dispatch surface |
| T10 live MCP ecosystem | **`not-run`** | Fixture-only / no live ecosystem claim; T-GOV projection already recorded |
| S4/S8 Agent Skill consumption | **`not-run`** | Plan: no equivalent governed-consumer paired path |
| C1/C2 B0/B1/B2 paired | **`not-run`** | Workspace bytes frozen; equivalent Pi adapter absent |
| B4 mixed Agent/local | **`not-run`** | Already recorded under B4 local-only pass |
| B5 paired soak blocks | **`not-run`** | Already recorded; B1/B2 consumed the Provider confirmatory denominator |
| B5 24 h | **conditional; default `not-run`** | Only if 8 h has an unresolved slope **and** owner budget |
| B6 optimization replay | **`not-run`** | Not this campaign |

## Unique next action

Do **not** stop B5 8 h pid **241537**, campaign daemon `127.0.0.1:48286`, or
broker `127.0.0.1:48386`. Wait for `CELL_DONE b5-8h` in `evidence/b5-8h.log`
(480 minutes, hourly restart at minutes 60,120,…,420). Then, without
touching `48181`/`48284`/`48383` or SecretStore `/12` until cleanup: UJ4
(30 admissions) + UJ3 task-watch 20, T2 lifecycle smoke, UJ2 cold stratum
with `--bind 127.0.0.1:48286`, UJ6 register, cleanup + secret scan, final
assessment. Claim ceiling `hypothesis`.
