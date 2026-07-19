# CognitiveOS Governed Object Contract

- Standard ID: `cognitiveos.governed-object-contract/0.1`
- Version: v0.1 Draft
- Machine version: `0.1.0-draft.1`
- Status: Normative machine-contract standard
- Date: 2026-07-19
- Normative sources: CognitiveOS Core v0.2, RFC-0001 v0.2, and CognitiveOS Development Plan item 3

This standard registers the governed object family as Draft 2020-12 normative machine contracts. JSON Schema defines machine shape; pinned Core and RFC requirements continue to define authority, authorization, transition, recovery, and conformance behavior. The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY follow RFC 2119 and RFC 8174.

## 1. Registered v0.1 Draft contracts

The following contracts are registered by the correspondingly named files under `specs/schemas/`:

| Contract | Machine schema |
|---|---|
| ObjectReference | [object-reference.schema.json](../../specs/schemas/object-reference.schema.json) |
| GovernedObjectHeader | [governed-object-header.schema.json](../../specs/schemas/governed-object-header.schema.json) |
| GovernanceDomainContext | [governance-domain-context.schema.json](../../specs/schemas/governance-domain-context.schema.json) |
| Principal | [principal.schema.json](../../specs/schemas/principal.schema.json) |
| Membership | [membership.schema.json](../../specs/schemas/membership.schema.json) |
| ActorChain | [actor-chain.schema.json](../../specs/schemas/actor-chain.schema.json) |
| AuthorizationDelegation | [authorization-delegation.schema.json](../../specs/schemas/authorization-delegation.schema.json) |
| ResourceScope | [resource-scope.schema.json](../../specs/schemas/resource-scope.schema.json) |
| Conversation | [conversation.schema.json](../../specs/schemas/conversation.schema.json) |
| ConversationBinding | [conversation-binding.schema.json](../../specs/schemas/conversation-binding.schema.json) |
| AgentExecutionBinding | [agent-execution-binding.schema.json](../../specs/schemas/agent-execution-binding.schema.json) |
| ExecutionContext | [execution-context.schema.json](../../specs/schemas/execution-context.schema.json) |
| ActivityContext | [activity-context.schema.json](../../specs/schemas/activity-context.schema.json) |

Schema registration does not claim implementation or conformance. A claim MUST pin the schema bundle digest, requirement set, implementation version, and evidence.

## 2. Identity and governance domains

[REQ-GOBJ-ID-001] Stable IDs introduced here MUST be lowercase canonical UUIDv7. Schemas enforce UUIDv7 by pattern and do not assume support for a UUID-version format.

[REQ-GOBJ-DOMAIN-001] Every governed object MUST explicitly carry `scope_domain`. A tenant object MUST carry `tenant_id`; a platform object MUST NOT carry it. Missing or null tenant identity MUST NOT mean public or platform.

[REQ-GOBJ-TENANT-001] A governed object's and AgentExecution's tenant binding is immutable for its stable identity lifetime. Changing tenant MUST create a new identity and auditable migration or recovery relation. `tenant_id`, GovernanceDomainContext, and bindings MUST NOT be replaced in place to switch tenants.

[REQ-GOBJ-HEADER-001] Persistent and cross-boundary governed objects MUST carry a `GovernedObjectHeader` or a verifiable strong reference to one. Owner denotes lifecycle responsibility; authority denotes the write or arbitration boundary. Neither alone grants body read access.

## 3. Strong and weak references

[REQ-GOBJ-REF-001] A strong reference contains exactly `kind=strong`, `id`, `object_version`, and `content_digest`. It fixes one immutable version of one logical object. IDs, URIs, digests, and successful resolution are not authorization.

[REQ-GOBJ-REF-002] A weak reference contains `kind=weak`, `id`, `freshness`, and `resolution`. Freshness declares acceptable age or version. Resolution declares selection timing and strategy, failure behavior, and mandatory result pinning.

[REQ-GOBJ-REF-003] Weak references MUST resolve under the current GovernanceDomainContext, ActorChain, purpose, ResourceScope, policy, membership, and revocation versions. Before mutation, external Effect, or authority commit, the selected result MUST be fixed as a strong reference. Audit evidence MUST retain resolution time, resolver policy/version, selected `object_version`, and `content_digest`.

The legacy `common-defs.schema.json#/$defs/strongRef` uses `version` and `digest`; it is not this contract. Migration MUST explicitly map them to `object_version` and `content_digest` and verify the digest.

[REQ-GOBJ-REF-004] Default digest projection: unless an object family registers a more specific projection, `content_digest` is computed over the canonical bytes ([canonical-encoding-and-digest](./canonical-encoding-and-digest.md)) of the schema-valid object with exactly these JSON Pointer paths declared `digest_excluded`: the object's own `content_digest` field (`/metadata/content_digest` for common-defs metadata objects, `/header/content_digest` for GovernedObjectHeader objects) and its own `signature` field if present. No other path is excluded by default. This registration satisfies the canonical standard's requirement that self-referential fields be excluded by a named contract, not by convention; object families whose digest must cover additional derived stores register their own projection with a new version.

