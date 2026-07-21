# Open PoC 执行记录模板（共享薄模板）

> 类别：informative template ｜ owner：Lane-CON ｜ 状态：`planned`（模板已提供；执行 evidence `none`）
>
> 用途：五平台与 Agent Hub Open PoC 执行时复制本表填一行/一份记录。不替代各平台产品设计中的 PoC 定义；不构成已执行证据。

## 1. 指针

| 用途 | 路径 |
|---|---|
| 证据指针总索引 | [governance/evidence-index.md](../../governance/evidence-index.md) |
| 结果五态 / digest 口径 | [conformance-evidence](../../../docs/standards/conformance-evidence.md)（`pass` / `fail` / `not-applicable` / `documented-degradation` + 报告级 `not-run`） |
| Agent Hub 条目模板 | [agent-hub open-poc](../../agent-hub/docs/templates/open-poc.md) |
| 共用测试策略 | [test-strategy.md](test-strategy.md) |

## 2. 环境指纹（必填字段）

复制后填写；缺任一字段不得宣称 pass。

| 字段 | 值 |
|---|---|
| PoC ID | |
| 平台 / OS 版本 / 补丁级 | |
| 设备或 VM 型号 / CPU / ABI | |
| 构建 channel / bundle 或包 identity（若适用） | |
| 签名 / 公证 / Play Protect / GMS 等（若适用） | |
| 辅助技术或浏览器/WebView 版本（若适用） | |
| 执行人 / 日期（UTC） | |
| 隔离说明（真实 API/OS；禁 mock） | |

## 3. 五态结果

| 字段 | 值 |
|---|---|
| 结果态 | `not-run`（默认）／执行后仅允许 `pass` / `fail` / `not-applicable` / `documented-degradation` |
| 通过标准对照 | （引用产品设计或 runbook 中的 oracle） |
| 安全负例 | （必须一并记录；不可豁免） |
| `not-applicable` 理由 | （仅当适用；须绑定声明范围） |

## 4. artifacts 落点与 digest

- 证据文件**不入库**；落在本地 `artifacts/evidence/clients/<platform>/<poc-id>/`（gitignore）。
- 在对应 evidence-index / runbook 行只登记：**相对指针 + content digest**（算法与全局证据口径一致），不得粘贴秘密或大段日志正文。
- 当前全部客户端 PoC：状态 `not-run`，证据路径 `none`。

## 5. 记录骨架（复制块）

```markdown
### <POC-ID> <短标题>

- 环境指纹：（上表）
- 断言：
- 方法（真实环境）：
- 结果态：not-run
- 证据路径：none
- digest：none
```
