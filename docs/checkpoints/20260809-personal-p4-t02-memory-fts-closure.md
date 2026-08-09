<!--
Task: P4-T02
Classification: implementation-only
Status: done
-->

# P4-T02 Memory FTS5 retrieval closure

## Delivered

Revision `aca44d13ba2ee97f758dc36ffc96066dc43af722` closes the daemon-private
SQLite FTS5 Memory retrieval baseline:

- migration v17 creates a disposable `memory_search_fts` derived index;
- daemon-owned admission atomically indexes only an admitted Memory object's
  current bound Context-source text;
- candidate discovery first joins authoritative admitted-decision, scope,
  purpose, retention, source digest, provenance, and resource-scope facts,
  then applies FTS ranking;
- retrieval returns metadata-only Memory/source bindings, never source body
  text or an authorization grant; and
- daemon-owned rebuild removes stale index rows and recreates them from the
  current authoritative Memory and Context-source records.

Focused negative coverage rejects scope and retention leakage, orphaned/stale
derived rows, immutable-metadata conflicts, and confirms rebuild preserves the
authoritative Memory object. Memory lifecycle/forget, public API/projection,
Context/Task consumption, embedding/vector/graph, B08, release, and Profile
claims remain outside P4-T02.

## Validation

- `cargo fmt --all`: passed.
- `git diff --check`: passed.
- `pnpm run check:consistency`: passed.
- Exact native Linux at `aca44d13ba2ee97f758dc36ffc96066dc43af722`:
  - `cargo test -p cognitive-store --test p1_t01_layout_migrations`: 8/8;
  - `cargo test -p cognitive-store --test p4_t02_memory_search`: 4/4;
  - `cargo test -p kernel-server personal::memory_admission`: passed; and
  - focused store/kernel-server Clippy validation passed.
- Required Ubuntu CI: passed at the same revision.
- Required Windows CI: passed at the same revision.

## Acceptance mapping

| Formal acceptance | Implementation and evidence |
|---|---|
| Metadata filter before retrieval/ranking | Authority-filtered SQL CTE joins admitted Memory and current Context-source facts before `memory_search_fts MATCH`; focused scope/purpose/retention regression passed. |
| Unauthorized/stale/conflict negatives | Orphaned FTS rows cannot produce candidates; immutable candidate metadata rejects updates; query returns metadata only. |
| Rebuild safety | Rebuild clears and reconstructs derived FTS rows without modifying the authoritative Memory object; focused regression passed. |
| Versioned SQLite baseline | Migration v17 and P1 layout migration coverage prove empty, replay, and prior-version upgrade paths. |

## Closure

Task branch: `personal/P4-T02-memory-fts`.

Draft PR: #173, to be marked ready and merged only after this closure record,
formal-plan/progress reconciliation, and lease closure are committed.
