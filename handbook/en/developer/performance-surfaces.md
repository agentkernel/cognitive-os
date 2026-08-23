---
doc_id: dev.performance-perf
locale: en
kind: concept
audience: [developer]
status: partial
generated: false
sources:
  - path: crates/cognitive-runtime/src/perf.rs
    symbols: ["GovernanceOverheadSample", "GovernedPathStageCollector"]
  - path: crates/cognitive-runtime/src/performance_campaign.rs
  - path: crates/cognitive-runtime/src/campaign_runner.rs
  - path: crates/cognitive-runtime/src/loopback_transport.rs
  - path: crates/cognitive-runtime/src/resource_sampler.rs
  - path: crates/cognitive-runtime/src/provider_route_policy.rs
  - path: crates/cognitive-runtime/src/task_scenario_harness.rs
  - path: packages/dsh-akp-adapter/src/index.ts
    symbols: ["DshAkpTiming"]
  - path: packages/dsh-akp-adapter/src/plugin.ts
    symbols: ["applyDshAkpCordisPlugin"]
  - path: packages/dsh-akp-adapter/scripts/dsh-real-process.mjs
  - path: packages/dsh-akp-adapter/scripts/dsh-web-preflight.mjs
  - path: packages/dsh-akp-adapter/scripts/paired-path.mjs
tests:
  - crates/cognitive-runtime/src/bin/p7_t04_module_benchmark.rs
fingerprint: "sha256:2ee1f60aba89c746bdeb9bcd989f3123d56f664718a4e670b994df5607d7a3e2"
non_claims:
  - Every surface here produces hypothesis-level, non-claim observations only; no benefit, Gate, release, or Profile result is created by this code, and campaign execution results are owned by the formal plan's evidence records.
---

# Performance surfaces

All performance code in this repository is **measurement plumbing with fail-closed
honesty rules**, never a benefit claim. Two generations exist:

## P7-T04 generation (regression floors)

`perf.rs`: `GovernanceOverheadSample` (fixed governed-path stage vocabulary),
deterministic module benchmarks (`p7_t04_module_benchmark` binary,
`COGNITIVEOS_BENCHMARK_SAMPLES`), stage collectors asserting complete
disjoint coverage of a governed exchange, and a module regression-floor policy
consumed by later structure work. `p9_t01_async_decision_gate` produced the
hypothesis-only "conservative-no-migration" async decision.

## P9-T04 generation (comprehensive campaign, ADR-0051)

Added by the campaign task and consumed by preregistered runs, not by the daemon:

- `performance_campaign.rs` + `campaign_report.rs`: typed L0–L5 campaign policy —
  retained-denominator accounting, eight hard safety counters (any nonzero ⇒
  outcome cannot promote), cleanup facts, claim ceiling forced to `hypothesis`
  without independent verification, `benefit_claimed=false` unless a completed
  A/B arm exists.
- `campaign_runner.rs` + the `p9_t04_l0_l1_campaign_runner` binary: admission
  refuses secret-shaped environment/arguments and unregistered environments;
  reports reject unredacted or self-promoted observations.
- `loopback_transport.rs`: decomposes the real loopback front door into disjoint
  stages, explicitly disclaiming `effect_persistence`, `provider_network`,
  `pi_process_launch`, `scheduler_wait`.
- `resource_sampler.rs`: bounded `/proc` sampler that never opens
  `cmdline`/`environ`, never resolves descriptor targets, and treats decreasing
  cumulative counters as PID reuse.
- `provider_route_policy.rs`: L3 rules — `retry=0`, every started request stays a
  classified outcome, no fabricated TTFT/cost, usage `not_available` unless
  complete counters exist.
- `task_scenario_harness.rs`: L4 governed-Task scenarios decided by a frozen
  oracle plus independent acceptance; a read-only scenario that mutated anything
  is a boundary violation outranking every other result.

`tools/personal/` holds the operator-driven runner scripts (smoke, L3 route, L3
cold journey, L4 T1). Campaign **results** (which cells ran, retained counts,
digests) live in the formal plan's evidence records — link, never copy.

## dsh AKP adapter timing (P8-T09 / P8-T11)

`@cognitiveos/dsh-akp-adapter` records serialization, transport, and total
durations on each candidate-only submit. Those fields are measurement hooks
for paired Path A (dsh → DeepSeek Flash) versus Path B (dsh → AKP → daemon →
Flash) observation. They do not claim zero overhead, losslessness, or any
Gate/release/Profile/B01/Agent-benefit result. The linux-002 harness
`scripts/linux002-e2e.mjs` records those timings on live shim submits and waits
for Task `COMPLETED`; `scripts/dsh-real-process.mjs` records real dsh process
elapsed time and first stdout (TTFT hook) through the daemon Provider SSE
proxy at `POST /provider/v1/dsh/chat/completions` (Path B) or direct Flash (Path A). Native web Path B also persists a settings overlay and aliases the official catalog key ref to the daemon bearer so the Models page does not require a second dsh-local key. The helper prefers compiled
`apps/cli/lib/bin.js` when `build:lib` outputs exist; tsx-from-source on a
2 vCPU guest was previously ~10 s of harness bootstrap.
`scripts/provider-raw-probe.mjs` measures the same host without dsh;
`scripts/paired-path.mjs` repeats Path A/B
on one host. Workspace* `startupEvents` are still candidate events. None of
this is a Gate sample.

Status `partial` because the daemon itself exposes no continuous performance
instrumentation; everything here is opt-in measurement tooling.
