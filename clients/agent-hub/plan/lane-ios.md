# 车道计划 — iOS（IOS）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> 目标：iPhone remote companion（先于 Android）。设计见 [product/journeys-and-screens.md §5](../docs/product/journeys-and-screens.md#5-手机-remote-companion-旅程要点)、[states-content-and-accessibility.md §4.2](../docs/product/states-content-and-accessibility.md#42-iphone)。

## 范围与路径

- 允许（激活后）：iOS companion app 模块。
- 禁止：他人车道代码；手机承载 runtime/authority/完整 Vault；手机直接扩权/发信号/扩大文件范围。
- 依赖：RELAY、DESK。gate：AH-B1、AH-B2（iOS PoC/GA）、AH-B3。

## 任务

### AH-IOS-01 配对与多 Host companion
- owner/lane：Lane-CON / IOS｜depends_on：AH-RELAY-02｜blocked_by：AH-B2
- 交付物：QR+短码+可读非扫码路径；Host switcher；每写页面显示 Host/账号/模式/freshness
- 安全负例：手机不能直接扩权
- oracle：跨 Host 不共享 lease｜evidence：none

### AH-IOS-02 监督/请求/裁剪投影
- owner/lane：Lane-CON / IOS｜depends_on：AH-IOS-01｜blocked_by：—
- 交付物：任务进度/takeover source/diff/artifact/clarification/permission/pause-cancel/current-state refresh；裁剪 metadata
- 安全负例：不暴露其他用户/无关路径/env/credential/raw auth
- oracle：push 仅 opaque hint｜evidence：none

### AH-IOS-03 iOS 无障碍
- owner/lane：Lane-CON / IOS｜depends_on：AH-IOS-01｜blocked_by：AH-B2
- 交付物：44×44pt；VoiceOver/Voice Control/Switch Control/Full Keyboard Access/外接键盘；最大 Dynamic Type 不截断关键动词；portrait/landscape 一致；Reduce Motion 等价
- 失败测试先行：关键旅程各 AT 可完成
- oracle：iOS 无障碍验收通过｜evidence：none

### AH-IOS-04 回前台安全与恢复
- owner/lane：Lane-CON / IOS｜depends_on：AH-IOS-01,AH-RELAY-04｜blocked_by：—
- 交付物：回前台 reauth+resnapshot；offline 队列 expiry；丢失设备 revoke
- 安全负例：过期请求不静默补发
- oracle：恢复演练证据｜evidence：none
