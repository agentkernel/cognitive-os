# cognitiveos-personal 1.0.0 — 边界定义与定稿说明

- Status: **finalized（定稿）** — 1.0.0 开发已完成；本文件与 annotated tag
  `personal-v1.0.0` 共同构成定稿记录
- Date: 2026-08-25
- Decision anchor: [ADR-0054](../../docs/adr/0054-repository-subproject-structure-and-1.0.0-finalization.md)；
  边界语义沿用 ADR-0034/0035/0036/0037/0038，推广 Gate 语义沿用 ADR-0049
- Claim ceiling: 本定稿逐字保留各 Gate 的 **MVP** 声明上限；它不升级任何
  Gate 证据，不构成 Profile、发行分发（distribution/RC）或性能声明

## 1. 一句话定义

`cognitiveos-personal` 1.0.0 是 **Linux x86_64 单服务的统一认知资源基座**：
一个 Rust daemon 作为唯一 authority writer，向 owner-local 单主体提供
Memory、Skill、Tool、Context、Task、Runtime 六类资源的最小真实切片，并只
product-qualify 官方 npm 获取并受管的 exact Pi + per-Agent sidecar。

## 2. 1.0.0 范围（boundary，按 ADR-0037/0038 固定）

- **平台与部署**：Linux x86_64；canonical user service
  `cognitiveos-personal.service`，固定 loopback 端口 48181；Extended Home
  部署边界；desktop Secret Service 或 headless encrypted vault。
- **六资源最小真实切片**：Memory（SQLite FTS5 baseline + lifecycle/forget）、
  Skill（local package/revision/import/binding）、Tool（native catalog +
  六 family 执行 sink）、Context（真实 source/builder/cache + Artifact CAS）、
  Task（intent→interpret→preview→admit→scheduler→independent verifier→
  acceptance 全链路）、Runtime（managed Pi 安装/监督/恢复 + sidecar）。
- **治理不变量**：daemon-only authority（A1）、candidate-only 概率组件
  （A2）、persist-before-dispatch Intent/Effect（A3）、独立 verification
  （A4）、SecretStore-only secrets（A5）、budget/fencing、Tier 0/1/2 低摩擦
  授权（ADR-0026）。
- **Agent 范围**：仅 pinned 官方 Pi + per-Agent sidecar 被 1.0 product-
  qualify；通用 adapter framework 交付但其他 Agent 需独立 qualification。
- **明确不在 1.0.0**：embedding/vector/graph 检索、MCP 与 dynamic Tool
  marketplace（P5-T03/T04 为 post-1.0 能力列车，虽已实现）、Multi-Agent
  （P6，默认关闭）、Web UI（P7-T05）、Windows 安装面（P7-T07/B01-W）、
  clean-VM RC 发行证据（P7-T06）。

## 3. 验收标准与达成证据

1.0.0 的验收 = 三条 release track 的全部 Gate 通过，由 `GMVP-LINUX` 汇合：

| Gate | 含义 | 结论 | 证据 anchor |
|---|---|---|---|
| B01 | 全新 Linux 安装到首次对话（6 次正式 campaign，≥5 成功，0 关键安全失败，独立 verifier 肯定） | **pass** | successor campaign `002`（ADR-0039），closure revision `0ef0b21` |
| B02/B04/B05/B12 | Runtime Spine（Task 闭环、Effect/权威、scheduler、projection） | **pass**（MVP） | ADR-0046 固定 denominator matrix，CI `31407542786` |
| B03 | Context correctness | **pass**（MVP） | ADR-0040，PR #171 |
| B08 | Memory + Skill 资源价值 | **pass**（MVP） | ADR-0048，CI `31479512940` |
| B09 | managed Pi + sidecar qualification | **pass**（MVP） | ADR-0047，CI `31423464703` |
| GMVP-LINUX | Linux 1.0 汇合 Gate（`1.0.0` 的既有发布 Gate，ADR-0035） | **pass**（MVP） | ADR-0049 composition binder，CI `31480604511` @ `b3f4b88` |

辅助事实：B06/B07 保持 non-claim observation；B10（dynamic Tool）与 B11
（Multi-Agent）属 post-1.0 列车；Profile 与 Windows B01-W 未声明。

## 4. 定稿声明

`PERSONAL-DEVELOPMENT-PLAN.md` 将 `GMVP-LINUX` 定义为 Personal `1.0.0` 的
既有发布 Gate；该 Gate 及其全部 acceptance_requires Gate 均已 pass。据此，
owner 于 2026-08-25 定稿 `cognitiveos-personal` 1.0.0：**1.0.0 版本开发已
完成**，范围以本文件 §2 为准，证据以 §3 为准，定稿 revision 为 tag
`personal-v1.0.0`。

诚实边界（逐字保留，不因定稿而改变）：

- 各 Gate 按其 ADR 登记的 **MVP 固定 denominator** 语义通过；定稿不把 MVP
  证据升级为完整统计 campaign、Profile 或 release-distribution 声明。
- P7-T06（RC、clean VM suite、支持矩阵与声明范围内 B01–B12 发行证据）仍
  `not-started`，是 1.0.x 发行工程的唯一归宿；本定稿不替代它。
- P7-T05 Web UI 于 2026-08-25 由 owner 决定在 D14 未完成 rendered review 的
  状态下强制合并收口（见 PARALLEL-LANES 关闭记录与 W5 pause handoff）；其
  后续 wave 属 post-1.0。

## 5. Post-1.0 路线图（登记归宿）

| 列车 | 任务归宿 | 状态 |
|---|---|---|
| Multi-Agent（默认关闭；NO-GO 合法） | P6-T01..T04 / B11 | not-started |
| Control Plane Web UI 后续 wave（W6+、legacy 收尾、rendered review 补全） | P7-T05 续（需新 lease） | paused by owner |
| RC / 发行证据 | P7-T06 | not-started |
| Windows 安装面 | P7-T07 / B01-W | blocked（B01-W 前置） |
| Embedding/vector/graph、MCP marketplace、非 Pi Agent qualification | P4 后续 / P5B / P8 模式 | 设计/能力列车 |
| 混合 crate core/personal 内部拆分 | ADR-0054 登记的结构重构 | not-started |
