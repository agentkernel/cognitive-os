# PERSONAL-PERF-EVAL-008 assessment (running)

- Campaign: `PERSONAL-PERF-EVAL-008`
- Frozen source target: `fb85cfff25d8dd9fc5e3a8743ab9fdb3b3586630` (P2-T32
  public launcher; unmerged freeze)
- Lease: `lease/personal/EVAL-008/c1-c2-paired-freeze`
- Claim ceiling: `hypothesis` / non-claim
- Reviewer: `not_reviewed`
- Document status: campaign **active**. Measurement-only. Evaluation routing ON.

This is the campaign's single running report. Append each finished cell before
starting the next (`TEST-REPORT-INCREMENTAL-01`).

Owner 2026-08-18 authorized C1/C2 真机 re-measure after P2-T32 public-launcher
stub proof. EVAL-007 B0 on `main@2a8d4d2f` stayed `DRAFT` (`lease_acquired` 0,
no Pi child) because public `cognitive daemon start` sent stderr to
`/dev/null`. P2-T32 retains `daemon.log` (mode `0600`) and Unix
`process_group(0)`. Stub Workspace* tests pass; this campaign uses a real
`pi-agent-adapter`.

## Cells

| Cell | Status | Note |
|---|---|---|
| P2-T32 lease close (coordination) | **pass** | task remains in-progress pending Windows merge; not a C1/C2 pass |
| Freeze (archive/binaries/root/port) | **pass** | pin `fb85cfff`; archive 14,653,440 bytes / 1538 entries / 0 `.git/`; SHA-256 `202384ee0b125c6600764042ddc7a2142bb1502da21be642b8c328440325ced3`; daemon `127.0.0.1:48294` pid 281083; `log_path` mode `0600` |
| SecretStore import | **pass** | new item `/org/freedesktop/secrets/collection/login/17` via stdin; D-Bus `SearchItems` paths only; never search/lookup |
| Pi 0.81.1 pin | **pass** | `--extension` absolute; package/pinned/observed `0.81.1`; doctor `first_conversation_ready: true` is **not** C1/C2 |
| Exact-source `pi-agent-adapter` | **pass** | same `fb85cfff` archive; SHA-256 `816856b49674d06f025f535fe2bf5219dd9744ab899250a489538ea687aa3167`; `o-arm-candidate.mjs` `29870821…` |
| B0 C1 WorkspaceSearch O-arm | **partial** | one retained sample; admit 200; lifecycle `DRAFT`; `lease_acquired` 0; no Pi child; skip class `private_completion_socket_could_not_be_created` |
| B0 remaining C1/C2 families | `not-run` | same public skip; do not open Provider spend |
| B0 P-arm / broker `48394` | `not-run` | O-arm path not fairly measurable |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B3 faults | `not-run` | no frozen fault runner on this freeze; C1/C2 path incomplete; do not cobble |
| B4 concurrency | `not-run` | B0 path/fairness incomplete |
| B5 soak | `not-run` | no frozen soak runner; 1 h trigger not met |
| C0 paired (G1/G2/G3/G4/G6/G9, A1/A4/A5) | `not-run` | broker `48394` never started; do not cobble a paired shell |
| Cleanup | `not-run` | stop `48294`; clear campaign SecretStore; leave prior roots/ports |

## Freeze (2026-08-18) — pass

Exact source `fb85cfff25d8dd9fc5e3a8743ab9fdb3b3586630` (P2-T32 public
launcher; unmerged). Guest root mode `0700`. Listeners `48181` / `48284` /
`48383` untouched. SecretStore item `/17` is new (not `/12`–`/16`). Public
doctor: all required components `ready`, Pi `0.81.1`,
`first_conversation_ready: true`. That is conversation readiness, not a
C1/C2 Task. Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or
Agent-benefit claim.

