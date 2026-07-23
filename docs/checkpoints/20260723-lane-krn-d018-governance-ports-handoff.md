# Lane-KRN D-018 governance-header resolution handoff

- Date: 2026-07-23
- Base: `origin/main@7007a6c91ea38124f8aae98b4065e38cc70ad2f9`
- Branch: `lane/krn-d018-governance-ports`
- Scope: D-018 KRN residual only; no schema, vector, runtime, or runner changes.

## Completed

- Added `GovernanceObjectStore::load_governed_object_header` to the kernel persistence boundary.
- SQLite resolves the immutable canonical header for persisted M5 UserIntentRecord, IntentInterpretation, and TaskContract identities. Missing identities return `None`; malformed, identity-mismatched, or ambiguous rows fail closed.
- Added a test-first SQLite behavior test proving that a persisted UserIntentRecord returns its durable owner, authority, resource-scope, and tenant references and that an unknown identity cannot synthesize them.

## Test-first and verification evidence

1. Before the port existed, the focused test failed to compile with unresolved `GovernanceObjectStore` / `load_governed_object_header` symbols.
2. After implementation, the focused test passed.
3. `cargo test -p cognitive-store --test m5_intent_chain`: 6/6 pass.
4. `cargo clippy -p cognitive-kernel -p cognitive-store --all-targets -- -D warnings`, `cargo fmt --check`, and `git diff --check`: pass.
5. Baseline static checks: `pnpm run check:consistency` passed at 273 requirements / 55 error codes / 63 schemas / 85 vectors; `gen-matrix --check` passed.

## Boundaries and next entry

- This is an implementation/test result, not a D-018 closure, Event envelope behavior result, Profile claim, or v0.1 claim change.
- Lane-RUN must consume this port at the publication boundary instead of accepting a caller-supplied header; Lane-CFR then needs real watch/shell behavior evidence before D-018 can be re-evaluated.
- Ordinary Core management operation implementation remains blocked by D-016/D-022 machine-registration prerequisites; do not implement or execute `MGMT-FALLBACK-008` by treating this port as a substitute.
