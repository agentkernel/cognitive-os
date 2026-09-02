---
doc_id: ai.docs-impact
locale: zh-CN
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: personal/handbook/_meta/source-map.json
  - path: docs/standards/docs-sync-contract.md
  - path: tools/src/docs-sync-gate.mjs
    symbols: ["routeChangedPaths", "decideDocsSync"]
fingerprint: "sha256:840ad92cf83ca04b31e1be4a2d6a4e3cf7426b40a53f3e972ad226defe029f64"
non_claims:
  - 本页是 docs-sync 契约面向手册的适配；旧文档义务仍由契约本身拥有。
---

# 文档影响

文档同步是**强制的 commit/push/merge 前置义务**（契约 §2），不是善意建议。机器门为
`node tools/src/docs-sync-gate.mjs --staged|--push|--range`：它把改动路径经 source
map 路由，命中即运行手册检查集，并使"映射源改动而无手册更新"的变更集失败。每个克隆
运行一次 `pnpm run hooks:install` 启用仓库 hooks。确实无文档影响的变更，唯一逃生口是
`DOCS_IMPACT_NONE="<具体理由>"`，且同一理由必须记入 commit/PR 描述。

在完成变更**之前**，用
[`personal/handbook/_meta/source-map.json`](../../_meta/source-map.json) 判定文档影响：

1. 将每个改动路径与 source-map 规则匹配，收集映射的 `doc_ids`。
2. 对每个映射页面：手写页通过编辑页面并刷新指纹更新
   （`node tools/src/fill-handbook-fingerprints.mjs`）；生成页只能由
   `node tools/src/generate-handbook.mjs` 刷新。
3. 用户可见行为（CLI 动词/参数、配置文件、错误面、安装、恢复、安全边界）必须同步
   user 与 reference 树；架构、数据、协议、authority 或测试环境变化必须同步
   developer 与 AI 树。
4. 全新 tracked 文件必须在
   [`source-coverage.json`](../../_meta/source-coverage.json) 中归类，否则手册检查
   失败。
5. 若变更确实不影响任何文档，PR 描述必须写明具体的
   `docs-impact: none — <理由>`，不得沉默跳过。
6. 受影响文档与代码属于同一正式任务、同一 PR。

## Personal 2.0 语义路由

source map 有意让已采纳目标无法静默变化：

- `personal-2-baseline` 路由 canonical Personal 2.0 产品与架构基线；
- `personal-2-desktop-account-hub` 路由 ADR-0055/0056 以及 Account Hub、Provider、
  Web UI 产品/架构来源；
- `personal-2-agent-supervision` 路由双语 Agent 对话设计及 Agent Shell/adapter/
  多 Agent/recovery/learning 架构；
- `personal-2-mcp-family` 路由 ADR-0037/0057 以及双语 MCP 资源族、认知资源与 Resource
  Manager 来源。
- `personal-2-opc-rebaseline` 路由 ADR-0059、Phase 11 与 Phase 12 formal/support/environment、
  focused Project/Conversation/Windows/Routine 产品与架构章，以及 current client OPC
  design corpus。Phase 12 在既有正式计划内登记冻结 prototype 接到 daemon `/ui/`
  的功能完备（非像素复制、非 2.1、非 T15）。**Phase 13**（`P13-T01`–`T13`，
  2026-09-02）同样登记在既有正式计划内：walking skeleton → 原型程度 + 设计目标；
  `P11-T15` 验收前置改为 Phase 13 done + 资格化 Windows；不是 release / signing /
  B01-W。
- `personal-2-opc-v9-implementation-mapping` 路由已定档 Personal 2.0.0 Scene →
  daemon 映射（`personal/docs/architecture/personal-2.0-opc-v9-implementation-mapping.md`；
  历史文件名与规则 id 含 v9，不是产品版本）
  到 `dev.architecture-overview` 与本页。仅 informative；canvas v9 是冻结设计原型，
  不是产品。daemon `/ui/` Dual Track hash 在 `P12-T01`–`T09` 收口后是 Now /
  hypothesis chrome。架构章节正文不再把 Team/Inbox 写成 2.0.0 一级（`DOC-P12-DEBT`）。
  权威仍是 P11 walking skeleton。不是 Gate、release 或 T15 领取。
- `personal-2-0-0-dev-prep` 路由 Personal 2.0.0 开发前期索引
  （`personal/docs/architecture/personal-2.0.0-dev-prep-index.md`）到
  `dev.architecture-overview` 与本页。计划卡已于 2026-08-30 对齐；Phase 12
  `P12-T01`–`T09` 于 2026-09-01 done（merged PR [#302](https://github.com/agentkernel/cognitive-os/pull/302)）；
  Phase 13 建造顺序与 2026-09-02 差距核对已写入该索引；
  仅文档；不是实现、Gate 或 T15 领取。OPC 设计语料（`clients/docs/design/opc-2.0/`，
  含维护索引）收录 2026-08-30 设计 Agent / Owner 旅程难点研判
  （[`13-personal-20-agent-design-difficulty-and-journey-assessment.md`](../../../../clients/docs/design/opc-2.0/13-personal-20-agent-design-difficulty-and-journey-assessment.md)；
  hypothesis；不是 Gate）。

每次命中都必须保留：**当前 Linux 1.0/当前 API**、**已采纳 Windows OPC target**、
**Requires-backend** 与 **Requires-environment/deferred**。绝不能从 design adoption、
Canvas、ordinary CI、Linux、WSL 或 Windows GNU evidence 推断 Project/Employee、
Personal Conversation/Vault/Memory、Pi Assistant、隐藏托管 DSH、Routine/HITL 画布、
binding/诚实 usage、OPC UI、X connector 或 fixed N=15 acceptance 已实现。MCP 是 advanced
deferred target；native mobile/E2E relay remote 属于 2.1。

本地化 canonical 设计文件路由到相同的双语 handbook `doc_id`。必须同步两个 locale。
只有在全部映射源存在后才刷新手写页指纹；不得为掩盖指纹漂移而手改或重生成无关生成页。

任何影响文档的变更的验证集：

```powershell
node tools/src/docs-sync-gate.mjs --staged   # 或 --push / --range
node tools/src/generate-handbook.mjs --check
node tools/src/check-handbook.mjs
pnpm run check:consistency
git diff --check
```

旧文档（`docs/**`）继续遵循
[`docs-sync-contract.md`](../../../../docs/standards/docs-sync-contract.md) 的自身义务；手册绝不吸收或替代
它们。canonical 来源与手册页冲突时，在同一交付中修正手册页——绝不为迎合文档去"修"
canonical 来源。
