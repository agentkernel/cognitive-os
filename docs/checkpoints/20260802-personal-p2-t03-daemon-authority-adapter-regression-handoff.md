# P2-T03 daemon-authority-adapter regression handoff

- Date: 2026-08-02
- Task: P2-T03 durable scheduler, lease and timer
- Lease: `lease/personal/P2-T03/daemon-authority-adapter-regression`
- Branch: `main`
- Change class: implementation-only
- Task status: `in-progress`
- Development track: `experimental-local-only`
- Implementation evidence: `provided`
- Normative surface: unchanged

## Delivered slice

`apps/kernel-server/src/personal/scheduler_authority.rs` now centralizes the
fail-closed parsing of scheduler-dispatchable TaskContracts. It reads the
version envelope before generated-contract deserialization, preserves v0.1
rows for audit, and rejects them as `LegacyContract` before they can enter the
scheduler path. Current v0.2 contracts still require the complete generated
shape; incomplete rows return `MalformedContract`.

The focused unit coverage protects both boundaries. It asserts that a minimal
v0.1 row is rejected before execution-binding parsing and that an incomplete
v0.2 row cannot bypass generated binding validation. The helper is used by
`load_scheduler_ceiling_facts`; it does not acquire a lease, dispatch a worker,
write a Loop transition, mutate an Effect, or alter a budget.

## Validation

| Check | Result |
|---|---|
| `git diff --check` | pass |
| `cargo fmt --all -- --check` | pass |
| `pnpm run check:consistency` | pass; 273 requirements, 55 error codes, 63 schemas, 85 vectors, links, traceability, Personal plan/Gates, design sources, prompt boundary and leases verified |
| `cargo test -p kernel-server scheduler_authority::tests` | not-run to completion: Windows GNU dependency build failed before compiling this crate because `x86_64-w64-mingw32-gcc` exited 121 |
| Linux host qualification probe | pass: `wuz@192.168.1.2` reports Linux x86_64, Rust/Cargo 1.97.1; non-claim environment evidence only |
| Linux focused test | not-run: `/home/wuz/agent-kernel` is a July no-Git source snapshot and lacks `apps/kernel-server/src/personal/scheduler_authority.rs`; copying uncommitted local code or testing stale sources was refused |
| `cargo check -p kernel-server` | not-run: local Windows GNU linker limitation; the qualified Linux host lacks the current source snapshot |
| clippy and protected CI | not-run; requires a supported local toolchain or pushed PR |

## Remaining work

- `blocked_paths`: no code path is blocked for a future scoped authority or
  worker slice; current-source validation is blocked until a current reviewed
  source snapshot exists on the qualified Linux host, while the local Windows
  GNU test path remains unavailable.
- `blocked_task_ids`: none.
- `blocked_gate_ids`: B02, B04, B05, B12, GMVP-LINUX.
- owner: next P2-T03 Lane-CTR/KRN or Lane-RUN session.
- environment owner: repository operator for provisioning or synchronizing a
  reviewed current source snapshot on `wuz@192.168.1.2`.
- next action: establish Loop-scoped, fenced dispatch-disablement and scoped
  pending-Effect closure evidence inside the authority transaction; provision
  a reviewed current Linux source snapshot before its focused test; only then
  wire scheduler ceiling outcomes to durable STOP before any worker lease.

## Non-claims

This slice adds no durable STOP handling, worker dispatch, BoundedHarness
integration, Effect closure, Gate result, release claim, or Profile claim.
The daemon remains the sole authority writer.
