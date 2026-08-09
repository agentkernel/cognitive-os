<!--
Task: P4-T01
Classification: implementation-only
Status: done
-->

# P4-T01 Memory admission closure

## Delivered

Revision `e4eb38ad9aaba13f04fb51657dfdc884af66cdc5` closes the daemon-private
Memory proposal-to-admission boundary:

- deterministic source-bound policy checks for digest, provenance, scope,
  retention, and current source facts;
- append-only SQLite migration v16 for candidates, decisions, and objects;
- atomic source revalidation and candidate/decision/object persistence;
- no object creation for rejected, stale, promoted, conflicting, or malformed
  proposals;
- daemon-private service re-derivation of policy, preventing producer-selected
  admission outcomes; and
- failure-first tests for direct-admit, source mismatch, retention/scope
  rejection, conflict/replay, and no-partial-object behavior.

Later FTS/retrieval, lifecycle/forget, public projection/API, B08, Gate,
release, and Profile claims remain outside this task.

## Validation

- `cargo fmt --all`: passed.
- `pnpm run check:consistency`: passed.
- `git diff --check`: passed.
- Exact native Linux at `e4eb38ad9aaba13f04fb51657dfdc884af66cdc5`:
  focused kernel admission tests, Memory-store tests, migration tests,
  kernel-server service test, and Clippy passed.
- Required Ubuntu CI: passed.
- Required Windows CI: passed.

## Closure

Task branch: `personal/P4-T01-memory-admission`.

Draft PR: #172.

The task lease is closed in the ownership ledger. No subsequent formal task
was selected in this closure.

