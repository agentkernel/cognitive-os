# State and Transition Contract

- Standard ID: `cognitiveos.standard.state-transition-contract/0.1`
- Version: v0.1 Draft
- Machine version: `0.1.0-draft.1`
- Status: `Draft`
- Date: 2026-07-19
- Classification: registered normative machine-contract companion
- Machine assets: `specs/schemas/state-transition-*.schema.json`, `specs/registry/state-domains.yaml`, and `specs/transitions/*.transitions.json`

## 1. Scope and authority

This contract defines deterministic processing rules for governed state transitions. It implements the five orthogonal execution lifecycle state machines described by the architecture and Core while preserving the RFC rule that authority-managed state domains are an open set.

The registered execution lifecycle domains are `agent-execution`, `task`, `loop`, `effect`, and `verification`. They do not close the set of state domains. World, Conversation, Membership, Policy, Knowledge, Memory, safety, and deployment-specific domains MAY be registered without changing the table schema. A domain name therefore MUST remain a string and MUST NOT be represented by a closed schema enum.

Each domain MUST identify one current write authority or an explicit consensus/arbitration protocol. An actor, model, runtime, tool, receipt, verifier, or remote `completed` claim does not acquire transition authority merely by producing a proposal or evidence.

## 2. Registered assets

Every executable transition table MUST validate against `state-transition-table.schema.json` and contain `domain`, `version`, `initial_state`, `states`, `terminal_states`, and `transitions`. Each transition contains `from`, `to`, `reason_codes`, `guards`, and `required_evidence`; transition-specific `metadata` MAY constrain deterministic interpretation.

`version: 0.1` and `status: Draft` identify this initial registration. A consumer MUST pin the canonical table digest used for a decision. A table update that changes a state, transition, reason code, guard, required evidence item, terminal declaration, or critical metadata requires a new table version and digest.

## 3. Transition request and record

A transition request MUST validate against `state-transition-request.schema.json`. It fixes the target domain and object, `from` and `to` states, `expected_version`, proposed `before_version` and `after_version`, structured reason, causation, actor, authority, request time, and transition-table digest.

The authority MUST reject a request when the table digest/version is not pinned, `expected_version` differs from the authoritative current version, before/after versions are invalid, no row matches the state pair and reason, a guard is false or indeterminate, required evidence is absent/stale/unauthorized/digest-mismatched, or the actor/authority binding is invalid.

An accepted transition MUST atomically advance the authoritative version and append an immutable record conforming to `state-transition-record.schema.json`. The record preserves the request, before/after states and versions, actor and deciding authority, reason, causation, evidence, decision time, and pinned table digest. Version mismatch MUST fail as `STATE_CONFLICT`; it MUST NOT use silent last-write-wins.

Illegal transitions MUST fail closed and return the current state/version plus safe available exits. Rejection is not itself a state transition unless the applicable table explicitly contains a transition representing that result.

## 4. Terminal semantics

A state listed in `terminal_states` has no legal outgoing transition in the same table version. Reopening a terminal object requires a new governed object or a later table version with explicit migration semantics; implementations MUST NOT invent an unregistered edge.

- AgentExecution `TERMINATED` is terminal. `SUSPENDED` returns only through readmission, and `QUARANTINED` returns only through reconcile and reauthorization into `RECOVERING`.
- Task completion is distinct from Loop stop, runtime termination, cancellation request, Effect execution, and verifier output. Only acceptance authority may move `CANDIDATE_COMPLETE` to `COMPLETED` using evidence for a fixed post-state.
- Loop uses the complete state set including `WAIT`, `QUARANTINE`, and `RECONCILE`. `QUARANTINE` cannot jump directly to `OBSERVE` or `RESOLVE`.
- Effect `OUTCOME_UNKNOWN` is neither success nor failure and has no direct path to `VERIFIED` or `COMMITTED`. Reconciliation result is transition metadata and a guard input with values `executed`, `not_executed`, or `still_unknown`. Only `executed` may proceed toward verification; `not_executed` terminates as `NOT_EXECUTED`; `still_unknown` must remain unresolved through quarantine or independently authorized compensation.
- Verification is bound to a fixed subject, criteria, verifier version, evidence set, and post-state version. `PASSED` remains valid only while that binding is current. A later fixed-post-state change moves `PASSED` to terminal `EXPIRED`; an expired result cannot support Task completion or Effect commit.

## 5. Deterministic consumption

Business code MUST consume the registered transition tables through a shared transition API or generated bindings. It MUST NOT scatter ad hoc `if`, `switch`, SQL predicate, UI-only, or model-authored state legality checks across services. The shared mechanism MUST perform table lookup, reason matching, guard evaluation, evidence validation, authority verification, CAS, record creation, and event/audit linkage.

Semantic components MAY propose a target state or reason, but authorization, guard evaluation, version comparison, transition legality, terminal enforcement, and commit MUST be deterministic. UI projections and caches are non-authoritative and MUST display the authority's committed state/version.

## 6. Recovery and audit

Recovery MUST replay committed transition records against the pinned table version and digest. It MUST NOT infer missing transitions from process state, model hidden state, timeout, receipt absence, or a remote success string. A digest mismatch, unknown critical metadata field, missing evidence, or impossible edge is a recovery barrier requiring reconciliation or quarantine.

Every committed record MUST be causally linkable to its request and resulting event/audit entries. Corrections use new records and causation/supersession relationships; committed records are never edited in place.
