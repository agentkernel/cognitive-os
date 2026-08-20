# PERSONAL-PERF-EVAL-012 assessment (running)

- Campaign: `PERSONAL-PERF-EVAL-012`
- Frozen source target: `370b26fcc05976c7c1c97e5510a99ed3ebc23f2c` (P9-T08
  merged; docs-head after PR [#247](https://github.com/agentkernel/cognitive-os/pull/247))
- Lease: `lease/personal/EVAL-012/c1-c2-paired-b0` (**closed** 2026-08-20)
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: campaign **closed**. Measurement-only. Evaluation routing OFF.

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
| B0 C1 O-arm Search/Read | **pass** | 3 Search warmups + counted Search `task://personal/eval012-b0-C1-a194b2f561562663` + counted Read `…/C1-read-a194b2f561562663`; all `COMPLETED`, `lease_acquired: 1`, verification `passed`/`current`, `ACCEPTANCE_GRANTED`, `reconcile_class: closed`. Seed `sha256:a194b2f561562663`. Fixture `note.txt` digest `4fb26b79…` |
| B0 C1 P-arm / broker `48400` | **pass** | broker pid 329483 on `127.0.0.1:48400`; SecretStore paths `["24"]`; `secret_material_written: false`; Pi placeholder token only. 3 Search warmups + counted Search + counted Read on seed `sha256:a194b2f561562663`; fixture Search hit `failing-line`; Read returned both note lines. No daemon Task. `retry=0` |
| B0 C2a O-arm | **partial** | 3 Write warmups + counted Write **pass**. Counted Patch **fail** retained (`fixed post-state is unavailable`; file unchanged). `retry=0` |
| B0 C2a P-arm | **pass** | fixture adapter Write warmups + counted Write `c2a-write\\n`; counted Patch replacement bytes → `c2a-patch-v2\\n`. Format is replacement bytes, not O-arm unified diff |
| B0 C2b O-arm | **partial** | public unsealed `POST /management/resource/v1/memory/remember` **201** `remembered` (memory_id present). Session-2 resume `not-run` (would restart campaign daemon). Skill bind `not-run` (no frozen Skill package on this EVAL) |
| B0 C2b P-arm | **pass** | WorkspaceRead of frozen `procedure.txt`; P does not use daemon Memory/Skill |
| B0 C2c O-arm | `not-run` | no frozen campaign-authorized default-off fault profile / original-key injector on this EVAL; do not invent faults on `B01-Desktop-Linux-002` |
| B0 C2c P-arm | **pass** | fixture Write of frozen `original-key.txt`; digest `f88e7a35…` matches freeze ledger |
| B0 C2d O-arm | **pass** (split-score) | public evidence of counted C2a Write: `COMPLETED` / verification passed/current / `ACCEPTANCE_GRANTED` / closed. Pure-Pi completion is not OS Task completion |
| B0 C2d P-arm | **pass** (split-score) | Pi stdout exactly `ANSWER: repaired` (mechanical oracle) |
| B0 C1 fairness (13 axes) | **fail** | `system_task_prompt_bytes` mismatch (P short instruction vs O CognitiveOS Extension session). 12/13 axes pass. `b0: false` |
| B0 qualification (package 15) | **fail** (retained) | fairness fail blocks B1; C2a O Patch fail retained; C2c O `not-run`. Secret-shaped scans 0 |
| B1/B2 C1/C2 paired | `not-run` | blocked by B0 fairness fail |
| Owner expansion to full execution plan | **pass** | remainder in scope; missing runner = `not-run` |
| Plan §9 independent reviewer | `not-run` | no designated reviewer |
| C0 G/A paired (G1–G9, A1/A4/A5) | `not-run` | no frozen C0 corpus/oracle/live paired runner on this pin |
| MS-AUTH / T-GOV / T3 / T4–T9 / S4/S8 | `not-run` | missing governed Agent consumer or production caller |
| OS O1–O14 extras | `not-run` / `not_available` | O1 used by C1 O-arm; O2/O3 `not_available` |
| B3 faults / B4 concurrency / B5 soak | `not-run` | B0 did not pass; no EVAL-012 soak/fault runner frozen |
| B6 replay | `not-run` | later-only after an optimization freeze |
| Cleanup | **pass** | daemon `48300` pid 326605 stopped; broker `48400` stopped (25 forwards); SecretStore paths `item_count_unlocked=0` `item_suffixes=[]`; leftover `48181`/`48284`/`48383` untouched |

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

## B0 C1 O-arm (2026-08-20) — pass; retained

Frozen C1 fixture `note.txt` SHA-256
`4fb26b79e8de937c59f203f9274d76998db1f063ae0de442fdbceedb6d74869b` was copied
into the campaign workspace. Public admit used UuidV7 budget/loop ids.
`retry=0`. Daemon pid 326605 on `127.0.0.1:48300`. Bootstrap value was not
printed. Secret-shaped scan of Pi launch stdout/stderr: 0 hits.

| Role | Task ref | O4 `lease_acquired` | Lifecycle | Verification | Acceptance |
|---|---|---:|---|---|---|
| warmup 1 (non-counted) | `task://personal/eval012-b0-C1-warmup-1` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |
| warmup 2 (non-counted) | `task://personal/eval012-b0-C1-warmup-2` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |
| warmup 3 (non-counted) | `task://personal/eval012-b0-C1-warmup-3` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |
| counted Search | `task://personal/eval012-b0-C1-a194b2f561562663` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |
| counted Read (same seed) | `task://personal/eval012-b0-C1-read-a194b2f561562663` | 1 | `COMPLETED` / `ACCEPTANCE_GRANTED` | `passed` / `current` | `current` |

Seed `sha256:a194b2f561562663` (`c1-c2-b0-qualification-v1|C1|0`). Each Task
has Intent and Effect refs and `reconcile_class: closed`. Public
`cognitive pi launch --print --task-ref` queued daemon-governed
WorkspaceSearch (`query=failing-line`, `target=workspace://`) or
WorkspaceRead (`target=workspace://note.txt`). No Pi-native bash/edit/write.

Counted B0 envelope: this is **one** C1 O-arm qualification (class × arm);
Read is the second required tool on the same seed, not a second class.
Warmups are non-counted.

Scheduler `daemon.log` also recorded epoch-1 skips on the **private**
candidate path (`candidate descriptor is unavailable or unsafe` on Search
tasks; `private Pi candidate adapter rejected the request (exit code 3)` on
Read). The public Extension path still completed. Those skips are retained
observations, not a product fix mid-campaign.

This is not C1 paired fairness yet, not B0 pass, and not Agent-benefit.

## B0 C1 P-arm (2026-08-20) — pass; retained

Loopback broker `127.0.0.1:48400` started from frozen `pure-pi-broker.mjs` +
`createLinuxSecretServiceGet(PRODUCT_PROVIDER_ATTRIBUTES)`. Bind facts:
`secret_material_written: false`, `pi_token=campaign-broker-nonsecret-token`,
D-Bus paths-only suffixes `["24"]`. Health `ok` after bind. Pi 0.81.1 was
launched **without** `cognitive pi launch` (no daemon, no Task) from an
isolated `HOME` under the campaign root, `--extension` to a campaign-local
fixture adapter, `--tools WorkspaceRead,WorkspaceSearch,WorkspaceWrite,WorkspacePatch`,
`--no-builtin-tools`. Fixture `note.txt` digest `4fb26b79…`. `retry=0`.
Secret-shaped scan of Pi stdout/stderr: 0 hits.

| Role | Label | Result |
|---|---|---|
| warmup 1 (non-counted) | `b0-c1-parm-warmup-1` | Search hit line 1 `alpha note: find-the-failing-line` |
| warmup 2 (non-counted) | `b0-c1-parm-warmup-2` | same Search hit |
| warmup 3 (non-counted) | `b0-c1-parm-warmup-3` | same Search hit |
| counted Search | `b0-c1-parm-counted` | same Search hit; seed `sha256:a194b2f561562663` |
| counted Read (same seed) | `b0-c1-parm-read-counted` | both fixture lines (`failing-line`, `keep-me`) |

Counted B0 envelope: this is **one** C1 P-arm qualification (class × arm).
This is not OS Task completion and not Agent-benefit.

## B0 C2a O-arm (2026-08-20) — partial; retained

Public admit used UuidV7 budget/loop ids. `retry=0`. Daemon pid 326605.
Counted seed `sha256:d83d0fbb8609a880`. Write files used campaign-unique
names under this root (not P2-T37 paths). Secret-shaped scan of Pi
stdout/stderr: 0 hits.

| Role | Task ref | O4 `lease_acquired` | Result |
|---|---|---:|---|
| warmup 1 Write | `task://personal/eval012-b0-C2a-warmup-1` | 1 | `COMPLETED` / verification passed/current / `ACCEPTANCE_GRANTED` / closed |
| warmup 2 Write | `…/C2a-warmup-2` | 1 | same |
| warmup 3 Write | `…/C2a-warmup-3` | 1 | same |
| counted Write | `task://personal/eval012-b0-C2a-d83d0fbb8609a880` | 1 | same; workspace `c2a-write.txt` bytes `c2a-write\\n` |
| counted Patch | `task://personal/eval012-b0-C2a-patch-d83d0fbb8609a880` | 1 | **fail** retained: `ACTIVE` / `must_reconcile`; daemon `fixed post-state is unavailable`; file unchanged `c2a-patch-v1\\n` `cb4ff53f…` |

Private-candidate epoch-1 skips were observed again (adapter exit 3 /
descriptor unsafe). Public Write still completed. Patch did not. This is
not a product fix mid-campaign.

## B0 C2a P-arm (2026-08-20) — pass; retained

Isolated fixture root `p-arm-fixtures/c2a`. Same placeholder-token broker.
Write warmups and counted Write produced `c2a-write\\n`. Counted Patch used
**replacement bytes** (`YzJhLXBhdGNoLXYyCg==`) with preimage `cb4ff53f…`;
file became `c2a-patch-v2\\n`. This is not comparable to O-arm Patch
(unified diff / `fixed post-state is unavailable`). Split on mutation
format is retained.

## B0 C2b (2026-08-20) — split-score; retained

O-arm: public unsealed remember **201** `remembered`. Session-2 resume and
Skill bind remain `not-run`. P-arm: WorkspaceRead of freeze
`procedure.txt`. P has no daemon Memory/Skill.

## B0 C2c (2026-08-20) — split-score / O `not-run`

O-arm original-key fault reconcile is `not-run` (no frozen default-off
fault profile on this EVAL). P-arm fixture Write reproduced
`original-key.txt` digest `f88e7a35a799f332ec60ac2ba31a714904bafb1ad314c5721d0b7beda25be9b4`.

## B0 C2d (2026-08-20) — split-score; retained

O-arm: counted C2a Write public evidence `COMPLETED` / `ACCEPTANCE_GRANTED`.
P-arm: mechanical oracle stdout `ANSWER: repaired`. Pure-Pi text is not OS
Task completion.

## B0 C1 fairness (2026-08-20) — fail; retained

Frozen `fairness-checker.mjs` on 13 §2.3 axes: **fail**, `failed_axes: 1`,
`b0: false`. Only `system_task_prompt_bytes` mismatched (P-arm campaign
short instruction vs O-arm CognitiveOS Extension session). Tool schemas,
workspace digest `4fb26b79…`, Pi `0.81.1`/SRI, model
`deepseek-v4-flash`, `retry=0`, and the other listed axes passed.

## B0 qualification — fail (retained); B1/B2 not opened

A fairness fail blocks B1. C2a O Patch fail is retained (`retry=0`).
C2c O is `not-run`. Secret-shaped scans of Pi stdout/stderr remained 0.
This is not Gate, release, Profile, B01, or Agent-benefit.

## Owner expansion remainder (plan §10)

| Batch / class | Status | Required result / disposition |
|---|---|---|
| B0 qualification | **fail** (retained) | fairness fail; Patch O fail; C2c O not-run |
| B1 pilot C0 | `not-run` | no frozen C0 corpus; B0 fail |
| B2 C0 G/A paired | `not-run` | no frozen C0 runner/oracle |
| B2 C1 read-only | `not-run` | B0 fairness fail blocks confirmatory |
| B2 C2 mutation | `not-run` | B0 C2a O Patch fail + fairness fail |
| B2 C2 Memory/Skill | `not-run` | split-score; B0 did not pass |
| MS-AUTH | `not-run` | no frozen MS-AUTH runner on this pin |
| S4/S8 | `not-run` | missing governed Agent consumer |
| T-GOV / T3 / T4–T9 | `not-run` | missing production caller |
| O1 | **pass** (via C1/C2a O) | public admission/evidence |
| O2/O3 | `not_available` | no public internal observation surface |
| O4–O6 | **partial** | O4 `lease_acquired` observed on completed Tasks; Patch O stuck ACTIVE |
| O10–O14 | `not-run` / covered by B0 secret counters | no extra cell |
| B3 / B4 / B5 1h/8h | `not-run` | B0 fail; no EVAL-012 soak runner |
| B5 24h | `not-run` | conditional; trigger not met |
| B6 | `not-run` | later-only |

## Capability matrix (hypothesis / not_reviewed)

- C1 Search/Read: both arms ran on this freeze. Not a paired latency claim
  (fairness fail on system prompt).
- C2a Write: both arms ran. C2a Patch: P fixture pass, O daemon fail.
- C2b/C2c/C2d: split-score as frozen overlay required.
- C0 G/A, B3–B5, T6–T9, S4/S8: `not-run` on this pin.

## Optimization priorities (evidence-ranked, non-claim)

1. Public WorkspacePatch post-state / verification so O-arm Patch can
   complete (retained fail: `fixed post-state is unavailable`).
2. Align P/O system-prompt bytes before any paired C1 claim (only failed
   fairness axis).
3. Decide Patch payload format (unified diff vs replacement bytes) before
   treating C2a as comparable.
4. Private-candidate epoch-1 skips remain on O-arm; public path still
   completed for Read/Search/Write.
5. Do not optimize daemon vs Provider latency from this EVAL (no matched
   fair pairs).

## Non-claims

No Gate, release, Profile, B01, or Agent-benefit promotion. Independent
reviewer `not_reviewed`. Closed EVAL-002 and EVAL-004–011 were not resumed.

## Cleanup (2026-08-20) — pass

Public `cognitive daemon stop` on the campaign runtime: `action: stopped`,
pid 326605, `stale_lock_removed: true`; `127.0.0.1:48300` closed. Broker
process stopped; `127.0.0.1:48400` closed after **25** placeholder-token
forwards. `secret-tool clear` on product attributes
`application=cognitiveos-personal provider=deepseek purpose=provider-api-key`
only. D-Bus SearchItems paths-only: `item_count_unlocked=0`,
`item_suffixes=[]`. No `secret-tool search`/`lookup`. Listeners `48181` /
`48284` / `48383` and closed EVAL roots were left untouched. Owner
plaintext key file was not read or deleted.

## Unique next action

None for this campaign. Evaluation routing OFF. Do not claim a `P*-T*`
task until the owner gives a fresh delivery instruction. Claim ceiling
`hypothesis`; `not_reviewed`. No Gate, release, Profile, B01, or
Agent-benefit promotion.
