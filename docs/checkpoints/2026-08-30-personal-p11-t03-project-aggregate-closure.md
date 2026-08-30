# P11-T03 Project aggregate walking skeleton closure

- Task: `P11-T03` / slice `P11-T03/D01`
- Change class: `implementation-only` (Personal-private tables + private projection; no `core/specs`, no Lane-CTR)
- Branch: `personal/P11-T03-project-aggregate`
- Implementation revision: `7d9f13e4cfca76525672fdabf4f624ca1fe98aee`
- Tempfile lock: `8374d560cd55e5a4cb322cbf6588218309565ccc`
- Merge revision: `main@464073809ffadf1f2c08e7391bbac5b4b2c0ed8b`
- Content head: `aef5574e3a4c76a5b5e0e19fe4ed4ab0b0872e88`
- Pull request: [#281](https://github.com/agentkernel/cognitive-os/pull/281) (merged 2026-08-30)
- Required CI on `aef5574e`: run [33288037382](https://github.com/agentkernel/cognitive-os/actions/runs/33288037382) **SUCCESS** (resolve 3s, Ubuntu 3m32s including Rust workspace, Windows 11m56s, `required-ci`)
- Claim ceiling: `hypothesis`
- Evaluation routing: **OFF**

## Acceptance mapping

| Acceptance item | Evidence |
|---|---|
| Project identity is queryable (not a Task-row rename) | `p11_t03_project_is_not_a_task_row` plus list/get `/management/project/v1/*`. Linux store **19/19** and HTTP **6/6** at `7d9f13e4`; required CI workspace tests **pass** at `aef5574e` |
| Unconfirmed cannot become `active` | `p11_t03_unconfirmed_activate_rejected` (N2) **pass** |
| Cross-project write fails | `p11_t03_cross_project_write_rejected` (N4) **pass** |
| Charter/Goal/Metric/Plan revision are daemon authority | v26 tables + G1/G2 confirm chain; walking skeleton, not Today/`/ui/` |
| Task/Attempt via Intent/Effect + independent verification | Completion negatives N6/N7/N9; production seating empty-table fail-closed (N8). Honest usage body remains T12 |
| Walking skeleton, not full `/ui/` / T02 / T04 | Routes are Personal-private `/management/project/v1/*`. Roster projection empty + `employee-authority-not-implemented`. Live `/ui/` remains Linux 1.0 six-family |

## Validation

| Unit | Result | Env | Revision |
|---|---|---|---|
| store N1–N15 + G2 + cadence + pending-digest | **pass** 19/19 | `DEV-LINUX-NATIVE-01` | `7d9f13e4` |
| HTTP G1/list/roster/N12/pending-digest | **pass** 6/6 | `DEV-LINUX-NATIVE-01` | `7d9f13e4` |
| N16 store-reopen half | **pass** | `DEV-LINUX-NATIVE-01` | `7d9f13e4` |
| N16 Pi/DSH process-death | **not-run** | handed to T09 | — |
| `check-consistency` | **pass** | `DEV-WIN-GNU-01` | `aef5574e` |
| `check-handbook` + `generate-handbook --check` | **pass** 58×2 / 18 pages | `DEV-WIN-GNU-01` | `aef5574e` |
| Required CI [33288037382](https://github.com/agentkernel/cognitive-os/actions/runs/33288037382) | **SUCCESS** | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | `aef5574e` |
| Rust link on `DEV-WIN-GNU-01` | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | — |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit, or live `/ui/` IA. Not T02 (Windows host). Not T04 (Employee). Not T05 conversation version. N16 process-death/Pi/DSH half remains T09. Honest usage body remains T12.

## Deterministic closure

1. required CI [33288037382](https://github.com/agentkernel/cognitive-os/actions/runs/33288037382) **SUCCESS** on `aef5574e`;
2. PR [#281](https://github.com/agentkernel/cognitive-os/pull/281) marked ready and merged as `main@46407380` on 2026-08-30;
3. lease `lease/personal/P11-T03/project-aggregate` moved to §3.1;
4. local and remote task branches deleted after this commit; local `main` fast-forwarded to the merge plus this status/closure commit.

Do not auto-claim `P11-T04`.
