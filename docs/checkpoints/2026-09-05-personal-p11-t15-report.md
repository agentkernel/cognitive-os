# P11-T15 N=15 Windows OPC acceptance — running report

- Task: `P11-T15` / slice `P11-T15/D01`
- Change class: `implementation-only` (fixed-denominator acceptance on the qualified Windows host; no `core/specs`, no Lane-CTR, no product CSS/IA change, no signed installer)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P11-T15/windows-opc-acceptance`
- Branch: `personal/P11-T15-windows-opc-acceptance`
- Exact qualified Windows revision: `main@4ca9b046` (merged PR [#324](https://github.com/agentkernel/cognitive-os/pull/324) `P13-T13`)
- Env ID: `DEV-WINDOWS-NATIVE-OPC-01` (Windows 10 Pro `10.0.19045`, x86_64; same physical host as `DEV-WIN-GNU-01`)
- Claim ceiling: `hypothesis`. Not Gate / release / Profile / B01-W / signed installer / prototype-completeness mutex.
- Evaluation routing: **OFF**
- Denominator: **frozen** at claim from [plan.md](../plan/plan.md) `P11-T15` card scenes 1–15. N=15 is not replaceable.
- Draft PR: [#325](https://github.com/agentkernel/cognitive-os/pull/325)
- Task HEAD: `85e2821e` (claim) + cell-2 report follow-up

## Unique next action

Continue `P11-T15/D01`: retain every started cell; required CI on the to-merge HEAD of Draft PR [#325](https://github.com/agentkernel/cognitive-os/pull/325). Do not auto-release. Cells 3–15 remain honest `not-run` (missing live Provider / Pi / sandbox / sleep / host FS).

## Frozen N=15 preregistration (claim 2026-09-05)

Oracle = the scene text in plan.md T15. Record format = this table (`pass` / `fail` / `partial` / `not-run`). a11y / visual qualification stays on `P13-T12` and does not occupy this denominator.

| # | Frozen scene | Oracle (pass only if all bullets hold on this host at `4ca9b046`) |
|---|---|---|
| 1 | inspectable install → Personal Home `app/`/`data/` → daemon/tray 就位 | Unsigned inspectable bootstrap actually runs; daemon admits a Windows `Personal Home`; tray process proves work |
| 2 | first `/ui/`: empty Home only-create, rail hidden, no fake buttons | Product-origin daemon `/ui/` Dual Track chrome (not a fixture file) |
| 3 | Settings Model Connections completes a real Provider connection; raw secret only via SecretStore | Live `connection.connect` through Credential Manager; no raw secret in UI/log |
| 4 | five-step wizard ①→⑤ with real assistant research/propose, confirm-before-activate, G1/G2 | Windows Pi route + live Project wizard |
| 5 | Project four submenus: read-only axis; select-then-configure; join = Intent | Live Project on this host |
| 6 | one Member hidden hosted DSH real Attempt producing artifacts | Windows sandbox/stdio child; Linux Path B does not transfer |
| 7 | independent verify → `outputs` openable → last-ring acceptance | Live CAS artifacts + host file-open |
| 8 | Routine armed from ③; manual trigger via Intent; `runs` shows occurrence + Attempt history | Live G2 Project + daemon tick on this host |
| 9 | close-window background-or-pause → OS sleep/offline → wake missed/resume, no overlap | Real OS sleep/restart of this daily machine |
| 10 | HITL: chat announces only; Today deep-link; Confirm/Narrow/Reject/Stop; stale/unknown cannot confirm | Live pending preview on product `/ui/` chrome |
| 11 | external-send AUTONOMY packet: canvas preview → confirm → receipt; planned ≠ published | Qualified connector; packet never silently published |
| 12 | Project group chat `@manager` / `@member`; speech rules; `@` only into draft; no Approve | Live Project chat on this host |
| 13 | Knowledge ingest → Why this fragment; Memory inspect/correct/forget without resurrection | Host FS/privacy E2E |
| 14 | Skill/MCP: review → Owner canvas preview → version-locked grant → rollback | Supply-chain host E2E |
| 15 | copy inactive → archive stops triggers → delete preview + second confirm → restore point / export excludes secrets | Windows FS E2E |

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-05 | `P13-T13` merged | recorded | GitHub | `main@4ca9b046` | PR [#324](https://github.com/agentkernel/cognitive-os/pull/324). Closure HEAD `65f6244a` required CI [33961252591](https://github.com/agentkernel/cognitive-os/actions/runs/33961252591) **SUCCESS** (resolve 2s; ubuntu 4m31s; windows 15m56s; required-ci 2s). Implementation `bcd8746b` required CI [33960301310](https://github.com/agentkernel/cognitive-os/actions/runs/33960301310) **SUCCESS**. D01-qualified this host. |
| 2026-09-05 | Lease claim `lease/personal/P11-T15/windows-opc-acceptance` | recorded | `DEV-WINDOWS-NATIVE-OPC-01` | `4ca9b046` | REFRAME product-doc lease retained; writable paths do not overlap. N=15 frozen above. |
| 2026-09-05 | Unrendered `install.ps1` re-run | **pass** (fail-closed) | `DEV-WINDOWS-NATIVE-OPC-01` | `4ca9b046` | system PowerShell exit **64**; stderr `release policy is not rendered`. Not a release installer. |
| 2026-09-05 | `kernel-server --test p13_t13_windows_native_host` | **pass 2/2** | `DEV-WINDOWS-NATIVE-OPC-01` | `4ca9b046` | Live unsigned daemon admits Windows `Personal Home`; GNU 422; task 403; `daemon.bind`; `/ui/` 503 without bundle / 200 with fixture `data/cognitiveos/ui/index.html`. |
| 2026-09-05 | Draft PR [#325](https://github.com/agentkernel/cognitive-os/pull/325) | recorded | GitHub | `85e2821e` | Claim/status + handbook routing. Required CI started (resolve **SUCCESS**; ubuntu/windows in progress). |
| 2026-09-05 | Dual Track empty Home dump-dom | **pass** | `DEV-WINDOWS-NATIVE-OPC-01` | `4ca9b046` product + local `clients/pc/web` dist | Disposable `kernel-server --personal` `:48791`; dist copied to runtime `data/cognitiveos/ui`; host Chrome headless CDP; session gate from file (never printed). `[data-page=opc-today]`; Start create → `#/projects/new`; empty-home copy; rail hidden; L1 Today/Projects/Knowledge; Team=0; Inbox=0; 0 fake Create/Activate/Approve. Honesty copy contains “Vite is not the product origin” (not a Vite preview). |

