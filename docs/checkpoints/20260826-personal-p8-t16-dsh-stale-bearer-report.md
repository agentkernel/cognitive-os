# P8-T16 — dsh Path B stale `DAEMON_BEARER` after daemon restart

Running validation report (`TEST-REPORT-INCREMENTAL-01`). Claim ceiling
`hypothesis`. No Gate, release, Profile, B01, EVAL, or Agent-benefit promotion.

- Task: `P8-T16`
- Lease: `lease/personal/P8-T16/dsh-stale-bearer` (closed with D04)
- Branch: `personal/P8-T16-dsh-stale-bearer`
- PR: [#275](https://github.com/agentkernel/cognitive-os/pull/275)
- Bug: [dsh-pathb-stale-daemon-bearer-after-daemon-restart.md](../bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md)

## Validation log

| Unit | Result | Notes |
|---|---|---|
| adapter node tests `dsh-web-preflight.test.mjs` | **pass** 7/7 | Worktree `D:\agent-kernel-p8-t16`; `node --test personal/packages/dsh-akp-adapter/scripts/dsh-web-preflight.test.mjs`. Includes 401 `LOCAL_SESSION_UNAUTHORIZED`/`LOCAL_SESSION_EXPIRED` as `stale_session`, unreachable wait, Cos-exit remint, overlay 401 not unbound. |
| `check-handbook` | **pass** | 57 documents × 2 locales |
| `generate-handbook --check` | **pass** | 18 pages byte-identical |
| `check:consistency` | **pass** | 275 requirements; Personal plan/Gates/leases verified |
| `git diff --check` | **pass** | worktree `D:\agent-kernel-p8-t16` |
| linux-002 live remint | **pass** | Exact adapter scripts `68355417` (`dsh-real-process.mjs` sha256 `f366002177147fab…`, `dsh-web-preflight.mjs` sha256 `df00533f252a57fc…`) on identity-confirmed `B01-Desktop-Linux-002`. Helper pid **1352747** survived daemon replace; remint log after 2 s; `.credentials.yaml` mtime advanced; Cos pid 1352760→1352902 `ACTIVE`/`process_alive=true`; Path B `POST /provider/v1/dsh/chat/completions` **200** `LongCat-2.0` before and after, without operator `dsh web`/`dsh apply`. Bearer shape `sess`; token not logged. EVAL listeners and hung P8-T10 helper **430838** untouched. |
| required CI | **pass** | run [32927209560](https://github.com/agentkernel/cognitive-os/actions/runs/32927209560) at `68355417`: ubuntu, windows, required-ci, resolve validation route |
| Local Windows GNU Rust | not-run | `RUST-LINK-DEV-WIN-GNU-01` |

## Acceptance mapping

| Formal acceptance | Evidence |
|---|---|
| Helper remints Path B `DAEMON_BEARER` after daemon restart while `cognitive dsh web` stays running | D03 linux-002: helper pid unchanged; remint stderr; credentials rewritten; Cos reloaded ACTIVE |
| 401 is not an invalid Provider API key and not an unbound overlay | D01 7/7; overlay/bindings 401 throws `stale_session`; Path B after remint is LongCat-2.0 |
| Sessions stay process-local | remint from `local-bootstrap.secret`; no session persistence added |
| Draft PR → required CI → merge | PR #275; CI `32927209560` at `68355417` |

## Closure

Task `P8-T16` is accepted at claim ceiling `hypothesis`. Ready/merge/lease/branch/main
follows this report. No Gate, release, Profile, B01, EVAL, or Agent-benefit promotion.
