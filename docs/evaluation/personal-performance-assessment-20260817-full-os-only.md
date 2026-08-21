# CognitiveOS Personal performance assessment — 2026-08-17 (EVAL-004 full-OS re-freeze)

- Campaign: `PERSONAL-PERF-EVAL-004` (owner-directed, Operating Model §2.5)
- Execution contract: [personal-performance-benchmark-execution-plan.md](./personal-performance-benchmark-execution-plan.md) v1.1
- Scope amendment: [personal-performance-benchmark-full-os-only-addendum.md](./personal-performance-benchmark-full-os-only-addendum.md)
- C1/C2 addendum: [personal-c1-c2-benchmark-execution-plan.md](./personal-c1-c2-benchmark-execution-plan.md)
- Running record: [20260816-personal-perf-eval-004-refreeze-preregistration.md](../checkpoints/20260816-personal-perf-eval-004-refreeze-preregistration.md)
- Frozen product: `origin/main@1e71344a7b2c4a443fd0581e7fd33f21e970efbd` (P2-T21..P2-T28 / BR-01..BR-08)
- Target: `B01-Desktop-Linux-002` (`hal9001@192.168.123.160`), campaign root
  `/home/hal9001/perfeval004-20260816`, daemon `127.0.0.1:48286`, broker
  `127.0.0.1:48386`
- Claim level: **`hypothesis` / non-claim**
- Independent verifier: **`not_reviewed`**
- Agent benefit claimed: **no**
- Document status: remaining required cells recorded 2026-08-17; campaign **closed 2026-08-17** after owner 「继续完成」

This report does **not** establish a Gate, release, Profile, B01, or
Agent-benefit outcome. `PERSONAL-PERF-EVAL-002` remains closed and is not
rewritten here.

## 1. Headline findings (hypothesis only)

**C0 paired tasks still show ~1.7 s OS overhead and no measurable completion
gain.** At the BR-01..BR-08 revision, B2 held-out 270 pairs / 540 runs (all
retained): pure Pi completed 247/270 = 91.5 % vs OS 241/270 = 89.3 %
(delta **−2.2 pp**, 95 % clustered-bootstrap CI **[−5.19, +0.74] pp**,
McNemar p = 0.2379). Median paired wall **+1695.6 ms** (CI [1595.7, 1833.4],
**+43.9 %**). B1 pilot (90 pairs) was the same direction for wall
(+1790 ms, +43.9 %) with completion +3.3 pp and a CI that includes zero.

**C1/C2 governed workspace arms remain unpaired** after a 2026-08-17
public-path re-check. P2-T21..P2-T28 provide the production daemon
Workspace* path (T-GOV 7/7 `execution_ready`). The frozen Extension still
has `READ_ONLY_TOOL_ALLOWLIST = []` and does not advertise Workspace* tools
to Pi. Paired C1/C2 stayed `not-run` rather than substituting a second
authority writer. Measurement-only; no mid-campaign product task.

**Local OS surfaces are fast; CLI status is not.** UJ3: in-daemon health
0.45 ms p50 (200/200); six-resource GET 0.29–0.41 ms; CLI `status`/`doctor`
~1.75 s p50 because this revision actually resolves SecretStore and probes
Pi. B4 local: 932/932, 0 non-OK, ~1.6–3.0 k rps. B5 8 h: 12 960/12 960, 0
non-OK, 7/7 hourly restarts with health 200 and 0 lock residue, RSS
10 280 → 10 260 kB (−20 kB / 8 h). B5 24 h `not-run` (no unresolved slope).

**Admission is not completion.** UJ4: 30/30 Tasks admitted, 0 independent
acceptances, 0 verified completions claimed, 0 mutations.

**Tool lifecycle is now a public management surface.** T2 65/65: disable /
enable / quarantine / channel isolation / unknown-id behave as registered
codes. EVAL-002's enable/disable fall-through is historical after P2-T25.

**UJ2 cold bind works.** The started pair `confirmatory-A4-000` remains a
retained 180 s timeout on both arms (0 output chars). Remaining seeds 1–9
were completed with `--bind`: **9/9 pairs** both arms `completed` and
oracle True; O-arm cold start health 200 × 9. Descriptive paired wall on
those 9 completed pairs: median **+2270.9 ms** (927–3111). No
cold-versus-warm comparison is made. Combined UJ2 cold denominator:
**10/10 pairs started and retained**.

