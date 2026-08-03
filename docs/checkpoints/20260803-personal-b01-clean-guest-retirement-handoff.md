# B01 clean guest retirement handoff

- Date: 2026-08-03
- Task: P1-T09 install-to-first-conversation route
- Change class: corrective environment-governance update
- Closed lease: `lease/personal/P1-T09/b01-clean-guest-retirement`

## Decision

The product owner selected `B01-Desktop-Linux-002` as the sole active KVM
guest for the B01 first-install/first-conversation campaign.

`B01-Clean-Linux-001` is retired from active environment selection. It remains
only as historical failed-qualification evidence: its Ubuntu Server/headless
reset could not provide the product-compatible persistent default/login Secret
Service collection required before an attempt can start. It never entered the
B01 denominator and cannot substitute for a future B01 attempt.

## Environment preflight

The authorized non-interactive, no-secret SSH preflight to `wuz@192.168.1.2`
confirmed native Linux host `hal9000`, kernel `6.8.0-83-generic`, systemd 249,
and system-libvirt 8.0.0. It confirmed both historical Clean and active Desktop
domain definitions and their preregistered baseline snapshots are visible.

The read-only preflight observed the Clean domain as `paused` and the Desktop
domain as `shut off`. It did not start, stop, resume, reset, snapshot, install
on, deploy to, or log into either guest. The observed state is not campaign
evidence and does not alter the B01 ledger.

## Operating boundary

- Future B01 attempts use only `B01-Desktop-Linux-002` and its preregistered
  `b01-platform-qualified-baseline` through an active B01 lease and procedure.
- Ordinary development continues to use disposable exact-revision worktrees on
  `DEV-LINUX-NATIVE-01`, not either B01 guest.
- This retirement is a documentation and selection decision, not VM deletion.
  Any lifecycle change to `B01-Clean-Linux-001`, including resume, reset or
  deletion, requires a separately explicit infrastructure authorization.

## Checks

| Check | Result |
|---|---|
| no-secret SSH and system-libvirt enumeration | pass |
| guest lifecycle mutation | not-run; prohibited by this lease |
| B01 campaign attempt | not-run; no reset checkpoint crossed |
| local Rust linking validation | not-run; prohibited on `DEV-WIN-GNU-01` |
| `pnpm run check:consistency` | pass |
| `node --test tools/test/check.test.mjs` | pass (5/5) |
| `git diff --check` and `git diff --cached --check` | pass |

## Non-claims

This change does not modify the formal B01 fixed denominator, its two recorded
outcomes, task acceptance, Gate status, release status, Profile status, guest
image, baseline snapshot, or any product implementation.
