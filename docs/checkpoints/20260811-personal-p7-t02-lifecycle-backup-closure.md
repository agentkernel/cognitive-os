<!--
Task: P7-T02
Slice: D04
Classification: MVP task closure
Status: acceptance mapped; awaiting required CI, PR merge, and lease closure
-->

# P7-T02 transactional lifecycle and Memory/Skill backup/restore closure

## Acceptance mapping

| Acceptance item | Evidence |
|---|---|
| secret-excluding Personal backup inventory | D01 `plan_personal_backup_inventory` / `validate_backup_inventory`; Linux at `8750666`; CI `31430152517` |
| Memory/Skill/bindings export binds digests without secret material over D01 categories | D02 `plan_memory_skill_export`; Linux 7/7 + Clippy at `6b6d245` |
| restore preflight rejects incompatible/incomplete backups before mutation | D03 `preflight_personal_backup_restore`; schema/incomplete/migration/digest negatives |
| transactional update/rollback/uninstall over D01–D03 evidence | D04 `plan_personal_lifecycle` / `commit_personal_lifecycle` / `abort_personal_lifecycle`; secret and unconfirmed-data uninstall refused |
| focused negatives + exact native Linux + Clippy | `personal_backup` 15/15 + Clippy at `68abc82` |
| final acceptance / docs / PR / lease / branch closure | this checkpoint + Draft PR #185 |

## Non-claims

No Gate, release, Profile, or GMVP-LINUX claim. No host filesystem archive write,
OS-level uninstall execution, or live Secret Store deletion is claimed. D04
authority-path staging/commit does not perform destructive host I/O.

## Remaining delivery actions

Mark Draft PR #185 ready after required CI for the closure HEAD, merge, delete
the task branch, and reconcile local `main`. Then claim `P7-T03`.
