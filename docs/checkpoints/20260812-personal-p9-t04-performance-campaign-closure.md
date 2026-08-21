# P9-T04 comprehensive performance campaign — final report and closure

- Status: campaign complete; closed as a **non-claim report**
- Task: `P9-T04`
- Campaign ID: `P9-T04-comprehensive-performance-001`
- Registration: [ADR-0051](../adr/0051-comprehensive-performance-campaign.md) and
  [preregistration](./20260812-personal-p9-t04-performance-campaign-preregistration.md)
- Claim level: `hypothesis`
- Independent verifier disposition: `not_reviewed`
- Agent-benefit claimed: **no**

## 1. What this report does and does not say

It reports measured durations, counts, outcome classes, and safety accounting
for a governed CognitiveOS Personal path, on named environments, at exact pushed
revisions. Every started sample is retained.

It does **not** establish a product Gate, a release, a Profile, a B01 outcome, a
governance non-inferiority result, or any generalized Agent benefit. The owner
dropped `L5` on 2026-08-12, so no `A` arm exists; without one, the measured
governance overhead is a single-arm observation and the execution plan's
`B`-versus-`A` thresholds are neither met nor failed — they were not evaluated.

## 2. Layer disposition

| Layer | Environment | Disposition | Denominator |
|---|---|---|---|
| `L0` eligibility | `DEV-LINUX-NATIVE-01` | completed | 1 / 1 |
| `L1` module benchmark | `DEV-LINUX-NATIVE-01` | completed | 200 / 200, 3 warmups excluded |
| `L2` governed path and store access | `DEV-LINUX-NATIVE-01` | completed | 52 / 52 |
| `L3` Provider route | `B01-Desktop-Linux-002` | completed | 160 / 160 across six cells |
| `L4` governed Task scenarios | `B01-Desktop-Linux-002` | partial | `T1` admission 10 / 10; `T2`-`T8` not-run |
| `L5` A/B/C/D benefit | — | not-run | owner disposition, 2026-08-12 |

## 3. Measured results

### 3.1 `L1` deterministic modules, 200 samples each

| Module | p50 | p95 |
|---|---:|---:|
| `context-cache-full-key-hit` | 0.31 µs | 0.33 µs |
| `context-resolution-filter-builder` | 37.4 µs | 44.4 µs |
| `artifact-cas-immutable-publish-readback` | 109.8 µs | 144.5 µs |
| `canonical-performance-report-serialization` | 113.1 µs | 127.5 µs |
| `memory-fts5-metadata-first-retrieval` | 321.2 µs | 364.7 µs |
| `intent-effect-durable-persist-before-dispatch` | 4.63 ms | 5.07 ms |
| `scheduler-eligible-lease-cas` | 6.33 ms | 8.28 ms |

### 3.2 `L2` governed path and store access

Governed `effect_persistence` dominated at roughly 265 ms cold and 242 ms warm,
against authorization at 147 µs cold / 7.9 µs warm, Context resolution at
158 µs / 110 µs, and cache reuse under 3 µs. Per-open store reads cost about
121 ms against about 0.7 ms for the long-lived handle over 50 iterations.

Both confirm prior decisions with data rather than intuition: `P9-T01` was right
that `effect_persistence` aggregates SQLite open, admission, persist and reload
and must never be read as transport cost, and `P9-T03`'s long-lived store reuse
is worth roughly two orders of magnitude on repeated reads.

### 3.3 `L3` Provider route, real DeepSeek, 160 retained samples

| Cell | Samples | Outcome | Key measurement |
|---|---:|---|---|
| `R1` proxy marker | 30 | 30 complete | Provider network 898.9 ms p50 / 1224.6 ms p95 |
| `R2` Pi first response | 30 | 30 complete | first response 4625 ms p50 / 5004 ms p95 |
| `R3` cold daemon journey | 20 | 20 ready, 20 complete | startup-to-ready 182.6 ms p50; journey 2069.2 ms p50 |
| `R4` warm repeated | 50 | 50 complete | Provider network 1016.1 ms p50; 0.85 req/s serial |
| `R5` model mismatch | 20 | 20 denied before dispatch | 35.1 ms p50, **zero Provider dispatches** |
| `R6` bounded timeout | 10 | 10 retained | bound held at 122.9 ms p50 against a 120 ms deadline |

Provider usage was `measured` on every completed sample. No time to first token
is reported anywhere: the proxy is non-streaming. No cost is reported anywhere:
there is no preregistered pricing snapshot. The rate-limit class is not-run
because inducing HTTP 429 would mean deliberately hammering a third party.

### 3.4 `L4` `T1` read-only admission, 10 retained runs

