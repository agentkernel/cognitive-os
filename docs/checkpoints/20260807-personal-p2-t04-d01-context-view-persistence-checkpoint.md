# P2-T04/D01 ContextView persistence checkpoint

- **Date:** 2026-08-07
- **Classification:** `closure`
- **Slice:** `P2-T04/D01`
- **Branch:** `lane/personal-p2-t04-context-view`
- **Lease:** `lease/personal/P2-T04/private-worker-composition`
- **Status:** `done`; P2-T04/D01 acceptance is closed at immutable
  `a8ef5c00654e1c05a4c30beb193b9c026654c2f1`

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

## Closure evidence

The final scheduler/Pi negative matrix is now covered on the real SQLite
authority path:

- `revocation_after_metadata_discovery_blocks_body_ranking_and_private_pi`
  appends revocation epoch 2 after metadata-only discovery. It proves the
  revoked source fails before body materialization and cannot reach ranking,
  ContextView persistence, private Pi, or candidate persistence.
- `missing_required_context_blocks_private_pi_and_candidate_admission` proves
  deterministic Context incompleteness cannot reach private Pi or candidate
  admission.
- `duplicate_candidate_retry_does_not_reinvoke_private_pi` proves an existing
  daemon-owned candidate identity resumes daemon-only admission rather than
  asking Pi for a second proposal or replacing the immutable candidate.
- Existing real-store `p2_t03_worker_authorization` coverage proves atomic
  candidate admission, budget debit, one-time WIA consumption, and rejection
  of stale or replaced scheduler lease bindings without consuming authority.
- `daemon_candidate_protocol` proves the Pi boundary accepts only the bounded
  candidate shape and rejects authority-shaped output and attempted Tool use.

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
- Exact native Linux `DEV-LINUX-NATIVE-01`: **passed** on final immutable
  `a8ef5c00654e1c05a4c30beb193b9c026654c2f1`, checked out detached in
  `/home/wuz/cognitiveos-validation-p2-t04-d3c6181`:
  - `cargo test -p kernel-server` (68 passed);
  - `cargo test -p cognitive-store --test p2_t03_worker_authorization`
    (18 passed);
  - `cargo test -p pi-agent-adapter --test daemon_candidate_protocol`
    (10 passed).
- Required CI: both Ubuntu and both Windows jobs passed for final immutable
  `a8ef5c00654e1c05a4c30beb193b9c026654c2f1`. Windows verified workspace
  build/test/Clippy/fmt, code-generation drift, consistency, and conformance.

## Explicit non-claims

P2-T04/D01 closes private candidate composition only. It does not perform
Tool execution, create worker progress/evidence, verify an Effect, accept or
complete a Task, satisfy B03, create a release, or establish a Profile claim.
