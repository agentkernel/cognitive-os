# Lane-RUN / CFR M6 delivery handoff

- Date: 2026-07-21
- Branch: `lane/run-m6-installer` (consumes CTR `013b4d7`)
- REQ: REQ-AGENT-INSTALL-001/002, REQ-AGENT-ADAPTER-001, REQ-AGENT-SANDBOX-001, REQ-AGENT-COMPLETE-001, REQ-AGENT-OOB-001, REQ-PERF-004, REQ-CONF-001/003
- Related: F-017 (still open), D-020/D-021 (honored), F-011 (regress only)

## Delivered (实现已提供 + 部分测试已执行)

### Runtime (`cognitive-runtime`)

- `installer.rs` — package verify + install orchestration; staging invisible; crash seams
- `sandbox.rs` — platform rows + channel gate + merge refuse
- `adapters.rs` — six families; batch proxy registration; candidate completion only
- `oob.rs` — first-read digest drift → candidate
- `readiness.rs` — MANAGEMENT→USER→OPERATIONAL + R0 non-degradable bounds (milestone only)
- `perf.rs` — REQ-PERF-004 overhead sample + digest; no benefit claim

### Conformance (`cognitive-conformance`)

- `behavior_m6.rs` — AGENT-INSTALL-001 / AGENT-BYPASS-002 / AGENT-OOB-001
- Pins: **pass 55 / not-run 29**; self-check must_flip **36**
- `release_candidate_profile_manifest` + runner output path

### Docs

- `docs/traceability/f017-platform-matrix.md`
- `docs/checkpoints/20260721-m6-milestone-review.md` → **NO-GO** (F-017)

## Not done / known limits

- No installation transition table consumption (D-020)
- No readiness registry carrier (D-021)
- Installation ledger is in-process (not KRN SQLite InstallationStore)
- F-017 platform CI evidence incomplete → v0.1 **NO-GO**
- D-018 residual unchanged

## Suggested next prompts

1. Platform CI evidence campaign for F-017 (CFR workflow-scope PR after permission)
2. Optional KRN InstallationStore if durable install authority is required
3. Re-open M6 milestone review only after F-017 rows have digests
