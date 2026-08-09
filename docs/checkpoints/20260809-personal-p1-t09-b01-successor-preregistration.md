# P1-T09 B01 successor campaign preregistration

- Date: 2026-08-09
- Classification: `corrective`
- Task: `P1-T09`
- Gate: `B01`
- Campaign: `B01-clean-linux-first-install-first-conversation-002`
- Branch: `personal/P1-T09-b01-campaign-continuation`
- Lease: `lease/personal/P1-T09/b01-campaign-continuation`
- Owner authorization: retain failed campaign `001` and authorize this
  separately preregistered successor

## Separation from the failed campaign

Campaign `001` remains an immutable failed record: 10 started attempts, 2
successes, and 8 failures. It can reach at most 12 successes of 20 and cannot
satisfy the formal >=90% threshold. Its attempts, artifact, secret opt-ins,
and result do not transfer to this campaign.

Campaign `002` starts with zero attempts. It retains the same formal threshold:
fixed N=20, >=90% success, zero critical safety failures, complete aggregate
statistics, and affirmative independent-verifier disposition. No early pass or
optional stopping is permitted.

## Bound start gate

| Field | Preregistered successor value | Status before Attempt 1 |
|---|---|---|
| Environment | Sole formal guest `B01-Desktop-Linux-002`; restore `b01-platform-qualified-baseline`; native Ubuntu Desktop x86_64, user systemd, native Secret Service, and no retained Pi/CognitiveOS state | not run |
| Source revision | reviewed GitHub `main@4ea42c0c8f856aa22e2a360bd42005c8dbec400f` | fixed |
| Artifact workflow | `Personal experimental Linux campaign` run `31284948257` at the source revision, version `0.0.0-campaign.20260809.1` | running |
| Artifact binding | fresh workflow run, artifact manifest, GitHub-artifact SHA-256, bundle SHA-256, signature, trusted key/keyring, and release attestation must be independently verified before guest activation | not run |
| Pi | `@earendil-works/pi-coding-agent@0.81.1`; exact observed version and SRI `sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==` | not run |
| Runtime prerequisite | baseline Node.js `v22.23.2` and npm `10.9.8`; install Pi only through a user-local prefix | not run |
| Operator and verifier | Operator A performs only graphical hidden-input Provider opt-in; Verifier B independently evaluates redacted evidence | assigned roles; no credential supplied |
| Workload | one fixed non-sensitive expected-marker prompt, bounded by the reviewed route runner; do not retain prompt or response body | fixed |
| Cleanup | remove all successor installation/Pi/config/temp state, delete the operator secret through the approved SecretStore flow, revert the baseline, and confirm guest shut off | not run |

The Pi SRI row above must match the environment registry before execution; the
workflow artifact's independently verified facts are authoritative if they
disagree. This document intentionally contains no Provider credential, raw
Provider traffic, response body, SecretRef, or SQLite path.

## Artifact and local-staging boundary

The ignored local `/artifacts/` directory is P1-T09/B01 staging only. It is not
an input to this campaign, was not inspected for this preregistration, and must
not be committed. The workflow artifact must be downloaded afresh and its
manifest, digest, signature, trusted-key metadata, and attestation independently
verified. A stale artifact, expired retention record, or unverifiable bundle
fails the start gate without starting Attempt 1.

## Attempt accounting

Only the clean-reset checkpoint starts an attempt. Any readiness, setup,
installation, Pi, Provider, marker, timeout, cleanup, or evidence-integrity
failure after that point receives the next immutable attempt number. Before the
checkpoint, artifact retrieval and independent verification are start-gate work
and do not contribute to the denominator.

## Next action

Dispatch the preregistered protected workflow from the fixed reviewed-main
revision, then independently verify its output. Do not start the B01 guest or
Attempt 1 until every bound start-gate field is recorded as passing.

## Non-claims

This preregistration does not start the guest, create an attempt, install an
artifact or Pi, configure a Provider, access a credential, or claim B01, G1,
GMVP-LINUX, release, or Profile success.