8 of 10 runs admitted; 2 returned the registered `TASK_ADMISSION_REJECTED` and
are retained in the denominator rather than replaced. Zero mutations occurred,
which is what `T1`'s read-only oracle requires. Independent acceptance did not
occur in any run, and the report therefore claims **zero verified completions**.

| Stage | p50 | p95 |
|---|---:|---:|
| session mint | 3.67 ms | 81.18 ms |
| `intent.record` | 13.90 ms | 17.73 ms |
| `intent.interpret` | 16.57 ms | 20.19 ms |
| `task.preview` | 8.65 ms | 13.53 ms |
| `task.admit` | 25.74 ms | 28.90 ms |
| admission total | 68.17 ms | 140.77 ms |

`T1` covers the admission half of its preregistered path. The execution half —
Context resolution, Pi candidate, read/search Tool, Artifact, independent
verifier — was not driven, so `T1` is reported as partial rather than complete.
`T2`-`T8` are not-run.

## 4. The finding that matters most

Governance overhead is small, stable, and not where the time goes.

Across `R1` and `R4`, local governance plus loopback overhead measured 126.5 ms
and 128.5 ms p50 — remarkably stable — while DeepSeek's own latency moved from
898.9 ms to 1016.1 ms p50 between those same two cells. Measuring them apart is
what makes this visible; combined, Provider variance would have read as
governance regression.

The dominant cost in the user-visible path is neither of those. The same
Provider call costs about 1.0 s through the daemon client and about 4.6 s
through Pi. Pi process spawn and agent initialisation account for roughly 3.5 s.
If first-response latency is a product concern, the Pi launch path holds the
budget, and this campaign says so with data instead of assuming it.

## 5. Safety accounting

| Counter | Result |
|---|---:|
| unauthorized or stale Context exposure | 0 |
| Provider secret exposure | 0 |
| duplicate external Effect | 0 |
| false completion | 0 |
| stale-epoch commit | 0 |
| unreconciled Effect | 0 |
| completion without independent acceptance | 0 |
| scenario boundary violation | 0 |

The Provider credential was entered once by the operator through the graphical
hidden-input path and never read, echoed, copied, passed in argv, or written to
the runtime tree by any campaign component. A key-shaped scan of the entire
campaign root found nothing.

## 6. Defects this campaign found

1. **`p1-t09-product-route-smoke.sh` omitted `XDG_STATE_HOME`.** Its minimal
   `env -i` allowlist forwarded four XDG roots but not the one the daemon
   publishes `daemon-endpoint.json` into, so the Extension could not find the
   daemon under any non-default runtime root. It passed in P1-T09 only because
   that campaign used the default state directory. Fixed.
2. **The B01 guest has no non-interactive access path.** SSH was refused and no
   QEMU guest agent was connected, so a B01 attempt is operator-driven end to
   end. The owner authorized a campaign-scoped SSH path using the pre-existing
   `b01-desktop-guest-002` key; this is recorded as a deliberate baseline change.

Three measurement-side corrections are recorded in the preregistration rather
than quietly patched: classifying route failures by fuzzy message text, reading
the client's transport wrapper code instead of the daemon code it already
preserves, and an initial Task draft that did not match the generated `Budget`
binding. In each case the product was correct and the measurement was not.

## 7. Acceptance mapping

| Acceptance element | Result |
|---|---|
| measurement-only correlation/timing/usage/transport/resource/evidence runner | complete — D01–D09 |
| complete `L1`–`L5` execution in one preregistered campaign | **partial** — `L1`–`L3` complete, `L4` partial, `L5` dropped by owner |
| full denominator retained | complete — every started sample retained, including 2 admission rejections and 10 timeouts |
| independent verifier | `not_reviewed`; the claim ceiling stays `hypothesis` accordingly |
| report and cleanup | complete — this report; cleanup recorded per run |
| no second authority writer | complete — every runner is an observer |
| no fabricated TTFT or Provider usage | complete — TTFT absent, usage `measured` only when the Provider supplied complete counters |
| failed or incomplete `L5` still yields a non-claim report and closure | complete — this document |

`L4` remaining scenarios and `L5` are closed as `not-run` with reasons, not
approximated. Under the acceptance clause that an incomplete `L5` must still
produce a complete non-claim report and closure, the task closes here.

## 8. Non-claims

This campaign does not pass or contribute to B01, B02, B03, B04, B05, B06, B07,
B08, B09, B10, B11, B12, or GMVP-LINUX. It does not authorize a release, a
Profile, a Windows claim, or a generalized Agent-benefit statement. Local,
fixture, and ordinary CI evidence is not promoted. `not-run` remains `not-run`.
