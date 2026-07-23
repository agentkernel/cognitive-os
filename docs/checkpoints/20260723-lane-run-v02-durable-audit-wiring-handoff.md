# Lane-RUN v0.2 durable AUDIT wiring handoff

- Date: 2026-07-23
- Branch: `lane/run-v02-durable-audit-wiring`
- Base: `main@03af1cb` (PR #62 merge)
- Classification: Ordinary Core internal candidate implementation; no machine registration
- Result: **lightweight durable AUDIT adapter and existing external inspect wiring provided; tests executed**

## Implementation

- `FileManagementAuditLog<C: Clock>` stores canonical JSONL frames with an
  exclusive OS writer lock, a durably advanced per-open writer epoch, one
  contiguous decision sequence, duplicate-record rejection, startup readback
  validation, and `sync_all` before returning a receipt.
- The journal persists only the safe digested `PrivilegedReadDecision`; raw
  lifecycle selectors and protected object identities are absent.
- `admin-cli inspect` now always calls `inspect_with_audit`. Its default journal
  is `<store>.management-audit.jsonl`; `--audit <journal>` is an override.
- Audit open/lock/corruption/commit failure maps to registered
  `STATE_STORE_UNAVAILABLE` and releases no inspect stdout.
- Receipt ordering now compares normalized timestamp precision, fixing the
  no-fraction versus fractional-second lexical-order edge case found by the
  real CLI test.

## Test-first and verification evidence

- The new durable-adapter test first failed because `FileManagementAuditLog`
  did not exist.
- The first real CLI run then exposed the timestamp-order bug; the deterministic
  fractional-second regression test and real process path now pass.
- `cargo test -p cognitive-management -p admin-cli`: cognitive-management 17
  tests pass; admin-cli 11 tests pass.
- Strict clippy for both packages: pass.
- Formatting, static consistency, matrix check and final diff checks are the
  remaining pre-PR gates for this branch.

## Boundaries and next entry

This is the lightweight Ordinary Core adapter, not High-Assurance independent
AUDIT deployment, detached signing, external notarization or adversarial
filesystem tamper proofing. No schema/error/registry/vector/generated binding
changed; matrix and conformance pins remain unchanged. The repository currently
has no external kernel-server `status.inspect` endpoint; any future HTTP/API
entry must call `inspect_with_audit` and may not use the compatibility/internal
`inspect` path. Final independent review of exact candidate bytes remains
deferred until immediately before registration.
