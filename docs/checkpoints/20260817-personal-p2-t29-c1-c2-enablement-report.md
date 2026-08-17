# P2-T29 C1/C2 enablement — running validation report

- Task: `P2-T29`
- Branch: `personal/P2-T29-c1-c2-enablement`
- Lease: `lease/personal/P2-T29/c1-c2-enablement`
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
- `B01-Desktop-Linux-002` 属于后续 EVAL freeze guest；本任务不使用。

## D01 — advertise daemon-governed Workspace*

### D01-IMPL-01 — Extension advertisement and adapter extraction

- Instrument: `packages/pi-cognitiveos` Workspace* `registerTool` + empty
  `READ_ONLY_TOOL_ALLOWLIST`; `apps/pi-agent-adapter` `--no-builtin-tools` and
  Workspace* event extraction onto the P2-T21 candidate shape.
- Outcome: authored. bash/edit/write remain blocked. Execute is I/O-free.

### D01-VAL-01 — local Node Extension tests (`DEV-WIN-GNU-01`)

- Instrument: `pnpm --filter @cognitiveos/pi-cognitiveos build` then package
  test (allowlist empty, three Workspace* tools registered, bash still blocked,
  Workspace* `tool_call` not blocked, execute I/O-free).
- Outcome: **pass** 92/92.
- Rust adapter extraction tests: **not-run** (`RUST-LINK-DEV-WIN-GNU-01`);
  routed to Ubuntu supporting CI and exact-revision `DEV-LINUX-NATIVE-01`.

## D02 — daemon-owned Memory composer

### D02-IMPL-01 — unsealed public remember

- Instrument: `ResourceApi` loads the persisted `GovernanceSeed` and composes
  sealed headers for unsealed public remember; sealed envelopes remain valid.
- Outcome: authored. Caller-minted headers on the unsealed path fail closed.

### D02-VAL-01 — local static gates (`DEV-WIN-GNU-01`)

- `cargo fmt --all`: **pass**.
- `node tools/src/check-handbook.mjs`: **pass** (54 documents × 2 locales).
- `node tools/src/generate-handbook.mjs --check`: **pass** (18 pages
  byte-identical).
- `git diff --check`: **pass**.
- `pnpm run check:consistency`: **pass** after lease `claimed/heartbeat`
  repaired to `2026-08-17 / 2026-08-17`.
- Rust `p4_t05_resource_api` unsealed/caller-header tests: **not-run**
  (`RUST-LINK-DEV-WIN-GNU-01`); same Linux/CI route as D01.

## Checkpoint

- HEAD: `df39b49cbb59960b0b73a26c45008e8a7186dfab`
- Draft PR: https://github.com/agentkernel/cognitive-os/pull/230
- Ubuntu supporting CI run `31998669185`: **fail** — `RUSTFLAGS=-D warnings` dead_code on
  `ResourceApi::new` (production callers now use `with_governance_data_dir`;
  `new` remains test-only). Same fail on windows-latest and linux-002 compile.
- Fix: `#[cfg(test)]` on `ResourceApi::new` at `a317d8171cacba4bbf69270e62aa7e720747c8ee`.

## D03 — exact-revision `DEV-LINUX-NATIVE-01` (`a317d817`)

- Host: `wuz@192.168.1.2` (`hal9000`), rustc 1.97.1.
- Worktree: `/home/wuz/agent-kernel-worktrees/p2-t29-df39b49c` at exact
  `a317d8171cacba4bbf69270e62aa7e720747c8ee` (bundle fetch after GitHub HTTP/2
  fetch failed on the host).
- `cargo test --locked -p kernel-server --test p4_t05_resource_api`: **pass**
  7/7, including `public_remember_accepts_unsealed_payload_and_daemon_composes_headers`
  and `public_remember_rejects_caller_minted_header_on_unsealed_payload`.
- `cargo test --locked -p pi-agent-adapter --test daemon_candidate_protocol`:
  **pass** 14/14, including one WorkspaceSearch extraction, bash still rejected,
  mixed tool+JSON rejected, and two Workspace* calls rejected.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: **pass**.
- `cargo test --workspace --locked`: **pass**, 0 failed. `kernel-server` bin
  **336/336**.
- `cargo fmt --all -- --check`: **pass**.
- Ubuntu supporting CI run `31999496789` `verify (ubuntu-latest)`: **pass**
  (workspace test, Clippy `-D warnings`, handbook). windows-latest pending at
  the time this unit was recorded.
- `B01-Desktop-Linux-002` untouched.

## Acceptance map (product task; not EVAL)

| Acceptance | Evidence |
|---|---|
| Extension advertises WorkspaceSearch/Write/Patch; allowlist empty; bash/edit/write blocked | Local Node 92/92; `workspace-tools.ts` execute I/O-free |
| Adapter `--no-builtin-tools`; one Workspace* maps to P2-T21 candidate shape | linux-002 `daemon_candidate_protocol` 14/14; `PiLaunchPolicy::deepseek_candidate` still `--no-tools` |
| Unsealed public remember; daemon-composed GovernanceSeed headers; caller header 400; sealed path unchanged | linux-002 `p4_t05_resource_api` 7/7 |
| Ubuntu supporting CI + exact-revision linux-002 | Ubuntu `31999496789` pass; linux-002 matrix above |
| Windows | `not-run by owner-directed Linux-only route` for extra native GNU; GitHub `windows-latest` is the workflow dual-platform job |
| No Gate/release/Profile/B01/EVAL/Agent-benefit promotion | claim ceiling hypothesis/non-claim |
