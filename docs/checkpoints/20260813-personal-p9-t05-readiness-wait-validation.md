# P9-T05 daemon readiness wait — running validation report

- Activity: owner-directed corrective delivery for a confirmed required-CI
  reliability defect in the Personal daemon integration tests
- Change class: `implementation-only`, test-harness surface only; no product
  code, contract, negative, assertion, or generated documentation source is
  touched
- Task / lease: P9-T05 / `lease/personal/P9-T05/daemon-readiness-wait`
- Branch: `personal/P9-T05-ci-readiness-wait`
- Base: `origin/main@326f97728ab6aaaacceaedd2156d953231b32e01`, deliberately
  independent of the in-flight P2-T16/P2-T17 branches so the fix can merge as
  soon as it is green
- Environment routing: local `DEV-WIN-GNU-01` runs formatting, static, Node and
  docs gates only (`RUST-LINK-DEV-WIN-GNU-01`). Rust build/test/Clippy routes to
  required GitHub Ubuntu/Windows CI; the Windows job is the one that carries the
  evidence for this defect. Exact native Linux is not required here and is
  recorded `not-run` unless it is actually executed.
- Evidence ceiling: test-harness reliability only. No Gate, release, Profile,
  B01, benchmark, or Agent-benefit claim.

This is the activity's single append-only validation report under
`TEST-REPORT-INCREMENTAL-01`. Each finished unit is appended before the next
one starts. Later entries may supersede an earlier result but never erase it.

## Defect

`apps/kernel-server/tests/p1_t05_personal_readiness.rs` spawns a real
`kernel-server` child and then allows it a fixed 100 probes × 20 ms = **2 s** to
publish `local-bootstrap.secret` before panicking. A GitHub-hosted Windows
runner took roughly 2.2 s, so the required check went red while all 151 unit
tests in the same run passed; re-running the identical revision passed. The
daemon was healthy and the product was correct — the wait was too thin.

Independently confirmed as pre-existing latent fragility, not introduced or
amplified by P2-T11. Corroborating repository facts:

- the merged `lease/personal/P2-T10/tool-executor-parity` record already
  classifies its first Windows attempt as "a confirmed daemon-startup-timing
  flake";
- `p1_t04_personal_daemon.rs` carries the comment "Windows CI can delay
  visibility of a just-created private file" above a privately raised 300-probe
  budget, and `p1_t07_provider_proxy.rs` carries "Windows hosted runners can
  delay a newly spawned process's first file write long enough to exceed the
  former two-second polling budget" above a 500-probe budget. Both are
  single-file reactions to the same defect; the remaining files were never
  raised.

### Inventory of the copied wait at the base revision

| File | Connect wait | Bootstrap secret wait | Other readiness wait |
|---|---|---|---|
| `p1_t05_personal_readiness.rs` | 100 × 20 ms = 2 s | 100 × 20 ms = **2 s** | — |
| `p1_t07_pi_readiness.rs` | 100 × 20 ms = 2 s | 100 × 20 ms = **2 s** | — |
| `p2_t02_task_api_watch.rs` | unbounded `loop` | 100 × 20 ms = **2 s** | — |
| `p2_t02_resource_projection.rs` | unbounded `loop` | 100 × 20 ms = **2 s** | — |
| `p4_t05_resource_api.rs` | unbounded `loop` | 100 × 20 ms = **2 s** | — |
| `p1_t04_personal_daemon.rs` | 100 × 20 ms = 2 s | 300 × 20 ms = 6 s | endpoint 100 × 20 ms = 2 s |
| `p1_t07_provider_proxy.rs` | 100 × 20 ms = 2 s | 500 × 20 ms = 10 s | — |
| `m5_http_sse.rs` | unbounded `loop` and 50 × 20 ms = 1 s | — (no personal runtime root) | — |

Two failure modes are present at once: budgets too thin to survive a slow but
healthy start, and unbounded `loop`s that cannot fail at all — a genuinely stuck
daemon there consumes the CI job timeout instead of failing with a reason.

Out of scope, recorded honestly rather than silently changed:
`apps/admin-cli/tests/p2_t02_cli_parity.rs` and
`apps/admin-cli/tests/p1_t06_cognitive_cli.rs` wait 250 × 20 ms = 5 s in a
different crate, and `p1_t06` already aborts early when the child exits. Neither
carries the identical 2 s bound and neither has been observed failing.

## Validation log

Appended in completion order.

### 1. `cargo fmt --all -- --check` — **pass** (D01, local `DEV-WIN-GNU-01`)

First run failed on one wrapped line in the new test file; `cargo fmt --all`
applied it and the re-check is clean. Formatting does not compile or link, so it
is inside the local allowlist.

### 2. `node tools/src/check-consistency.mjs` — **pass** (D01, local)

First run reported 7 violations, all in my own registration and all repaired
before this entry: the Phase 9 and total summary counts in the formal plan had
to move to 5/4/1/0/0 and 73/64/1/1/7; `not-started` is not a delivery-slice
status, so D02/D03 are registered `ready`; and the lease row needed the status
cell to be exactly `active`, a `YYYY-MM-DD / YYYY-MM-DD` claimed/heartbeat pair,
and no claim over `docs/plan/PARALLEL-LANES.md` itself. Re-run is clean: 275
requirements, 55 error codes, 74 schemas, 89 vectors, links, traceability,
Personal plan/Gates, routing, delivery and leases verified.

### 3. `node tools/src/docs-sync-gate.mjs --staged` — **pass** (D01, local)

"no documentation-relevant changes (6 path(s) checked)". The gate passed on the
change set itself: `apps/kernel-server/tests/**` is not routed by
`handbook/_meta/source-map.json`, and `docs/plan` / `docs/checkpoints` are not
handbook sources. No `DOCS_IMPACT_NONE` escape is claimed, because none is
needed — the escape is only for changes that do hit mapped sources.

### 4. Rust build / test / Clippy for D01 — **not-run** (local)

`RUST-LINK-DEV-WIN-GNU-01`: this host is a registered unsupported
`x86_64-pc-windows-gnu` link host. The D01 proof is executed on required
Ubuntu/Windows CI, where the failing case is the evidence.

### 5. Required CI run `31717595506` at `54a0668` — **failed as designed** (D01)

This is the failure-first proof. Both required jobs failed, on the intended
case and only on it. Draft PR
[#213](https://github.com/agentkernel/cognitive-os/pull/213).

| Job | Result | Detail |
|---|---|---|
| `verify (ubuntu-latest)` | **fail (expected)** | `readiness_wait_tolerates_a_start_slower_than_two_seconds` panicked at `common/mod.rs`: "bootstrap secret at `/tmp/cos-p9t05-slow-start-.../local-bootstrap.secret` did not become ready within 2000 ms (101 probes over 2000 ms)" |
| `verify (windows-latest)` | **fail (expected)** | same case, same wait: "did not become ready within 2000 ms (99 probes over 2000 ms)" |

The `p9_t05_daemon_readiness_wait` target reports `FAILED. 1 passed; 1 failed`
on both platforms: the fail-closed case already passes, so the wait was not
broken into failing — the 2 s budget genuinely cannot survive a healthy start
that takes 2.5 s. Every other test target in the run passed on both platforms,
so extracting the wait changed nothing else.

The observed Windows failure was at 2.33 s of wall time for a 2.5 s publication,
which reproduces the reported production symptom (a ~2.2 s start against a 2 s
budget) deterministically rather than by chance.
