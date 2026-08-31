# P12-T04 select-then-configure + add member — closure

- Task: `P12-T04` / slice `P12-T04/D01`
- Branch: `personal/P12-T04-member-config`
- Lease: `lease/personal/P12-T04/member-config` → §3.1
- PR: [#297](https://github.com/agentkernel/cognitive-os/pull/297)
- Content: `ac93ac23`; docs-head `49ad8812`
- Required CI: [33383681338](https://github.com/agentkernel/cognitive-os/actions/runs/33383681338) **SUCCESS** at `49ad8812` (resolve 2s, ubuntu 3m41s, windows 12m37s, required-ci 4s)
- Change class: `implementation-only`
- Claim ceiling: `hypothesis`

`#/projects/:id/members/new` and `#/projects/:id/members/:memberId` walk current-Project roster, axis slots, and grant catalog. Write join posts `roster.register` then `seat.request` then `seat.confirm`. Refuse does not mint. Surplus member without a slot fails closed. No Install store. No member-level budget. Dual Track TS: personal-web-ui **367/367**. GNU cargo **not-run**. NVDA/200%/host-theme **not-run**. Native UI E2E **not-run**. Product origin is daemon `/ui/`. Not T05 packets. Not T06 HITL Confirm. Not T15.

Unique next: merge #297, then claim `P12-T05` (Today decision packets).
