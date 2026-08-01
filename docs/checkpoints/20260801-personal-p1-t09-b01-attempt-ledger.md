# P1-T09 B01 attempt ledger

- Date: 2026-08-01
- Task: P1-T09 install-to-first-conversation route
- Campaign: `B01-clean-linux-first-install-first-conversation-001`
- Gate: B01 first-install/first-conversation
- Lease: `lease/personal/P1-T09/b01-execution`
- Change class: implementation-only (documentation closure for attempt records)

## Fixed attempt contract (from preregistration)

- Every invocation after the clean-reset checkpoint is an attempt. A timeout,
  nonzero exit, missing marker, readiness failure, setup fault, or cleanup
  failure is recorded as a failure; retries receive a new attempt number.
- Success threshold: one started attempt, one expected marker, one bounded
  response, zero authority side effects, and no unredacted secret/internal
  material; no retries are permitted for a passing B01 claim.
- Redaction: retain only attempt number, phase, fixed error class, exit
  category, bounded duration, boolean marker/response/authority fields,
  artifact identity, and cleanup result.

## Pre-attempt facts

| Field | Value |
|---|---|
| Environment | `B01-Desktop-Linux-002`; Ubuntu Desktop 24.04.4; x86_64; non-WSL; native user-systemd running |
| Guest user | `hal9001`; operator-held desktop login password is the encrypted keyring master |
| Reset snapshot | `b01-platform-qualified-baseline` (taken after first login, keyring creation, and SSH provisioning) |
| Artifact | `0.0.0-campaign.20260801.1` from `main@0a5524bc7a867090e71f24d65b3faf3657b32d44`; run `30687541828` |
| Artifact digest | artifact `sha256:80e6a4d0d633b34e949fce92afb8b8fcfc4ae6dca6c4fd244888540a777a3394` |
| Signature | Ed25519; key `p1-t09-experimental`; keyring `p1-t09-experimental-20260730`; public key `Uui0QQibM4z49Md4N55ANrkpli_12IMpn_W8rmB5vdk` |
| Independent verification | host locked verifier accepted the bundle, signature, key, and expected Pi `0.81.1` pin |
| Pi pin | `@earendil-works/pi-coding-agent@0.81.1`; integrity `sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==` |
| Secret Service | native default/login collection probed with non-sensitive sentinel; store/lookup/clear passed |
| Workload | one fixed non-sensitive prompt requesting the configured non-secret expected marker; text not retained |
| Route runner | reviewed `tools/personal/p1-t09-product-route-smoke.sh` with absolute installed paths, explicit `--extension`, bounded timeout, closed stdin |
| Operator A | `hal9001` owner performs hidden-input Provider credential opt-in only |
| Verifier B | independent verifier reviews redacted evidence; must not receive the credential |

## Start-gate checklist

| # | Item | Status |
|---|---|---|
| 1 | Owner allocated a new clean VM and provided only its non-secret access endpoint | pass |
| 2 | Linux x86_64, non-WSL, native user-systemd, native Secret Service, clean state, disposable directory | pass |
| 3 | Reviewed `main` artifact selected; version, source commit, SHA-256, signature, trusted key, Pi pin independently verified | pass |
| 4 | Campaign names operator and independent verifier | pass |
| 5 | Clean/reset snapshot, workload, timeout, attempt ledger location, redacted evidence collector, cleanup procedure recorded | pass |
| 6 | B01 runner reviewed as a formal Gate runner, not reusing the experimental route runner result or host | pass |

## Attempt records

| Attempt | Phase | Result | Evidence |
|---|---|---|---|
| 1 | clean-reset checkpoint | pass | snapshot `b01-platform-qualified-baseline`; clean state, native user-systemd, Secret Service probe passed |
| 1 | immutable artifact verification | pass | `0.0.0-campaign.20260801.1` from `main@0a5524b`; SHA-256 `80e6a4d0...`; Ed25519 signature accepted by locked host verifier |
| 1 | exact Pi installation | pass | `@earendil-works/pi-coding-agent@0.81.1`; registry SRI matched campaign `pi_integrity` |
| 1 | verified installer activation | pass | `installed-cognitiveos-personal version=0.0.0-campaign.20260801.1 previous=none service=cognitiveos-personal.service port=48181` |
| 1 | readiness | pass | `overall=ready`; `first_conversation_ready=true`; all components ready |
| 1 | Operator A credential opt-in | pass | hidden-input `read -s` through `cognitive init`; no credential entered argv, chat, config, logs, or evidence |
| 1 | bounded first response | **pass** | `{"status":"ok","phase":"first_response","duration_ms":6295,"expected_reply_observed":true,"response_received":true,"authority_side_effects":false}` |
| 1 | cleanup | **pass** | service stopped/disabled; deployment, Pi runtime, campaign copies, temp files removed; operator-entered secret deleted through product-equivalent SecretStore delete (`secret-tool clear` with product `provider_secret_attributes`); post-clear not-found verified (`secret-residual=absent`) |

## Attempt 1 result

`B01-clean-linux-first-install-first-conversation-001` attempt 1 satisfies the
success threshold: one started attempt, one expected marker, one bounded
response (6295 ms), zero authority side effects, and no unredacted
secret/internal material. The redacted route output above is the retained
evidence record.

## Checks for this slice

| Check | Result |
|---|---|
| `pnpm run check:consistency` | not-run until documentation closure |
| `git diff --check` | not-run until documentation closure |
