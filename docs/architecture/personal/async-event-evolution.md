# Async Event Evolution (design)

- Status: informative Personal architecture design
- Related: P7-T04 stage timing, P9-T01

## 1. Decision gate

Use P7-T04/D02 governed-path stage timing to separate **governance tax** from
**implementation tax**. If connection/open/lock contention dominates p95, stage a
migration of HTTP/watch/sidecar streaming toward an async runtime while keeping
authority SQLite single-writer semantics. Otherwise record a conservative
no-migration conclusion with evidence.

## 2. Non-negotiables

- Single authority writer and persist-before-dispatch remain.
- Async I/O must not invent a second Task/Effect writer.
- Fail-closed behavior on fence/budget/secret boundaries is preserved.

## 3. Non-claims

No async rewrite is authorized by this document alone. P9-T01 owns the measured
decision.
