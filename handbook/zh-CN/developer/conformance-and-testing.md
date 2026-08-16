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
tests:
  - tools/test/check.test.mjs
  - .github/workflows/ci.yml
fingerprint: "sha256:23373611a640fdfca4dfb3a1746f4e535a6b69028ce44a9cfb3f25efe36a1e54"
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

## CI 矩阵

`.github/workflows/ci.yml` 的 `verify` 在 Ubuntu 与 Windows MSVC 上运行：pnpm 构建/
测试、cargo build/test（`--test-threads=1`）/clippy（-D warnings）/fmt、codegen 重
生成 diff、consistency、traceability、钉住计数的符合性、错误实现自检、golden
digest 字节奇偶。Rust 验证绝不在已登记不支持的本地 Windows GNU 主机运行；native
Linux 证据只消费已推送的精确 revision。
