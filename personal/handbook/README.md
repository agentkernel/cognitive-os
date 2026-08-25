# CognitiveOS Personal Handbook / 手册

An independent, bilingual documentation system for **CognitiveOS Personal** — the
repository's sole active implementation project. Every factual page is mapped to
tracked sources, contracts, and tests, fingerprinted against them, and verified by
`node tools/src/check-handbook.mjs`. The handbook is an informative derived layer: it
never owns task status, Gate results, contracts, or release claims. Canonical sources
always win; the machine model lives in [`manifest.json`](_meta/manifest.json),
[`source-map.json`](_meta/source-map.json), and
[`source-coverage.json`](_meta/source-coverage.json).

这是 **CognitiveOS Personal** 的独立双语文档系统。每个事实页面都映射到 tracked 源码、
合同与测试，并以指纹与之绑定，由 `node tools/src/check-handbook.mjs` 机器校验。手册是
informative 派生层：不拥有任务状态、Gate 结果、合同或发布声明；与 canonical 来源冲突时
一律以后者为准。机器模型见 [`manifest.json`](_meta/manifest.json)、
[`source-map.json`](_meta/source-map.json) 与
[`source-coverage.json`](_meta/source-coverage.json)。

## Choose your entry / 选择入口

| Reader / 读者 | English | 中文 |
|---|---|---|
| Personal users / 使用者 | [User guide](en/user/README.md) | [用户指南](zh-CN/user/README.md) |
| Project developers / 开发者 | [Developer guide](en/developer/README.md) | [开发者指南](zh-CN/developer/README.md) |
| Reference / 参考 | [Reference](en/reference/README.md) | [参考手册](zh-CN/reference/README.md) |
| AI coding tools / AI 编程工具 | [AI entry](en/ai/README.md) | [AI 入口](zh-CN/ai/README.md) |

## Canonical sources this handbook defers to / 手册让位的 canonical 来源

- Governance and axioms / 治理与公理: [`docs/governance/`](../../docs/governance/AXIOMS.md)
- Formal tasks and Gates / 正式任务与 Gate: [`docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`](../../docs/plan/PERSONAL-DEVELOPMENT-PLAN.md)
- Current facts / 当前事实: [`docs/plan/PROGRESS.md`](../../docs/plan/PROGRESS.md) `Current snapshot` (linked, never copied / 只链接，绝不复制)
- Machine contracts / 机器合同: [`core/specs/`](../../core/specs/core/README.md), [`core/conformance/`](../../core/conformance/README.md), [`docs/standards/`](../../docs/standards/docs-sync-contract.md)
- Stable product/architecture design / 产品与架构设计: [`personal/docs/product/`](../docs/product/README.md), [`personal/docs/architecture/`](../docs/architecture/README.md)

## Keeping it honest / 防漂移

- `node tools/src/check-handbook.mjs` — manifest, locale pairing, links, sources,
  symbols, fingerprints, total coverage, generated-page equality, forbidden content.
- `node tools/src/generate-handbook.mjs` — regenerates reference pages from
  implementation and machine contracts; `--check` fails on drift.
- Sync obligations / 同步义务: [`personal/handbook/_meta/sync-policy.md`](_meta/sync-policy.md)
  and the always-applied rule `.cursor/rules/20-cognitiveos-personal-handbook-sync.mdc`.
