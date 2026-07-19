# ADR-0006: Schema-to-Code Generation Policy

- Status: Accepted for the reference implementation baseline
- Date: 2026-07-20
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: reference implementation decision. This ADR binds this
  repository's implementation only; it is NOT a CognitiveOS specification
  requirement.

## Context

56 JSON Schemas (draft 2020-12) under `specs/schemas/` are the machine
truth for governed objects. Hand-written Rust structs and TypeScript
interfaces would drift from schemas silently — exactly the dual-track
failure mode F-003 documented at the schema layer. The repository needs one
policy fixing how language bindings relate to schemas before Lane-CTR (M1)
builds the pipeline.

## Decision

1. Language bindings for registered schemas are generated, never
   hand-written: schema → Rust types (into `cognitive-contracts`) and
   schema → TypeScript types (into `packages/contracts-ts`).
2. Generated artifacts are committed to the repository, clearly marked with
   a generation header (source schema path + schema content digest +
   generator version), and are reviewable in PRs like any other code.
3. Hand-editing generated files is forbidden. A needed change goes into the
   schema (through the change process of
   `.cursor/rules/12-schemas-protocol.mdc` and `docs-sync-contract.md`) or
   into the generator, then everything regenerates.
4. CI regenerates and diffs: a dirty diff after regeneration fails the
   build (drift gate). The pinned schema digest in each header is verified
   against the actual schema file.
5. Newtypes and enums: generated Rust uses newtype IDs and exhaustive
   enums for registered enumerations; generated TS uses union literal
   types. Generator configuration is code, versioned with the repo.
6. The generator toolchain is chosen and delivered by Lane-CTR in M1
   (candidates: `typify`/custom for Rust, `json-schema-to-typescript`/custom
   for TS). Until then, the only hand-written contract code allowed is the
   canonical encoding layer (`canonical.rs` / `canonical.ts`), which is
   fixture-proven rather than schema-derived.

## Alternatives considered

### Hand-written types with review discipline

Rejected: review does not scale to 56 schemas times two languages; F-003
shows drift happens even inside the schema layer itself.

### Runtime-only validation (no static types)

Rejected: loses compile-time exhaustiveness for state machines and IDs,
which the kernel discipline depends on (`10-rust-kernel.mdc`).

### Generating at build time without committing artifacts

Rejected: uncommitted generated code cannot be reviewed, digest-pinned, or
diffed by CI; supply-chain surface of the generator grows silently.

## Consequences

PRs that change schemas include regenerated bindings — larger diffs but
visible impact. The generator becomes a governed tool: its version bump is
a reviewable change. Bootstrap ordering: M1 delivers the generator plus the
first generated object families (common-defs, governed-object-header,
object-reference, effect, intent); remaining families follow their
consuming milestones.

## Compliance checks

From M1: CI "regenerate and diff" job green; every generated file carries a
header whose schema digest matches the source schema; grep for the
generation header in hand-edited hunks is part of code review
(`.cursor/rules/12-schemas-protocol.mdc` checklist).

## Delivery record (M1, Lane-CTR)

The generator is the committed custom binary
`crates/cognitive-contracts/src/bin/contracts-codegen.rs` (single source for
both target languages; the typify / json-schema-to-typescript candidates were
rejected because neither covers both languages and neither pins deterministic
committed output with digest headers). Facts of the delivered pipeline:

1. Input: the IMP-08 minimal core object set (appendix A.1, 14 objects)
   mapped to 17 registered schemas plus the transitive `$ref` closure
   (19 files; the closure adds actor-chain and conversation-binding). A.1
   objects without a same-named document schema map to their closest
   registered machine surface (mapping table in the generator header).
2. Output: `crates/cognitive-contracts/src/generated/` and
   `packages/contracts-ts/src/generated/`, one module per schema, namespaced
   re-exports. Every file header carries source path + canonical schema
   content digest (domain `schema-bundle/0.1`, identical to the
   schema-bundle manifest per-asset digest) + generator version.
3. Bindings are shape-level: `if`/`then` conditionals, `allOf` const
   refinements, `contains`, and cross-field constraints remain enforced by
   JSON Schema validation; the generator fails loudly on constructs outside
   its supported subset. Deprecated legacy `$defs` (common-defs
   metadata/strongRef) are excluded by policy (F-003).
4. Regeneration procedure:
   `cargo run -p cognitive-contracts --bin contracts-codegen && cargo fmt --all`.
   CI runs exactly this and fails on a dirty diff.
5. Header digests are additionally verified without regeneration by
   `crates/cognitive-contracts/tests/generated_types.rs`
   (`generated_headers_pin_current_schema_digests`).
6. Remaining object families follow their consuming milestones (unchanged
   from the Consequences section).
