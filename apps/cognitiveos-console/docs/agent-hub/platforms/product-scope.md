# Agent Hub — 平台范围

> 类别：informative product design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 与既有平台设计联动见 [docs/platforms/README.md](../../../../../docs/platforms/README.md) 与 [docs/platforms/agent-hub-platform-parity.md](../../../../../docs/platforms/agent-hub-platform-parity.md)。既有移动/桌面 canonical 决策不被本文改写。

## 1. 平台优先级

已冻结（[CONSOLE-AGENTHUB-V1-DEC-002/003](../decisions/decision-log.md)）：

- 部署模式：Direct Takeover 先行，Governed 保留完整设计等后端 gate；
- 桌面首发：Windows；macOS/Linux 保留完整设计；
- 移动：iPhone 先于 Android，两者均保留完整设计；
- 手机是 Takeover Host 的远程 companion，不承载本地 Agent runtime / 完整 Vault / CognitiveOS node / authority。

## 2. Windows GA 基线

已冻结（[CONSOLE-AGENTHUB-V1-DEC-004](../decisions/decision-log.md)），按 edition/build/补丁动态 gate：

| 层 | 版本 | 处置 |
|---|---|---|
| 主 GA | Windows 11 25H2（build 26200，滚动最新补丁） | 目标基线 |
| 兼容 | Windows 11 24H2（build 26100） | 现有 Enterprise/Education/LTSC 允许；Home/Pro 2026-10-13 后按 edition 阻断 |
| Experimental/ESU | Windows 10 22H2（build 19045） | 仅证明有效 ESU 且滚动补丁；使用 legacy ConPTY teardown；否则 `unsupported-os` |

运行时 `RtlGetVersion` 取真实 build + 关键 API `GetProcAddress` self-test；版本比较用“≥ floor”，并记录 edition/build/UBR/补丁/servicing 状态。

## 3. 进程/终端能力平台差异（摘要）

详见 [../architecture/process-and-terminal.md](../architecture/process-and-terminal.md)。

| 能力 | Windows | macOS | Linux |
|---|---|---|---|
| 强 containment | Job Object（KILL_ON_JOB_CLOSE） | 进程组 + launchd（弱，双 fork 可逃逸） | cgroup v2 `cgroup.kill`（强） |
| Host-owned 终端 | ConPTY | openpty/forkpty | openpty/forkpty |
| L4 multiplexer | 未来（无安全既有 console 抢占） | tmux/screen 独立 socket（未来） | tmux/screen 独立 socket（未来） |
| 进程身份锚 | handle + PID + creation time | PID + start time + unique/version + code requirement | pidfd + starttime + exe(dev,ino) |

## 4. 移动范围（摘要）

- iPhone/Android 均为 remote companion；能力矩阵见 [docs/platforms/mobile-parity-matrix.md](../../../../../clients/mobile/shared/docs/mobile-parity-matrix.md) 与本目录 parity。
- 手机可查看投影、请求扩权；高后果动作强制 PC-local 确认。
- 无障碍验收见 [../product/states-content-and-accessibility.md](../product/states-content-and-accessibility.md)。

## 5. 平台 gate

各平台实现 gate 见 [docs/platforms/README.md](../../../../../docs/platforms/README.md) 与 [../GOVERNANCE.md](../GOVERNANCE.md#7-实现-gate不可跳过)；当前全部 `not-implemented / evidence none`。
