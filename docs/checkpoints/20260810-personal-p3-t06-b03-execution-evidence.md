<!--
Task: P3-T06
Slice: D02
Gate: B03
Campaign: B03-personal-context-correctness-001
Status: B03 MVP pass recorded under ADR-0040
-->

# B03 execution evidence — native Linux

## Bound execution

- Campaign: `B03-personal-context-correctness-001`
- Registered implementation revision: `53c2e43`
- Exact native checkout: `53c2e430e1de51b200d4b9eeb3204a00d1b431dc`
- Environment: owner-approved `DEV-LINUX-NATIVE-01` / `personal-linux-native-01`
- Environment attestation: Ubuntu 22.04.5 LTS, native `x86_64`, kernel
  `6.8.0-83-generic`; checkout was clean before cleanup
- Operator: Cursor agent
- Product owner: focused evidence affirmed; ADR-0040 defines this evidence as
  the MVP B03 denominator and decision boundary
- Secret and network boundary: no credential, subscription URL, raw provider
  traffic, or raw secret was included in the checkout or evidence

## Required correctness evidence

| Authority path | Result | Evidence |
|---|---:|---|
| Context store authorization, currentness, source discovery, request/view binding, and revocation | 9/9 | `cargo test -p cognitive-store --test m5_context_store` |
| Artifact CAS digest validation, immutable publication, and partial-write cleanup | 3/3 | `cargo test -p cognitive-store --test p3_t03_artifact_store` |
| Context builder scope-before-ranking, required fail-closed, revocation, loss, and stable rendering | 8/8 | `cargo test -p cognitive-kernel --test context_pipeline` |
| Context cache current-key and revocation/tool-drift negatives | 2/2 | `cargo test -p cognitive-kernel --lib context_cache` |
| B03 non-claim evaluator and failure-first rejection tests | 11/11 | `pnpm --filter @cognitiveos/repo-tools test` |

Primary correctness matrix result: **22/22 Rust focused tests passed** and
**11/11 evaluator/tooling tests passed**. The executed negatives cover scope
filtering before ranking, missing required Context, stale/revoked Context and
cache material, Artifact digest/access failures, explicit loss, and evaluator
authority-boundary rejection. The evaluator output remains non-claim and did
not mutate B03 or any Gate state.

## Supported checks

- `pnpm run check:consistency`: passed.
- `node --check tools/src/ucr-runner.mjs`: passed.
- Focused native Linux Clippy for the Context/Artifact test targets with
  `-D warnings`: passed.
- Required CI run `31346057300`: Ubuntu `pass`, Windows `pass`.
- Disposable checkout cleanup: passed; `/tmp/cognitiveos-p3-t06-d02-53c2e43`
  was removed after validation.
- Initial tooling invocation found missing disposable-clone dependencies
  (`ajv`, `yaml`) before test execution. `pnpm install --frozen-lockfile`
  restored the clean checkout dependencies; the subsequent 11/11 tooling run
  passed. This setup event is retained for complete accounting and is not a
  B03 correctness outcome.

## Product-owner MVP disposition

The user/owner affirmed the focused evidence matrix and its failure-first
results. ADR-0040 makes the 33 executed checks, native Linux/Clippy, required
CI, cleanup/redaction, and this owner review the P3-T06 MVP B03 decision
boundary. It does not authorize expanding the claim to B06/B07 benefit,
UCR-01 utility, release, Profile, or Task completion.

## Disposition boundary

The required CI run `31347323835` passed Ubuntu and Windows for the policy-
aligned revision `7ea39472899e8ac77f30e589da89b7b4e0b316a2`. Under ADR-0040,
the bounded B03 MVP result is **pass**. Normal PR, lease, branch, and main
reconciliation remain delivery closure work; this B03 pass does not pass
GMVP-LINUX, release, Profile, B06/B07, or UCR-01 utility.

## Non-claims

This evidence does not claim B06/B07 benefit, UCR-01 utility, GMVP-LINUX,
release, Profile conformance, Agent benefit, or Task completion.
