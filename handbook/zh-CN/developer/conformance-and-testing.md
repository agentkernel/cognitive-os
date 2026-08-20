---
doc_id: dev.conformance-testing
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-conformance/src/main.rs
  - path: conformance/README.md
  - path: tools/src/check-consistency.mjs
  - path: tools/src/gen-matrix.mjs
  - path: tools/src/generate-handbook.mjs
  - path: tests/golden/README.md
  - path: tools/src/p2_t28_uj_matrix.mjs
    symbols: ["validateUjCapabilityTruthMatrix"]
tests:
  - tools/test/check.test.mjs
  - tools/test/p2_t28_capability_truth.test.mjs
  - tools/test/c1_c2_paired_p_arm.test.mjs
  - .github/workflows/ci.yml
fingerprint: "sha256:fa3576dafe87d14837215beb718bc3a1a77c5ef0061a5ead86792da5a56ad830"
non_claims:
  - CI 全绿只是工程证据；绝不升格为 Gate、release 或 Profile 声明（公理 A7）。
---

# 符合性与测试

## 测试分类

- **聚焦 failure-first 测试**贴近各 crate（`crates/*/tests/*.rs`、
  `apps/*/tests/*.rs`、`packages/*/src/*.test.ts`），以引入它们的任务命名
  （`p1_t04_…`）。先断言拒绝路径，后断言正常路径。
- **跨语言 golden fixture**（`tests/golden/`）钉住 canonical 编码奇偶。
- **符合性向量**（`conformance/vectors/`，89 个）是合同派生的行为用例，由
  `conformance-runner` 执行。
- **C1/C2 成对测量仪器**（`tools/personal/c1-c2-paired/`）仅用于 campaign：loopback
  纯 Pi credential broker（经 D-Bus 做 Secret Service `get`，禁止 `secret-tool
  lookup`/`search`）、等价 Workspace* fixture adapter、冻结 seeds/`retry=0`，以及
  §2.3 fairness checker。它们不是第二 authority writer，也不升格 Gate、release、
  Profile、B01 或 Agent-benefit。聚焦测试：
  `tools/test/c1_c2_paired_p_arm.test.mjs`。

## 符合性 runner

`cognitive-conformance` 把每个向量归入五态报告
（`pass / fail / not-implemented / not-applicable / skipped`），CI 钉住期望计数，
带证据诚实断言（无法执行的向量不得报 pass），以及 **41 项自检翻转**：故意错误的实
现必须使其向量失败——不会失败的检查器视为已坏。

## 静态一致性

`tools/src/check-consistency.mjs` 强制仓库不变量：schema 合法性（draft 2020-12）、
registry↔schema↔vector 双向引用、Markdown 链接解析、Personal 计划/lease/slice/
Gate 记账形状（含 `lease/personal/EVAL-<id>/…` owner-directed 评测 campaign lease
类别：必须命名已在 Current snapshot 登记的 campaign，且只能拥有
`docs/evaluation/`、`docs/checkpoints/` 与 `docs/plan/PROGRESS.md`）、命令/环境路由
文本、checkpoint-delivery 与 task-atomic 措辞等。
`tools/src/gen-matrix.mjs --check` 保持 `docs/traceability/matrix.yaml` 新鲜。两者
在 CI 与本地（`pnpm run check:consistency`）都运行。手册新增自己的检查器
（`check-handbook.mjs`）与生成器漂移门——见
[`_meta/sync-policy.md`](../../_meta/sync-policy.md)。HTTP 路由生成还会读取
`apps/kernel-server/src/personal/tool_lifecycle.rs`、
`apps/kernel-server/src/personal/pinned_https.rs` 与
`apps/kernel-server/src/personal/observation.rs`，使已标注的 Tool lifecycle、
钉住 HTTPS 与观测平面路径无法腐烂。

## UJ capability-truth 冻结

`tools/src/p2_t28_uj_matrix.mjs` 冻结 BR-08 的 UJ1..UJ6 行。必选行必须命名已存在的
公开调用方文件和机械 oracle 文件，并给出 cleanup 与有界 evidence schema。Web UI 与
Multi-Agent 保持显式 `excluded`，不得标为 required。daemon 侧登记表是
`apps/kernel-server/src/personal/capability_truth.rs`。该冻结不是 EVAL-004、Gate、
release 或 Profile 结果。D02 密闭公开调用方冒烟是
`apps/kernel-server/tests/p2_t28_end_to_end_journey.rs`。命名 UJ oracle 在精确
revision 的 `DEV-LINUX-NATIVE-01` 上执行；Windows GNU 对该 Rust 矩阵记 `not-run`。

## CI 矩阵

`.github/workflows/ci.yml` 的 `verify` 在 Ubuntu 与 Windows MSVC 上运行：pnpm 构建/
测试、cargo build/test（`--test-threads=1`）/clippy（-D warnings）/fmt、codegen 重
生成 diff、consistency、traceability、钉住计数的符合性、错误实现自检、golden
digest 字节奇偶。Rust 验证绝不在已登记不支持的本地 Windows GNU 主机运行；native
Linux 证据只消费已推送的精确 revision。
