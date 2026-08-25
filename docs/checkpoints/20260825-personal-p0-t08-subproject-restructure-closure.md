# P0-T08 subproject restructure and 1.0.0 finalization — closure

- Task: `P0-T08` / slice `P0-T08/D01`
- Status: `done` (implementation and required CI; merge/tags/branch/main remainder of this session)
- Branch: `structural/subprojects-and-1.0.0`
- PR: [#273](https://github.com/agentkernel/cognitive-os/pull/273)
- Required-CI content head: `366c0bee8d98c32d5d5ca7c1fda5759cb95ef511`
- Lease: closed `lease/personal/P0-T08/subproject-restructure`
- Change class: **structural** (path moves and path-literal rewrites; no product, contract, negative-test, or runtime semantic change)
- Claim ceiling: `hypothesis`
- Non-claims: this restructure creates **no** Gate, release, Profile, B01, EVAL, or Agent-benefit promotion. Annotated tags `core-v1.0.0` / `personal-v1.0.0` record existing MVP Gate outcomes only (ADR-0046..0049). P7-T06 remains the release-evidence task.

## Acceptance mapping

| Formal acceptance | Evidence |
|---|---|
| ADR-0054 four-root tree (`core/` `personal/` `enterprise/` `clients/`) | `git mv` plus subtree import of `cognitiveos-clients` `main@034ef91` into `clients/` |
| core / personal 1.0.0 finalization documents | `core/docs/VERSION-1.0.0.md`, `personal/docs/VERSION-1.0.0.md` |
| enterprise 1.0.0 boundary + activation gate (no implementation) | `enterprise/docs/VERSION-1.0.0.md` |
| Path-bearing toolchain rewrite | root `Cargo.toml` / `pnpm-workspace.yaml` / `.github/workflows/ci.yml`; `tools/src/` SCAN_ROOTS, consistency, handbook, docs-sync-gate; `personal/handbook/_meta/source-map.json` + regenerated pages + fingerprints; `AGENTS.md` / `.cursor/rules/` / `PROJECT-IDENTITY.md` |
| Local Node gates | `check:consistency`, `check-handbook` (57 docs × 2 locales), `generate-handbook --check`, tools suite, `docs-sync-gate`, `pnpm -r build` / `pnpm -r test`, `cargo metadata`, `git diff --check` |
| Required CI on exact HEAD | run [32834054468](https://github.com/agentkernel/cognitive-os/actions/runs/32834054468) **SUCCESS** at `366c0bee` (resolve validation route, verify ubuntu, verify windows, required-ci) |
| Annotated tags + lease/branch/main | remaining operational steps of this closure: merge PR #273 (no force push), tag the merge revision, delete the task branch, fast-forward local `main` |

## Validation

| Unit | Environment | Revision | Result |
|---|---|---|---|
| resolve validation route | GitHub Actions | `366c0bee` | **pass** |
| verify (ubuntu-latest) | GitHub Actions | `366c0bee` | **pass** |
| verify (windows-latest) | GitHub Actions | `366c0bee` | **pass** |
| required-ci | GitHub Actions | `366c0bee` | **pass** (run `32834054468`) |
| `check:consistency` / handbook / tools / `pnpm -r` / `cargo metadata` / `git diff --check` | local Windows (allowed GNU subset) | this worktree | **pass** |
| `cargo test -p cognitive-conformance --test runner_execution` | `DEV-LINUX-NATIVE-01` | `af130c76` | **pass** 13/13 (later CI re-ran the suite at `366c0bee`) |
| Local Windows GNU Rust build/test/Clippy | `DEV-WIN-GNU-01` | — | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |

## Unique next action

Merge PR [#273](https://github.com/agentkernel/cognitive-os/pull/273) once this closure head's required CI is green; create annotated tags `core-v1.0.0` and `personal-v1.0.0` on the merge revision; delete the local and remote task branch; check out `main` and fast-forward. Do **not** auto-claim P6, P7-T05, P7-T06, or P7-T07 — P7-T05 remains paused by owner (D14 rendered review `not-run`).
