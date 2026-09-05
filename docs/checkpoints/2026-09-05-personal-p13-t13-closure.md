# P13-T13 closure

- Task: `P13-T13`
- Status: **done** (acceptance mapped; required CI on to-merge HEAD)
- Draft PR: [#324](https://github.com/agentkernel/cognitive-os/pull/324)
- To-merge HEAD: `bcd8746b` (`personal/P13-T13-windows-host`)
- Required CI: [33960301310](https://github.com/agentkernel/cognitive-os/actions/runs/33960301310) **SUCCESS** (resolve 3s; ubuntu 6m28s; windows 11m43s; required-ci 3s)
- Env: `DEV-WINDOWS-NATIVE-OPC-01` D01-qualified (Windows 10 Pro `10.0.19045`, x86_64; same physical host as `DEV-WIN-GNU-01`)
- Claim ceiling: `hypothesis`. Not Gate / release / Profile / B01-W / T15 / signed installer.
- Evaluation routing: **OFF**

## Acceptance mapping

| Exit | Evidence |
|---|---|
| D01: unsigned path actually ran; image/tools/pins written | Unrendered `personal/deploy/windows/install.ps1` system PowerShell exit **64**; live cargo-built `kernel-server --personal` admits disposable `Personal Home`, GNU 422, task 403, `daemon.bind`, `/ui/` 503 without bundle / 200 with fixture. Test `p13_t13_windows_native_host` **2/2** on this host and on `CI-WINDOWS-MSVC-01`. Pins in `PERSONAL-TEST-ENVIRONMENTS.md` §2 / §5.3. |
| D02: hung native E2E accounted pass/fail/`not-run` | [report](2026-09-05-personal-p13-t13-report.md) cell grid. T02 install **pass**; T02 SecretStore **pass** (Credential Manager 7/7 synthetic); remaining hung cells honest **not-run** (tray, OS sleep/restart, sandbox Unsupported, OS ACL, supply chain, Settings `connection.connect`, host file-open, product-chrome UI, live X). |
| Negatives | CI/GNU/WSL/Linux not cited as native qualification; `not-run` never written as pass; cargo not cited as install/tray/sleep E2E; B01-W not used as daily machine; no signing/release claim; `B01-Desktop-Linux-002` not used as Windows. |
| Merge validation | `CI-WINDOWS-MSVC-01` compile/test **pass** on `bcd8746b`; local MSVC results remain development evidence only. |

## Unique next

Claim `P11-T15` (N=15 Windows OPC acceptance) on this D01-qualified host. Do not auto-release.
