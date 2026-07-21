# Run evidence directory

Machine-produced evidence lands here: conformance runner reports, sample
profile manifests, fault-injection logs, performance reports, golden digest
emissions. Everything in this directory except this README is gitignored —
evidence is referenced by digest from committed documents
(`docs/standards/conformance-evidence.md`), never committed as mutable files.

Layout convention:

```text
artifacts/evidence/
  conformance/     conformance-report.json, sample-profile-manifest.json,
                   self-check-report.json (wrong-implementation self-check),
                   release-candidate-profile-manifest.json
  golden/          rust-digests.json, ts-digests.json (CI cross-language diff)
  performance/     performance reports (REQ-PERF-004/005); v0.1 auto-run writes
                   performance-report-v01-sample.json (sample/builder only;
                   campaign=not_executed — see docs/plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md)
  faults/          fault-injection and crash-recovery evidence (from M4)
  v01-auto-run/    per-run orchestrator evidence (<run_id>/summary.json, summary.md,
                   sha256-manifest.json, stage logs). Produced by:
                   `pnpm run verify:local`
```

Reproduce the conformance artifacts locally:

```powershell
cargo run -p cognitive-conformance --bin conformance-runner
cargo run -p cognitive-conformance --bin conformance-runner -- --self-check
```

One-shot Boot→Connect→Verify→Perf (non-claim) local gate:

```powershell
pnpm run verify:local
```

The runner prints each emitted report's sha256; committed documents cite
evidence by path + digest (`docs/standards/conformance-evidence.md` §4).

A file in this directory is NOT a conformance claim. Claims live only in a
profile manifest whose `test_runs` reference digest-pinned evidence
(see `conformance/README.md`, status language). Auto-run L0–L3 green does
**not** mean Profile `implemented`.
