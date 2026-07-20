# Agent Hub 依赖 DAG

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON

## 车道依赖图

```text
GOV ─┬─────────────────────────────────────────────► (贯穿所有车道：术语/状态/完成语言)
     │
CTR ─┴─► HOST ─► PROC ─┐
                        ├─► DESK ─► RELAY ─► IOS ─► MULTI ─┐
        HOST ─► SESS ───┤                         AND ─────┤
        HOST ─► CRED ───┘                                  └─► QRM(发布/迁移)
CTR ─► [6× Adapter] ─► (依附 HOST/PROC/SESS/CRED，按各自接口 gate)
```

## 关键依赖说明

- **GOV** 贯穿全程：术语、状态用语、完成语言、文案守则先冻结，其他车道遵循。
- **CTR** 是能力/接口/契约前置：接口未核验则 Adapter 与 HOST 高层能力 `blocked`。
- **HOST** 是 PROC/SESS/CRED 的公共基座（控制面、lease、generation、ledger）。
- **DESK** 依赖 HOST+PROC+SESS+CRED 提供的本机能力；**RELAY** 在桌面本机能力稳定后接入。
- **IOS/AND** 依赖 RELAY 与桌面投影；iPhone 先于 Android。
- **MULTI** 依赖单 Agent 全旅程稳定（DESK）与调度器基座（HOST）。
- **QRM** 收口发布 gate 与 Direct→Governed 迁移，依赖 Governed 契约（外部 M6）。
- 每个 **Adapter** 依附 HOST/PROC/SESS/CRED，但按各自接口/条款/PoC gate 独立解锁，互不阻塞。

## 外部依赖（gate）

- Console 后端组 1/2/7 + M5（来自 `docs/plan/DEVELOPMENT-PLAN.md`，仓库路径 `../../../docs/plan/DEVELOPMENT-PLAN.md`）。
- 平台 PoC/GA gate（canonical：`clients/governance/readiness-gates.md`，路径 `../../governance/readiness-gates.md`）。
- Governed `REQ-AGENT-*` 等契约（经 Lane-CTR，外部 M6）。
- Paseo/AGPL 法务 gate。

## 并行性

- GOV、CTR 可先行（文档/核验，不含实现）。
- HOST 就绪后 PROC/SESS/CRED 可并行。
- 6 个 Adapter 在接口 gate 通过后可并行，互不阻塞。
- 所有并行仍受全局 gate 与车道所有权约束（不跨车道改他人 crate/package）。
