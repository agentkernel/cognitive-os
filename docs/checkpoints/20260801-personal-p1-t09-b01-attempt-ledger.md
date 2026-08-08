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
| 2 | clean-reset checkpoint | pass | exact `b01-platform-qualified-baseline` restored through authorized system-libvirt control; guest started only for readiness verification |
| 2 | guest network readiness | **fail** | bounded SSH readiness probes to the preregistered guest address timed out; no artifact, Pi, product service, Provider, credential, prompt, or route runner was used |
| 2 | cleanup | **pass** | exact baseline restored again; domain confirmed `shut off`; no post-baseline state retained |
| 3 | clean-reset checkpoint | pass | exact `b01-platform-qualified-baseline` restored and guest started through authorized system-libvirt control |
| 3 | guest network readiness | **fail** | four bounded non-interactive SSH probes reached the guest SSH service but could not authenticate with the available public-key path; no artifact, Pi, product service, Provider, credential, prompt, or route runner was used |
| 3 | cleanup | **pass** | exact baseline restored again; domain confirmed `shut off`; no post-baseline state retained |
| 4 | clean-reset checkpoint | pass | exact repaired `b01-platform-qualified-baseline` restored and guest started through authorized system-libvirt control |
| 4 | guest SSH readiness | pass | bounded non-interactive local-key ProxyJump probe reached `hal9001@192.168.123.160` successfully |
| 4 | immutable artifact availability | **fail** | the preregistered artifact `0.0.0-campaign.20260801.1` is expired from workflow run `30687541828`; the only host-local candidate was an older, non-registered campaign and was not used |
| 4 | cleanup | **pass** | exact baseline restored; domain confirmed `shut off`; no artifact, Pi, product service, Provider, credential, prompt, route runner, request, response, or authority state was created |
| 5 | clean-reset checkpoint | pass | exact repaired `b01-platform-qualified-baseline` restored and guest started through authorized system-libvirt control |
| 5 | guest SSH readiness | pass | bounded non-interactive local-key ProxyJump probe reached `hal9001@192.168.123.160` successfully |
| 5 | staged bundle digest check | **fail** | an automation-shell quoting defect made the preregistered digest-check command invalid before installer activation; no artifact was installed or executed |
| 5 | cleanup | **pass** | exact baseline restored; domain confirmed `shut off`; no Pi, product service, Provider, credential, prompt, route runner, request, response, or authority state was created |
| 6 | clean-reset checkpoint | pass | exact repaired `b01-platform-qualified-baseline` restored and guest started through authorized system-libvirt control |
| 6 | guest SSH readiness | pass | bounded non-interactive local-key ProxyJump probe reached `hal9001@192.168.123.160` successfully |
| 6 | immutable installation and activation | pass | `0.0.0-campaign.20260808.1` installer accepted the staged signature/bundle/Pi pin and activated `cognitiveos-personal.service` on port 48181 |
| 6 | user-service readiness | **fail** | the guest had no persistent Desktop login session; the SSH-created user manager exited after the remote session ended and the user service became inactive before Provider configuration |
| 6 | cleanup | **pass** | exact baseline restored; domain confirmed `shut off`; no Provider credential, prompt, route runner, request, response, or authority state was created |
| 7 | clean-reset checkpoint | pass | exact repaired `b01-platform-qualified-baseline` restored, normal `hal9001` Desktop session established, and guest started through authorized system-libvirt control |
| 7 | immutable installation and user-service readiness | pass | `0.0.0-campaign.20260808.1` installer activated successfully; `cognitiveos-personal.service` remained active after a bounded post-install interval |
| 7 | exact Pi prerequisite | **fail** | the clean baseline had neither `node` nor `npm`, so the preregistered exact Pi installation could not begin; no Pi package was downloaded or installed |
| 7 | cleanup | **pass** | exact baseline restored; domain confirmed `shut off`; no Provider credential, prompt, route runner, request, response, or authority state was created |
| 8 | clean-reset checkpoint | pass | repaired baseline restored with Node.js `v18.19.1`, normal Desktop session established, and guest started through authorized system-libvirt control |
| 8 | immutable installation and user-service readiness | pass | `0.0.0-campaign.20260808.1` installer activated successfully; `cognitiveos-personal.service` remained active after a bounded post-install interval |
| 8 | exact Pi installation | **fail** | Pi `0.81.1` requires Node.js `>=22.19.0`; baseline Node.js `v18.19.1` did not meet the package engine requirement and no Pi executable was produced |
| 8 | cleanup | **pass** | guest shut off and reverted to exact baseline; no Provider credential, prompt, route runner, request, response, or authority state was created |
| 9 | clean-reset checkpoint | pass | repaired baseline restored with Node.js `v22.23.2`, normal Desktop session established, and guest started through authorized system-libvirt control |
| 9 | immutable installation and user-service readiness | pass | `0.0.0-campaign.20260808.1` installer activated successfully; `cognitiveos-personal.service` remained active after a bounded post-install interval |
| 9 | exact Pi installation | **fail** | a global npm installation targeted the root-owned `/opt` Node.js prefix and was denied write access; no Pi executable was produced |
| 9 | cleanup | **pass** | guest shut off and reverted to exact baseline; no Provider credential, prompt, route runner, request, response, or authority state was created |
| 10 | clean-reset checkpoint | pass | repaired baseline restored with Node.js `v22.23.2`, normal Desktop session established, and guest started through authorized system-libvirt control |
| 10 | immutable installation and user-service readiness | pass | `0.0.0-campaign.20260808.1` installer activated successfully; `cognitiveos-personal.service` remained active after a bounded post-install interval |
| 10 | exact Pi installation | pass | user-local npm prefix installed `@earendil-works/pi-coding-agent@0.81.1`; observed Pi version `0.81.1`; registry SRI matched the preregistered pin |
| 10 | Provider configuration | pass | product `cognitive init` configured DeepSeek official API values and accepted Operator A's hidden-input key; secret material remained in native Secret Service and was not retained in evidence |
| 10 | readiness | pass | `cognitive doctor` reported `overall=ready` and `first_conversation_ready=true`; native Secret Service probe and daemon readiness passed |
| 10 | bounded first response | **pass** | `{"status":"ok","phase":"first_response","duration_ms":5272,"expected_reply_observed":true,"response_received":true,"authority_side_effects":false}` |
| 10 | cleanup | **pass** | route runner and temporary files were removed; guest was shut off and reverted to exact `b01-platform-qualified-baseline`; no retained Provider request, response, authority side effect, Task, Effect, or Verification state |

