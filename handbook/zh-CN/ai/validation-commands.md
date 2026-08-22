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
fingerprint: "sha256:a54dc671076b25a76d1b42b3e5defea840c135af4d6cadf51671e7ca578663bd"
non_claims:
  - 命令可用不等于证据；只有实际执行的检查才算数，且本地结果绝不升格 Gate/release/Profile 声明。
---

# 验证命令

环境路由是前置条件，由
[`PERSONAL-TEST-ENVIRONMENTS.md`](../../../docs/plan/PERSONAL-TEST-ENVIRONMENTS.md) 拥有。

## 全平台安全（含 Windows GNU 主机）

```powershell
pnpm install --frozen-lockfile
pnpm -r build
pnpm -r test
pnpm run check:consistency          # tools/src/check-consistency.mjs
node tools/src/gen-matrix.mjs --check
node tools/src/check-handbook.mjs   # 手册防漂移门
node tools/src/generate-handbook.mjs --check
node tools/src/docs-sync-gate.mjs --staged   # commit 前文档同步门（--push / --range）
pnpm run hooks:install              # 每克隆一次：注册 .githooks pre-commit/pre-push
cargo fmt --all -- --check          # 仅格式化；不触发链接
git diff --check
node --test tools/test/c1_c2_paired_p_arm.test.mjs  # P9-T12 live paired executor + unified-diff WorkspacePatch；不是 Gate 证据
```

## 必须走受支持 CI（Ubuntu / Windows MSVC）或 exact-revision native Linux

```bash
cargo build --workspace --locked
cargo test --workspace --locked -- --test-threads=1
cargo test -p kernel-server tool_executor --locked -- --test-threads=1
cargo test -p kernel-server p4_t05_resource_api --locked -- --test-threads=1
cargo test -p kernel-server --test p8_t12_resource_manager --locked -- --test-threads=1
cargo test -p pi-agent-adapter --locked -- --test-threads=1
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run -p cognitive-conformance --bin conformance-runner
cargo run -p cognitive-contracts --bin contracts-codegen   # 之后 diff 生成目录
```

绝不在本地 Windows GNU 主机运行上述命令：linker 失败（exit 121）是已登记的环境边界，
不是需要复现的信号。远程/native 验证只消费已推送的不可变 revision——绝不复制工作树。
P2-T25 的聚焦 HTTP 覆盖在 `apps/kernel-server/tests/p2_t25_tool_lifecycle.rs`
（lifecycle、selection 与钉住 HTTPS origin 登记表）。
P2-T26 的聚焦 HTTP 覆盖在 `apps/kernel-server/tests/p2_t26_observation_plane.rs`
（O2/O3/O4/O5/O13 观测平面、受控零值、审计游标负例与通道负例）。
P2-T27 的聚焦 HTTP 覆盖在 `apps/kernel-server/tests/p2_t27_backup_restore.rs`
（排除 secret 的 backup/restore、预检、篡改与 task 通道拒绝）。
P2-T28 D01 冻结是 `tools/test/p2_t28_capability_truth.test.mjs`，对照
`tools/fixtures/p2_t28_uj_matrix.json`（已存在的公开调用方/oracle；Web UI/
Multi-Agent 保持显式 `excluded`）。对应 daemon 登记表是
`apps/kernel-server/src/personal/capability_truth.rs`（仅 Linux/CI）。
P2-T28 D02 公开调用方冒烟是 `apps/kernel-server/tests/p2_t28_end_to_end_journey.rs`。
P2-T28 D03 精确 revision 的 `DEV-LINUX-NATIVE-01` 聚合运行命名 UJ oracle，以及
`cargo test -p kernel-server --bins`、`cargo test --workspace`、
`cargo fmt --all -- --check` 和 `cargo clippy --workspace --all-targets -- -D warnings`。
P2-T30 公开 admit 调度 lease 覆盖是 kernel-server 聚焦测试
`public_admit_c1_search_leaves_draft_only_until_scheduler_acquires_lease`
（仅 Linux/CI；Windows GNU `not-run`）。
P2-T33 私有 candidate 主机路径覆盖是
`apps/admin-cli/tests/p2_t33_private_candidate_host_path.rs` 以及
`provider_proxy` 的私有 candidate 单元测试（仅 Linux/CI；Windows GNU `not-run`）。
P8-T12 Resource Manager 覆盖是 `apps/kernel-server/tests/p8_t12_resource_manager.rs`
（management list/inspect/mutate、task 通道 403、generic create 拒绝；仅 Linux/CI；
Windows GNU `not-run`）。

## CI 在每个 PR 上强制什么

[`ci.yml`](../../../.github/workflows/ci.yml) 的 `verify` 矩阵（Ubuntu + Windows
MSVC）：TypeScript 构建/测试、Rust 构建/测试/clippy/fmt、codegen 漂移 diff、
consistency 检查、traceability 新鲜度、带固定五态计数与证据诚实断言的符合性 runner、
错误实现自检、跨语言 golden digest 字节一致。

## 已知过期入口

`pnpm run verify:local`（V01 编排器）钉住了过期的符合性计数，在本基线不是可用的本地
门；请改用上面的单项命令。
