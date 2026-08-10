# ADR-0040: Personal B03 MVP Validation Policy

- Status: Accepted
- Date: 2026-08-10
- Decision owner: CognitiveOS Personal product owner
- Classification: product-semantic documentation decision
- Related: P3-T06, B03, B06, B07, P7-T08, GMVP-LINUX, ADR-0034, ADR-0037
- Supersedes: the formal-campaign and independent-verifier closure requirements
  for **P3-T06 MVP B03 acceptance only**

## Context

P3-T06 already has executable Context correctness coverage across the real
authority paths: Context storage, authorization/currentness/revocation,
Artifact CAS integrity, Context building and loss behavior, and governed
cache reuse. The owner directed that the MVP validate functional correctness
and collect performance evidence without making a separate statistical
workload campaign or an additional verifier ceremony a delivery mutex.

The previous P3-T06 documentation treated these implementation-level checks
as preparatory evidence only. That rule was appropriate for a larger campaign,
but adds process complexity without improving the MVP correctness signal.

## Decision

For the P3-T06 MVP B03 decision, the fixed validation denominator is:

1. 9 `m5_context_store` tests;
2. 3 `p3_t03_artifact_store` tests;
3. 8 `context_pipeline` tests;
4. 2 `context_cache` tests; and
5. 11 repository-tool/evaluator tests.

The MVP B03 pass conditions are all of the following:

1. all 33 tests pass at one exact reviewed revision;
2. the focused Rust checks run on qualified native Linux and pass Clippy with
   warnings denied;
3. required Ubuntu and Windows CI pass for the review revision;
4. the disposable native validation checkout is cleaned up and evidence is
   redacted; and
5. the product owner records an affirmative review of the bounded evidence.

B06/B07 measurements remain optional, repeatable raw performance observations.
They may describe Context delta, stable-prefix, cache, and loop behavior, but
are not an MVP B03 pass condition and cannot establish a general Agent-benefit
claim.

Larger real-workload campaigns, statistical denominators, and separately
assigned independent verification are deferred to release-promotion or later
performance work when their additional signal is needed. They are not a
P3-T06 MVP completion mutex.

## Consequences

- Existing native evidence has a deterministic MVP interpretation instead of
  requiring a second synthetic campaign.
- The B03 evaluator remains non-authoritative: tests and reports cannot mutate
  Gate state; the documented product decision owns the B03 status.
- P3-T06 may close after the fixed evidence, required CI, and normal PR/lease
  closure chain complete.
- The change does not reduce daemon-only authority, scope-before-ranking,
  required fail-closed, Artifact integrity, revocation, stale-cache, or
  explicit-loss requirements.

## Non-goals and non-claims

This decision does not pass GMVP-LINUX, create a release, establish Profile
conformance, change public schemas or authority boundaries, or claim B06/B07,
UCR-01 utility, or general Agent benefit.
