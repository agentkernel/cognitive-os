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
  - path: docs/bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md
  - path: tools/src/p7_t05_web_ui_inventory.mjs
    symbols: ["validateWebUiRouteInventory"]
  - path: tools/src/personal-rc-gate.mjs
    symbols: ["buildPersonalRcDeclarationReport"]
fingerprint: "sha256:b173c9f14af8f2053a107a6573bcc236b76ffaf77f9d1299e283381701ebc3b9"
non_claims:
  - Command availability is not evidence; only actually executed checks count, and local results never promote Gate/release/Profile claims.
---

# Validation commands

P11-T08 focused store/HTTP tests route to `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` or exact-revision `DEV-LINUX-NATIVE-01`. `DEV-WIN-GNU-01` cargo link is `not-run`. Clock/sleep/restart host E2E is `not-run`.

Environment routing is a precondition, owned by
[`PERSONAL-TEST-ENVIRONMENTS.md`](../../../../docs/plan/PERSONAL-TEST-ENVIRONMENTS.md).

Personal 2.0 Phase 11 (Personal 2.0.0 chrome) routes daily Project/Employee
authority tests to `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` (plus exact-revision
`DEV-LINUX-NATIVE-01` when a native daemon/store is required). Host/DSH native
E2E stays future qualified `DEV-WINDOWS-NATIVE-OPC-01` (`Requires-environment` /
`not-run` until provisioned). Unparked T15 still needs a preregistered N=15
acceptance at one qualified Windows revision and is not the Phase 12 mutex;
since 2026-09-02 its acceptance requires Phase 13 `P13-T02`–`T13` done plus
`P13-T13` qualifying the Windows host. Phase 12 Dual Track UI uses
`DEV-WIN-GNU-01` TS plus required CI. Phase 13 card routing is in
`PERSONAL-TEST-ENVIRONMENTS.md` §5.2 (Linux native closes "implementation
exists" only; Windows native cells are backfilled by P13-T13). The native environment and B01-W
are not provisioned; `B01-DESKTOP-002` is not the 2.0 daily default. Local
Windows GNU, WSL, ordinary CI and Canvas cannot substitute Gate/release;
record native cells `not-run` until available. `not-run` is never pass.

## Safe on every platform (including the local Windows host)

```powershell
pnpm install --frozen-lockfile
pnpm -r build
pnpm -r test
pnpm run check:consistency          # tools/src/check-consistency.mjs (links/paths are git-tracked-only; also Phase 13 build-order edge set)
node tools/src/gen-matrix.mjs --check
node tools/src/check-handbook.mjs   # handbook drift gate
node tools/src/generate-handbook.mjs --check
node tools/src/docs-sync-gate.mjs --staged   # pre-commit docs-sync gate (--push / --range)
pnpm run check:rules                # tools/src/check-agent-rules.mjs: AGENTS.md / .cursor/rules / .cursor/commands references and frontmatter (git-tracked-only paths; local-only assets warn when absent)
pnpm run hooks:install              # once per clone: registers .githooks pre-commit/pre-push
# bash hosts (Cloud Agent / Linux) bootstrap everything at once:
#   bash scripts/setup-dev-env.sh   # deps + pinned toolchain + docs-sync hooks
cargo fmt --all -- --check          # formatting only; no linking
git diff --check
node --test tools/test/p7_t05_web_ui_inventory.test.mjs  # P7-T05 route inventory; not a Gate result
node --test tools/test/personal-rc-gate.test.mjs         # P7-T06 RC binder; does not set Gate state
# SPA (this repo clients/pc/web): pnpm test; pnpm build
# Product origin is daemon GET /ui after copying dist/ into data_dir()/ui. Vite preview is not the product origin.
# After linux-002 Control Plane / dsh deploy, owner viewing is local Windows via:
#   ssh -J wuz@192.168.1.2 -L 48681:127.0.0.1:48681 -L 3080:127.0.0.1:3080 hal9001@192.168.123.160
# then http://127.0.0.1:48681/ui/ and http://127.0.0.1:3080/.
# After daemon restart, restart cognitive dsh web; apply cannot recover the new daemon's INACTIVE dsh state.
```

## Requires supported CI (Ubuntu / Windows MSVC) or exact-revision native Linux

```bash
cargo build --workspace --locked
cargo test --workspace --locked -- --test-threads=1
cargo test -p kernel-server tool_executor --locked -- --test-threads=1
cargo test -p kernel-server p4_t05_resource_api --locked -- --test-threads=1
cargo test -p kernel-server --test p8_t12_resource_manager --locked -- --test-threads=1
cargo test -p kernel-server --test p8_t13_provider_control_plane --locked -- --test-threads=1
cargo test -p cognitive-secret --test p8_t13_endpoint_trust --locked -- --test-threads=1
cargo test -p cognitive-store --test p8_t13_provider_store --locked -- --test-threads=1
cargo test -p pi-agent-adapter --locked -- --test-threads=1
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run -p cognitive-conformance --bin conformance-runner
cargo run -p cognitive-contracts --bin contracts-codegen   # then git diff generated trees
```

Never run these on the local Windows **GNU** host: the linker failure (exit 121) is a
registered environment boundary, not a signal to reproduce. Since 2026-09-03
(P0-T01/D02, owner decision) the registered local directories `D:\agent-kernel`
and the current task worktree carry `rustup override set
1.97.1-x86_64-pc-windows-msvc`; there — and only where `rustc -vV` reports
`host: x86_64-pc-windows-msvc` — the four workspace commands below run locally
as **development iteration** (recorded first in the P0-T01/D02 running report;
`CARGO_PROFILE_DEV_DEBUG=0` in the session keeps the test build inside this
machine's disk budget):

```powershell
# only inside a registered MSVC-override directory; development evidence, not supported validation
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"   # if cargo is not on PATH
rustc -vV                                              # must print host: x86_64-pc-windows-msvc
$env:CARGO_PROFILE_DEV_DEBUG = "0"                     # disk budget on this machine
cargo build --workspace --locked
cargo test --workspace --locked -- --test-threads=1
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

The tracked `rust-toolchain.toml` is unchanged and CI does not see the
override. Known local gap: the four `kernel-server` `tool_executor` symlink /
reparse-point fixture tests fail at setup with OS error 1314 in a non-elevated
shell without Developer Mode (they pass on elevated CI) — record them as
`not-run (host privilege)`, never skip them in code. `CLOUD-AGENT-LINUX-01`
can run the whole block — it is a native GNU/Linux link host — but its results are
container-class pre-CI triage, never a substitute for required CI or for
exact-revision native evidence. Remote/native validation
consumes only pushed, immutable revisions — never a copied working tree.
P2-T25 focused HTTP coverage is `personal/apps/kernel-server/tests/p2_t25_tool_lifecycle.rs`
(lifecycle, selection, and pinned HTTPS origin registry).
P2-T26 focused HTTP coverage is `personal/apps/kernel-server/tests/p2_t26_observation_plane.rs`
(O2/O3/O4/O5/O13 observation plane, controlled zeros, audit cursor negatives,
and channel negatives).
P2-T27 focused HTTP coverage is `personal/apps/kernel-server/tests/p2_t27_backup_restore.rs`
(secret-excluding backup/restore, preflight, tamper, and task-channel denial).
P2-T28 D01 freeze is `tools/test/p2_t28_capability_truth.test.mjs` against
`tools/fixtures/p2_t28_uj_matrix.json` (existing public callers/oracles;
Web UI/Multi-Agent stay explicit `excluded`). The matching daemon register is
`personal/apps/kernel-server/src/personal/capability_truth.rs` (Linux/CI only).
P2-T28 D02 public-caller smoke is `personal/apps/kernel-server/tests/p2_t28_end_to_end_journey.rs`.
P2-T28 D03 exact-revision `DEV-LINUX-NATIVE-01` aggregate runs the named UJ
oracles plus `cargo test -p kernel-server --bins`, `cargo test --workspace`,
`cargo fmt --all -- --check`, and `cargo clippy --workspace --all-targets -- -D warnings`.
P2-T30 public-admit scheduler lease coverage is the kernel-server focused test
`public_admit_c1_search_leaves_draft_only_until_scheduler_acquires_lease`
(Linux/CI only; Windows GNU `not-run`).
P2-T33 private-candidate host-path coverage is
`personal/apps/admin-cli/tests/p2_t33_private_candidate_host_path.rs` plus the
`provider_proxy` private-candidate unit tests (Linux/CI only; Windows GNU
`not-run`).
P8-T12 Resource Manager coverage is `personal/apps/kernel-server/tests/p8_t12_resource_manager.rs`
(management list/inspect/mutate, task-channel 403, generic create refuse;
Linux/CI only; Windows GNU `not-run`).
P8-T13 Provider Control Plane coverage is
`personal/apps/kernel-server/tests/p8_t13_provider_control_plane.rs`,
`personal/crates/cognitive-secret/tests/p8_t13_endpoint_trust.rs`, and
`personal/crates/cognitive-store/tests/p8_t13_provider_store.rs`
(endpoint trust/SSRF negatives, catalog preservation, Pi vs dsh binding isolation;
Linux/CI only; Windows GNU `not-run`).
P7-T05/D01 Web UI route inventory is
`tools/test/p7_t05_web_ui_inventory.test.mjs` against
`personal/docs/architecture/web-ui-route-inventory.json` (invented lifecycle,
missing daemon route, Task-channel secrets, Web storage, and browser-direct
targets fail closed). It is not a Gate or release result. Daemon Origin/Referer
and `GET /ui` serving tests live in
`personal/apps/kernel-server/src/personal/server.rs` (foreign/null Origin,
missing-bundle `not_available`, path traversal); they require supported
Rust linking (CI-UBUNTU-01 / CI-WINDOWS-MSVC-01 / DEV-LINUX-NATIVE-01).
P7-T06 Personal Linux RC binder is
`tools/test/personal-rc-gate.test.mjs` (incomplete observation, Profile keys,
enabled P6, and production publication fail closed). It does not set Gate state.
P7-T05/D08 binding CAS is `expected_revision` on
`POST /management/agent-bindings` with 409 `PROVIDER_BINDING_REVISION_STALE`
(`personal/apps/kernel-server/tests/p8_t13_provider_control_plane.rs`; Linux/CI only).
P11-T12 honest usage is `personal/crates/cognitive-store/tests/p11_t12_honest_usage.rs`
plus in-process HTTP tests in
`personal/apps/kernel-server/src/personal/provider_control_plane.rs`
(unknown≠0, labelled actual|estimated|unknown, silent rebind
`PROVIDER_SILENT_REBIND_REJECTED`, secrets omitted; Linux/CI only; Windows GNU `not-run`).
SPA unit tests live in the external clients checkout `pc/web` (`vitest run`);
they are not kernel CI and not live SecretStore proof.

## What CI enforces on every PR

The `verify` matrix (Ubuntu + Windows MSVC) in
[`ci.yml`](../../../../.github/workflows/ci.yml): TypeScript build/test, Rust
build/test/clippy/fmt, codegen drift diff, consistency check, traceability freshness,
agent-rule reference check (`check-agent-rules`), handbook check and generated-page
byte equality, conformance runner with pinned five-state counts and
evidence-honesty assertions, wrong-implementation self-check, cross-language golden
digest byte equality.

## Known stale entry

`pnpm run verify:local` (the V01 orchestrator, `scripts/v01-auto-run.*`) pins
outdated conformance counts (85/60/25 versus the 89/62/27 pinned in `ci.yml`)
and is not a usable local gate at this baseline; prefer the individual commands
above. Whether it is re-pinned or removed is the open `P0-T01/D02` owner
sub-decision.
