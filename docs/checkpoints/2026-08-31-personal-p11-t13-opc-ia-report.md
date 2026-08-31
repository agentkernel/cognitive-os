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

Keep Draft PR [#291](https://github.com/agentkernel/cognitive-os/pull/291) until required-ci is green on this head. L1 honesty now includes Settings Advanced collapsed, Today → Projects HITL canvas deep-link (`?preview=`), and assistant-rail announce-only. Remaining: required-ci on this head; host UI E2E vs a **T13-revision** daemon `/ui/` on `DEV-LINUX-NATIVE-01` (not B01 guest); NVDA/200%/host-theme hung `not-run`. Do not auto-claim T02. Do not unpark T14/T15. Do not flip ready/merge yet.

## Closed predecessor

`P11-T08` **done**: merged PR [#290](https://github.com/agentkernel/cognitive-os/pull/290) at `main@bda740f6`. Close commit `d897f540` is on `origin/main`. Lease `lease/personal/P11-T08/routine` closed into PARALLEL-LANES §3.1. Clock/sleep/restart E2E remains `not-run`.

## Identifier

Acceptance: `TODAY_PROJECTS_KNOWLEDGE_SETTINGS_UI`.

Product origin is daemon-served hash `/ui/`. Linux 1.0 Home stays at `#/home`. Team/Inbox are not L1. Chat Approve is not a Control Plane control.

Reused: SessionGate, hash `/ui/`, P7-T05 inventory honesty (empty ≠ denied ≠ disconnected ≠ stub), `GET /management/project/v1/list`, `GET /management/project/v1/pending-previews`, `GET /management/project/v1/vault.index`, `GET /management/project/v1/standing-policies`, `GET /management/resource/v1/list?family=memory`. No kernel-server path claimed; those GETs already exist on `main`.

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
| N8 | no Project id ⇒ no pending-previews / vault.index / memory list call | `opcIa.test.tsx` | **pass** |
| N9 | HITL announce-only; 403 does not invent preview rows; no Confirm | `opcIa.test.tsx` + `hitl.test.ts` | **pass** |
| N10 | Vault/Memory 403 does not invent files; no ingest | `opcIa.test.tsx` + `vault.test.ts` | **pass** |
| N11 | Settings StandingApprovalPolicy list-only; no Team/Inbox/member budget control | `opcIa.test.tsx` + `standingPolicies.test.ts` | **pass** |
| N12 | L1/Settings copy does not claim Vite as product origin | `opcIa.test.tsx` | **pass** |
| N13 | POST confirm / vault.apply-authority stay off the client whitelist | `normalize.test.ts` | **pass** |
| N14 | Settings Advanced (Linux 1.0) is collapsed by default | `opcIa.test.tsx` | **pass** (Vitest, `DEV-WIN-GNU-01`) |
| N15 | Today deep-links into `#/projects?preview=`; never `#/hitl` / Inbox L1 | `opcIa.test.tsx` + `hitl.test.ts` | **pass** |
| N16 | empty Project list ignores `?preview=` (no invented rows / HITL GET) | `opcIa.test.tsx` | **pass** |
| N17 | `#/hitl`, `#/inbox`, `#/team` are missing routes, not L1 | `opcIa.test.tsx` | **pass** |

## Validation log (`TEST-REPORT-INCREMENTAL-01`)

| Unit | Environment | Result |
|---|---|---|
| `pnpm test` in `clients/pc/web` | `DEV-WIN-GNU-01` | **pass** 40 files / 319 tests |
| `pnpm run build` (`tsc --noEmit` + Vite) | `DEV-WIN-GNU-01` | **pass** (CSS 23.25 kB, JS 442.89 kB) |
| Rust cargo | `DEV-WIN-GNU-01` | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |
| Host UI E2E vs daemon `/ui/` | guest not used this session | **not-run** |
| NVDA / 200% layout / host-theme contrast | hung | **not-run** |
| required-ci | not yet on this head | **not-run** |
| HTTPS `git push -u origin HEAD` | `DEV-WIN-GNU-01` | **pass** `b5b604c0` → `origin/personal/P11-T13-opc-ia` (SSH host-key failed first; HTTPS retry 1 succeeded) |
| Draft PR | GitHub | **pass** [#291](https://github.com/agentkernel/cognitive-os/pull/291) stays Draft |
| `pnpm test` in `clients/pc/web` (fail-closed reads) | `DEV-WIN-GNU-01` | **pass** 43 files / 335 tests |
| `pnpm run build` (`tsc --noEmit` + Vite) | `DEV-WIN-GNU-01` | **pass** (CSS 23.25 kB, JS 450.94 kB) |
| Host UI E2E vs **this** T13 `/ui/` | linux-002 `:48681` LISTEN on loopback; this branch not deployed | **not-run** |
| NVDA / 200% layout / host-theme contrast | hung | **not-run** |
| required-ci | pending push of this head | **not-run** |
| `pnpm test` in `clients/pc/web` (Settings Advanced + HITL canvas deep-link) | `DEV-WIN-GNU-01` | **pass** 43 files / 340 tests |
| `pnpm run build` (`tsc --noEmit` + Vite) | `DEV-WIN-GNU-01` | **pass** (CSS 23.25 kB, JS 452.22 kB) |
| `resolve validation route` on `865339f2` | GitHub Actions run [33344595222](https://github.com/agentkernel/cognitive-os/actions/runs/33344595222) | **pass** job [99346265478](https://github.com/agentkernel/cognitive-os/actions/runs/33344595222/job/99346265478) |
| `verify (ubuntu-latest)` on `865339f2` | `CI-UBUNTU-01` run [33344595222](https://github.com/agentkernel/cognitive-os/actions/runs/33344595222) | **pass** job [99346278145](https://github.com/agentkernel/cognitive-os/actions/runs/33344595222/job/99346278145) (3m50s) |
| `verify (windows-latest)` on `865339f2` | `CI-WINDOWS-MSVC-01` run [33344595222](https://github.com/agentkernel/cognitive-os/actions/runs/33344595222) | **pending** job [99346278170](https://github.com/agentkernel/cognitive-os/actions/runs/33344595222/job/99346278170) (superseded if this head is pushed) |
| `required-ci` on `865339f2` | GitHub Actions | **not-run** (windows still pending; do not invent green) |
| Host UI E2E vs **this** T13 `/ui/` | `DEV-LINUX-NATIVE-01` (not B01 guest) | **not-run** until this head is pushed and a disposable daemon `/ui/` listens |
| NVDA / 200% layout / host-theme contrast | hung | **not-run** |
| Rust cargo | `DEV-WIN-GNU-01` | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |

## Non-claims

- Not Gate, release, Profile, B01, Agent-benefit, or complete T13 `/ui/` acceptance.
- Empty honesty is not a Today packet canvas and not a fake Requires-backend control.
- Linux 1.0 six-family pages remain real secondary routes (`#/home`, Work, …), not L1.
- Vite is not the product origin.
- linux-002 `:48681` LISTEN is not this T13 `/ui/` bundle (no guest deploy this turn).
- CI ≠ Gate (A7).

## Implemented in this slice

- L1: Today / Projects / Knowledge; Settings in side-foot; assistant rail (candidate-only).
- Dual Track: `fetchProjection` + whitelist `GET /management/project/v1/list`, `pending-previews`, `vault.index`, `standing-policies` (Memory list already whitelisted).
- Today/Projects: HITL announce-only for the first daemon Project id; no Confirm/Approve.
- Knowledge: Vault index + Memory envelope after a real Project id; no ingest.
- Settings: StandingApprovalPolicy list-only; member budget remains 2.1 / Deferred.
- Settings Advanced (Linux 1.0 Home/Work/Agents/…) hidden by default (`<details>` closed).
- Today deep-links pending ApprovalPreview into `#/projects?preview=` (project-center canvas). Not `#/hitl`. Not Inbox L1. Canvas does not mint Confirm.
- Assistant rail announces pending HITL already loaded in this tab; no Approve control.
- Projects populated only from GET list; `?preview=` cannot invent a Project.
- Keyboard: `g` then t/p/n/s; keep w/a/h/v/r/c.
- Palette destinations include L1 + Linux 1.0 + Settings.
- Draft PR [#291](https://github.com/agentkernel/cognitive-os/pull/291).
