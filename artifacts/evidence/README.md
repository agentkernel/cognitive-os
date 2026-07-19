# Run evidence directory

Machine-produced evidence lands here: conformance runner reports, sample
profile manifests, fault-injection logs, performance reports, golden digest
emissions. Everything in this directory except this README is gitignored —
evidence is referenced by digest from committed documents
(`docs/standards/conformance-evidence.md`), never committed as mutable files.

Layout convention:

```text
artifacts/evidence/
  conformance/     conformance-report.json, sample-profile-manifest.json
  golden/          rust-digests.json, ts-digests.json (CI cross-language diff)
  performance/     performance reports (REQ-PERF-004/005, from M6)
  faults/          fault-injection and crash-recovery evidence (from M4)
```

Reproduce the conformance artifacts locally:

```powershell
cargo run -p cognitive-conformance --bin conformance-runner
```

A file in this directory is NOT a conformance claim. Claims live only in a
profile manifest whose `test_runs` reference digest-pinned evidence
(see `conformance/README.md`, status language).
