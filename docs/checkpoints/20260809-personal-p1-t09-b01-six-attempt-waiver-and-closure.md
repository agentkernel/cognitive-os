# P1-T09 B01 six-attempt waiver and closure handoff

- Date: 2026-08-09
- Task: `P1-T09`
- Gate: `B01`
- Campaign: `B01-clean-linux-first-install-first-conversation-002`
- Change class: owner-approved `product-semantic`

## Owner waiver for transition Attempt 7

Attempt 7 crossed the clean-reset checkpoint before the owner completed the
N=6 decision. The guest was then immediately reverted. No artifact, Pi,
Provider credential, product service, route runner, request, response, or
authority state was created. The owner explicitly waived Attempt 7 from the
revised `002` N=6 denominator. This record preserves rather than deletes that
transition fact.

## Revised closure input

The revised denominator is the immutable ordered Attempt 1--6 ledger:
five successes, one failure, zero critical safety failures, and no remaining
attempts. The numerical threshold is satisfied at `5/6 = 83.33%` under
[ADR-0039](../adr/0039-personal-b01-six-attempt-campaign-policy.md).

## Remaining independent work

B01 remains `running`. The independent verifier must review the complete
aggregate, including the named binomial interval method, successful-run median
and p95, all failure categories, zero-critical result, waiver boundary, and
redaction. Only an affirmative disposition may promote B01.

## Non-claims

This waiver and handoff do not declare B01, G1, GMVP-LINUX, release, or Profile
pass. Campaign `001` remains its own immutable N=20 failure.
