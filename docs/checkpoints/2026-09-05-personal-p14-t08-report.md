# P14-T08 Knowledge v9 IA — running report

- Task: `P14-T08` / slices `P14-T08/D01` + `P14-T08/D02`
- Change class: `implementation-only` (Knowledge `/ui/` IA; reuse P13-T07 Vault/Memory authority and P12-T07 `vault.import`; no new numbered migration; no Obsidian)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P14-T08/knowledge-ia`
- Branch: `personal/P14-T08-knowledge-ia` (worktree `D:\agent-kernel-p14-t08`; do not edit dirty `D:\agent-kernel`)
- Base: `origin/main` after DOC PR #326 (`e14bc7a7` content)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI is not Gate / release / Profile / Windows qualification)
- Evaluation routing: **OFF**
- Product origin: daemon-served `/ui/` (Vite is not the product origin)
- Host FS E2E: `not-run`

## Unique next action

D02 live `/ui/` J5 + regression pack observed. Required CI [33975024580](https://github.com/agentkernel/cognitive-os/actions/runs/33975024580) **SUCCESS** at `899b65f6` (resolve 4s; ubuntu 4m32s; windows 15m7s; required-ci 3s). Ready/merge PR #328, then this close is consumed. Do not claim T02/T03/T04 or T07/T06. T05 waits for T03.

## Wait-gate

`lease/personal/DOC-P14-GAP-CLOSE/plan-registration` is **closed** (PARALLEL-LANES §3.1). Implementation branched from the commit that contains Phase 14 cards (`e14bc7a7`). `DOC-PERSONAL-2.0-OPC-REFRAME` remains active; `personal/docs/product/knowledge-memory-vault.md` is read-only.

## Failure-first (D01)

| ID | Negative | Surface |
|---|---|---|
| N1 | File import cannot become Project authority (`is_authority` refused; no `vault.apply-authority`) | Dual Track Knowledge Import |
| N2 | Secret-shaped file/paste is not POSTed; original fields stay | Dual Track Knowledge Import |
| N3 | No Project id → no Files/Import/Why/Memory tabs (honest lock) | Dual Track Knowledge |
| N4 | 0 fake Admit buttons; chat auto-admission stays Requires-backend | Dual Track Memory tab |
| N5 | Obsidian is not bundled / not named as a product surface | Dual Track copy |

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-05 | Dual Track `knowledgeIa` + `knowledgeIngest` + `knowledgeMemory` + `opcIa` | **pass** 41/41 | `DEV-WIN-GNU-01` / Node vitest | `6223b277` | development evidence only |
| 2026-09-05 | `pnpm run check:consistency` | **pass** | local Node | `6223b277` | 275 requirements; leases + Layer 1 counts |
| 2026-09-05 | `pnpm build` (`tsc --noEmit` + vite) | **fail then pass** | `DEV-WIN-GNU-01` | worktree | first fail TS2783 FileList mock `length` spread; fixed; knowledgeIa 7/7 |
| 2026-09-05 | Host FS / live `/ui/` J5 | **not-run** | `P14-T08/D02` | — | superseded by live `/ui/` row below; host OS file-picker E2E still `not-run` |
| 2026-09-05 | Required CI `verify (ubuntu-latest)` | **pass** | `CI-UBUNTU-01` | `899b65f6` | [33975024580](https://github.com/agentkernel/cognitive-os/actions/runs/33975024580) job ubuntu SUCCESS |
| 2026-09-05 | Required CI `verify (windows-latest)` + `required-ci` | **pass** | `CI-WINDOWS-MSVC-01` | `899b65f6` | [33975024580](https://github.com/agentkernel/cognitive-os/actions/runs/33975024580) windows 15m7s SUCCESS; required-ci 3s SUCCESS |
| 2026-09-05 | Live `/ui/` J5 Knowledge v9 IA | **pass** | `B01-Desktop-Linux-002` daemon `/ui/` via host SSH tunnel `127.0.0.1:48681` | UI bundle `899b65f6`; daemon pin `711a5a7c` (EVAL-016; TS-only, binary not replaced) | Product origin `http://127.0.0.1:48681/ui/#/knowledge`. `data-knowledge-ia=v9`. Lede: files / Why this fragment / import. Tabs Files / Import / Why this fragment / Memory. Files empty: “No files yet” + Import files CTA. Import: `input[name=vault-files]` type=file; empty submit “Import needs a file. The original fields stay.”; palette **not** open. Why: inject_order task-contract → older-narrative; honest empty excerpts. Memory: “Chat auto-admission Requires-backend”; 0 Admit buttons; inspect/correct/promote/forget retained. No “Obsidian”. No HTTP-paste headline. Vite not product origin. |
| 2026-09-05 | `JOURNEY-BROWSER-SYNC-01` regression J0 | **pass** (same minor as EVAL-016) | same `/ui/` | `899b65f6` | Gate copy forbids Provider keys; sessions memory-only; Session page principal + Clear memory session |
| 2026-09-05 | Regression J10 parked X | **pass** | Today + Settings `/ui/` | `899b65f6` | No X/Twitter P0 hero. Settings exists as `#/settings` (P13-T08 surface; T07 owns L1/palette — not claimed here) |
| 2026-09-05 | Regression J18 session | **pass** | `#/session` | `899b65f6` | Session link; principal `principal://local/owner`; Clear memory session |
| 2026-09-05 | Regression J19 dead routes | **pass** | `#/inbox` `#/team` `#/hitl/prev-1` | `899b65f6` | Each → “No such route” / “This address does not exist in the Control Plane.” |
| 2026-09-05 | Closed Phase 14 pack besides T01 docs | **n/a** | — | — | Only `P14-T01` is done; no other Phase 14 journeys to regress |
| 2026-09-05 | Host OS file-dialog ingest E2E | **not-run** | browser cannot drive native file picker here | — | Dual Track covers file-as-authority + secret ingest; live empty-file refuse observed |
