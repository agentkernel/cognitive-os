# P2-T03 authority-adapter handoff

- Task: `P2-T03`
- Lease: `lease/personal/P2-T03/authority-adapter`
- Branch: `lane/personal-p2-t03-authority-adapter`
- Date: 2026-08-01

## Delivered boundary

`apps/kernel-server/src/personal/scheduler_authority.rs` reloads scheduler
ceiling inputs only from durable authority records. It rejects empty bindings,
missing or legacy contracts, malformed bound identities, unavailable Loop
states, and unavailable or inconsistent budgets before any scheduler lease or
worker operation. Retry counts fold persisted non-advanced progress facts for
the exact action fingerprint; cost derives from the contract-bound budget
ledger.

Linux evidence actually run:

```text
cargo check -p kernel-server
```

This completed successfully after the first dependency fetch. The compiler
reported dead-code warnings because the daemon-private adapter is deliberately
not connected to dispatch yet.

## Explicit non-delivery

No scheduler lease, worker dispatch, `BoundedHarness` execution, or Loop STOP
transition was added. The Loop STOP table requires durable proof of both
`new_activity_dispatch_disabled` and
`pending_effects_closed_or_quarantined`. The current authority model does not
provide loop-scoped, fenced proof for either guard; asserting their names from
the daemon would be unsafe.

## Required successor

A Lane-CTR/KRN contract slice must first provide a loop-scoped dispatch barrier
checked inside the `AUTHORIZED -> EXECUTING` authority transaction and a
durable, scoped Effect-closure proof. It must also repair the TaskContract
v0.1/v0.2 compatibility classification so historical v0.1 rows are recognized
as legacy/non-dispatchable before v0.2 deserialization. Only then may Lane-RUN
wire ceiling evaluation to a fenced STOP path before scheduler lease creation.

No P2 Gate, release, Profile, task completion, or B01 claim is made.
