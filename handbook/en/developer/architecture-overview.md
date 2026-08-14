---
doc_id: dev.architecture-overview
locale: en
kind: concept
audience: [developer]
status: partial
generated: false
sources:
  - path: docs/architecture/personal/system-architecture.md
  - path: apps/kernel-server/src/personal/mod.rs
  - path: crates/cognitive-kernel/src/lib.rs
    symbols: ["KERNEL_PORTS"]
fingerprint: "sha256:00d909e7c077d98acb68f2a25b63784b63305ca4d6b60431df451f6d5f74f776"
non_claims:
  - The target architecture documents intent; this page tracks which pieces exist. Neither is Gate/release evidence.
---

# Architecture overview

## The invariant everything hangs on

> A probabilistic component may produce a candidate or observation. Only the
> deterministic Rust daemon may authorize, apply CAS, advance lifecycle state,
> grant budget or capability, persist and reconcile an Effect, or accept a Task.

Concretely: every authority mutation flows through `cognitive-kernel`'s
`TransitionEngine` ten-step gate into `cognitive-store`'s single-writer SQLite WAL
adapter, checked against embedded, digest-pinned transition tables from
`cognitive-domain` and canonical digests from `cognitive-contracts`.

## Target layers vs current composition

The target design ([`system-architecture.md`](../../../docs/architecture/personal/system-architecture.md))
draws five layers: experience clients → Task/Resource application services → six
domain services → sidecar/scheduler/executor/verifier execution layer → SQLite +
artifact + secret + Linux ports.

What exists today:

- **Experience**: `cognitive` CLI, Pi extension, TypeScript SDK/Shell library —
  all real clients over loopback HTTP with channel-bound bearers. `implemented`.
- **Application services**: `TaskApi` (record/interpret/preview/admit + watch) and
  the private six-family resource projection + Memory/Skill routes. `implemented`
  for those operations; `control`/`query_intent` and a generic
  `ResourceApplicationService` vocabulary (bind/unbind/enable/…) are not exposed.
- **Domain services**: authority stores + kernel services exist for all six
  families (see the per-domain pages). `implemented` at the storage/service level.
- **Execution layer**: every primitive exists (scheduler CAS leases, sealed
  Context, candidate admission, tool executors, verifier seam, recovery), but the
  autonomous loop that connects them is not wired — `partial`; see
  [execution-chain status](./execution-chain-status.md).
- **Platform ports**: SQLite WAL (two databases), filesystem artifact CAS, Linux
  Secret Service, systemd user service. `implemented`.

## Design decisions that explain surprises

- One canonical service + fixed loopback port 48181 (ADR-0034) — earlier UDS and
  two-unit promotion designs (ADR-0019/0032/0033) survive as text but are
  superseded for the product path.
- Pi is deliberately two roles: shell host (client) and managed agent (governed
  runtime). Identities never merge (ADR-0035).
- Six families, no universal `Resource` table (ADR-0037); per-agent sidecar as the
  integration boundary (ADR-0038).
- MVP-first authorization: owner-local, single-principal, task-scoped; RBAC and
  approval chains are explicitly deferred.
