# P9-T07 running validation report — Pi route nested stage timing

Single running report for `P9-T07` under Operating Model `TEST-REPORT-INCREMENTAL-01`.
Every finished validation unit is appended here immediately, before the next unit
starts. Entries are append-only; a later entry may supersede an earlier one only by
stating so explicitly.

- Task: `P9-T07` — Pi route nested per-request stage timing and Provider usage exposure
- Branch: `personal/P9-T05-pi-nested-stage-timing`
- Base: `origin/main@326f97728ab6aaaacceaedd2156d953231b32e01`
- Lease: `lease/personal/P9-T07/pi-nested-stage-timing`
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
  gap `P9-T07` must close; it is not a defect claim about any measured latency.

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
- Environment: `DEV-LINUX-NATIVE-01` at exact
  `554c6cf9f69af836032af207eeb04a800ac55063`
- Result: **pass** on `DEV-LINUX-NATIVE-01` at exact
  `554c6cf9f69af836032af207eeb04a800ac55063`.
  `cargo test -p kernel-server route_observation` **9/9**;
  `timed_forward_splits_preflight` **1/1**;
  `cargo fmt --all -- --check` **pass**;
  `cargo clippy -p kernel-server --all-targets -- -D warnings` **pass**.
  Local `DEV-WIN-GNU-01` remains **not-run** (`RUST-LINK-DEV-WIN-GNU-01`).

### U05 — front-door header-only effect

- Unit: `apps/kernel-server/tests/p9_t07_route_observation.rs`
- Environment: `DEV-LINUX-NATIVE-01` at exact
  `554c6cf9f69af836032af207eeb04a800ac55063`
- Command: `cargo test -p kernel-server --test p9_t07_route_observation`
- Result: **pass** — 2/2. With and without daemon observation authorization, a
  missing Provider config still returns `PERSONAL_PROVIDER_NOT_CONFIGURED`;
  absent, well-formed, malformed (including secret-shaped) and duplicated
  correlation headers produce the identical error body; no observation headers
  appear on the error path; the refused value and session/bootstrap secrets
  never appear in the response; the runtime root gains no observation or
  campaign file. Local `DEV-WIN-GNU-01` remains **not-run**.

### U06 — failure-first: terminal outcomes and Provider-usage provenance

- Unit: `packages/pi-cognitiveos` targeted observation/provider suites
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation.test.js" "dist/pi-route-observation-negatives.test.js" "dist/daemon-provider.test.js"`
- Result: **fail (expected)** — 31/35 pass, with four discriminating contract
  failures:
  - a pre-dispatch cancellation produced 0 records instead of retaining one
    `cancelled` sample;
  - a no-Provider refusal produced 0 records instead of retaining one `error`
    sample;
  - self-asserted, internally consistent counters were accepted as measured
    Provider usage rather than refused for missing response provenance;
  - a successful record did not state the fixed `non_streaming` request mode or
    terminal outcome.
- Concurrent request correlation already passed in the same red run: two
  overlapping Pi requests produced two distinct ids whose set exactly matched
  the two daemon request headers. That protects an existing property; it is not
  counted as a newly implemented behavior.
- Non-claim: fixture-only failure-first evidence; no Provider, benchmark, Gate,
  release, Profile, B01 or overhead-attribution result.

### U07 — terminal outcomes, concurrent correlation and usage provenance

- Unit: `packages/pi-cognitiveos` targeted observation/provider suites
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation.test.js" "dist/pi-route-observation-negatives.test.js" "dist/daemon-provider.test.js"`
- Result: **pass** — 35/35.
- Evidence:
  - every authorized started attempt publishes one content-free terminal record:
    success is `completed`, pre-dispatch abort is `cancelled`, and a
    `PERSONAL_PROVIDER_NOT_CONFIGURED` refusal is `error`;
  - each record states `requestMode=non_streaming` and the last measured Pi
    stage (or `before_request`), so partial prefixes are explicit without
    inventing missing durations;
  - two overlapping requests retained two unique correlation ids, and their set
    exactly matched the two ids observed by the daemon;
  - internally consistent counters built by a caller were refused both at
    assembly and publication; measured counters are accepted only when the
    authenticated daemon-response parser created the in-process provenance
    marker. Missing or inconsistent counters remain `not_available`.
- Clock boundary: all Pi durations remain on Node's monotonic clock, daemon
  durations remain on Rust `Instant`, and validation performs no cross-domain
  subtraction; only nested elapsed-duration containment is checked.
