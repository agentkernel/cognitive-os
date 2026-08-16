# P2-T25 Tool lifecycle and real invocation — running validation report

- Task: `P2-T25`
- Branch: `personal/P2-T25-tool-lifecycle-invocation`
- PR: https://github.com/agentkernel/cognitive-os/pull/224
- Lease: `lease/personal/P2-T25/tool-lifecycle-invocation`
- Change class: `implementation-only`
- Claim ceiling: hypothesis/non-claim. No Gate, release, Profile, B01, EVAL, or
  Agent-benefit promotion.

本文件是本任务唯一的增量验证报告。每个已完成单元在下一个单元开始前追加记录。

## 预登记验证路由

- 本地 `DEV-WIN-GNU-01`：只运行 `cargo fmt --check`、静态一致性、Node、handbook、
  docs-sync 与 diff 检查；不运行 Rust build/test/Clippy（`RUST-LINK-DEV-WIN-GNU-01`）。
- Rust 主验证：已推送精确 revision 的 GitHub Ubuntu supporting CI
  (`verify (ubuntu-latest)` workspace test + Clippy `-D warnings` + handbook)。
  Windows 是 `not-run by owner-directed Linux-only route`。
- `DEV-LINUX-NATIVE-01`（`wuz@192.168.1.2` / `hal9000`）是 D03 所需的 exact
  pushed-revision 验收环境。
- `B01-Desktop-Linux-002` 属于 owner-directed EVAL-004-only guest；本任务不使用。

## D01 — public lifecycle projection, Agent exposure, bounded selection

### D01-TEST-01 — task channel cannot mutate lifecycle

- Instrument: `apps/kernel-server/tests/p2_t25_tool_lifecycle.rs`
  (`public_tool_lifecycle_propagates_to_agent_exposure_and_rejects_least_set_widening`)
- Oracle: task bearer `POST /task/resource/v1/tool/disable` returns 403
  `RESOURCE_TOOL_LIFECYCLE_CHANNEL_FORBIDDEN`.
- Initial status: `not-run` locally (`RUST-LINK-DEV-WIN-GNU-01`). Ubuntu
  supporting CI is the first execution.

### D01-TEST-02 — disable drops Agent exposure; stale digest fails closed

- Instrument: same HTTP test plus
  `personal::tool_lifecycle` unit tests
  (`missing_overlay_exposes_enabled_execution_ready_tools`,
  `disable_drops_agent_exposure_and_rejects_stale_selection_digest`,
  `quarantine_blocks_enable_and_prompt_restatement_is_rejected`)
- Oracle: default catalog exposes enabled execution-ready tools;
  `POST /management/resource/v1/tool/disable` updates `agent_exposed` atomically;
  `GET /management/resource/v1/tool/discover` no longer falls through to generic
  authority; selection with a pre-disable `candidate_set_digest` returns 409
  `RESOURCE_TOOL_SELECTION_EXPOSURE_MISMATCH`; matching digest + exposed
  `operation_id` records a receipt; `prompt` restatement returns 400
  `RESOURCE_TOOL_SELECTION_QUERY_FORBIDDEN`. Overlay state never enters the
  immutable descriptor digest.
- Initial status: `not-run` locally.

### D01-IMPL-01 — public surfaces

- Instrument: `apps/kernel-server/src/personal/tool_lifecycle.rs`,
  `apps/kernel-server/src/personal/server.rs`
- Outcome: authored. Overlay file `$data_dir/personal-tool-lifecycle.json`
  schema `cognitiveos.personal.tool-lifecycle/0.1`. States:
  `enabled` (catalog default) / `disabled` / `quarantined` / `revoked`.
  Quarantine blocks enable (`RESOURCE_TOOL_QUARANTINED`); revoke is terminal
  (`RESOURCE_TOOL_REVOKED`). Task POST enable/disable/quarantine/revoke/discover
  is channel-forbidden. Management POST selection is channel-forbidden. Existing
  `GET /resource/v1/projection?family=tool` is unchanged.

