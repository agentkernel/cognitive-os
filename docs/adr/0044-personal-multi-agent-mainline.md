# ADR-0044: Personal Multi-Agent Mainline

- Status: Accepted
- Date: 2026-08-10
- Decision owner: CognitiveOS Personal product owner
- Classification: product-semantic documentation decision
- Related: P8-T01, P8-T02, P8-T03, ADR-0034, ADR-0043

## Context

Earlier Personal planning treated Multi-Agent as a deferred capability train
where NO-GO / default-off was a legitimate Linux 1.0 outcome. Owner direction
for the 2.0 design baseline requires multi-agent collaboration to be a
**documented mainline design path** with fail-closed isolation, while keeping
Linux 1.0 claim boundaries unchanged (Pi remains the only qualified agent).

## Decision

1. Multi-agent orchestration is a first-class architecture chapter and product
   concept: multiple agents may propose candidates into shared Task/Context
   governance under daemon arbitration.
2. Linux 1.0 claim set is unchanged: only Pi is product-qualified; multi-agent
   runtime remains non-claim for 1.0.
3. Default posture for unregistered multi-agent collaboration remains
   fail-closed / off until an agent is independently qualified and an owner
   enables the path.
4. NO-GO remains a legitimate *qualification* result for a specific agent or
   campaign; it is no longer a standing product statement that multi-agent is
   out of design scope.

## Consequences

- Architecture docs describe orchestration, isolation, and arbitration without
  inventing a second authority writer.
- Phase 8 tasks implement adapter and first non-Pi qualification under this
  mainline design.
- GMVP-LINUX and Profile matrices are not expanded by this ADR.

## Non-goals and non-claims

No multi-agent runtime implementation, no B09 transfer, no Gate/release/Profile
evidence.