## N=15 cell grid (started; retain every cell)

| # | Result | Why (this host, `4ca9b046`) |
|---|---|---|
| 1 | **partial** | Unsigned bootstrap **pass** (exit 64) + live daemon admit + Personal Home **pass**. Tray **not-run** (`tray_proves_work=false`; no tray binary). Scene requires tray 就位, so the cell is not pass. |
| 2 | **pass** | Product-origin daemon `/ui/` Dual Track empty Home on this host (Chrome headless CDP). Only-create + rail hidden + no fake buttons + no Team/Inbox L1. Fixture `/ui/` is not this cell. |
| 3 | **not-run** | Live Settings `connection.connect` / real Provider not invoked (no new SecretStore Provider entry). Credential Manager backend **pass 7/7** at this revision is not this scene. |
| 4 | **not-run** | Windows Pi route remains unqualified; five-step wizard with real `assistant.turn` not run on this host. |
| 5 | **not-run** | No live Project on a disposable T15 runtime; four-submenu native chrome not exercised. |
| 6 | **not-run** | Windows native sandbox channels remain `Unsupported`. Linux Path B does not transfer. |
| 7 | **not-run** | No live Attempt CAS artifacts on this host; host file-open (`ShellExecute`) not-run. |
| 8 | **not-run** | No G2-accepted live Project + Routine tick on this host. Clock E2E not-run. |
| 9 | **not-run** | Daily machine not put through OS sleep/restart. Typed `host.offline.record(sleep)` is not OS sleep. |
| 10 | **not-run** | No live pending ApprovalPreview on product `/ui/` chrome this session. |
| 11 | **not-run** | No connector qualified; external-send never leaves the machine. Packet chrome not exercised here. |
| 12 | **not-run** | No live Project group chat on this host. |
| 13 | **not-run** | Host FS / privacy E2E not-run. |
| 14 | **not-run** | Supply-chain host E2E not-run (no rendered/signed Windows bundle). |
| 15 | **not-run** | Windows FS copy/archive/delete/restore/export E2E not-run. |

Started cells: **15/15**. `not-run` is never pass.

## Independent review (second pass, same session)

Re-read the frozen 15 oracles against the cell grid without changing any judgement except cell 2, which was executed after the first pass:

- Cell 1 stays **partial** (tray missing). Not upgraded to pass.
- Cell 2 is **pass** on Dual Track empty Home (this host, product `/ui/`). Not rewritten from Linux T12 dump-dom.
- Cells 3–15 stay **not-run**. None rewritten as pass. Linux/CI/WSL/GNU evidence was not used as a Windows native pass.
- SecretStore 7/7 is bound to cell 3's *backend* note only; the scene remains **not-run**.
- Zero critical A1–A8 observed in this session (no secret in git/chat/argv/evidence; daemon-only admit path; bootstrap file deleted after dump-dom).
- Denominator still 15. Not Gate / release / Profile.

Independent review **pass** as a non-claim integrity check of the grid. Product scenes that remain `not-run` are capability gaps, not T15 failures.

## Non-claims

- T15 is not release, Gate, Profile, B01-W, signed installer, or prototype-completeness.
- Ordinary CI / Linux / WSL / GNU cannot promote a T15 cell.
- Local MSVC cargo is development evidence bound to this qualified host; it is not Gate evidence.
- Denominator stays 15; no scene substituted.
