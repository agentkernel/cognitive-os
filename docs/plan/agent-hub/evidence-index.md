# Agent Hub 开发证据索引

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON
>
> 本页汇总开发侧证据要求，与产品侧 [证据索引](../../../apps/cognitiveos-console/docs/agent-hub/traceability/evidence-index.md) 对齐。**当前全部 `not-run / none`。**

## 证据类别

| 类别 | 要求 | 当前状态 |
|---|---|---|
| Open PoC | 见产品证据索引 27+ 项 `CONSOLE-AGENTHUB-V1-POC-*` | not-run |
| 单元/集成失败测试 | 各任务失败测试先行 | none |
| 安全负例 | squatting/PID reuse/breakaway/stdin 抢占/双 writer/symlink/MITM/replay 等 | none |
| 无障碍验收 | WCAG 2.2 + 各平台原生 AT 关键旅程 | none |
| 恢复演练 | Host 崩溃/孤儿/split-brain/丢失设备 | none |
| 跨语言 golden | 如涉及契约，走既有 golden job | n/a（未实现） |
| 法务评估 | Paseo/AGPL/第三方组件/条款 | not-run |

## 规则

- evidence 转 `pass` 前，任何文档不得声明能力 implemented 或 Profile 已符合。
- PoC 证据落 `artifacts/`（gitignore），此处只登记状态与路径。
- 安全负例不可豁免。
