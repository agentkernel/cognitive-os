# PERSONAL-PERF-EVAL-015 — preregistration

- Campaign ID: `PERSONAL-PERF-EVAL-015`
- Status: **active** (2026-08-21). Evaluation routing **ON**.
- Lease: `lease/personal/EVAL-015/remaining-plan-cells`
- Branch: `evaluation/EVAL-015-freeze`
- Claim ceiling: `hypothesis` / non-claim. Reviewer: `not_reviewed`.
- Parent: [personal-performance-benchmark-execution-plan.md](personal-performance-benchmark-execution-plan.md)
  v1.1. C1/C2 overlay:
  [PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md](../plan/PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md).
- Freeze checkpoint:
  [20260821-personal-perf-eval-015-preregistration.md](../checkpoints/20260821-personal-perf-eval-015-preregistration.md)
- Running report:
  [personal-performance-assessment-20260821-eval-015.md](personal-performance-assessment-20260821-eval-015.md)

This is a **new freeze**. Closed EVAL-002 and EVAL-004 through EVAL-014 are
never resumed.

Owner 2026-08-21 directed continuous autonomous completion of the parent
execution plan after EVAL-014 closed. EVAL-014 counted C1/C2a B0/B1/B2 on
the same product pin and is **carried as closed prior evidence**. This freeze
covers remaining required cells (C0, B3–B5, T/S/O/UJ, C2b resume/Skill,
C2c fault profile). Do not re-run counted C1/C2a B1/B2.

## 1. Activation gate

Activation requires **all** of:

1. Owner delivery instruction after EVAL-014 close (2026-08-21). **pass.**
2. Current snapshot `Owner-directed campaign` row names
   `PERSONAL-PERF-EVAL-015`. **pass** (this activation).
3. Evaluation lease `lease/personal/EVAL-015/<purpose>` owns only
   `docs/evaluation/`, `docs/checkpoints/`, and `docs/plan/PROGRESS.md`.
   **pass** (`lease/personal/EVAL-015/remaining-plan-cells`).

## 2. Isolation (bind at activation)

| Resource | Bound value |
|---|---|
| Guest | `B01-Desktop-Linux-002` only |
| Route | `wuz@192.168.1.2` → ProxyJump `hal9001@192.168.123.160` |
| Root | `/home/hal9001/perfeval015-20260821` mode `0700` |
| Daemon | `127.0.0.1:48306` |
| P-arm broker | `127.0.0.1:48406` |
| SecretStore | new item `/27` (≠ `/12`–`/19` / `/24` / `/25` / `/26`) |
| Git revision | `adc404990974f6fbda91d57a5556d78a145ef596` (same pin as closed EVAL-014; `origin/main` after P9-T12). Product merge: PR [#252](https://github.com/agentkernel/cognitive-os/pull/252) at `39cf8019`. |

Do not restore or delete snapshots. Do not contact `B01-Clean-Linux-001`.
Do not reuse closed EVAL ports `48286`–`48298` / `48300` / `48302` /
`48304` / `48386`–`48398` / `48383` / `48400` / `48402` / `48404`,
SecretStore `/12`–`/19` / `/24` / `/25` / `/26`, or closed EVAL roots as
runtime binds. Residue listeners `48181`/`48284`/`48383` stay untouched.

`tools/personal/c1-c2-paired/cells.json` still names reserved historical id
`PERSONAL-PERF-EVAL-012`. That file is not edited mid-campaign.

Source archive and exact-source binaries are reused by digest from the
EVAL-014 host freeze of the **same pin** (`sha256:0d4552c6…`); they are
copied into the new root, not executed from `perfeval014`.

## 3. Cell list

Execute parent plan §9 remainder on this freeze:

1. Freeze source/environment/corpus/arms/oracles/secret broker (new isolation).
2. Campaign fairness/secret/denominator review (independent reviewer remains
   `not_reviewed`).
3. `B0` qualification for **C0** (C1/C2a B0 carried from EVAL-014; not re-run).
4. `B1` C0 pilot (9 families × 5 seeds × 1 replica = 90 pairs).
5. Freeze B2 N (formal N = 30 per C0 family unless power analysis raises it).
6. `B2` C0 confirmatory (9 × 30 × 1 replica = 270 pairs). Replica count
   matches the executed EVAL-004 composition; the plan’s 3-replica ceiling is
   recorded and not used.
7. `MS-AUTH`, `T-GOV`, `UJ2` cold, `UJ3`, `UJ4`.
8. `B3` faults (N=10 per executable class). Stale/`OUTCOME_UNKNOWN` only if
   a frozen campaign-authorized default-off injector is bound.
9. `B4` local concurrency. Mixed Agent if Provider budget remains.
10. `B5` 1 h then 8 h. 24 h default deferred unless 8 h shows an unresolved
    slope **and** owner budget allows.
11. C2b session-2 resume + Skill bind (campaign-local; do not restart the
    campaign daemon for resume). C2c campaign-authorized default-off fault
    profile if the public management surface accepts `PERSONAL-PERF-EVAL-015`.
12. Cleanup + secret scan + analysis + report.

Missing runner or capability is `not-run`/`not_available`. `retry=0`.
Retain every started sample. B6 is later-only.

Campaign-local C0/B3/B4/B5/UJ/MS-AUTH adapters are copied from closed
EVAL-004 instruments into this root and re-pathed; they are not product
edits. The P-arm broker is the EVAL-004 stdin broker (no
`secret-tool lookup`/`search`), fed from D-Bus `GetSecret` through a pipe.
FAULT_MODE `timeout`/`oversize` spend no Provider calls.

## 4. Measurement rules

- Measurement-only (Operating Model §2.5): no product/contract/negative/test
  or generated-handbook edits to make a cell runnable.
- Provider cells `retry=0`.
- No Gate, release, Profile, B01, or Agent-benefit promotion.
- EVAL-014 C1/C2a counted cells are prior evidence on this pin, not this
  campaign’s denominator.

## 5. Provider budget (recorded at activation, before B0)

| Batch | Counted arm-runs | Note |
|---|---:|---|
| B0 C0 | 18 | 9 families × 2 arms; 6 warmups non-counted |
| B1 C0 | 180 | 90 pairs |
| B2 C0 | 540 | 270 pairs, 1 replica |
| UJ2 cold | 20 | 10 pairs |
| B5 paired soak | ≤120 | 1 h every 5 min + 8 h every 10 min |
| **Ceiling** | **≈878** | stop when exhausted; do not invent extra cells |

B3 fault cells that deny before dispatch do not consume the ceiling.
C1/C2a B1/B2 are **not** in this envelope.

## 6. Cleanup

Stop campaign daemon `48306` and broker `48406` only. Clear only the
campaign-unique SecretStore item with `secret-tool clear` on product
non-secret attributes; confirm with D-Bus `SearchItems` paths. Never
`secret-tool search` / `lookup`. Leave `48181`/`48284`/`48383` and closed
EVAL roots untouched.

## 7. Non-claims

Activation is not B0 pass, not a paired performance result, and not Gate /
release / Profile / B01 / Agent-benefit evidence. Carrying EVAL-014 C1/C2a
does not promote those results.

## Unique next action

Finish the live B5 8 h continuation (pid **408358**; last completed
minute 324; 33/33 pairs; kernel 411171). Do not start a second
continuation. Record B5 24 h default deferred unless the 8 h slope
trigger is met. Then cleanup (`48306`/`48406` and SecretStore `/27`
only), secret scan, final assessment, and close the campaign row and
lease.
