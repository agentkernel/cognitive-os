# P14-T03 Write Project titled live Project — running report

- Task / slice: `P14-T03/D01` local Dual Track pass (D02 not started)
- Lease: `lease/personal/P14-T03/write-live-project`
- Branch: `personal/P14-T03-write-live-project`
- Change class: `implementation-only` (G1 Write Project durable title + PlanRevision axis + leave `creating`; no `core/specs`; no numbered migration — v41 remains reserved)
- Claim ceiling: `hypothesis`
- Product origin: daemon `/ui/`
- Evaluation routing: **OFF**
- Do not claim T07. T02 merged PR [#329](https://github.com/agentkernel/cognitive-os/pull/329) at `main@c9bb291d`. T08 merged PR [#328](https://github.com/agentkernel/cognitive-os/pull/328).

## Units

| Unit | Result | Evidence |
|---|---|---|
| Claim + T02 lease close | pass | this branch from `origin/main@c9bb291d`; T02 row → PARALLEL-LANES §3.1 |
| Failure-first Dual Track (empty title / `unknown` / leave `creating` + axis / no-axis still live) | fail then pass | observed 4/4 fail on current `creating` G1; after `activate_locked` Dual Track path: `cargo test -p cognitive-store --test p14_t03_write_live_project --locked -- --test-threads=1` **4/4 pass** (local MSVC host `x86_64-pc-windows-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`; development evidence only) |
| Behavior change | pass | Dual Track titled+process → `active` + `accepted_at` + PlanRevision + Owner `title_summary`; empty/`unknown`/empty `process:` refuse with no row; G1 without `process:` stays `creating`. HTTP list/detail use daemon title. Local: P11-T03 store 19/19; kernel-server `write_project_http_*` 2/2 + `g1_confirm_mints_creating_project` pass; web `projects.test.ts` 5/5 |
| `JOURNEY-BROWSER-SYNC-01` D02 | not-run | after D01 push + guest `/ui/` including T02 pack (J0/J1 wizard/J10/J18/J19) |

Unique next: commit/push D01, open Draft PR, then `P14-T03/D02` guest `/ui/` + `JOURNEY-BROWSER-SYNC-01`.
