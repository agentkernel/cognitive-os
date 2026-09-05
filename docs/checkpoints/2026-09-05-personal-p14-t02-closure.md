# P14-T02 create wizard Dual Track ①–⑤ — closure

- Task: `P14-T02` **done** / slices `P14-T02/D01` **done** + `P14-T02/D02` **done**
- Change class: `implementation-only` (Dual Track `/ui/` create wizard surfaces wired to existing P11-T03 confirm-before-activate; no `core/specs`, no Lane-CTR, no contract or negative weakened)
- Lease: `lease/personal/P14-T02/create-wizard` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P14-T02-create-wizard` → Draft PR [#329](https://github.com/agentkernel/cognitive-os/pull/329) (ready/merge in this close)
- Implementation revision: `1118076c` (Dual Track ①–⑤). D02 report + this closure on the same PR.
- Validated HEAD before this closure: `c9de0907` required CI [33976523572](https://github.com/agentkernel/cognitive-os/actions/runs/33976523572) **SUCCESS** (resolve 4s, ubuntu 3m58s, windows 17m21s, required-ci 3s)
- Running report: [P14-T02 report](2026-09-05-personal-p14-t02-report.md)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI / guest `/ui/` close "the Dual Track wizard exists" only). Not Gate / release / Profile / B01. Windows native chrome JOURNEY **not-run**.
- Evaluation routing: **OFF**. Formal-plan status row for `P14-T02` stays `not-started` until last-merger of `PERSONAL-DEVELOPMENT-PLAN.md` (sibling `P14-T08` lease). Current snapshot owns the done fact.

## 1. Acceptance mapping (formal plan P14-T02 card + D01/D02)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| daemon `/ui/` create wizard = prototype ①–⑤ (process / members / test / joint), not five note textareas | `CreateWizardPage.tsx` + `createWizardModel.ts`: ① Charter title+charter; ② process axis one ring at a time (`确认这一环` / `本环留缺口`) then `确认总目标与项目触发`; ③ sequential seating from confirmed process (model required, six runtime slots, `确认就位` / `拒绝此岗`); ④ openable pass/fail; ⑤ 验收 → existing `draft.create` → `preview.request` → `confirm` | Dual Track forbids `textarea[name=process\|members\|verification]`; FAKE_ACTION regex has 0 Create/Activate hits | vitest `createWizard.test.tsx` **7/7** after observed 6-fail; live `/ui/` J1 walk at `1118076c` |
| Wire into existing confirm-before-activate (P11-T03). No authority write without preview | Write Project disabled until digest-bound preview; `requestPreview` refuses unless joint pass + all tests pass | bypass preview → no confirm post; 422 honesty when `draft.create` rejects | Dual Track preview→confirm cell; live: Write disabled until preview; draft `draft-01a07247-…` / preview `preview-01a07247-…` then confirm |
| 0 fake Create/Activate buttons | Existing P11-T03 labels `Request preview` / `Write Project` only | FAKE_ACTION | Dual Track + live CDP `fake=[]` |
| Failure-first Dual Track tests before behavior change | Tests written against note-textarea wizard; 6/7 failed (missing `确认这一环`) | — | report unit "fail observed" |
| D02 real `/ui/` caller ①–⑤ + `JOURNEY-BROWSER-SYNC-01` | Exact-revision guest daemon `/ui/` (`B01-Desktop-Linux-002`, bundle `index-IWe3ScfU.js` at `1118076c`) | Vite not used | J1 **pass**; J0/J10/J18/J19 **pass** |

Formal-plan 关闭门: ①–⑤ 接到既有 confirm-before-activate — **true**; 无权威不写 Project — **true**; 0 假 Create/Activate — **true**.

Drift negatives: 五张 textarea 冒充 ①–⑤ — not present; 假 Create/Activate — 0; 向导绕过 preview 写权威 — Write stays disabled; Vite 当产品源 — guest `/ui/` hash only; secret 进 DOM — `sk-` pattern false.

Honest leftover owned by **P14-T03** (not a T02 fail): after Write, Today stays continue-create; list title `unknown`; state `creating`; no PlanRevision axis (EVAL-016 J1 blocker 1).

## 2. Validation summary

| Environment | Result |
|---|---|
| `DEV-WIN-GNU-01` (Node) | `createWizard.test.tsx` **7/7**; `createAssistantChat.test.tsx` **10/10**; `clients/pc/web` `pnpm build` **pass**; `check:consistency` **pass** |
| Local MSVC override | **not-run** this task (no Rust behavior change in D01) |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | [33976523572](https://github.com/agentkernel/cognitive-os/actions/runs/33976523572) **SUCCESS** at `c9de0907`. Closure-head CI recorded after this commit. |
| `DEV-LINUX-NATIVE-01` | Exact-revision `pnpm build` of `1118076c` (worktree `/home/wuz/cognitiveos-personal-worktrees/p14-t02-1118076c`, dirty=0); dist `index-IWe3ScfU.js` |
| `B01-Desktop-Linux-002` | Static `/ui/` swap into existing daemon `:48681` (pid left running). J1 ①–⑤ + preview→confirm **pass**. Did not rebuild `kernel-server`. |
| `DEV-WINDOWS-NATIVE-OPC-01` chrome | **not-run** (walk used Cursor browser against forwarded guest `/ui/`) |

## 3. Non-claims

Not T03 titled live Project / PlanRevision axis / leave `creating`. Not T07 Settings L1. Not T08 Knowledge files/why/import. No Gate / release / Profile / B01. No Vite origin. No secret in Git/DOM/report. Formal-plan `P14-T02` status cell is sibling-owned until last-merger.

## 4. Unique next

Ready/merge PR [#329](https://github.com/agentkernel/cognitive-os/pull/329) after required CI on this closure HEAD. Then claim `P14-T03` (Write Project → titled live Project + PlanRevision axis). Do not claim T07/T08.
