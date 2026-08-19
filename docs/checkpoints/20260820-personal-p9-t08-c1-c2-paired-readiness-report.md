# P9-T08 C1/C2 paired-benchmark readiness — running report

- Task: `P9-T08`
- Lease: `lease/personal/P9-T08/c1-c2-paired-readiness`
- Branch: `personal/P9-T08-c1-c2-paired-readiness`
- Claim ceiling: `hypothesis` / non-claim
- Navigation: [PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md](../plan/PERSONAL-C1-C2-READINESS-DELIVERY-PLAN.md)

Per `TEST-REPORT-INCREMENTAL-01`, append each finished validation unit here
before starting the next.

## D01 programme amendment

1. **Owner instruction recovered against canonical sources — pass.**
   `PROGRESS.md` had closed packages 6–9 as assessment-only. Evaluation
   routing was OFF. Closed EVAL-002 and EVAL-004 through EVAL-011 are not
   resumed. Active lease table was empty.
2. **Programme rewritten — pass.** Definition of done is “ready to start B0
   on `B01-Desktop-Linux-002` for paired C1+C2”, not “a B0 may be requested”.
   Packages 6–14 are readiness delivery; 15–17 are the measurement campaign.
3. **Formal task registered — pass.** `P9-T08` is `in-progress` with slices
   D01–D04. Layer 1 counts 98 / 90 / 1 / 1 / 6 / 8.
4. **Isolation reserved, not activated — pass.** `PERSONAL-PERF-EVAL-012` is
   named only as a reserved future EVAL ID. The Owner-directed campaign row
   does not activate it.
5. **D01 durable — pass.** Commit `6ceb8d29` pushed; Draft PR
   [#247](https://github.com/agentkernel/cognitive-os/pull/247).

## D02 P-arm instruments

6. **Failure-first P-arm broker and fixture adapter — pass (local Node).**
   `node --test tools/test/c1_c2_paired_p_arm.test.mjs` **8/8 pass**. Broker
   refuses non-loopback binds and secret-shaped argv/env, binds only a
   non-secret placeholder token, and exposes no Context/Memory/Task/retry/
   verify surface. Fixture adapter schemas match O-arm Workspace* names and
   parameter keys; C1 read/search and C2a write/patch run inside a temp
   fixture root; path escape and preimage mismatch fail closed.
7. **Linux Secret Service get and live Pi — not-run.** Remaining D02 work on
   `DEV-LINUX-NATIVE-01`. No B01 sample.

Next unit: Linux Secret Service `get` into the broker without argv/env/log
material, still on a non-B01 host.
