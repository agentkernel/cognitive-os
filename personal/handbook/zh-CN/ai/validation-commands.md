---
doc_id: ai.validation-commands
locale: zh-CN
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: package.json
  - path: tools/package.json
  - path: tools/src/generate-handbook.mjs
  - path: .github/workflows/ci.yml
  - path: docs/plan/PERSONAL-TEST-ENVIRONMENTS.md
    symbols: ["COMMAND-SHELL-PS51", "RUST-LINK-DEV-WIN-GNU-01"]
  - path: docs/bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md
  - path: tools/src/p7_t05_web_ui_inventory.mjs
    symbols: ["validateWebUiRouteInventory"]
  - path: tools/src/personal-rc-gate.mjs
    symbols: ["buildPersonalRcDeclarationReport"]
fingerprint: "sha256:d8b50ce4000fe4a869d41112a3799c7398d3b70e150328e72e1fcf840f0e2213"
non_claims:
  - 命令可用不等于证据；只有实际执行的检查才算数，且本地结果绝不升格 Gate/release/Profile 声明。
---

# 验证命令

P11-T08 聚焦 store/HTTP 测试路由到 `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` 或 exact-revision `DEV-LINUX-NATIVE-01`。`DEV-WIN-GNU-01` cargo link 为 `not-run`。clock/sleep/restart 宿主 E2E 为 `not-run`。

环境路由是前置条件，由
[`PERSONAL-TEST-ENVIRONMENTS.md`](../../../../docs/plan/PERSONAL-TEST-ENVIRONMENTS.md) 拥有。

Personal 2.0 Phase 11（Personal 2.0.0 chrome）把 T03/T04 日常权威测试路由到
`CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01`（需要 native daemon/store 时加 exact-revision
`DEV-LINUX-NATIVE-01`）。host/DSH 原生 E2E 仍是 future qualified
`DEV-WINDOWS-NATIVE-OPC-01`（未 provision 则为 `Requires-environment` / `not-run`）。
unparked 的 T15 仍需同一 qualified Windows revision 上的 preregistered N=15
acceptance，不是 Phase 12 mutex；自 2026-09-02 起其验收前置 = Phase 13
`P13-T02`–`T13` done + `P13-T13` 资格化 Windows。Phase 12 Dual Track UI 用
`DEV-WIN-GNU-01` TS 加 required CI。Phase 13 卡的验证路由见
`PERSONAL-TEST-ENVIRONMENTS.md` §5.2（Linux native 只关闭"实现已存在"，Windows
原生格由 P13-T13 回填）。native environment 与 B01-W 尚未 provision；`B01-DESKTOP-002` 不是
2.0 日常默认机。本地 Windows GNU、WSL、ordinary CI 与 Canvas 都不能替代
Gate/release；可用前 native cell 记 `not-run`。`not-run` 永远不是 pass。

## 全平台安全（含 Windows GNU 主机）

