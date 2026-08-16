# P2-T23 Public Memory/Skill consumption and resume — running validation report

- Task: `P2-T23`
- Branch: `personal/P2-T23-memory-skill-consumption-resume`
- Lease: `lease/personal/P2-T23/public-memory-skill-consumption-resume`
- Base: `origin/main` after P2-T22 merge `03bc530d692649a00efa7918bfeadddf8ba0ebea` / PR #221
- Change class: `implementation-only` (public redacted consumption GET and
  failure-first public lifecycle/task-consumption tests). Mapped handbook
  pages updated bilingually; fingerprints refreshed in the same change set.
- Claim ceiling: implementation evidence only; hypothesis/non-claim. No Gate,
  release, Profile, B01, EVAL, or Agent-benefit promotion.

本文件是本任务唯一的增量验证报告。每个已完成单元在下一个单元开始前追加记录；已发布
结果只通过追加的 superseding entry 更正。

## 预登记验证路由

- 本地 `DEV-WIN-GNU-01`：只运行 `cargo fmt --check`、静态一致性、Node、handbook、
  docs-sync 与 diff 检查；不运行 Rust build/test/Clippy（`RUST-LINK-DEV-WIN-GNU-01`
  已登记 exit 121 linker failure）。
- Rust 主验证：已推送精确 revision 的 GitHub Ubuntu required CI（`verify (ubuntu-latest)`
  workspace test + Clippy + handbook）。Windows 是
  `not-run by owner-directed Linux-only route`。
- `DEV-LINUX-NATIVE-01`（`wuz@192.168.1.2` / `hal9000`）：exact pushed-revision
  worktree；只做 native build/test/clippy/fmt 验证；不触碰 `B01-Desktop-Linux-002`
  guest / EVAL-004 campaign roots。
- `B01-Desktop-Linux-002` guest 属于 owner-directed evaluation campaign，与本 task
  验证无关，本任务不使用。

## D01 — failure-first public lifecycle and task consumption journey

### D01-DOC-01 — lease, plan, and BR-03 registration

- Instrument: `docs/plan/PARALLEL-LANES.md` active table,
  `docs/plan/PROGRESS.md` Current snapshot,
  `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`,
  `docs/evaluation/personal-performance-benchmark-readiness-closure-plan.md`
- Outcome: `pass` (authored). Lease
  `lease/personal/P2-T23/public-memory-skill-consumption-resume` claimed with
  `P2-T23/D01`. P2-T22 flipped to merged
  `03bc530d692649a00efa7918bfeadddf8ba0ebea` / PR #221. Layer 1
  `88 | 75 | 1 | 1 | 11 | 13`. BR-02 `done`, BR-03 `in-progress`.
- Disposition: opens D01; does not execute Rust tests.

### D01-IMPL-01 — public redacted consumption GET

- Instrument: `apps/kernel-server/src/personal/resource_api.rs`,
  `apps/kernel-server/src/personal/server.rs`
- Outcome: authored. `GET /task/resource/v1/consumption?task_ref=…` is a
  task-channel read of the latest daemon-authored v24 consumption record.
  It returns only exact Memory/Skill pins, session/`reuse_of` linkage, and
  `authorized_exact_pin`. `query_text` and `skill_binding_id` fail closed
  with `RESOURCE_CONSUMPTION_RESTATEMENT_FORBIDDEN`. Forgotten, revoked, or
  digest-drifted pins fail closed with `RESOURCE_CONSUMPTION_NOT_ELIGIBLE`
  before any pin is returned. Memory/Skill bodies are absent.
- Disposition: focused tests below must prove the public lifecycle journey
  and pre-rank negatives.

### D01-TEST-01 — public lifecycle then governed consumption without restatement

- Instrument:
  `personal::scheduler_authority::tests::public_memory_skill_lifecycle_then_task_consumption_does_not_require_query_restatement`
- Fixture: public `remember`/`review` plus `import`/`inspect`/`bind`, then
  production `resolve_authorized_task_context`, then GET consumption.
