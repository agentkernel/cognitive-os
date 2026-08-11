<!--
Task: P8-T06
Slice: D04
Classification: MVP task closure
Status: in-progress pending required CI
-->

# P8-T06 Cross-episode learning loop closure

## Acceptance mapping

| Acceptance item | Evidence |
|---|---|
| Reflexion-family failure experience → digest-bound Memory candidate without self-authorization | D01 `plan_failure_lesson_memory_candidate`; Linux at `8ba3fe0`; self-auth / direct-promotion / missing-identity negatives |
| Admitted Memory candidates only through existing daemon Memory admission; forget explainable and non-resurrecting | D02 `decide_learning_memory_admission` + `plan_learning_memory_forget`; Linux 4/4 + Clippy at `31e7384`; forged-admit and source-mismatch negatives |
| Skill candidate import/bind/revoke path without capability grant | D03 `plan_failure_lesson_skill_candidate` + `plan_learning_skill_binding_revoke`; Linux 5/5 + Clippy at `b81414d`; capability-grant negatives |
| final acceptance / docs / PR / lease / branch closure | this checkpoint; required CI pending on Draft PR #190 HEAD `b81414d` |

## Non-claims

No Gate, release, Profile, GMVP-LINUX, public Memory/Skill API expansion,
capability grant, or self-authorization claim. Learning remains candidate→admission
only.

## Closure

Required Ubuntu/Windows CI for HEAD will be recorded when green. PR #190 is the
task closure PR. Lease `lease/personal/P8-T06/learning-loop` closes with merge;
task branch deleted after merge; local `main` reconciled.
