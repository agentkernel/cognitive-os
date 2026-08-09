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

## Repeatable Desktop-session procedure

The sole formal guest restores from `b01-platform-qualified-baseline` and
starts shut off. The baseline does not persist an active graphical user session
across a libvirt snapshot restore. Therefore every attempt must use this fixed
operator procedure:

1. Restore the baseline and start the guest through the authorized system
   libvirt path.
2. The verifier confirms bounded SSH readiness and the registered Node/npm
   versions, but does not install the artifact or start product state.
3. Operator A logs into `hal9001` through the VM graphical console. The
   Provider credential is never supplied through SSH, argv, environment, files,
   chat, or logs.
4. The verifier waits only for `graphical-session.target=active`, confirms the
   clean-state probes, and then runs the fixed installation/Pi/doctor/route
   sequence.
5. On every outcome, fixed cleanup removes product/Pi/config/temp state,
   clears the Provider secret through the approved SecretStore flow, and
   restores the baseline with the guest shut off.

Failure of step 3 or 4 is an immutable attempt failure after the clean-reset
checkpoint; it is not retried under the same attempt number and does not permit
an alternate credential or non-graphical route.

## 2026-08-09 owner-approved campaign amendment

[ADR-0039](../adr/0039-personal-b01-six-attempt-campaign-policy.md) supersedes
the N=20 and >=90% policy above only for successor `002`. Its revised fixed
denominator is exactly six immutable outcomes, with at least five successes,
zero critical safety failures, complete aggregate statistics, and affirmative
independent-verifier disposition required for B01 pass. Attempts 1--6 remain
immutable; none is deleted, renumbered, or reclassified.

Attempt 7 crossed a reset checkpoint during the semantic-decision window but
created no artifact, Pi, Provider, service, route, request, response, or
authority state. The owner explicitly waived it from the revised denominator;
the waiver record remains auditable. No alternate credential route is allowed.

## Next action

Do not execute another attempt. Produce the revised aggregate and obtain
affirmative independent-verifier closure before changing B01 status.

## Non-claims

This preregistration does not start the guest, create an attempt, install an
artifact or Pi, configure a Provider, access a credential, or claim B01, G1,
GMVP-LINUX, release, or Profile success.
