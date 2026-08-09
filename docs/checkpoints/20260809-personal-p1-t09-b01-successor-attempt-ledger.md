# P1-T09 B01 successor attempt ledger

- Date: 2026-08-09
- Classification: `corrective`
- Task: `P1-T09`
- Gate: `B01`
- Campaign: `B01-clean-linux-first-install-first-conversation-002`
- Lease: `lease/personal/P1-T09/b01-successor-native-verification`

## Fixed campaign boundary

This is a separately preregistered fixed-N=20 successor to failed campaign
`001`. Every operation after its clean-reset checkpoint receives an immutable
attempt number. Campaign `001` does not transfer attempts, artifacts,
credentials, or result. B01 requires all 20 attempts, >=90% success, zero
critical safety failures, and independent campaign disposition; this ledger
does not make a Gate, release, or Profile claim.

## Attempt records

| Attempt | Phase | Result | Redacted evidence |
|---|---|---|---|
| 1 | clean-reset checkpoint | pass | `B01-Desktop-Linux-002` restored from `b01-platform-qualified-baseline`; registered ProxyJump readiness confirmed native desktop user session, Node `v22.23.2`, and npm `10.9.8`. |
| 1 | artifact and installation | pass | Exact revision `4ea42c0c8f856aa22e2a360bd42005c8dbec400f`; independently verified signed `0.0.0-campaign.20260809.1`; guest bundle SHA-256 matched; canonical user service activated. |
| 1 | Pi and Provider readiness | pass | User-local exact Pi `0.81.1` with explicit extension; Operator A entered Provider credential only in graphical hidden input; `doctor` reported `overall=ready`, `first_conversation_ready=true`, native secret backend, and redacted secret reference. |
| 1 | bounded first response | **pass** | `{"status":"ok","phase":"first_response","duration_ms":5855,"expected_reply_observed":true,"response_received":true,"authority_side_effects":false}` |
| 1 | cleanup | **pass** | Provider secret cleared using product non-secret attributes; post-clear residual absent; user service disabled/stopped; Pi, bundle, config, and temporary scripts removed; baseline restored and guest confirmed `shut off`. |
| 2 | clean-reset checkpoint | pass | `B01-Desktop-Linux-002` restored from `b01-platform-qualified-baseline` and started through authorized libvirt control. |
| 2 | graphical Desktop readiness | **fail** | Registered ProxyJump SSH, Node `v22.23.2`, and npm `10.9.8` were available, but the user graphical session remained `inactive` after bounded readiness probes. No artifact, Pi, Provider credential, product service, route runner, request, response, or authority state was created. |
| 2 | cleanup | **pass** | Guest was stopped and baseline restored; domain confirmed `shut off`. |
| 3 | clean-reset and Desktop readiness | pass | Refreshed `b01-platform-qualified-baseline` restored; Operator A logged into `hal9001` through the VM graphical console; `graphical-session.target=active`, Node `v22.23.2`, npm `10.9.8`, and clean-state probes passed. |
| 3 | artifact, installation, Pi, and Provider readiness | pass | Independently verified signed artifact activated; user-local Pi `0.81.1` and explicit extension configured; Operator A entered Provider credential only through graphical hidden input; redacted `doctor` reported `overall=ready` and `first_conversation_ready=true`. |
| 3 | bounded first response | **pass** | `{"status":"ok","phase":"first_response","duration_ms":5518,"expected_reply_observed":true,"response_received":true,"authority_side_effects":false}` |
| 3 | cleanup | **pass** | Provider secret cleared through product non-secret attributes; post-clear residual absent; service stopped/disabled; Pi, bundle, config, and temporary files removed; baseline restored and guest confirmed `shut off`. |

## Aggregate

| Started | Successes | Failures | Critical safety failures | Remaining |
|---:|---:|---:|---:|---:|
| 3 | 2 | 1 | 0 | 17 |

## Non-claims

This successful individual attempt is experimental local evidence only. B01,
G1, GMVP-LINUX, release, and Profile remain unclaimed.
