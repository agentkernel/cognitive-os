# P8-T14 Provider Control Plane operator usage docs — closure

- Task: `P8-T14` / slice `P8-T14/D01`
- Status: `done`
- Branch: `codex/P8-T14-provider-control-plane-docs` (deleted after merge)
- PR: [#260](https://github.com/agentkernel/cognitive-os/pull/260) **merged** at `main@a2b8ddb3`
- Docs head: `7694f747`
- Lease: closed `lease/personal/P8-T14/provider-control-plane-docs`
- Change class: documentation-only; does not reopen P8-T13
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, provider-quality, Web/desktop panel, or Agent-benefit promotion

## Acceptance mapping

| Formal acceptance | Evidence |
|---|---|
| Bilingual operator handbook (en + zh-CN) for shipped daemon API and `cognitive` CLI | Authored `handbook/en/user/provider-control-plane.md` and `handbook/zh-CN/user/provider-control-plane.md`; related user/reference pointers |
| source-map / manifest routing | `handbook/_meta/source-map.json`, `manifest.json`, `source-coverage.json` in PR #260 |
| docs-sync / consistency | Required CI handbook + consistency steps **pass** on run `32609687956` |
| Draft PR then ready/merge | PR #260 flipped ready after required checks; merged at `main@a2b8ddb3` |

## Validation

| Unit | Environment | Revision | Result |
|---|---|---|---|
| resolve validation route | GitHub Actions | `7694f747` | **pass** |
| verify (ubuntu-latest) | GitHub Actions | `7694f747` | **pass** (3m19s) |
| verify (windows-latest) | GitHub Actions | `7694f747` | **pass** (9m31s) |
| required-ci | GitHub Actions | `7694f747` | **pass** (run `32609687956`) |
| Live Secret Store / Provider / Pi / dsh | — | — | **not-run** |
| Local Windows GNU Rust | `DEV-WIN-GNU-01` | — | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |

## Unique next action

Wait for a fresh owner delivery instruction. Do not auto-claim P6 / P7-T05 / P7-T06 / P7-T07.
