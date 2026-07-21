# Agent Hub — 证据索引

> 类别：informative traceability ｜ 日期：2026-07-20（2026-07-21 Phase 0：法务备注 + planned PoC 登记）｜ canonical owner：Lane-CON
>
> 本文汇总所有 Open PoC（`CONSOLE-AGENTHUB-V1-POC-*` 短写 `POC-*`）与证据状态。**当前全部 `not-run` 或 `planned`，evidence `none`。** 任何设计文档中的 prevention/detection/oracle 描述都不得被引用为已实现或已验证。
>
> **威胁状态口径**：21 项威胁已规范登记；oracle/evidence 全 not-run——**不是**「21 威胁项实测」。设计摘要见 [../security/threat-test-oracles.md](../security/threat-test-oracles.md)。
>
> **执行准备（非证据）**：[poc-prep-checklist.md](poc-prep-checklist.md)；条目模板 [../templates/open-poc.md](../templates/open-poc.md)；共享记录模板 [../../../shared/docs/poc-execution-record.md](../../../shared/docs/poc-execution-record.md)。

## 1. 证据状态总声明

- machine contract：Direct 多数 `product-only`；Governed 复用部分已登记 `REQ-*` 但关键 carrier 仍缺失。
- implementation：`not-implemented`。
- platform / PoC evidence：`none / not-run`（含下方 planned 新 ID，均未执行）。
- 既有 conformance vectors：`84`（全局 46 `pass` / 38 `not-run`）；这些结果不是 Agent Hub 平台或实现证据。
- Direct / Governed Profile：`not implemented`。

## 2. Open PoC 清单

| PoC ID | 领域 | 断言 | 状态 | 备注 |
|---|---|---|---|---|
| POC-PROC-001 | 进程 | Windows suspended→Job→resume 无逃逸窗口 | not-run | |
| POC-PROC-002 | 进程 | KILL_ON_JOB_CLOSE 在 Host 崩溃路径触发 | not-run | |
| POC-PROC-003 | 进程 | Linux cgroup.kill 对双 fork 树整树终止 | not-run | |
| POC-PROC-004 | 进程 | pidfd/handle 在 PID 复用压力下身份稳定 | not-run | |
| POC-PROC-005 | 进程 | Job breakaway / WMI 逃逸阻止或检测 | planned | 2026-07-21 正式登记；Security Auditor 建议 |
| POC-TERM-001 | 终端 | ConPTY 全动作在 25H2/24H2/22H2-ESU 行为差异 | not-run | |
| POC-TERM-002 | 终端 | OSC/ANSI 注入清洗覆盖 | not-run | |
| POC-TERM-003 | 终端 | 外部 PID stdin 抢占 / PTY 劫持确定性拒绝 | planned | 2026-07-21 正式登记；TM-005 |
| POC-SESS-001 | session | Tier 1 官方 resume/fork/list 跨版本漂移 | not-run | |
| POC-SESS-002 | session | 旧 writer 存活时写 adopt 冲突后果（隔离沙箱） | not-run | |
| POC-FILE-001 | 文件 | JSONL 半写/rotation/truncate parser 处置 | not-run | |
| POC-FILE-002 | 文件 | SQLite WAL 只读 snapshot 写压力一致性 | not-run | |
| POC-FILE-003 | 文件 | symlink/reparse/junction/cloud placeholder 逃逸拒率 | not-run | |
| POC-SEC-001 | 安全 | named pipe DACL + impersonation 拒绝未授权 client | not-run | |
| POC-SEC-002 | 安全 | Unix socket peer credential + pidfd PID 复用稳定 | not-run | |
| POC-SEC-003 | 安全 | credential handle 注入零 secret 落盘 | not-run | |
| POC-SEC-004 | 安全 | 账号切换新 session+handoff 无上下文泄露 | not-run | |
| POC-HOST-001 | Host | single-controller lease + ownership generation 并发拒绝 | planned | 2026-07-21 正式登记；TM-006/014 覆盖缺口 |
| POC-RELAY-001 | Relay | 配对 MITM（matching code 绕过）被拒 | not-run | |
| POC-RELAY-002 | Relay | 重复/乱序/重放幂等与 anti-replay | not-run | |
| POC-RELAY-003 | Relay | revoke/rotation 生效延迟 | not-run | |
| POC-RELAY-004 | Relay | push hint 无敏感字段审计 | not-run | |
| POC-RELAY-005 | Relay | 稳态 E2EE：恶意中继仅见密文、无法伪造明文 | planned | 2026-07-21 正式登记；TM-010 |
| POC-CC-001 | 电脑控制 | selected-window 越窗口输入被拒 | not-run | |
| POC-CC-002 | 电脑控制 | secure desktop 截屏保护如实降级 | not-run | |
| POC-CC-003 | 电脑控制 | 隔离浏览器与用户主会话无凭据交叉 | not-run | |
| POC-COLLAB-001 | 协作 | 单层 Lead+Workers DAG 确定性派发 | not-run | |
| POC-COLLAB-002 | 协作 | 多 Worker worktree 隔离与冲突检测 | not-run | |
| POC-COLLAB-003 | 协作 | handoff scope/预算无越权 | not-run | |
| POC-GOV-001 | 治理 | Direct 完成语言与 CognitiveOS authority 语义隔离 | planned | 2026-07-21 正式登记；TM-019 |
| POC-LIC-001 | 法务 | Paseo AGPL + 第三方组件义务清单 | not-run | **评估材料已整理，法务评估未执行** |
| POC-LIC-002 | 法务 | Tier 1 provider 条款逐项允许性 | not-run | **评估材料已整理，法务评估未执行** |
| POC-LIC-003 | 法务 | 客户端/服务端许可分界方案 | not-run | **评估材料已整理，法务评估未执行** |

### 2.1 本轮正式登记的 planned PoC（未占用编号）

| PoC ID | 来源 | 说明 |
|---|---|---|
| POC-HOST-001 | Security Auditor → 协调者确认 | lease + generation；原仅有 AH-HOST-03 散文 oracle |
| POC-TERM-003 | 同上 | stdin 抢占负例 |
| POC-PROC-005 | 同上 | breakaway / WMI |
| POC-RELAY-005 | 同上 | 稳态密文保密性 |
| POC-GOV-001 | 同上 | Direct vs authority 隔离 |

计数：原 28 项 not-run + 5 项 planned = **33** 登记项；执行证据仍为 none。

## 3. 更新规则

- PoC 真实执行并留证后，在此更新状态与证据路径（`artifacts/` 为 gitignore 运行证据），并按 docs-sync 联动 PROGRESS/findings-ledger。
- `planned` → `not-run` 可在任务领取时改；二者均非 pass。
- 在 evidence 转为 `pass` 前，任何文档不得声明对应能力 implemented 或 Profile 已符合。
