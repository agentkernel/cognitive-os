---
doc_id: dev.contracts-codegen
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-contracts/src/bin/contracts-codegen.rs
    symbols: ["CORE_SET"]
  - path: crates/cognitive-contracts/src/canonical.rs
    symbols: ["canonicalize", "digest_json"]
  - path: packages/contracts-ts/src/canonical.ts
  - path: crates/cognitive-domain/src/transitions.rs
    symbols: ["table", "find_edge"]
contracts:
  - specs/registry/requirements.yaml
  - specs/registry/errors.yaml
  - specs/registry/state-domains.yaml
tests:
  - crates/cognitive-contracts/tests/golden_fixtures.rs
  - tests/golden/README.md
fingerprint: "sha256:a1c451d34454a9eb22c9936b6b0ea92f8536a5df3dd8bd25d18fb0e6b724f8ee"
non_claims:
  - Generated bindings are shape-level projections; the JSON Schemas remain the only shape truth, and codegen never relaxes them.
---

# Contracts and codegen

## Canonical encoding

Both languages implement the identical canonical JSON profile (sorted keys, no
insignificant whitespace, forbidden non-finite numbers, `-0` normalization,
integer-only range, NFC strings) and domain-separated SHA-256 digests
(`digest_json(input, domain)`). Cross-language byte equality is enforced by
golden fixtures (`tests/golden/*.json`) that both Rust and TS emitters must
reproduce byte-identically in CI.

## Codegen pipeline

`contracts-codegen` reads a pinned `CORE_SET` of schemas from `specs/schemas/`
and emits deterministic Rust modules
(`crates/cognitive-contracts/src/generated/`, 53 modules) and TypeScript modules
(`packages/contracts-ts/src/generated/`, 55 including index/registry). Output is
committed; CI regenerates and diffs, so hand edits or schema drift fail the
build. The error registry (`specs/registry/errors.yaml`, 55 codes) generates
`RegisteredErrorCode` enums on both sides; unknown codes fail closed at parse
time.

## Embedded transition tables

`cognitive-domain` embeds the five `specs/transitions/*.transitions.json` tables
at compile time; `table(domain)` exposes version + canonical digest, and the
kernel's step-1 table-pin check compares against exactly these. Editing a
transition JSON without the Lane-CTR contract procedure breaks the pin — by
design.

## Contract-change discipline (Lane-CTR)

A real contract change updates together: registry entries, schemas, both
generated trees, transition tables (if applicable), conformance vectors, and the
docs-sync obligations — then the drift gates, consistency checker, traceability
matrix, and conformance runner all agree. Implementation code never edits
`specs/` to make a test pass (axiom A6).