- Non-claim: loopback fixtures only; this is no Provider, benchmark, Gate,
  release, Profile, B01 or overhead-attribution result.

### U08 — failure-first: terminal failure classification

- Unit: `packages/pi-cognitiveos` success/cancellation/error route contracts
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation.test.js" "dist/daemon-provider.test.js"`
- Result: **fail (expected)** — 4/8 pass. Completed, pre-dispatch-cancelled,
  no-Provider and malformed-Provider-response records all lacked the required
  content-free `failureClass` label.
- Required distinction: `none`, `cancelled`, `provider_unavailable` and
  `protocol_error`; no response body, message, status text or credential may be
  copied into the observation.

### U09 — terminal failure classification

- Unit: `packages/pi-cognitiveos` targeted observation/provider suites
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation.test.js" "dist/pi-route-observation-negatives.test.js" "dist/daemon-provider.test.js"`
- Result: **pass** — 35/35.
- Evidence: completed, cancelled, no-Provider and malformed-response samples
  carry respectively `none`, `cancelled`, `provider_unavailable` and
  `protocol_error`. Classification derives only from typed local/daemon error
  codes; no raw error text or body enters the observation.

### U10 — complete Pi package regression

- Unit: `packages/pi-cognitiveos` full Node suite
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run build; pnpm test`
- Result: **pass** — 84/84.
- Evidence boundary: includes the existing non-streaming request-body contract
  (`stream:false`), default-disabled behavior, Provider/no-Provider and protocol
  error paths, pre-dispatch cancellation, complete/partial usage, concurrent
  correlation, content exclusion, no filesystem writer, and no extra request.
  Rust's existing `stream:true` rejection remains a separate supported-CI unit;
  this Node result does not stand in for it.

### U11 — handbook check before generated-page refresh

- Unit: bilingual handbook machine model
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:handbook`
- Result: **fail** — four expected synchronization violations, all on the
  generated `reference/http-api.md` pair: HB008 fingerprint drift plus HB010
  generated-byte drift after the daemon route source changed.
- Recovery: run the registered generator, never hand-edit generated pages, then
  rerun the check. This is a docs synchronization failure, not a product-test
  result.

### U12 — generated handbook refresh

- Unit: registered handbook generator
- Environment: `DEV-WIN-GNU-01`
- Command: `node tools/src/generate-handbook.mjs`
- Result: **pass**. Generated reference families were rebuilt from canonical
  sources; the following check determines byte/fingerprint equality.

### U13 — handbook machine gate

- Unit: bilingual handbook machine model after regeneration
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:handbook`
- Result: **pass** — 54 documents × 2 locales, 9 generated families, with
  coverage/link/fingerprint/status/secret checks verified.

### U14 — generated handbook byte gate

- Unit: generated handbook reproducibility
- Environment: `DEV-WIN-GNU-01`
- Command: `node tools/src/generate-handbook.mjs --check`
- Result: **pass** — 18 generated pages byte-identical.

### U15 — repository consistency

- Unit: repository static consistency
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:consistency`
- Result: **pass** — 275 requirements, 55 error codes, 74 schemas, 89 vectors,
  links, traceability, Personal plan/Gates, environment routing,
  task/checkpoint-delivery and active-lease checks verified.

### U16 — Rust formatting gate

- Unit: workspace Rust formatting
- Environment: `DEV-WIN-GNU-01` (non-linking allowlist)
- Command: `cargo fmt --all -- --check`
- Result: **pass**. No Rust build/test/Clippy was attempted locally.

### U17 — failure-first: retained observation immutability

- Unit: `packages/pi-cognitiveos` usage/stage mutation negative
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **fail (expected)** — 27/28 pass. A runner holding the in-memory
  record could mutate a measured usage counter and a stage duration after
  publication.
- Required behavior: freeze the published record and its nested stages/usage so
  post-publication mutation cannot turn authentic counters into different
  evidence.

### U18 — retained observation immutability

- Unit: `packages/pi-cognitiveos` usage/stage mutation negative
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **pass** — 28/28. Published observations, stage objects/array and
  usage object are frozen before entering the retained session view or injected
  sink, so runner-side mutation throws and leaves the measured values unchanged.

### U19 — failure-first: exact content-free schema

- Unit: `packages/pi-cognitiveos` extra-field injection negative
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **fail (expected)** — 28/29 pass. A structurally valid record carrying
  extra `prompt` and `authorization` fields reached the injected sink.
