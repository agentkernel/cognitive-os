# P8-T16 — dsh Path B stale `DAEMON_BEARER` after daemon restart

Running validation report (`TEST-REPORT-INCREMENTAL-01`). Claim ceiling
`hypothesis`. No Gate, release, Profile, B01, EVAL, or Agent-benefit promotion.

- Task: `P8-T16`
- Lease: `lease/personal/P8-T16/dsh-stale-bearer`
- Branch: `personal/P8-T16-dsh-stale-bearer`
- Bug: [dsh-pathb-stale-daemon-bearer-after-daemon-restart.md](../bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md)

## Validation log

| Unit | Result | Notes |
|---|---|---|
| adapter node tests `dsh-web-preflight.test.mjs` | **pass** 7/7 | Worktree `D:\agent-kernel-p8-t16`; `node --test personal/packages/dsh-akp-adapter/scripts/dsh-web-preflight.test.mjs`. Includes 401 `LOCAL_SESSION_UNAUTHORIZED`/`LOCAL_SESSION_EXPIRED` as `stale_session`, unreachable wait, Cos-exit remint, overlay 401 not unbound. |
| `check-handbook` | **pass** | 57 documents × 2 locales |
| `generate-handbook --check` | **pass** | 18 pages byte-identical |
| `check:consistency` | **pass** | 275 requirements; Personal plan/Gates/leases verified |
| `git diff --check` | **pass** | worktree `D:\agent-kernel-p8-t16` |
| linux-002 live remint | not-run | `P8-T16/D03`; requires pushed exact revision |
| required CI | not-run | `P8-T16/D04` |
| Local Windows GNU Rust | not-run | `RUST-LINK-DEV-WIN-GNU-01` |
