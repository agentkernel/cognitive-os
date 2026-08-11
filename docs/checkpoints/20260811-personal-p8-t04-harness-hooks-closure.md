<!--
Task: P8-T04
Slice: D04
Classification: MVP task closure
Status: acceptance mapped; awaiting required CI, PR merge, and lease closure
-->

# P8-T04 Deterministic harness hooks and graded loading closure

## Acceptance mapping

| Acceptance item | Evidence |
|---|---|
| daemon-owned lifecycle interception points (admission/pre-dispatch/post-effect/verification) | D01 `register_harness_hook`; Linux `harness_hooks` 3/3 + Clippy at `3103a80`; axiom-relaxation and authority-writer negatives |
| owner-programmable deterministic hooks over digests with channel isolation | D02 `invoke_registered_harness_hook`; Linux `harness_hooks` 4/4 + Clippy at `169b303`; Task-channel and digest negatives |
| Skill/rule graded loading by context cost; fail-closed overflow | D03 `plan_graded_context_load`; Linux `graded_load` 2/2 + Clippy at `bc3dacd`; undeclared and overflow negatives |
| final acceptance / docs / PR / lease / branch closure | this checkpoint + Draft PR #188 |

## Non-claims

No Gate, release, Profile, GMVP-LINUX, Task/Effect completion, capability grant,
or axiom relaxation claim. Hooks remain observation-only.

## Remaining delivery actions

Mark Draft PR #188 ready after required CI for the closure HEAD, merge, close
`lease/personal/P8-T04/harness-hooks`, delete the task branch, and reconcile
local `main`. Then claim the next ready Personal task.