## 2. Cell register

| Cell | Disposition | Denominator (started/retained) |
|---|---|---|
| B0 C0 qualification | pass | 9/9 (+ 3 discarded warmups) |
| B1 C0 pilot | pass (hypothesis) | 90 pairs / 180 runs |
| B2 C0 confirmatory | pass (hypothesis) | 270 pairs / 540 runs |
| C1/C2 paired B0/B1/B2 | `not-run` | equivalent Pi adapter absent after public-path re-check |
| UJ1 | `not-run` | no guest reinstall |
| UJ2 cold | **pass** (hypothesis; 1 retained timeout pair) | 10/10 pairs; seed 0 timeout both arms; seeds 1–9 complete/oracle True |
| UJ2 Pi-warm | `not_available` | no process reuse |
| UJ3 daily ops | pass | health 200; CLI 100/50/50; 300 GET; 60 watch |
| UJ3 task-channel watch | **partial** | 20/20 HTTP 403 (expected channel isolation, not a defect) |
| UJ4 / O1 admission | pass | 30/30 admitted; 0 completions |
| UJ6 | register only | table in the running record |
| T-GOV projection | pass | 7/7 `execution_ready` |
| T2 lifecycle | pass | 65/65 |
| T3; T4–T9 invoke; T10 live | `not-run` | optional or missing public dispatch / ecosystem |
| MS-AUTH | **partial** | 6/6 negatives; Skill lifecycle 10/10; Memory sealed-header positives `not-run` |
| S4/S8 | `not-run` | no paired governed consumer |
| B3 faults | **partial** | mismatch/restart/Pi-kill 10/10; deadline/broker/upstream-timeout/oversize 10/10; stale `not-run` |
| B4 local | pass | 932/932; mixed Agent `not-run` |
| B5 1 h | pass | 1620/1620 |
| B5 8 h | pass | 12960/12960; 7/7 restarts |
| B5 24 h | `not-run` | slope resolved; no owner budget |
| B5 paired soak / B4 mixed Agent | `not-run` | Provider confirmatory already consumed |
| O2/O3/O4/O14 | `not_available` | no compliant public surface |
| O5/O6 | `not-run` | C1/C2 adapter absent |
| O10–O13 | covered by B3/UJ3/B0 | no extra denominator |
| B6 | `not-run` | not this campaign |
| Cleanup | pass | 2026-08-17 remaining-cells cleanup: daemon/broker stopped; SecretStore `/13` cleared (no search/lookup); 48181/48284/48383 untouched; redactor 0 hits |

## 3. Capability truth (not a slow-path claim)

Reachable on the public campaign daemon at this freeze: C0 Provider proxy,
management health/status/doctor, six-resource projection/watch,
Task admission, Tool catalog and lifecycle mutation, Skill
import/inspect/bind/supersede/revoke, fail-closed Memory/Skill negatives,
local concurrency and soak.

Not paired / not claimed as OS Agent benefit: C1/C2 workspace tools,
governed Skill consumption, verified Task completion, backup/restore,
fairness telemetry, cache observation, live MCP.

## 4. Evidence-ranked follow-ups (not a delivery backlog)

1. Nested per-request Pi/Extension timing to locate the ~1.7 s C0 overhead
   (still unattributed beyond “not daemon local residual / not broker”).
2. Equivalent Pi Workspace* adapter (product: Extension advertise daemon
   Workspace* tools) before any C1/C2 paired cell. That is a product mutex,
   not a measurement gap that this campaign may implement.
3. Sealed-header composer from daemon-owned `GovernanceSeed` so MS-AUTH
   Memory positives can run without fabricating governance.
4. Do not use `secret-tool search` or `lookup` in cleanup; they print secret
   material. Rotate the Provider key that appeared in the earlier operator
   SSH transcript. The 2026-08-17 remaining-cells session used stdin import
   and `secret-tool clear` plus D-Bus attribute listing only.
5. Independent review before any claim promotion.

## 5. Non-claims

No Gate, release, Profile, B01, B01-W, or Agent-benefit promotion. No
optimization success. Campaign closure does **not** resume development.
