# 20260721 Lane-CTR Handoff（M6 Batch-0A：schema bindings + 冻结裁决）

## 1. 本次会话完成

- **codegen CORE_SET 33→38**：纳入既有 schema（零 schema/registry/vector 语义变更）：
  - `agent-package-manifest` / `agent-installation` / `agent-compatibility-report`
  - `performance-report` / `profile-manifest`
- **GENERATOR_VERSION 0.2.0→0.2.1**：新增 JSON Schema `number`→`f64` 与 `type: ["number","integer"]` widen-to-f64（performance-report 所需）。
- 生成绑定：Rust/TS **SCHEMA_DIGESTS 35→40**；双语言 M6 smoke 测试。
- **冻结裁决**写入台账：
  - **D-020**：不新增 installation transition table；按 companion prose 状态序列实现；可用错误码 = `AGENT_PACKAGE_VERIFICATION_FAILED` / `STATE_CONFLICT` / `AGENT_COMPATIBILITY_DEGRADED`。
  - **D-021**：不新增 readiness carrier；证据归 milestone e2e/fault，不得写 Profile conformance。

## 2. 未完成 / 进行中

- Lane-RUN Batch-1：篡改拒装 tracer（消费本批绑定）。
- KRN 安装事务端口（WP2）待 RUN 消费面后协作。
- CFR 行为执行 AGENT-INSTALL/BYPASS/OOB 仍 not-run。

## 3. 测试与证据状态

- `cargo test -p cognitive-contracts --test generated_types`：8/8 pass（含 M6 bindings）。
- `pnpm --filter @cognitiveos/contracts-ts build && test`：38 pass。
- 状态用语：绑定**实现已提供** + 车道测试已执行；**不构成** Profile 符合；向量 pins 未改（52/32）。

## 4. 给下游的消费边界

1. RUN/KRN：**禁止**声称消费 installation transition 机器表（D-020）。
2. RUN readiness：**禁止**新增/宣称 registry readiness 合同（D-021）。
3. 生成模块：`cognitive_contracts::generated::{agent_package_manifest, agent_installation, agent_compatibility_report, performance_report, profile_manifest}`（TS 同名 kebab 文件）。

## 5. 下一步入口

- 提示词：`docs/prompts/m6-batch1-installer.md`（Lane-RUN）
- 分支：干净 worktree `lane/run` ← 本 PR 合入后的 `origin/main`

## 6. 快照

- PROGRESS：本批随 docs 更新 CTR 行。
- 关联：REQ-AGENT-INSTALL-001/002、REQ-AGENT-COMPAT-001、REQ-PERF-004、REQ-CONF-001/003、IMP-01、D-020、D-021。
