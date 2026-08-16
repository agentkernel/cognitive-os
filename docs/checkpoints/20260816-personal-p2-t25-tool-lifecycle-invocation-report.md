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

- Instrument: pending after the combined D01 Clippy fix + D02 head is pushed.
- Initial status: `not-run`.

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

### D02-CI-01 — Ubuntu supporting CI

- Instrument: pending after the D02 head is pushed.
- Initial status: `not-run`.
