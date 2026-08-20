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
  - path: tools/src/generate-handbook.mjs
  - path: tests/golden/README.md
  - path: tools/src/p2_t28_uj_matrix.mjs
    symbols: ["validateUjCapabilityTruthMatrix"]
tests:
  - tools/test/check.test.mjs
  - tools/test/p2_t28_capability_truth.test.mjs
  - tools/test/c1_c2_paired_p_arm.test.mjs
  - .github/workflows/ci.yml
fingerprint: "sha256:fa3576dafe87d14837215beb718bc3a1a77c5ef0061a5ead86792da5a56ad830"
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
- **C1/C2 paired measurement instruments** (`tools/personal/c1-c2-paired/`) are
  campaign-only: a loopback pure-Pi credential broker (Secret Service `get` via
  D-Bus, never `secret-tool lookup`/`search`), an equivalent Workspace* fixture
  adapter, frozen seeds/`retry=0`, and a §2.3 fairness checker that records
  `system_task_prompt_bytes` from `frozen-system-task-prompt.txt` (not a shared
  placeholder) and live P/O `--append-system-prompt` command manifests that share
  that file. They are not a
  second authority writer and do not promote Gate, release, Profile, B01, or
  Agent-benefit. Focused tests: `tools/test/c1_c2_paired_p_arm.test.mjs`.

## The conformance runner

`cognitive-conformance` classifies every vector into a five-state report
(`pass / fail / not-implemented / not-applicable / skipped`) with pinned expected
counts in CI, evidence-honesty assertions (a vector that cannot execute must not
report pass), and a **41-flip self-check**: deliberately wrong implementations
must fail their vectors — a checker that cannot fail is treated as broken.

## Static consistency

`tools/src/check-consistency.mjs` enforces repository invariants: schema
validity (draft 2020-12), registry↔schema↔vector bidirectional references,
Markdown link resolution, Personal plan/lease/slice/Gate bookkeeping shape
(including the `lease/personal/EVAL-<id>/…` owner-directed evaluation-campaign
lease class, which must name a snapshot-registered campaign and may own only
`docs/evaluation/`, `docs/checkpoints/`, and `docs/plan/PROGRESS.md`),
command/environment routing text, checkpoint-delivery and task-atomic wording,
and more. `tools/src/gen-matrix.mjs --check` keeps
`docs/traceability/matrix.yaml` fresh. Both run in CI and locally
(`pnpm run check:consistency`). The handbook adds its own checker
(`check-handbook.mjs`) and generator drift gate — see
[`_meta/sync-policy.md`](../../_meta/sync-policy.md). HTTP route generation
also reads `apps/kernel-server/src/personal/tool_lifecycle.rs`,
`apps/kernel-server/src/personal/pinned_https.rs`, and
`apps/kernel-server/src/personal/observation.rs` so annotated Tool lifecycle,
pinned-HTTPS, and observation-plane paths cannot rot.

## UJ capability-truth freeze

`tools/src/p2_t28_uj_matrix.mjs` freezes the BR-08 UJ1..UJ6 rows. Required rows
must name an existing public caller file and a mechanical oracle file, plus
cleanup and a bounded evidence schema. Web UI and Multi-Agent stay explicit
`excluded` rows and cannot be marked required. The daemon-side register is
`apps/kernel-server/src/personal/capability_truth.rs`. This freeze is not an
EVAL-004, Gate, release, or Profile result. The D02 hermetic public-caller
smoke is `apps/kernel-server/tests/p2_t28_end_to_end_journey.rs`. Named UJ
oracles run on exact-revision `DEV-LINUX-NATIVE-01`; Windows GNU is `not-run`
for that Rust matrix.

## CI matrix

`.github/workflows/ci.yml` `verify` runs on Ubuntu and Windows MSVC: pnpm
build/test, cargo build/test (`--test-threads=1`)/clippy(-D warnings)/fmt,
codegen regeneration diff, consistency, traceability, conformance with pinned
counts, wrong-implementation self-check, and golden digest byte-parity. Rust
validation never runs on the registered-unsupported local Windows GNU host;
native Linux evidence consumes pushed exact revisions only.