## Attempt 5 preregistration

| Field | Registered value |
|---|---|
| Owner authorization | complete fresh counted Attempt 5 approved after replacement-artifact verification |
| Artifact | `0.0.0-campaign.20260808.1` from `main@bde3e3cab94063705e46d4a9e72db15feb631cbc`; workflow run `31253008076` |
| GitHub artifact digest | `sha256:698a8e2baa8e5833134177d3396776a3dc7bbcf068c81e98cf36c439a2a3b659` |
| Bundle digest | `sha256:c0795081860311cf537fb92af388ee94519933ef28149faf1abc90449223cf22` |
| Independent verification | exact `main@bde3e3c` locked Linux verifier accepted the staged bundle, signature, `p1-t09-experimental` key, `p1-t09-experimental-20260730` keyring, and Pi `0.81.1` pin |
| Provider configuration | `deepseek`; `https://api.deepseek.com`; model ID `deepseek-v4-flash`; all values are non-secret and must be supplied only through product configuration |
| Operator action | Operator A enters the DeepSeek API key only in the Desktop product hidden-input prompt; no key is entered in chat, command arguments, ordinary files, logs, or evidence |

## Attempt 5 result

`B01-clean-linux-first-install-first-conversation-001` attempt 5 reached the
clean-reset checkpoint and passed bounded SSH readiness. Before installer
activation, the runner's staged-bundle digest command failed due to shell
quoting. The independently verified artifact was not executed or installed.
No Pi state, product service, Provider credential, Provider request, prompt,
expected marker, response, authority side effect, Task, Effect, or Verification
was created. Cleanup restored the exact baseline and left the domain shut off.
This is a recorded fixed-N setup failure, not a retry or critical-safety
failure. The next attempt must use the already verified artifact and a
prevalidated non-shell-interpolated digest procedure.

## Attempt 6 result

`B01-clean-linux-first-install-first-conversation-001` attempt 6 reached the
clean-reset checkpoint, passed bounded SSH readiness, and completed immutable
installer activation for `0.0.0-campaign.20260808.1`. The user service became
inactive after the SSH-created user manager exited because the Desktop user had
not logged in graphically. The service therefore could not satisfy readiness
before Provider configuration. No Provider credential, Provider request,
prompt, expected marker, response, authority side effect, Task, Effect, or
Verification was created. Cleanup restored the exact baseline and left the
domain shut off. This is a recorded fixed-N readiness failure, not a retry or
critical-safety failure. A fresh attempt must establish the normal Desktop
login session before installation and hidden-input credential opt-in.

## Attempt 7 result

`B01-clean-linux-first-install-first-conversation-001` attempt 7 reached the
clean-reset checkpoint, established the normal Desktop session, and retained
the installed user service through bounded readiness. The clean baseline had no
Node.js or npm runtime, so the preregistered exact Pi installation could not
begin. No Pi package, Provider credential, Provider request, prompt, expected
marker, response, authority side effect, Task, Effect, or Verification was
created. Cleanup restored the exact baseline and left the domain shut off. This
is a recorded fixed-N prerequisite failure, not a retry or critical-safety
failure. Owner-authorized baseline maintenance may install only Node.js/npm
before a fresh attempt is requested.

## Attempt 8 preregistration

