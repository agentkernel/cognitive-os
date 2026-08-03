# P2-T02 D03 deterministic CLI parity closure handoff

- Date: 2026-08-03
- Task and slice: `P2-T02/D03`
- Change class: implementation-only
- Lease: `lease/personal/P2-T02/deterministic-cli-parity`, closed in PR #143
  closure delivery
- Status: slice `done`; parent `P2-T02` remains `in-progress`

## Validated checkpoint

`af2f6c9b261efe5014ac829e872653244fb22ef2` on
`lane/run-p2-t02-cli-parity`.

## Delivered boundary

The deterministic `cognitive` CLI now invokes the same daemon read/watch
surfaces introduced by P2-T02 D01/D02:

- `resource get` and `resource watch` use a management-scoped session;
- `task watch` uses a separately minted Task-scoped session;
- resource and Task cursor parameters remain path and channel scoped;
- the delivered CLI commands are read-only, so no admission or other mutation
  is retried or replayed after transport ambiguity.

The CLI remains a daemon client: it does not open authority SQLite tables,
mint governance facts, dispatch work, advance Effects, verify Tasks, or claim
Task completion.

## Validation

| Check | Result |
|---|---|
| `cargo fmt --all` | pass |
| `git diff --check` | pass |
| exact Linux `cargo test -p admin-cli --test p2_t02_cli_parity` | pass, 1/1 |
| required CI Ubuntu | pass |
| required CI Windows | pass |
| local Windows GNU Rust build/test/Clippy | not-run; prohibited by `RUST-LINK-DEV-WIN-GNU-01` |

Linux validation used a disposable source archive made from the exact local Git
object at `/tmp/cognitiveos-p2-t02-d03-af2f6c9`; no Provider, secret,
service-manager, privilege, release, or B01 operation was used.

## Remaining work

P2-T02/D04 must create the Pi Shell private sidecar parity path with the same
daemon result semantics and prove that Pi remains non-authority. P2-T02,
B02/B04/B05/B12, release, and Profile remain incomplete or not-run.
