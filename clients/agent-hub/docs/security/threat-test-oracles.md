# Agent Hub — 威胁测试 Oracle 设计摘要（informative）

> 类别：informative security design ｜ 日期：2026-07-21 ｜ owner：Lane-CON
>
> 收录 Security Auditor 建议的 **约 30 条**测试 oracle 设计摘要，供 AH-M2/M3/M5/M6 出口规划。**全部为设计意图，零条已执行**；不得引用为「威胁项实测」或测试已执行证据。
>
> 配套：`threat-model.md`（21 项威胁已规范登记，oracle/evidence 全 not-run）；`../traceability/evidence-index.md`（Open PoC，含本轮正式登记的 planned 新 ID）。

## 0. 状态纪律

- **正确说法**：21 项威胁已规范登记（specified）；oracle/evidence 全 `not-run` / `none`。
- **禁止说法**：「21 威胁项实测」「已测试通过」「Profile 已符合」。
- 新 PoC ID 已由协调者正式登记为 `planned`（见 evidence-index §2.1）；执行前仍须真实 API/OS 注入与留证。

## 1. AH-M2 出口（Host / Process / Terminal）— oracle 摘要

| Oracle ID | 场景要点 | 注入/机制 | 映射 TM / PoC |
|---|---|---|---|
| AH-ORCL-M2-01 | 未授权 client 调控制面 | 异 SID/无 capability 连 pipe/socket | TM-001 / POC-SEC-001 |
| AH-ORCL-M2-02 | pipe/socket squatting | 先占名假 Host；client 反验 server | TM-002 / POC-SEC-001 |
| AH-ORCL-M2-03 | PID reuse 零误杀 | 高频 PID 复用压力；handle/pidfd | TM-003 / POC-PROC-004 |
| AH-ORCL-M2-04 | Job/cgroup 崩溃连带 | Host 强杀；KILL_ON_JOB_CLOSE / cgroup.kill | TM-015 邻接 / POC-PROC-002/003 |
| AH-ORCL-M2-05 | suspended→Job→resume 无逃逸窗 | Windows 创建序 | POC-PROC-001 |
| AH-ORCL-M2-06 | **stdin 抢占 / PTY 劫持拒** | TIOCSTI / AttachConsole / `/proc/PID/fd/0` | TM-005 / **POC-TERM-003** |
| AH-ORCL-M2-07 | **breakaway / WMI 逃逸** | JOB_OBJECT_LIMIT_BREAKAWAY*；WMI 建子进程 | TM-015 / **POC-PROC-005** |
| AH-ORCL-M2-08 | OSC/ANSI 注入清洗 | 注入样本库 | TM-004 / POC-TERM-002 |
| AH-ORCL-M2-09 | **single-controller lease + generation** | 双 controller；旧 generation 重放 | TM-006/014 / **POC-HOST-001** |
| AH-ORCL-M2-10 | 忽略信号不误报 stopped | 屏蔽 SIGINT；状态机 unknown | TM-017 |
| AH-ORCL-M2-11 | ledger 零 secret | 含 token 输入全链扫描 | TM-009 / POC-SEC-003 |
| AH-ORCL-M2-12 | 跨用户/提权目标不可写 | 管理员/他 UID 目标 | TM-018 |

## 2. AH-M3 出口（Session / File / Credential）— oracle 摘要

| Oracle ID | 场景要点 | 映射 |
|---|---|---|
| AH-ORCL-M3-01 | 双 writer 写 adopt 不可达 | TM-006 / POC-SESS-002 |
| AH-ORCL-M3-02 | symlink/junction/hardlink/TOCTOU 全拒 | TM-007 / POC-FILE-003 |
| AH-ORCL-M3-03 | JSONL 半写/rotation 不伪完成 | TM-016 / POC-FILE-001 |
| AH-ORCL-M3-04 | SQLite WAL 只读一致 snapshot | TM-016 / POC-FILE-002 |
| AH-ORCL-M3-05 | credential handle 零 secret 落盘 | TM-009 / POC-SEC-003 |
| AH-ORCL-M3-06 | L6 写阻断 & L8 无入口 | TM-008（候选扩展 PoC，本回合未新开 ID） |
| AH-ORCL-M3-07 | 账号切换 handoff 无泄露 | POC-SEC-004 |
| AH-ORCL-M3-08 | per-agent 工作区隔离 | POC-COLLAB-002 扩展 |
| AH-ORCL-M3-09 | cloud placeholder 不静默下载 | TM-007 |
| AH-ORCL-M3-10 | 完成判定双轴（check + user-accepted） | DEC-026 |

## 3. AH-M5 出口（Relay / Pairing）— oracle 摘要

| Oracle ID | 场景要点 | 映射 |
|---|---|---|
| AH-ORCL-M5-01 | 配对 MITM（matching code 绕过）拒 | TM-010 / POC-RELAY-001 |
| AH-ORCL-M5-02 | 重放/乱序幂等 | TM-011 / POC-RELAY-002 |
| AH-ORCL-M5-03 | revoke/rotation 延迟 | TM-012 / POC-RELAY-003 |
| AH-ORCL-M5-04 | push hint 无敏感字段 | POC-RELAY-004 |
| AH-ORCL-M5-05 | **稳态 E2EE：恶意中继只见密文** | TM-010 / **POC-RELAY-005** |
| AH-ORCL-M5-06 | 设备丢失后待处理队列作废 | TM-012 |

## 4. AH-M6 出口（Governance / Multi-Agent）— oracle 摘要

| Oracle ID | 场景要点 | 映射 |
|---|---|---|
| AH-ORCL-M6-01 | **Direct 完成语言不混入 CognitiveOS authority** | TM-019 / **POC-GOV-001** |
| AH-ORCL-M6-02 | Lead+Workers DAG 确定性派发 | POC-COLLAB-001 |
| AH-ORCL-M6-03 | Worker worktree 隔离与冲突 | POC-COLLAB-002 |
| AH-ORCL-M6-04 | handoff scope/预算无越权 | POC-COLLAB-003 |
| AH-ORCL-M6-05 | 手机高后果动作须 PC-local 确认 | TM-013 |
| AH-ORCL-M6-06 | fail-closed：歧义即拒/unknown | 多 TM 横切 |

## 5. 更新规则

- 本文件只增删 oracle **设计**摘要；执行结果只写入 evidence-index / artifacts（gitignore）。
- 新增正式 PoC ID 须同步 evidence-index，并保持 `planned` 或 `not-run` 直至真实执行。
