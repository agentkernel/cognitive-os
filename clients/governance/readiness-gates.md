# 客户端实现 gate 入口（readiness-gates）

> 类别：informative gate registry ｜ owner：Lane-CON ｜ 日期：2026-07-20
>
> 状态：**骨架**。Console 实现 gate 的 canonical 正文当前仍在 [docs/platforms/README.md#console-实现-gate](../../docs/platforms/README.md#console-实现-gate)；迁移批 B6 将把 gate 正文（四条 gate 与激活规则）收纳进本文件并把旧位置压成 stub。在那之前本文件只登记指针，不复制正文。

## 1. Console 实现 gate（canonical 现址指针）

- [Console 实现 gate](../../docs/platforms/README.md#console-实现-gate)——依赖组 1/2/7、M5 出口评审、目标平台真实 PoC/GA gate、技术栈 ADR、machine contract/implementation/evidence 门槛。

## 2. 各平台 PoC / GA gate 入口（迁移前现址）

| 平台 | gate 入口 | 状态 |
|---|---|---|
| Windows | [windows-v1-scope §10](../pc/docs/platforms/windows/windows-v1-scope.md#10-技术候选与-release-gate)（release gate；无独立编号 PoC 表） | 未满足 |
| macOS | [macOS Open PoC 与 GA gates](../pc/docs/platforms/macos/macos-product-design.md#13-open-poc-and-ga-gates)（`MAC-POC-01..12`） | 全部 `not-run` |
| Linux | [Linux Open PoC 与 GA gates](../pc/docs/platforms/linux/linux-product-design.md#13-open-poc-and-ga-gates)（`LNX-POC-01..12`） | 全部 `not-run` |
| iPhone | [iPhone Open PoC 与 GA gates](../../docs/platforms/ios-product-design.md#18-open-poc-与-ga-gates)（`IOS-POC-01..18`） | 全部 `not-run` |
| Android | [Android Open PoC 与 GA gates](../../docs/platforms/android-product-design.md#18-open-poc-与-ga-gates)（`POC-001..018`） | 全部 `not-run` |

## 3. Agent Hub gate

- [Agent Hub 实现 gate（GOVERNANCE §7）](../../apps/cognitiveos-console/docs/agent-hub/GOVERNANCE.md#7-实现-gate不可跳过)：Console gate 全项 + Paseo/AGPL 法务 gate（`POC-LIC-001..003` 全部 `not-run`）+ Tier 1 provider 一手接口核验。

## 4. GO / NO-GO 记录

readiness 双结论（structure/implementation）唯一记录点是 [clients/READINESS.md](../READINESS.md)；本文件不重复记录结论。