- Required behavior: validate the exact top-level, stage and usage key sets
  before serialization; unknown fields are schema failures, not tolerated
  extensions.

### U20 — exact content-free schema

- Unit: `packages/pi-cognitiveos` observation negative suite
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **pass** — 29/29. Exact top-level, stage and usage key sets are
  validated before retention/serialization; extra prompt/authorization fields
  are refused and produce zero sink writes. Joined daemon-stage absence is
  represented as explicit JSON `null`, not an omitted field.

### U21 — failure-first: usage/request binding

- Unit: `packages/pi-cognitiveos` usage replay negative
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **fail (expected)** — 29/30 pass. A genuinely parsed measured-usage
  object could be replayed under a second correlation id.
- Required behavior: bind the in-process usage provenance marker to the exact
  request correlation id; session duplicate-id refusal then prevents both
  replay directions.

### U22 — usage/request binding

- Unit: `packages/pi-cognitiveos` observation negative suite
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **pass** — 30/30. The measured-usage provenance marker is bound to
  the exact request correlation id; replay under another id is refused, while a
  second publication under the original id is already blocked by id uniqueness.

### U23 — failure-first: cross-session usage replay

- Unit: `packages/pi-cognitiveos` usage replay negative
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **fail (expected)** — 29/30 pass. Reopening a campaign session allowed
  the already-published measured-usage object to be retained again under its
  original correlation id.
- Required behavior: measured usage is single-publication evidence across all
  in-process sessions, not merely unique inside one session.

### U24 — single-publication usage evidence

- Unit: `packages/pi-cognitiveos` observation negative suite
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **pass** — 30/30. Measured usage is correlation-bound, immutable and
  consumable by exactly one publication across in-process campaign sessions.

### U25 — final complete Pi package regression

- Unit: `packages/pi-cognitiveos` full Node suite
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run build; pnpm test`
- Result: **pass** — 87/87, including all added terminal-schema, concurrency,
  exact-field, immutability and usage-provenance negatives.

### U26 — final handwritten-page fingerprint refresh

- Unit: handbook source fingerprints
- Environment: `DEV-WIN-GNU-01`
- Command: `node tools/src/fill-handbook-fingerprints.mjs`
- Result: **pass** — the six affected bilingual Pi/client/lifecycle pages were
  refreshed from the final implementation sources.

### U27 — generated page detects final client-source drift

- Unit: bilingual handbook machine model
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:handbook`
- Result: **fail** — four HB008/HB010 violations on the generated HTTP reference
  pair after the final `daemon-client.ts` usage-binding change.
- Recovery: regenerate again from the final source revision, then rerun both
  handbook gates. No generated page is hand-edited.

### U28 — final generated handbook refresh

- Unit: registered handbook generator
- Environment: `DEV-WIN-GNU-01`
- Command: `node tools/src/generate-handbook.mjs`
- Result: **pass**. Generated reference bytes/fingerprints now consume the final
  client source; following gates verify equality.

### U29 — final handbook machine gate

- Unit: bilingual handbook machine model
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:handbook`
- Result: **pass** — 54 documents × 2 locales, 9 generated families; coverage,
  links, fingerprints, statuses and secret checks verified.

### U30 — final generated handbook byte gate

- Unit: generated handbook reproducibility
- Environment: `DEV-WIN-GNU-01`
- Command: `node tools/src/generate-handbook.mjs --check`
- Result: **pass** — 18 generated pages byte-identical.

### U31 — final repository consistency

- Unit: repository static consistency
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:consistency`
- Result: **pass** — 275 requirements, 55 error codes, 74 schemas, 89 vectors,
  links, traceability, Personal plan/Gates, environment routing,
  task/checkpoint delivery and leases verified.

### U32 — local instrumentation-only cost probe

- Unit: 1,000 in-memory records, each including correlation-id minting, ten
  monotonic clock reads, five stage closures, schema validation, freezing,
  serialization and bounded-session retention
- Environment: `DEV-WIN-GNU-01`, Node fixture only; no daemon or Provider
- Result: **pass for the narrow “not obviously self-dominating” check** —
  p50 `80.0 µs`, p95 `327.4 µs`, max `5.1812 ms` (runtime/GC outlier), 1,000/1,000
  retained.
- Non-claim: this is not a benchmark, production latency result, Gate, B01 or
  overhead-attribution cell. It only excludes an instrumentation implementation
  that mechanically adds milliseconds at the median before any real route work;
  `EVAL-003` would still need its own frozen paired design.

