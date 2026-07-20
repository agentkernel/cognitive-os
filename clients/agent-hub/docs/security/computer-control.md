# Agent Hub — 电脑控制

> 类别：informative security design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 状态：`accepted product direction / implementation not-implemented / evidence none`。

## 1. 范围与已冻结决策

已冻结（[CONSOLE-AGENTHUB-V1-DEC-022](../decisions/decision-log.md)）：首发桌面控制仅限 **selected-window**（用户为某个受管任务显式选择的单个目标窗口/应用），不提供通用全桌面 GUI 控制。

理由：通用桌面控制会带来误点击、访问无关敏感窗口、UAC secure desktop 无法安全控制、截屏泄露等高风险，且难以给出可解释的最小权限边界。

## 2. selected-window 控制模型

- 用户为具体 WorkItem 显式选择一个目标窗口；控制能力绑定该窗口句柄与所属受管进程。
- 输入/截屏范围限定该窗口；目标窗口失焦、最小化或关闭即暂停控制并要求重新授权。
- 禁止：跨窗口全局输入、全屏截取、控制 secure desktop / UAC 提示 / 登录界面 / 凭据输入框。
- 每次启用或变更桌面控制默认 PC-local 确认（[CONSOLE-AGENTHUB-V1-DEC-014](../decisions/decision-log.md)）。

## 3. 隔离浏览器

- 需要 Agent 浏览网页时，使用受控隔离浏览器上下文（独立 profile、受限权限、无用户默认 cookie/凭据），不复用用户主浏览器会话。
- 隔离浏览器内容属不可信输入；渲染与系统控件不共享安全边界。
- 下载/上传范围限定 Host-owned workspace 目录。

## 4. PC-local 确认矩阵

| 动作 | 手机可发起 | 需 PC-local 确认 |
|---|---|---|
| 查看受管窗口截图（裁剪） | 是（请求） | 否（只读投影） |
| 启用/变更 selected-window 控制 | 请求 | 是 |
| 在目标窗口内输入/点击 | 请求 | 首次启用时确认；之后按会话策略 |
| 启动隔离浏览器 | 请求 | 是 |
| 全桌面控制 | 不可 | 不适用（blocked-by-policy） |

## 5. 平台差异

- **Windows**：UAC secure desktop 与凭据 UI 在独立 desktop，产品无法也不应控制；截屏 API 对受保护内容返回黑屏，需如实呈现为 `unavailable`。
- **macOS**：需 Accessibility 与 Screen Recording 授权；未授权即 `unavailable`，不诱导用户全局授权。
- **Linux**：Wayland 下全局输入注入受限，X11 与 Wayland 行为不同，逐会话协议记录能力，不外推。

## 6. Open PoC（电脑控制）

全部 `not-run / evidence none`：

- `POC-CC-001` selected-window 输入范围限制（越窗口输入被拒）；
- `POC-CC-002` secure desktop / 凭据界面截屏保护如实降级；
- `POC-CC-003` 隔离浏览器 profile 与用户主会话无凭据交叉。
