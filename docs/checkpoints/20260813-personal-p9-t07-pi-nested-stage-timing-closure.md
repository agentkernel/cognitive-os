# P9-T07 Pi nested stage timing — closure

- Task: `P9-T07` (corrected from the parallel pre-merge P9-T05 registration)
- Lease: `lease/personal/P9-T07/pi-nested-stage-timing`
- Branch: `personal/P9-T05-pi-nested-stage-timing` (legacy name retained; no history rewrite)
- Draft PR: [#216](https://github.com/agentkernel/cognitive-os/pull/216)
- Accepted implementation head: `20f2636ea34619e3d01d870326dc8ac5e8678265`
- Change class: implementation-only product instrumentation plus corrective task identity; no normative machine contract change

## Acceptance mapping

1. **Per-request nested stages:** one completed attempt contains five sequential
   Pi-domain stages plus daemon `preflight` and `provider_network` nested inside
   `loopback_wait`. Pi uses Node's monotonic clock; daemon uses Rust `Instant`.
   Wall-clock time is absent and cross-domain subtraction is forbidden.
2. **Terminal and missing-field semantics:** every authorized started attempt
   publishes one exact-schema record. `completed` requires the full Pi route;
   `cancelled`/`error` retain the exact measured prefix, terminal stage and a
   content-free failure class. Joined absence is explicit JSON `null`;
   unjoined daemon pairs carry one registered reason.
3. **Concurrent correlation:** each request mints one opaque
   `campaign-<32 lowercase hex>` id. Concurrent request ids are distinct and
   match exactly the daemon request/echo set; malformed, duplicate, mismatched
   and cross-campaign publication fail closed.
4. **Provider usage provenance:** counters become `measured` only at the
   authenticated daemon-response parser. The in-process evidence is bound to
   the exact correlation id, frozen, and consumable by one publication across
   campaign sessions. Caller-built, replayed, partial, fractional, negative,
   inconsistent or unknown-labelled usage is refused or `not_available`.
5. **Path matrix:** disabled instrumentation creates no session/timer/record;
   no-Provider and protocol failures retain error records; cancellation retains
   a cancelled record; the product request remains `stream:false`, while
   `stream:true` is rejected before secret resolution and cannot appear as a
   successful streaming observation.
6. **Content and authority boundary:** exact top-level/stage/usage key sets
   reject extra prompt/response/header/authorization fields. No package-owned
   filesystem writer, database, capability, Task/Intent/Effect transition or
   measurement-only network request exists.

## Validation

- Failure-first/red-green ledger: [running report](20260813-personal-p9-t07-pi-nested-stage-timing-validation-report.md).
- Local Pi package: **90/90 pass** after merging `origin/main@d24f7d00`.
- Exact native Linux daemon proof at unchanged implementation revision
  `554c6cf9f69af836032af207eeb04a800ac55063`: route observation **9/9**,
  Provider timing split **1/1**, front-door target **2/2**, fmt and
  kernel-server Clippy pass.
- Required CI at main-reconciled head `20f2636e`: run
  [31732673976](https://github.com/agentkernel/cognitive-os/actions/runs/31732673976)
  passed Ubuntu and Windows.
- Local `cargo fmt --check`, consistency, bilingual handbook, generated-page
  byte gate, docs-sync gate and diff check pass.
- Instrumentation-only local probe (1,000 in-memory records; no daemon or
  Provider): p50 `80.0 µs`, p95 `327.4 µs`. Fixture-only; not a benchmark.

## EVAL-003 / PI-NESTED disposition

Capability is reachable after merge, but execution is **not-run**. No
`EVAL-003` owner-directed campaign row, frozen execution plan, manifest, seed,
oracle, runner or running report exists in repository/GitHub facts. Inventing
one after implementation would violate preregistration. No B01 guest, snapshot,
credential or campaign process was touched.

## Non-claims

No stage is attributed as the earlier approximately `1.8 s` O-arm overhead.
The usage provenance is not a cryptographic attestation of upstream Provider
accounting. No benchmark, Gate, release, Profile, B01 or Agent-benefit claim is
created.
