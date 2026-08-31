# P12-T05 Today decision packets — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P12-T05` / slice `P12-T05/D01`
- Branch: `personal/P12-T05-today-packets`
- Lease: `lease/personal/P12-T05/today-packets`
- Change class: `implementation-only` (daemon-served `/ui/` Today packets; no new authority writer; no `core/specs`)
- Unique next: Dual Track TS + Draft PR + required CI

Product origin is daemon-served `/ui/`. Vite/canvas is not the product. NVDA/200%/host-theme remain hung. Native UI E2E = `DEV-WINDOWS-NATIVE-OPC-01` / `not-run`. `DEV-WIN-GNU-01` cargo is `not-run` (`RUST-LINK-DEV-WIN-GNU-01`). Creating-only = continue-create (`today-incomplete`). Live = pending-previews packets deep-linked to `/projects/:id?preview=`. No KPI wall. Chat has no Approve. Not T06 Confirm. Not T15.

## Units

| Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|
| Claim `lease/personal/P12-T05/today-packets` | **pass** | `DEV-WIN-GNU-01` | worktree `D:/agent-kernel-wt-P12-T05` stacked on `origin/main@8c413648` | T04 PR [#297](https://github.com/agentkernel/cognitive-os/pull/297) **merged** at `main@8c413648`. DOC-REFRAME retained. Evaluation routing OFF. |
| Dual Track TS Today packets (`todayPackets` + `opcIa` + projections) | **pass** | `DEV-WIN-GNU-01` | pending push | personal-web-ui **373/373**. Creating-only does not GET pending-previews. Live packets deep-link `/projects/:id?preview=`. Empty home remains only-create. Native UI E2E **not-run**. NVDA/200%/host-theme **not-run**. GNU cargo **not-run**. |
| NVDA / 200% / host-theme | **not-run** | Requires-environment | — | hung; not a P12 close gate |
| Native UI E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` unqualified | — | not a product fail |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | — | route to `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` |
