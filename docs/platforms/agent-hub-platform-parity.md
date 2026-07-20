# Agent Hub — 平台 Parity

> 类别：informative product design（激活前文档例外）｜ 日期：2026-07-20 ｜ owner：Lane-CON
>
> 状态：`planned / implementation not-implemented / test none / Profile not implemented`
>
> canonical 设计在 [apps/cognitiveos-console/docs/agent-hub/](../../apps/cognitiveos-console/docs/agent-hub/README.md)。本文只登记 **Direct Takeover 接管能力** 的跨平台差异，作为既有 [桌面 parity](../../clients/pc/docs/platforms/desktop-parity-matrix.md) 与 [移动 parity](./mobile-parity-matrix.md) 的补充；不改写既有平台 canonical 决策、ID 或 anchor。

## 1. 桌面接管能力 parity

| 能力 | Windows（首发） | macOS | Linux |
|---|---|---|---|
| 强进程 containment | Job Object + KILL_ON_JOB_CLOSE | 进程组 + launchd（弱，双 fork 可逃逸） | cgroup v2 `cgroup.kill`（强） |
| Host-owned 终端（L4） | ConPTY | openpty/forkpty；tmux/screen 独立 socket（未来） | openpty/forkpty；tmux/screen 独立 socket（未来） |
| 普通既有 console 抢占 | 不支持（归 L7/重启） | 不支持 | 不支持 |
| 进程身份锚 | handle+PID+creation time | PID+start+unique/version+code requirement | pidfd+starttime+exe(dev,ino) |
| 只读文件（L5）约束 | reparse/junction/cloud placeholder 检查 | `O_NOFOLLOW_ANY/RESOLVE_BENEATH/UNIQUE`（feature-test） | `openat2 RESOLVE_BENEATH/NO_XDEV` |
| 本机控制面 | named pipe + DACL + impersonation | Unix socket + peer cred；XPC per-message | Unix socket + `SO_PEERCRED`/`SO_PEERPIDFD` |
| 首发状态 | Direct v1 目标 | 设计保留，后于 Windows | 设计保留，后于 Windows |

平台事实来源见 [apps/cognitiveos-console/docs/agent-hub/sources/platform-security-ledger.md](../../apps/cognitiveos-console/docs/agent-hub/sources/platform-security-ledger.md)。

## 2. 移动 companion parity

| 能力 | iPhone（先发） | Android phone |
|---|---|---|
| 角色 | remote companion（不承载 runtime/authority） | 同左 |
| 触控目标 | 44×44 pt | 48×48 dp |
| 辅助技术 | VoiceOver/Voice Control/Switch Control/Full Keyboard Access | TalkBack/Switch Access/Voice Access/外接键盘 |
| 扩权 | 只能请求；高后果 PC-local 确认 | 同左 |
| 配对 | QR + 短码 + matching code + PC-local approve | 同左 |
| 首发状态 | 设计保留，先于 Android | 设计保留 |

移动无障碍验收见 [apps/cognitiveos-console/docs/agent-hub/product/states-content-and-accessibility.md](../../apps/cognitiveos-console/docs/agent-hub/product/states-content-and-accessibility.md#4-跨端无障碍验收未来门禁)。

## 3. gate

Agent Hub 各平台实现 gate 与 [Console 实现 gate](./README.md#implementation-gate) 一致，另加 Paseo/AGPL 复用法务 gate；当前全部平台 `not-implemented / evidence none`。既有移动/桌面 canonical 决策（`CONSOLE-MAC/LNX/IOS/AND-V1-*`）不受本文影响。