- Oracle: GET before resolve is `RESOURCE_CONSUMPTION_NOT_FOUND`; GET with
  `query_text` is restatement-forbidden; after resolve GET returns the
  remembered Memory and bound Skill pins and no body/instructions.
- Initial status: `not-run` locally (`RUST-LINK-DEV-WIN-GNU-01`). Ubuntu
  supporting CI is the first execution.

### D01-TEST-02 — pre-rank cross-scope / forgotten / revoked negatives

- Instrument:
  `personal::scheduler_authority::tests::public_consumption_rejects_cross_scope_forgotten_and_revoked_before_rank`
- Oracle: cross-scope remembered procedure text never appears in Context
  bodies or GET consumption; public forget makes reuse and GET fail closed
  without returning Memory pins; public revoke is accepted on the
  management path.
- Initial status: `not-run` locally.

### D01-TEST-03 — HTTP channel and restatement guards

- Instrument: `apps/kernel-server/tests/p4_t05_resource_api.rs`
- Oracle: GET consumption requires a task bearer and `task_ref`; management
  bearer is 403; `query_text` is restatement-forbidden; unknown Task is
  `RESOURCE_TASK_NOT_FOUND`.
- Initial status: `not-run` locally.

### D01-CI-01 — Ubuntu supporting CI at `a1845730`

- Instrument: GitHub Actions run `31921344719` (`verify (ubuntu-latest)` Test Rust workspace)
- Outcome: `fail`. `295 passed; 2 failed`. Both D01 public-journey tests panicked in
  `public_remember_and_bind` while sealing the MemoryCandidate:
  `canonical encoding failed: unsafe-integer: 9223372036854775807` (`i64::MAX`
  is outside I-JSON 2^53−1). HTTP guards in `p4_t05_resource_api` passed.
- Disposition: repair the sealed retention timestamp to an I-JSON-safe far-future
  value in the same change set as D02. Do not weaken assertions.

### D01-CI-02 — Ubuntu supporting CI at `beb4afb3`

- Instrument: GitHub Actions run `31922114988`
- Outcome: `fail`. `295 passed; 4 failed`. Public remember returned
  `RESOURCE_MEMORY_CONFLICT` because the helper set `target_scope` to the Task
  ref (policy requires same-scope as `governance_scope`) and retention
  `4_102_444_800` exceeded the 1-year admission ceiling.
- Disposition: public remember now uses workspace `target_scope` and
  `now + 3600` retention, matching `p4_t05_resource_api`. Skill bind remains
  task-targeted.

## D02 — persist bounded consumption and resume session 2 from durable state

### D02-TEST-01 — public session-2 GET after restart, zero restatement

- Instrument:
  `personal::scheduler_authority::tests::public_session_two_resumes_from_durable_state_after_restart_without_restatement`
- Oracle: public remember/bind then resolve session 1; GET returns exact pins and
  `reuse_of=null`. Session 2 changes only `conversation_ref`; resolve and GET
  return the same pins with `reuse_of` set and no Memory body. Store close/reopen
  GET still returns those pins without `query_text`.
- Initial status: `not-run` locally (`RUST-LINK-DEV-WIN-GNU-01`). Ubuntu
  supporting CI is the first execution.

### D02-TEST-02 — forged prompt and forged durable record

- Instrument:
  `personal::scheduler_authority::tests::public_forged_prompt_cannot_replace_durable_consumption_pins`
- Oracle: POST consumption with caller `query_text` is not the resume path
  (`RESOURCE_TASK_POLICY_MISSING` on this fixture); GET with `query_text` remains
  restatement-forbidden; GET after the POST still returns the daemon pins. A
  forged v24 row with a drifted request digest makes GET
  `RESOURCE_CONSUMPTION_NOT_ELIGIBLE` without returning pins.
- Initial status: `not-run` locally.

### D02-IMPL-01 — GET digest revalidation

- Instrument: `apps/kernel-server/src/personal/resource_api.rs`
- Outcome: authored. `GET /task/resource/v1/consumption` now also fails closed
when the durable record's `context_request_digest` differs from the live
ContextRequest, before any pin is returned.
