---
doc_id: ai.docs-impact
locale: zh-CN
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: handbook/_meta/source-map.json
  - path: docs/standards/docs-sync-contract.md
fingerprint: "sha256:8850171a249edefcb02838b5f515e05567defc06dcbd2fe3b2e6320ccfb96481"
non_claims:
  - 本页是 docs-sync 契约面向手册的适配；旧文档义务仍由契约本身拥有。
---

# 文档影响

在完成变更**之前**，用
[`handbook/_meta/source-map.json`](../../_meta/source-map.json) 判定文档影响：

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

任何影响文档的变更的验证集：

```powershell
node tools/src/generate-handbook.mjs --check
node tools/src/check-handbook.mjs
pnpm run check:consistency
git diff --check
```

旧文档（`docs/**`）继续遵循
[`docs-sync-contract.md`](../../../docs/standards/docs-sync-contract.md) 的自身义务；手册绝不吸收或替代
它们。canonical 来源与手册页冲突时，在同一交付中修正手册页——绝不为迎合文档去"修"
canonical 来源。
