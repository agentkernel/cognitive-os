# P2-T02 D04 Pi sidecar parity closure handoff

- Date: 2026-08-03
- Task and slice: `P2-T02/D04`
- Change class: implementation-only
- Lease: `lease/personal/P2-T02/pi-sidecar-parity`, closed in PR #144 closure
  delivery
- Status: slice `done`; P2-T02 acceptance assessment remains separate

## Validated checkpoint

`ed01c271ee3b8f5dee46d3c230b58ec4e3b4d2e5` on
`lane/run-p2-t02-pi-sidecar-parity`.

## Delivered boundary

`@cognitiveos/pi-cognitiveos` is still a Pi-local, non-authority daemon
client. It now holds independent bearer caches for:

- management-only private Resource projection and snapshot-first Resource
  watch; and
- Task-only snapshot-first Task watch.

The client validates session channel echoing, resource projection shape,
non-authority facts, snapshot-first streams, and non-negative safe-integer
cursor values. Read-session reminting is bounded to one retry after `401`.
It sends no mutation request and does not write SQLite, mint capabilities,
dispatch work, create Effects, verify or complete Tasks.

## Validation

| Check | Result |
|---|---|
| failure-first Pi build before client methods existed | pass: expected TS compile failure |
| `pnpm --filter @cognitiveos/pi-cognitiveos build` | pass |
| `pnpm --filter @cognitiveos/pi-cognitiveos test` | pass |
| `git diff --check` | pass |
| exact Linux daemon-side Resource projection/watch + Task watch | pass, 1/1 |
| required CI Ubuntu | pass |
| required CI Windows | pass |
| local Windows GNU Rust build/test/Clippy | not-run; prohibited by `RUST-LINK-DEV-WIN-GNU-01` |

Exact Linux evidence used the immutable source archive for `ed01c27`, built
the Personal daemon on native Linux, then called its loopback endpoint using
the Pi sidecar artifact from that same revision. The test used isolated
temporary XDG roots and no Provider, Pi installation, secret, service-manager,
privilege, release, or B01 operation.

## Remaining work

P2-T02 is eligible for an honest formal-acceptance assessment across D01-D04.
B02/B04/B05/B12, release, and Profile remain `not-run` or incomplete and no
claim advances from this slice.
