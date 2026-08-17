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
| B0 C1 WorkspaceSearch O-arm | `not-run` | first qualification sample; WorkspaceSearch only |
| B0 remaining C1/C2 families | `not-run` | C2a/C2b/C2c/C2d only if O-arm leaves `DRAFT` and `lease_acquired` ≥ 1 |
| B0 P-arm / broker `48394` | `not-run` | only after O-arm path is fairly measurable |
| B1 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B2 C1/C2 paired | `not-run` | B0 path/fairness incomplete |
| B3 faults | `not-run` | after B2 |
| B4 concurrency | `not-run` | after B3 |
| B5 soak | `not-run` | 1 h first; 8 h only if 1 h has no leak; 24 h default deferred |
| C0 paired (G1/G2/G3/G4/G6/G9, A1/A4/A5) | `not-run` | if broker/runner still unqualified, keep `not-run` |
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

## Non-claims

No Gate, release, Profile, B01, B01-W, or Agent-benefit promotion. No
optimization success. Never `secret-tool search`/`lookup`. Do not print
Provider keys. Do not treat P2-T32 stub pass as EVAL-007 repaired or as
C1/C2 Agent benefit.
