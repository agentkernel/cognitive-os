---
doc_id: dev.repo-map
locale: zh-CN
kind: reference
audience: [developer, ai]
status: implemented
generated: false
sources:
  - path: Cargo.toml
  - path: pnpm-workspace.yaml
  - path: package.json
fingerprint: "sha256:46b917bfe579dbf7c271b70078934ab94e4005fe0d05477789e247eb1a021ffe"
non_claims:
  - 目录存在不是实现或 Gate 证据；接线状态见执行链状态页。
---

# 仓库地图

| 树 | 内容 | 变更纪律 |
|---|---|---|
| `crates/` | 十个 Rust crate（contracts、domain、kernel、store、runtime、management、secret、provider-transport、akp、conformance） | 实现面；kernel 不得引入 HTTP/SQLite/模型 SDK |
| `apps/` | `kernel-server`（daemon）、`admin-cli`（两个二进制）、`pi-agent-adapter`、`agent-shell`（TS 库）、`cognitiveos-console`（deprecated 存根） | 实现面 |
| `packages/` | `pi-cognitiveos`（Pi 扩展）、`sdk-ts`、`contracts-ts` | 实现面；`*/src/generated/` 归生成器所有 |
| `specs/` | 需求/错误/状态域注册表、74 个 JSON schema、5 张转移表、叙述伴随文档 | 架构合同——只走 Lane-CTR；绝不为迎合代码改写 |
| `conformance/` | 89 个向量 + README | 合同资产——同等保护 |
| `tests/` | baseline/e2e/faults/security 索引 + `tests/golden/` 跨语言 fixture | golden JSON 为生成物 |
| `tools/` | Node 检查器/生成器（consistency、traceability、Gate 评估器、handbook、docs-sync 门） | 由 `@cognitiveos/repo-tools` 语法检查与测试 |
| `.githooks/` | 仓库内置 pre-commit/pre-push 文档同步 hooks（opt-in：`pnpm run hooks:install`） | `tools/src/docs-sync-gate.mjs` 之上的薄 `sh` 包装 |
| `docs/` | 治理、正式计划 + 当前快照 + lease 台账、产品/架构设计、ADR、标准、checkpoint、prompt | canonical 文档系统；手册只链接、绝不编辑 |
| `handbook/` | 本双语派生文档系统 | 由 `tools/src/check-handbook.mjs` 校验 |
| `deploy/` | 可检查安装模板 + systemd unit 模板 | 由 campaign 构建器渲染 |
| `scripts/` | V01 auto-run 编排器（pin 过期；非当前门） | 历史 |
| `History/` | 冻结归档 | 绝不读取或引用 |

根文件：`Cargo.toml`（workspace + 共享 lint）、`package.json`（pnpm 脚本）、
`pnpm-workspace.yaml`、`rust-toolchain.toml`（钉住 1.97.1）、`AGENTS.md`（代理入
口）、`LICENSE`/`NOTICE`（Apache-2.0；不再分发 Pi 与 Node）、`llms.txt`（AI 指针）。
白皮书、评审与 RFC-0001 在 `docs/architecture/cognitiveos/`（`FROZEN_DOCS`）。研究
任务卡细节是 `docs/plan/plan.md`。owner 分析笔记在 `docs/research/`（informative，
不是 backlog）。绝不引用 `History/`。

依赖方向（由 crate manifest 强制）：
`contracts → domain → kernel → {store, management, runtime} → apps`；
`secret`/`provider-transport`/`akp` 为叶子工具，`conformance` 消费全部用于行为门。
