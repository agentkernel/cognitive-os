<!--
Task: P3-T06
Gate: B03
Classification: campaign-preregistration-draft
Status: draft; not registered; not run
-->

# P3-T06 B03 Context correctness campaign preregistration draft

## Claim boundary

This document is a preregistration **draft** only. It does not register or
start B03, produce a Gate decision, or claim release, Profile, UCR-01 utility,
or general Agent benefit. No workload attempt has started and no denominator
exists.

## Fixed campaign identity

| Field | Draft value | Confirmation status |
|---|---|---|
| Task | `P3-T06` | fixed |
| Gate | `B03` | fixed; remains `not-run` |
| Campaign ID | `B03-personal-context-correctness-001` | proposed; owner confirmation required |
| Formal plan revision | `main@9fa5f127c25769c2d153c0214b48472c5c216305` | source revision to bind at registration |
| Implementation revision | `personal/P3-T06-context-correctness@22689de` | code revision to bind at registration |
| UCR-01 revision | `docs/evaluation/personal-unified-cognitive-resource-workload.md` | reference only; fixed-scenario utility remains P7-T08 scope |
| Operator | `TBD` | user/operator confirmation required |
| Independent verifier | `TBD` | independent identity and acceptance required |
| Registration authority | `TBD` | owner confirmation required |
| Execution authorization | `not granted` | must be explicit before execution |

## Qualified environment and reset

| Field | Required value | Status |
|---|---|---|
| Environment | `DEV-LINUX-NATIVE-01` (`personal-linux-native-01`) or another owner-approved native Linux environment | requires owner selection |
| OS/image | exact native Linux distribution/image, architecture, kernel, and reset procedure | must be captured before registration |
| Source checkout | clean disposable clone/worktree at the fixed implementation revision | required |
| Database state | fresh authority store or explicitly reset fixture state; schema/migration versions recorded | required |
| Context sources | fixed real source fixture with current, stale, unauthorized, and required-source cases | required |
| Artifact CAS | one daemon-owned digest-valid Artifact reference and bytes, with access policy recorded | required |
| Cache state | cold-start and any warm-cache state fixed before execution; cache rebuild policy recorded | required |
| Network | registered user-level proxy may be used only as an environment fact; endpoint and credentials stay out of evidence | record exact route without storing subscription material |
| Cleanup | stop processes, remove disposable checkout/fixture state, and redact evidence before retention | required |

## Required B03 assertions

Every assertion is evaluated from daemon/store authority facts and exact
evidence references. A client-provided report cannot satisfy an assertion.

1. **Real source:** the Context trace references real source rows and their
   current source/version/content digests.
2. **Scope before ranking:** tenant/governance scope filtering occurs before
   body loading and ranking; unauthorized candidates are not exposed.
3. **Required fail-closed:** missing or invalid required Context fails closed;
   it cannot silently degrade into a successful Task/Context result.
4. **Explicit loss:** excluded, stale, unauthorized, duplicate, or budget-
   rejected sources are represented by explicit reasoned loss facts.
5. **Artifact access:** authorized digest-valid Artifact bytes can be resolved;
   missing, tampered, or unauthorized Artifact references fail closed before
   verifier/report persistence.
6. **Revocation:** authorization/revocation facts are revalidated before body
   access; revoked sources and views cannot be served from a prior discovery.
7. **Stale cache:** cache reuse rechecks current governance and source facts;
   stale or revoked cache material is rejected and cannot be replayed.

## Failure-first matrix

The campaign must include at least one deterministic negative for each row and
retain the complete outcome, including rejected or unavailable cases:

| Negative | Expected authority outcome |
|---|---|
| scope mismatch after candidate discovery | reject before body/ranking exposure |
| required source missing | fail closed with explicit required-loss fact |
| stale source revision | reject stale body and preserve stale-loss evidence |
| revoked source/view | revalidate and reject before body access |
| stale/revoked cache entry | purge or bypass; never serve stale material |
| missing Artifact bytes | fail before verification report persistence |
| tampered Artifact bytes | digest mismatch; fail before report persistence |
| unauthorized Artifact access | reject before content access |
| malformed/authority-shaped evaluator input | reject without Gate mutation |
| false-completion field | reject and retain `claim_scope: non-claim` |

## Accounting and evidence

- Primary result: B03 correctness assertion matrix, with every required
  assertion independently marked `pass`, `fail`, or `not-run`.
- No assertion averaging: a missing required assertion cannot be hidden by
  passing assertions.
- Complete denominator: every started validation outcome, rejection, timeout,
  cleanup failure, and evidence-integrity failure is retained.
- Evidence must bind source revision, schema/migration versions, ContextView
  digest, source/version digests, Artifact digest, cache key facts, authority
  error code, and verifier evidence digest.
- Evidence must be redacted. It must not include Provider/user secrets, raw
  subscription URLs, raw source bodies where prohibited, or raw credential
  material.
- B06/B07 delta, stable-prefix, cache, and loop measurements may be collected
  as raw observations, but they are optional and cannot alter B03 status.

## Registration gates

Before changing this document from `draft` to `registered`, obtain:

1. owner approval of the campaign ID and exact formal-plan/implementation
   revisions;
2. named operator and independent verifier, with verifier independence
   recorded;
3. qualified native environment, reset, source/artifact/database/cache pins;
4. fixed assertion matrix, negative matrix, accounting and cleanup procedure;
5. explicit authorization to start the campaign;
6. a dedicated campaign execution lease that does not reuse B01's reserved
   desktop guest and does not write secrets or raw evidence into the repo.

## User/operator input required

Please provide or confirm these four items before formal registration:

- the operator identity;
- the independent verifier identity;
- the approved native Linux environment/reset procedure; and
- authorization to register and start `B03-personal-context-correctness-001`.

Until then, `P3-T06` remains `blocked` and `B03` remains `not-run`.
