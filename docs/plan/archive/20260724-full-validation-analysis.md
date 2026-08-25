# 2026-07-24 Full Validation and Product Analysis

## Verdict

This report records a clean-worktree validation of
`origin/main@a5e179caacb31b4971df47429c160485f3672d3c` on Windows native.
The reference implementation is buildable and locally runnable as an Ordinary
Core single-node loopback service. Its Boot -> Connect -> Verify ->
Perf-report-ready automation reached **L3** with no automatic failures.

This is not a production-release or Profile-conformance conclusion. Release is
`non_claim_preserved`, Profile implemented remains 0, and the performance
result is a schema-validated sample rather than a hardware campaign.

## Evidence

| Area | Result |
| --- | --- |
| Rust workspace | `cargo test --workspace` passed with the pinned Rust 1.97.1 toolchain and LLVM-MinGW linker. |
| Rust quality | `cargo build --workspace`, strict clippy, format, and diff checks passed. |
| TypeScript | Locked offline install, recursive build and tests passed: SDK 69 pass / 3 skipped; Agent Shell 13 pass. |
| Consistency | 273 requirements, 55 error codes, 63 schemas and 85 vectors; matrix current. |
| Conformance | 85 vectors: 60 pass, 25 not-run, 0 fail; report digest `fa26a8c64e768630754102c6d1cdbc5577a2b123d8debcc21418f1f83a9e4f12`. |
| Negative control | 41 deliberately corrupted gates all flipped to fail; digest `b135371c82cb16fa59cfac630cebf3dec212727b6627ccd4d1248fb1b879fe88`. |
| Transport stability | The `kernel-server` HTTP/SSE suite passed 10 consecutive 4/4 rounds before this campaign. |
| Local operating chain | `pnpm run verify:local` run `20260724-011357-998fe139`: L3, `stopped=false`, no auto failures. |

Evidence is local and intentionally not a source-controlled release claim:
`artifacts/evidence/v01-auto-run/20260724-011357-998fe139/summary.json`.

## What is working

- Deterministic authorization, state transitions, CAS, fencing, idempotency,
  effect recovery and audit-before-result have Rust and behavior-vector
  evidence.
- The local management and transport path builds `kernel-server`, exercises
  deterministic management fallback, SDK HTTP live tests and loopback HTTP/SSE
  watch behavior.
- Malformed management envelopes and unconfigured paths fail closed; loopback
  serving is enforced; corrupted implementations are rejected by self-check.
- Registry/schema traceability, matrix freshness and generated artifact
  consistency are checked.

## Non-claims and remaining gaps

| Boundary | Honest status |
| --- | --- |
| Profile conformance | 0 implemented; 25 vectors remain not-run. |
| Enterprise deployment | Not established: evidence is one local loopback process, not an authenticated multi-node deployment. |
| Windows sandbox | Unsupported; WSL2 is not tested. |
| Durable installation authority | In-process-ledger non-claim. |
| REQ-PERF-004 | Full preregistered hardware campaign not executed. |
| REQ-PERF-005 | Benefit claim forbidden: no M7 mechanism, W1/W2 A/B/C/D harness, preregistration or independent verifier. |
| High-Assurance | Deferred from Ordinary Core: external verifier, detached signatures, retention/legal hold and independent review remain outside this evidence. |

## Performance assessment

The test proves performance-report integrity, not performance attainment. The
sample requires an ungoverned baseline and forbids silently asserting benefit,
but it does not measure authorization, Context or Effect p50/p95/p99 on
declared hardware. REQ-PERF-004 needs L2-green reference hardware, fixed
topology/concurrency/baseline, measured persistence and latency metrics, and a
digested report. REQ-PERF-005 additionally needs a preregistered four-arm study
over W1 and W2 with an independent verifier. No all-design-standards
performance claim is currently supportable.

## Tool correction made in this batch

The first automated run stopped at L0 although its isolated Rust build passed.
It revealed two tooling defects: the binary resolver ignored
`CARGO_TARGET_DIR`, and its pins still said 84/55/29 with a 36 self-check floor
instead of the runner's 85/60/25 and 41. The resolver now supports default,
absolute and repository-relative targets; pins are refreshed from the measured
runner. The same isolated target then completed L3.

## Continuation

1. Lane-RUN: real configured server startup using a validated privileged
   management session and `SqliteAuthorityStore`, retaining fail-closed
   unconfigured requests.
2. D-018 durable governance-object resolution and InstallationStore evidence.
3. Portable disk-full/store-degradation injection and its remaining vectors.
4. A Linux-native REQ-PERF-004 campaign only after hardware/baseline
   preregistration. M7 evaluation infrastructure before any REQ-PERF-005
   benefit claim.