### U33 — failure-first: unknown usage availability label

- Unit: `packages/pi-cognitiveos` usage discriminator negative
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **fail (expected)** — 30/31 pass. A counter object labelled
  `availability=estimated` bypassed the measured-usage provenance branch.
- Required behavior: only the exact `not_available` and `measured`
  discriminators exist; every other label is inconsistent and refused.

### U34 — strict usage discriminator

- Unit: `packages/pi-cognitiveos` observation negative suite
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **pass** — 31/31. Any availability label other than exact
  `not_available`/`measured` is refused before provenance or retention.

### U35 — failure-first: unknown daemon-stage availability

- Unit: `packages/pi-cognitiveos` daemon-domain discriminator negative
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **fail (expected)** — 31/32 pass. A complete nested pair labelled
  `daemonStages=estimated` was treated as joined.
- Required behavior: only exact `joined`/`not_available` labels exist; unknown
  availability cannot bypass domain validation.

### U36 — strict daemon-stage discriminator

- Unit: `packages/pi-cognitiveos` observation negative suite
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **pass** — 32/32. Unknown daemon-stage availability labels are
  refused before they can be interpreted as a joined nested pair.

### U37 — failure-first: unknown daemon-stage reason

- Unit: `packages/pi-cognitiveos` daemon-domain reason negative
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **fail (expected)** — 32/33 pass. An unregistered
  `daemonStagesUnavailableReason` value was accepted.
- Required behavior: every unavailable nested pair carries one exact registered
  reason; unknown reasons cannot become report strata.

### U38 — registered daemon-stage reasons

- Unit: `packages/pi-cognitiveos` observation negative suite
- Environment: `DEV-WIN-GNU-01`
- Command:
  `pnpm run build; node --test "dist/pi-route-observation-negatives.test.js"`
- Result: **pass** — 33/33. Unavailable nested stages accept only the six
  registered reason labels; unknown values are refused.

### U39 — accepted complete Pi package regression

- Unit: `packages/pi-cognitiveos` full Node suite
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run build; pnpm test`
- Result: **pass** — 90/90. This supersedes U25 as the final local Node
  acceptance result.

### U40 — accepted source fingerprint refresh

- Unit: handbook source fingerprints
- Environment: `DEV-WIN-GNU-01`
- Command: `node tools/src/fill-handbook-fingerprints.mjs`
- Result: **pass** — six affected bilingual pages refreshed from accepted
  implementation sources.

### U41 — accepted handbook machine gate

- Unit: bilingual handbook machine model
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:handbook`
- Result: **pass** — 54 documents × 2 locales, 9 generated families; coverage,
  links, fingerprints, statuses and secret checks verified.

### U42 — accepted generated-page byte gate

- Unit: generated handbook reproducibility
- Environment: `DEV-WIN-GNU-01`
- Command: `node tools/src/generate-handbook.mjs --check`
- Result: **pass** — 18 generated pages byte-identical.

### U43 — accepted consistency gate

- Unit: repository static consistency
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:consistency`
- Result: **pass** — 275 requirements, 55 error codes, 74 schemas, 89 vectors,
  links, traceability, Personal plan/Gates, environment routing,
  task/checkpoint delivery and leases verified.

### U44 — staged docs-sync gate

- Unit: exact staged checkpoint change set
- Environment: `DEV-WIN-GNU-01`
- Command: `node tools/src/docs-sync-gate.mjs --staged`
- Result: **pass**. Eight Pi-source paths mapped to `user.pi-shell`,
  `dev.agent-pi-lifecycle` and generated environment-variable reference;
  eight handbook paths mapped to the handbook-owning pages. The nested
  handbook and generator checks both passed.

### U45 — canonical task-id reconciliation

- Unit: merge `origin/main@d24f7d00` after PR #213
- Result: **corrective renumber, no product change**. Main had already assigned
  P9-T05 to the merged daemon readiness-wait repair, while the piinstrument
  branch had independently registered the same id. P9-T06 is also assigned to
  the separate readiness/SecretStore workflow, so this task, lease, slices,
  test/report paths and current facts are corrected to the next unused
  `P9-T07`. Draft PR #216 and its legacy-named branch are retained to avoid
  rewriting pushed history.
- Boundary: only task identity/evidence names changed; implementation semantics,
  frozen test results and all non-claims are unchanged.

### U46 — post-merge Rust format gate

- Unit: merged workspace Rust formatting
- Environment: `DEV-WIN-GNU-01` (non-linking allowlist)
- Command: `cargo fmt --all -- --check`
- Result: **pass**. No local Rust build/test/Clippy was attempted.

### U47 — post-merge Pi regression

- Unit: `packages/pi-cognitiveos` full Node suite after `origin/main` merge
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run build; pnpm test`
- Result: **pass** — 90/90.

