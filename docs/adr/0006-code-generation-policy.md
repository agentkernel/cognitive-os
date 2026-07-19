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
