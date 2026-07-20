# 车道计划 — Android（AND）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> 目标：Android phone remote companion（后于 iPhone）。设计见 [states-content-and-accessibility.md §4.3](../docs/product/states-content-and-accessibility.md#43-android-phone)。

## 范围与路径

- 允许（激活后）：Android companion app 模块。
- 禁止：他人车道代码；手机承载 runtime/authority；直接扩权。
- 依赖：RELAY、DESK、IOS（形态对齐）。gate：AH-B1、AH-B2（Android PoC/GA）、AH-B3。

## 任务

### AH-AND-01 配对与多 Host companion
- owner/lane：Lane-CON / AND｜depends_on：AH-RELAY-02｜blocked_by：AH-B2
- 交付物：与 iOS 对齐的配对/Host switcher/持续状态显示；StrongBox 尽力硬件绑定
- 安全负例：手机不能直接扩权
- oracle：跨 Host 不共享 lease｜evidence：none

### AH-AND-02 监督/请求/裁剪投影
- owner/lane：Lane-CON / AND｜depends_on：AH-AND-01｜blocked_by：—
- 交付物：与 iOS 一致的投影集；裁剪 metadata；push 仅 opaque hint
- 安全负例：不暴露其他用户/凭据
- oracle：功能与 iOS parity｜evidence：none

### AH-AND-03 Android 无障碍
- owner/lane：Lane-CON / AND｜depends_on：AH-AND-01｜blocked_by：AH-B2
- 交付物：48×48dp；TalkBack/Switch Access/Voice Access/外接键盘；200% font 与最大 Display size；high contrast/color correction 保状态；predictive back/IME/cutout 不遮挡；Remove animations 等价；Compose 真实 role/state/action semantics（非 aria）
- 失败测试先行：关键旅程各 AT 可完成
- oracle：Android 无障碍验收通过｜evidence：none

### AH-AND-04 回前台安全与恢复
- owner/lane：Lane-CON / AND｜depends_on：AH-AND-01,AH-RELAY-04｜blocked_by：—
- 交付物：回前台 reauth+resnapshot；offline 队列 expiry；丢失设备 revoke
- oracle：恢复演练证据｜evidence：none
