# PC 客户端技术栈候选比较（非正式 ADR）

> 类别：informative comparison draft ｜ owner：Lane-CON ｜ 日期：2026-07-21
>
> **本文是候选比较，不是已批准 ADR。** 不得据此批准技术栈、启动 `clients/pc/app` 实现，或改写 implementation-ready。
>
> 冻结条件（仍未满足）：[windows-v1-scope §10](../platforms/windows/windows-v1-scope.md#10-技术候选与-release-gate) 十条真实 PoC 留证 + 正式 ADR 评审。相关产品声明见 [decision-log](../product/decision-log.md)、[product-brief](../product/product-brief.md)。

## 1. 评估维度

| 维度 | 关注点（摘要） |
|---|---|
| 安全 | renderer/host/service 隔离；IPC allowlist；WebView 攻击面；密钥与 capability 边界 |
| 原生集成 | Windows Service、托盘、通知、UAC、文件系统、Keychain 等价物、自动更新钩子 |
| 无障碍 | Narrator / 键盘 / 高对比 / 缩放 / reduced motion；系统控件可达性 |
| 更新 | 签名制品、anti-rollback、Service 与 UI 协同升级、失败回滚 |
| 调试 | 崩溃符号、IPC 追踪、多进程调试、CI 可复现 |
| 包体积 | 安装包与常驻内存预算；与 §10 资源门对齐 |
| 性能 | 冷启动、长会话、10k 事件、托盘 24h |
| 跨平台维护 | Windows 首发 vs macOS/Linux parity 文档切片的共享面与分叉成本 |
| 许可证 | 运行时/工具链/再分发义务；与 Agent Hub POC-LIC 分界不混淆 |
| 长期维护 | 上游 release cadence、CVE 响应、团队技能与供应商锁定 |

## 2. 候选（文档已提及 + 常见对照）

状态列仅表示「文档中的位置」，**无一批准**。

| 候选 | 文档位置 / 备注 | 比较状态 |
|---|---|---|
| **Tauri 2 + React/TypeScript** | Windows §10、product-brief、decision-log：**首选候选** | 未批准；待 §10 PoC |
| 受控原生 Host + 独立 WebView2/系统 WebView（非 Tauri 壳） | trust-safety-ux / 部署边界要求「受控原生 Host」；可与或不与 Tauri 组合 | 架构约束，非栈批准 |
| Electron + React/TS | 未选为首选；对照用（体积/更新/多进程模型差异大） | 仅对照，非候选批准 |
| 纯原生（WinUI / WPF 等）+ 最小 WebView 面 | 未立项；对照无障碍与系统控件边界 | 仅对照 |
| macOS/Linux 同构壳（Tauri 或其他） | macos/linux 产品设计提及 Tauri isolation **不构成安全边界** | parity 切片；非 Windows v1 范围 |

手机栈（SwiftUI / Kotlin Compose 等）不在本文范围；各有独立 ADR 缺口（见 READINESS）。

## 3. 维度速览（定性，无得分排名）

> 下列为准备 PoC/ADR 的讨论锚点，**不是评分结论**。空格表示「须用真实 PoC 填空」。

| 维度 | Tauri 2 + React/TS | Electron + React/TS | 纯原生 + 最小 WebView |
|---|---|---|---|
| 安全 | 依赖 Capabilities + 自建 Host/Service 隔离；WebView CVE floor 仍要自证 | Chromium 面大；多进程成熟但攻击面与体积成本高 | 系统控件边界清晰；富内容面需自建 |
| 原生集成 | Rust/宿主侧可接 Service；须自证 SID/UAC/托盘 | Node/原生 addon 路径多；Service 集成仍定制 | 最贴近 OS API；跨平台成本高 |
| 无障碍 | Web + 系统控件双轨；须 §10.6 真机证据 | 同类双轨 | 系统控件优势；富文本仍要证明 |
| 更新 | 须自建签名/anti-rollback 与 Service 协同 | 生态更新器多；仍须产品级 anti-rollback | 平台安装器友好；逻辑仍自建 |
| 调试 | Rust+Web 双工具链 | Chromium DevTools 成熟 | 平台工具成熟 |
| 包体积 | 通常小于 Electron（待测） | 偏大（待测） | 视 UI 栈（待测） |
| 性能 | 待 §10.7 | 待测 | 待测 |
| 跨平台维护 | 文档假设共享壳；macOS/Linux **非** v1 | 共享面大但 Linux WebKit/Win 差异仍在 | 三端分叉最大 |
| 许可证 | 查 Tauri/Rust/前端依赖再分发；**未做法务结论** | Chromium/Electron 义务清单；未评估 | 视 UI 框架 |
| 长期维护 | 上游活跃；须跟踪 WebView2/WebKitGTK floor | 上游活跃；体积与 CVE 面持续成本 | 平台绑定强 |

## 4. 明确非结论

1. **不批准**任何技术栈；不产生 ADR 编号。
2. Tauri 2 + React/TS **仍是首选候选**，不是决定。
3. CSP / Tauri isolation / `WKContentWorld` **不是**独立安全边界（见桌面 parity / 平台设计）。
4. 比较表填满之前，`clients/pc/app` 保持 **blocked**。

## 5. 下一步（仍 informative）

1. 按 [windows-poc-runbook](../platforms/windows/windows-poc-runbook.md) 执行 `WIN-RG-*` 并留 digest。
2. 用实测回填本表「待测」格。
3. 另开正式 ADR 流程（Lane-CON + 安全/法务评审）；本文可作附录引用，不可替代 ADR。
