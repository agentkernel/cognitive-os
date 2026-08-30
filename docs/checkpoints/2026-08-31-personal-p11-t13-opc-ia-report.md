# P11-T13 OPC `/ui/` IA — running report

- Task: `P11-T13` / slice `P11-T13/D01`
- Change class: `implementation-only` (Dual Track `clients/pc/web` chrome; no `core/specs`, no kernel-server lease, no Vite-as-product)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P11-T13/opc-ia`
- Branch: `personal/P11-T13-opc-ia`
- Worktree: `D:\agent-kernel-wt-P11-T05` (original `d:\agent-kernel` left untouched)
- Claim ceiling: `hypothesis` (A7: local/CI is not Gate/release/Profile; NVDA / 200% / host-theme / host UI E2E are `not-run`)
- Evaluation routing: **OFF** (`PERSONAL-PERF-EVAL-015` closed)

## Unique next action

Continue `P11-T13/D01` on `personal/P11-T13-opc-ia`: Dual Track empty/unavailable L1 is in this commit. Remaining is the rest of T13 acceptance (no full fake OPC chrome; NVDA/200%/host-theme stay hung `not-run`; host UI E2E vs daemon `/ui/` `not-run` until a guest is up). Do not auto-claim T02. Do not unpark T14/T15.

## Closed predecessor

`P11-T08` **done**: merged PR [#290](https://github.com/agentkernel/cognitive-os/pull/290) at `main@bda740f6`. Close commit `d897f540` is on `origin/main`. Lease `lease/personal/P11-T08/routine` closed into PARALLEL-LANES §3.1. Clock/sleep/restart E2E remains `not-run`.

## Identifier

Acceptance: `TODAY_PROJECTS_KNOWLEDGE_SETTINGS_UI`.

Product origin is daemon-served hash `/ui/`. Linux 1.0 Home stays at `#/home`. Team/Inbox are not L1. Chat Approve is not a Control Plane control.

Reused: SessionGate, hash `/ui/`, P7-T05 inventory honesty (empty ≠ denied ≠ disconnected ≠ stub), `GET /management/project/v1/list`. No kernel-server path claimed; `/ui/` static mount and project list already exist on `main`.

## Failure-first (this slice)

| ID | Test | Surface | Status |
|---|---|---|---|
| N1 | empty Project list is empty, not fake OPC chrome | `opcIa.test.tsx` Today `#/` with `projects: []`; 0 fake action buttons; no Create/Activate/Approve | **pass** (Vitest, `DEV-WIN-GNU-01`) |
| N2 | 403 is denied, not empty | Projects `#/projects` HTTP 403 | **pass** |
| N3 | 503 is unexpected, not empty | Projects HTTP 503 | **pass** |
| N4 | 200-stub is not-run/unavailable, not empty | Today daemon stub note | **pass** |
| N5 | fetch throw is disconnected, not empty | Today `Failed to fetch` | **pass** |
| N6 | L1 is Today/Projects/Knowledge; Settings in side-foot; no Team/Inbox | PrimaryNav + App.test + opcIa | **pass** |
| N7 | Assistant rail has no Approve control | `AssistantRail` | **pass** |
| projector | malformed list does not invent a Project | `projects.test.ts` | **pass** |

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

| Unit | Environment | Result |
|---|---|---|
| `pnpm test` in `clients/pc/web` | `DEV-WIN-GNU-01` | **pass** 40 files / 319 tests |
| `pnpm run build` (`tsc --noEmit` + Vite) | `DEV-WIN-GNU-01` | **pass** (CSS 23.25 kB, JS 442.89 kB) |
| Rust cargo | `DEV-WIN-GNU-01` | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |
| Host UI E2E vs daemon `/ui/` | guest not used this session | **not-run** |
| NVDA / 200% layout / host-theme contrast | hung | **not-run** |
| required-ci | not yet on this head | **not-run** |

## Non-claims

- Not Gate, release, Profile, B01, Agent-benefit, or complete T13 `/ui/` acceptance.
- Empty honesty is not a Today packet canvas and not a fake Requires-backend control.
- Linux 1.0 six-family pages remain real secondary routes (`#/home`, Work, …), not L1.
- Vite is not the product origin.
- CI ≠ Gate (A7).

## Implemented in this slice

- L1: Today / Projects / Knowledge; Settings in side-foot; assistant rail (candidate-only).
- Dual Track: `fetchProjection` + whitelist `GET /management/project/v1/list`.
- Keyboard: `g` then t/p/n/s; keep w/a/h/v/r/c.
- Palette destinations include L1 + Linux 1.0 + Settings.
