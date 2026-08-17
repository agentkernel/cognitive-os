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
  - path: tools/src/generate-handbook.mjs
  - path: .github/workflows/ci.yml
  - path: docs/plan/PERSONAL-TEST-ENVIRONMENTS.md
    symbols: ["COMMAND-SHELL-PS51", "RUST-LINK-DEV-WIN-GNU-01"]
fingerprint: "sha256:f43b60785ea64b6180ae29f910c7524667abf487881c0a82cd1f6329a5aca09e"
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
node tools/src/docs-sync-gate.mjs --staged   # pre-commit docs-sync gate (--push / --range)
pnpm run hooks:install              # once per clone: registers .githooks pre-commit/pre-push
cargo fmt --all -- --check          # formatting only; no linking
git diff --check
```

## Requires supported CI (Ubuntu / Windows MSVC) or exact-revision native Linux

```bash
cargo build --workspace --locked
cargo test --workspace --locked -- --test-threads=1
cargo test -p kernel-server tool_executor --locked -- --test-threads=1
cargo test -p kernel-server p4_t05_resource_api --locked -- --test-threads=1
cargo test -p pi-agent-adapter --locked -- --test-threads=1
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run -p cognitive-conformance --bin conformance-runner
cargo run -p cognitive-contracts --bin contracts-codegen   # then git diff generated trees
```

Never run these on the local Windows GNU host: the linker failure (exit 121) is a
registered environment boundary, not a signal to reproduce. Remote/native validation
consumes only pushed, immutable revisions — never a copied working tree.
P2-T25 focused HTTP coverage is `apps/kernel-server/tests/p2_t25_tool_lifecycle.rs`
(lifecycle, selection, and pinned HTTPS origin registry).
P2-T26 focused HTTP coverage is `apps/kernel-server/tests/p2_t26_observation_plane.rs`
(O2/O3/O4/O5/O13 observation plane, controlled zeros, audit cursor negatives,
and channel negatives).
P2-T27 focused HTTP coverage is `apps/kernel-server/tests/p2_t27_backup_restore.rs`
(secret-excluding backup/restore, preflight, tamper, and task-channel denial).
P2-T28 D01 freeze is `tools/test/p2_t28_capability_truth.test.mjs` against
`tools/fixtures/p2_t28_uj_matrix.json` (existing public callers/oracles;
Web UI/Multi-Agent stay explicit `excluded`). The matching daemon register is
`apps/kernel-server/src/personal/capability_truth.rs` (Linux/CI only).
P2-T28 D02 public-caller smoke is `apps/kernel-server/tests/p2_t28_end_to_end_journey.rs`.
P2-T28 D03 exact-revision `DEV-LINUX-NATIVE-01` aggregate runs the named UJ
oracles plus `cargo test -p kernel-server --bins`, `cargo test --workspace`,
`cargo fmt --all -- --check`, and `cargo clippy --workspace --all-targets -- -D warnings`.
P2-T30 public-admit scheduler lease coverage is the kernel-server focused test
`public_admit_c1_search_leaves_draft_only_until_scheduler_acquires_lease`
(Linux/CI only; Windows GNU `not-run`).

## What CI enforces on every PR

The `verify` matrix (Ubuntu + Windows MSVC) in
[`ci.yml`](../../../.github/workflows/ci.yml): TypeScript build/test, Rust
build/test/clippy/fmt, codegen drift diff, consistency check, traceability freshness,
conformance runner with pinned five-state counts and evidence-honesty assertions,
wrong-implementation self-check, cross-language golden digest byte equality.

## Known stale entry

`pnpm run verify:local` (the V01 orchestrator) pins outdated conformance counts and
is not a usable local gate at this baseline; prefer the individual commands above.
