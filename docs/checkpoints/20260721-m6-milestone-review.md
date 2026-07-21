# M6 / v0.1 Milestone Review

- Date: 2026-07-21
- Baseline tip consumed: Lane-CTR `013b4d7` (bindings + D-020/D-021) atop plan `b7bb68e`
- Reviewer lane: Lane-DOC (consuming RUN+CFR delivery evidence)
- Verdict: **NO-GO for v0.1 release** (hard blockers remain)

## 1. Entry check (`M6-ENTRY`)

| Check | Result |
|---|---|
| M5 GO with conditions | pass (prior review) |
| Contract freeze adjudications D-020/D-021 | recorded (no new transition/readiness carriers invented) |
| Pins after this batch | **pass 55 / not-run 29 / self-check ≥36** (measured locally; CI pending merge) |

## 2. M6 acceptance matrix

| ID | Result | Notes |
|---|---|---|
| M6-A1 tamper/signature refuse install | **pass** (behavior) | `AGENT-INSTALL-001` → behavior pass; in-process `InstallationLedger` |
| M6-A2 adapter/sandbox bypass | **pass** (behavior, Linux unit) | `AGENT-BYPASS-002` pass; Windows native not claimed |
| M6-A3 install crash atomicity | **pass** (unit) | crash seams in runtime ledger; **not** yet SQLite `InstallationStore` (KRN residual) |
| M6-A4 OOB reconciliation | **pass** (behavior) | `AGENT-OOB-001` pass |
| M6-A5 readiness order | **pass** (milestone unit) | evaluator + R0 boundaries; **not** registry conformance (D-021) |
| M6-A6 PERF overhead baseline | **partial** | report builder + digest + ungoverned baseline declared; not full HW campaign |
| M6-A7 real profile manifest | **pass** (honesty) | RC manifest binds report digest; profiles ≤ `experimental`; **zero** `implemented` |
| M6-F017 platform matrix | **fail / open** | see [f017-platform-matrix.md](../traceability/f017-platform-matrix.md) — **exit hard block** |
| M6-F011-REG | **pass** (regression retained) | prior M5 three negatives remain in pin set |
| M6-D018 | **open / residual** | not closed this batch; scope risk deferred |

## 3. No-Go triggers hit

- **F-017**: platform/channel evidence incomplete; Windows native unsupported without native backend — cannot GO.
- Installation authority persistence is still in-runtime ledger (honest limitation), not store-backed KRN port — soft for A3 unit, hard if release claims durable install authority.

## 4. Honesty audit

- Sample manifest remains all-`planned`.
- Release-candidate manifest may mark `core_digital` / `agent_compatibility` as `experimental` only; never `implemented`.
- PERF path does not emit REQ-PERF-005 benefit claims.
- No installation transition table or readiness carrier added (IMP-01 / D-020 / D-021).

## 5. Follow-ups before re-review

1. Lane-CFR/CI: land platform matrix jobs + digests; close or narrow F-017 claims.
2. Lane-KRN: optional durable `InstallationStore` if release requires crash-surviving install authority beyond process memory.
3. Lane-DOC: re-run this checklist; only then consider GO / GO-with-explicit-non-claim.

## 6. Allowed non-claims (while NO-GO)

- Windows-native sandbox
- Profile `implemented` for any profile
- Agent benefit (REQ-PERF-005)
- R2/R3, distributed, clients implementation, Agent Hub
