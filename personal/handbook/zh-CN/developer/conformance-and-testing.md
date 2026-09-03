---
doc_id: dev.conformance-testing
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: personal/crates/cognitive-conformance/src/main.rs
  - path: core/conformance/README.md
  - path: tools/src/check-consistency.mjs
  - path: tools/src/gen-matrix.mjs
  - path: tools/src/generate-handbook.mjs
  - path: core/tests/golden/README.md
  - path: tools/src/p2_t28_uj_matrix.mjs
    symbols: ["validateUjCapabilityTruthMatrix"]
  - path: tools/src/p7_t05_web_ui_inventory.mjs
    symbols: ["validateWebUiRouteInventory"]
  - path: tools/src/personal-rc-gate.mjs
    symbols: ["buildPersonalRcDeclarationReport"]
tests:
  - tools/test/check.test.mjs
  - tools/test/p2_t28_capability_truth.test.mjs
  - tools/test/c1_c2_paired_p_arm.test.mjs
  - tools/test/p7_t05_web_ui_inventory.test.mjs
  - tools/test/personal-rc-gate.test.mjs
  - .github/workflows/ci.yml
fingerprint: "sha256:e3d039a3805aef4d754eb3f3d78abe6a064d22b0afdbebd5452f5e666f3876c6"
non_claims:
  - CI 全绿只是工程证据；绝不升格为 Gate、release 或 Profile 声明（公理 A7）。
---

# 符合性与测试

## 测试分类

- **聚焦 failure-first 测试**贴近各 crate（`crates/*/tests/*.rs`、
  `apps/*/tests/*.rs`、`packages/*/src/*.test.ts`），以引入它们的任务命名
  （`p1_t04_…`）。先断言拒绝路径，后断言正常路径。
- **跨语言 golden fixture**（`core/tests/golden/`）钉住 canonical 编码奇偶。
- **符合性向量**（`core/conformance/vectors/`，89 个）是合同派生的行为用例，由
  `conformance-runner` 执行。
- **C1/C2 成对测量仪器**（`tools/personal/c1-c2-paired/`）仅用于 campaign：loopback
  纯 Pi credential broker（经 D-Bus 做 Secret Service `get`，禁止 `secret-tool
  lookup`/`search`）、等价 Workspace* fixture adapter、冻结 seeds/`retry=0`，以及
  §2.3 fairness checker（`system_task_prompt_bytes` 取自
  `frozen-system-task-prompt.txt` 的字节长度，而不是共享占位符；live P/O
  `--append-system-prompt` 命令清单共用该文件）。Live `runLivePairedCell` 必须注入
  `executeArm`（禁止意外 spawn）；`counted_sample` 仅在冻结 b1/b2 且 fairness 通过、
  两臂均 exit 0 且未超时时报 true。Dry-run 不得标为 counted。P-arm `WorkspacePatch` 的
  `input_b64` 是 UTF-8 unified diff（`workspace_patch_payload: unified-diff`），
  replacement bytes 失败闭合。它们不是第二
  authority writer，也不升格 Gate、release、
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
`docs/evaluation/`、`docs/checkpoints/` 与 `docs/plan/PROGRESS.md`；以及
`lease/personal/GOV-<id>/…` owner-directed 治理交付 lease 类别（ADR-0055）：
描述必须命名与 lease id 相同且已在 Current snapshot 登记的 `GOV-<id>`，且只能拥有
`docs/governance/`、`docs/adr/`、`docs/plan/PROGRESS.md`、lease 语法检查器面
（`tools/src/check-consistency.mjs`、`tools/test/check.test.mjs`）与
`personal/handbook/` 下被映射的手册页面；以及 `lease/personal/DOC-<id>/…`
owner-directed 文档对齐 lease 类别：描述必须命名与 lease id 相同且已在 Current
snapshot 登记的 `DOC-<id>`，且只能拥有精确的 plan/product/architecture/handbook/design
文档、`AGENTS.md`、`.cursor/rules/`、本交付自己的带日期 `docs/checkpoints/`
report/closure 文件——不得整目录——以及同一 lease 语法检查器面）、命令/环境路由
文本、checkpoint-delivery 与 task-atomic 措辞等。
`tools/src/gen-matrix.mjs --check` 保持 `docs/traceability/matrix.yaml` 新鲜。两者
在 CI 与本地（`pnpm run check:consistency`）都运行。

