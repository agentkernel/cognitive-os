# ADR-0005: Identifier and Clock Model

- Status: Accepted for v0.1 Draft normative baseline
- Date: 2026-07-19
- Decision owners: CognitiveOS specification maintainers

## Context

CognitiveOS needs stable identifiers for objects, messages, events, executions, correlations, and negotiation epochs. IDs should be locally generatable, operationally sortable, and interoperable across Rust and TypeScript. They are identity, not authorization, and must not depend on process, node, model session, or storage location.

The system also uses time for audit, valid time, leases, deadlines, retries, timeout enforcement, ordering, and performance. Wall clocks can jump. Monotonic clocks are process or boot scoped and cannot be serialized as global time. One undifferentiated clock would make expiry, duration, replay, and audit ambiguous.

## Decision: UUIDv7

CognitiveOS v0.1 freezes newly generated general-purpose stable IDs to UUID version 7 under RFC 9562.

JSON UUIDs MUST use lowercase canonical `8-4-4-4-12` form without braces or `urn:uuid:`. A versioned legacy adapter MAY accept another declared form, but canonical output and signed objects use lowercase canonical UUID.

Generators MUST use cryptographically secure random bits and follow RFC 9562 collision and same-tick monotonicity guidance. On backward wall-clock movement, a generator MUST preserve uniqueness and SHOULD preserve non-decreasing local generation order using the RFC method. It MUST NOT forge unbounded future timestamps.

UUIDv7 timestamp bits are only an ID-generation hint. They MUST NOT be authoritative event time, creation time, valid time, lease time, order proof, freshness proof, or authorization input. Authorization treats the full UUID as opaque.

Externally assigned and protocol-specific identifiers MAY remain in schema-declared namespaces. Migration MUST NOT rewrite historical IDs in place; it creates a target identity with source provenance.

ULID is not the v0.1 general-purpose format. ULID support requires an explicit field or legacy adapter and MUST NOT be silently mixed into a UUIDv7 ID type.

## Decision: separate clock domains

CognitiveOS distinguishes:

1. `wall_clock`: UTC civil time for audit, creation/ingest/event observations, valid-time claims, interoperability, and human correlation. Serialization follows the canonical RFC 3339 UTC `Z` rules.
2. `monotonic_clock`: Non-decreasing local time for elapsed duration, timeout, scheduling, backoff, and local deadline enforcement. A raw reading is meaningful only inside its clock instance and MUST NOT be serialized as global time.
3. `logical_version`: Object versions, event sequences, high-watermarks, contract/negotiation/revocation/fencing epochs. These are authority or protocol ordering values, not time.

Authority changes MUST NOT be ordered solely by wall timestamp or UUIDv7 lexical order. They use the applicable version, sequence, CAS, epoch, fencing, or consensus result.

## Serializable deadlines and expiry

A persisted or cross-boundary deadline MUST carry enough information to survive restart: an absolute wall-clock deadline/expiry and, when duration semantics matter, the granted duration and observation wall time. A live runtime SHOULD also maintain a monotonic deadline.

Before restart, expiration is enforced when either the monotonic deadline or a trusted wall deadline has expired. After restart or transfer, the receiver reconstructs a conservative local monotonic deadline from trusted wall time. It MUST NOT extend an expired lease because the prior monotonic reading is unavailable.

Material wall-clock uncertainty or rollback MUST fail closed for security leases, capabilities, privileged management sessions, and fencing-sensitive operations, or require authority refresh.

Serialized monotonic measurements for diagnostics or evidence MUST use `(clock_id, tick, frequency)` or an equivalent profile type. They are comparable only when clock identity and frequency semantics match and MUST NOT be interpreted as UTC.

Audit wall timestamps SHOULD identify clock source and uncertainty where available. `event_time`, `ingest_time`, `valid_time`, and `observed_at` are distinct and MUST NOT be substituted.

## Ordering and correction

UUIDv7 gives approximate generation order, not global total order. Protocols needing deterministic order MUST define a stable tuple, for example `(logical_sequence, object_id)` or `(event_time, source_id, source_sequence, event_id)`. Timestamp ties MUST NOT use arrival order unless the protocol explicitly makes it authoritative.

Clock skew MUST NOT be hidden by editing immutable event timestamps. Correction creates a new observation or audit record linked by causation or supersession.

## Alternatives considered

### UUIDv4

Rejected as the default because it lacks time locality for indexing and operations. It remains valid where a schema explicitly requires an opaque random ID.

### ULID

Rejected for the baseline because UUIDv7 is standardized by RFC 9562 and uses the broad UUID ecosystem. ULID remains an explicit legacy mapping option.

### Database sequences

Rejected as general IDs because they need a central allocator, leak topology, and prevent disconnected generation. They remain appropriate for authority-owned sequences and high-watermarks.

### Wall clock for all timing

Rejected because clock steps cause premature/late timeout behavior and negative durations.

### Persist raw monotonic timestamps

Rejected because monotonic epochs are not portable across restart, process, node, or device.

## Consequences

Implementations need a conforming UUIDv7 generator and tests for format, collision resistance, same-tick generation, and clock rollback. IDs remain separate from authorization and scope.

Runtime APIs should make wall timestamps, monotonic instants, durations, logical versions, and epochs distinct types. Persistence must never imply raw monotonic portability.

Recovery, AKP continuation, leases, capability expiry, management sessions, and deadlines store reconstructable wall bounds and revalidate authority after restart. Distributed order still depends on epochs, fencing, sequences, or consensus.

## Compliance checks

Tests MUST cover UUIDv7 canonical form, same-tick generation, wall rollback, ID opacity in authorization, wall versus monotonic timeouts, restart reconstruction, non-extension of expired leases, clock uncertainty fail-closed behavior, and rejection of cross-clock monotonic comparison.