### U48 — post-merge fingerprint refresh

- Unit: handbook source fingerprints after test/report task renumber
- Environment: `DEV-WIN-GNU-01`
- Command: `node tools/src/fill-handbook-fingerprints.mjs`
- Result: **pass** — ten affected bilingual lifecycle/daemon/client/Pi/Provider
  pages refreshed from reconciled sources.

### U49 — post-merge handbook machine gate

- Unit: bilingual handbook machine model
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:handbook`
- Result: **pass** — 54 documents × 2 locales, 9 generated families; coverage,
  links, fingerprints, statuses and secret checks verified.

### U50 — post-merge generated-page byte gate

- Unit: generated handbook reproducibility
- Environment: `DEV-WIN-GNU-01`
- Command: `node tools/src/generate-handbook.mjs --check`
- Result: **pass** — 18 generated pages byte-identical.

### U51 — consistency rejects self-owned lease ledger

- Unit: corrected P9-T07 task/lease state
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:consistency`
- Result: **fail** — one precise violation: an active lease must not list
  `PARALLEL-LANES.md` as its own writable path.
- Recovery: remove the ledger itself from the exact-path set; the lease row
  remains the authority granting the other paths.

### U52 — corrected task/lease consistency

- Unit: P9-T07 task, slices, counts and active exact-path lease
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:consistency`
- Result: **pass** — requirements/errors/schemas/vectors, Personal task counts,
  delivery slices, environment routing and leases verified.

### U53 — pre-merge handbook machine gate

- Unit: bilingual handbook machine model after canonical task reconciliation
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:handbook`
- Result: **pass** — 54 documents × 2 locales, 9 generated families; coverage,
  links, fingerprints, statuses and secret checks verified.

### U54 — pre-merge generated-page byte gate

- Unit: generated handbook reproducibility
- Environment: `DEV-WIN-GNU-01`
- Command: `node tools/src/generate-handbook.mjs --check`
- Result: **pass** — 18 generated pages byte-identical.

### U55 — required CI on main-reconciled implementation head

- Unit: required Ubuntu and Windows jobs
- Environment: `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01`
- Exact revision: `20f2636ea34619e3d01d870326dc8ac5e8678265`
- Run: [31732673976](https://github.com/agentkernel/cognitive-os/actions/runs/31732673976)
- Result: **pass** — Ubuntu `2m39s`, Windows `10m45s`. Both jobs passed
  TypeScript build/tests, Rust workspace build/tests, Clippy, rustfmt, codegen
  drift, consistency/traceability/handbook gates, conformance, report honesty,
  wrong-implementation self-check and cross-language golden comparison.
- Annotation: GitHub's Node-20 action deprecation warning is workflow
  infrastructure metadata, not a failing check or product result.

### U56 — formal acceptance and lease closure mapping

- Result: **pass**. P9-T07 acceptance maps to D01-D04 implementation,
  failure-first negatives, native daemon proof, required CI and documentation
  gates in the closure record. The corrective P9-T07 lease moved out of the
  active table into recent closed history; the formal task and D04 are `done`.
- `EVAL-003/PI-NESTED`: **not-run**, not failed — no owner-registered frozen
  campaign plan, runner or report exists, and no B01 state was touched.

### U57 — formal closure consistency

- Unit: P9-T07 done status, D04 closure, task counts and released lease
- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:consistency`
- Result: **pass** — all formal task, slice, count and lease invariants verified.

### U58 — closure handbook machine gate

- Environment: `DEV-WIN-GNU-01`
- Command: `pnpm run check:handbook`
- Result: **pass** — bilingual documents, source coverage, links, fingerprints,
  statuses and secret checks verified with the new closure report tracked.

### U59 — closure generated-page byte gate

- Environment: `DEV-WIN-GNU-01`
- Command: `node tools/src/generate-handbook.mjs --check`
- Result: **pass** — 18 generated pages byte-identical.

### U60 — running-report order integrity

- Result: **corrective pass**. U01–U59 content was preserved byte-for-byte per
  unit and mechanically reordered by unit number after append patches had
  inserted later units beside repeated command-result anchors. No result,
  denominator, claim or disposition changed.
