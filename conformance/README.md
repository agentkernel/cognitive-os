# CognitiveOS Conformance Assets

This directory contains machine-readable conformance vectors. It is a data package, not an executable runner, and its presence does not claim that any implementation or Profile conforms.

## Fifteen test layers

1. Wire/schema and version negotiation.
2. State-machine, CAS and conflict handling.
3. Effect, idempotency and crash recovery.
4. Security negatives and information-flow isolation.
5. Context resolution and semantic boundaries.
6. Harness, Loop, progress and Verification.
7. Knowledge compilation and invalidation.
8. Performance and reproducibility contracts.
9. Privileged management session and deterministic fallback.
10. Agent installation, adapters and sandbox interception.
11. Governed memory admission and lifecycle.
12. Cognitive discovery, delta and stagnation.
13. Operation catalog lifecycle, match and binding.
14. Semantic mediation and CRB hard-bound enforcement.
15. User intent, Agent Shell target/preview/channel/control/watch and acceptance semantics.

测试层编号是单一累积分类，不表示存在实现或通过结果。

Layers 7 and 8 keep no dedicated vector `layer` slug (documented disposition
of drift D-004, findings ledger): their scenarios are genuinely
cross-cutting and are hosted under other slugs. The pinned mapping (kept in
sync with `cognitive-conformance::CROSS_SLICE_HOSTED`):

- Layer 7 (knowledge compilation and invalidation):
  `KNOW-INVALIDATION-001` (`context-semantic`), `KNOW-POISON-001`
  (`security-negative`), `KNOW-MAINTENANCE-001` (`harness-loop`).
- Layer 8 (performance and reproducibility contracts):
  `PERF-REPORT-CONTRACT-001` (`wire-schema`).

Runner reports list these vectors under their primary slug's layer and
additionally show them as `cross_slice_hosted` on layers 7/8, so the zero
dedicated-slug count is never read as zero coverage.
## Manifests and status language

A conformance claim is represented by `specs/schemas/profile-manifest.schema.json`. A manifest pins the specification, requirement and schema bundle digests, encoding/canonicalization profile, suite digest, implementation version, results, degradations, and evidence. Profile values have distinct meanings:

- `implemented`: every applicable MUST requirement has passing behavioral evidence or a justified `not-applicable` determination. A documented degradation on an applicable MUST does not preserve `implemented`: it must either shrink the declared scope so the requirement becomes not-applicable, or downgrade the profile claim to `experimental`. Safety-negative requirements can never be degraded away.
- `planned`: intended future work; excluded from conformance coverage.
- `experimental`: available for evaluation but excluded from conformance coverage.
- `unsupported`: no support is claimed.

The registry status `specified` means a normative requirement and its test mapping are defined. It does not mean a runner or implementation exists. A future execution system may additionally track `implemented` for runnable tests; it must not infer that state from these vectors.

## Intelligent Management Shell assets

`specs/schemas/privileged-management-session.schema.json`, `management-action-proposal.schema.json`, and `management-approval-decision.schema.json` define the signed session, proposal, and approval data contracts. The `management-*.json` vectors are declarative scenarios for the optional `intelligent_management_shell` profile, including negative authorization cases and deterministic fallback behavior. They do not provide or imply a Shell, Management API, CLI, test runner, or conforming implementation.

The management error codes used by expected denials are registered in `specs/registry/errors.yaml`. Every vector ID is mapped from its normative `REQ-MGMT-*` entries in `specs/registry/requirements.yaml`; a runner must verify those gates and observable outcomes rather than treating a schema-valid document as a pass.

## Performance and knowledge assets

`specs/schemas/performance-report.schema.json` defines the report contract. `vectors/performance-report-contract.json` is a declarative schema example; knowledge vectors exercise invalidation, poisoning isolation, and bounded maintenance. These files are evidence formats and test cases, not measured implementation results. A profile manifest uses `cognitiveos_conformance.performance_reports` to reference digest-pinned reports.

The following registered requirements are normatively owned by this document (metric semantics and the BenchmarkManifest input list are described informatively in the whitepaper §19.4):

[REQ-PERF-002] An implementation MAY publish profile-specific dashboards or multi-objective Pareto frontiers, but it MUST NOT use a single universal composite score to mask safety failures, denial paths, p95/p99 tail latency, risk-class breakdowns, or unknown-outcome counts.

