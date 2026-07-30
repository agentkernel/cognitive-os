# Personal P1-T09 deterministic Provider fixture handoff

- Date: 2026-07-30
- Task: P1-T09 install-to-first-conversation route
- Classification: implementation-only; normative surface unchanged
- Implementation commit: `96844b8`
- Branch: `lane/personal-p1-t09-provider-fixture`
- Remote visibility: branch pushed at `c608dd7`; pull request creation is
  pending because the GitHub GraphQL request returned EOF

## Completed

1. Added `p1-t09-provider-fixture`, a deterministic loopback-only HTTPS binary
   that publishes an ephemeral `localhost` endpoint and a test-only DER trust
   root through explicit files. It supports ready, malformed catalog,
   unauthorized, non-chat-capable, timeout, oversized-response, and redirect
   scenarios. It records only method/path and authorization presence, never
   request headers, bodies, or secret-like values.
2. Added an implementation-local additional-root constructor to
   `RustlsProviderTransport`. Production defaults remain unchanged: HTTPS-only,
   credential-free URLs, no redirects, header-injection rejection, bounded
   timeout/cancellation, Rustls, and the 1 MiB response limit remain enforced.
3. Added failure-first process-level tests that run the real Rustls transport
   and `ProviderDiscoveryService`, assert exact model selection and capability
   outcomes, verify deterministic request counts, exercise malformed/status/
   timeout/size/redirect negatives, and check that Provider state, diagnostics,
   and fixture observations do not contain the synthetic secret marker.
4. Reconciled the formal P1-T09 row, current progress snapshot, and active
   Lane-RUN lease. No registry, schema, public DTO, registered error,
   transition, generated contract, conformance vector, Pi argv/environment,
   Task, Effect, Verification, capability, or authority writer was changed.

## Failure-first evidence

The new focused test was written before the fixture and transport seam. The
first attempted command was:

```text
cargo test -p cognitive-provider-transport --test p1_t09_deterministic_provider_fixture --locked
```

It exited 101 before repository tests started because the local unsupported
Windows GNU linker exited 121 while building dependencies. After the
implementation, the same local environment also blocked `cargo check` before
type checking for the same linker reason. WSL was inspected for a supported
toolchain but has no `cargo` available. Therefore the intended focused suite
is explicitly **not-run**, not pass evidence.

## Verification

- `cargo fmt --all -- --check` — passed.
- `git diff --check` — passed.
- IDE lint diagnostics for the three changed Rust source/test paths — none.
- `cargo test ...p1_t09_deterministic_provider_fixture...` — not-run to
  completion; unsupported Windows GNU linker exit 121 before tests.
- `cargo check -p cognitive-provider-transport --tests --locked` — not-run to
  completion; same linker limitation.
- WSL Linux focused tests — not-run; WSL has no Cargo/Rust toolchain.
- Strict Clippy — not-run for the same unsupported local toolchain limitation.
- `pnpm run check:consistency` — not-run; this implementation-only batch did
  not modify normative assets, but supported CI remains required evidence.
- Secret scan, full workspace regression, and real Pi load — not-run.

## Status and explicit non-claims

- `P1-T09`: `in-progress`; `experimental-local-only`; prior route evidence
  remains `tested-local`, while this fixture focused execution is `not-run`.
- `B01`: `not-run`.
- `GMVP-LINUX`: `not-run`.
- Profile `implemented`: `0`.
- This batch does not claim a real Provider conversation, real pinned Pi
  Extension load, native Secret Service campaign, clean Linux VM campaign,
  B01, release, GMVP-LINUX, or Profile conformance.
- The fixture cannot create Task, Effect, Verification, capability, or
  authority side effects; its process boundary is only a Provider test seam.

## Lease and next entry

- The P1-T09 Lane-RUN lease is closed for this atomic slice after the closure
  docs are committed. Its paths were limited to the Provider transport crate,
  `Cargo.lock`, and the listed plan/progress/handoff documents.
- Next entry: create the pull request, inspect required CI, and run the focused
  fixture suite on supported Linux. Fix any CI findings before promoting
  implementation evidence. Then add the smallest separate real pinned Pi
  Extension-load slice; do not run B01 until all route acceptance prerequisites
  and campaign pre-registration are complete.
        - docs/checkpoints/20260730-personal-p1-t09-provider-fixture-handoff.md
| P1-T09 deterministic binary Provider fixture | Lane-RUN | `lane/personal-p1-t09-provider-fixture` | `crates/cognitive-provider-transport/**`, `Cargo.lock`, `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`, `docs/plan/PROGRESS.md`, `docs/plan/personal-trace.yaml`, `docs/plan/PARALLEL-LANES.md`, `docs/checkpoints/20260730-personal-p1-t09-provider-fixture-handoff.md`; secondary local control-plan path `C:/Users/wuron/.cursor/plans/personal_linux_mvp_fd48be65.plan.md` | current Provider fixture session | 2026-07-30 / 2026-07-30 | closed; implementation-only, normative surface unchanged; handoff above |
| Active task lease | P1-T09 deterministic binary Provider fixture lease closed in [handoff](../checkpoints/20260730-personal-p1-t09-provider-fixture-handoff.md) | no active P1-T09 writable lease; normative assets remain Lane-CTR-owned | re-claim a task-correct lease after CI result, then run/fix the focused fixture suite |