自 `P0-T09` 起，`check-consistency.mjs` 与 `check-agent-rules.mjs` 的路径存在性由
`git ls-files` 决定而不是文件系统：已提交文档或规则链接到只存在于作者工作树的文件时，
本机即以 CI 相同的信息红灯（`… (exists locally but is not tracked by Git)`），且未跟踪的
本地 Markdown 不参与扫描，本机与 CI 结论一致。不在 Git checkout 内时一致性检查器
fail closed（`TRACKED_PATHS_UNAVAILABLE`）；规则检查器仅为其 focused fixture 回退到
文件系统并明示模式（`path existence = …`）。owner 本机未跟踪的编辑器资产
（`.cursor/skills/`、`.cursor/commands/`、规则 30/40、`.cursor/mcp.json`）保持
"缺失告警 / 存在严格检查"。同一检查器还解析 `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`
的 Phase 13 建造顺序 mermaid 与 `personal/docs/architecture/personal-2.0.0-dev-prep-index.md`
中的副本，要求边集合完全相等（含实线/虚线；节点 id 去掉 `P13`/`P11` 前缀后比较）——
缺边/多边即 `BUILD_ORDER_EDGE_MISSING` / `BUILD_ORDER_EDGE_EXTRA`；正式计划为权威，
绝不为迎合索引反向修改。

手册新增自己的检查器
（`check-handbook.mjs`）与生成器漂移门——见
[`_meta/sync-policy.md`](../../_meta/sync-policy.md)。HTTP 路由生成还会读取
`personal/apps/kernel-server/src/personal/tool_lifecycle.rs`、
`personal/apps/kernel-server/src/personal/pinned_https.rs` 与
`personal/apps/kernel-server/src/personal/observation.rs`，使已标注的 Tool lifecycle、
钉住 HTTPS 与观测平面路径无法腐烂。

## UJ capability-truth 冻结

`tools/src/p2_t28_uj_matrix.mjs` 冻结 BR-08 的 UJ1..UJ6 行。必选行必须命名已存在的
公开调用方文件和机械 oracle 文件，并给出 cleanup 与有界 evidence schema。Web UI 与
Multi-Agent 保持显式 `excluded`，不得标为 required。daemon 侧登记表是
`personal/apps/kernel-server/src/personal/capability_truth.rs`。该冻结不是 EVAL-004、Gate、
release 或 Profile 结果。D02 密闭公开调用方冒烟是
`personal/apps/kernel-server/tests/p2_t28_end_to_end_journey.rs`。命名 UJ oracle 在精确
revision 的 `DEV-LINUX-NATIVE-01` 上执行；Windows GNU 对该 Rust 矩阵记 `not-run`。

## P7-T05 Web UI 路由清单

`tools/src/p7_t05_web_ui_inventory.mjs` 冻结 UI 能力到既有 daemon 路由的映射。
声称不存在的路由、通用 lifecycle 转换、Task 通道上的 secret-bearing 写、以及
浏览器直连 SQLite/SecretStore/文件系统/Provider 都会 fail closed。缺失的 typed
HTTP（Task cancel、Agent pause/resume/stop/restart/quarantine）必须记
`unavailable`/`not-run`。该清单不是 SPA 实现、浏览器旅程、Gate 或 release 结果。

## Personal Linux RC 声明合成器

`tools/src/personal-rc-gate.mjs` 把既有 MVP Gate 结论与可运维性证据绑定成 digest
绑定的 Personal Linux RC 声明。不完整 observation、缺失 digest、Profile 键、启用
P6、RC 范围关键风险非 0、以及生产 GitHub Release 声明都会 fail closed。求值器不设置
Gate 或 Profile 状态。聚焦测试：`tools/test/personal-rc-gate.test.mjs`。

## CI 矩阵

`.github/workflows/ci.yml` 的 `verify` 在 Ubuntu 与 Windows MSVC 上运行：pnpm 构建/
测试、cargo build/test（`--test-threads=1`）/clippy（-D warnings）/fmt、codegen 重
生成 diff、consistency、traceability、agent 规则引用检查
（`tools/src/check-agent-rules.mjs`：`AGENTS.md`、`.cursor/rules`、`.cursor/commands`
的 frontmatter 与路径/skill/命令引用）、handbook 检查、钉住计数的符合性、错误实现
自检、golden digest 字节奇偶。Rust 验证绝不在已登记不支持的本地 Windows GNU 主机运行；native
Linux 证据只消费已推送的精确 revision。
