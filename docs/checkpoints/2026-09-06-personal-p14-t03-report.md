# P14-T03 Write Project titled live Project — running report

- Task / slice: `P14-T03/D02` guest `/ui/` (D01 Dual Track local pass)
- Lease: `lease/personal/P14-T03/write-live-project`
- Branch: `personal/P14-T03-write-live-project`
- Draft PR: [#330](https://github.com/agentkernel/cognitive-os/pull/330)
- Fold HEAD: `1d50fcc9` (`origin/main@2d92dd16` P14-T07)
- Change class: `implementation-only` (G1 Write Project durable title + PlanRevision axis + leave `creating`; no `core/specs`; no numbered migration — v41 remains reserved)
- Claim ceiling: `hypothesis`
- Product origin: daemon `/ui/` (`http://127.0.0.1:48681/ui/`) — Vite is not the product source
- Guest: `B01-Desktop-Linux-002` `hal9001@192.168.123.160` runtime `/home/hal9001/p13-main-711a5a7c/`
- Evaluation routing: **OFF**
- Do not claim T05/T06/T07/T08. T02/T07/T08 remain **done**. After T03 close, claim T04.

## Units

| Unit | Result | Evidence |
|---|---|---|
| Claim + T02 lease close | pass | this branch from `origin/main@c9bb291d`; T02 row → PARALLEL-LANES §3.1 |
| Failure-first Dual Track (empty title / `unknown` / leave `creating` + axis / no-axis still live) | fail then pass | observed 4/4 fail on current `creating` G1; after `activate_locked` Dual Track path: `cargo test -p cognitive-store --test p14_t03_write_live_project --locked -- --test-threads=1` **4/4 pass** (local MSVC host `x86_64-pc-windows-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`; development evidence only) |
| Behavior change | pass | Dual Track titled+process → `active` + `accepted_at` + PlanRevision + Owner `title_summary`; empty/`unknown`/empty `process:` refuse with no row; G1 without `process:` stays `creating`. HTTP list/detail use daemon title. Local: P11-T03 store 19/19; kernel-server `write_project_http_*` 2/2 + `g1_confirm_mints_creating_project` pass; web `projects.test.ts` 5/5 |
| Fold `origin/main@2d92dd16` (P14-T07) | pass | merge `1d50fcc9`; last-merger keeps T02/T07/T08 **done** and T03 **in-progress**. Settings/Knowledge/wizard not reverted. |
| Required CI at fold `1d50fcc9` | fail | run [33982467722](https://github.com/agentkernel/cognitive-os/actions/runs/33982467722): tools `check.test.mjs` — blank line between consecutive active lease table rows left DOC-REFRAME in the fixture after inject. Fix: adjacent active rows. |
| Exact-revision Linux build | pass | `wuz@192.168.1.2` worktree `/home/wuz/cognitiveos-personal-worktrees/p14-t03-1d50fcc9`; `kernel-server` ELF 42798272; UI dist `index-CvtgBWRI.js` |
| Guest daemon replace on 48681 | pass | PID 2632150 `kernel-server --personal --bind 127.0.0.1:48681`; `REVISION=1d50fcc9…`; `/ui/` 200 serves `index-CvtgBWRI.js`. dsh web restarted on 3080. Left `:48181` untouched. |
| D02 Dual Track activation (title ≠ unknown, leave `creating`, PlanRevision axis) | pass | Same guest daemon: `draft.create` → `preview.request` → `confirm` 200. List: `state=active`, `title_summary=P14-T03 D02 titled live` (not `unknown`). Detail `plan.plan_revision_id` prefix `plan-01a072c7-cf`. Bootstrap read only from guest file; not printed. |
| J10 no X/Twitter P0 hero | pass | Cursor browser → tunnel `http://127.0.0.1:48681/ui/#/`. CDP `bodyHasTwitter=false`. Bundle `index-CvtgBWRI.js`. |
| J19 retired routes | pass | `#/inbox`, `#/team`, `#/hitl/prev-1` each `No such route` / “This address does not exist in the Control Plane.” |
| Settings L1 + Knowledge L1 chrome | pass | Primary nav Settings and Knowledge are `role=link` (`#/settings`, `#/knowledge`) after T07/T08 fold. |
| J0 / J18 / J1 wizard click-walk | not-run | Kernel replace cleared in-memory sessions. Gate asks for daemon bootstrap (not a Provider key). Issue without paste stays fail-closed. Browser walk of ①–⑤ waits for owner paste on the forwarded `/ui/`. |

Unique next: push lease-table CI fix; re-run required CI; owner paste bootstrap on forwarded `/ui/` then finish J1 wizard + J0/J18; then ready/merge #330 and claim T04.
