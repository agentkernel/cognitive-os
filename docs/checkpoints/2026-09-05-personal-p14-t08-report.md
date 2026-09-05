# P14-T08 Knowledge v9 IA — running report

- Task: `P14-T08` / slice `P14-T08/D01`
- Change class: `implementation-only` (Knowledge `/ui/` IA; reuse P13-T07 Vault/Memory authority and P12-T07 `vault.import`; no new numbered migration; no Obsidian)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P14-T08/knowledge-ia`
- Branch: `personal/P14-T08-knowledge-ia` (worktree `D:\agent-kernel-p14-t08`; do not edit dirty `D:\agent-kernel`)
- Base: `origin/personal/DOC-P14-GAP-CLOSE` `@e14bc7a7` (Phase 14 cards; `origin/main` does not yet contain them)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI is not Gate / release / Profile / Windows qualification)
- Evaluation routing: **OFF**
- Product origin: daemon-served `/ui/` (Vite is not the product origin)
- Host FS E2E: `not-run`

## Unique next action

Finish `P14-T08/D01` Dual Track + required CI, then `P14-T08/D02` exact-revision `B01-Desktop-Linux-002` `/ui/` J5 + `JOURNEY-BROWSER-SYNC-01` (regression J0/J10/J18/J19 + closed Phase 14 pack). Do not claim T02/T03/T04 (Worker 1) or T07/T06 (Worker 2). After T08 full close, claim T05 only if T03 is done and T05 is unclaimed.

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
| 2026-09-05 | Host FS / live `/ui/` J5 | **not-run** | `P14-T08/D02` | — | D02 after this tsc fix is pushed |