### D01-CI-01 — Ubuntu supporting CI at `6da631ed` (superseded)

- Instrument: GitHub Actions run
  [31928197414](https://github.com/agentkernel/cognitive-os/actions/runs/31928197414)
- Outcome: `fail` — `kernel-server` did not compile under `-D warnings`:
  `ToolLifecycleResponse` lacks `Debug`, so unit-test `.expect("file")` on
  `load_file` is rejected.
- Disposition: derive `Debug` on `ToolLifecycleResponse`.

### D01-CI-02 — Ubuntu supporting CI at `c1cfcc13` (superseded)

- Instrument: GitHub Actions run
  [31929145967](https://github.com/agentkernel/cognitive-os/actions/runs/31929145967)
- Outcome: `fail` — Clippy `-D warnings` rejected
  `clippy::comparison_to_empty` in `tool_lifecycle.rs`
  (`key != ""` in the exposure query parser).
- Disposition: use `!key.is_empty()`.

### D01-CI-03 — Ubuntu supporting CI after empty-comparison Clippy fix

- Instrument: combined into the D02 head `4dfd4f89` (run
  [31930110947](https://github.com/agentkernel/cognitive-os/actions/runs/31930110947)).
- Outcome: superseded by D02-CI-01 / D02-LINUX-01 (integration test E0716).

## D02 — RegisteredCheckRun success under lifecycle, pinned read-only HTTPS

### D02-TEST-01 — task channel cannot pin HTTPS origins

- Instrument: `apps/kernel-server/tests/p2_t25_tool_lifecycle.rs`
  (`pinned_https_origin_registry_is_campaign_scoped_and_task_forbidden`)
  plus `personal::pinned_https` unit tests.
- Oracle: task bearer `POST /task/resource/v1/http-origin` returns 403
  `RESOURCE_PINNED_HTTPS_CHANNEL_FORBIDDEN`; credential origins return 400
  `RESOURCE_PINNED_HTTPS_ORIGIN_INVALID`; authorized campaign `P2-T25` can pin
  `https://example.com` and `https://localhost:8443` (host:port).
- Initial status: `not-run` locally (`RUST-LINK-DEV-WIN-GNU-01`).

### D02-TEST-02 — disable RegisteredCheckRun drops Agent exposure

- Instrument: same HTTP test.
- Oracle: `POST /management/resource/v1/tool/disable` for
  `native.registered-check.run` updates overlay; subsequent
  `GET /task/resource/v1/tool/exposure` no longer lists that operation_id.
  No ProcessRun family is added.
- Initial status: `not-run` locally.

### D02-IMPL-01 — production HttpFetchReadOnly consults the pin registry

- Instrument: `pinned_https.rs`, `tool_executor/http_fetch.rs` /
  `router.rs`, `scheduler_authority/dispatch.rs`,
  `validate_read_only_http_fetch` now admits `host` or `host:port`.
- Outcome: authored. Overlay file `$data_dir/personal-pinned-https.json`
  schema `cognitiveos.personal.pinned-https/0.1`. Default allowlist empty.
  Production staging uses Intent/`authorization.task_ref` pins; missing pin
  stays fail closed. Dispatch remains GET-only; the validator still admits
  HEAD. Unauthorized campaign `owner-local` is refused.

### D02-TEST-03 — descriptor drift, redirect, oversize, duplicate/restart

- Instrument: `production_router_stages_http_fetch_after_campaign_pin_and_rejects_drift`;
  `http_fetch_refuses_redirect_status_without_following`; existing oversize
  (`ResponseTooLarge` → `NotExecuted`) and original-key restart/duplicate
  HttpFetch tests.
- Oracle: campaign pin of `https://example.com` lets production staging
  succeed; drifted descriptor digest fails closed; HTTP 302 is `NotExecuted`
  and is not followed; oversize retains nothing; duplicate dispatch and
  restart query the original key.
- Initial status: `not-run` locally.

### D02-CI-01 — Ubuntu supporting CI at `de36ff59` (superseded)

- Instrument: GitHub Actions run
  [31930110947](https://github.com/agentkernel/cognitive-os/actions/runs/31930110947)
  (PR head `de36ff59`, product tests identical to `4dfd4f89`)
- Outcome: `fail` (compile):
  1. E0716 temporary `response_json(&exposure)["exposed"]` dropped while borrowed;
  2. E0061 `PersonalDataLayout::from_xdg_roots` in `pinned_https` tests still
     passed 3 arguments after the 5-root layout API.
- Disposition: bind `exposure_json` before indexing; pass five XDG roots.

### D02-LINUX-01 — exact-revision `DEV-LINUX-NATIVE-01` at `4dfd4f89` (superseded)

- Instrument: worktree `/home/wuz/agent-kernel-worktrees/p2-t25-4dfd4f89`,
  rustc 1.97.1, `cargo test -p kernel-server --test p2_t25_tool_lifecycle --locked`
- Outcome: `fail` compile E0716 in
  `apps/kernel-server/tests/p2_t25_tool_lifecycle.rs:350`.
- Disposition: bind `exposure_json` before indexing. Clippy/fmt `not-run`
  (compile failed first). Windows `not-run by owner-directed Linux-only route`.
  `B01-Desktop-Linux-002` untouched.

### D02-CI-02 — Ubuntu supporting CI at `6ae64b50` (superseded)

- Instrument: GitHub Actions run
  [31930441274](https://github.com/agentkernel/cognitive-os/actions/runs/31930441274)
- Outcome: `fail` — workspace tests compiled and ran; Clippy `-D warnings`
  rejected `clippy::expect_used` on `pinned_https` test `layout()` (`expect("clock")`
  and `expect("temp layout")`).
- Disposition: allow `expect_used`/`unwrap_used`/`panic` on the test module,
  matching `tool_lifecycle` tests.

### D02-CI-03 — Ubuntu supporting CI after Clippy test-module allow

- Instrument: GitHub Actions run
  [31930697492](https://github.com/agentkernel/cognitive-os/actions/runs/31930697492)
- Outcome: `pass` at exact `492f9e9f6bc626e864efb9b7564823c1b20c0b7f`
  (workspace tests, Clippy `-D warnings`, rustfmt, handbook, consistency).
  Windows `not-run by owner-directed Linux-only route`.

### D02-LINUX-02 — exact-revision `DEV-LINUX-NATIVE-01` at `941a0a29` (superseded)

- Instrument: worktree `/home/wuz/agent-kernel-worktrees/p2-t25-4dfd4f89`,
  rustc 1.97.1
- Outcome: `cargo test -p kernel-server --test p2_t25_tool_lifecycle --locked`
  **2/2 pass**. The follow-on `cargo test -p kernel-server --lib --locked
  pinned_https` was a command error (`kernel-server` is binary-only; no library
  target). cognitive-kernel / fmt / Clippy `not-run` (command stopped).
- Disposition: unit tests run as
  `cargo test -p kernel-server --bin kernel-server --locked pinned_https`.

### D02-LINUX-03 — exact-revision `DEV-LINUX-NATIVE-01` at `6ae64b50` (superseded)

- Instrument: same worktree after LAN bundle fetch, rustc 1.97.1
- Outcome: `p2_t25_tool_lifecycle` 2/2; `pinned_https` 5/5;
  `production_router_stages_http_fetch_after_campaign_pin_and_rejects_drift` 1/1;
  `http_fetch_refuses_redirect_status_without_following` 1/1; `cargo fmt --all
  -- --check` pass. Filter `valid_https` matched 0 tests (the HTTPS validator
  cases live in `workspace_process_and_http_validators_fail_closed`). Clippy
  `-D warnings` failed on the same `expect_used` helpers as D02-CI-02.

### D02-LINUX-04 — exact-revision `DEV-LINUX-NATIVE-01` at `492f9e9f`

- Instrument: same worktree after LAN bundle fetch of `492f9e9f`, rustc 1.97.1
- Outcome: `pass` — `workspace_process_and_http_validators_fail_closed` 1/1;
  `cargo fmt --all -- --check` pass; `cargo clippy --workspace --all-targets
  --locked -- -D warnings` pass. Windows `not-run by owner-directed Linux-only
  route`. `B01-Desktop-Linux-002` untouched.

## D03 — exact-revision linux-002 lifecycle/call/reconcile/cleanup matrix

Product revision remains `492f9e9f6bc626e864efb9b7564823c1b20c0b7f`.

### D03-LINUX-01 — lifecycle and call matrix

- Instrument: `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2` / `hal9000`), worktree
  `/home/wuz/agent-kernel-worktrees/p2-t25-4dfd4f89`, rustc 1.97.1, exact
  `492f9e9f`
- Outcome: `pass`
  - `p2_t25_tool_lifecycle` 2/2
  - `tool_lifecycle` units 3/3
  - `pinned_https` 5/5
  - `registered_check` 24/24 (includes production TypeScript/Rust repair
    journeys; no ProcessRun; argv/env/cwd/credentials/network injection
    fail closed)
  - `production_router_stages_http_fetch_after_campaign_pin_and_rejects_drift` 1/1
  - `http_fetch_refuses_redirect_status_without_following` 1/1
  - `workspace_process_and_http_validators_fail_closed` 1/1 (HTTPS path,
    `host:port`, credential URL, HEAD ok, POST refused)

### D03-LINUX-02 — original-key reconcile regression

- Instrument: `cargo test -p kernel-server --locked p2_t17` at the same SHA
- Outcome: `pass` — 15/15. `select_single_effect_intent` is unchanged.

### D03-LINUX-03 — kernel-server bin, workspace, Clippy, fmt, residue

- Instrument: `cargo test -p kernel-server --locked --bins`;
  `cargo test --workspace --locked`;
  `cargo clippy --workspace --all-targets --locked -- -D warnings`;
  `cargo fmt --all -- --check`
- Outcome: `pass` — kernel-server bin 319/319; workspace tests 0-failed;
  Clippy `-D warnings` green; fmt green.
- Cleanup: removed leftover `/tmp/cos-p2t25-*` hermetic roots and
  `/tmp/p2-t25-*.bundle` LAN bundles. `B01-Desktop-Linux-002` was not used
  (EVAL-004-only guest). Windows `not-run by owner-directed Linux-only
  route`.

### D03-ACCEPT-01 — formal acceptance mapping

| Acceptance | Evidence |
|---|---|
| Lifecycle to Agent exposure is atomic | D01 HTTP test + `tool_lifecycle` 3/3: disable drops `agent_exposed`; stale selection digest 409 |
| Bounded selection receipt | matching digest + exposed `operation_id` records a receipt; prompt restatement 400 |
| Immutable RegisteredCheckRun | `registered_check` 24/24; descriptor digest unchanged by overlay; no ProcessRun |
| Campaign-scoped pinned HTTPS GET/HEAD | origin registry + production staging 1/1; validator HEAD ok / POST err |
| Drift, redirect, credential URL, oversize, duplicate/restart | drift 1/1; 302 `NotExecuted`; credential origin 400; existing oversize/original-key tests in bin 319/319 |
| Task cannot mutate lifecycle or pin origins | both HTTP tests 403 channel-forbidden |
| Exact-revision linux-002 | this D03 matrix at `492f9e9f` |
| Ubuntu supporting CI | run `31930697492` pass |

### D03-MERGE-01 — ready/merge

- Instrument: GitHub PR [#224](https://github.com/agentkernel/cognitive-os/pull/224)
- Outcome: **merged** at `main@4b10db9a64584bda42ae249bd1df289475bd6324`.
  Lease `lease/personal/P2-T25/tool-lifecycle-invocation` closed. Remote task
  branch deleted. Claim ceiling: hypothesis/non-claim. No Gate, release,
  Profile, B01, EVAL, or Agent-benefit promotion.
