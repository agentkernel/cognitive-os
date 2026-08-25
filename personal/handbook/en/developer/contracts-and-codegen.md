---
doc_id: dev.contracts-codegen
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: core/crates/cognitive-contracts/src/bin/contracts-codegen.rs
    symbols: ["CORE_SET"]
  - path: core/crates/cognitive-contracts/src/canonical.rs
    symbols: ["canonicalize", "digest_json"]
  - path: core/packages/contracts-ts/src/canonical.ts
  - path: core/crates/cognitive-domain/src/transitions.rs
    symbols: ["table", "find_edge"]
contracts:
  - core/specs/registry/requirements.yaml
  - core/specs/registry/errors.yaml
  - core/specs/registry/state-domains.yaml
tests:
  - core/crates/cognitive-contracts/tests/golden_fixtures.rs
  - core/tests/golden/README.md
fingerprint: "sha256:022dfbbee98991f7715eb4e092f6ebc52e28b6ed95b90dc122c2a1cb0fcc8a46"
non_claims:
  - Generated bindings are shape-level projections; the JSON Schemas remain the only shape truth, and codegen never relaxes them.
---

# Contracts and codegen

## Canonical encoding

Both languages implement the identical canonical JSON profile (sorted keys, no
insignificant whitespace, forbidden non-finite numbers, `-0` normalization,
integer-only range, NFC strings) and domain-separated SHA-256 digests
(`digest_json(input, domain)`). Cross-language byte equality is enforced by
golden fixtures (`core/tests/golden/*.json`) that both Rust and TS emitters must
reproduce byte-identically in CI.

## Codegen pipeline

`contracts-codegen` reads a pinned `CORE_SET` of schemas from `core/specs/schemas/`
and emits deterministic Rust modules
(`core/crates/cognitive-contracts/src/generated/`, 53 modules) and TypeScript modules
(`core/packages/contracts-ts/src/generated/`, 55 including index/registry). Output is
committed; CI regenerates and diffs, so hand edits or schema drift fail the
build. The error registry (`core/specs/registry/errors.yaml`, 55 codes) generates
`RegisteredErrorCode` enums on both sides; unknown codes fail closed at parse
time.

## Embedded transition tables

`cognitive-domain` embeds the five `core/specs/transitions/*.transitions.json` tables
at compile time; `table(domain)` exposes version + canonical digest, and the
kernel's step-1 table-pin check compares against exactly these. Editing a
transition JSON without the Lane-CTR contract procedure breaks the pin — by
design.

## Contract-change discipline (Lane-CTR)

A real contract change updates together: registry entries, schemas, both
generated trees, transition tables (if applicable), conformance vectors, and the
docs-sync obligations — then the drift gates, consistency checker, traceability
matrix, and conformance runner all agree. Implementation code never edits
`core/specs/` to make a test pass (axiom A6).
