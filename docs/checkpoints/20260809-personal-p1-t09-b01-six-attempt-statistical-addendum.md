# P1-T09 B01 six-attempt statistical addendum

- Date: 2026-08-09
- Task: `P1-T09`
- Gate: `B01`
- Campaign: `B01-clean-linux-first-install-first-conversation-002`
- Change class: owner-approved `product-semantic`
- Authority: [ADR-0039](../adr/0039-personal-b01-six-attempt-campaign-policy.md)

## Supersession boundary

This addendum supersedes the denominator and threshold policy of the
2026-08-02 N=20 interpretation only for successor campaign `002`. It does not
change retained failed campaign `001`, delete a historical attempt, or replace
an execution result.

## Fixed campaign rule

- Fixed denominator: **N=6** immutable outcomes.
- Numerical threshold: at least **5 successes of 6** (`>= 83.33%`).
- Safety threshold: zero critical safety failures.
- Closure: complete aggregate statistics and affirmative independent-verifier
  disposition are required; no optional stopping or selective exclusion.

The six ordered outcomes are Attempts 1 through 6 in the successor ledger:
five successes and one graphical Desktop readiness failure. The failure remains
included. Attempt 7 was cancelled during the semantic-decision window under an
explicit owner waiver before artifact, Pi, Provider, or route activity; its
audit record is retained but it is outside this revised fixed denominator.

## Required aggregate and verifier closure

The closure report must retain the ordered ledger and provide: success/failure
arithmetic, a named two-sided 95% binomial interval method, median and p95 over
the five successful response durations, every failure category, zero-critical
disposition, and redacted artifact/environment/Pi evidence. The independent
verifier must independently affirm that report, the `5/6` threshold, the owner
waiver boundary, and the secret/evidence redaction boundary.

## Current non-claim

The numerical criterion is complete at `5/6`, but the aggregate report and
affirmative verifier disposition are not yet recorded. B01 remains `running`;
G1, GMVP-LINUX, release, and Profile remain unclaimed.
