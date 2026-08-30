# P11-T05 Conversation archive closure

- Task: `P11-T05` / slices `P11-T05/D01` + `P11-T05/D02` (full Phase 11 T05 acceptance)
- Change class: `implementation-only` (Personal-private archive table + private projection; no `core/specs`, no Lane-CTR)
- Branch: `personal/P11-T05-conversation`
- D01 implementation revision: `90f1ba4b6017058c4d233534eaed200cb6b9264a`
- D01 layout-aligned revision: `66b18a143464381f535487bab9f6a8f08c050cd1`
- D02 Linux evidence revision: `aeae75f221881be55ff284f206f219c1a43a39a3`
- Closure / merge HEAD: `02476d60178ca9e6f708dc897cb22cada4391f7f`
- Pull request: [#283](https://github.com/agentkernel/cognitive-os/pull/283) **Draft** (parent flips ready/merge; this checkpoint does not)
- Lease: `lease/personal/P11-T05/conversation` (stays active until parent merge/lease close)
- Required CI on `02476d60`: **pass** (run [33304121486](https://github.com/agentkernel/cognitive-os/actions/runs/33304121486); ubuntu [99237534399](https://github.com/agentkernel/cognitive-os/actions/runs/33304121486/job/99237534399), windows [99237534446](https://github.com/agentkernel/cognitive-os/actions/runs/33304121486/job/99237534446), required-ci [99239185986](https://github.com/agentkernel/cognitive-os/actions/runs/33304121486/job/99239185986)). Incremental log: [report](2026-08-30-personal-p11-t05-conversation-report.md)
- Claim ceiling: `hypothesis`
- Evaluation routing: **OFF**

## Acceptance mapping

D01+D02 cover full Phase 11 T05 acceptance. Host archive E2E and `DEV-WIN-GNU-01` cargo remain honest **not-run**.

| Acceptance item | Evidence |
|---|---|
| New Personal-private identifier; no first Lane-CTR | v28 `p11_conversation_archive` + envelope `cognitiveos.personal.conversation-archive/0.1`. Implementation-only; no `core/specs` |
| ADR-0058 `conversation-projection/0.1` not coerced (T05-N1) | `p11_t05_legacy_projection_not_coerced` **pass**. HTTP speech/archive with `conversation-projection/0.1` / `v01` **422** at `aeae75f2` |
| Speech whitelist lands archive; chatter does not (T04-N9) | `p11_t05_deliverable_lands_chatter_does_not` **pass**. HTTP `speech.candidate` deliverable lands / chatter audit-only **pass** at `aeae75f2` |
| Secret-shape body refused (T05-N2) | `p11_t05_append_rejects_secret_shape` **pass** |
| Cross-Project / cross-Employee read refused (T05-N3) | `p11_t05_cross_scope_read_rejected` **pass** |
| Bounded authorized index (`limit` 1..=32); unbounded resume refused (T05-N4) | `p11_t05_unbounded_resume_rejected` **pass**. HTTP missing `limit` **422** at `aeae75f2` |
| Index returns refs only; full-archive injection refused (T05-N5) | `p11_t05_index_does_not_embed_bodies` **pass**. HTTP `include_bodies=1` **422**; index body omitted **pass** at `aeae75f2` |
| Archive ≠ Task/Project completion (T05-N6) | `p11_t05_archive_is_not_completion` **pass**. HTTP `observation_only: true` |
| Owner `conversation.append` + single-record fetch | Store append in N5; HTTP `conversation.append` / `conversation.record` **pass**; task-channel append **403** at `aeae75f2` |
| Linux store T04-N9 + T05-N1..N6 | **pass** 7/7 at `aeae75f2` |
| Host archive E2E | **not-run** (`DEV-WINDOWS-NATIVE-OPC-01` / Requires-environment) |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |

## Validation

| Unit | Result | Env | Revision |
|---|---|---|---|
| store T04-N9 + T05-N1..N6 | **pass** 7/7 | `DEV-LINUX-NATIVE-01` | `aeae75f2` |
| `delivered_speech_lands_in_archive_via_http` (bounded archive + append + record + 422/403) | **pass** | `DEV-LINUX-NATIVE-01` | `aeae75f2` |
| `p1_t01_layout_migrations` v28 | **pass** 8/8 | `DEV-LINUX-NATIVE-01` | `66b18a14` (D01; D02 did not re-run) |
| D01 `required-ci` | **pass** (not merge HEAD) | required CI [33302761491](https://github.com/agentkernel/cognitive-os/actions/runs/33302761491) | `66b18a14` |
| `verify (ubuntu-latest)` on D02 code | **pass** 3m33s (not merge HEAD) | `CI-UBUNTU-01` [33303578323](https://github.com/agentkernel/cognitive-os/actions/runs/33303578323) | `aeae75f2` |
| `check-consistency` / handbook / generate `--check` | **pass** | `DEV-WIN-GNU-01` | D02 commits through `aeae75f2` |
| Host E2E / `DEV-WINDOWS-NATIVE-OPC-01` | **not-run** | unqualified host | — |
| Rust link on `DEV-WIN-GNU-01` | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | — |
| `verify (ubuntu-latest)` on merge HEAD `02476d60` | **pass** 5m1s (job [99237534399](https://github.com/agentkernel/cognitive-os/actions/runs/33304121486/job/99237534399)) | `CI-UBUNTU-01` | `02476d60178ca9e6f708dc897cb22cada4391f7f` |
| `verify (windows-latest)` on merge HEAD `02476d60` | **pass** 15m15s (job [99237534446](https://github.com/agentkernel/cognitive-os/actions/runs/33304121486/job/99237534446)) | `CI-WINDOWS-MSVC-01` | `02476d60178ca9e6f708dc897cb22cada4391f7f` |
| `required-ci` on merge HEAD `02476d60` | **pass** 2s (job [99239185986](https://github.com/agentkernel/cognitive-os/actions/runs/33304121486/job/99239185986); run [33304121486](https://github.com/agentkernel/cognitive-os/actions/runs/33304121486)) | GitHub Actions | `02476d60178ca9e6f708dc897cb22cada4391f7f` |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit, or live `/ui/` IA (A7: local/CI is hypothesis only). Not T02 (Windows host). Not T03/T04 redo. Not T06 hidden Pi Assistant. Not T07 hosted DSH. Not T10 Knowledge/Vault. Archive rows are observation-only, not Task/Project completion. ADR-0058 `conversation-projection/0.1` is retained. Live `/ui/` remains Linux 1.0 six-family.

## Remaining parent closure

D01+D02 acceptance mapping for `P11-T05` is recorded at product HEAD `02476d60`, with Linux proof at `aeae75f2`. Merge-HEAD `required-ci` on `02476d60` is **pass** (run [33304121486](https://github.com/agentkernel/cognitive-os/actions/runs/33304121486)). This checkpoint commit creates a new SHA that needs its own required-ci. This checkpoint does **not** flip PR [#283](https://github.com/agentkernel/cognitive-os/pull/283), merge, close the lease, or claim `P11-T06`.

After the parent confirms required-ci on the new checkpoint HEAD, marks #283 ready, and merges:

1. close `lease/personal/P11-T05/conversation`;
2. delete the task branch when safe;
3. **then** claim `P11-T06` (implementation_requires: `P11-T03` done, `P11-T05` done, exact Pi foundation). Do not treat this file as that claim.
