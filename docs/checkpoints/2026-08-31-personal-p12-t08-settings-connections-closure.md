# P12-T08 Settings connections — closure

- Task: `P12-T08` / slice `P12-T08/D01`
- Branch: `personal/P12-T08-settings`
- Lease: `lease/personal/P12-T08/settings-connections` → §3.1
- PR: [#301](https://github.com/agentkernel/cognitive-os/pull/301)
- Content: `bd440f72`; docs-head `21036106`
- Required CI: [33418686755](https://github.com/agentkernel/cognitive-os/actions/runs/33418686755) **SUCCESS** at `21036106` (resolve 3s, ubuntu 3m35s, windows 12m22s, required-ci 4s)
- Change class: `implementation-only`
- Claim ceiling: `hypothesis`

Settings connections live on daemon `/ui/` Settings. The table reads GET `/management/providers/accounts` + GET `/management/usage`; unknown / `cost_unavailable` never render as 0; secret presence only. 「本周不再问」revoke is POST `/management/project/v1/standing-policy.revoke` (time-box, not permanent; chat cannot mint). CloseBackgroundDialog POSTs `/management/host/v1/close.request` choice `background`|`pause` only when GET `host/v1/status` reports a home. Dual Track TS: personal-web-ui **405/405**. GNU cargo **not-run**. NVDA/200%/host-theme **not-run**. Native close/host E2E **not-run**. Product origin is daemon `/ui/`. Not T09 rail-write. Not T15.

Unique next: merge #301, then claim `P12-T09` (right-rail edit → confirm → write canvas).
