# 客户端全里程碑（全部 blocked）

> 类别：plan ｜ owner：Lane-CON ｜ 日期：2026-07-20
>
> 排期不构成实现授权；每个里程碑的解阻条件是真实 gate 证据，不是文档完成度。

| 里程碑 | 范围 | 状态 | 解阻 gate |
|---|---|---|---|
| CL-M1 PC Windows MVP（只读监督起步） | Task/Execution 五轨 + watch 投影 | `blocked` | [Console 实现 gate](../governance/readiness-gates.md#console-实现-gate)：依赖组 1/2/7 + M5 + Windows 真实 PoC + PC 技术栈 ADR |
| CL-M2 macOS / Linux parity | 桌面 parity 切片 | `blocked` | 同上 + [macOS](../pc/docs/platforms/macos/macos-product-design.md#13-open-poc-and-ga-gates) / [Linux](../pc/docs/platforms/linux/linux-product-design.md#13-open-poc-and-ga-gates) PoC |
| CL-M3 iPhone remote companion | 受限远程 Console | `blocked` | Console gate + [iPhone PoC](../mobile/ios/docs/ios-product-design.md#18-open-poc-与-ga-gates) + iOS ADR |
| CL-M4 Android remote companion | 受限远程 Console | `blocked` | Console gate + [Android PoC](../mobile/android/docs/android-product-design.md#18-open-poc-与-ga-gates) + Android ADR |
| Agent Hub AH-M0..M6 | 见 Master plan | 全部 `blocked` | [Agent Hub 六类 gate](../agent-hub/docs/GOVERNANCE.md#7-实现-gate不可跳过)；里程碑定义见 [milestones](../agent-hub/plan/milestones.md) |

CL-M* 是客户端域的本地编号（非全局里程碑）；全局 M0–M11 唯一定义在 [DEVELOPMENT-PLAN](../../docs/plan/DEVELOPMENT-PLAN.md)，状态唯一真相在 [PROGRESS](../../docs/plan/PROGRESS.md)。

## 当前可执行工作面（2026-07-26）

上表全部 `blocked` 指**实现**里程碑；以下工作**现在即可推进**，不违反任何 gate：

| 工作 | 依据 | owner |
|---|---|---|
| Phase 0 剩余文档/工具任务（P0-T5..T7） | [development-plan.md](development-plan.md) | Lane-CON（T5 归 Lane-CFR） |
| Windows PoC WIN-RG-01..10 真实执行（代码落位 `poc/windows/`） | [CLIENTS-DEC-002](../governance/decision-log.md#clients-dec-002-poc-证据采集代码豁免与落位) + [runbook](../pc/docs/platforms/windows/windows-poc-runbook.md) | Lane-CON |
| POC-LIC-001..003 法务评估发起 | [development-plan.md](development-plan.md) Phase 1B | Lane-CON + 法务 |
| Adapter 首发子集（6→2）决策准备 | Phase 1C | Lane-CON |
| Console gate 依赖组 1/2/7 对账留证（M0–M6 已过后的逐条核验） | [readiness-gates](../governance/readiness-gates.md#console-实现-gate) | Lane-CON |

PoC/评估/决策的产出是**证据与决策记录**，不是实现；四类状态用语照旧。
