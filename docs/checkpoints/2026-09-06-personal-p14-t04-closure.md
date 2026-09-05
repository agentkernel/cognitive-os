# P14-T04 member join Dual Track — closure

- Task: `P14-T04` **done** / slices `P14-T04/D01` **done** + `P14-T04/D02` **done**
- Change class: `implementation-only` (Dual Track process-ring ids become PlanRevision `responsible_slot`; write join seats those slots; no `core/specs`; no numbered migration — v41 remains reserved)
- Lease: `lease/personal/P14-T04/member-join` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P14-T04-member-join` → Draft PR [#331](https://github.com/agentkernel/cognitive-os/pull/331) (ready/merge in this close)
- Implementation revision: `fe498997` (D01 Dual Track). Docs-only D01 report `3a39614e`.
- Validated HEAD before this closure: `3a39614e` required CI [33987473832](https://github.com/agentkernel/cognitive-os/actions/runs/33987473832) **SUCCESS** (resolve, ubuntu, windows, required-ci)
- Running report: [P14-T04 report](2026-09-06-personal-p14-t04-report.md)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI / guest `/ui/` close "write join seats PlanRevision ring slots" only). Not Gate / release / Profile / B01. Windows native chrome JOURNEY **not-run**.
- Evaluation routing: **OFF**. T02/T03/T07/T08 remain **done**. T05 already claimed (Draft PR [#332](https://github.com/agentkernel/cognitive-os/pull/332) at `d72c2847`). Do not claim T05/T07/T08 from this close.

## 1. Acceptance mapping (formal plan P14-T04 card + D01/D02)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| After Dual Track activation, PlanRevision responsible slots exist | `parse_dual_track_stages` sets `responsible_slot = stage_id` (`collect` / `analyze` / `draft`); `rights=` stays Owner access text | T03 collapsed `rights=owner` into one seating slot (observed fail) | store `dual_track_activation_mints_responsible_slots`; guest axis `slots=collect,analyze,draft` on `plan-01a0731e-cfb5-7611-bb17-4c55852933d9` |
| Write join seats those slots | management `roster.register` + `seat.request` + `seat.confirm` for each ring id | surplus/fake slot does not seat | store `write_join_seats_members_on_plan_revision_slots`; HTTP 1/1; guest Write join seated 3 employees; `@ collect` / `@ analyze` / `@ draft` |
| 「no PlanRevision slots」fail-closed no longer blocks Owner | Dual Track titled+process path mints ring slots; G1 without `process:` stays `creating` with no fake join | no-slot fake join refused | store `no_slot_fake_join_is_refused`; surplus `manager` 422 |
| Refuse = not joined | Refuse join does not register; chat cannot Approve | chat Approve; task-channel register | store `chat_approve_must_not_join`; HTTP task register **403**; guest chat “Approve this join” → conversational `no-current-manager`, roster stayed empty until Write join |
| EVAL-016 J4 + `JOURNEY-BROWSER-SYNC-01` | exact-revision guest daemon `/ui/` on `B01-Desktop-Linux-002` | Vite not used; unauthenticated Issue fail-closed | J4 Write join **pass**; J1 regression ①–⑤ **pass**; J0/J10/J18/J19 **pass** |

Formal-plan 关闭门: 激活后 join 落真实槽 — **true**; 拒绝 = 未加入 — **true**.

Drift negatives: 无槽仍假装加入 — Dual Track + HTTP 422; 聊天 Approve — conversational only, 403 on task register; Install 商店 — not offered; 把 fail-closed 写成 pass — unauthenticated list stays 401; Windows chrome **not-run**.

## 2. Validation summary

| Environment | Result |
|---|---|
| Local MSVC override | store Dual Track **5/5**; T03 regression **4/4**; kernel-server HTTP **1/1**; web **73 files / 529 tests**. Development evidence only. |
| `DEV-WIN-GNU-01` | Rust link **not-run** (routed). |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | [33987473832](https://github.com/agentkernel/cognitive-os/actions/runs/33987473832) **SUCCESS** at `3a39614e`. |
| `DEV-LINUX-NATIVE-01` | Exact-revision `kernel-server` + UI dist at `fe498997` (`index-CW6on2hU.js`). Docs-only `3a39614e` did not change the guest binary. |
| `B01-Desktop-Linux-002` | Daemon `:48681` Dual Track wizard Write titled live + Write join **pass**. Left `:48181` untouched. |
| `DEV-WINDOWS-NATIVE-OPC-01` chrome | **not-run** (walk used Cursor browser against forwarded guest `/ui/`) |

## 3. Non-claims

Not T05 Attempt/Runs/Outputs (already claimed, Draft PR [#332](https://github.com/agentkernel/cognitive-os/pull/332); T05 D02 may use guest `:48681` after this close). Not T06 Today live packet (already claimed, Draft PR [#333](https://github.com/agentkernel/cognitive-os/pull/333)). T02/T03/T07/T08 stay **done**. No Gate / release / Profile / B01. No Vite origin. No secret in Git/DOM/report. No numbered migration.

## 4. Unique next

Ready/merge PR [#331](https://github.com/agentkernel/cognitive-os/pull/331) after required CI on this closure HEAD (implementation pin `fe498997` / docs `3a39614e` already green). T05 and T06 are already claimed — do not claim them. Unique next: T05/D02 and/or T06/D02 on freed guest `:48681`. Do not claim T07/T08.