```powershell
pnpm install --frozen-lockfile
pnpm -r build
pnpm -r test
pnpm run check:consistency          # tools/src/check-consistency.mjs（链接/路径只认 git tracked；含 Phase 13 建造顺序边集合比对）
node tools/src/gen-matrix.mjs --check
node tools/src/check-handbook.mjs   # 手册防漂移门
node tools/src/generate-handbook.mjs --check
node tools/src/docs-sync-gate.mjs --staged   # commit 前文档同步门（--push / --range）
pnpm run check:rules                # tools/src/check-agent-rules.mjs：AGENTS.md / .cursor/rules / .cursor/commands 引用与 frontmatter（路径只认 git tracked；本地专用资产缺失时告警）
pnpm run hooks:install              # 每克隆一次：注册 .githooks pre-commit/pre-push
# bash 主机（Cloud Agent / Linux）可一次性引导：
#   bash scripts/setup-dev-env.sh   # 依赖 + 钉住的工具链 + docs-sync hooks
cargo fmt --all -- --check          # 仅格式化；不触发链接
git diff --check
node --test tools/test/p7_t05_web_ui_inventory.test.mjs  # P7-T05 路由清单；不是 Gate 结果
node --test tools/test/personal-rc-gate.test.mjs         # P7-T06 RC 合成器；不设置 Gate 状态
# SPA（本仓 clients/pc/web）：pnpm test；pnpm build
# 产品源是 daemon GET /ui（将 dist/ 复制到 data_dir()/ui）。Vite preview 不是产品源。
# linux-002 部署 Control Plane / dsh 后，owner 在本机 Windows 经 SSH 转发查看：
#   ssh -J wuz@192.168.1.2 -L 48681:127.0.0.1:48681 -L 3080:127.0.0.1:3080 hal9001@192.168.123.160
# 然后打开 http://127.0.0.1:48681/ui/ 与 http://127.0.0.1:3080/。
# daemon 重启后必须重启 cognitive dsh web；apply 不能恢复新 daemon 的 INACTIVE dsh 状态。
```

## 必须走受支持 CI（Ubuntu / Windows MSVC）或 exact-revision native Linux

```bash
cargo build --workspace --locked
cargo test --workspace --locked -- --test-threads=1
cargo test -p kernel-server tool_executor --locked -- --test-threads=1
cargo test -p kernel-server p4_t05_resource_api --locked -- --test-threads=1
cargo test -p kernel-server --test p8_t12_resource_manager --locked -- --test-threads=1
cargo test -p kernel-server --test p8_t13_provider_control_plane --locked -- --test-threads=1
cargo test -p cognitive-secret --test p8_t13_endpoint_trust --locked -- --test-threads=1
cargo test -p cognitive-store --test p8_t13_provider_store --locked -- --test-threads=1
cargo test -p pi-agent-adapter --locked -- --test-threads=1
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run -p cognitive-conformance --bin conformance-runner
cargo run -p cognitive-contracts --bin contracts-codegen   # 之后 diff 生成目录
```

