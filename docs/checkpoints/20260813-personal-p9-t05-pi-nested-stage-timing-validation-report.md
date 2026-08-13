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

### U02 — `packages/pi-cognitiveos` full Node suite with the observation core wired

- Unit: `packages/pi-cognitiveos` → `dist/*.test.js`
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run build ; node --test "dist/*.test.js"`
- Result: **pass** — 81/81.
- Evidence: U01's failure-first test now passes end to end over a real
  loopback socket: one run publishes exactly one observation whose seven
  stages appear in route order with positive monotonic durations, whose
  `campaign-<32 hex>` correlation id is the same value the daemon received,
  and whose Provider usage reads `measured` 7/3/10. The pre-existing
  source-level safety suite passes unchanged — the observation module is
  registered under the same guards and required two design corrections to
  satisfy them: the package writes nothing to the filesystem (a durable sink
  is an injected port owned by the embedding harness), and `process.env` is
  still read in exactly one place, so the authorization decision moved behind
  `PersonalDaemonClient.openCampaignObservationSession`.
- Non-claim: the suite proves the measurement capability exists and refuses
  bad input. It measures no real Provider traffic and attributes no share of
  the campaign's +1828.5 ms overhead to any stage.

### U03 — required negatives, Pi side

- Unit: `packages/pi-cognitiveos` → `dist/pi-route-observation-negatives.test.js`
- Environment: `DEV-WIN-GNU-01`
- Command: `node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **pass** — 20/20, covering all eight required families.

| Required negative | Covered by |
|---|---|
| malformed or duplicate correlation id | ten malformed forms refused; a second publish of one id refused; a mismatched daemon echo degrades the daemon domain to `correlation_mismatch` instead of joining |
| missing or overlapping stages | a second stage cannot open while one is open; each of the five Pi stages refused when omitted; nested daemon total above the loopback wait dropped as `exceeds_loopback_wait` and refused by the validator; duplicate, unknown, misattributed and out-of-order stages refused; a half-reported daemon group refused as `incomplete_stage_group` |
| zero or negative duration | `0`, `-1`, `-0.5`, `1.5`, `NaN` and above-safe-integer refused; non-integer, empty, padded, hex and out-of-range daemon headers not read; a backwards clock refused; a sub-resolution stage floored at 1 ns, never zero |
| internally inconsistent usage counters | mismatched, negative and fractional counters refused; a real run with `total_tokens` but no `completion_tokens` stays `not_available` |
| secret-shaped observation | five credential-shaped campaign ids denied authorization; a refusal message and stack never echo the refused value |
| raw body/header capture | a published record contains none of the prompt, response, bootstrap secret, bearer, `authorization`, session token, endpoint or route strings; its key set and each stage's key set are exactly the schema's; an oversized record is refused |
| instrumentation enabled without authorization | seven default and partial environments denied with a distinct reason each; an unauthorized run emits the same five Pi events, the same text and the same three daemon requests as before; a closed session and a foreign campaign id are refused |
| instrumentation writing authority state | a sink inside any of the three Personal roots refused, including nested paths; relative and non-NDJSON targets refused; a named sink with no injected writer performs zero writes; an instrumented run issues no request beyond the three the operator asked for |

### U04 — daemon nested stage headers (unit)

- Unit: `apps/kernel-server` → `personal::route_observation` plus
  `provider_proxy` timing split
- Environment: `DEV-WIN-GNU-01` **not-run** (`RUST-LINK-DEV-WIN-GNU-01`); routed
  to exact-revision `DEV-LINUX-NATIVE-01` / required CI
- Result: **not-run** locally. The tests are written: malformed/duplicate/absent
  correlation ids are refused rather than echoed; an unauthorized daemon emits
  an empty header block; a zero-duration stage is dropped; an authorized joined
  request emits only the correlation echo and preflight header (no body, no
  `\r\n\r\n`, no secret-shaped material); authorized vs unauthorized header
  blocks leave identical body bytes; `forward_chat_completion_with_timing`
  returns a positive preflight and a positive network duration with the
  transport body unchanged.

### U05 — front-door header-only effect

- Unit: `apps/kernel-server/tests/p9_t05_route_observation.rs`
- Environment: `DEV-WIN-GNU-01` **not-run** (`RUST-LINK-DEV-WIN-GNU-01`); routed
  to exact-revision `DEV-LINUX-NATIVE-01` / required CI
- Result: **not-run** locally. The tests are written: with and without daemon
  observation authorization, a missing Provider config still returns
  `PERSONAL_PROVIDER_NOT_CONFIGURED`; absent, well-formed, malformed (including
  secret-shaped) and duplicated correlation headers produce the identical error
  body; no observation headers appear on the error path; the refused value and
  session/bootstrap secrets never appear in the response; the runtime root gains
  no observation or campaign file.
