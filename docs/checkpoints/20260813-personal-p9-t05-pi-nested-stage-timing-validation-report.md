# P9-T05 running validation report — Pi route nested stage timing

Single running report for `P9-T05` under Operating Model `TEST-REPORT-INCREMENTAL-01`.
Every finished validation unit is appended here immediately, before the next unit
starts. Entries are append-only; a later entry may supersede an earlier one only by
stating so explicitly.

- Task: `P9-T05` — Pi route nested per-request stage timing and Provider usage exposure
- Branch: `personal/P9-T05-pi-nested-stage-timing`
- Base: `origin/main@326f97728ab6aaaacceaedd2156d953231b32e01`
- Lease: `lease/personal/P9-T05/pi-nested-stage-timing`
- Claim ceiling: measurement capability only. Nothing here is a Gate, release,
  Profile, B01, benchmark or overhead-attribution claim.

## Environment routing

| Environment | Use |
|---|---|
| `DEV-WIN-GNU-01` (local) | Node/TypeScript package tests, `check:consistency`, `check:handbook`, generator `--check`, docs-sync gate, `cargo fmt`. Rust build/test/Clippy is barred by `RUST-LINK-DEV-WIN-GNU-01` |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | Primary Rust and workspace validation at an exact pushed revision |
| `DEV-LINUX-NATIVE-01` | Optional exact-revision focused Rust runs in a disposable clone; shared host, used only when CI is insufficient |

## Units

### U01 — failure-first: one Pi run cannot produce a joined observation

- Unit: `packages/pi-cognitiveos` → `dist/pi-route-observation.test.js`
- Environment: `DEV-WIN-GNU-01`
- Command: `node --test "dist/pi-route-observation.test.js"`
- Result: **fail (expected)** — 0/1 pass.
- Evidence: the run stops at
  `the package exposes no campaign-runner entry point for Pi route observations`
  (`openPiRouteObservationSession` is `undefined`). The assertions behind it —
  seven ordered stages with positive monotonic durations, one opaque
  `campaign-<32 hex>` correlation id observed by both sides, and measured
  Provider usage readable by a campaign runner — are therefore unreached.
- Reading: today a single Pi run publishes no joined, monotonic per-stage
  timing and exposes no runner-readable Provider usage. This is the capability
  gap `P9-T05` must close; it is not a defect claim about any measured latency.
