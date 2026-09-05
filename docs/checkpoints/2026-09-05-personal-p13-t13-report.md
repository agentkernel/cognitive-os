# P13-T13 Windows native host qualification — running report

- Task: `P13-T13` / slices `P13-T13/D01` then `P13-T13/D02`
- Change class: `implementation-only` (unsigned Windows native host qualification + hung native E2E backfill; no `core/specs`, no Lane-CTR, no signed installer, no B01-W)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P13-T13/windows-native-qualification`
- Branch: `personal/P13-T13-windows-host`
- Base: `origin/main@08d62e8d` (merged PR [#323](https://github.com/agentkernel/cognitive-os/pull/323) `DOC-LOCAL-RUNTIME-HOST`)
- Env ID: `DEV-WINDOWS-NATIVE-OPC-01` (same physical host as `DEV-WIN-GNU-01`, `D:\agent-kernel`)
- Claim ceiling: `hypothesis` (A7: local MSVC cargo is development evidence; `CI-WINDOWS-MSVC-01` compile is required for merge; neither is Gate / release / Profile / B01-W / T15)
- Evaluation routing: **OFF**

## Unique next action

Consumed. T13 is ready to merge on green required CI. Claim `P11-T15` after merge. Do not auto-release.

## Host pins recorded 2026-09-05 (`DEV-WINDOWS-NATIVE-OPC-01`)

| Item | Fact |
|---|---|
| OS | Windows 10 专业版 (Pro) `10.0.19045`, x86_64 (not a provision gate) |
| rustc | `1.97.1` (`8bab26f4f 2026-07-14`), `host: x86_64-pc-windows-msvc`, LLVM 22.1.6; directory override on `D:\agent-kernel` |
| Node | `24.15.0` |
| pnpm | `10.33.2` |
| Linker temp | `TEMP`/`TMP` = `D:\tmp\rust-link` for this session |
| Unsigned bootstrap | `personal/deploy/windows/install.ps1` via system `powershell.exe -NoProfile -NonInteractive -File` → **exit 64**, stderr `release policy is not rendered` (unrendered template; not a usable release installer) |

## Failure-first (D01)

| ID | Negative | Surface |
|---|---|---|
| N1 | CI / GNU / WSL / Linux cited as native qualification | env registry + report |
| N2 | unrendered `install.ps1` proceeds to network or temp side effects | unsigned bootstrap |
| N3 | GNU/Linux install root admitted as Windows Personal Home | `host.home.admit` |
| N4 | task channel writes host authority | `POST /task/host/v1/home.admit` → 403 |
| N5 | tray process or OS DACL claimed from SQLite policy strings | status `tray_proves_work=false`; ACL E2E stays `not-run` unless a real DACL is applied |
| N6 | cargo test cited as native install E2E (cargo is development evidence; the live daemon + bootstrap are the native path) | report wording |
| N7 | signed / B01-W / release claim | non-claims |

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-05 | `DOC-LOCAL-RUNTIME-HOST` merged | recorded | GitHub | `main@08d62e8d` | PR [#323](https://github.com/agentkernel/cognitive-os/pull/323); designation complete. Qualification is this task. |
| 2026-09-05 | Lease claim `lease/personal/P13-T13/windows-native-qualification` on `personal/P13-T13-windows-host` | recorded | `DEV-WIN-GNU-01` / designated `DEV-WINDOWS-NATIVE-OPC-01` | uncommitted | REFRAME product-doc lease retained; paths do not overlap. |
| 2026-09-05 | Unrendered `install.ps1` on this host | **pass** (fail-closed) | `DEV-WINDOWS-NATIVE-OPC-01` | working tree | system PowerShell exit **64**; no rendered Windows bundle exists; this **is** the unsigned path until B01-W has rendered artifacts. Not a release installer. |
| 2026-09-05 | Host pins (`rustc -vV`, Node, pnpm, OS) | recorded | `DEV-WINDOWS-NATIVE-OPC-01` | working tree | MSVC override present; GNU linking still forbidden outside override directories. |
| 2026-09-05 | `kernel-server --test p13_t13_windows_native_host` | **pass 2/2** | `DEV-WINDOWS-NATIVE-OPC-01` | working tree | Unrendered bootstrap fail-closed; live unsigned daemon admits Windows `Personal Home`, GNU 422, task 403, `daemon.bind`, `/ui/` 503 without bundle / 200 with `data/cognitiveos/ui/index.html`. First `/ui/` assert failed because the bundle lives under `cognitiveos/`, not `data/ui/` — test path corrected; not a fake 200. |
| 2026-09-05 | D01 pin write-back | **pass** | `DEV-WINDOWS-NATIVE-OPC-01` | working tree | Env registry §2 / §5.3 flipped designated → D01-qualified. Not Gate/release/Profile/B01-W/T15. |
| 2026-09-05 | `cognitive-runtime --test p7_t07_windows_install_surface` | **pass 10/10** | `DEV-WINDOWS-NATIVE-OPC-01` | working tree | Template + fail-closed rendered-bootstrap negatives. Development evidence. |
| 2026-09-05 | `cognitive-secret --test p7_t07_windows_credential_store` | **pass 7/7** | `DEV-WINDOWS-NATIVE-OPC-01` | working tree | Real Windows Credential Manager round-trip with synthetic non-production bytes. Not Settings `connection.connect`. Not Gate. |
| 2026-09-05 | `cognitive-runtime` sandbox unit tests matching `sandbox` | **pass** (Windows native remains `Unsupported`) | `DEV-WINDOWS-NATIVE-OPC-01` | working tree | `matrix_keeps_windows_native_unsupported_without_evidence` confirms honesty. Native containment E2E stays **not-run**. |

## D02 hung native E2E cell grid

| Cell | Result | Why |
|---|---|---|
| T02 install (unsigned) | **pass** | `install.ps1` exit 64; install-surface 10/10 |
| T02 tray | **not-run** | `tray_proves_work=false`; `tray_role=observe-and-request`; no tray binary |
| T02 OS sleep | **not-run** | daily machine not slept; typed `host.offline.record` cause `sleep` is not OS sleep |
| T02 SecretStore | **pass** | Credential Manager 7/7 synthetic |
| P13-T02 sandbox | **not-run** | Windows native channels remain `Unsupported`; Linux Path B does not transfer |
| P13-T02 OS ACL | **not-run** | `home.admit` writes SQLite policy only; no OS DACL applied |
| P13-T02 supply chain | **not-run** | no rendered/signed Windows bundle |
| P13-T05 host clock | **not-run** | Routine tick E2E on this host not re-run; Linux evidence does not transfer |
| P13-T05 OS sleep | **not-run** | daily machine not slept |
| P13-T05 OS restart | **not-run** | daily machine not restarted |
| P13-T08 SecretStore/proxy | **not-run** for `connection.connect`; SecretStore backend **pass** above | live Provider discovery not invoked (hang risk); API field `windows_secretstore_e2e` left `not-run` |
| P13-T04 host file-open | **not-run** | no ShellExecute of an exported copy; `host_file_open_e2e` stays `not-run` |
| UI native chrome E2E | **not-run** | fixture `/ui/` 200 is product-origin path, not `clients/pc/web` chrome |
| Live X (T14) | **not-run** | allowed |

## Non-claims

- Not Gate, release, Profile, B01-W, T15, or signed installer.
- Local MSVC cargo is development evidence. `CI-WINDOWS-MSVC-01` compile remains required for merge.
- GNU host linking remains forbidden.
- `not-run` is never pass.
