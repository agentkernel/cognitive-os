---
doc_id: dev.conformance-testing
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: personal/crates/cognitive-conformance/src/main.rs
  - path: core/conformance/README.md
  - path: tools/src/check-consistency.mjs
  - path: tools/src/gen-matrix.mjs
  - path: tools/src/generate-handbook.mjs
  - path: core/tests/golden/README.md
  - path: tools/src/p2_t28_uj_matrix.mjs
    symbols: ["validateUjCapabilityTruthMatrix"]
  - path: tools/src/p7_t05_web_ui_inventory.mjs
    symbols: ["validateWebUiRouteInventory"]
  - path: tools/src/personal-rc-gate.mjs
    symbols: ["buildPersonalRcDeclarationReport"]
tests:
  - tools/test/check.test.mjs
  - tools/test/p2_t28_capability_truth.test.mjs
  - tools/test/c1_c2_paired_p_arm.test.mjs
  - tools/test/p7_t05_web_ui_inventory.test.mjs
  - tools/test/personal-rc-gate.test.mjs
  - .github/workflows/ci.yml
fingerprint: "sha256:25dfb012a8ffe84ba221c95d748c7bde565f663479395a422aac98634fbd6467"
non_claims:
  - Green CI is engineering evidence only; it never promotes Gate, release, or Profile claims (axiom A7).
---

# Conformance and testing

## Test taxonomy

- **Focused failure-first tests** live next to their crates
  (`crates/*/tests/*.rs`, `apps/*/tests/*.rs`, `packages/*/src/*.test.ts`) and
  are named for the task that introduced them (`p1_t04_…`). They assert denial
  paths first; happy paths second.
- **Cross-language golden fixtures** (`core/tests/golden/`) pin canonical-encoding
  parity.
- **Conformance vectors** (`core/conformance/vectors/`, 89) are contract-derived
  behavioral cases executed by the `conformance-runner`.
- **C1/C2 paired measurement instruments** (`tools/personal/c1-c2-paired/`) are
  campaign-only: a loopback pure-Pi credential broker (Secret Service `get` via
  D-Bus, never `secret-tool lookup`/`search`), an equivalent Workspace* fixture
  adapter, frozen seeds/`retry=0`, and a §2.3 fairness checker that records
  `system_task_prompt_bytes` from `frozen-system-task-prompt.txt` (not a shared
  placeholder) and live P/O `--append-system-prompt` command manifests that share
  that file. Live `runLivePairedCell` requires an injected `executeArm` (no
  accidental spawn); `counted_sample` is true only for frozen b1/b2 cells when
  fairness passes and both arms exit 0 without timeout. Dry-run cannot be
  labeled counted. P-arm `WorkspacePatch` `input_b64` is a UTF-8 unified diff
  (`workspace_patch_payload: unified-diff`); replacement bytes fail closed.
  They are not a
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
`docs/evaluation/`, `docs/checkpoints/`, and `docs/plan/PROGRESS.md`, and the
`lease/personal/GOV-<id>/…` owner-directed governance-delivery lease class
(ADR-0055), whose description must name the same snapshot-registered
`GOV-<id>` and which may own only `docs/governance/`, `docs/adr/`,
`docs/plan/PROGRESS.md`, the lease-grammar checker surface
(`tools/src/check-consistency.mjs`, `tools/test/check.test.mjs`), and mapped
handbook pages under `personal/handbook/`; and the `lease/personal/DOC-<id>/…`
owner-directed documentation-alignment lease class, whose description must name
the same snapshot-registered `DOC-<id>` and which may own only exact
plan/product/architecture/handbook/design documents, `AGENTS.md`,
`.cursor/rules/`, its own dated `docs/checkpoints/` report/closure files — never
the directory — and the same lease-grammar checker surface),
command/environment routing text, checkpoint-delivery and task-atomic wording,
and more. `tools/src/gen-matrix.mjs --check` keeps
`docs/traceability/matrix.yaml` fresh. Both run in CI and locally
(`pnpm run check:consistency`).

