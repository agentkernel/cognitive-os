# Context Evolution (design)

- Status: informative Personal architecture design
- Related: AXIOMS P1, [ADR-0042](../../../docs/adr/0042-personal-three-pillar-engineering.md),
  P3-T02/P3-T04, P8-T05

## 1. Current baseline (already shipped design/impl path)

Authorize-before-rank, required fail-closed budgets, explicit loss, digest-bound
source traces, stable-prefix/delta **metadata** reuse with full revalidation, and
B03 correctness as the Gate signal (benefit observations non-blocking).

## 2. Planned evolution (P8-T05)

1. **Compaction:** daemon-owned summarization of session/Context material into
   digest-bound compact artifacts with explicit loss records.
2. **Adaptive budgets:** adjust fragment budgets from durable loop telemetry
   without skipping body reauthorization.
3. **Benefit observation:** UCR-01-compatible raw measurements; never Gate
   authority from the runner.

## 3. Hard constraints

- Compaction output is a Context source candidate, not Task completion.
- Stale/revoked material never re-enters via cache or compact reuse.
- Model-written summaries cannot self-authorize inclusion.

## 4. Non-claims

Design only until P8-T05 acceptance. No B06/B07/Gate promotion from this file.
