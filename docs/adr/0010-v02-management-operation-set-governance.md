# ADR-0010: v0.2 Management Operation-Set Governance

- Status: Proposed for owner review
- Date: 2026-07-22
- Decision owners: repository owner and CognitiveOS maintainers
- Classification: structural design governance; docs-only; no machine registration
- Baseline: `origin/main@41fce4dea27c5bfed515d8dcf8b078200eccb901` (PR #50 merge)
- Decision packet: [V02-CA-OPS-01](../plan/V02-CA-OPS-DESIGN-DECISION.md)

## Context

V02-CA-GOV-00 authorizes v0.2 design of a Management operation set but does not approve members or machine contracts. D-016 records that management operation names are not registered. D-022 records that operation, target, signature, and audit contracts are insufficient for Configuration Authority implementation.

Eight strings occur in fallback/configuration vectors, but reachability and spelling do not define a closed set, complete descriptors, request/result contracts, target authority, or authorization. Treating those strings as a set would silently expand the normative wire and permission surface.

## Decision

1. Use a finite, explicit, closed v0.2 core plus digest-pinned, explicitly negotiated, versioned extensions.
2. Treat `capability.revoke`, `effect.reconcile`, `execution.stop`, `session.create_restricted`, and `status.inspect` as intended-core candidates only.
3. Treat `diagnostics.configure`, `gateway.configure`, and `system.configure` as intended critical-extension candidates only.
4. Keep all eight candidates blocked until each mandatory descriptor binding closes. This ADR establishes no design-approved member and no machine membership.
5. Separate operation membership from authorization. Effective authority is the intersection of membership, epoch, channel, session scope, capability bounds, risk ceiling, approval policy, and target authority.
6. Keep `OperationDescriptor` and `AuthorizationCapability` as separate types and decisions. Names, routes, summaries, reachability, or caller values cannot widen authority.
7. Identify each published set, extension, descriptor, and schema by `(asset_id, complete SemVer, sha256 digest)`. Reject the same ID/SemVer with different bytes or digest.
8. Digest the RFC 8785 canonical logical manifest, not archive layout. Register exact domains and exclusions in the later machine batch.
9. Bind negotiation in the direction epoch → specification set → operation set → descriptor → request/result schemas. Any mismatch fails before payload-dependent side effects.
10. Require explicit critical-extension preservation, collision rejection, finite compatibility, migration, and authorization revalidation.
11. Add new fail-closed vectors in later registration/CFR work; never modify existing vector `expected` values.
12. Do not start implementation or behavior execution until TARGET, SIG, AUDIT, and all four machine-registration lines have merged and CA-0 has returned explicit GO.

## Consequences

- Open operation strings are insufficient for dispatch.
- Unknown and unnegotiated operations fail closed, but their exact registered error closure remains unresolved.
- Old epochs cannot acquire v0.2 operations or extensions.
- Extension and descriptor drift terminates or suspends an epoch and requires reauthorization.
- The three configure candidates require TARGET closure and explicit critical negotiation.
- This ADR adds governance obligations but registers no machine asset, member, digest, implementation, test result, or Profile claim.

## Alternatives considered

### Register all eight vector spellings as one fixed set

Rejected. It would turn reachability fixtures into business, wire, and authorization contracts without request/result, target, risk, error, and audit closure.

### Keep an open string operation namespace

Rejected. Caller/plugin injection and silent semantic drift would be possible; deterministic digest membership and collision handling would be absent.

### Put every operation in a closed core

Rejected. Configuration operations depend on target-specific authority and critical negotiation; forcing them into core would make incomplete governance universally available.

### Let operation membership imply authorization

Rejected. It violates the descriptor/capability separation and expands authority through discovery or negotiation.

## Rollback and failure strategy

If owner review rejects or materially changes this model, do not register machine assets. Keep D-016 open, D-022 blocking, all eight candidates blocked, and all management vectors at their current evidence status. Any later correction to a published Draft creates a new SemVer/digest; no pinned identity is rewritten.
