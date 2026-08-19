# PERSONAL-PERF-EVAL-012 — preregistration scaffolding

- Campaign ID: `PERSONAL-PERF-EVAL-012`
- Status: **reserved / not active**. Evaluation routing remains **OFF**.
- This document is P9-T08 package 14 scaffolding. It must not start samples,
  bind guest ports, or replace the `PROGRESS.md` Owner-directed campaign row.
- Claim ceiling: `hypothesis` / non-claim. Reviewer: `not_reviewed`.
- Parent: [personal-performance-benchmark-execution-plan.md](personal-performance-benchmark-execution-plan.md)
  v1.1. C1/C2 overlay:
  [PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md](../plan/PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md)
  §6 (replaces stale “OS arm unreachable” gates for a new EVAL only).

Closed EVAL-002 and EVAL-004 through EVAL-011 are never resumed.

## 1. Activation gate (not satisfied by this file)

Activation requires **all** of:

1. Packages 6–13 have supported evidence in the P9-T08 running report.
2. Owner sets the Current snapshot `Owner-directed campaign` row to
   `PERSONAL-PERF-EVAL-012`.
3. An evaluation lease `lease/personal/EVAL-012/<purpose>` owns only
   `docs/evaluation/`, `docs/checkpoints/`, and `docs/plan/PROGRESS.md`.

Until then, every B0/B1/B2 cell is **not-run** because the campaign is not
active — not because C1/C2 product paths are missing.

## 2. Isolation (bind at activation)

| Resource | Reserved |
|---|---|
| Guest | `B01-Desktop-Linux-002` only |
| Route | `wuz@192.168.1.2` → ProxyJump `hal9001@192.168.123.160` |
| Root | `/home/hal9001/perfeval012-<activation-date>` |
| Daemon | `127.0.0.1:48300` |
| P-arm broker | `127.0.0.1:48400` |
| SecretStore | planned `/20` (≠ `/12`–`/19`) |
| Git revision | freeze at activation from the then-current pushed HEAD of the readiness branch after merge, recorded here before B0 |

Procedure:
[20260820-personal-c1-c2-b01-guest-procedure.md](../checkpoints/20260820-personal-c1-c2-b01-guest-procedure.md).
Bind:
[20260820-personal-c1-c2-secret-bind-runbook.md](../checkpoints/20260820-personal-c1-c2-secret-bind-runbook.md).

## 3. Cell list (must not forget C2b–d)

B0 (package 15): one qualification seed per class C1, C2a, C2b, C2c, C2d;
three warmups per arm; secret scan; tool-equivalence; timeout; cleanup; no
claim samples.

B1 (package 16): five pilot seeds per class; two runs per arm.

B2 (package 17): 30 held-out paired seeds per class; three runs per arm when
the Provider lacks deterministic replay; `retry=0`; started = retained.

Frozen cell overlay: `tools/personal/c1-c2-paired/cells.json`.
C2b/C2c/C2d are split-score / capability-gap unless tool sets match.

C0, B3–B5, T6–T9, S4/S8 are **out of 完整 C1+C2** unless the owner expands
this EVAL after activation.

## 4. Measurement rules

- Measurement-only (Operating Model §2.5): no product/contract/negative/test
  or generated-handbook edits to make a cell runnable.
- Provider cells `retry=0`.
- Missing capability is `not-run` / `not_available`.
- No Gate, release, Profile, B01, or Agent-benefit promotion.

## 5. Provider budget

Record a numeric ceiling in this file **at activation**, before B0. Scaffolding
does not invent a live budget. Default planning envelope: reuse the closed
EVAL-004 DeepSeek ceiling style (counted B0+B1+B2 C1/C2 cells only) and stop
when the owner-stated remainder is exhausted.

## 6. Cleanup

Stop campaign daemon `48300` and broker `48400` only. Clear only the
campaign-unique SecretStore item with `secret-tool clear` on product
non-secret attributes; confirm with D-Bus `SearchItems` paths. Never
`secret-tool search` / `lookup`. Leave `48181` and closed EVAL roots untouched.

## 7. Non-claims

This scaffolding is not an active campaign, not B0, and not a performance
result.
