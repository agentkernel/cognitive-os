---
doc_id: dev.conformance-testing
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-conformance/src/main.rs
  - path: conformance/README.md
  - path: tools/src/check-consistency.mjs
  - path: tools/src/gen-matrix.mjs
  - path: tests/golden/README.md
tests:
  - tools/test/check.test.mjs
  - .github/workflows/ci.yml
fingerprint: "sha256:cb544d78a172f53192bf769985d062742e3f9829688f917d25f441028663301e"
non_claims:
  - Green CI is engineering evidence only; it never promotes Gate, release, or Profile claims (axiom A7).
---

# Conformance and testing

## Test taxonomy

- **Focused failure-first tests** live next to their crates
  (`crates/*/tests/*.rs`, `apps/*/tests/*.rs`, `packages/*/src/*.test.ts`) and
  are named for the task that introduced them (`p1_t04_…`). They assert denial
  paths first; happy paths second.
- **Cross-language golden fixtures** (`tests/golden/`) pin canonical-encoding
  parity.
- **Conformance vectors** (`conformance/vectors/`, 89) are contract-derived
  behavioral cases executed by the `conformance-runner`.

## The conformance runner

`cognitive-conformance` classifies every vector into a five-state report
(`pass / fail / not-implemented / not-applicable / skipped`) with pinned expected
counts in CI, evidence-honesty assertions (a vector that cannot execute must not
report pass), and a **41-flip self-check**: deliberately wrong implementations
must fail their vectors — a checker that cannot fail is treated as broken.

## Static consistency

`tools/src/check-consistency.mjs` enforces repository invariants: schema
validity (draft 2020-12), registry↔schema↔vector bidirectional references,
Markdown link resolution, Personal plan/lease/slice/Gate bookkeeping shape,
command/environment routing text, checkpoint-delivery and task-atomic wording,
and more. `tools/src/gen-matrix.mjs --check` keeps
`docs/traceability/matrix.yaml` fresh. Both run in CI and locally
(`pnpm run check:consistency`). The handbook adds its own checker
(`check-handbook.mjs`) and generator drift gate — see
[`_meta/sync-policy.md`](../../_meta/sync-policy.md).

## CI matrix

`.github/workflows/ci.yml` `verify` runs on Ubuntu and Windows MSVC: pnpm
build/test, cargo build/test (`--test-threads=1`)/clippy(-D warnings)/fmt,
codegen regeneration diff, consistency, traceability, conformance with pinned
counts, wrong-implementation self-check, and golden digest byte-parity. Rust
validation never runs on the registered-unsupported local Windows GNU host;
native Linux evidence consumes pushed exact revisions only.