| Field | Registered value |
|---|---|
| Owner authorization | continuous execution authorization received; fresh counted Attempt 8 may proceed without a separate per-attempt confirmation |
| Baseline | repaired `b01-platform-qualified-baseline` with Node.js `v18.19.1` and npm `9.2.0`; no Pi or CognitiveOS state present; guest shut off before restore |
| Artifact | `0.0.0-campaign.20260808.1` from `main@bde3e3cab94063705e46d4a9e72db15feb631cbc`; workflow run `31253008076` |
| Bundle digest | `sha256:c0795081860311cf537fb92af388ee94519933ef28149faf1abc90449223cf22` |
| Independent verification | exact `main@bde3e3c` locked Linux verifier accepted the staged bundle, signature, trusted key, and Pi `0.81.1` pin |
| Pi pin | `@earendil-works/pi-coding-agent@0.81.1`; integrity `sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==` |
| Provider configuration | `deepseek`; `https://api.deepseek.com`; model ID `deepseek-v4-flash`; non-secret values only |
| Operator action | Operator A enters the DeepSeek API key only in the graphical product hidden-input prompt; no key enters chat, argv, ordinary files, logs, or evidence |

## Attempt 8 execution status

Attempt 8 is registered and begins only after the repaired baseline is restored
and the guest is started. The normal `hal9001` graphical Desktop session must
remain active through installation, exact Pi setup, hidden-input Provider
configuration, route execution, and cleanup.

## Attempt 1 result

`B01-clean-linux-first-install-first-conversation-001` attempt 1 satisfies the
success threshold: one started attempt, one expected marker, one bounded
response (6295 ms), zero authority side effects, and no unredacted
secret/internal material. The redacted route output above is the retained
evidence record.

## Attempt 2 result

`B01-clean-linux-first-install-first-conversation-001` attempt 2 failed during
bounded guest network readiness after the clean-reset checkpoint. The
preregistered guest address did not accept SSH before the bounded readiness
probes elapsed. No installation, Pi state, Provider credential, Provider
request, prompt, expected marker, response, authority side effect, Task,
Effect, or Verification was created. Cleanup restored the exact baseline and
left the domain shut off. This is a recorded failed attempt in the fixed
N=20 denominator, not a retry or a critical-safety failure.

## Attempt 3 result

`B01-clean-linux-first-install-first-conversation-001` attempt 3 reached the
clean-reset checkpoint when the authorized system-libvirt host restored the
exact Desktop baseline and started the guest. Four bounded non-interactive SSH
probes reached the guest SSH service but failed public-key authentication. The
available automation identity cannot complete the preregistered SSH readiness
path, so this is a recorded fixed-N readiness failure rather than a retry.
No artifact, Pi state, product service, Provider credential, Provider request,
prompt, expected marker, response, authority side effect, Task, Effect, or
Verification was created. Cleanup restored the exact baseline and left the
domain shut off. This is not a critical-safety failure.

## Checks for this slice

| Check | Result |
|---|---|
| `pnpm run check:consistency` | pass |
| `git diff --check` | pass |

## Attempt 8 result

`B01-clean-linux-first-install-first-conversation-001` attempt 8 reached the
clean-reset checkpoint, established the normal `hal9001` graphical Desktop
session, and activated the independently verified artifact. The user service
remained active after the bounded readiness interval. Exact Pi installation
then failed because the repaired baseline provides Node.js `v18.19.1`, while
the preregistered `@earendil-works/pi-coding-agent@0.81.1` package requires
Node.js `>=22.19.0`; the bounded npm probe also could not produce the `pi`
executable. No Provider credential, prompt, request, response, authority side
effect, Task, Effect, or Verification was created. The guest was shut down
and reverted to the exact `b01-platform-qualified-baseline` snapshot. This is
a recorded fixed-N Pi prerequisite failure, not a retry or critical-safety
failure. The next recovery step is baseline maintenance to provide the exact
supported Node.js runtime, followed by a new counted attempt.

## Attempt 9 preregistration

| Field | Registered value |
|---|---|
| Owner authorization | continuous execution authorization remains in effect; fresh counted Attempt 9 may proceed |
| Baseline | `b01-platform-qualified-baseline` with Node.js `v22.23.2`, npm `10.9.8`, no Pi state, and no CognitiveOS state |
| Artifact | `0.0.0-campaign.20260808.1` from `main@bde3e3cab94063705e46d4a9e72db15feb631cbc`; workflow run `31253008076` |
| Bundle verification | exact locked Linux verifier previously accepted the recorded bundle digest, signature, trusted key, and Pi `0.81.1` pin |
| Pi pin | `@earendil-works/pi-coding-agent@0.81.1`; integrity `sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==` |
| Provider configuration | `deepseek`; `https://api.deepseek.com`; model ID `deepseek-v4-flash`; non-secret values only |
| Operator action | Operator A enters the DeepSeek API key only in the graphical product hidden-input prompt; no key enters chat, argv, ordinary files, logs, or evidence |

## Attempt 10 result

`B01-clean-linux-first-conversation-001` attempt 10 completed the repaired
install-to-first-conversation route. The normal Desktop session and stable
user service passed, exact Pi `0.81.1` installed under a user-local prefix,
DeepSeek Provider configuration completed through the product hidden-input
flow, `cognitive doctor` reported first-conversation readiness, and the
bounded route returned the expected marker in 5272 ms. The retained route
record reported `authority_side_effects=false`. Cleanup stopped the service,
removed temporary route files, shut off the guest, and reverted the exact
baseline. This is the first successful counted attempt since the repaired
runtime baseline; it is not a retry.
