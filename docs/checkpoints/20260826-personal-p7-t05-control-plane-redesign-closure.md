# P7-T05 Non-blocking Web UI — Control Plane redesign closure

- Task: `P7-T05` / slices `P7-T05/D01`–`D26`
- Status: `done`
- Branch: `personal/P7-T05-d14-rendered-review` (deleted after merge)
- PR: [#274](https://github.com/agentkernel/cognitive-os/pull/274) **merged** at `main@5996afbb`
- Content head: `26e7ae47` (D26 acceptance mapping); product SPA head `872074bf`
- Required-CI content head: `b147711a`
- Lease: closed `lease/personal/P7-T05/d14-rendered-review`
- Change class: implementation + handbook + plan closure
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, or Agent-benefit promotion. Local rendered reviews are browser observations, not a product Gate. Web UI remains non-blocking for Linux 1.0.

## Acceptance mapping

| Formal acceptance | Evidence |
|---|---|
| Clients gate / SPA in this repo `clients/pc/web` | ADR-0054 path; D11–D25 implementation on that tree; `pnpm test` **306/306** and production build **pass** at `872074bf` |
| localhost Control Plane: LLM API key → approved SecretStore | D08/D09 (merged kernel PR [#262](https://github.com/agentkernel/cognitive-os/pull/262), clients PR [cognitiveos-clients#2](https://github.com/agentkernel/cognitiveos-clients/pull/2)); D11 Providers key handoff rendered review (`leakedInDom=false`, `leakedInUrl=false`, `secretRefShown=false`, field cleared) |
| Bind Agent to fixed account+provider+model | D08/D09 live bind + CAS; D11 Providers CAS exact-tuple preview and 409 stale → reread → fresh preview → reconfirm |
| Task preview / admit / watch | D08/D09 live admit; D13 `#/work/new` governed chain; D23 Work detail Run `GET /task/watch` SSE |
| Control Plane redesign waves W1–W12 | D11–D24 exclusive Chrome reviews (W5 63/63 through W12 15/15, 107/107); D25 retirement 15/15, 115/115 |
| HTTP cancel / class-C Agent lifecycle | **`not-run`** (no typed HTTP); UI does not invent routes |
| Live linux-002 `/ui/` re-drive of D25 | **`not-run`** (fixture review only at `872074bf`) |
| Draft PR → required CI → merge | PR [#274](https://github.com/agentkernel/cognitive-os/pull/274) merged at `main@5996afbb`; required CI run [32942980183](https://github.com/agentkernel/cognitive-os/actions/runs/32942980183) **SUCCESS** at `b147711a` |

## Redesign slice evidence (D14–D25 on PR #274)

| Slice | Revision | Tests / build | Rendered review |
|---|---|---|---|
| D14 W5 Work detail | `main@b77a0243` | 192/192 | 63/63 |
| D15 W6 Agents | `633215d9` | 208/208 | 15/15, 213/213 |
| D16 W7 Resources hub | `6c33c94f` | 221/221 | 12/12, 215/215 |
| D17 W7 Memory | `32119056` | 230/230 | 12/12, 127/127 |
| D18 W7 Skills | `27e454a3` | 241/241 | 12/12, 135/135 |
| D19 W7 Tools | `f007f352` | 246/246 | 12/12, 118/118 |
| D20 W8 Activity | `acc5814e` | 267/267 | 15/15, 170/170 |
| D21 W9 System | `32ebbbe9` | 273/273 | 15/15, 154/154 |
| D22 W10 command | `5f4185fd` | 285/285 | 15/15, 156/156 |
| D23 W11 watch | `db599bfd` | 295/295 | 15/15, 188/188 |
| D24 W12 a11y | `b30314f3` | 305/305 | 15/15, 107/107 |
| D25 retire `styles.css` / `#/tasks` | `872074bf` | 306/306; CSS 22.93 kB, JS 436.50 kB | 15/15, 115/115 |

Every exclusive Chrome review used Chrome 151.0.7922.174 over CDP against a clean rebuild, with `overflow=0`, `clipped=0`, `consoleErrors=0`, and 0 contrast findings on or off screen. Harness artifacts stay outside Git.

D25 bundle SHA-256: JS `0eafc04968c8aa8ee4522fb8e7fc89cd2f1b1a52e58a5e01aecf1c3744e48f05`, CSS `9a6e96d6793800b284d8d3ba61f58b9b036d7a20482c95815ba4162a09adaa31`.

## Validation

| Unit | Environment | Revision | Result |
|---|---|---|---|
| resolve validation route | GitHub Actions | `b147711a` | **pass** |
| verify (ubuntu-latest) | GitHub Actions | `b147711a` | **pass** (3m32s, run `32942980183`) |
| verify (windows-latest) | GitHub Actions | `b147711a` | **pass** (12m42s, run `32942980183`) |
| required-ci | GitHub Actions | `b147711a` | **pass** (run `32942980183`) |
| `check-handbook` / `generate-handbook --check` | local Windows (allowed GNU subset) | `b147711a` | **pass** (57×2; 18 pages) |
| `check:consistency` | local Windows | this worktree | **pass** |
| SPA `pnpm test` / `pnpm build` | local Windows | `872074bf` | **pass** 306/306 |
| Local Windows GNU Rust | `DEV-WIN-GNU-01` | — | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |

## Unique next action

Merged PR [#274](https://github.com/agentkernel/cognitive-os/pull/274) at `main@5996afbb`. Local and remote task branches deleted; local `main` matches `origin/main`. Claim **P7-T06** (RC / docs / support matrix). **P7-T07** stays `blocked` on owner: Windows release artifacts, `B01-W-DESKTOP-001` provisioning, operator for graphical hidden-input credential entry. Do not auto-claim P6.