绝不在本地 Windows GNU 主机运行上述命令：linker 失败（exit 121）是已登记的环境边界，
不是需要复现的信号。`CLOUD-AGENT-LINUX-01` 可以运行整段命令——它是 native GNU/Linux
link 主机——但其结果只是 container 级的 pre-CI 排查，绝不替代 required CI 或
exact-revision native 证据。远程/native 验证只消费已推送的不可变 revision——绝不复制工作树。
P2-T25 的聚焦 HTTP 覆盖在 `personal/apps/kernel-server/tests/p2_t25_tool_lifecycle.rs`
（lifecycle、selection 与钉住 HTTPS origin 登记表）。
P2-T26 的聚焦 HTTP 覆盖在 `personal/apps/kernel-server/tests/p2_t26_observation_plane.rs`
（O2/O3/O4/O5/O13 观测平面、受控零值、审计游标负例与通道负例）。
P2-T27 的聚焦 HTTP 覆盖在 `personal/apps/kernel-server/tests/p2_t27_backup_restore.rs`
（排除 secret 的 backup/restore、预检、篡改与 task 通道拒绝）。
P2-T28 D01 冻结是 `tools/test/p2_t28_capability_truth.test.mjs`，对照
`tools/fixtures/p2_t28_uj_matrix.json`（已存在的公开调用方/oracle；Web UI/
Multi-Agent 保持显式 `excluded`）。对应 daemon 登记表是
`personal/apps/kernel-server/src/personal/capability_truth.rs`（仅 Linux/CI）。
P2-T28 D02 公开调用方冒烟是 `personal/apps/kernel-server/tests/p2_t28_end_to_end_journey.rs`。
P2-T28 D03 精确 revision 的 `DEV-LINUX-NATIVE-01` 聚合运行命名 UJ oracle，以及
`cargo test -p kernel-server --bins`、`cargo test --workspace`、
`cargo fmt --all -- --check` 和 `cargo clippy --workspace --all-targets -- -D warnings`。
P2-T30 公开 admit 调度 lease 覆盖是 kernel-server 聚焦测试
`public_admit_c1_search_leaves_draft_only_until_scheduler_acquires_lease`
（仅 Linux/CI；Windows GNU `not-run`）。
P2-T33 私有 candidate 主机路径覆盖是
`personal/apps/admin-cli/tests/p2_t33_private_candidate_host_path.rs` 以及
`provider_proxy` 的私有 candidate 单元测试（仅 Linux/CI；Windows GNU `not-run`）。
P8-T12 Resource Manager 覆盖是 `personal/apps/kernel-server/tests/p8_t12_resource_manager.rs`
（management list/inspect/mutate、task 通道 403、generic create 拒绝；仅 Linux/CI；
Windows GNU `not-run`）。
P8-T13 Provider Control Plane 覆盖是
`personal/apps/kernel-server/tests/p8_t13_provider_control_plane.rs`、
`personal/crates/cognitive-secret/tests/p8_t13_endpoint_trust.rs` 与
`personal/crates/cognitive-store/tests/p8_t13_provider_store.rs`
（端点信任/SSRF 负例、目录保留、Pi 与 dsh binding 隔离；仅 Linux/CI；Windows GNU
`not-run`）。
P7-T05/D01 Web UI 路由清单是
`tools/test/p7_t05_web_ui_inventory.test.mjs`，对照
`personal/docs/architecture/web-ui-route-inventory.json`（伪造 lifecycle、缺失
daemon 路由、Task 通道 secret、Web storage、浏览器直连目标 fail closed）。它不是
Gate 或 release 结果。daemon Origin/Referer 与 `GET /ui`
静态服务测试在 `personal/apps/kernel-server/src/personal/server.rs`（外来/null Origin、
缺失 bundle 的 `not_available`、路径穿越）；需要受支持的 Rust 链接
（CI-UBUNTU-01 / CI-WINDOWS-MSVC-01 / DEV-LINUX-NATIVE-01）。
P7-T06 Personal Linux RC 合成器是
`tools/test/personal-rc-gate.test.mjs`（不完整 observation、Profile 键、启用 P6、
生产发布 fail closed）。它不设置 Gate 状态。
P7-T05/D08 binding CAS 是 `POST /management/agent-bindings` 上的
`expected_revision`，不匹配时 409 `PROVIDER_BINDING_REVISION_STALE`
（`personal/apps/kernel-server/tests/p8_t13_provider_control_plane.rs`；仅 Linux/CI）。
P11-T12 诚实 usage 是 `personal/crates/cognitive-store/tests/p11_t12_honest_usage.rs`
以及 `personal/apps/kernel-server/src/personal/provider_control_plane.rs` 内
进程 HTTP 测试（unknown≠0、labelled actual|estimated|unknown、静默 rebind
`PROVIDER_SILENT_REBIND_REJECTED`、省略 secret；仅 Linux/CI；Windows GNU `not-run`）。
SPA 单元测试在外部 clients checkout `pc/web`（`vitest run`）；它们不是 kernel CI，
也不是 live SecretStore 证明。

## CI 在每个 PR 上强制什么

[`ci.yml`](../../../../.github/workflows/ci.yml) 的 `verify` 矩阵（Ubuntu + Windows
MSVC）：TypeScript 构建/测试、Rust 构建/测试/clippy/fmt、codegen 漂移 diff、
consistency 检查、traceability 新鲜度、agent 规则引用检查（`check-agent-rules`）、
handbook 检查与生成页字节比对、带固定五态计数与证据诚实断言的符合性 runner、
错误实现自检、跨语言 golden digest 字节一致。

## 已知过期入口

`pnpm run verify:local`（V01 编排器）钉住了过期的符合性计数，在本基线不是可用的本地
门；请改用上面的单项命令。
