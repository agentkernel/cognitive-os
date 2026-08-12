---
doc_id: ai.validation-commands
locale: en
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: package.json
  - path: tools/package.json
  - path: .github/workflows/ci.yml
  - path: docs/plan/PERSONAL-TEST-ENVIRONMENTS.md
    symbols: ["COMMAND-SHELL-PS51", "RUST-LINK-DEV-WIN-GNU-01"]
fingerprint: "sha256:f936a3519e209064b1fc9933429884b531e8834ceef383854384b260fc0d002b"
non_claims:
  - Command availability is not evidence; only actually executed checks count, and local results never promote Gate/release/Profile claims.
---

# Validation commands

Environment routing is a precondition, owned by
[`PERSONAL-TEST-ENVIRONMENTS.md`](../../../docs/plan/PERSONAL-TEST-ENVIRONMENTS.md).

## Safe on every platform (including the Windows GNU host)

```powershell
pnpm install --frozen-lockfile
pnpm -r build
pnpm -r test
pnpm run check:consistency          # tools/src/check-consistency.mjs
node tools/src/gen-matrix.mjs --check
node tools/src/check-handbook.mjs   # handbook drift gate
node tools/src/generate-handbook.mjs --check
cargo fmt --all -- --check          # formatting only; no linking
git diff --check
```

## Requires supported CI (Ubuntu / Windows MSVC) or exact-revision native Linux

```bash
cargo build --workspace --locked
cargo test --workspace --locked -- --test-threads=1
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run -p cognitive-conformance --bin conformance-runner
cargo run -p cognitive-contracts --bin contracts-codegen   # then git diff generated trees
```

Never run these on the local Windows GNU host: the linker failure (exit 121) is a
registered environment boundary, not a signal to reproduce. Remote/native validation
consumes only pushed, immutable revisions — never a copied working tree.

## What CI enforces on every PR

The `verify` matrix (Ubuntu + Windows MSVC) in
[`ci.yml`](../../../.github/workflows/ci.yml): TypeScript build/test, Rust
build/test/clippy/fmt, codegen drift diff, consistency check, traceability freshness,
conformance runner with pinned five-state counts and evidence-honesty assertions,
wrong-implementation self-check, cross-language golden digest byte equality.

## Known stale entry

`pnpm run verify:local` (the V01 orchestrator) pins outdated conformance counts and
is not a usable local gate at this baseline; prefer the individual commands above.
