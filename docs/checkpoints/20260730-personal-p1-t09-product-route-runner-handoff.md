# P1-T09 redacted product-route runner handoff

- Date: 2026-07-30
- Task: P1-T09 install-to-first-conversation route
- Closed lease: `lease/personal/P1-T09/redacted-product-route-runner`
- Branch: `lane/personal-p1-t09-coherent-bundle-delivery`
- Change class: implementation-only; normative surface unchanged
- Development track: `experimental-local-only`

## Delivered

`tools/personal/p1-t09-product-route-smoke.sh` is a reproducible runner for a
future installed product bundle. It accepts only absolute regular-file paths
for the product `cognitive` client, exact Pi executable, and deployed
Extension entry. It then:

1. invokes non-secret `cognitive pi configure`;
2. parses only the doctor status/overall/first-conversation readiness fields;
3. verifies exact Pi `0.81.1` through a cleared allowlist environment;
4. performs one bounded `--extension <absolute-path>` Pi print attempt; and
5. emits only a redacted structured result: status, phase, error class,
   duration, response/expected-marker booleans, and
   `authority_side_effects: false`.

The runner writes all CLI, doctor, version, and Pi output to a private
temporary directory and deletes it on exit. It does not print or accept
Provider configuration, SecretRefs, keys, tokens, SQLite paths, model
identifiers, selected-model digests, or response contents.

## Executed checks

| Check | Result |
|---|---|
| `bash -n tools/personal/p1-t09-product-route-smoke.sh` | pass |
| `bash -n tools/personal/p1-t09-product-route-smoke.test.sh` | pass |
| `bash tools/personal/p1-t09-product-route-smoke.test.sh` | pass; missing path, relative path, invalid timeout, unsafe marker, successful exact-marker fixture, and child-environment secret exclusion covered |
| Same syntax and fixture command on `personal-linux-native-01` | pass; non-secret fixture only |
| Real installed-product runner invocation | not-run; no signed/deployed coherent product bundle |
| `cognitive pi launch` product invocation | not-run; no signed/deployed coherent product bundle |
| B01 / GMVP-LINUX / release / Profile | not-run or non-claim |

## Status and blocker

P1-T09 stays `in-progress`. The runner closes the reproducibility and focused
negative-coverage part of the route acceptance; it does not demonstrate a
first conversation or native Secret Service route smoke.

`blocked_paths`: coherent product bundle deployment paths on the experimental
host. `blocked_task_ids`: `P1-T09`. `blocked_gate_ids`: `B01`, `GMVP-LINUX`,
and Profile. Owner: authorized campaign signing-material and deployment
workflow owner. Next action: authorize that protected workflow, sign and
offline-verify the coherent experimental artifact, deploy it through the
verified installer, then claim a separate route-validation lease and run this
script against the installed paths.
