# Agent Hub — 许可与条款

> 类别：informative（非法律意见）｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 本文汇总供应商条款/许可事实与复用 gate；不构成法律结论。逐条来源见 [../sources/terms-and-licenses-ledger.md](../sources/terms-and-licenses-ledger.md) 与 [../sources/paseo-and-comparables-ledger.md](../sources/paseo-and-comparables-ledger.md)。

## 1. 复用与许可 gate（不可跳过）

已冻结（[CONSOLE-AGENTHUB-V1-DEC-023](../decisions/decision-log.md)）：任何对第三方源码/文档/测试/协议实现/派生结构的复用，必须先完成独立法务与第三方组件义务评估，未完成前对应任务 `blocked`。

## 2. Paseo 复用方向与 AGPL 义务

已冻结（[CONSOLE-AGENTHUB-V1-DEC-024](../decisions/decision-log.md)）：未来 Takeover Host / Provider 适配 / Relay 若复用 Paseo，将整体以 **AGPL-3.0-or-later** 发布；桌面/移动客户端许可另行评估，不默认继承 AGPL。

AGPL-3.0-or-later 关键义务（来自 Paseo LICENSE 核验）：

- 分发源码/修改版需保留版权、许可与免责声明；
- 分发 object code 需按 §6 提供 Corresponding Source；
- 若修改并通过网络提供远程交互（§13），须显著提供该版本 Corresponding Source 的免费获取方式；
- 第三方组件继续服从其原许可证。

实际复用前必须闭合：

1. AGPL-3.0-or-later 兼容性与本仓库其他许可证的边界；
2. 每个第三方组件义务清单；
3. §13 网络交互的 Corresponding Source 提供方案；
4. 客户端/服务端许可分界；
5. clean-room 与直接复用的界定。

在完成上述评估前，只允许借鉴**架构思想**（clean-room 重新设计），不复制任何 Paseo 代码/文档/测试。

## 3. 供应商条款要点（逐项见 source ledger）

产品设计遵循以下条款约束（事实与查询日以 source ledger 为准，不在此复述细节）：

- 使用官方提供的控制接口（SDK/App Server/ACP/CLI/session API），不绕过登录、计费、安全或组织策略；
- 不抽取/存储/传输 provider 凭据（除官方支持的 opaque profile）；
- 不写入 provider 未公开支持外部写入的 session/内部数据；
- 不将 provider 私有 binary/hash 上传第三方（如默认 VirusTotal 上传）；
- 尊重各 Agent 的 rate limit、配额与账号边界。

## 4. source-available 与非 OSI 许可提示

竞品研究中若干项目使用 source-available 或附加限制许可（如 Commons Clause、Elastic-based、Open WebUI License、PolyForm），不得当作无限制开源复用；本产品不复用其代码，仅在明确标注下参考公开行为事实。

## 5. Open 评估项（法务/条款）

全部 `not-run / evidence none`（此处 evidence 指“已完成评估并留证”）：

- `POC-LIC-001` Paseo AGPL + 第三方组件义务完整清单；
- `POC-LIC-002` 六个 Tier 1 provider 条款对“第三方 Host 启动/接管/读取 session”的逐项允许性；
- `POC-LIC-003` 客户端/服务端许可分界方案。
