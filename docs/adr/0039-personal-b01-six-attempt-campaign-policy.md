# ADR-0039: Personal B01 Six-Attempt Campaign Policy

- Status: Accepted
- Date: 2026-08-09
- Decision owner: CognitiveOS Personal product owner
- Classification: product-semantic documentation decision
- Related: P1-T09, B01, G1, GMVP-LINUX, ADR-0034, ADR-0036
- Supersedes: the B01 campaign denominator and threshold portions of the
  2026-08-02 statistical interpretation addendum for active successor campaign
  `B01-clean-linux-first-install-first-conversation-002`

## Context

The retained B01 campaign `001` remains an immutable failed N=20 record. Its
attempts and result do not transfer. Separately preregistered successor `002`
recorded six clean-reset outcomes: five successful routes and one graphical
Desktop readiness failure. The owner has selected a smaller, fixed campaign
for the active successor so that B01 measures six fully independent clean
install-to-first-conversation executions rather than the former N=20 rule.

Attempt 7 crossed a reset checkpoint while this policy decision was being
requested. It installed no artifact or Pi, received no Provider credential,
and executed no product route. The owner expressly waived it from the revised
denominator; its audit record remains retained.

## Decision

For successor campaign `002` only, B01 uses the following campaign rule:

1. the fixed denominator is exactly **N=6**;
2. every one of its six numbered outcomes remains immutable and counted;
3. at least **5 of 6** attempts must succeed (`>= 83.33%` observed rate);
4. critical safety failures must equal zero;
5. a complete aggregate report and affirmative independent-verifier
   disposition are mandatory before B01 can become `pass`.

The retained campaign `001` remains governed by its historical N=20 rule and
cannot be revived, reclassified, or used toward successor `002`. Attempt 7 is
an auditable cancelled transition, not a sixth-success replacement or an
unrecorded execution. No further B01 attempt is needed merely to satisfy the
superseded N=20 denominator.

## Consequences

- `5/6` satisfies the revised numerical threshold but does not alone pass B01.
- The aggregate must retain all six outcomes, describe the 95% binomial
  interval and successful-route median/p95, and report every failure category.
- An independent verifier must affirm the ledger, arithmetic, zero-critical
  result, redaction boundary, and aggregate before `PROGRESS.md` may change
  B01 from `running` to `pass`.
- The B01 test checker must reject incomplete denominator, insufficient
  successes, critical failures, malformed arithmetic, missing aggregate, and
  missing affirmative verifier closure.

## Non-goals and non-claims

This decision does not change public requirements, schemas, registry entries,
transitions, vectors, implementation authority, product release composition,
or Profile conformance. It creates no B01, G1, GMVP-LINUX, release, or Profile
pass claim by itself.
