# P14-T03 Write Project titled live Project — closure

- Task: `P14-T03` **done** / slices `P14-T03/D01` **done** + `P14-T03/D02` **done**
- Change class: `implementation-only` (Dual Track Write Project mints Owner title + PlanRevision axis + leaves `creating`; no `core/specs`; no numbered migration — v41 remains reserved)
- Lease: `lease/personal/P14-T03/write-live-project` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P14-T03-write-live-project` → Draft PR [#330](https://github.com/agentkernel/cognitive-os/pull/330) (ready/merge in this close)
- Implementation revision: `9bca48dc` (D01 Dual Track). Fold `1d50fcc9` (`origin/main@2d92dd16` P14-T07). Lease-table CI fix `625e9ccd`.
- Validated HEAD before this closure: `625e9ccd` required CI [33983778699](https://github.com/agentkernel/cognitive-os/actions/runs/33983778699) **SUCCESS** (resolve, ubuntu, windows, required-ci)
- Running report: [P14-T03 report](2026-09-06-personal-p14-t03-report.md)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI / guest `/ui/` close "Write Project mints a titled live Project" only). Not Gate / release / Profile / B01. Windows native chrome JOURNEY **not-run**.
- Evaluation routing: **OFF**. T02/T07/T08 remain **done**. Do not claim T05/T06/T07/T08 from this close.

## 1. Acceptance mapping (formal plan P14-T03 card + D01/D02)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| Write Project activates a live named Project: title is Owner-written, not `unknown` | Dual Track `activate_locked`: titled charter + `process:` rings INSERT `p11_project` as `active` with `accepted_at` and persist Owner `title_summary` | empty title; title `unknown` (trim, case) → refuse, no row | store Dual Track 4/4 after observed fail; guest HTTP list `title_summary=P14-T03 D02 titled live`; wizard list `P14-T03 D02 wizard titled live` / `state=active` |
| PlanRevision axis exists | mint PlanRevision via `apply_plan_revision_locked`; HTTP detail + `/axis` | no-axis still marked live is refused | guest HTTP detail `plan.plan_revision_id` prefix `plan-01a072c7-cf`; wizard detail `Plan revision` `plan-01a072df-8ff5-…`; axis stages collect/analyze/draft |
| State does not stay `creating` | Dual Track path leaves G1 `creating` only when charter has no `process:` | G1 without `process:` stays `creating` (not a Dual Track pass) | guest Dual Track + wizard rows `state=active`; leftover T02 `creating`/`unknown` rows remain honest pre-T03 G1 |
| EVAL-016 J1 blocker 1 + `JOURNEY-BROWSER-SYNC-01` | exact-revision guest daemon `/ui/` on `B01-Desktop-Linux-002` | Vite not used; fail-closed unauthenticated Issue | J1 wizard ①–⑤ + preview→confirm **pass**; J0/J10/J18/J19 **pass**; Settings/Knowledge L1 chrome **pass** |
| Failure-first Dual Track before behavior change | tests written against then-current G1 `creating` | empty/`unknown`/empty `process:` | observed 4/4 fail then 4/4 pass |

Formal-plan 关闭门: Write 后列表/详情标题 = Owner 所写 — **true**; PlanRevision axis 存在 — **true**; 离开 `creating` — **true**.

Drift negatives: 标 live 但 title=`unknown` — Dual Track refuses; 无 axis 仍标 live — Dual Track refuses; 卡在 `creating` — Dual Track titled+process path does not; 把 fail-closed 写成 pass — unauthenticated Issue stays 401; Windows chrome **not-run**.

Honest leftover (not a T03 fail): pre-T03 G1 rows remain `creating`/`unknown` (T02 continue-create). Today may still show Continue create for those rows. T06 owns live Today packet. T04 owns member join on the new PlanRevision slots.

## 2. Validation summary

| Environment | Result |
|---|---|
| Local MSVC override | store Dual Track **4/4**; P11-T03 **19/19**; kernel-server HTTP **2/2** + G1 creating still pass; web `projects.test.ts` **5/5**. Development evidence only. |
| `DEV-WIN-GNU-01` | Rust link **not-run** (routed). `check:consistency` **pass** on lease-table fix. |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | [33983778699](https://github.com/agentkernel/cognitive-os/actions/runs/33983778699) **SUCCESS** at `625e9ccd`. Fold CI [33982467722](https://github.com/agentkernel/cognitive-os/actions/runs/33982467722) **FAILED** (lease fixture blank line) then repaired. |
| `DEV-LINUX-NATIVE-01` | Exact-revision `kernel-server` + UI dist at fold `1d50fcc9` (`index-CvtgBWRI.js`). Docs-only `625e9ccd` did not change the guest binary. |
| `B01-Desktop-Linux-002` | Daemon `:48681` Dual Track HTTP activation **pass**; wizard click-walk ①–⑤ + `Request preview` / `Write Project` **pass**. Left `:48181` untouched. |
| `DEV-WINDOWS-NATIVE-OPC-01` chrome | **not-run** (walk used Cursor browser against forwarded guest `/ui/`) |

## 3. Non-claims

Not T04 member join on PlanRevision slots. Not T05 Attempt/Runs/Outputs. Not T06 Today live packet. T02/T07/T08 stay **done**. No Gate / release / Profile / B01. No Vite origin. No secret in Git/DOM/report. No numbered migration.

## 4. Unique next

Ready/merge PR [#330](https://github.com/agentkernel/cognitive-os/pull/330) after required CI on this closure HEAD (implementation pin `625e9ccd` already green). Then claim `P14-T04` and start `P14-T04/D01` failure-first Dual Track (PlanRevision responsible slots; write-join seats; negatives: no-slot fake join, chat Approve). Do not claim T05/T06/T07/T08.
