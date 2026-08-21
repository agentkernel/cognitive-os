# PERSONAL-PERF-EVAL-014 — preregistration

- Campaign ID: `PERSONAL-PERF-EVAL-014`
- Status: **closed** (2026-08-20). Evaluation routing **OFF**.
- Lease: `lease/personal/EVAL-014/execution-plan-b0` (**closed**)
- Branch: `evaluation/EVAL-014-freeze`
- Claim ceiling: `hypothesis` / non-claim. Reviewer: `not_reviewed`.
- Parent: [personal-performance-benchmark-execution-plan.md](./personal-performance-benchmark-execution-plan.md)
  v1.1. C1/C2 overlay:
  [PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md](../plan/PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md).
- Freeze checkpoint:
  [20260820-personal-perf-eval-014-preregistration.md](../checkpoints/20260820-personal-perf-eval-014-preregistration.md)
- Running report:
  [personal-performance-assessment-20260820-eval-014.md](./personal-performance-assessment-20260820-eval-014.md)

This is a **new freeze**. Closed EVAL-002 and EVAL-004 through EVAL-013 are
never resumed.

Owner 2026-08-20 activated this campaign by directing continuous autonomous
progress after P9-T12 closure (live paired executor). The campaign is now
**closed**. Do not auto-claim P6, P7-T05, P7-T06, or P7-T07.

## 1. Activation gate

Activation requires **all** of:

1. Product follow-ups that would otherwise force B1/B2 `not-run` are closed:
   P2-T38, P9-T09, P9-T10, P9-T11, **P9-T12**. **pass.**
2. Owner sets the Current snapshot `Owner-directed campaign` row to
   `PERSONAL-PERF-EVAL-014`. **pass** (owner: continue autonomously after
   P9-T12; 2026-08-20).
3. An evaluation lease `lease/personal/EVAL-014/<purpose>` owns only
   `docs/evaluation/`, `docs/checkpoints/`, and `docs/plan/PROGRESS.md`.
   **pass** (`lease/personal/EVAL-014/execution-plan-b0`).

## 2. Isolation (bind at activation)

| Resource | Bound value |
|---|---|
| Guest | `B01-Desktop-Linux-002` only |
| Route | `wuz@192.168.1.2` → ProxyJump `hal9001@192.168.123.160` |
| Root | `/home/hal9001/perfeval014-20260820` |
| Daemon | `127.0.0.1:48304` |
| P-arm broker | `127.0.0.1:48404` |
| SecretStore | new item `/26` (≠ `/12`–`/19` / `/24` / `/25`) |
| Git revision | `adc404990974f6fbda91d57a5556d78a145ef596` (pushed `origin/main` after P9-T12 lease close). Product merge: PR [#252](https://github.com/agentkernel/cognitive-os/pull/252) at `39cf8019`. |

Procedure:
[20260820-personal-c1-c2-b01-guest-procedure.md](../checkpoints/20260820-personal-c1-c2-b01-guest-procedure.md)
(ports/root in that file remain EVAL-012 history; this campaign binds the
table above). Bind:
[20260820-personal-c1-c2-secret-bind-runbook.md](../checkpoints/20260820-personal-c1-c2-secret-bind-runbook.md).

Do not restore or delete snapshots. Do not contact `B01-Clean-Linux-001`.
Do not reuse closed EVAL ports `48286`–`48298` / `48300` / `48302` /
`48386`–`48398` / `48383` / `48400` / `48402`, SecretStore `/12`–`/19` /
`/24` / `/25`, or P2-T37 roots.

`tools/personal/c1-c2-paired/cells.json` still names reserved historical id
`PERSONAL-PERF-EVAL-012`. That file is not edited mid-campaign.

Frozen seed *generator* is `freeze.mjs` on this pin. Prior EVAL-013 samples
on `6c415625` are a different denominator and are not imported.

## 3. Cell list

Execute parent plan §9 on this freeze: freeze → fairness/secret/denominator
review → **B0** → B1 → freeze B2 N → B2 → B3 → B4 → B5 (1 h then 8 h; 24 h
conditional/default deferred) → cleanup + secret scan → analysis → report.

B0 uses one qualification seed per class C1, C2a, C2b, C2c, C2d; three
warmups per arm; secret scan; tool-equivalence; timeout; cleanup; no claim
samples. `retry=0`. Missing runner or capability is `not-run` /
`not_available`. B5 24 h stays conditional. B6 stays later-only.

B1/B2 use the P9-T12 live `runLivePairedCell` with campaign-injected
`executeArm` (no cobbled B0 shell as the formal paired campaign). Fairness
must pass before a cell is counted. Dry-run cannot be labeled counted.

## 4. Measurement rules

- Measurement-only (Operating Model §2.5): no product/contract/negative/test
  or generated-handbook edits to make a cell runnable.
- Provider cells `retry=0`.
- No Gate, release, Profile, B01, or Agent-benefit promotion.

## 5. Provider budget (recorded at activation, before B0)

Counted C1/C2 envelope (same EVAL-013 style; new roots/ports/item):

| Batch | Counted arm-runs | Note |
|---|---:|---|
| B0 | 10 | 5 classes × 2 arms × 1 qualification; 30 warmups are non-counted |
| B1 | 100 | after B0 pass |
| B2 | 900 | after B1 |
| **Ceiling** | **1010** | stop when exhausted; do not invent extra cells |

A fairness fail blocks C1/C2 B1. Expanded remainder (C0, B3–B5, T/S/O/UJ)
does **not** inherit extra counted Provider budget.

## 6. Cleanup

Stop campaign daemon `48304` and broker `48404` only. Clear only the
campaign-unique SecretStore item with `secret-tool clear` on product
non-secret attributes; confirm with D-Bus `SearchItems` paths. Never
`secret-tool search` / `lookup`. Leave `48181` and closed EVAL roots untouched.

## 7. Non-claims

Activation is not B0 pass, not a paired performance result, and not Gate /
release / Profile / B01 / Agent-benefit evidence. Campaign closure does not
resume development.

## Unique next action

Campaign closed 2026-08-20. Do not reopen this freeze. Wait for an explicit
owner delivery instruction before claiming any implementation task.