## 4. Binding consistency

[REQ-GOBJ-BIND-001] ConversationBinding MUST fix Conversation version, participant relation version, history scope, and working scope. These MUST share the Conversation governance domain and tenant. Stale participant, policy, membership, or revocation versions require reauthorization.

[REQ-GOBJ-BIND-002] AgentExecutionBinding MUST bind one execution identity to one GovernanceDomainContext, ActorChain, initiating/effective/workload subjects, governance versions, and fencing epoch. Explicit subjects MUST agree with ActorChain. Tenant references and scopes MUST use the execution tenant; platform bindings contain no tenant identity.

[REQ-GOBJ-BIND-003] Each Activity MUST select exactly one ConversationBinding or `kind=non_conversational` ResourceScope. It MUST NOT infer a recent Conversation from authentication/runtime sessions, caches, preferences, or prior Activities.

[REQ-GOBJ-BIND-004] ExecutionContext MUST NOT widen AgentExecutionBinding. ActivityContext MUST NOT widen ExecutionContext, ActorChain, purpose, ResourceScope, capabilities, or governance versions. ContextViews, Effects, audit events, and derivatives MUST trace to ActivityContext. ActivityContext creation is two-phase: it is created without `context_view_ref`, the ContextRequest binds the ActivityContext, and the resolved ContextView is bound exactly once via compare-and-set before any governed proposal in that Activity; re-binding a different view to the same Activity is forbidden (new resolution requires a delta bound to the same base view or a new Activity).

[REQ-GOBJ-BIND-005] Reusing an AgentExecution for another Conversation is allowed only at an Activity boundary after checkpoint, pending-Effect reconciliation, working-state isolation, governance and ActorChain revalidation, context resolution, and binding/fencing epoch advance.

## 5. Authorization delegation boundary

[REQ-GOBJ-AUTHDEL-001] AuthorizationDelegation conveys only a monotonically attenuated subset of audience, purpose, resources, actions, parameters, expiry, and depth. A child MUST be no broader than its parent and fails closed on expiry, revocation, suspension, digest mismatch, or exhausted depth.

[REQ-GOBJ-AUTHDEL-002] AuthorizationDelegation is distinct from `delegation.schema.json` and Distributed Profile Delegation. It MUST NOT represent TaskContract subdivision, child-task acceptance, budget escrow/grant, work data visibility, distributed lease ownership, mailbox delivery, remote completion, result acceptance, or authority transfer. It creates no child Task and proves no Effect or Task complete.

[REQ-GOBJ-AUTHDEL-003] AuthorizationDelegation is not AuthorizationCapability. It supplies an attenuation boundary; local authority still evaluates policy, capability, resource, purpose, parameters, governance versions, and explicit deny.

## 6. Migration

[REQ-GOBJ-MIG-001] `agentos.*` identifiers/properties MAY be accepted only through a version-pinned legacy adapter recording source/target versions, mapping digest, authority decision, unresolved fields, and audit correlation. `agentos.*` and `cognitiveos.*` MUST NOT be silently mixed in one object or protocol epoch.

[REQ-GOBJ-MIG-002] Legacy objects missing `scope_domain`, tenant ownership, ResourceScope, owner, or authority MUST enter quarantine. They MUST NOT default to public, platform, current tenant, or importer. Only explicit platform governance authority may classify platform objects.

[REQ-GOBJ-MIG-003] A legacy session ID MUST NOT become a Conversation ID. Transcript migration creates a new Conversation, participants, ResourceScope, retention decision, and lineage. An execution without ActorChain and an exclusive Conversation/non-conversational Activity binding is rejected or explicitly read-only quarantined.

[REQ-GOBJ-MIG-004] Breaking semantics require a new schema major version, explicit negotiation, or auditable migration. Downgrade MUST NOT discard critical governance fields. Non-preserving mappings fail with `PROTOCOL_MAPPING_INCOMPLETE`.

## 7. Validation and conformance

[REQ-GOBJ-VALID-001] Producers MUST emit Draft 2020-12-valid instances without undeclared properties. Consumers reject unknown fields unless a future contract adds an explicit namespaced extension mechanism.

[REQ-GOBJ-VALID-002] Schema validity is insufficient by itself. Evidence MUST cover tenant immutability, strong-reference checks, weak-reference resolution/pinning, binding consistency, stale governance versions, exclusive Activity binding, and attenuation negative cases.

[REQ-GOBJ-VALID-003] Claims report these contracts as `implemented`, `planned`, `experimental`, or `unsupported`. Draft registration MUST NOT be reported as implemented behavior.
