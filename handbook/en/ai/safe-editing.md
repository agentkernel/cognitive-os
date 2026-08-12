---
doc_id: ai.safe-editing
locale: en
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: docs/governance/AXIOMS.md
    symbols: ["A1", "A8"]
  - path: docs/standards/docs-sync-contract.md
fingerprint: "sha256:63a1b9f80879c9580e59eee41ae25b239915dbc0c77d282d9f79b1650c9081c1"
non_claims:
  - This page summarizes for orientation only; the linked governance documents own the binding wording.
---

# Safe editing boundaries

## Never relax (axioms, owned by [`AXIOMS.md`](../../../docs/governance/AXIOMS.md))

- A1/A2 — only the Rust daemon writes authority state; Pi, CLI, SDK, sidecars,
  fixtures, and third-party agents produce candidates and observations only.
- A3 — every external or irreversible mutation persists Intent/Effect before
  dispatch, with idempotency keys and fencing.
- A4 — Task completion requires independent verification; process exit, Provider
  response, or `agent_end` is never completion.
- A5 — secrets enter approved Secret Stores only; never argv, config, SQLite, logs,
  CI, tests, or evidence.
- A6 — contracts and negative vectors are never rewritten to fit an implementation
  (contract change goes through Lane-CTR).
- A7 — local/fixture/WSL/ordinary-CI evidence never becomes a Gate, release, or
  Profile claim.
- A8 — unknown worktree changes are protected: never overwrite, revert, stage, or
  mix them; never use `git add -A`.

## Protected trees

- `specs/**` and `conformance/**`: architecture contracts; implementation-motivated
  edits are forbidden — a real contract change follows Lane-CTR and updates registry,
  schemas, bindings, transitions, and vectors together.
- `docs/governance/**`, `docs/plan/**`: governance and formal-plan sources; edited
  only through their own governance procedures and active leases.
- `History/**`: frozen; never read, cite, or modify. `personal-blog/` never enters
  this repository.
- Generated trees (`crates/cognitive-contracts/src/generated/`,
  `packages/contracts-ts/src/generated/`, generated handbook reference pages,
  `docs/traceability/matrix.yaml`, `tests/golden/*.json`): regenerate via their
  generators; hand edits fail CI drift gates.

## Before writing anything

1. Read `PROGRESS.md` Current snapshot and the `PARALLEL-LANES.md` active lease
   table fresh. Writable ownership is an exact-path lease; active leases must not
   overlap.
2. One formal task = one branch, one Draft PR, one lease, through complete
   acceptance ([`DEVELOPMENT-OPERATING-MODEL.md`](../../../docs/governance/DEVELOPMENT-OPERATING-MODEL.md)).
3. Declare a change class (`implementation-only`, `corrective`, `product-semantic`,
   `normative-semantic`, `structural`) and complete the sync obligations of
   [`docs-sync-contract.md`](../../../docs/standards/docs-sync-contract.md) in the same delivery.
4. Consult [docs impact](./docs-impact.md): handbook pages mapped to the paths you
   change must be updated or regenerated in the same PR.

## Local environment hard facts

- The local shell is Windows PowerShell 5.1: `&&`/`||` do not parse; use separate
  commands or `if ($LASTEXITCODE -eq 0) { … }`.
- The local Windows GNU host cannot link Rust (registered linker exit 121): never
  run workspace `cargo build/test/clippy/run/bench` there; route to CI or native
  Linux (see [validation commands](./validation-commands.md)).
