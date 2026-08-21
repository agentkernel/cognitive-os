# P1-T09 B01 statistical interpretation addendum

- Date: 2026-08-02
- Task: `P1-T09`
- Gate: `B01`
- Change class: owner-approved `product-semantic` statistical interpretation
- Current task status: unchanged, `in-progress`
- Current Gate status: unchanged, `running`
- Current evidence: unchanged, attempt 1 of the required fixed campaign passed
- Claim scope: unchanged, non-claim for B01 pass, reliability, release and Profile

## 1. Purpose and authority boundary

This addendum resolves the statistical wording conflict between the
[original preregistration](./20260731-personal-p1-t09-b01-preregistration.md),
whose `Success threshold` row describes one attempt, and the formal plan's
campaign-level requirement of at least 20 attempts. It is an interpretation
and campaign-accounting addendum. It does not edit, replace, invalidate or
rerun the original preregistration or
[attempt 1 ledger](./20260801-personal-p1-t09-b01-attempt-ledger.md).

The original phrases "one clean Linux x86_64 first install", "one started
attempt" and "no retries ... for a passing B01 claim" define the atomic
execution unit and the pass criteria for one numbered attempt. They do not
define the campaign denominator and cannot authorize a campaign-level pass
after one attempt. A retry of any failed or uncertain execution is a new
numbered attempt, never a replacement for the prior outcome.

This addendum does not change the formal thresholds:

- fixed campaign denominator of at least 20 attempts;
- campaign success rate of at least 90%;
- zero critical safety failures;
- complete aggregate statistics and independent verifier disposition.

## 2. Fixed denominator and attempt accounting

For `B01-clean-linux-first-install-first-conversation-001`, the fixed planned
denominator is **N = 20** numbered attempts. N was fixed before attempt 2 and
will not be reduced in response to observed results. A later owner-approved
campaign may preregister a larger fixed N before its next attempt, but it may
not retrospectively exclude, renumber or replace any already started attempt.

Every invocation that crosses the original clean-reset attempt checkpoint is a
started attempt and enters the denominator. This includes success, timeout,
nonzero exit, readiness failure, setup failure after the checkpoint, missing
marker, cleanup failure and an interrupted or uncertain result. All started
attempts remain in the immutable ledger. Environment qualification work that
does not cross that checkpoint is not an attempt, consistent with the original
preregistration.

For N = 20, at least 18 attempts must satisfy every per-attempt success
criterion. A failure cannot be retried under the same attempt number, hidden,
deleted or replaced by a later success.

## 3. Decision rule and no optional stopping

B01 can pass only after all 20 fixed attempts have reached recorded outcomes
and the independent verifier has accepted the complete denominator and
statistics. Interim success rate, confidence intervals, median or p95 cannot
be used for early pass. The operator may not stop when the result first looks
favorable, extend N only because the result looks unfavorable, or choose a
reported subset after observing outcomes.

Any critical safety failure makes the campaign ineligible to pass. A mandatory
safety stop after such a failure is not optional stopping for success: the
campaign disposition is fail, never an early pass. The critical set includes
the original secret/internal-material disclosure, direct authority or mutating
Tool bypass, synthetic readiness acceptance, authority side effects, omitted
started attempt, or evidence/cleanup integrity failure designated critical by
the preregistered campaign.

## 4. Aggregate report

The final campaign report must include:

1. fixed planned N and the complete ordered ledger of all started attempts;
2. success count, failure count and success rate over that denominator;
3. a two-sided 95% binomial confidence interval for the success proportion,
   with the exact calculation method named;
4. aggregate median and p95 time-to-first-conversation over successful
   attempts, plus two-sided 95% confidence intervals with the bootstrap method,
   sample count and deterministic analysis version/seed recorded;
5. every failure category and the zero-critical-safety-failure disposition;
6. artifact, environment, Pi and evidence-collector pins without secret or
   sensitive response material;
7. explicit non-claims for any metric or environment not preregistered.

The threshold is the observed campaign success rate `successes / all started
attempts >= 0.90`; the confidence interval is reported uncertainty and is not
a substitute threshold. Median, p95 and their intervals are descriptive and
cannot override a failed success-rate or critical-safety decision.

## 5. Independent verifier disposition

The independent verifier must confirm, without relying on the operator's
summary alone:

- the fixed N and absence of optional stopping or selective extension;
- the one-to-one sequence of reset checkpoints, numbered attempts and ledger
  outcomes;
- retention of attempt 1 exactly as recorded;
- inclusion of every started failure and uncertain outcome;
- reproducibility of success-rate, median, p95 and confidence-interval
  calculations from the redacted ledger;
- zero critical safety failures and no secret-bearing evidence;
- satisfaction of the formal `>=20`, `>=90%` and zero-critical thresholds.

Only an affirmative final verifier disposition after the fixed campaign is
complete can support changing B01 from `running` to `pass` in the canonical
current snapshot.

## 6. Current non-claim

This statistical interpretation is not evidence that reliability is at least
90%. Attempt 1 remains one successful formal attempt and no more. The complete
denominator, aggregate analysis and final independent verifier disposition do
not yet exist, so `P1-T09` remains `in-progress`, B01 remains `running`,
`GMVP-LINUX` remains `not-run`, and no release or Profile claim is created.
