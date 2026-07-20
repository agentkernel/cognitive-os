# 客户端全里程碑（全部 blocked）

> 类别：plan ｜ owner：Lane-CON ｜ 日期：2026-07-20
>
> 排期不构成实现授权；每个里程碑的解阻条件是真实 gate 证据，不是文档完成度。

| 里程碑 | 范围 | 状态 | 解阻 gate |
|---|---|---|---|
| CL-M1 PC Windows MVP（只读监督起步） | Task/Execution 五轨 + watch 投影 | `blocked` | [Console 实现 gate](../../docs/platforms/README.md#console-实现-gate)：依赖组 1/2/7 + M5 + Windows 真实 PoC + PC 技术栈 ADR |
| CL-M2 macOS / Linux parity | 桌面 parity 切片 | `blocked` | 同上 + [macOS](../pc/docs/platforms/macos/macos-product-design.md#13-open-poc-and-ga-gates) / [Linux](../pc/docs/platforms/linux/linux-product-design.md#13-open-poc-and-ga-gates) PoC |
| CL-M3 iPhone remote companion | 受限远程 Console | `blocked` | Console gate + [iPhone PoC](../../docs/platforms/ios-product-design.md#18-open-poc-与-ga-gates) + iOS ADR |
| CL-M4 Android remote companion | 受限远程 Console | `blocked` | Console gate + [Android PoC](../../docs/platforms/android-product-design.md#18-open-poc-与-ga-gates) + Android ADR |
| Agent Hub AH-M0..M6 | 见 Master plan | 全部 `blocked` | [Agent Hub 六类 gate](../../apps/cognitiveos-console/docs/agent-hub/GOVERNANCE.md#7-实现-gate不可跳过)；里程碑定义见 [milestones](../../docs/plan/agent-hub/milestones.md)（B5 迁移前现址） |

CL-M* 是客户端域的本地编号（非全局里程碑）；全局 M0–M11 唯一定义在 [DEVELOPMENT-PLAN](../../docs/plan/DEVELOPMENT-PLAN.md)，状态唯一真相在 [PROGRESS](../../docs/plan/PROGRESS.md)。
