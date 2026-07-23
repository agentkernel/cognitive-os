# Lane-RUN D-018 governance-port consumption handoff

- Date: 2026-07-23
- Base: `origin/main@8683930`
- Branch: `lane/run-d018-governance-port-consumption`
- Scope: Ordinary Core publication-boundary consumption only.

## Completed

- Replaced the public caller-supplied-header envelope entry point with
  `assemble_persisted_event`.
- The runtime resolves the header through `GovernanceObjectStore` using the
  committed event identity and fails closed on a missing or unreadable header.
- Tests cover both successful durable-header assembly and withholding the
  envelope when no durable header exists.

## Verification

- `cargo test -p cognitive-runtime --test m5_event_envelope`: 2/2 pass.
- `cargo clippy -p cognitive-runtime --all-targets -- -D warnings`: pass.
- `cargo fmt --check` and `git diff --check`: pass.

## Boundary and next entry

- D-018 remains partially implemented: a real watch/shell publication path and
  CFR behavior evidence are still required. No High-Assurance audit, signature,
  independent verifier, retention, export, or Profile claim was added.
- Per ADR-0015, keep those High-Assurance areas deferred unless a named customer
  or regulatory need supplies the re-entry criteria.
