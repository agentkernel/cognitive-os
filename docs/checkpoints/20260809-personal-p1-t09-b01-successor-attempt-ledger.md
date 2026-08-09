# P1-T09 B01 successor attempt ledger

- Date: 2026-08-09
- Classification: owner-approved `product-semantic`
- Task: `P1-T09`
- Gate: `B01`
- Campaign: `B01-clean-linux-first-install-first-conversation-002`
- Lease: `lease/personal/P1-T09/b01-successor-native-verification`

## Fixed campaign boundary

This was separately preregistered as a fixed-N=20 successor to failed campaign
`001`. Every operation after its clean-reset checkpoint receives an immutable
attempt number. Campaign `001` does not transfer attempts, artifacts,
credentials, or result. The owner-approved ADR-0039 amendment supersedes that
denominator only for successor `002`: it requires fixed counted outcomes 1--6,
at least 5 successes, zero critical safety failures, complete aggregate
statistics, and affirmative independent-verifier disposition. This ledger does
not make a Gate, release, or Profile claim.

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
| 4 | clean-reset and Desktop readiness | pass | Baseline restored; Operator A logged into `hal9001` through the VM graphical console; `graphical-session.target=active`, Node `v22.23.2`, npm `10.9.8`, and clean-state probes passed. |
| 4 | artifact, installation, Pi, and Provider readiness | pass | Independently verified signed artifact activated; user-local Pi `0.81.1` and extension configured; graphical hidden-input Provider opt-in completed; redacted `doctor` reported `overall=ready` and `first_conversation_ready=true`. |
| 4 | bounded first response | **pass** | `{"status":"ok","phase":"first_response","duration_ms":6315,"expected_reply_observed":true,"response_received":true,"authority_side_effects":false}` |
| 4 | cleanup | **pass** | Provider secret cleared and residual absent; service stopped/disabled; Pi, bundle, config, and temporary files removed; baseline restored and guest confirmed `shut off`. |
| 5 | clean-reset and Desktop readiness | pass | Baseline restored; Operator A logged into `hal9001` through the VM graphical console; `graphical-session.target=active`, Node `v22.23.2`, npm `10.9.8`, and clean-state probes passed. |
| 5 | artifact, installation, Pi, and Provider readiness | pass | Independently verified signed artifact activated; user-local Pi `0.81.1` and extension configured; graphical hidden-input Provider opt-in completed; redacted `doctor` reported `overall=ready` and `first_conversation_ready=true`. |
| 5 | bounded first response | **pass** | `{"status":"ok","phase":"first_response","duration_ms":5409,"expected_reply_observed":true,"response_received":true,"authority_side_effects":false}` |
| 5 | cleanup | **pass** | Provider secret cleared and residual absent; service stopped/disabled; Pi, bundle, config, and temporary files removed; baseline restored and guest confirmed `shut off`. |
| 6 | clean-reset and Desktop readiness | pass | Baseline restored; Operator A logged into `hal9001` through the VM graphical console; `graphical-session.target=active`, Node `v22.23.2`, npm `10.9.8`, and clean-state probes passed. |
| 6 | artifact, installation, Pi, and Provider readiness | pass | Independently verified signed artifact activated; user-local Pi `0.81.1` and extension configured; graphical hidden-input Provider opt-in completed; redacted `doctor` reported `overall=ready` and `first_conversation_ready=true`. |
| 6 | bounded first response | **pass** | `{"status":"ok","phase":"first_response","duration_ms":5473,"expected_reply_observed":true,"response_received":true,"authority_side_effects":false}` |
| 6 | cleanup | **pass** | Provider secret cleared and residual absent; service stopped/disabled; Pi, bundle, config, and temporary files removed; baseline restored and guest confirmed `shut off`. |
| 7 | transition clean-reset checkpoint | **waived** | Reset succeeded during the ADR-0039 decision window. Before graphical login, artifact, Pi, Provider, service, route, request, response, or authority activity, the guest was reverted and confirmed `shut off`. The owner explicitly waived this retained transition record from the revised N=6 denominator. |

## Aggregate

| Started | Successes | Failures | Critical safety failures | Remaining |
|---:|---:|---:|---:|---:|
| 6 | 5 | 1 | 0 | 0 |

Attempt 7 is not an unrecorded outcome: it is retained above as an owner-waived
transition event. It is excluded from this aggregate because it did not begin a
product execution under the revised policy.

## Non-claims

The revised numerical criterion is `5/6 = 83.33%`. B01 remains `running`
pending the aggregate report and affirmative independent-verifier disposition;
G1, GMVP-LINUX, release, and Profile remain unclaimed.
