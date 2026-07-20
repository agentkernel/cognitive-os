# End-to-end tests (structural index)

Per repository convention, executable end-to-end suites live next to their
crates as Cargo integration tests; this directory stays a structural index
so the workspace layout is stable.

| Slice | Suite (cargo target) | What it proves |
|---|---|---|
| M4 tracer bullet | `crates/cognitive-store/tests/m4_tracer_bullet.rs` | Intent → authorize → dispatch → reconcile → verify → accept, end to end against the real gate and store |
| M5 deterministic management fallback | `apps/admin-cli/tests/m5_deterministic_fallback.rs`（spawns the real `admin-cli` binary against a real SQLite store）+ `crates/cognitive-management/tests/m5_fallback_verbs.rs` | inspect / stop / revoke / reconcile stay available with no model (REQ-MGMT-FALLBACK-001); session-gate denials dispatch nothing |

Run: `cargo test -p admin-cli --test m5_deterministic_fallback` /
`cargo test -p cognitive-management --test m5_fallback_verbs`.
