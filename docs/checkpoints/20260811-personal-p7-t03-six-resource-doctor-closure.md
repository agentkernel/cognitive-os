<!--
Task: P7-T03
Slice: D04
Classification: MVP task closure
Status: acceptance mapped; awaiting required CI, PR merge, and lease closure
-->

# P7-T03 six-resource doctor / vault / operability closure

## Acceptance mapping

| Acceptance item | Evidence |
|---|---|
| redacted six-resource doctor health with stable error codes | D01 `evaluate_six_resource_doctor_health`; Linux 4/4 + Clippy at `13e46eb`; wired into `/personal/doctor` |
| headless vault locked/TTY/unattended paths remain redacted and secret-free | D02 `evaluate_headless_vault_doctor`; Linux 3/3 + Clippy at `9dc2dcd`; unit/env/argv contamination negatives |
| sidecar drift, process/effect reconcile, migration doctor facts with recovery hints | D03 `evaluate_operability_doctor`; Linux 3/3 + Clippy at `749a0c3`; authority-claim negatives |
| final acceptance / docs / PR / lease / branch closure | this checkpoint + Draft PR #186 |

## Non-claims

No Gate, release, Profile, GMVP-LINUX, live vault unlock, Secret Store mutation,
Task/Effect completion, or SidecarSession mutation claim.

## Remaining delivery actions

Mark Draft PR #186 ready after required CI for the closure HEAD, merge, close
`lease/personal/P7-T03/six-resource-doctor`, delete the task branch, and
reconcile local `main`. Then claim the next ready Personal task.
