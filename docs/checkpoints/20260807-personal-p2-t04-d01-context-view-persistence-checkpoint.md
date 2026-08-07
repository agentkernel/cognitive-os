# P2-T04/D01 ContextView persistence checkpoint

- **Date:** 2026-08-07
- **Classification:** `implementation-only`
- **Slice:** `P2-T04/D01`
- **Branch:** `lane/personal-p2-t04-context-view`
- **Lease:** `lease/personal/P2-T04/private-worker-composition`
- **Status:** `in-progress`; the persistence checkpoint is validated, while
  the revoked-source integration negative remains required before slice closure

## Delivered

The daemon now persists the exact resolved `ContextView` immediately after
daemon-owned authorization, metadata filtering, body loading, ranking, and
deterministic rendering, and before invoking the private Pi candidate
proposer. The persisted view is immutable, sealed, request-bound, and carries
only governed strong references, source metadata, costs, rejection/loss facts,
and the renderer verification digest. Source bodies are not copied into the
durable ContextView row.

This preserves the authority order:

```text
durable ContextRequest -> current authorization/revocation -> resolve ->
durable ContextView -> bounded Pi candidate proposal -> daemon admission
```

The ContextView persistence step has no worker authority, Effect, budget debit,
progress, evidence, verification, acceptance, or Task-completion semantics.

Follow-up immutable `8f2d6e53e37cacdc3572305718df6bb29be22bb3` hardens the
same boundary: each source body load reconstructs authorization from the
latest durable facts and revocation epoch, and ContextView persistence carries
the representation already preserved by the authorized resolver rather than
reloading a source body. A revocation observed after metadata discovery thus
denies the body before it can enter ranking, rendering, durable view emission,
or Pi transport.

## Validation evidence

- Local focused Rust validation: `not-run`; the Windows GNU host cannot
  compile/link Rust feature work.
- Exact native Linux `DEV-LINUX-NATIVE-01`: **passed** on immutable
  `d3c6181d6e5bc892871aad0896006443b233ce61` in the disposable Git clone
  `/home/wuz/cognitiveos-validation-p2-t04-d3c6181`:
  `cargo test -p kernel-server` completed with exit 0.
- Exact native Linux `DEV-LINUX-NATIVE-01`: **passed** on immutable
  `8f2d6e53e37cacdc3572305718df6bb29be22bb3`, transferred as an incremental
  Git bundle only after the commit was pushed and checked out detached in the
  same disposable clone: `cargo test -p kernel-server` completed with exit 0.
- Required CI: both Ubuntu jobs passed for `8f2d6e5`; the Windows jobs remain
  pending at this checkpoint. No pending check is claimed as passing.

## Remaining negative path

- Required next negative: add a deterministic scheduler/Pi integration test
  that revokes the source between initial durable authorization observation
  and candidate proposal, then directly proves the revoked source cannot
  reach body loading, ranking, or Pi. The per-body revalidation mechanism is
  implemented, but this dedicated race regression must be added within the
  active scheduler/Pi lease before claiming the P3 Context integration exit.
- B03, release, Profile, and Task completion remain `not-run`/unclaimed.
