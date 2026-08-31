# P12-T04 select-then-configure + add member — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P12-T04` / slice `P12-T04/D01`
- Branch: `personal/P12-T04-member-config`
- Lease: `lease/personal/P12-T04/member-config`
- Change class: `implementation-only` (daemon-served `/ui/` select-then-configure + add-member; no new authority writer; no `core/specs`)
- Unique next: merge PR [#297](https://github.com/agentkernel/cognitive-os/pull/297) then claim `P12-T05`

Product origin is daemon-served `/ui/`. Vite/canvas is not the product. NVDA/200%/host-theme remain hung. Native UI E2E = `DEV-WINDOWS-NATIVE-OPC-01` / `not-run`. `DEV-WIN-GNU-01` cargo is `not-run` (`RUST-LINK-DEV-WIN-GNU-01`). Write join = `roster.register` → `seat.request` → `seat.confirm`. Refuse = no mint or `seat.confirm accept:false`. No Install store. No member-level budget chrome. Not T05 packets. Not T06 Confirm. Not T15.

## Units

| Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|
| Claim `lease/personal/P12-T04/member-config` | **pass** | `DEV-WIN-GNU-01` | worktree `D:/agent-kernel-wt-P12-T04` stacked on `origin/main@1e736aae` | T03 PR [#296](https://github.com/agentkernel/cognitive-os/pull/296) **merged** at `main@1e736aae`. DOC-REFRAME retained. Evaluation routing OFF. |
| Dual Track TS member-config (`projectWork` + `memberConfig` + `projectSubmenus` + `opcIa` + `normalize`) | **pass** | `DEV-WIN-GNU-01` | `ac93ac23` | personal-web-ui **367/367**. `#/projects/:id/members/new` and `#/projects/:id/members/:memberId`. Write join posts register/seat.request/seat.confirm. Surplus/empty-slot fail closed. Native UI E2E **not-run**. NVDA/200%/host-theme **not-run**. GNU cargo **not-run**. |
| Draft PR [#297](https://github.com/agentkernel/cognitive-os/pull/297) | **pass** | GitHub | `ac93ac23` | Unique next = required CI. |
| required CI [33383681338](https://github.com/agentkernel/cognitive-os/actions/runs/33383681338) | **pass** | GitHub | `49ad8812` | ubuntu 3m41s, windows 12m37s, required-ci 4s |
| NVDA / 200% / host-theme | **not-run** | Requires-environment | — | hung; not a P12 close gate |
| Native UI E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` unqualified | — | not a product fail |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | — | route to `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` |