`git archive --format=tar --prefix=cognitiveos-personal-fb85cfff/` copied
with `scp` (not SSH-pipe). Host `ldd` on `kernel-server` / `cognitive` /
`pi-agent-adapter` resolves only glibc/`libgcc`/`libm`. Windows GNU Rust
build remains `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).

Public start JSON: `action=started`, pid `281083`, endpoint
`127.0.0.1:48294`, lock `…/runtime/cognitiveos/daemon.lock`, `log_path`
`…/runtime/state/cognitiveos/daemon.log` (mode `0600`). Pre-credential
status: provider `blocked` (`provider_config_missing`), pi
`not_configured`, `first_conversation_ready: false`. After stdin import
and Pi configure, doctor is ready as recorded below.

| Asset | Value |
|---|---|
| Archive | 14,653,440 bytes; 1538 entries; 0 `.git/` members; SHA-256 `202384ee0b125c6600764042ddc7a2142bb1502da21be642b8c328440325ced3` |
| `kernel-server` | 16,534,712 bytes; SHA-256 `e603edab9a594e41177f89ac105b2755bff34cdb980c30faece03de87610ec55` |
| `cognitive` | 10,313,736 bytes; SHA-256 `6917dca3a0f294c34d1f177dd5ebd3e1a36fff1c71de7661094049b30741a65f` |
| `pi-agent-adapter` | 1,126,192 bytes; SHA-256 `816856b49674d06f025f535fe2bf5219dd9744ab899250a489538ea687aa3167` |
| Pi tarball `@earendil-works/pi-coding-agent@0.81.1` | 4,967,228 bytes; SHA-256 `420113c0282160e6181656fd16cf18742f76bf9040ee3dfb9cb67e3e6ad5641c` |
| Pi `package-lock.json` | SHA-256 `ee9402c698efd83729dde02e93ad4a6518401bee514bbe4252f7b0a184812200` |
| `pi.json` | SHA-256 `07bb1797b6a46ba2362c5933ca2135e5feb758b84efef64e9c16444bf2b44743`; absolute paths only |
| `dist/index.js` | SHA-256 `d27f97764e55b9a9b22bbf7e22e48c0ef2a017924ed13684b143b196991c1a57` |
| `dist/extension.js` | SHA-256 `d5ba4e47d2e05a260f9c5e3850572edf228628ab02c78e7acd75c98f2278d880` |
| `dist/workspace-tools.js` | SHA-256 `233d77268519992453293ea9bde463ad548db6e720c22e3478b0322301336c5a` |
| `dist/tool-policy.js` | SHA-256 `4ce7dc2f4c6f2381805ed5c0ba66d4cd1f5ccdff712d6ae9c2a845601cb2916c` |
| `o-arm-candidate.mjs` | SHA-256 `29870821488451b5728f88c4612e1616fd65681adaf23011dd898d459428e573` |
| `private_candidate_provider.mjs` | SHA-256 `2b7e52a6afe205e5997c58fe59b096fc7666dfd8733e196777e915d3a0bc245b` |

SecretStore import (stdin; D-Bus paths only): item
`/org/freedesktop/secrets/collection/login/17` (1 unlocked, 0 locked).
`login` collection `Items` contains only `/17`. Product report:
`secret_backend=linux-secret-tool`, `secret_material_written=true`,
`secret_ref_redacted=true`, `selected_model=deepseek-v4-flash`,
`snapshot_digest=fnv1a64:c58ce6f2f7521544`. Doctor after configure:
provider `secret_ref_resolves=true`, `secret_material_exposed=false`, Pi
`package_status=ready` / `pinned_version=0.81.1` /
`observed_version=0.81.1`, `first_conversation_ready: true`. Daemon pid
281083 still bound to `127.0.0.1:48294`.

`kernel-server` digest matches EVAL-007 (`2a8d4d2f`); `cognitive` CLI
digest differs, as expected for P2-T32 launcher/log/`process_group`
changes. Adapter and Extension dist match EVAL-007 because those trees
are unchanged at `fb85cfff`.

## B0 C1-search O-arm (2026-08-18) — partial; retained

One O-arm C1-search qualification Task was started with `retry=0` against
the public Task admit surface on the public `cognitive daemon start`
launcher. It is retained. It did not leave `DRAFT`. No Intent, Effect,
verification, or acceptance row exists. WorkspaceRead is still not
advertised; this cell used WorkspaceSearch only. No Provider spend: the
scheduler never spawned Pi.

| Seed | Task ref | Admit | Probe | Lifecycle |
|---|---|---|---|---|
| `b0-0` | `task://local/eval008-b0-C1-search-b0-0-5390a313b553` | 200 (record 13 ms, interpret 6 ms, preview 1 ms, admit 30 ms) | wall 179178 ms; `acceptance_ref` absent | `DRAFT`; minted `2026-08-17T17:52:02.893Z` |

