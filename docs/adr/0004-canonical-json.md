# ADR-0004: Canonical JSON

- Status: Accepted for v0.1 Draft normative baseline
- Date: 2026-07-19
- Decision owners: CognitiveOS specification maintainers

## Context

CognitiveOS objects cross Rust, TypeScript, storage, HTTP, event, AKP, signature, and conformance boundaries. Ordinary JSON permits insignificant whitespace, arbitrary object order, multiple number spellings, and implementation-specific Unicode and large-number handling. Hashing source bytes therefore cannot provide semantic interoperability, while an ad hoc normalizer creates security and cross-language risk.

Core requires defined map, number, time, Unicode, missing-value, and critical-extension behavior. The development baseline also requires Rust and TypeScript to produce identical golden digests.

## Decision

CognitiveOS v0.1 freezes canonical JSON to `cognitiveos.canonical-json/0.1` as specified by `docs/standards/canonical-encoding-and-digest.md`.

The profile uses:

- UTF-8 without BOM;
- I-JSON, RFC 7493;
- RFC 8785 JCS;
- RFC 3339 timestamps constrained to UTC `Z`;
- SHA-256 represented as `sha256:` plus full lowercase 64 hex;
- explicit versioned domain separation for every digest and signature;
- schema-defined missing/null behavior with no default insertion;
- explicit contract-defined digest/signature projections;
- cross-language positive and negative golden fixtures.

RFC 8785 is authoritative for object ordering, escaping, and number serialization. The canonicalizer does not normalize Unicode. Exact integers outside the I-JSON safe range and scale-sensitive decimals use schema-defined strings or structures.

Changing canonical bytes, digest or signature preimages, or exclusion rules is breaking. It requires a new encoding profile and specification set and cannot be an in-place correction.

## Alternatives considered

### Hash original JSON bytes

Rejected because formatting and producer differences produce different digests for the same semantic data.

### Project-specific canonicalizer

Rejected because number serialization, Unicode escaping, and ordering edge cases would create avoidable interoperability and security risk.

### Canonical CBOR as the initial profile

Deferred. Deterministic CBOR may be a separately negotiated profile, but v0.1 already has JSON boundaries and needs one small interoperable baseline.

### NFC normalization during hashing

Rejected because normalization changes the received value, may collapse distinct identifiers, and is not part of JCS. A field contract may require NFC before object construction.

## Consequences

Implementations need a conforming JCS library or equivalent code proven by the normative fixtures. Parsing must reject duplicate keys, invalid I-JSON, and precision loss before security decisions.

Human-readable JSON may differ from canonical bytes. Systems regenerate canonical bytes from the semantic object and never treat pretty-printed JSON as signature input.

Contract authors must explicitly define timestamps, large integers, nullable fields, domains, and self-referential exclusions. This additional contract work removes ambiguity from content addressing, bundles, authorization binding, and evidence.

## Compliance checks

The ADR is implemented only when Rust and TypeScript produce identical canonical bytes, digest preimages, full SHA-256 values, and signature inputs for all applicable golden fixtures. Schema validity alone is not evidence of implementation.