[REQ-PERF-004] Performance claims for governed workloads MUST report the Governance overhead metric family (authorization / context-resolution / effect-protocol stage latencies, cache-hit preservation ratio, extra persistence per governed call, approval latency and rubber-stamp rate, overhead share of end-to-end latency and cost by risk class) and MUST declare the ungoverned baseline used; if governance overhead data is missing, the report MUST NOT claim that governance overhead is negligible.

Agent benefit claims are additionally governed by [REQ-PERF-005] in [docs/evaluation/agent-benefit-benchmark.md](../docs/evaluation/agent-benefit-benchmark.md): a claim of significant agent benefit MUST be supported by the four-arm (native / governance-only / optimized / ablation) design with preregistered thresholds, and non-inferiority MUST NOT be reported as performance improvement.

The current namespace is `cognitiveos.*` and the manifest root is `cognitiveos_conformance`. Legacy `agentos.*` or `agentos_conformance` documents require an explicit old schema/adapter and cannot be silently mixed.

## Running

The reference runner is `crates/cognitive-conformance` (M4 capability:
static-contract execution plus kernel-behavioral execution, including
fault-injected crash recovery).
`vectors/*.json` stay declarative inputs and expected outcomes usable by
any runner author.

```powershell
cargo run -p cognitive-conformance --bin conformance-runner
cargo run -p cognitive-conformance --bin conformance-runner -- --self-check
```

The default mode enumerates every vector, executes the statically decidable
subset against deterministic reference gates grounded in the registered
machine assets (schema validation, registry traceability, performance-report
contract), executes the M2 kernel-backed vectors behaviorally against the
real `cognitive-kernel` transition engine over the `cognitive-store` SQLite
WAL authority adapter (stale CAS rejection, illegal Effect `OUTCOME_UNKNOWN`
exits, forced remote-completed acceptance), executes the M3
governance/context vectors behaviorally against the
`authz`/`context`/`context_cache`/`capability` surface (lateral-read denial
isomorphism, attenuation arithmetic, revocation-bound caches,
filter-before-rank, budget fail-closed, prefix-stable render, bounded
stagnation, candidate narrowing, prompt-injection isolation), executes the
M4 effect/recovery vectors through the public fault-injection framework
(`cognitive_store::faults`: CrashHarness drop-and-reopen crashes and a
scripted external executor — three crash points, unknown-outcome
quarantine, idempotency-conflict refusal, reconcile-before-resume), and
writes a five-state machine report plus the sample profile manifest to
`artifacts/evidence/conformance/` (gitignored). Vectors whose expectations
require runtime behavior of later milestones are reported `not-run` with a
recorded reason (`docs/standards/conformance-evidence.md` section 2). A
`pass` is scoped to its recorded execution mode and is never a Profile
conformance claim.

`--self-check` executes a deliberately wrong implementation (schema-valid
outputs, wrong behavior: a gate-bypassing direct store writer, governance
anti-patterns such as rank-before-auth and stale cache serving, and
effect/recovery anti-patterns such as fresh-key re-minting after a crash
and blind re-dispatch on unknown outcome) and exits non-zero unless the
runner fails every corrupted vector
(`docs/standards/conformance-evidence.md` section 3).

Schema `$id` policy: every schema under `specs/schemas/` declares a top-level `$id` exactly equal to its own file name (for example `"$id": "effect.schema.json"`). The file name is therefore the retrieval URI, and a relative `$ref` such as `common-defs.schema.json#/$defs/digest` resolves from the containing schema file without any base-URI rewriting. Consumers MUST NOT depend on absolute schema URLs.

Do not report a vector as passed merely because the JSON parses. A runner must execute the stated input against an implementation, compare the observable result with `expected`, preserve evidence, and report pass, fail, not-applicable, or documented-degradation for each applicable requirement.


## Shell and intent assets

User-intent, Shell proposal/preview and watch schemas define machine contracts. Shell/intent/lifecycle vectors cover ambiguity, correction fencing, channel isolation, detach/attach, cancellation, watch replay, acceptance and Effect-state closure. They are declarative scenarios, not a Shell or runner.