Since `P0-T09`, path existence in `check-consistency.mjs` and
`check-agent-rules.mjs` is decided by `git ls-files`, not by the filesystem: a
committed document or rule that links a file which exists only in the author's
working tree fails locally with the same message CI would print (`… (exists
locally but is not tracked by Git)`), and untracked local Markdown is not
scanned, so the local and CI verdicts are identical. Outside a Git checkout the
consistency checker fails closed (`TRACKED_PATHS_UNAVAILABLE`); the agent-rule
checker falls back to the filesystem only for its focused fixtures and labels
that mode (`path existence = …`). The owner's untracked local editor assets
(`.cursor/skills/`, `.cursor/commands/`, rules 30/40, `.cursor/mcp.json`) keep
their warn-when-absent / strict-when-present behaviour. The same checker also
parses the Phase 13 build-order mermaid graph in
`docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` and its copy in
`personal/docs/architecture/personal-2.0.0-dev-prep-index.md` and requires
identical edge sets (solid vs dashed included; ids normalized by dropping the
`P13`/`P11` prefix) — a missing or extra edge is `BUILD_ORDER_EDGE_MISSING` /
`BUILD_ORDER_EDGE_EXTRA`; the formal plan is authoritative and is never edited
to match the index.

The handbook adds its own checker
(`check-handbook.mjs`) and generator drift gate — see
[`_meta/sync-policy.md`](../../_meta/sync-policy.md). HTTP route generation
also reads `personal/apps/kernel-server/src/personal/tool_lifecycle.rs`,
`personal/apps/kernel-server/src/personal/pinned_https.rs`, and
`personal/apps/kernel-server/src/personal/observation.rs` so annotated Tool lifecycle,
pinned-HTTPS, and observation-plane paths cannot rot.

## UJ capability-truth freeze

`tools/src/p2_t28_uj_matrix.mjs` freezes the BR-08 UJ1..UJ6 rows. Required rows
must name an existing public caller file and a mechanical oracle file, plus
cleanup and a bounded evidence schema. Web UI and Multi-Agent stay explicit
`excluded` rows and cannot be marked required. The daemon-side register is
`personal/apps/kernel-server/src/personal/capability_truth.rs`. This freeze is not an
EVAL-004, Gate, release, or Profile result. The D02 hermetic public-caller
smoke is `personal/apps/kernel-server/tests/p2_t28_end_to_end_journey.rs`. Named UJ
oracles run on exact-revision `DEV-LINUX-NATIVE-01`; Windows GNU is `not-run`
for that Rust matrix.

## P7-T05 Web UI route inventory

`tools/src/p7_t05_web_ui_inventory.mjs` freezes UI capabilities onto existing
daemon routes. Invented routes, generic lifecycle transitions, Task-channel
secret-bearing writes, and browser-direct SQLite/SecretStore/filesystem/Provider
access fail closed. Missing typed HTTP (Task cancel; Agent
pause/resume/stop/restart/quarantine) must stay `unavailable`/`not-run`. The
inventory is not a SPA implementation, browser journey, Gate, or release result.

## Personal Linux RC declaration binder

`tools/src/personal-rc-gate.mjs` binds existing MVP Gate dispositions and
operability evidence into a digest-bound Personal Linux RC declaration. Incomplete
observations, missing digests, Profile keys, an enabled P6, non-zero RC-scope
critical risks, and a production GitHub Release claim fail closed. The evaluator
does not set Gate or Profile state. Focused tests:
`tools/test/personal-rc-gate.test.mjs`.

## CI matrix

`.github/workflows/ci.yml` `verify` runs on Ubuntu and Windows MSVC: pnpm
build/test, cargo build/test (`--test-threads=1`)/clippy(-D warnings)/fmt,
codegen regeneration diff, consistency, traceability, the agent-rule reference
check (`tools/src/check-agent-rules.mjs`: frontmatter and path/skill/command
references of `AGENTS.md`, `.cursor/rules`, `.cursor/commands`), handbook checks,
conformance with pinned counts, wrong-implementation self-check, and golden digest
byte-parity. Rust
validation never runs on the local Windows GNU host; the registered local
MSVC-override directories (P0-T01/D02) may run the same cargo commands for
development iteration only, never as the cited validation; native Linux
evidence consumes pushed exact revisions only.
