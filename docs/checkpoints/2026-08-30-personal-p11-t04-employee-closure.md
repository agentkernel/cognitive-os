# P11-T04 Role Blueprint / Assignment / Employee closure

- Task: `P11-T04` / slice `P11-T04/D01`
- Change class: `implementation-only` (Personal-private tables + private projection; no `core/specs`, no Lane-CTR)
- Branch: `personal/P11-T04-employee`
- Implementation revision: `3c7a419080f8e5702a99f5b8746b8f7415a47aa0`
- Clippy `HandoffSpec`: `9cb1b6dcf15964b3107f4e4baf4aca17decfa3c4`
- Closure HEAD: `62840c9b076c4095817660575d3392a3c43e2d69`
- Pull request: [#282](https://github.com/agentkernel/cognitive-os/pull/282) **Draft** (parent flips ready/merge; this checkpoint does not)
- Lease: `lease/personal/P11-T04/employee` (stays active until parent merge/lease close)
- Required CI on `62840c9b`: run [33292431146](https://github.com/agentkernel/cognitive-os/actions/runs/33292431146) **SUCCESS** (resolve 3s, Ubuntu 3m37s including Rust workspace, Windows 11m9s, `required-ci`)
- Claim ceiling: `hypothesis`
- Evaluation routing: **OFF**

## Acceptance mapping

| Acceptance item | Evidence |
|---|---|
| Employee is the authoritative id; chrome may say Member Runtime | v27 `p11_employee` + roster/seat/catalog routes. Linux store **16/16** at `9cb1b6dc`; required CI workspace tests **pass** at `62840c9b` |
| employee≠runtime; runtime is replaceable and does not merge identity | `p11_t04_employee_survives_process_death` (store observer must not mutate Employee); `runtime_binding_ref` is an opaque adapter-identity string. Real DSH process bind = T07 **not-run** |
| Blueprint has no Provider binding | `p11_t04_blueprint_has_no_provider_binding` **pass** |
| Blueprint upgrade is versioned + per-Project opt-in | `p11_t04_blueprint_upgrade_is_opt_in` **pass** |
| Roster derived from confirmed PlanRevision `responsible_slot` (full cover; surplus refused) | `p11_t04_roster_must_cover_all_slots` **pass** |
| Sequential seating daemon-guarded (at most one seating per Project) | `p11_t04_sequential_seating_enforced` **pass** |
| Seating progress reads committed seated facts only | `p11_t04_progress_reads_committed_facts_only` **pass** |
| One current manager per active Project; removed manager keeps history | `p11_t04_one_current_manager` + `p11_t04_removed_manager_keeps_history` **pass** |
| Employee not shared across Projects | `p11_t04_employee_not_shared_across_projects` **pass** |
| Recipe ≠ authorization (InstallFact / Grant split; seat confirm grants nothing) | `p11_t04_recipe_mention_grants_nothing` + `p11_t04_mcp_grant_is_per_scope` **pass** |
| Seating gate reads seated facts + model binding | `p11_t04_seated_facts_unblock_stage_predicate` **pass**; HTTP register+seat **pass** at `9cb1b6dc` |
| Role≠Agent; chat/handoff cannot transfer authority | `p11_t04_role_is_not_agent` + `p11_t04_chat_cannot_transfer_authority` **pass** |
| Speech whitelist + audit row (T04 half) | `p11_t04_speech_whitelist_enforced` **pass**. Conversation archive landing = T05 **not-run** / handoff |
| Input tab is PlanRevision seam; Employee stores `responsible_stage_ids` only | Schema + seating path (no contract-byte column). Eight-tab chrome remains T13 |
| Host lifecycle E2E | **not-run** (`DEV-WINDOWS-NATIVE-OPC-01` / Requires-environment) |
| T04-N1 real DSH child kill | **not-run**; handed to T07 |
| Grant-expansion HITL preview / time-box | **not-run**; handed to T09 |

## Validation

| Unit | Result | Env | Revision |
|---|---|---|---|
| store T04-N1..N11 + Role≠Agent + Blueprint no Provider + history + seated predicate + task-channel | **pass** 16/16 | `DEV-LINUX-NATIVE-01` | `9cb1b6dc` (code same ancestry as `62840c9b`) |
| `p1_t01_layout_migrations` v27 | **pass** 8/8 | `DEV-LINUX-NATIVE-01` | `9cb1b6dc` |
| T03 store retained | **pass** 19/19 | `DEV-LINUX-NATIVE-01` | `9cb1b6dc` |
| HTTP empty-roster + register/seat + T03 G1/N12 | **pass** 7/7 | `DEV-LINUX-NATIVE-01` | `9cb1b6dc` |
| Clippy `-D warnings` store + kernel-server | **pass** | `DEV-LINUX-NATIVE-01` | `9cb1b6dc` |
| `check-consistency` / handbook / generate `--check` | **pass** | `DEV-WIN-GNU-01` | `9cb1b6dc` |
| resolve validation route | **pass** | required CI [33292431146](https://github.com/agentkernel/cognitive-os/actions/runs/33292431146) | `62840c9b` |
| `verify (ubuntu-latest)` | **pass** 3m37s | `CI-UBUNTU-01` | `62840c9b` |
| `verify (windows-latest)` | **pass** 11m9s | `CI-WINDOWS-MSVC-01` | `62840c9b` |
| `required-ci` | **pass** | required CI [33292431146](https://github.com/agentkernel/cognitive-os/actions/runs/33292431146) | `62840c9b` |
| Host E2E / `DEV-WINDOWS-NATIVE-OPC-01` | **not-run** | unqualified host | — |
| Rust link on `DEV-WIN-GNU-01` | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | — |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit, or live `/ui/` IA (A7: local/CI is hypothesis only). Not T02 (Windows host). Not T03 redo. Not T05 conversation archive. Not T07 hosted DSH / real process bind. Not T09 grant-expansion HITL chrome. Live `/ui/` remains Linux 1.0 six-family.

## Remaining parent closure

Acceptance evidence for `P11-T04` is complete at `62840c9b`. This checkpoint does **not** flip PR [#282](https://github.com/agentkernel/cognitive-os/pull/282), merge, close the lease, or claim `P11-T05`.

After the parent marks #282 ready and merges:

1. close `lease/personal/P11-T04/employee`;
2. delete the task branch when safe;
3. **then** claim `P11-T05` (implementation_requires: `P11-T03` done, `P11-T04` done, ADR-0058 retained). Do not treat this file as that claim.
