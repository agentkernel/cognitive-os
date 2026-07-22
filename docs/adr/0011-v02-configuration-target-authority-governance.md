# ADR-0011: v0.2 Configuration Target Authority Governance

- Status: Proposed for owner review
- Date: 2026-07-22
- Decision owners: repository owner, TARGET consumer owners, security reviewer, and audit reviewer
- Classification: structural docs-only design; no machine registration
- Baseline: `origin/main@88d5374430263c52c7b67e3178dcd752ad984dbc` (PR #51 merge)
- Decision packet: [V02-CA-TARGET-01](../plan/V02-CA-TARGET-DESIGN-DECISION.md)

## Context

PR #51 merged the OPS design that classifies `system.configure`,
`gateway.configure`, and `diagnostics.configure` as intended critical-extension
candidates only. OPS deliberately left their target authority, payload,
consumer, readback/verifier, receipt, risk, approval, error, and negotiation
bindings unresolved.

The registered `GovernedObjectHeader` and `ObjectReference` contracts provide
stable identity, version, scope, owner, authority, policy, sensitivity,
retention, lineage, and digest facts. Core and the state/effect standards
provide CAS, fencing, Intent/Effect/Verification, reconciliation, and atomic
Event obligations. No registered machine asset, however, identifies a system,
gateway, or diagnostics configuration target or fixes its body, consumer,
readback, verifier, or receipt. A URI, open JSON object, private store row,
`OperationSummary`, API route, CLI spelling, or caller value cannot supply the
missing authority semantics.

## Decision

1. Reuse the governed-object authority model as the mandatory outer governance
   model for any future configuration target: a stable target identity and
   version, a strong authority reference, explicit scope/policy/sensitivity/
   retention, and a pinned content digest.
2. Do not treat an existing governed-object type, lifecycle object, proposal,
   Intent, Effect, Event, catalog projection, private DTO, or store row as a
   configuration target merely because it carries some reusable fields.
3. Do not create one opaque `ConfigurationTarget` whose meaning is supplied by
   an open body, URI, operation name, plugin, consumer, or caller. Such a type
   would be a namespace for unreviewed authority expansion.
4. A future structural machine-registration proposal may introduce a bounded
   configuration-target profile family or a general authority-managed
   configuration state domain only after each operation independently closes:
   target identity, authority, CAS object, writer epoch, payload, consumer,
   readback, verifier, receipt, risk/approval, errors, audit, and migration.
5. The three operation profiles need not share one semantic body. Common
   governance fields may be shared; system, gateway, and diagnostics payloads,
   consumers, readback criteria, and risk policies remain operation-specific.
6. Any new configuration state domain is a general authority-managed state
   domain, not a sixth execution lifecycle. It requires a later independent
   machine-registration PR, versioned migration, finite compatibility, and
   owner review.
7. Until the operation-specific facts are uniquely fixed, all three candidates
   remain `blocked`; no target profile, object family, state domain, descriptor,
   extension member, or machine membership is approved by this ADR.

## Alternatives considered

### Reuse `ManagementActionProposal` as the target

Rejected. `target_refs` are URI references and `parameters` is open JSON. The
proposal records a requested action; it is not authoritative target state,
readback, a consumer contract, or a committed receipt.

### Reuse Intent, Effect, VerificationReport, or Event as the target

Rejected. They provide process and evidence relationships. Intent target and
postcondition expressions are open, Effect receipt is a URI, Event payload is
open, and VerificationReport has no target-specific readback profile. None
defines the configuration authority body.

### Reuse one existing governed object without a target-specific profile

Rejected. A governed header proves governance metadata, not the semantics of
the object body or which subsystem consumes it. No existing registered object
uniquely means system configuration, a gateway instance/configuration, or a
diagnostics policy/sink/collection profile.

### Introduce one generic configuration object now

Rejected. The three operations have materially different consumer, sensitivity,
retention, export, external-apply, verification, and failure obligations. An
opaque generic object would hide rather than close those differences.

### Introduce three complete object profiles in this docs-only batch

Not selected. Current source facts do not uniquely determine the system target,
gateway target granularity, diagnostics target kind, payload fields, real
consumers, or readback APIs. Inventing them would turn design preference into
machine authority. The TARGET decision records bounded alternatives and the
facts required for a later structural registration proposal.

## Consequences

- The governed-object header, strong references, CAS semantics, fencing,
  Intent/Effect/Verification flow, and fail-closed store behavior remain
  reusable inputs to later design.
- A future target contract must add operation-specific machine facts; those
  facts cannot be inferred from the reused governance skeleton.
- `system.configure`, `gateway.configure`, and `diagnostics.configure` remain
  intended critical-extension candidates and remain blocked.
- TARGET design does not register a schema, requirement, error, state domain,
  transition, vector, descriptor, extension, digest, generated binding,
  implementation, evidence artifact, or Profile claim.
- D-016 remains open. D-022 remains a blocker. CA-1 through CA-8 remain blocked.

## Rollback and failure strategy

If owner review rejects this structural direction, no machine asset is changed.
The safe state is to retain the OPS classification, keep the three candidates
blocked, preserve v0.1 assets and vector expectations, and require a new TARGET
decision before any target machine registration or implementation.
