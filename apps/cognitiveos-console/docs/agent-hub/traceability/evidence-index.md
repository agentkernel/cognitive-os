# Agent Hub — 证据索引

> 类别：informative traceability ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 本文汇总所有 Open PoC（`CONSOLE-AGENTHUB-V1-POC-*`）与证据状态。**当前全部 `not-run`，evidence `none`。** 任何设计文档中的 prevention/detection/oracle 描述都不得被引用为已实现或已验证。

## 1. 证据状态总声明

- machine contract：Direct 多数 `product-only`；Governed 复用部分已登记 `REQ-*` 但关键 carrier 仍缺失。
- implementation：`not-implemented`。
- platform / PoC evidence：`none / not-run`。
- 既有 conformance vectors：`76`，全部 `not-run`。
- Direct / Governed Profile：`not implemented`。

## 2. Open PoC 清单

| PoC ID | 领域 | 断言 | 状态 |
|---|---|---|---|
| POC-PROC-001 | 进程 | Windows suspended→Job→resume 无逃逸窗口 | not-run |
| POC-PROC-002 | 进程 | KILL_ON_JOB_CLOSE 在 Host 崩溃路径触发 | not-run |
| POC-PROC-003 | 进程 | Linux cgroup.kill 对双 fork 树整树终止 | not-run |
| POC-PROC-004 | 进程 | pidfd/handle 在 PID 复用压力下身份稳定 | not-run |
| POC-TERM-001 | 终端 | ConPTY 全动作在 25H2/24H2/22H2-ESU 行为差异 | not-run |
| POC-TERM-002 | 终端 | OSC/ANSI 注入清洗覆盖 | not-run |
| POC-SESS-001 | session | Tier 1 官方 resume/fork/list 跨版本漂移 | not-run |
| POC-SESS-002 | session | 旧 writer 存活时写 adopt 冲突后果（隔离沙箱） | not-run |
| POC-FILE-001 | 文件 | JSONL 半写/rotation/truncate parser 处置 | not-run |
| POC-FILE-002 | 文件 | SQLite WAL 只读 snapshot 写压力一致性 | not-run |
| POC-FILE-003 | 文件 | symlink/reparse/junction/cloud placeholder 逃逸拒率 | not-run |
| POC-SEC-001 | 安全 | named pipe DACL + impersonation 拒绝未授权 client | not-run |
| POC-SEC-002 | 安全 | Unix socket peer credential + pidfd PID 复用稳定 | not-run |
| POC-SEC-003 | 安全 | credential handle 注入零 secret 落盘 | not-run |
| POC-SEC-004 | 安全 | 账号切换新 session+handoff 无上下文泄露 | not-run |
| POC-RELAY-001 | Relay | 配对 MITM（matching code 绕过）被拒 | not-run |
| POC-RELAY-002 | Relay | 重复/乱序/重放幂等与 anti-replay | not-run |
| POC-RELAY-003 | Relay | revoke/rotation 生效延迟 | not-run |
| POC-RELAY-004 | Relay | push hint 无敏感字段审计 | not-run |
| POC-CC-001 | 电脑控制 | selected-window 越窗口输入被拒 | not-run |
| POC-CC-002 | 电脑控制 | secure desktop 截屏保护如实降级 | not-run |
| POC-CC-003 | 电脑控制 | 隔离浏览器与用户主会话无凭据交叉 | not-run |
| POC-COLLAB-001 | 协作 | 单层 Lead+Workers DAG 确定性派发 | not-run |
| POC-COLLAB-002 | 协作 | 多 Worker worktree 隔离与冲突检测 | not-run |
| POC-COLLAB-003 | 协作 | handoff scope/预算无越权 | not-run |
| POC-LIC-001 | 法务 | Paseo AGPL + 第三方组件义务清单 | not-run |
| POC-LIC-002 | 法务 | Tier 1 provider 条款逐项允许性 | not-run |
| POC-LIC-003 | 法务 | 客户端/服务端许可分界方案 | not-run |

## 3. 更新规则

- PoC 真实执行并留证后，在此更新状态与证据路径（`artifacts/` 为 gitignore 运行证据），并按 docs-sync 联动 PROGRESS/findings-ledger。
- 在 evidence 转为 `pass` 前，任何文档不得声明对应能力 implemented 或 Profile 已符合。
