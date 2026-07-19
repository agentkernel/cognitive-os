# Normative Source and Versioning Standard

- Standard ID: `cognitiveos.standard.normative-source-versioning/0.1`
- Version: v0.1 Draft
- Machine version: `0.1.0-draft.1`
- Status: Draft Normative Standard
- Date: 2026-07-19

## 1. Scope and normative language

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are interpreted as described by RFC 2119 and RFC 8174 when they appear in uppercase.

This standard defines how CognitiveOS specification assets are classified, selected, versioned, bundled, negotiated, migrated, and cited. It does not register a schema, requirement, error, or conformance vector. Publication does not establish implementation or conformance.

## 2. Normative asset classification

Every specification asset MUST have exactly one primary class and an explicit status.

1. `normative-machine`: A registered machine-readable contract, including JSON Schema, requirement/error/operation registry entries, or another registered grammar. It is authoritative only for constraints its format can express.
2. `normative-behavior`: A versioned Core, RFC, Profile, or standard containing authority rules, state transitions, failure semantics, security boundaries, or conformance obligations.
3. `normative-test`: A registered vector, executable transition table, suite manifest, or golden fixture. It supplies evidence for requirements it cites; it does not create missing requirements.
4. `informative`: Architecture explanation, rationale, whitepaper, example, diagram, and implementation guidance.
5. `implementation-private`: Source types, private schemas, database layouts, generated bindings, transport DTOs, and unregistered extensions.
6. `historical`: Superseded or archived provenance. It MUST NOT enter a current specification set unless explicitly identified as migration input.

Pseudo-schema, example JSON, prose field lists, and planned vectors are not `normative-machine`. They MUST remain labeled pseudo, example, planned, or implementation-private until separately registered and digest-pinned.

Asset status MUST be `draft`, `candidate`, `approved`, `deprecated`, `withdrawn`, or `historical`. A Draft is normative for a claim only when the claim explicitly opts in and pins its version and digest.

## 3. Shape and behavior authority

A registered machine schema is authoritative for machine shape: names, JSON types, required members, value sets, structural constraints, and extension points.

A normative behavior asset is authoritative for authority ownership, authorization, transitions, ordering, idempotency, reconciliation, recovery, failure, audit, and acceptance.

Schema validity MUST NOT be treated as behavioral conformance. Prose MUST NOT silently add a field to a registered schema. An implementation-private type MUST NOT be represented as a registered contract.

If shape and behavior assets conflict, the implementation MUST fail closed and report the inconsistency. It MUST NOT invent a field, remove a security condition, or reinterpret a transition. Until a reviewed specification change resolves ambiguity, interpretation MUST NOT expand read, write, delegation, purpose, retention, propagation, risk, budget, or completion authority.

## 4. Source precedence

For a declared scope, precedence is:

1. Exact digest-pinned normative machine, behavior, and test assets in the applicable specification set.
2. Exact version-pinned Core, normative RFC, Profile, and standard selected by that set.
3. Informative architecture material.
4. Implementation guidance and examples.

Precedence does not allow a schema to define behavior or prose to redefine shape. A later unpinned repository revision has no authority over a pinned claim.

## 5. Draft and Semantic Versioning

Normative assets MUST use Semantic Versioning 2.0.0. Display labels MAY use `vMAJOR.MINOR Draft`; manifests MUST use complete SemVer, for example `0.1.0-draft.1`.

- PATCH: Editorial correction with no change to accepted input, output, evidence, authority, failure, or wire meaning.
- MINOR: Backward-compatible addition, such as an optional non-critical field or independently negotiated operation.
- MAJOR: Incompatible shape or semantic change, including required-field, digest-input, authority, state, error-meaning, or criticality changes.

During `0.y.z`, a MINOR MAY contain a breaking Draft change only when release notes and a migration plan identify it. It still MUST create a new specification set and MUST NOT enter an existing negotiation epoch. PATCH releases MUST remain compatible.

Two publications with the same asset ID and SemVer but different bytes or digest MUST be rejected.

## 6. Immutable digest identity

A published revision is identified by `(asset_id, semantic_version, sha256_digest)`. Once published, pinned, negotiated, signed, or cited in evidence, its bytes MUST NOT be changed in place.

A correction MUST create a new version and digest. Retagging or serving different bytes under the same identity is an integrity failure. Mirrors MAY change location but MUST preserve and verify digest identity.

## 7. Specification sets and bundles

A `specification set` is the logical ordered manifest governing a claim or negotiation. It MUST include:

- set ID, complete SemVer, status, and publication time;
- canonical encoding profile ID and version;
- each asset ID, class, SemVer, media type, and full digest;
- requirement set and operation set digests;
- schema bundle and conformance suite digests when applicable;
- selected Core, RFC, Profile, and standard versions;
- compatibility window, migration references, exclusions, and rationale.

A `bundle` packages bytes referenced by a specification set. Membership MUST be verified against the set manifest. Archive order, compression, timestamps, and file paths MUST NOT affect the logical set digest. The set digest covers the canonical logical manifest.

A schema bundle MUST close all normative `$ref` dependencies or identify immutable external dependencies by asset ID and digest. Mutable branches, tags, `latest`, or network location alone are forbidden for conformance.

A claim MUST pin the specification set digest, encoding profile, requirement set digest, schema bundle digest, applicable suite digest, implementation version or commit, and degradations.

## 8. Compatibility windows

Each protocol or Profile release MUST publish a finite list or closed range of supported specification sets. `0.x`, `latest`, or an unbounded compatibility statement is insufficient.

A receiver MAY select an older set in the same major version only when it is inside the window, all critical extensions are understood, pinned schemas and descriptors remain available, no governance or security condition is lost, and the selection is recorded in the negotiation epoch and audit.

Unknown major versions MUST NOT be guessed. A downgrade that loses critical governance data MUST fail with `VERSION_UNSUPPORTED`, `CRITICAL_EXTENSION_UNKNOWN`, or `PROTOCOL_MAPPING_INCOMPLETE` as applicable. Deprecation MUST publish a final window and removal release; removal of prior support is breaking.

## 9. Extensions and criticality

Extensions MUST use globally namespaced names such as `com.example.feature` and declare extension version, `critical: true|false`, and a schema or opaque-value rule.

An unknown critical extension MUST be rejected before business payload processing. An unknown non-critical extension MAY be ignored or preserved only when doing so cannot change authorization, authority, scope, transitions, idempotency, Effect, verification, audit, privacy, or safety.

A receiver or gateway MUST NOT strip an unknown critical extension and forward the remainder. A gateway that cannot preserve critical semantics MUST return `PROTOCOL_MAPPING_INCOMPLETE`.

## 10. Negotiation epochs

A negotiation epoch is an immutable selection bound to a peer or local trust-boundary session. It MUST identify:

- unique epoch ID and authenticated peer identities;
- selected protocol and specification set;
- canonical encoding profile;
- schema bundle and operation set digests;
- understood critical extensions and mapping profile;
- creation time, expiry or termination rule, and superseded epoch.

Version selection, size limits, digest verification, and critical extension checks MUST finish before business payload interpretation. Messages MUST carry or be unambiguously bound to the epoch ID.

Peer identity, trust boundary, selected set, schema, descriptor, or extension drift MUST terminate or suspend the epoch and establish a new one. Authorization and continuation MUST be revalidated. Reconnect MUST NOT silently restore an old epoch. Old or unknown epochs fail closed.

## 11. Legacy `agentos.*` migration

Current identifiers are `cognitiveos.*` and `cognitiveos_conformance`. Legacy `agentos.*`, `agentos_conformance`, and legacy shapes MAY be accepted only by an explicitly enabled legacy adapter or pinned legacy schema.

The adapter MUST pin source and target sets and digests; create a canonical mapping record and digest; preserve source provenance; create new target object versions instead of rewriting history; record loss, defaults, owner/authority decisions, and audit refs; reject mixed old/new identifiers in one object, message, manifest, bundle, or epoch; and return `PROTOCOL_MAPPING_INCOMPLETE` for lossy required semantics.

Legacy objects lacking tenant, `scope_domain`, authority, purpose, or resource scope MUST enter quarantine. Missing or null scope MUST NOT mean public, platform, or current tenant. Only platform governance authority may classify platform scope.

A legacy session ID MUST NOT be converted in place to Conversation ID. Migration creates a new Conversation, participant relations, scope, retention, and admission decision. A legacy AgentExecution lacking ActorChain, governance domain, Conversation or explicit non-conversational scope, or fencing epoch remains read-only quarantined or is rejected.

## 12. Conformance

A claim to this Draft MUST publish this standard's exact digest and selected specification set digests. Tests MUST cover immutable replacement, duplicate SemVer with changed digest, unknown critical extension, epoch drift, finite compatibility windows, shape/behavior conflict, and lossy legacy migration.

The claim status remains `draft` and MUST NOT be reported as approved or as evidence that a CognitiveOS capability is implemented.
