# P11-T12 Provider honest usage closure

- Task: `P11-T12` / slice `P11-T12/D01` (full Phase 11 T12 acceptance)
- Change class: `implementation-only` (labelled usage read on v25 Provider Control Plane; no `core/specs`, no Lane-CTR, no Settings chrome, no member-level budget hard-stop)
- Branch: `personal/P11-T12-usage`
- D01 implementation revision: `eb27cb8625cd2cde2494b451547a2984f126feee`
- clippy `collapsible_if` revision: `453ffc371d498cb17be91a71cc395983db497403`
- clippy `redundant_guards` pattern revision: `3aaac23dd5fb8bef2f8209c666eb066e0a47cedb`
- Product HEAD recorded for required-ci: `60c310ae34596fdc55875b6c53aade0b7897b823`
- Pull request: [#286](https://github.com/agentkernel/cognitive-os/pull/286) **Draft** (parent flips ready/merge; this checkpoint does not)
- Lease: `lease/personal/P11-T12/usage` (stays active until parent merge/lease close)
- Required CI on `60c310ae`: **pass** (run [33315265506](https://github.com/agentkernel/cognitive-os/actions/runs/33315265506): [required-ci](https://github.com/agentkernel/cognitive-os/actions/runs/33315265506/job/99269075926) 3s, [ubuntu](https://github.com/agentkernel/cognitive-os/actions/runs/33315265506/job/99267396578) 3m42s, [windows](https://github.com/agentkernel/cognitive-os/actions/runs/33315265506/job/99267396612) 13m33s). Incremental log: [report](2026-08-30-personal-p11-t12-usage-report.md)
- Claim ceiling: `hypothesis`
- Evaluation routing: **OFF**

## Acceptance mapping

D01 covers full Phase 11 T12 close gate including workspace `required-ci` **pass** on `60c310ae`. Host SecretStore/Provider E2E, Settings chrome, member-level budget hard-stop, and `DEV-WIN-GNU-01` cargo remain honest **not-run**. Linux store 6/6 and named usage/silent-rebind HTTP at `eb27cb86` are **pass**. Crate-scoped clippy at `60c310ae` is **pass**.

| Acceptance item | Evidence |
|---|---|
| Honest usage; unknown cost never serializes as JSON `0` | store `p11_t12_unknown_cost_never_serializes_as_zero` **pass**; HTTP `http_usage_unknown_cost_never_zero_and_omits_secrets` **pass** at `eb27cb86`. T03 `unknown_cost_projection()` delegates to `honest_unknown_cost("project")` |
| Labelled read `actual \| estimated \| unknown`; do not pretend `locally_estimated` | store `p11_t12_labelled_read_maps_existing_enums_honestly` **pass**. `locally_estimated` mapped only when that metering_source was recorded |
| Binding explanation global→Project→employee→Task; missing layers unbound, no invented zeros | store `p11_t12_binding_explanation_is_durable_and_unbound_at_missing_layers` **pass**. Project/employee/Task unbound today |
| Account vs quota fields separated | store `p11_t12_account_and_quota_fields_are_separated` **pass** |
| Silent rebind rejected | store `p11_t12_silent_rebind_is_rejected` **pass**; HTTP `http_silent_rebind_is_rejected` → `PROVIDER_SILENT_REBIND_REJECTED` **pass** at `eb27cb86` |
| Raw secret never in usage HTTP / SQLite scan | store `p11_t12_secret_never_lands_in_usage_read_model` **pass**; HTTP same |
| Member-level budget hard-stop is not current chrome / not this card's done | **not-run** / Deferred 2.1. Not a T12 close-gate item |
| Did not wait on T07 to start honest usage | T12 `implementation_requires` is T03+T04 + Provider CP only. T07 is the consumer |
| Linux store T12 focused negatives | **pass** 6/6 at `eb27cb86` (`DEV-LINUX-NATIVE-01`) |
| Linux crate-scoped Clippy `-D warnings` (kernel-server + cognitive-store) | **pass** at `60c310ae` (`DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t12-eb27cb86`) |
| Host SecretStore / Provider E2E | **not-run** (card allows until qualified) |
| Settings chrome | **not-run** (T13; HTTP labelled usage is the caller) |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |
| Workspace `required-ci` on `60c310ae` | **pass** run [33315265506](https://github.com/agentkernel/cognitive-os/actions/runs/33315265506): [required-ci](https://github.com/agentkernel/cognitive-os/actions/runs/33315265506/job/99269075926), [ubuntu](https://github.com/agentkernel/cognitive-os/actions/runs/33315265506/job/99267396578), [windows](https://github.com/agentkernel/cognitive-os/actions/runs/33315265506/job/99267396612) |

## Validation

| Unit | Result | Env | Revision |
|---|---|---|---|
| store unknown≠0 + labelled read + binding explanation + account/quota + silent rebind + no secret | **pass** 6/6 | `DEV-LINUX-NATIVE-01` | `eb27cb8625cd2cde2494b451547a2984f126feee` |
| kernel-server `http_usage_unknown_cost_never_zero_and_omits_secrets` | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `eb27cb8625cd2cde2494b451547a2984f126feee` |
| kernel-server `http_silent_rebind_is_rejected` | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `eb27cb8625cd2cde2494b451547a2984f126feee` |
| `cargo clippy -p kernel-server --all-targets -- -D warnings` | **pass** | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t12-eb27cb86` | `60c310ae34596fdc55875b6c53aade0b7897b823` |
| `cargo clippy -p cognitive-store --all-targets -- -D warnings` | **pass** | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t12-eb27cb86` | `60c310ae34596fdc55875b6c53aade0b7897b823` |
| `check-consistency` / handbook / generate `--check` / docs-sync-gate | **pass** | `DEV-WIN-GNU-01` | D01 commits through `60c310ae` |
| SecretStore/Provider host E2E | **not-run** | unqualified host | `60c310ae` |
| Settings chrome / member budget stop | **not-run** | T13 / 2.1 Deferred | `60c310ae` |
| Rust link on `DEV-WIN-GNU-01` | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | `60c310ae` |
| workspace `required-ci` on product HEAD `60c310ae` | **pass** | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | `60c310ae34596fdc55875b6c53aade0b7897b823` |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit, or live `/ui/` IA (A7: local/CI is hypothesis only). Not T07 hosted DSH. Not T09 HITL rewrite. Not T13 Settings chrome. Not member-level budget hard-stop as 2.0.0 first-class chrome. Not inventing Project/employee/Task Provider bindings or quota numbers. `locally_estimated` is not pretended. This checkpoint records workspace `required-ci` **pass** on `60c310ae` and does not ready/merge #286, and does not claim the next P11 task. Live `/ui/` remains Linux 1.0 six-family.

## Remaining parent closure

D01 acceptance mapping for `P11-T12` is recorded at product HEAD `60c310ae`, with Linux store+HTTP **pass** at `eb27cb86`, crate-scoped clippy **pass** at `60c310ae`, and workspace `required-ci` **pass** on `60c310ae` (run 33315265506). This file does **not** flip PR [#286](https://github.com/agentkernel/cognitive-os/pull/286), merge, close the lease, or claim `P11-T07`. Committing this checkpoint moves HEAD off `60c310ae`; that new SHA needs its own required-ci before merge.

After the parent confirms required-ci on `60c310ae` (or the committed checkpoint HEAD), marks #286 ready, and merges:

1. close `lease/personal/P11-T12/usage`;
2. delete the task branch when safe;
3. **then** claim `P11-T07` (the T12 unlock: `implementation_requires` `P11-T03` done, `P11-T04` done, `P11-T12`). `P11-T10` already had `P11-T05` done and did not wait on T12. Other already-ready non-overlapping tasks (`P11-T02`, `P11-T08`, `P11-T10`) are not this unlock. `P11-T11` waits on T10. `P11-T13` is `/ui/` IA. `P11-T14`/`P11-T15` stay **parked** — do not unpark. Do not treat this file as that claim.
