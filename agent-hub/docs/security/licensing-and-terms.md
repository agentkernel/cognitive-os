# Agent Hub — 许可与条款

> 类别：informative（非法律意见）｜ 日期：2026-07-20（2026-07-21 Legal Auditor 材料回填）｜ canonical owner：Lane-CON
>
> 本文汇总供应商条款/许可事实与复用 gate；**不构成法律结论或法务批准**。逐条来源见 [../sources/terms-and-licenses-ledger.md](../sources/terms-and-licenses-ledger.md) 与 [../sources/paseo-and-comparables-ledger.md](../sources/paseo-and-comparables-ledger.md)。**POC-LIC-001..003 仍为 not-run**（评估材料已整理，法务评估未执行）。

## 1. 复用与许可 gate（不可跳过）

已冻结（[CONSOLE-AGENTHUB-V1-DEC-023](../decisions/decision-log.md)）：任何对第三方源码/文档/测试/协议实现/派生结构的复用，必须先完成独立法务与第三方组件义务评估，未完成前对应任务 `blocked`。

## 2. Paseo 复用方向与 AGPL 义务

**指认**：Paseo = GitHub [`getpaseo/paseo`](https://github.com/getpaseo/paseo)（查询日 2026-07-20/21）。

已冻结（[CONSOLE-AGENTHUB-V1-DEC-024](../decisions/decision-log.md)）：未来 Takeover Host / Provider 适配 / Relay 若复用 Paseo，将整体以 **AGPL-3.0-or-later** 发布；桌面/移动客户端许可另行评估，不默认继承 AGPL。

**许可证来源层级（一手）**：

- 根 `LICENSE` = 自定义头 + **AGPL-3.0 全文**（GitHub licensee 可能显示 Other/NOASSERTION）；
- **`or-later` 声明仅见于 `package.json` 的 `"license": "AGPL-3.0-or-later"`**——登记时须注明「package metadata」。

### AGPL §13 两要件（网络交互）

若修改并通过网络提供远程交互，须同时满足：

1. **显著告知**：以合适方式告知所有用户，可按 AGPL 获得该版本 Corresponding Source；
2. **免费提供 Corresponding Source**：通过网络服务器等，对应用户无额外费用地提供完整 Corresponding Source。

（条文摘自 Paseo 所附 AGPLv3 全文「13. Remote Network Interaction」；检索日 2026-07-20/21。）

### 其他关键义务

- 分发源码/修改版需保留版权、许可与免责声明；
- 分发 object code 需按 §6 提供 Corresponding Source；
- 第三方组件继续服从其原许可证（完整 workspace SBOM 在实际复用前闭合；当前 clean-room 下不构成实现阻断）。

### clean-room 边界

在完成法务评估前，只允许借鉴**架构思想**（clean-room 重新设计），**不复制**任何 Paseo 代码/文档/测试/协议实现/类型或 schema。若协议 schema 从 Paseo 复制，客户端也可能被拉入 AGPL——这是分界红线。

实际复用前必须闭合：

1. AGPL-3.0-or-later 兼容性与本仓库其他许可证的边界；
2. 每个第三方组件义务清单（全 workspace SBOM）；
3. §13 网络交互的 Corresponding Source 提供方案；
4. 客户端/服务端许可分界；
5. clean-room 与直接复用的界定。

**非 AGPL 复用 Paseo 代码**：须与版权人协商**双许可**（外部权利人项）；按 AGPL 复用本身不需上游批准，但需自身合规方案。

## 3. 供应商条款要点（逐项见 source ledger）

产品设计遵循以下条款约束（事实与查询日以 source ledger 为准）：

- 使用官方提供的控制接口（SDK/App Server/ACP/CLI/session API），不绕过登录、计费、安全或组织策略；
- 不抽取/存储/传输 provider 凭据（除官方支持的 opaque profile）；
- 不写入 provider 未公开支持外部写入的 session/内部数据；
- 不将 provider 私有 binary/hash 上传第三方；
- 尊重各 Agent 的 rate limit、配额与账号边界；
- **Claude TS SDK**：受 Anthropic Commercial ToS（非 OSS）；**不得**把 Python MIT wrapper 误读为 CLI/服务条款豁免；
- **OpenHands enterprise/**：PolyForm Free Trial，禁分发，不复用。

## 4. 候选栈义务摘要（客户端分界材料）

候选客户端栈（Tauri 2 + React 等，**非批准 ADR**）组件义务摘要（Legal Auditor 材料级，非批准）：

| 组件 | 许可要点 | 义务摘要 |
|---|---|---|
| Tauri 2 | MIT / Apache-2.0 双许可 | 保留声明；商标指南另遵 |
| React | MIT | 保留声明 |
| SQLite | Public Domain | 无强制开源传染 |
| libsodium | ISC | 保留声明 |
| WebView2（Windows） | Microsoft 专有运行时许可 | 可随应用再分发 bootstrapper/offline installer（细则以包内 LICENSE 为准；NuGet 文本待补证） |
| WKWebView（Apple） | 系统框架 + **Apple Developer Program License Agreement（PLA）** | PLA 文本需开发者账号核验（外部项）；iOS App Review 2.5.6 要求浏览类 App 用 WebKit |
| WebKitGTK（Linux） | LGPL-2.1 + BSD 部分 | 动态链接系统包时保留声明、保证可替换 |

结论骨架：候选客户端组件**不存在强制客户端整体开源的义务**；若 Host/Relay 因复用 Paseo 变为 AGPL，分界要件 = 独立进程 + 公开文档化协议 + 客户端零 AGPL 代码/类型/schema 复制 + 独立构建产物。

## 5. Open 评估项（法务/条款）

全部 **`not-run`**（evidence 指「已完成评估并留证」；备注：评估材料已整理，法务评估未执行）：

| PoC ID | 断言 | 状态 | 备注 |
|---|---|---|---|
| `POC-LIC-001` | Paseo AGPL + 第三方组件义务完整清单 | not-run | 指认/§13/来源层级已整理；全 workspace SBOM 待复用前闭合 |
| `POC-LIC-002` | 六个 Tier 1 provider 条款对「第三方 Host 启动/接管/读取 session」的逐项允许性 | not-run | 软件 SPDX 已文档级核验；服务条款外部确认仍开放 |
| `POC-LIC-003` | 客户端/服务端许可分界方案 | not-run | 分界骨架与候选栈义务已整理；待法务正式意见 |

## 6. 外部阻断（登记于 risk-register）

见 [../../plan/risk-register.md](../../plan/risk-register.md) 外部阻断表：Anthropic 订阅自动化确认、OpenAI ChatGPT 包装意见、Apple PLA、非 AGPL 复用 Paseo 双许可。
