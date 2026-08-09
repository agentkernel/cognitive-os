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

## Redacted aggregate report

| Measure | Result | Method |
|---|---:|---|
| Counted outcomes | 6 | Immutable ordered Attempts 1--6 in the successor ledger |
| Successes / failures | 5 / 1 | `5 + 1 = 6`; Attempt 2 graphical Desktop readiness failure remains counted |
| Observed success rate | 83.33% | `5 / 6` |
| 95% binomial interval | 43.64%--96.99% | Two-sided Wilson score interval, `z=1.96`, `n=6`, `x=5`; descriptive only |
| Critical safety failures | 0 | Review of all retained redacted outcome and cleanup records |
| Successful-route durations | 5855, 5518, 6315, 5409, 5473 ms | Bounded route-runner output for Attempts 1, 3, 4, 5, 6 |
| Median TTFC | 5518 ms | Middle value after ascending sort: 5409, 5473, 5518, 5855, 6315 |
| p95 TTFC | 6315 ms | Nearest-rank percentile: `ceil(0.95 * 5) = 5` |

All successful routes used the independently verified signed artifact,
registered Pi `0.81.1`, graphical hidden Provider input, redacted doctor
readiness, bounded marker response, SecretStore cleanup, and baseline revert.
Attempt 2 is the sole counted failure category. No credential, Provider request
or response body, SecretRef, or SQLite detail is retained in this report.

## Independent verifier disposition

Verifier B independently reviewed ADR-0039, the successor ledger, this
aggregate, the waiver boundary, native artifact verification, and the Current
snapshot. The review affirms that the fixed counted denominator is N=6, the
immutable ledger reconciles to 5 successes / 1 failure / 0 critical safety
failures, and Attempt 7 is retained but excluded under the explicit owner
waiver. It independently reproduced the stated Wilson interval, median, and
nearest-rank p95 from the redacted values and found the cleanup, artifact, and
redaction evidence internally consistent.

The disposition covers only redacted successor `002` closure inputs. It does
not reproduce guest operations or inspect omitted secret-bearing material, does
not alter retained failed campaign `001`, and does not make a release or Profile
claim. It is an affirmative B01 campaign-verifier disposition under ADR-0039.

## Non-claims

This closure permits only the canonical B01 status update for successor `002`.
G1, GMVP-LINUX, release, and Profile remain unclaimed. Campaign `001` remains
its own immutable N=20 failure.