Guest evidence file:
`/home/hal9001/perfeval008-20260818/evidence/b0-oarm-C1-search-b0-0.json`
SHA-256 `8ab7f84120a211058c487654ae2d55ab0d99a973b65f3975c77c40d365b72626`.
Instrument `eval008-b0-c1-search.py`
SHA-256 `c79475819d1f00c0fcbf635b0f1768bdaf03eacb9aaf9e1cf80c849bcbd89ff5`.
Public `cognitive task evidence`: `lifecycle.current_state=DRAFT`;
`intent_refs` / `effect_refs` empty; `latest_verification` /
`latest_acceptance` null. Bounded O4: `runnable_count` 32,
`lease_acquired` 0 (`observed_zero` true). O5 named zero. No
`pi-agent-adapter` or `pi-coding-agent` child. Campaign kernel-server pid
281083. Listeners `48181` / `48284` / `48383` untouched. P-arm / broker
`48394` not started.

Claim ceiling `hypothesis`. No Gate, release, Profile, B01, or Agent-benefit
claim.

## Private-candidate skip (2026-08-18) — public `daemon.log` fact

Campaign-only observation. No product change. Public `cognitive status` /
`doctor` were ready (`first_conversation_ready: true`) before B0; that is
not C1/C2. The admitted Task stayed `DRAFT` for the full 180 s probe.
O4 `lease_acquired` 0. No Pi child. Freeze assets required for a spawn
were present (`pi.json` candidate paths, adapter binary, selected model).
This is therefore **not** a missing-freeze-asset `not_available`.

P2-T32's focused test uses a stub Workspace* adapter and proved the public
launcher can leave `DRAFT` with `lease_acquired` ≥ 1. This cell used the
real `pi-agent-adapter` plus operator `pi.json`. Skip rows are now a
public fact on `runtime/state/cognitiveos/daemon.log` (mode `0600`):

`kernel-server personal scheduler tick: skip row
task://local/eval008-b0-C1-search-b0-0-5390a313b553 at epoch 1: scheduler
private Pi candidate proposal failed: private completion socket could
not be created`

Skip class: `private_completion_socket_could_not_be_created`. This is not
EVAL-007's non-public stderr `/dev/null` skip, and it is not a claim that
P2-T32 "fixed" C1/C2.

This is not a real public C1/C2 caller. Remaining paired B0 classes,
B1/B2, and P-arm stay `not-run`. Do not open Provider spend on a path
that never leaves `DRAFT`. Do not patch product code in this campaign.

## C1/C2 paired remainder and later batches (2026-08-18) — `not-run`

| Cell | Disposition | Cause |
|---|---|---|
| B0 remaining C1/C2 classes (C2a/C2b/C2c/C2d) | `not-run` | same public skip; one C1-search O-arm sample was started and is retained |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| P-arm / broker `48394` | `not-run` | equivalent Pi adapter not started; fairness not measurable |
| O5/O6 as C1/C2 dependents | `not-run` | no Intent/Effect |
| T4–T5/T8/T9 | `not-run` | no public dispatch |
| B3 faults | `not-run` | no frozen fault runner on this freeze; do not cobble |
| B4 concurrency | `not-run` | B0 path/fairness incomplete |
| B5 1 h / 8 h / 24 h | `not-run` | no frozen soak runner; 1 h trigger not met; 24 h default deferred |
| C0 paired (G1/G2/G3/G4/G6/G9, A1/A4/A5) | `not-run` | broker/runner not qualified on this freeze; do not cobble a paired shell |

## Non-claims

No Gate, release, Profile, B01, B01-W, or Agent-benefit promotion. No
optimization success. Never `secret-tool search`/`lookup`. Do not print
Provider keys. Do not treat P2-T32 stub pass as EVAL-007 repaired or as
C1/C2 Agent benefit.
