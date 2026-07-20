# 客户端实现 gate（readiness-gates）

> 类别：informative gate registry ｜ owner：Lane-CON ｜ 日期：2026-07-20
>
> 本文件是 **Console 实现 gate 的 canonical 定义点**（自 `docs/platforms/README.md` 迁入，旧位置保留 anchor stub）；同时收纳各平台 PoC/GA gate 入口与 Agent Hub gate 指针。

<a id="console-实现-gate"></a>
<a id="implementation-gate"></a>
## 1. Console 实现 gate（canonical）

任何 Console 实现里程碑仍须同时满足：

1. [DEVELOPMENT-PLAN Console 节](../../docs/plan/DEVELOPMENT-PLAN.md) 依赖组 1、2、7 已交付；
2. M5 出口评审通过；
3. 目标平台文档的 Open PoC 与 GA gates 使用真实 API/真实 OS 行为完成并留下可复现实测证据（各平台入口见 §2）；
4. 技术栈 ADR 已批准；
5. 适用 machine contract、implementation 和 executed evidence 分别达到其声明门槛。

在此之前，所有平台 Profile 均保持 `planned`，所有平台专属测试证据均为 `none`。

**激活规则**：依赖组 1/2/7 交付并过 M5 出口评审后，且目标平台 Open PoC/GA gate 用真实 API/真实 OS 行为出具可复现实测报告，才可启动 Console "MVP Desktop 只读监督"实现里程碑规划；不得用 mock 冒充。Lane-CON 激活前 informative 文档例外（[PARALLEL-LANES §2.1](../../docs/plan/PARALLEL-LANES.md)）不改变此 gate。

## 2. 各平台 PoC / GA gate 入口（迁移前现址）

| 平台 | gate 入口 | 状态 |
|---|---|---|
| Windows | [windows-v1-scope §10](../pc/docs/platforms/windows/windows-v1-scope.md#10-技术候选与-release-gate)（release gate；无独立编号 PoC 表） | 未满足 |
| macOS | [macOS Open PoC 与 GA gates](../pc/docs/platforms/macos/macos-product-design.md#13-open-poc-and-ga-gates)（`MAC-POC-01..12`） | 全部 `not-run` |
| Linux | [Linux Open PoC 与 GA gates](../pc/docs/platforms/linux/linux-product-design.md#13-open-poc-and-ga-gates)（`LNX-POC-01..12`） | 全部 `not-run` |
| iPhone | [iPhone Open PoC 与 GA gates](../mobile/ios/docs/ios-product-design.md#18-open-poc-与-ga-gates)（`IOS-POC-01..18`） | 全部 `not-run` |
| Android | [Android Open PoC 与 GA gates](../mobile/android/docs/android-product-design.md#18-open-poc-与-ga-gates)（`POC-001..018`） | 全部 `not-run` |

## 3. Agent Hub gate

- [Agent Hub 实现 gate（GOVERNANCE §7）](../agent-hub/docs/GOVERNANCE.md#7-实现-gate不可跳过)：Console gate 全项 + Paseo/AGPL 法务 gate（`POC-LIC-001..003` 全部 `not-run`）+ Tier 1 provider 一手接口核验。

## 4. GO / NO-GO 记录

readiness 双结论（structure/implementation）唯一记录点是 [clients/READINESS.md](../READINESS.md)；本文件不重复记录结论。
