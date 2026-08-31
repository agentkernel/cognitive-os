# P12-T08 Settings connections — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P12-T08` / slice `P12-T08/D01`
- Branch: `personal/P12-T08-settings`
- Lease: `lease/personal/P12-T08/settings-connections`
- Change class: `implementation-only` (daemon-served `/ui/` Settings connections; no new authority writer; no `core/specs`)
- Unique next: required CI on Draft PR [#301](https://github.com/agentkernel/cognitive-os/pull/301)

Product origin is daemon-served `/ui/`. Vite/canvas is not the product. Connection table reads GET `/management/providers/accounts` + GET `/management/usage`; unknown/`cost_unavailable` never render as 0. 「本周不再问」revoke is POST `/management/project/v1/standing-policy.revoke` (time-box, not permanent; chat cannot mint). CloseBackgroundDialog POSTs `/management/host/v1/close.request` choice `background`|`pause` only when GET `host/v1/status` reports a home; fake background is not posted. Native close/host E2E is `not-run`. Not T09. Not T15. NVDA/200%/host-theme remain hung. `DEV-WIN-GNU-01` cargo is `not-run`.

## Units

| Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|
| Merge PR [#300](https://github.com/agentkernel/cognitive-os/pull/300) (P12-T07) | **pass** | GitHub | `main@081c40d0` | Required CI [33401268090](https://github.com/agentkernel/cognitive-os/actions/runs/33401268090) **SUCCESS** at `fefd6872`. Remote task branch retained historically. |
| Claim `lease/personal/P12-T08/settings-connections` | **pass** | `DEV-WIN-GNU-01` | worktree `D:/agent-kernel-wt-P12-T08` from `origin/main@081c40d0` | Dirty `d:\agent-kernel` (DOC-REFRAME) not overwritten. Evaluation routing OFF. |
| Dual Track TS Settings connections (`settingsConnections` + `host` + `connectionUsage` + `standingPolicies` + `opcIa` + `normalize`) | **pass** | `DEV-WIN-GNU-01` | `bd440f72` | personal-web-ui **405/405** (53 files). Honest empty table; unknown usage never 0; secret presence only; revoke via standing-policy.revoke; revoke reject keeps row; CloseBackgroundDialog posts `background`/`pause` only with a home; no-home does not post; `can_honor_background=false` does not post fake background. Native close/host E2E **not-run**. NVDA/200%/host-theme **not-run**. GNU cargo **not-run**. |
| Draft PR [#301](https://github.com/agentkernel/cognitive-os/pull/301) | **pass** | GitHub | `bd440f72` | Unique next = required CI. |
| NVDA / 200% / host-theme | **not-run** | Requires-environment | — | hung; not a P12 close gate |
| Native close/host E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` unqualified | — | not a product fail |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | — | route to `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` |
