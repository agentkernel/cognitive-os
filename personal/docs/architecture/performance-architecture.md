# Performance Architecture (design)

- Status: informative Personal architecture design
- Related: P7-T04 regression floor, P9-T01..T03

## 1. Layers

1. **Module microbenchmarks** — deterministic floors (P7-T04).
2. **Governed-path stage timing** — distinguish governance vs implementation tax.
3. **Campaign observations (B06/B07)** — non-claim benefit evidence.
4. **Structure evolution** — god-file splits and long-lived store (P9-T02/T03).

## 2. Known structure/performance debts (candidates, not current mutex)

- Oversized modules (`scheduler_authority`, `sqlite`, `tool_executor`).
- Per-request `SqliteAuthorityStore::open` vs long-lived single-writer store.
- Personal vertical logic concentration in `kernel-server` composition root.

## 3. Rules

- Optimize with before/after floors; do not weaken negatives for speed.
- Windows GNU remains a non-linking host; measure on supported Linux/CI.
- Performance wins never become Gate/release/Profile claims without their own
  campaign rules.

## 4. Non-claims

Design and candidate backlog only.
