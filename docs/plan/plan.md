# CognitiveOS Personal 产品化研究与任务卡草案

> **项目身份：** `cognitiveos-personal` 是当前唯一活动实现项目；原 CognitiveOS 设计、
> 规范和通用实现是其架构/合同基础，不是并行项目。身份与工作范围以
> [PROJECT-IDENTITY.md](../governance/PROJECT-IDENTITY.md) 为准。
> **文档状态：研究与任务卡草案；不代表实现已提供、测试已执行或 Profile 已符合。**
> **正式开发计划与进度台账：** [docs/plan/PERSONAL-DEVELOPMENT-PLAN.md](PERSONAL-DEVELOPMENT-PLAN.md)。后续开发完成任一部分时，必须更新该文件对应任务的状态、日期和证据。
> **研究与审计日期：2026-07-24。**
> **审计基线：`origin/main@9b53cf4c6c2b744a60283c3ea1431a9d1090aafd`。**
> **2026-07-26 一致性评审批：** 任务卡不再承载正式状态行；正式状态、完成日期与证据一律以台账为准。§3/§6 为审计日快照，此后交付不逐项回写。本批新增 P7-T07（Windows 安装面归宿）、P2-T08 增补 ADR-0018 例外到期核查、§9 改用 DEC-P-* 编号、§12 修正依赖图与 critical path。
> **生产就绪与低摩擦授权批（2026-07-26）：** 新增 DEC-P-20 授权交互模型并落地 [ADR-0026](../adr/0026-personal-trust-profile-low-friction-authorization.md)（Tier 0/1/2 分层、准入预览为唯一默认授权点、预算硬轨、不建审批链）；P2-T01/P2-T02/P2-T08/P5-T01/P5-T02/P7-T02 卡增补对应验收 bullet；§14 增 Approval Interactions/Task 指标；§16 R-22 与 §17 企业审批行改引 ADR-0026。
> **P2 卡扩写批（2026-07-26）：** 依 §11.1 状态纪律，将 P2-T01..P2-T08 压缩卡预先扩写为完整强制字段集：仅补足字段、仓库锚点与既有决策引用（ADR-0026/0018、§12.1/§12.2、§13/§14/§15）；任务范围、依赖、验收语义与 §12 依赖图均不变。documentation-only；本批 §15.2 命令因环境阻断未执行（记 not-run，见当日 handoff），不改变任何任务状态、Gate、证据或 Profile 结论。
> **MVP-first 对齐（2026-07-29，ADR-0034）：** 保留现有任务 ID，首个生产安装路径改为 single canonical user service/48181；新增 P7-T08 / `GMVP-LINUX`。该日曾将 P3/P4 放到 scoped MVP 后，已由 2026-08-02 ADR-0037/0038 的六资源 Linux 1.0 重基线取代；P5 Tool/MCP、Windows、Web UI 与 Multi-Agent 仍不阻塞。ADR-0036 将 P5 managed-Pi/B09 slice 加入 Linux 1.0 promotion path。Multi-Agent 仍为独立 go/no-go，NO-GO 且默认关闭是合法结果。§2.1 仍是 2026-07-24 审计快照，不用于覆盖正式台账当前状态。
> **开发治理对齐（2026-07-30）：** task status、implementation evidence、Gate 和 claim scope 正交记账；在该日记录中 P1-T09 为 `in-progress` / `tested-local`，B01 为 `not-run`。当前事实只见 `PROGRESS.md` Current snapshot。Gate/阶段依赖不再作为 isolated implementation mutex；B01 的 attempt、阈值、零容忍失败与 cleanup 边界已明确。工具无关规则见 [Development Operating Model](../governance/DEVELOPMENT-OPERATING-MODEL.md)。
> **Linux 1.0 / managed Pi 对齐（2026-08-02，ADR-0035/0036）：** 正式计划将既有 `GMVP-LINUX` 定义为 Personal `1.0.0` release Gate。Pi-hosted Agent Shell 与 managed Pi 是独立角色；P2-T02 负责 Shell/application-service composition，P5-T01/T02/B09 负责 official npm acquisition、installation、registry/instance/supervision/lifecycle。Pi 是 1.0 唯一 product-qualified Agent；通用 adapter framework 保留给后续 Agent 的独立 qualification。本文件只同步详细卡片，不拥有当前状态。
> **统一认知资源与 sidecar 重基线（2026-08-02，ADR-0037/0038）：** owner 已批准 Personal 作为 Memory/Skill/Tool/Context/Task/Runtime 统一认知资源基座，六类最小真实 slice 进入 Linux 1.0；Agent 路径采用 per-Agent sidecar-first，Pi 仍是唯一 qualified Agent。1.0 由 Runtime Spine、Resource Value、Product Operability 三条 active track 汇合；Context correctness 与 Memory+Skill actual consumption 进入 promotion，复杂 Context 收益、embedding/vector/graph、MCP/dynamic Tool、Multi-Agent、Web UI 和 Windows 后置。本批不改变任何 task status、attempt、evidence、Gate current status 或 Profile：P1-T09 仍 `in-progress`、B01 仍 `running`（1/至少20）、P2-T01/P2-T03 仍 `in-progress`、`GMVP-LINUX` 仍 `not-run`、Profile `implemented: 0`。本文件仍不是状态源；该 product-semantic + structural 文档批不含实现、规范变更或 Gate/release/Profile evidence。
> **B01 successor campaign amendment（2026-08-09，ADR-0039）：** owner set separately preregistered successor `002` to fixed N=6, at least 5 successes, zero critical safety failures, complete aggregate statistics, and affirmative independent-verifier closure. Attempts 1--6 remain immutable at 5 successes / 1 failure. Transition Attempt 7 is retained but owner-waived outside the denominator because no artifact, Pi, Provider, service, or route operation occurred. Retained `001` remains its historical N=20 failure. This decision does not claim B01, release, or Profile pass; `PROGRESS.md` remains the current-status source.
> **2.0 设计基线与计划扩展（2026-08-10，P8-T01 / ADR-0041+）：** 正式台账新增 Phase 8/9；公理单一文档见 `docs/governance/AXIOMS.md`；产品定位为认知资源操作系统与主流 Agent 统一管理底座。本文件只补充研究层卡片与依赖说明，不拥有当前状态；Multi-Agent 设计正线化但 Linux 1.0 claim 仍仅资格化 Pi。
> **Personal 2.0 desktop + MCP semantic adoption（2026-08-27，P10-T01 /
> ADR-0056/0057）：** 正式台账新增 Phase 10。Personal 2.0 采用 desktop-first
> Control Plane 与 Home/Agents/Work/Library/Activity/Settings IA（Providers、System
> 归 Settings），global Agent Shell 仅产 candidate，installed-Agent conversation
> 通过 vendor-specific adapter + common internal capability projection，native Agent app
> 仍可使用；MCP 成为第七 family，advertised tools 仍走 Tool candidate，resources/prompts
> 走 Context/Skill admission，不建 generic Resource schema。Linux/Personal 1.0 六 family
> 定稿不变；本文件不拥有 P10-T01 当前状态。
> **本草案不包含生产代码、规范或数据库 Schema 变更。**
> **落盘说明：** 正式计划与进度台账已保存于 `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`；本文件保留研究结论、详细任务卡和原始审计材料。

---

## 1. 文档层级与使用方式

`docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` 是 Personal 开发的唯一正式入口和进度台账：任务 ID 的名称、范围、依赖、状态和阶段 Gate 以该文件为准。

本文件保留研究结论、详细任务卡、ADR 候选和机器可读依赖图。开发会话领取任务时，应先在正式台账确认对应 `P*-T*` 的定义与状态，再阅读本文件中相同 ID 的实施细节；不得以本文件重新定义、交换或复用任务 ID。未来若需拆分本文件，必须先由 Lane-DOC 批准目录和迁移方案，且不得影响正式台账的 canonical 地位。

---

# 2. 执行摘要

## 2.1 2026-07-24 历史审计快照（非当前状态）

> 本节只保存审计时事实，不能覆盖正式计划或 `PROGRESS.md` Current snapshot。

在 2026-07-24 审计时，CognitiveOS 尚不是可用的 Personal 产品，而是一个具有较强
确定性内核、规范合同和符合性基础的参考实现：

- 合同、生成绑定、状态迁移、CAS、事件、预算、Intent/Effect、恢复、授权和 Context 解析具备 L3-L4 基础；
- Rust/TS 两端在远端 Windows/Linux CI 通过；
- 当前没有可供普通个人用户使用的：
  - 安装器；
  - `cognitive init`；
  - 原生 Secret Store；
  -通用 Provider 服务；
  - 可用 daemon API；
  - Pi Interactive Package/Extension；
  - Task 调度器；
  - Process Runtime；
  - Tool Registry；
  - durable Memory；
  - Personal E2E；
  - 升级和卸载流程。

因此，当前不适合直接从 Memory、多 Agent 或 Web UI 开始。这样会产生第二套 Runtime、第二套状态、未受治理的 Pi/Bash 路径，以及无法安装和验证的功能孤岛。

## 2.2 最短产品化路径

```text
Linux x86_64 可验证安装包
→ XDG 数据布局与数据库迁移
→ desktop Secret Service / headless encrypted-vault `SecretStore`
→ DeepSeek/OpenAI-compatible Provider 探测
→ 单一 Rust daemon
→ cognitive init / doctor / status
→ Pi 官方交互式 CLI + CognitiveOS Extension
→ 首次对话
→ TaskApplicationService
→ real Resource + Task API/watch/private versioned projection
→ durable scheduler→real Context→pinned Pi sidecar→BoundedHarness
→ native Tool family + bounded Tool/process executor
→ checkpoint/unique Artifact CAS/evidence/recovery
→ independent verifier
→ B01/B02/B04/B05/B12
→ real Context correctness + Memory/Skill lifecycle/actual consumption
→ B03/B08（同时采集但不要求 B06/B07 收益）
→ official Pi + sidecar acquisition, registration and managed lifecycle
→ B09
→ six-resource manifest + production trust/update/uninstall/backup/doctor
→ GMVP-LINUX / Personal 1.0.0
```

## 2.3 关键决策

1. **默认 Pi 集成采用 sidecar-first，不是 SDK authority。**
   - 默认交互路径为 **Pi Interactive CLI + presentation-only Extension + pinned per-Agent Pi sidecar + daemon application services**。
   - Pi sidecar/RPC 是 client transport；Extension 与 sidecar 都不得拥有 daemon bootstrap/management authority。
   - Pi SDK 只在未来确实需要自建 UI 且有测量收益时重评。

2. **daemon 使用 Rust。**
   - 复用现有 kernel/runtime/store/management；
   - daemon 是唯一 authority composition、scheduler 和 SQLite writer。

3. **SQLite 继续使用，但先补迁移和运维体系。**
   - 不引入 Temporal、PostgreSQL、Redis 或消息队列作为 Personal v1 前置条件。

4. **Provider Secret 只保存在 approved `SecretStore` backend。**
   - Desktop 使用 Secret Service；headless 使用 encrypted vault，locked start + SSH
     TTY unlock，可选 systemd encrypted credential 只承载 vault-unlock material。
   - Pi、service unit/credential、配置文件、SQLite、环境继承、命令参数和日志不得持有原始 API Key；唯一
     窄幅例外是 ADR-0018 的 P0-T06 本机 Linux 开发路径：默认拒绝、精确显式开启，
     仅从 native store 注入初始 Pi child，且在 P2 结束到期。它不适用于 CI、发布或
     containment 声明。

5. **Native Tool 进入 1.0；Personal 2.0 将 MCP 从 transport 扩展为第七 family。**
   - Linux/Personal 1.0 的 P5 MCP 路径仍只是 post-1.0 Tool transport/dynamic
     ecosystem evidence；它不改变 six-family 1.0 boundary。
   - Personal 2.0 中 MCP server/package/connection/capability/binding/health/quarantine
     是独立 family identity/lifecycle；advertised tools 仍是 Tool candidate，
     resources/prompts 经 Context/Skill admission。discovery、connection 和 protocol
     completion 都不等于 CognitiveOS 授权、Effect 提交或 Task 完成。

6. **Memory + Skill 是 1.0 Resource Value，而 embedding 后置。**
   - Memory 采用 SQLite source-of-record + FTS5/metadata filter；Skill 采用 local package/revision/import/binding。
   - Embedding/vector/graph/semantic retrieval 是 post-1.0 独立决策，不阻塞 B08 或 GMVP-LINUX。

7. **Multi-Agent 不进入首个 Personal v1 关键路径。**
   - 只有受治理单 Agent benchmark 与明确并行收益假设具备后才能进入 Phase 6；Memory、
     MCP/B10 或其他非 Pi adapter 不应成为 isolated implementation mutex。

8. **版本化 family 统一呈现但不统一 authority schema。**
   - Linux/Personal 1.0 的 Memory、Skill、Tool、Context、Task、Runtime 六 family
     保持 finalized；Personal 2.0 增 MCP 为第七 family。
   - 各 family 使用 private versioned Personal projection；P10-T02 在 Lane-CTR
     决定 MCP/conversation 的 public/private compatibility surface。
   - 不新增 `Process` domain 或 giant `Resource` schema；public prerequisite 逐项走 Lane-CTR。

## 2.4 Linux 1.0 三条 active release track

| Track | 正式任务范围 | Promotion contribution |
|---|---|---|
| Runtime Spine | P1-T08/T09、P2-T01..T08、P5-T01/T02/T05 B09 | B01/B02/B04/B05/B09/B12 |
| Resource Value | P3-T01..T06、P4-T01..T06 | B03/B08；B06/B07 仅采集 |
| Product Operability | P7-T01..T03、P7-T08 | exact GMVP composition 与 release-operability evidence 汇合 |

P3/P4 是 1.0 active track，不是 1.0 后 Beta。Post-1.0 单列 embedding/semantic
retrieval/vector/graph、MCP/dynamic Tool、Multi-Agent、Web UI/Windows 与 non-Pi adapters。

## 2.5 阶段数量

共十二个阶段（Phase 0 至 Phase 11）：

- Phase 0：研究、基线和决策；
- Phase 1：安装到首次对话；
- Phase 2：单 Agent 任务闭环；
- Phase 3：Context、Token 和 Loop 效率；
- Phase 4：Memory；
- Phase 5：Agent 与 Tool 生态；
- Phase 6：Multi-Agent；
- Phase 7：产品化和发布；
- Phase 8：通用 Agent 适配与 2.0 设计基线（post-1.0）；
- Phase 9：性能与结构演进候选池；
- Phase 10：已显式退出 current 2.0 主链的 desktop/MCP 计划（T01/T02 done，其余 cancelled）；
- Phase 11：Windows-first OPC Project、Digital Employee、managed DSH、Conversation/Memory/Vault、Routine/Inbox/Provider/UI/X acceptance。

## 2.6 最大风险

最大风险不是模型效果，而是：

> **Pi、CLI、daemon、Task、Process 和 Memory 各自形成一套状态，导致双 Runtime、假恢复和假完成。**

防线是：

- daemon 单 writer；
- Pi 非 authority；
- 所有变更操作走 Intent/Effect；
- Task 完成只由独立 verifier 和 acceptance authority 推进；
- Session ID 永远不等于 Task ID；
- 对外工具成功永远不等于 Task 完成。

---

# 3. 仓库版本、环境和运行基线

## 3.1 基线清单

| 项目 | 审计结果 |
|---|---|
| Git 基线 | `9b53cf4c6c2b744a60283c3ea1431a9d1090aafd` |
| 分支 | `lane/doc-cognitiveos-personal-plan`，从刷新后的 `origin/main` 创建 |
| 初始 worktree | 创建时 clean；研究文档生成后的最终 dirty state 本轮未重新宣称 |
| OS | Windows 10 Pro 10.0.19045，AMD64 |
| Rust | 1.97.1 |
| Node | 24.15.0；CI 为 Node 22 |
| pnpm | 10.33.2 |
| Rust workspace | 11 members |
| TS workspace | contracts-ts、sdk-ts、agent-shell、tools 等 |
| 数据库 | SQLite WAL；authority store 与 installation store |
| 服务入口 | `personal/apps/kernel-server/src/main.rs` |
| 管理 CLI | `personal/apps/admin-cli/src/main.rs` |
| Pi 候选适配器 | `personal/apps/pi-agent-adapter/src/main.rs` |
| TS Shell | library，无 `bin` |
| CI | `.github/workflows/ci.yml`，Windows/Linux |
| License | **Apache-2.0**（P0-T03 / ADR-0025）；Rust `publish=false`；TS `private: true`；Pi/Node 不 vendor |
| 部署 | 无 Dockerfile/Compose、安装器、Homebrew、OS service、release workflow |
| 迁移 | 无版本化 SQL migrations 框架（审计日快照；P1-T01 已交付 adapter 内嵌 migrations，正式状态见台账） |

## 3.2 已执行命令

| 命令 | 结果 | 结论 |
|---|---:|---|
| `pnpm install --frozen-lockfile` | 0 | lockfile 当前可安装 |
| `pnpm -r build` | 0 | TS build 通过 |
| `pnpm -r test` | 0 | contracts 39；tools 2；SDK 69 pass/3 skip；Shell 13 |
| `pnpm run check:consistency` | 0 | 273 REQ、55 errors、63 schemas、85 vectors |
| `node tools/src/gen-matrix.mjs --check` | 0 | matrix 当前 |
| `cargo fmt --all -- --check` | 0 | 格式通过 |
| `cargo build --workspace --locked` | 101 | 本机 LLVM-MinGW 缺 `libgcc_eh/libgcc` |
| `cargo test --workspace --locked` | 101 | 在第三方 build script 链接阶段失败，未执行仓库 Rust tests |
| `cargo clippy ...` | 101 | 同一环境错误 |
| `pnpm run verify:local` | 1 | L0 停止，诚实记录 BOOT build failure |

远端 GitHub Actions run `30067192424` 对 `main@9b53cf4` 的 Windows/Linux jobs 为 green。该 CI 是当前实现证据，但不等于 Personal B01-B12 或 Profile 证据。

### Personal experimental-local-only 轨道（2026-07-26）

产品 Gate 只控制正式产品主路径、Profile 和 release；不再阻断本机或隔离环境中的
P1/P2/P3 及后续开发。该轨道允许真实本机 DeepSeek/Pi 调试、Extension/RPC load、
Pi proxy、Task runtime、scheduler、Effect、recovery、Context、Memory，以及
kernel/store/runtime benchmark harness。所有结果必须标为 `experimental-local-only`
或真实执行后的 `tested-local`，不得冒充正式验收。

Personal 端到端性能 runner 先分离四类成本：CognitiveOS deterministic overhead、
Pi/Node process overhead、Provider/network/model latency、filesystem/SQLite overhead。
固定平台 campaign 和 A/B/C/D agent-benefit 测评后置；在预注册 workload、固定拓扑和
独立 verifier 完成前，性能结果只允许 non-claim。

以下边界不可协商：secret 不进入日志、argv、普通配置、SQLite、证据或 CI；Pi、CLI、
SDK、UI 不成为 authority；sample/fixture/smoke/局部测试不成为 Profile/release
证据；规范向量不得为实现迎合而改写。

---

# 4. 结构化研究结论

## 4.1 关键研究证据表

| ID | 主题、来源与版本 | 访问日期 | 关键结论 | 限制与适用条件 | 采纳与计划影响 |
|---|---|---|---|---|---|
| PI-01 | [Pi repository](https://github.com/earendil-works/pi)；MIT | 2026-07-24 | Pi 提供 TUI、工具、Session、Extensions、Skills、Templates、SDK、RPC | Pi 明确没有内建 permission system，扩展拥有进程完整权限 | 采纳 TUI，不采纳其权限模型；对应 P1-T07、P1-T08 |
| PI-02 | [`@earendil-works/pi-coding-agent` 0.82.0](https://registry.npmjs.org/@earendil-works%2fpi-coding-agent/latest) | 2026-07-24 | 当前 registry observation 为 0.82.0，Node `>=22.19.0`，带 npm provenance URL | volatile；不能浮动安装；attestation 未经 CognitiveOS verifier 验证 | 固定版本、integrity、source commit；P0-T06、P7-T01 |
| PI-03 | [Pi RPC docs](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/docs/rpc.md) | 2026-07-24 | RPC 是严格 LF JSONL，命令响应和异步事件分离 | prompt accepted 不等于执行成功；TUI 不保留 | 后台 Agent 才采用；P6 前不得接主路径 |
| PI-04 | [Pi SDK docs](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/docs/sdk.md) | 2026-07-24 | `AgentSession`/runtime 可嵌入 Node | 扩大 Node TCB，必须自行重建 UI/生命周期 | Personal 默认不采纳；仅 ADR 重评条件 |
| PI-05 | [Pi Extensions](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/docs/extensions.md) | 2026-07-24 | Extension 可注册 provider/tool/command/UI，拦截事件 | Extension 任意代码、完整系统权限；project trust 不是 OS sandbox | 仅做 daemon client 和 UI；禁直接 mutating tools |
| PI-06 | [Pi Packages](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/docs/packages.md) | 2026-07-24 | npm/git/local package 和固定版本受支持 | package、skill 均可能执行危险行为 | CognitiveOS Package 必须固定、验证、受控加载 |
| PI-07 | [Pi Providers](https://raw.githubusercontent.com/earendil-works/pi/main/packages/coding-agent/docs/providers.md) | 2026-07-24 | Pi 可用 env 或 `auth.json`，甚至执行命令解析 Key | 不符合 daemon-owned secret boundary | Personal 禁用 Pi credential authority；P1-T02/P1-T07 |
| DS-01 | [DeepSeek `/models`](https://api-docs.deepseek.com/api/list-models/) | 2026-07-24 | 返回当前账户可见模型 ID；示例为 `deepseek-v4-flash/pro` | 只证明可见，不证明 chat/tool/stream/cancel 能力 | 模型发现后必须主动 probe；P1-T03 |
| DS-02 | [DeepSeek Tool Calls](https://api-docs.deepseek.com/guides/tool_calls/) | 2026-07-24 | 模型只产生 function request，由调用者执行 | strict 是 beta，只支持 JSON Schema 子集 | Tool call 只转成 OperationCandidate；P2-T05 |
| MCP-01 | [MCP lifecycle 2025-06-18](https://modelcontextprotocol.io/specification/2025-06-18/basic/lifecycle) | 2026-07-24 | 有版本协商、capability、timeout、shutdown | protocol capability 不等于 CognitiveOS capability | MCP 仅作为 Tool adapter；P5-T03 |
| WF-01 | [Temporal workflow execution](https://docs.temporal.io/workflow-execution) | 2026-07-24 | durable history/replay/timers/activities | 与现有 authority event、fencing、Effect 语义重叠且引入重基础设施 | Personal v1 不采纳；若 SQLite scheduler 达不到需求才重评 |
| LG-01 | [LangGraph persistence](https://docs.langchain.com/oss/python/langgraph/persistence) | 2026-07-24 | 区分 thread checkpoint 与 cross-thread store | 框架 checkpoint 不满足 CognitiveOS authority/recovery 顺序 | 仅吸收概念，不嵌入 Runtime |
| OSS-LETTA | [Letta Code](https://github.com/letta-ai/letta) | 2026-08-02 | Agent memory/Context UX 的 informative 对照 | 外部 store/agent state 不是 CognitiveOS authority | 仅吸收可解释资源 UX，不继承证据 |
| OSS-OH | [OpenHands ACP](https://github.com/All-Hands-AI/OpenHands) | 2026-08-02 | Agent/client protocol 与 headless lifecycle 对照 | protocol 存在不等于 adapter qualification | 研究 sidecar ports，不纳入 1.0 claim |
| OSS-GOOSE | [Goose](https://github.com/block/goose) | 2026-08-02 | local Agent、Extension 和 Tool UX 对照 | Extension/plugin 不得成为 daemon authority | 研究 presentation/sidecar 分层 |
| OSS-TOOLHIVE | [ToolHive](https://github.com/stacklok/toolhive) | 2026-08-02 | MCP/tool packaging、isolation 与 catalog 对照 | MCP/dynamic ecosystem 是 post-1.0；不绕过 native Registry | 研究 P5-T03/T04，不阻塞 1.0 |
| OSS-MEM0 | [Mem0](https://github.com/mem0ai/mem0) | 2026-08-02 | Memory extraction/retrieval UX 对照 | 不采纳为 authority store，不据宣传推断 B08 | 研究 provenance/lifecycle 交互 |
| OSS-SKILLS | [Anthropic Agent Skills](https://github.com/anthropics/skills) | 2026-08-02 | local Skill package/revision 结构对照 | Skill 不自授权、不直接执行、不成为 public contract 真相源 | 研究 P4-T04 package/binding |
| OSS-ELIZA | [ElizaOS](https://github.com/elizaOS/eliza) | 2026-08-02 | plugin/Multi-Agent ecosystem 对照 | Multi-Agent post-1.0 且默认关闭 | 只作 P6 informative input |
| SEC-01 | [Secret Service 0.2 Draft](https://specifications.freedesktop.org/secret-service/latest/) | 2026-07-24 | Linux D-Bus secret collections/items/sessions/prompts | desktop/headless 可用性不能假设；规范为 draft | Linux desktop 首发候选；PoC 是 Phase 0 gate |
| DIST-01 | [GitHub artifact attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations) | 2026-07-24 | 可生成 binary/SBOM provenance 并用 `gh` 验证 | provenance 不证明无恶意；离线策略需单独设计 | Linux bundle/SBOM/attestation；P7-T01 |
| DIST-02 | [XDG Base Directory 0.8](https://specifications.freedesktop.org/basedir/latest/) | 2026-07-24 | config/data/state/cache/runtime 分离，runtime 0700 | 首先适用于 Unix-like | P1 数据目录和 socket 布局 |

## 4.2 研究采纳边界

### 采纳

- Pi 官方 Interactive CLI；
- Pi Package/Extension；
- Pi RPC 作为未来 headless protocol；
- OpenAI-compatible Provider wire format；
- DeepSeek `/models` + active probes；
- MCP lifecycle/transport；
- SQLite durable local execution；
- XDG；
- 原生 Secret Store；
- artifact attestations 和 SBOM。

### 不采纳为 Personal v1 Runtime

- Pi SDK 嵌入；
- Pi `auth.json` 作为密钥权威；
- Pi 内建 Bash/write/edit 作为 CognitiveOS 工具路径；
- Temporal；
- LangGraph；
- 默认向量数据库；
- Kubernetes；
- Redis、Kafka、PostgreSQL；
- 多 Agent framework；
- Docker Compose 作为首个交互式安装目标。

---

# 5. 仓库现状与复用策略

| 分类 | 内容 |
|---|---|
| 直接复用 | `TransitionEngine`、`AuthorityStore`、`SqliteAuthorityStore`、replay、authz、TaskContract、Intent/Effect、recovery、Context resolver、budgets、generated contracts、TS channel isolation |
| 适配 | `kernel-server` composition/router；`admin-cli` application logic；TS SDK/Shell transport；installation store；Context candidate source |
| 旁路 | 当前 canned HTTP routes；Pi uncontained evaluator 主路径；Pi 自己的 credential store；直接 CLI→SQLite 组合 |
| 新增 | migration runner、Personal application services、SecretStore、Provider service、scheduler、process supervisor、Tool Registry、Memory repository、Artifact Store、metrics/benchmark harness、installer/update/uninstall |
| 逐步废弃 | synthetic `management_ready`、synthetic watch、canned attach/detach/cancel；`admin-cli` 直接组合 store/runtime；当前无治理的 Pi candidate launch 作为产品入口 |
| 禁止复制 | canonical JSON/digest；状态机；授权算法；Task/Effect completion；Pi TUI/session UI；MCP protocol implementation若官方 SDK满足适配需求 |

---

# 6. 当前成熟度与差距矩阵

| 能力 | 当前 | 目标 | 代码证据 | 主要差距 | 计划 |
|---|---:|---:|---|---|---|
| 安装 | L0 | L5 | 无 personal/deploy/install/release | 无 bundle、verifier、rollback | P1-T08、P7-T01/T02 |
| 初始化 | L0 | L5 | `admin-cli` 无 init | 数据布局、Secret、Provider、daemon、Pi | P1-T01..T09 |
| CLI | L3 管理型 | L5 产品型 | `personal/apps/admin-cli/src/main.rs` | 缺共享 Personal application service | P1-T05/T06 |
| daemon | L2 | L5 | `personal/apps/kernel-server/src/main.rs` | 手写无界 parser、无 auth、canned routes | P1-T04 |
| Pi 集成 | L1/L2 | L5 Interactive | `personal/apps/pi-agent-adapter/src/main.rs` | evaluator 与 admission 未连接 | P0-T06、P1-T07 |
| Provider | L1 | L5 narrow slice | proposal trait、Pi-specific path | 无 catalog/probes/snapshot/proxy | P1-T03 |
| Secret Store | L0 | L5 Linux desktop | 不存在 | native backend、redaction、rotation | P0-T05、P1-T02 |
| Task | L3/L4 core | L5 product | `intent_chain.rs` | 无应用服务、API、queue | P2-T01/T02 |
| Scheduler | L0 | L5 | 不存在 | durable lease/timer/worker | P2-T03 |
| Agent Loop | L3 | L5 | `harness_loop.rs` | 未接 product worker | P2-T04 |
| Process | L0 | L5 narrow | 不存在 | identity、logs、stop/reconcile | P2-T06 |
| Tools | L1 | L5 narrow | `ToolAdapter`、executor ports | 无 Registry/real executor | P2-T05/T06 |
| MCP | L1 naming | L4 adapter | sandbox registration concepts | 无 client/qualification | P5-T03 |
| Context | L3 algorithm | L5 | `context.rs::resolve` | 无 source/retrieval/durable cache | P3-T01/T02 |
| Memory | L1 | L5 narrow | `MemoryAdapter` names | 无 store/index/policy/API | P4-T01..T05 |
| Agent Registry | L1 | L5 | install contracts/evidence | 无 definition/instance lifecycle | P5-T01/T02 |
| Multi-Agent | L0 | L5 later | 不存在 | lifecycle、budget、mailbox、merge | P6 |
| Intent/Effect | L4 | L5 real executor | `effects.rs` | 缺产品 executor | P2-T05/T06 |
| Evidence | L3/L4 core | L5 | event/effect/conformance evidence | 无 artifact product store | P3-T03 |
| Verification | L4 core | L5 product | verification/acceptance semantics | 无独立产品 verifier | P2-T07 |
| Checkpoint | L3 | L5 | SQLite checkpoint rows | scheduler/product resume 未接 | P2-T07 |
| Recovery | L4 protocol | L5 E2E | `recovery.rs` | 无 daemon/process/task restart E2E | P2-T07/T08 |
| Metrics | L1 | L5 | PERF sample builder | 无 live trace/store/campaign | P3-T05、P7-T04 |
| Web UI | L0 code/L1 docs | L4 later | 仅文档，且位于独立仓库 cognitiveos-clients（本仓无 `clients/**`） | gate、PoC、ADR、真实 API | P7-T05 |
| Deployment | L0 | L5 Linux | 无 | service/manifest/SBOM | P1-T08、P7 |
| Upgrade | L0 | L5 | 无 | staged migration/health/rollback | P7-T02 |
| Uninstall | L0 | L5 | 无 | binary/config/data/secret policy | P7-T02 |

---

# 7. Personal 产品需求

以下是**规划追踪 ID，不是 registry REQ-ID**。

`docs/plan/personal-trace.yaml` 是 PERS-PR、正式任务与 Gate/benchmark 的可机读交叉引用；它刻意不并入 registry-derived `docs/traceability/matrix.yaml`，且所有 `evidence_status` 初始为 `not-run`。

| ID | 需求 | 现有规范域 | 验证 |
|---|---|---|---|
| PERS-PR-001 | Linux 用户可验证安装并回滚 | install/audit | B01 |
| PERS-PR-002 | `cognitive init` 完成目录、Secret、Provider、daemon、Pi 初始化 | management/provider | B01 |
| PERS-PR-003 | Provider model 必须通过主动能力探测 | proposal/tool | B01 |
| PERS-PR-004 | daemon 是唯一 authority writer | state/event/CAS | B02/B05 |
| PERS-PR-005 | Pi 是 Shell，不是 Task/Process/Memory authority | shell/task | B02/B05 |
| PERS-PR-006 | 自然语言目标必须先形成 UserIntent，再确定性 admission | intent/task | B04 |
| PERS-PR-007 | 每个 Task 具有 scope、criteria、budget、retry/iteration bounds | task/budget | B04/B07 |
| PERS-PR-008 | 所有外部 mutating 工具走 Intent/Effect | effect | B04/B12 |
| PERS-PR-009 | unknown outcome 不盲重试 | effect/recovery | B12 |
| PERS-PR-010 | completion 由独立 verifier/acceptance authority 决定 | verification/task | B04 |
| PERS-PR-011 | Context 最小充分、授权先于排名、loss 显式 | context | B03/B06 |
| PERS-PR-012 | Memory 有 provenance、scope、freshness、forgetting | context/memory | B08 |
| PERS-PR-013 | Agent/Tool install 不自动授予运行权限 | installation/capability | B09/B10 |
| PERS-PR-014 | CLI 与 Pi 自然语言管理调用同一应用服务 | management/shell | B02 |
| PERS-PR-015 | 重启后 fencing/replay/reconcile/reauthorize/re-resolve | recovery | B05 |
| PERS-PR-016 | 多 Agent 默认关闭，必须相对单 Agent 证明收益 | task/budget | B11 |
| PERS-PR-017 | Token、Tool、Loop、Memory、Cost 可观测 | audit/perf | B03-B11 |
| PERS-PR-018 | API Key 不出现在 config/SQLite/Pi/env/log/evidence | security | B01/security suite |
| PERS-PR-019 | 更新失败必须恢复旧 binary/config/database compatibility | install/recovery | B01/P7 |
| PERS-PR-020 | 企业治理不进入个人版默认关键路径 | architecture boundary | plan review |
| PERS-PR-021 | Windows install parity 只在独立 B01-W 后声明 | product-only | B01-W |
| PERS-PR-022 | Tier 0/1/2 默认路径人工批准不超过一次/task | product-only | B01/B02/B04/B09 |
| PERS-PR-023 | 用户 backup/restore 排除 secret | product-only | GMVP-LINUX |
| PERS-PR-024 | Linux 1.0 包含六类最小真实资源、Pi+sidecar 与 operability | product-only | B01/B02/B03/B04/B05/B08/B09/B12 |
| PERS-PR-025 | exact official Pi acquisition 与 signed lock | product-only | B09 |
| PERS-PR-026 | managed Pi lifecycle 不等于 permission/completion | product-only | B09 |
| PERS-PR-027 | 非 Pi Agent 必须独立 qualification | product-only | B09/RC |
| PERS-PR-028 | 六类资源经 daemon service 与 private versioned projection 提供最小真实 slice | product-only | GMVP-LINUX |
| PERS-PR-029 | per-Agent sidecar 独立固定，1.0 只资格化 Pi+sidecar | product-only | B09 |
| PERS-PR-030 | future Linux/hardware/client 经 ports 演进，不扩大 substrate claim | product-only | GMVP-LINUX |

---

# 8. 目标架构

```text
                    ┌──────────────────────────┐
                    │ Pi Interactive TUI       │
                    │ Presentation Extension   │
                    │ pinned Pi sidecar        │
                    └────────────┬─────────────┘
                                 │ authenticated loopback
       ┌─────────────────────────┴──────────────────────────┐
       │            CognitiveOS Personal Daemon             │
       │                                                    │
       │  Readiness / Config / Provider / Resource + Task API│
       │  Scheduler / Loop / Context / Memory / Skill / Tool │
       │  Process Supervisor / Evidence / Verifier / Audit  │
       └───────┬───────────┬────────────┬─────────────┬─────┘
               │           │            │             │
       ┌───────▼─────┐ ┌───▼──────┐ ┌──▼────────┐ ┌──▼─────────┐
       │ Authority   │ │ Artifact │ │ OS Secret │ │ Operational│
       │ SQLite WAL  │ │ Store    │ │ Service   │ │ logs/cache │
       └───────┬─────┘ └──────────┘ └────┬──────┘ └────────────┘
               │                          │
       ┌───────▼──────────────────────────▼─────────────────┐
       │ Existing deterministic kernel/domain/contracts     │
       │ CAS / transitions / authz / Task / Effect /        │
       │ Context ordering / budgets / recovery / acceptance │
       └───────────────────────┬─────────────────────────────┘
                               │ admitted egress only
                 ┌─────────────▼─────────────┐
                 │ DeepSeek / Tool / MCP     │
                 │ untrusted external systems│
                 └───────────────────────────┘
```

## 8.1 核心控制流

```text
Natural language
→ UserIntentRecord
→ InterpretationCandidate
→ Preview
→ deterministic admission
→ TaskContract
→ scheduler lease
→ ContextView
→ Plan/Tool Candidate
→ OperationDescriptor validation
→ Intent
→ persist-before-dispatch Effect
→ executor/process
→ receipt or OUTCOME_UNKNOWN
→ reconcile
→ evidence
→ independent verification
→ acceptance transition
```

## 8.2 恢复流

```text
daemon restart
→ recovery barrier
→ validate identity/epoch
→ fence stale writers
→ replay committed history
→ reconcile in-flight Effects
→ reauthorize
→ re-resolve Context
→ reacquire scheduler lease
→ resume eligible Loop
```

## 8.3 Pi 边界

Pi 可以：

- 显示模型输出；
- 显示 authority projections；
- 提交自然语言；
- 调用 CognitiveOS Extension commands；
- 展示确认界面；
- 维护自己的 presentation Session。

Pi 不可以：

- 读取/写入 SQLite；
- 持有原始 Provider Key；
- 直接调用 mutating Bash/write/edit/MCP；
- 推进 Task/Effect/Verification 状态；
- 把 Session resume 当 Task resume；
- 把 `agent_end` 当 Task completion。

---

# 9. 设计决策集（DEC-P-*）

本节是研究期的设计决策候选表，编号 `DEC-P-01..20` 为规划 ID，与 `docs/adr/` 的正式 ADR 编号**无对应关系**（正式 Personal ADR 现为 0017–0026；新决策落地时取 `docs/adr/` 下一个可用编号，并在本表补注对应关系）。正式 ADR 都必须包含：状态、背景、仓库约束、候选、评价维度、决策、理由、后果、迁移成本、回滚和重评条件。

已落地对应：DEC-P-05 → ADR-0018/0020；DEC-P-19（部分）→ ADR-0025；DEC-P-20 → ADR-0026；daemon transport → ADR-0019（P0-T07）。

| DEC | 候选与决策 | 主要后果、回滚与重评条件 |
|---|---|---|
| DEC-P-01 Pi 集成 | SDK / RPC / Interactive Extension；**选择 Interactive Extension** | 保留 Pi TUI；RPC 留给后台；若 Extension API 无法稳定固定或无法禁用 mutating tools，停止并重评 RPC/custom UI |
| DEC-P-02 daemon 语言 | Rust / Node / Python；**Rust** | 直接复用 kernel/store；Node 只留 Pi 侧；若 OS/API ecosystem 阻塞，可新增窄 sidecar，不迁 authority |
| DEC-P-03 SQLite | SQLite / Postgres / workflow DB；**SQLite 单 writer + migrations** | 适合本地；若写争用、恢复或数据量实测超预算再重评 |
| DEC-P-04 Artifact Store | SQLite blob / filesystem CAS / object store；**filesystem CAS + SQLite metadata** | 大文件不塞主 DB；可通过 digest 重建引用 |
| DEC-P-05 Secret Store | Pi auth.json / encrypted config / native store；**`SecretStore` port：desktop Secret Service + approved headless encrypted vault**（ADR-0018/0020，部署扩展见 ADR-0038） | desktop 默认 Secret Service；headless locked start + SSH TTY unlock，可选 systemd encrypted credential 只承载 vault-unlock material；无明文 fallback |
| DEC-P-06 Provider | Pi-owned / daemon OpenAI-compatible / vendor SDK；**daemon OpenAI-compatible** | DeepSeek 默认只是配置，不硬编码模型；vendor 差异放 adapter |
| DEC-P-07 Task 状态机 | 新工作流 / Pi Session / 现有 transitions；**复用现有 Task state machine** | 不新增第二套 Task |
| DEC-P-08 Event/Snapshot | snapshot-only / event-only / event+projection；**authority event + rebuildable projections** | snapshot 不成为独立 authority |
| DEC-P-09 Memory | vector-first / files / SQLite+FTS；**SQLite source + FTS** | Embeddings 仅派生索引 |
| DEC-P-10 Embedding | Linux 1.0 / post-1.0 experiment / never；**post-1.0 独立实验** | 不阻塞 B08/GMVP；未证明收益不得进入默认路径 |
| DEC-P-11 MCP | direct MCP / MCP authority / adapter；**Tool Registry adapter** | MCP auth 与 CognitiveOS operation auth 分离 |
| DEC-P-12 Process | Pi-owned / OS service per task / daemon supervisor；**daemon supervisor** | 进程必须有 task/attempt/epoch identity |
| DEC-P-13 Agent Package | raw Pi package / arbitrary git / CognitiveOS manifest；**digest-pinned Agent + sidecar manifest** | Pi/sidecar package 可作为 payload，但 activation 由 daemon 决定 |
| DEC-P-14 Tool Package | MCP discovery / arbitrary executable / qualified manifest；**qualified manifest** | schema、risk、sandbox、reconcile、health evidence 必须齐全 |
| DEC-P-15 Blackboard | shared chat / mutable KV / append-only findings；**append-only non-authority findings** | 黑板消息不能提交状态 |
| DEC-P-16 Git Worktree | shared cwd / mandatory worktree / selective worktree；**仅写任务选择性使用** | read-only research不强制；冲突时取消/重新派发 |
| DEC-P-17 Intent-first | 仅高风险 / 所有 mutating external operations；**后者** | read-only 可短路径，但仍审计和 scope-check |
| DEC-P-18 Web UI | Next/React/Vite/Tauri；**暂定 React+TS+Vite 静态客户端，gate 后重评** | 不进入 P0-P6；只能调用 daemon API |
| DEC-P-19 安装更新 | Homebrew / Compose / attested bundle；**Linux attested bundle + inspectable script**（平台/分发部分已落地 ADR-0025；Windows 安装面归 P7-T07） | Brew/macOS 和 Compose 不首发；失败原子回滚 |
| DEC-P-20 授权交互模型 | 每动作审批 / 企业审批链 / **风险分层 Trust Profile：Tier 0 静默、Tier 1 首用记住（capability lease）、Tier 2 显式确认 + 任务级准入授权 + 预算硬轨（已落地 ADR-0026）** | 治理记录全保留，仅改交互层；默认路径人工确认 ≤1/task；若 Tier 分类无法由 Operation Catalog 元数据确定性判定，回退该操作到 Tier 2 并重评 |

---

# 10. 阶段路线图和门禁

> **依赖解释（2026-07-30）：** Entry / Exit / “禁止提前开发”约束产品集成、任务 `done`、promotion 和声明范围，不是 isolated implementation mutex。满足 `implementation_requires` 的工作可在 `experimental-local-only` 先行；acceptance/promotion requirements 仍须在相应 Gate 前真实满足。后续任务依赖应写为 `implementation_requires`、`acceptance_requires`、`promotion_requires`，不得再用一个“依赖”字段阻断所有开发。

| Phase | Entry | Exit | Blocking tests/evidence | Rollback point | 禁止提前开发 |
|---|---|---|---|---|---|
| 0 基线 | 当前计划批准 | toolchain、ADR、platform、Secret/Pi PoC、benchmark spec 完成 | CI、Linux runner、PoC reports、plan consistency | `main@9b53cf4` | 功能实现、Memory/Multi-Agent/UI |
| 1 首次对话 | P0 全绿 | B01 在干净 Linux VM 真实通过 | install/init/secret/provider/daemon/Pi E2E；secret leak negatives | 未初始化快照/旧 binary | Task autonomy、Memory、MCP、多 Agent |
| 2 Runtime Spine | P1 contracts；B01 不是 implementation mutex | B02/B04/B05/B12 | unified projection/scheduler/Context/sidecar/native Tool/process/recovery/verifier | Phase 1 release | 未资格化 adapter |
| 3 Context Resource Value | P2-T01/P2-T02 stable application contracts，不等待 P2-T08 acceptance | B03 correctness；采集 B06/B07 | context loss/revocation/cache/Artifact/loop negatives | Runtime Spine ports | 未执行的收益 claim |
| 4 Memory + Skill Resource Value | P3-T01/P3-T02 stable ports | B08 lifecycle/correctness/actual consumption | Memory provenance/forget + Skill revision/binding | 无持久资源值的 Runtime Spine | embedding/vector/graph |
| 5 Agent sidecar / post-1.0 Tool ecosystem | P5-T01 与所需 P2 supervision contracts | B09 for Pi+sidecar；B10 post-1.0 independent | package/protocol/instance/process lifecycle；MCP negatives 后置 | native Tool + single Pi | 非 Pi/dynamic marketplace |
| 6 Post-1.0 Multi-Agent | 单 Agent benchmark稳定 | B11 GO 或合法 NO-GO/default-off | isolation/budget/cancel/merge/reviewer | 默认单 Agent | 无证据默认启用 |
| 7 Product Operability | Runtime Spine + Resource Value application contracts | GMVP 后按声明范围汇合 RC | six-resource manifest + full release checklist | 最近健康 release | 未执行能力扩大 claim |

---

# 11. Task Cards

## 11.1 卡片通用规则

以下字段适用于每张卡，且不得由后续 AI 省略：

- 预计删除文件默认 `none`；只能在卡片明确标注时废弃；
- 默认安全约束：Pi/client 非 authority、secret redaction、default deny、Intent/Effect、stable idempotency、fencing、无假完成；
- 默认测试：先写失败测试；Rust 变更执行 fmt/targeted test/clippy；TS 执行 build/test；合同相关执行 consistency/matrix/conformance；
- 默认文档：更新对应 ADR、Task 状态、PROGRESS、Implementation Record 和 handoff；
- 默认完成定义：卡片验收命令全绿且无计划外文件；
- 遇到机器合同不足，停止并走 Lane-CTR Plan Amendment，不得创建平行 DTO 真相源。

每张卡以下均覆盖强制字段：目标/价值、证据/研究、依赖/不包含、文件、数据/API/配置/迁移、步骤、验收/测试/基准/性能、安全/可观测、回滚/文档/解锁、风险/不确定项。

状态纪律（2026-07-30 修订）：任务卡不承载正式状态行；正式 task status、implementation evidence、Gate status、完成日期与证据一律记录在 `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` 台账。`not-started` 表示尚无任务专属工作；首个设计/实现/测试批开始即改为 `in-progress`。P2 及以后压缩卡须在首个 atomic delivery 内补齐本批实际需要的强制字段，不再把“先完整扩写整张卡”作为实现前置锁。

---

## Phase 0

### P0-T01 — 固定可复现基线与支持工具链

- **优先级/目标/价值：** P0；让后续会话可复现 Rust、TS、CI 基线。
- **证据/研究：** 本机 LLVM-MinGW 缺 `libgcc`；远端双 OS CI green。2026-07-25 的正式台账记录确认：`01ceb93` 的 CI run 30140381194 在 Ubuntu 与 Windows/MSVC 均通过；本机 Windows GNU 在默认和已记录的 LLVM-MinGW/shim 处置下均于 linker exit 121 失败，故 GNU host 是非支持开发环境。详见 `personal/tests/baseline/README.md`。
- **依赖/不包含：** 无；不改生产逻辑。
- **文件：** 修改 `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`、本研究报告及 CI/toolchain docs；计划新增 `personal/tests/baseline/README.md`；不删文件。
- **数据/API/配置/迁移：** 无数据/API；明确 Linux runner、Windows GNU/MSVC 支持组合。
- **步骤：** 重跑 Git 状态、两套工具链、全部 baseline commands；记录 SHA、exit、耗时。
- **验收：** Linux clean runner 全绿；Windows 选择官方支持 linker 后全绿或明确 non-supported。
- **基准/性能：** 记录 build/test p50，不设改进目标。
- **安全/观测/回滚：** 不读取 secret；只生成 ignored artifacts；失败即恢复环境，不改源码。
- **解锁/风险/不确定：** 解锁全部任务；Windows GNU 是否继续支持需在此关闭。

#### P0-T01/D02 — 本机 Rust 工具链修复 Slice（2026-09-02 owner 指令登记；未执行）

- **目标：** 让 `DEV-WIN-GNU-01` 具备 workspace Rust 本地迭代能力：`cargo build --workspace --locked`、`cargo test --workspace --locked -- --test-threads=1`、`cargo clippy --workspace --all-targets --locked -- -D warnings` 在本机通过。
- **事实基线（2026-09-02 探测，未跑 cargo）：** 默认 host `x86_64-pc-windows-gnu`；rustup 已装 `1.97.1-x86_64-pc-windows-msvc` 与 `gnullvm`；`D:\VSBuildTools` 存在且 vswhere 报告 `VC.Tools.x86.x64`，但 `link.exe` 不在 Cursor Shell PATH；pwsh 7.6.5 已装；`core.autocrlf=true`（被 `.gitattributes eol=lf` 覆盖）。这些事实尚未写入环境登记，由本 Slice 写回。
- **Owner 决策点（执行前必须确认）：** (a) 本机 override——`rustup override set 1.97.1-x86_64-pc-windows-msvc`（目录级）或本机 `.cargo/config.toml`（不提交），不改 tracked 文件，CI/其他机器不受影响；**推荐默认**。(b) 改 tracked `rust-toolchain.toml`——影响所有 clone 与 CI，corrective 决策，需 owner 明示。子决策：`pnpm run verify:local` + `scripts/v01-auto-run.*` 重钉到 CI 计数（89/62/27）还是废弃并从 `package.json` 移除。未确认前可做：事实探测、环境登记草稿、running report 骨架、connected-docs 改写草稿。
- **步骤：** 领取 lease（Lane-DOC + 工具面路径：`docs/plan/PERSONAL-TEST-ENVIRONMENTS.md`、`AGENTS.md`、`.cursor/rules/10-*.mdc`、`docs/governance/DEVELOPMENT-OPERATING-MODEL.md` §3.0、`personal/tests/baseline/README.md`、`tools/src/check-consistency.mjs` 6c 守卫片段、handbook 三页双语、`package.json`/`scripts/` 视子决策）→ P0-T01 改 `in-progress` 并同步进度汇总 → 决策点确认 → 执行切换 → 在本机跑三条 cargo 命令并记 running report（`rustc -vV`、`link.exe` 路径/版本、exact revision）→ 写回环境登记 §3（allowlist 扩展；**能力上限不变**）→ 改写 `RUST-LINK-DEV-WIN-GNU-01` 相关禁令为"历史 GNU 结论 + 当前 MSVC 允许面"（6c 守卫片段同步）→ handbook 双语 + 指纹 → required CI 仍绿 → 收口。
- **validation environment：** `DEV-WIN-GNU-01`（执行对象）+ `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01`。
- **关闭门：** 三条 cargo 命令在本机 exact revision 上通过且记账；环境登记与全部引用文件同步；`check:consistency`（含 6c）、`check:handbook`、generator `--check`、`check:rules`、docs-sync 绿；required CI 绿。
- **漂移检测负例：** 本机 Rust 证据被写成 Gate/Profile/Windows 产品支持或 `DEV-WINDOWS-NATIVE-OPC-01`；在 feature Slice 内顺手改工具链；只改 PATH/override 不写回登记；未确认决策点即改 `rust-toolchain.toml`；把过期 `verify:local` 当本地门。
- **不阻塞：** 任何 P13 卡（旁支加速项）。

### P0-T02 — 冻结 Personal 需求、追踪与架构边界

- **目标：** 将 PERS-PR、现有 REQ、任务、测试、Benchmark 建立双向映射。
- **证据：** 273 REQ/85 vectors；禁止虚构 REQ。
- **依赖：** P0-T01；不新增对象族/Profile/REQ 域。
- **文件：** 修改 `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` 与本研究报告；计划新增 machine-readable trace YAML。
- **数据/API：** 无运行时变化。
- **步骤：** 对每个 PERS-PR 映射 registry REQ、现有 schema/vector或标记 product-only/internal。
- **验收：** 无孤儿 PERS-PR、Task、Benchmark；`check:consistency` 不受影响。
- **回滚/风险：** 映射不足即 Plan Amendment；不得用计划 ID冒充 REQ-ID。
- **解锁：** P0-T03..T07。

### P0-T03 — License、首发平台与分发决策

- **状态：** 以正式台账为准。
- **目标：** 关闭根仓 License、首发平台 support matrix、Node/Pi redistribution 义务。
- **Owner 决议：** Apache-2.0；首发产品平台 Linux x86_64 + Windows x86_64；GitHub Release 可检查 bundle；不 vendor Pi/Node；crates.io/npm 仍不发布。
- **依赖：** P0-T02；不开发 installer。
- **文件：** 根 `LICENSE`/`NOTICE`；`docs/adr/0025-personal-license-platform-distribution.md`；`docs/legal/THIRD-PARTY-NOTICES.md`；`docs/plan/PERSONAL-SUPPORT-MATRIX.md`；workspace license 元数据。
- **API/配置：** 定义 release manifest 字段，不实现（P7-T01）。
- **验收：** owner GO/NO-GO 与 notices 完整；非 G0/B01/Profile。
- **回滚：** 变更许可证须新的 owner 决议与 ADR 修订。
- **解锁：** P0-T06、P1-T08、P7-T01。

### P0-T04 — 数据布局、迁移、备份与回滚设计验证

- **目标：** 为现有 inline SQLite Schema 建立迁移策略。
- **证据：** `personal/crates/cognitive-store/src/sqlite.rs`、`installation.rs` 无 schema version。
- **依赖：** P0-T02；不改变当前 DB。
- **文件：** 新增 ADR-0017（ADR-0003 已是 HTTP/SSE 传输决策，不得复用）；新增 `crates/cognitive-store` 本地迁移适配器和 focused tests；不创建运行时 XDG 目录或用户数据。
- **数据/API：** 设计 `schema_migrations(version, digest, applied_at)`、preflight、backup、rollback policy。调用方显式提供 scratch/backup 路径；不暴露客户端 authority 写入口。
- **验收：** 用复制数据库完成 dry-run、重复执行、checksum mismatch、失败恢复设计评审；实际 Rust 测试须在支持工具链执行后才能将本任务标记 done。
- **风险：** authority 与 installation DB 顺序；必须定义跨 DB failure semantics。
- **解锁：** P1-T01。

### P0-T05 — Linux Secret Service PoC

- **目标：** 证明 user daemon 能安全 set/get/replace/delete secret。
- **证据：** SecretStore L0；Secret Service 0.2 Draft；ADR-0018。
- **依赖：** P0-T01；不持久化真实生产 Key。
- **文件：** 新增 `crates/cognitive-secret`（isolated PoC）；`Cargo.toml` workspace member；`docs/adr/0018-personal-secret-store-boundary.md`。不修改 `cognitive-runtime` 依赖除非后续评审。
- **API：** 冻结 `SecretStore::{probe,put,get,delete}` 和 opaque `SecretRef`；attribute-keyed `put` 即 rotate。
- **步骤：** 模拟 backend 覆盖 put/get/rotate/delete、service absent、locked、prompt unavailable、Debug/Display/env redaction；Linux native probe-only（mutating D-Bus 归 P1-T02）。
- **验收：** 测试 secret 不出现在 env/args/config/SQLite/log/artifacts；不可用时 fail-closed；无明文 fallback。
- **回滚：** 删除 PoC secret/item；不提供明文 fallback。
- **后续演进：** 当时的 headless Linux 不确定性已由 ADR-0038 选择 approved encrypted
  vault + locked start/TTY unlock 路径；该 target 仍需 P7-T03/P7-T08 实现和资格证据，
  不改本 PoC 的既有完成事实。
- **解锁：** P1-T02。

### P0-T06 — Pi 版本、Extension 与 RPC 兼容性 PoC

- **目标：** 固定 Pi 版本、integrity、source commit、Extension API 和 RPC JSONL fixture。
- **状态：** 以正式台账为准（台账记录：in-progress；已交付 version pin/SRI + strict-LF RPC parser、pinned Extension fixture 两个原子部分；剩余：pinned package 的实际 session/RPC load evidence）。ADR-0018 本机开发例外（显式开关 + 独立 Provider config 目录，默认拒绝，不用于 Windows/CI/发布，P2 结束到期）已获 owner 批准，到期核查归属 P2-T08 验收。
- **证据：** registry 0.82.0；仓库 smoke 使用 0.81.1；API 快速变化。
- **依赖：** P0-T03；不启动 governed background Agent。
- **文件：** 修改 `apps/pi-agent-adapter` tests/docs；计划新增 `core/tests/golden/pi-rpc/` 和 Extension PoC。
- **API：** 固定 Extension command/provider/event subset；RPC 只做 contract fixture。
- **验收：** project trust、tool replacement、session event、strict LF framing、version mismatch 均有测试。
- **安全：** Extension 无 DB/secret；built-in write/edit/bash 在 governed mode 不可用。通常 Provider material 不进入 Pi 或环境；唯一的 ADR-0018 local-development exception 仅传给初始 Pi child，不构成 sandbox/containment 声明，P2 后必须移除、替换为 proxy 或重新批准。
- **回滚：** pin 回最后兼容版本。
- **解锁：** P1-T07；其固定 RPC fixture 可作为 P6 设计输入，但不构成 P6 的直接任务依赖。

### P0-T07 — daemon transport、认证和威胁模型

- **目标：** 选择 bounded loopback HTTP/Unix socket、task/management channel bootstrap。
- **证据：** 当前 server 手写无界 HTTP，routes synthetic。
- **依赖：** P0-T02；不实现业务路由。
- **文件：** 新增 `docs/adr/0019-personal-daemon-transport-auth-threat-model.md`（ADR-0003 已是 HTTP/SSE 基线，不得复用编号；ADR-0002/0017 是 store/migration）；P1-T04 再改 `apps/kernel-server` / Personal daemon entry 与 management ports。
- **API：** 请求大小、header、timeout、concurrency、session issuance/expiry、channel binding。
- **验收：** ADR-0019 冻结 UDS 默认 + loopback TCP 可选、channel-scoped bearer bootstrap、资源上限；threat model 覆盖 CSRF、DNS rebinding、token theft、channel confusion、replay；不实现业务路由、不声明 G0/B01-B12/Profile。
- **回滚：** 保持 loopback disabled-by-default。
- **解锁：** P1-T04。

#### DOC-P13-DRIFT-FIX — 文档漂移对齐（owner-directed，非正式任务；P0-T09 前置）

- **状态：** 以正式台账为准（PERSONAL-DEVELOPMENT-PLAN Phase 13「配套维护交付」行；2026-09-03 `done`，PR [#309](https://github.com/agentkernel/cognitive-os/pull/309)）。
- **内容：** (a) dev-prep index「Phase 13 build order」边集合逐条对齐正式计划 mermaid（补 `T05→T12b`、`T07→T12b`、`T05→T13`、`T09→T15`、`T10→T15`、`T11→T15`；27 边全等）；(b) Pi 包名权威 = 代码 `OFFICIAL_PI_PACKAGE = "@earendil-works/pi-coding-agent"`（与环境登记 §1、本表 PI-02 一致），handbook `reference/compatibility` + `developer/agent-and-pi-lifecycle` 双语四页统一到该常量；(c) `developer/development-environments` 双语补 autocrlf/`.gitattributes` 说明。
- **不包含：** 不改正式计划建造顺序；不改代码常量；不实现机械校验（归 `P0-T09`）。
- **证据：** [running report](../checkpoints/2026-09-03-personal-doc-p13-drift-fix-report.md)；[closure](../checkpoints/2026-09-03-personal-doc-p13-drift-fix-closure.md)。

### P0-T09 — 计划/规则漂移的机械校验（2026-09-02 登记）

- **状态：** 以正式台账为准（2026-09-03 `done`；merged PR [#312](https://github.com/agentkernel/cognitive-os/pull/312) at `main@8badb83c`，lease 已关闭；[running report](../checkpoints/2026-09-03-personal-p0-t09-drift-checks-report.md)；[closure](../checkpoints/2026-09-03-personal-p0-t09-drift-checks-closure.md)）。来源：`DOC-AGENT-RULES`（PR #306）复审「发现但未修改」第 1/4/6 项；`DOC-P13-DRIFT-FIX` 是其前置（先对齐再上校验；已 done，PR #309）。
- **实现要点（D01）：** `tools/src/lib.mjs` `loadTrackedPaths()`（`git ls-files -z`）；`check-consistency` 不在 Git checkout 内 fail closed（`TRACKED_PATHS_UNAVAILABLE`），本机与 CI 扫描集一致；`check-agent-rules` 对本地专用资产保留"缺失告警/存在严格"，非 Git 根仅 fixture 回退文件系统并明示；边集合比对解析两处 mermaid（节点 id 去 `P13`/`P11` 前缀，区分实线/虚线）；source-map 规则 `pi-official-package-pin` + 新 handbook 规则 HB016（带 `symbols` 的规则必须被每个被路由页双语钉住）。
- **目标：** 把三类已发生的漂移变成机械红灯：(1) 已提交文档链接到本机未跟踪文件（`clients/docs/design/opc-2.0/` 14–18、21–26、`window-c-*.md`、`docs/plan/p11-plan-review-and-optimization.md`）——`tools/src/check-consistency.mjs` 与 `tools/src/check-agent-rules.mjs` 的相对链接/路径存在性检查改为基于 `git ls-files`（tracked-only），本机与 CI 结论一致；(2) 正式计划 Phase 13 mermaid 建造顺序边集合 == `personal/docs/architecture/personal-2.0.0-dev-prep-index.md`「Phase 13 build order」边集合；(3) `personal/handbook/_meta/source-map.json` 新增 `personal/crates/cognitive-runtime/src/installer.rs`（`OFFICIAL_PI_PACKAGE`）→ `ref.compatibility`、`dev.agent-pi-lifecycle` 路由，两页 `sources` 加 `symbols` 钉住包名。
- **依赖 / 不包含：** P0-T01、P8-T08；`DOC-P13-DRIFT-FIX` 完成。不改正式计划的建造顺序本身；不改产品代码。
- **文件：** `tools/src/check-consistency.mjs`、`tools/src/check-agent-rules.mjs`、`tools/test/check.test.mjs`、`tools/test/check-agent-rules.test.mjs`、`tools/test/handbook-check.test.mjs`、`personal/handbook/_meta/source-map.json`、受影响 handbook 页指纹。
- **Lane / lease：** Lane-CFR 主责（工具面）+ handbook `_meta`；正式 `P0-T09` task lease（DOC/GOV 类不能拥有 `tools/**`）。
- **验收：** 三项校验各有 focused negative fixture 且在当前树绿；按 docs-sync-contract §5 重跑注入演练并把输出贴入 PR；required CI 绿。
- **validation environment：** `DEV-WIN-GNU-01`（Node 工具面）+ `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01`。
- **关闭门：** 本机对未跟踪文件的链接即红；两处建造顺序边集合不等即红；`installer.rs` 变动触发 docs-sync 路由；负例 fixture 全绿。
- **漂移检测负例：** 校验只在 CI 生效而本机放行；用文件系统存在性冒充 tracked；边集合写死在文档而非解析 mermaid；为让校验通过反向改正式计划。
- **不阻塞：** 任何 P13 卡。

---

## Phase 1 — 安装到首次对话

### P1-T01 — 版本化数据库迁移与 XDG 布局

- **目标：** 初始化 config/data/state/cache/runtime，安全迁移两套 SQLite。
- **证据：** inline schema、无 migration framework。
- **依赖：** P0-T04。
- **文件：** 修改 `personal/crates/cognitive-store/src/{sqlite,installation,lib}.rs`；新增 `layout.rs`、`personal_db.rs`、`tests/p1_t01_layout_migrations.rs`；ADR-0017 补记 XDG 实现。
- **数据变化：** 新增 migration metadata；不改变 authority transition semantics。
- **配置：** XDG paths，目录 0700，DB 0600。
- **迁移：** 先备份、checksum、transaction、integrity check；失败保留旧 DB。
- **验收：** empty→latest、previous fixture→latest、reapply、corrupt/checksum mismatch、disk failure tests（`p1_t01_layout_migrations`；CI 执行）。
- **性能：** 小型 DB migrate 建议目标 <5 s；未测前不宣称。
- **回滚：** 恢复备份并启动旧 binary。
- **解锁：** P1-T02/T04/T08。

### P1-T02 — SecretStore 正式后端和 Provider 配置

- **目标：** daemon 以 opaque ref 管理 DeepSeek Key。
- **依赖：** P0-T05、P1-T01。
- **状态：** 以正式台账为准。
- **文件：** 扩展 `crates/cognitive-secret`（ProviderConfig、ProviderKeyService、LinuxSecretToolStore、backend_select、secret_input、tests/p1_t02_provider_secret.rs）；ADR-0020；未改 management/runtime（避免 Lane-RUN 所有权冲突）。
- **数据：** Provider config 只存 provider、base URL、secret_ref、selected snapshot digest。
- **API：** put/rotate/delete/probe via ProviderKeyService；hidden-input helper `read_secret_material_from_reader`（CLI echo-off 归 P1-T06）。
- **验收：** rotation、locked store、deleted secret、daemon restart、redaction negatives（CI p1_t02_provider_secret 通过）。
- **不包含：** cloud secret manager、明文 fallback、Pi auth.json、真实 Provider Key、G0/Profile。
- **回滚：** 删除 ref；不自动删除用户未确认的数据。
- **解锁：** P1-T03/P1-T06。
### P1-T03 — OpenAI-compatible Provider、模型发现与能力快照

- **目标：** DeepSeek 默认初始化，但模型 ID 动态发现和主动验证。
- **状态：** 以正式台账为准。
- **文件：** 扩展 `crates/cognitive-secret`（`provider_transport` / `provider_snapshot` / `provider_probe`、`tests/p1_t03_provider_discovery.rs`）；ADR-0021。**未**修改 `cognitive-runtime`（避免 Lane-RUN 所有权冲突；HTTPS client 由 daemon 注入 transport）。
- **API：** `ProviderDiscoveryService::{list_models, discover_probe_and_persist}`；`ModelSelection`；`ProviderCapabilitySnapshot` / readiness snapshot digest。
- **数据：** observed models、selected model、probe version、capability flags、product-local `fnv1a64` digest 写入 `provider.json` 的 `selected_snapshot_digest`。
- **步骤：** HTTPS policy→GET `/models`→选择→chat/stream/tool/cancel 主动 probe→persist digest。
- **验收：** 401/403/404/429/5xx、alias drift、HTTP 200 但 tool 失败、timeout、manual model fallback、Authorization/body redaction（mock transport）。
- **安全：** response/body redaction；Key 仅在最终 egress Authorization header；tool_call 成功仅为 candidate 形状，非 Effect。
- **性能：** init probe 建议总预算 60 s；不测 live latency 本批。
- **不包含：** 真实 DeepSeek 网络、G0/B01-B12/Profile、registry DTO、runtime 接线。
- **解锁：** P1-T05/T07/T09。

### P1-T04 — 有界 Personal daemon 与本地认证

- **目标：** 替换 synthetic composition 的 Personal 入口，建立 loopback 有界 front door 与本地认证。
- **状态：** 以正式台账为准（台账记录：done，PR #95 + PR #96，timeout/concurrency 行为测试已由 CI 执行）。
- **文件：** `personal/apps/kernel-server/src/personal/{mod,auth,bounds,lifecycle,server}.rs`、`main.rs --personal`、`tests/p1_t04_personal_daemon.rs`；layout daemon 路径；ADR-0022。
- **配置：** loopback-only bind、ADR-0019 body/header/concurrency bounds、single-instance `daemon.lock`、runtime bootstrap secret。
- **API：** `POST /local/session`；channel-scoped bearer on `/management/*` and `/task/*`；`GET /personal/health`（non-claim）。
- **验收：** oversized body、bad auth、wrong channel、cookie/host reject、second-instance lock、restart。
- **不包含：** Task scheduler、Memory、MCP、full readiness projection（P1-T05）、UDS product default path（design remains ADR-0019）。
- **回滚：** 未认证/超限请求 fail-closed；无 authority mutation from this front door.
- **解锁：** P1-T05/T06/T07。

### P1-T05 — Readiness、status 和 doctor 应用服务

- **目标：** CLI、Pi、未来 UI 共用同一事实源。
- **状态：** 以正式台账为准。
- **文件：** `personal/apps/kernel-server/src/personal/readiness.rs`、`server.rs` 路由、`tests/p1_t05_personal_readiness.rs`、ADR-0023。**未**修改 `cognitive-management`（Lane-RUN 所有权；Personal 组合根承载 projection）。
- **API：** management-channel `GET /personal/status`、`GET /personal/readiness`、`GET /personal/doctor`；组件 system/database/secret/provider/daemon/pi；返回事实、duration、source、error_class 与 non-claim。
- **验收：** degraded/blocked/ready 分离；静态检查通过不写成 runtime ready（`static_check_is_not_runtime_ready`）；secret_ref/bootstrap 不入投影。
- **观测：** 每项 check duration/error class；doctor guidance 可操作。
- **不包含：** CLI 产品入口（P1-T06）、Pi package（P1-T07）、G0/B01-B12/Profile、registry/schema/vector。
- **解锁：** P1-T06/T07/T09。

### P1-T06 — `cognitive init/doctor/status/daemon`

- **状态：** 以正式台账为准。
- **目标：** 将 `admin-cli` 演进为 `cognitive` 产品入口。
- **文件：** 修改 `personal/apps/admin-cli/Cargo.toml`、`src/main.rs`、tests；计划添加 `cognitive` bin，保留 `admin-cli` 兼容。
- **API：** 只调用 daemon/application service，不直接编排 SQLite。
- **步骤：** env check→dirs→secret→provider→model→daemon→Pi check→self-test。
- **验收：** hidden input、URL修正、重试、手填 model、可操作错误、重复 init 幂等。
- **不包含：** Task/Memory/Tool management。
- **回滚：** 初始化失败清理临时状态，不删除既有数据/secret。
- **解锁：** P1-T09。

### P1-T07 — CognitiveOS Pi Package/Extension 与 Provider proxy

- **优先级/目标/价值：** P1 收尾前的关键件；复用 Pi TUI 完成 `cognitive` 首次受治理会话，同时把"Pi 是 Shell 不是 authority"（PERS-PR-005）从纸面约束变成进程内可执行的拒绝。
- **状态：** 以正式台账为准（[PERSONAL-DEVELOPMENT-PLAN.md](PERSONAL-DEVELOPMENT-PLAN.md)）。
- **证据/研究：** PI-01/PI-05/PI-06/PI-07（§7 来源表）；P0-T06 已固定 `@earendil-works/pi-coding-agent@0.81.1` 的版本/SRI/source commit/Node engine 于 `personal/apps/pi-agent-adapter/src/lib.rs`，并交付 Extension fixture 与 strict-LF RPC parser；`personal/apps/kernel-server/src/personal/{server,auth,bounds}.rs` 已有有界 loopback front door、bootstrap secret 与 channel bearer（ADR-0022）；`crates/cognitive-secret` 已有 `ProviderKeyService`/`ProviderTransport`/`ProviderDiscoveryService`（ADR-0020/0021），但**仓库内无生产 `ProviderTransport` 实现，也无 HTTP/TLS 依赖**，且 Personal front door 单请求单连接、无 SSE。
- **依赖：** P0-T06（Pi 表面固定）、P1-T03、P1-T04、P1-T05。P0-T06 的剩余缺口是 Linux-native 上的运行时加载证据，不阻断本任务的接口实现，按台账"开发状态解耦"注记走 `experimental-local-only`。
- **不包含：** 真实 Task API 与 watch（P2-T02）、scheduler/worker（P2-T03/T04）、受治理工具执行与 Tool Registry（P2-T05/T06）、Memory、MCP、多 Agent、Pi 供应链 provenance verifier（Pi P2）、OS sandbox（Pi P4）。不得 vendor 或分发 Pi/Node（ADR-0025）。
- **文件：** 新增 `personal/packages/pi-cognitiveos/`（`pi-api.ts` 固定 API 结构镜像、`pin.ts` 兼容 pin、`tool-policy.ts`、`daemon-discovery.ts`、`daemon-client.ts`、`status.ts`、`extension.ts`、`index.ts` 与对应测试）；修改 `pnpm-lock.yaml`（**不得**出现 `@earendil-works/*`）；后续批次修改 `personal/apps/kernel-server/src/personal/{server,readiness}.rs` 并新增 `personal/apps/kernel-server/tests/p1_t07_provider_proxy.rs`；无删除文件。
- **数据/API/配置/迁移：** 不新增 SQLite 表、不新增迁移。Extension 只读两个既有本地文件（`$XDG_STATE_HOME/cognitiveos/daemon-endpoint.json`、`$XDG_RUNTIME_DIR/cognitiveos/local-bootstrap.secret`），只调用既有 `POST /local/session` 与 `GET /personal/status`。daemon 侧新增 provider proxy 路由必须复用同一 bearer/bounds/错误信封；Pi 配置固定 version/integrity/package path，且 project trust 恒拒绝。
- **步骤：** (1) Extension 包与默认拒绝的 tool 策略；(2) daemon 只读事实消费与显式失败路径；(3) daemon 侧 provider proxy 路由与生产 `ProviderTransport`（HTTP/TLS 依赖或子进程方案须单独决策并记录）；(4) `readiness.rs` 的 `pi` 组件从硬编码 `not_configured` 翻转为真实检查，**不改动既有聚合规则**（ADR-0023）；(5) 真实 Pi 进程加载证据（依赖 P0-T06 `extension-load`，须 Linux-native 主机）。
- **验收：** direct `bash`/`write`/`edit` 禁用（且未分级工具不得放行）；无 API Key、无 env key、无 SQLite path；daemon unavailable 时明确失败且不得渲染为 ready；project trust 恒拒绝；Pi 不进入 lockfile；TS pin 与 Rust `PiCompatibilityPin` 无漂移。
- **测试：** 包内 `node --test`（工具策略正负例、发现路径 fail-closed、真实 loopback 假 daemon 覆盖 401 重发一次与持续拒绝、投影畸形→协议错误、源码扫描断言无 key/db/子进程/文件写入）；proxy 落地后另加 `kernel-server` 集成测试与跨渠道负例；命令按 §15.2。
- **基准/性能：** Extension startup 建议 <2 s；首个 proxy token 单独计量。按 §15 四段归因（CognitiveOS 确定性处理 / Pi-Node 进程与 RPC / Provider 网络与模型 / 文件系统与 SQLite），本阶段只记录，不设阈值、不构成 REQ-PERF-004 campaign。
- **安全/可观测：** Extension 不持有任何 Provider 凭据；bootstrap secret 仅用于一次会话请求，不落盘、不展示、不入错误消息；session token 不得出现在任何 UI 表面；所有失败使用稳定错误码；不得合成 Gate/Profile/release 声明。
- **回滚/文档：** disable/remove Extension 不影响 authority data（Extension 无写路径）；provider proxy 路由可整体关闭且不改变既有 front door 语义；同批更新台账、PROGRESS、handoff。
- **解锁：** P1-T09、P2-T02。
- **风险/不确定：** R-07（Pi Bash 绕过）由进程内默认拒绝 + 启动 flag 双重覆盖；生产 `ProviderTransport` 需引入 HTTP/TLS 依赖或子进程方案，属供应链决策，须在实施批次中显式记录；Personal front door 无 streaming，流式补全在当前表面不可表达，须在 proxy 批次中明确取舍。

### P1-T08 — 可检查 Linux bundle installer 与 user service

- **目标：** 支持 `curl -o install.sh; less; sh`，不是 `curl|sh`。
- **文件：** 新增 `personal/deploy/linux/install.sh`、manifest、systemd user unit、uninstall skeleton；修改 CI release dry-run。
- **数据：** staged install state；不迁移用户数据直到 verifier通过。
- **步骤：** platform check→download manifest/artifacts→digest/attestation→stage→candidate-service liveness→atomic switch→pointer/service confirmation；candidate liveness 只证明 daemon 存活，不证明 product readiness。
- **bootstrap slice clarification (ADR-0029):** checked-in `install.sh` is an
  unrendered fail-closed release template; an inspected rendered script binds
  its URL/version/redirect host/bootstrap digest/public keyring/Pi pin, cleans
  only its private download directory, authenticates a verify-only adapter,
  and delegates to ADR-0028 before any future health-gated activation. This
  slice does not invent a health callback or call systemd.
- **验收：** tamper、interrupted download/install、no Node、wrong Pi integrity、existing version。
- **不包含：** Homebrew、Docker、root service、自动更新。
- **回滚：** 切回前一版本；保留数据备份。
- **后续前置项：** 当前 artifact 尚无经过审计的安全解包和可运行
  `kernel-server` archive layout；该独立前置项完成前，user-unit/controller
  必须 fail-closed，不能把 fake/WSL/CI 结果写成真实 systemd 或 release 证据。
- **解锁：** P1-T09；为 P7-T01/P7-T02 提供安装器输入。

### P1-T09 — 首次安装到首次对话 route 与 B01 campaign

- **目标：** 分阶段完成 route implementation、deterministic binary fixture、development smoke、usability learning 和 formal B01；只有最后一项决定 B01 Gate。
- **依赖：** `implementation_requires`: P1-T08 及现有 Secret/Provider/daemon/Pi contracts；`acceptance_requires`: 当前 route 的真实 pinned Pi Extension load、真实首个响应、native Secret Service smoke 与可复现 runner；`promotion_requires`: formal B01 pass。
- **文件：** route/fixture 使用现有内部 seam；formal campaign 新增 `personal/tests/e2e/personal/b01-*` 和 evidence schema/runner。公共 DTO/schema/error/transition/vector 缺口走 Lane-CTR。
- **验收：** successor `002` 使用 ADR-0039 的固定 N=6 独立 clean Linux VM outcomes；六个 counted outcomes 均入账，至少 5 次成功，报告全部结果、median/p95 TTFC 与 binomial 95% CI；关键安全失败为 0，并有 affirmative independent-verifier closure。失败不重试/抹除；retained `001` 与 owner-waived transition Attempt 7 保持审计边界。除 API Key 与模型选择外无必选交互（ADR-0026）。TTFC p95 ≤10 min 在 owner 明确升级前保持 advisory。
- **失败条件：** Key 泄漏、工具未禁用、daemon synthetic ready、模型仅凭 `/models`、漏报 attempt 或 test-root cleanup 失败。B01 只要求 disposable VM/owned test root cleanup；产品 uninstall 归 P7-T02，不构成 B01 循环依赖。
- **证据：** logs、timestamps、versions、snapshot digests 和全部 attempt outcome，绝不含 Key。fixture/WSL/dev smoke 只推进 implementation evidence，不推进 B01。
- **解锁：** Phase 2。

---

## Phase 2 — 单 Agent 任务闭环

### P2-T01 — TaskApplicationService

- **优先级/目标/价值：** P2 首任务；把 L3/L4 意图链内核暴露为 L5 产品应用服务：proposal/clarify/preview/admit/control/query 六个操作面构成任务生命周期的唯一产品入口（§6 Task 行差距）。
- **状态：** 以正式台账为准。
- **证据/研究：** `core/crates/cognitive-kernel/src/intent_chain.rs` 已提供 `record_user_intent`、`record_interpretation_candidate`、`admit_interpretation`、`mint_task_contract`、`verify_task_binding_current`、`supersede_task_contract` 与 `GovernanceSeed`/`AcceptanceCommand`/`TaskContractCommand`；`cognitive-store` 已有 `user_intent_records`/`intent_interpretations`/`task_contracts`/`budgets`/`fencing` 表。缺口仅在应用服务与产品 ports（§6：无应用服务、API、queue）。
- **依赖：** `implementation_requires`: 既有 authority/store/Intent/TaskContract contracts；`acceptance_requires`: P1-T09 route implementation 可集成；`promotion_requires`: B01 pass。`experimental-local-only` 可先行实现，不得把 B01 当作代码开工锁。
- **不包含：** scheduler（P2-T03）、HTTP/API 路由（P2-T02）、Memory、多 Agent；不得新增平行 Task 类型或第二状态机（DEC-P-07）。
- **文件：** 修改 `crates/cognitive-management`（task application service 模块与 ports）与 `crates/cognitive-runtime`（组合根接线）；复用 `intent_chain.rs`、TaskContract、budgets，不复制内核逻辑；新增 `personal/crates/cognitive-runtime/tests/p2_t01_task_application_service.rs`（先写失败测试）；无删除文件。
- **数据/API/配置/迁移：** 不新增 SQLite 表；若实测需要投影辅助结构，必须经 P1-T01 迁移框架并单独评审。服务操作：proposal（raw intent 持久化）、clarify（`AmbiguityFact`/`InterpretationCandidate`）、preview（TaskContract 摘要 + preview digest）、admit（`AcceptanceCommand` 绑定 digest）、control（supersede/cancel 请求）、query（只读投影）。
- **步骤：** (1) service trait/DTO 定义，机器合同不足即停走 Lane-CTR，不建平行 DTO 真相源；(2) raw intent 先持久化再解释；(3) preview digest 生成与 admit 绑定校验；(4) 修订经 `supersede_task_contract` 产生新 epoch 并 fence 旧任务；(5) 组合根接线与负例。
- **验收：** raw intent 先持久化（崩溃后可重放）；preview digest 与 admit 绑定（digest 不匹配拒绝）；修订产生新 epoch 并 fence 旧任务（`verify_task_binding_current` 拒绝旧绑定）；admission preview 是唯一默认人工授权点：批准即覆盖任务范围内 Tier 0 与既有授权下的 Tier 1 动作（ADR-0026）；预算在准入时冻结为硬轨。
- **测试：** 纯逻辑单元 + SQLite 集成（restart/replay）；负例：digest mismatch、stale epoch、重复 admit、缺 governance seed；命令按 §15.2。
- **基准/性能：** 记录 preview/admit 延迟作为 B02/B04 埋点；无目标值。
- **安全/可观测：** §11.1 默认安全约束；admission 决策与 digest 全审计（REQ-AUDIT-001/002）；intent 原文 redaction 负例。
- **回滚/文档：** 服务层可整体停用且不影响 kernel 语义；更新台账、PROGRESS、Implementation Record、handoff。
- **解锁：** P2-T02、P2-T03。
- **风险/不确定：** preview digest 的 canonical 序列化边界；Tier 1 既有授权（capability lease）在 P5-T01 前仅有本地最小实现，catalog 元数据不足以判 Tier 时回退 Tier 2（ADR-0026）。

### P2-T02 — 真实 Resource + Task API/watch、统一 projection 与 CLI/Shell parity

- **目标/价值：** 把 P2-T01 服务经 Personal daemon 暴露为真实 Resource + Task API，替换 canned proposal/attach/detach/cancel/watch routes；用 private versioned Personal projection 统一呈现 Memory/Skill/Tool/Context/Task/Runtime；Pi-hosted Shell 经 pinned sidecar 与 deterministic CLI 调用相同 daemon application services（B02 基础）。
- **状态：** 以正式台账为准。
- **证据/研究：** `personal/apps/kernel-server/src/personal/{server,auth,bounds}.rs` 已有有界 front door 与 channel bearer（ADR-0022）；`packages/sdk-ts`/`apps/agent-shell` 现消费 M5 HTTP/SSE 面；Pi 表面由 P1-T07 `personal/packages/pi-cognitiveos/` 承载。
- **依赖：** `implementation_requires`: P1-T07、P2-T01 与 task/management channel contracts；`acceptance_requires`: real API/watch、CLI/Shell parity 与 sidecar/client channel isolation；与 P2-T03 可并行（§12.1）。
- **不包含：** scheduler/worker 执行（P2-T03/T04）；独立造类型（`packages/sdk-ts` 合同跟随 Lane-CTR，§12.2）。
- **文件：** 修改 `apps/kernel-server`（resource/task channel 路由）、`packages/sdk-ts`、`apps/agent-shell`、P1-T07 Pi package/sidecar composition；新增 real API/watch 与 TS parity tests；公共合同变化一律经 Lane-CTR；无删除文件。
- **数据/API/配置/迁移：** 不新增 giant Resource 表；统一 projection 先 private + versioned；watch 基于 SSE resume/dedup 与 server cursor；错误映射真实 HTTP status，无 canned 200。第二个真实 adapter/client 出现后才评估 public `ResourceSummary`。
- **步骤：** (1) 路由接 P2-T01 和现有 resource application ports；(2) 建立六类 private versioned projection；(3) CLI 与 Shell→sidecar 两表面统一消费；(4) watch resume/dedup 与 detach/cancel；(5) task/management channel isolation negatives。
- **验收：** detach 不 cancel；watch resume/dedup/cursor 正确；cancel 只产生 authority request；server-issued preview 绑定 admission；CLI 与 Shell 不产生第二策略路径；sidecar 的 task/management bearer、retry、cache、cursor 与 projection 完全隔离，普通 conversation 不能以措辞升级为 management context；Extension/sidecar 不持 daemon bootstrap/management authority；未知 Tier 默认 Tier 2。
- **测试：** Rust 集成 + TS build/test；负例：wrong channel、过期 bearer、stale epoch cancel、伪造 watch cursor；命令按 §15.2。
- **基准/性能：** 记录 NL→management intent→六资源 projection/watch 延迟与 token（B02/UCR-01 口径）；无目标值。
- **安全/可观测：** 三表面均无 secret/authority 泄漏；管理映射来源入审计；SSE 有界。
- **回滚/文档：** 单 PR 可 revert 回 canned 实现；文档联动 §6 清单。
- **解锁：** B02（证据采集口）、P2-T04；application contracts 稳定后解锁 P3-T01，不等待 P2-T08 acceptance。
- **风险/不确定：** SSE dedup 键与 event envelope 稳定性；Pi 表面在 P0-T06 收尾前只能以 fixture 验证，不得写成 runtime load evidence。

### P2-T03 — durable scheduler、lease 和 timer

- **目标/价值：** 准入后的任务在 daemon 内可恢复地调度：scheduler 状态 durable，worker crash 后无双活、无双派发（§6 Scheduler 现状 L0）。
- **状态：** 以正式台账为准。
- **证据/研究：** `cognitive-store` 已有 `faults.rs` 故障注入与 `clock.rs`；lease/epoch fencing 模式可复用 `fencing` 表语义。
- **依赖：** P1-T01、P2-T01；与 P2-T02 可并行（§12.1）。
- **不包含：** Temporal/queue server 等外部编排；多 Agent 调度策略（P6-T01）；process supervisor（P2-T06）。
- **文件：** `crates/cognitive-store` 新增 scheduler repository 模块（按 §12.2 先拆模块再占所有权，避开 `sqlite.rs` 热点）；`crates/cognitive-runtime` 新增 scheduler service；新增 `personal/crates/cognitive-runtime/tests/p2_t03_scheduler_lease_timer.rs`；无删除文件。
- **数据/API/配置/迁移：** 新增 scheduler 持久化：runnable、lease owner/epoch、next eligible、attempt、cancel request；**数据迁移必须由 P1-T01 框架执行**（migration 编号单一分配，§12.2）；配置：poll/timer 间隔、lease TTL、deadline/retry/step/cost ceilings。
- **步骤：** (1) migration + repository（先失败测试）；(2) lease acquire/renew/expire 与 epoch fence；(3) timer/next-eligible 推进；(4) cancel request 传播；(5) crash/clock 故障注入。
- **验收：** worker crash 后 lease 到期可被安全接管且旧 epoch 被 fence；duplicate lease 不可能（CAS 负例）；clock shift 不产生双派发或饿死（时钟策略明确记录）；deadline/retry/step/cost ceiling 到达即停并落 authority 事实；loop dispatch durable quiescence、pending Effect closure/quarantine proof 与 worker-side stop integration 不得用 evaluator return value 替代。
- **测试：** SQLite 集成 + fault injection（§15.1 层 3/7）；负例：双 worker 抢同 lease、续租已过期 lease、cancel 后再派发；命令按 §15.2。
- **基准/性能：** 记录调度决策延迟与空转率；无目标值。
- **安全/可观测：** scheduler 不成为第二 authority：状态推进仍经 kernel transitions；lease 事件可查询。
- **回滚/文档：** migration 按 P1-T01 backup 语义可回滚；scheduler 可停用回 admit-only 模式；文档联动。
- **解锁：** P2-T04、P2-T07。
- **风险/不确定：** 单机多 worker 与 WAL 写争用；wall/monotonic 时钟选型须在实现批固化并记录理由。

### P2-T04 — scheduler→Context→Pi sidecar→BoundedHarness worker

- **目标/价值：** 把 scheduler 拉起的任务接入真实链路：scheduler→TaskContract→Context port→pinned Pi sidecar→candidate→BoundedHarness/LoopDriver，形成 sidecar 仍非 authority 的产品 worker。
- **状态：** 以正式台账为准。
- **证据/研究：** `core/crates/cognitive-kernel/src/harness.rs`（`LoopDriver`）、`personal/crates/cognitive-runtime/src/harness_loop.rs`（`BoundedHarness`）与 `loop_progress_facts` 表已存在。
- **依赖：** P2-T02、P2-T03。
- **不包含：** durable Memory/Skill（P4）、MCP、dynamic Tool、多 Agent；不允许 Pi Extension 直接管理 daemon。
- **文件：** `crates/cognitive-runtime` 新增 worker 模块（组装链路，不复制 harness 逻辑）；新增 `personal/crates/cognitive-runtime/tests/p2_t04_worker_harness.rs`；无删除文件。
- **数据/API/配置/迁移：** 不新增表；progress facts 复用 `loop_progress_facts`；预算绑定取自 TaskContract，准入后为硬轨。
- **步骤：** (1) worker 拉取 runnable + lease；(2) 每轮重载 contract/governance/lease 并解析真实 Context；(3) 通过 pinned Pi sidecar 请求 candidate；(4) BoundedHarness 判 progress；(5) no-progress/budget/stale-lease/wait-user 停机与恢复。
- **验收：** 每轮重新加载 contract/governance/lease；Context required fragment fail-closed；sidecar 只能返回 candidate/proposal；模型 self-report 不算 progress；no-progress 触发 switch/block；wait-user 不消耗预算；budget stop 落 authority 事实；stale lease 立即终止且无状态写出。
- **测试：** 纯逻辑 + 集成；负例：stale lease 内尝试提交、self-report 伪 progress、预算溢出后继续执行；命令按 §15.2。
- **基准/性能：** 记录每轮 overhead，并按 §3 实验轨道口径拆分 CognitiveOS deterministic overhead 与 Provider/network latency。
- **安全/可观测：** loop telemetry 事实化（P3-T04 输入）；无 secret 进入 loop 记录。
- **回滚/文档：** worker 可停用且 scheduler 状态不损坏；文档联动。
- **解锁：** P2-T05、P2-T07。
- **风险/不确定：** P2-T07 verifier 就绪前 progress 判定的降级口径：保守计 no-progress，不得放宽为 self-report。

### P2-T05 — Native Tool Registry 与 useful operation family

- **目标/价值：** 建立 immutable native ToolDescriptor registry 与 Linux 1.0 useful family，使 tool call 只能转成 catalog-bound OperationCandidate：workspace read/search/write/patch、bounded process/check、read-only HTTP fetch。
- **状态：** 以正式台账为准。
- **证据/研究：** §6 Tools 现状 L1：`ToolAdapter`、executor ports 存在（`core/crates/cognitive-kernel/src/{effects,executor,ports}.rs`），无 Registry/real executor。
- **依赖：** P2-T04。
- **不包含：** MCP adapter/dynamic marketplace（P5-T03/T04）、unbounded generic Bash、未经 catalog 的任意网络 mutation。
- **文件：** `crates/cognitive-runtime` 新增 tool_registry 模块；新增 `personal/crates/cognitive-runtime/tests/p2_t05_tool_registry.rs`；无删除文件。
- **数据/API/配置/迁移：** 建立 immutable descriptor：schema digest、risk、effect class、query/idempotency、sandbox、verification、health、state（enabled/disabled/quarantined），并含 ADR-0026 Tier 分类元数据；如需持久化经 P1-T01 迁移框架。
- **步骤：** (1) descriptor/version/digest 与不可变性；(2) 注册/启停/隔离；(3) workspace read/search/write/patch；(4) bounded process/check；(5) read-only HTTP fetch；(6) candidate→operation 解析与错误实现自检。
- **验收：** 未注册、schema drift、disabled/quarantined、伪造 capability 均拒绝且 **dispatch=0**；workspace 写/patch 受 scope/CAS/Intent-Effect；process/check 有 argv/env/cwd/output/time bounds；HTTP 只读、拒绝 redirect/credential/超限；每个可 mutation operation 具 stable idempotency/reconcile。
- **测试：** 合同/纯逻辑/集成；自检：放行 drift descriptor 的故意错误实现必须 fail；命令按 §15.2。
- **基准/性能：** 记录 registry 解析延迟；无目标值。
- **安全/可观测：** default deny；descriptor 生命周期全审计；descriptor 无 secret 字段。
- **回滚/文档：** operation 可禁用；如有表按迁移框架回滚；文档联动。
- **解锁：** P2-T06。
- **风险/不确定：** 第一个 operation 具体选型（倾向 workspace 查询/幂等窄写）在实现批定案并记录理由；Tier 元数据与 P5-T01 capability lease 模型的衔接。

### P2-T06 — Tool/process executor、supervisor、cursor、fault 与 reconcile

- **目标/价值：** 为 P2-T05 Tool family 提供 daemon-owned Tool/process executor 与 supervisor：stable identity、有界 cursor、timeout/stop/restart、fault handling 与 reconcile，使 Effect 协议在真实执行上落地。
- **状态：** 以正式台账为准。
- **证据/研究：** `core/crates/cognitive-kernel/src/{executor,effects}.rs` 的 Effect 协议与 `intents`/`outbox` 表；`personal/crates/cognitive-runtime/src/sandbox.rs`；`cognitive-store/src/faults.rs` 故障注入模式。
- **依赖：** P2-T05。
- **不包含：** Linux-native OS sandbox 强化、Pi/sidecar package lifecycle（P5-T01/T02）、并行 executor 池；不新增 public Process domain。
- **文件：** `crates/cognitive-runtime` 新增 process_supervisor/executor 模块；新增 `personal/crates/cognitive-runtime/tests/p2_t06_process_executor.rs`；无删除文件。
- **数据/API/配置/迁移：** 新增 stable process/task/attempt/epoch identity、CWD、stdout/stderr cursor、timeout、stop、restart、reconcile 记录；持久化经 P1-T01 框架；配置：timeout、output limit、restart policy。
- **步骤：** (1) Tool/process identity 与 **persist-before-dispatch**；(2) stdout/stderr/result cursor 与上限；(3) timeout/stop/restart；(4) before/mid/after dispatch fault injection；(5) orphan/outcome-unknown reconcile；(6) 幂等冲突拒绝。
- **验收：** crash before/mid/after dispatch 三相故障后恢复且无重复副作用（稳定幂等键 + reconcile）；orphan 进程被发现并终止/对账；output limit 截断留证据；secret redaction（env/argv/log）负例通过；same-key/different-input 拒绝；stdout zero exit 只作为 evidence，不得直接判 Effect 成功或 Task 完成。
- **测试：** fault injection 全三相 + 集成；自检：zero-exit 即 completed 的故意错误实现必须 fail；命令按 §15.2。
- **基准/性能：** 记录 spawn→dispatch 延迟与 reconcile 耗时；无目标值。
- **安全/可观测：** 子进程环境最小化、不继承 secret；进程事件可查询。
- **回滚/文档：** executor 可禁用回 candidate-only dry-run；文档联动。
- **解锁：** P2-T07。
- **风险/不确定：** OUTCOME_UNKNOWN 判定窗口与对账查询可用性；首发仅 Linux 进程语义（Windows 安装面归 P7-T07）。

### P2-T07 — Checkpoint、Artifact、Evidence 与独立 Completion Verifier

- **目标/价值：** 任务闭合证据链：把 checkpoint、effect closure、artifacts、criteria results、verification event 接入 task closure；完成只能由独立 verifier 判定（无假完成）。
- **状态：** 以正式台账为准。
- **证据/研究：** `checkpoints` 表与 kernel verification/acceptance 语义已存在（§6 Verification L4 core / Checkpoint L3）；`core/crates/cognitive-kernel/src/recovery.rs` 恢复协议与 `personal/crates/cognitive-runtime/src/recovery_flow.rs`。
- **依赖：** P2-T03、P2-T04、P2-T06。
- **不包含：** Memory 证据（P4）、性能 campaign（P7-T04）、多 Agent reviewer 编排（P6-T03）。
- **文件：** `crates/cognitive-runtime` 新增 verifier service 与 closure 组装；新增 `personal/crates/cognitive-runtime/tests/p2_t07_verifier_closure.rs`；无删除文件。
- **数据/API/配置/迁移：** criteria result（criterion、pass/fail/unknown、evidence digest）持久化经 P1-T01 框架；artifact 引用只接 P3-T03 的唯一 filesystem CAS + authority metadata contract，不建平行 store。P3-T03 未实现前用 stable port + failure-first fake，不把 fixture 当 acceptance。
- **步骤：** (1) verifier port 与执行 agent 分离（独立代码路径与 principal）；(2) criteria evaluation→verification event；(3) effect closure 汇总（存在未闭合 Effect/OUTCOME_UNKNOWN 时拒绝闭合）；(4) partial completion 语义；(5) recovery 顺序严格使用现有 `recovery.rs`，不重排、不复制。
- **验收：** verifier 与执行 agent 分离；每个 criterion 记录 pass/fail/unknown 与 evidence digest；Partial completion 不得升级为 completed；remote done 和 receipt 不够（须独立证据）；未闭合 Effect 存在时任务不得 completed。
- **测试：** 集成 + 自检：伪完成实现必须 fail（为 P2-T08 False Completion=0 提供地板）；命令按 §15.2。
- **基准/性能：** 记录 verification 延迟；B04 verified criteria 口径对齐 §14。
- **安全/可观测：** evidence digest 链可审计；无 secret 入 evidence。
- **回滚/文档：** verifier 停用时任务只能停在待验证态，不得自动完成；文档联动。
- **解锁：** P2-T08。
- **风险/不确定：** criteria evidence 最小 schema 与 conformance 向量的衔接——若需合同登记走 Lane-CTR。

### P2-T08 — Runtime Spine E2E Gate

- **目标/价值：** 自动化真实 projection→scheduler→Context→Pi sidecar→native Tool/process→checkpoint/recovery/verifier 的 B02、B04、B05、B12 E2E 套件与证据采集，构成 Runtime Spine 出口。
- **状态：** 以正式台账为准。
- **证据/研究：** §13 B02/B04/B05/B12 规格与 §14 指标；依赖 P2-T01..T07 全链路。正式 Gate 环境不可得时，suite 本身可按 experimental-local-only 推进，Gate 结果保持 `not-run` + blocked 原因（台账解耦注记）。
- **依赖：** P2-T07。
- **不包含：** B01（P1-T09）、B03/B06/B07（P3-T06）、性能 campaign（P7-T04）。
- **文件：** 新增 `personal/tests/e2e/personal/b02|b04|b05|b12-*` 与 evidence schema/runner（复用 P1-T09 runner 骨架）；无删除文件。
- **数据/API/配置/迁移：** raw evidence 入 ignored `artifacts/evidence/personal/`；summary 带 suite digest 与 non-claim（§13 通则）。
- **步骤：** (1) suite harness 与 UCR-01-compatible Task trace；(2) projection/watch 与 sidecar channel isolation；(3) Shell/daemon/process 故障和 OUTCOME_UNKNOWN；(4) false completion/盲重试负例；(5) ADR-0018 到期核查；(6) Tier-2 与 evidence redaction。
- **验收：** 必须覆盖 Shell 关闭、daemon 关闭、OUTCOME_UNKNOWN、不盲重试（不换幂等键，原键可查证）、false completion negative；建议普通重启 authority state recovery=100%；False Completion Rate 在 gate suite 中必须 0/所有故意错误案例；核查 ADR-0018 本机开发例外已到期移除（或已替换为 daemon proxy 并重新批准），例外残留视为 Gate 不通过；B04 证据记录默认路径人工确认次数（目标 ≤1/task，Tier-2 除外），并含 Tier-2 负例：purge 类操作缺显式确认必须失败（ADR-0026）。
- **测试：** E2E + 每个关键 gate ≥1 个故意错误实现自检；命令按 §15.2 并追加 suite。
- **基准/性能：** 按 §13 通则采样（≥30 次有效 run，装机类高成本场景 ≥20 次）并报告 median/p95/bootstrap CI；未测不宣称。
- **安全/可观测：** evidence 全 redacted、无 key；确认次数字段入 evidence rows（ADR-0026）。
- **回滚/文档：** Gate fail 只允许修复或 revert，不得带红宣 GO；台账 Gate 行与文档联动。
- **解锁：** Runtime Spine acceptance；P3-T01/P3-T02 的 implementation 已可在 P2 application contracts 稳定后并行，不等待本 Gate。
- **风险/不确定：** B04/B12 需真实 Provider key 与环境；不可得时按 §3 纪律记 blocked/not-run 并转做其他任务，不停摆。

---

## Phase 3 — Context、Token 与 Loop 效率

### P3-T01 — Context source/retrieval port

- `implementation_requires`: P2-T01/P2-T02 stable application contracts；不等待 P2-T08 acceptance/Gate。
- 为现有 `context.rs::resolve` 提供真实 workspace/task/evidence source，接 private versioned Resource/Task projection。
- candidate references 先 scope-filter，正文授权后才交给 ranker；测 revoked/out-of-scope/rank-before-auth/source-version variants。
- 不含 Memory semantics；stable source port 解锁 P3-T02 与 P4-T01。

### P3-T02 — 最小充分 Context Builder 与预算

- `implementation_requires`: P3-T01 stable Context source port。
- 建立真实 System/Shell/Task/Working/Evidence fragments、required/optional、dedup、freshness、token budget，并保留同一 Task trace。
- Required fragment缺失或超预算 fail-closed；loss显式。
- 记录 fragment source digest、included/excluded reason、token estimate/actual。
- stable builder port 解锁 P3-T03/T04 与 P4-T01/P4-T04。

### P3-T03 — 唯一 Artifact CAS

- 只实现一个 filesystem CAS + authority metadata port；限制 size、retention、access、content-type，不建立 checkpoint/evidence 专用平行 store。
- 大日志/工具输出外部化，Context 只持摘要和 digest-bound 引用；P2-T07 通过同一 port 消费。
- 测 digest mismatch、partial write、orphan GC、unauthorized fetch。
- 解锁 P3-T04。

### P3-T04 — Context delta、stable prefix、cache 与 telemetry

- 在最小 Context Builder 上实现 delta assembly 与 stable prefix；cache key 绑定 source/version/governance/Tool/Task epoch。
- 记录 model tokens/cache/latency/cost、tool calls/failures、progress points、retries、loss 与 loop signatures。
- Loop detection比较 action+target+error+evidence digest；触发 switch/wait/block，不无限 retry。
- Tool result先结构化过滤，再摘要；Pi compaction仅压 presentation session；TaskContract/criteria/evidence不从 Pi summary恢复。
- 测 constraint loss、stale cache、revocation、required source removed、no-progress/repeat/strategy 控制可观测。
- Cardinality和日志体积有预算；不记录 secret/raw sensitive body。
- 解锁 P3-T05。

### P3-T05 — UCR-01 benefit runner 与性能基线

- 实现 [UCR-01](../evaluation/personal-unified-cognitive-resource-workload.md) runner：同一 Task trace 使用六类资源；建立 raw run、稳定基线、CI 采集和 non-claim 报告。
- 预注册后可为 B02/B03/B04/B05/B08/B09/B12 分别贡献 evidence；每个 Gate 单独绑定环境、阈值和 verifier，一次 run 不自动 pass 多 Gate。
- 固定场景 assertions 包含 cross-session recall、Skill digest reuse、required recall=100%、
  unauthorized/stale=0、duplicate Effect=0、false completion=0，以及 stable/changed Context
  相对 full replay 重复输入 token 降低 `>=20%` 且 verified completion 不下降；它们进入
  P7-T08 acceptance，但不自动成为跨 W1/W2 的一般 Agent-benefit claim。
- 解锁 P3-T06。

### P3-T06 — B03 Context correctness 与 B06/B07 采集

- MVP B03 依 ADR-0040 使用固定 33-test authority-path/evaluator denominator：真实
  source、scope-before-ranking、required fail-closed、显式 loss、Artifact access、
  revocation 与 stale-cache negatives；并要求 qualified native Linux/Clippy、Ubuntu/
  Windows CI、redacted cleanup 和 owner review。更大 workload/statistical campaign 进入
  后续 promotion，不是 MVP mutex。
- B06/B07 只采集可重复的 delta/stable-prefix/cache/loop raw metrics；未达或未运行不
  阻塞 Linux 1.0，也不得形成一般 Agent-benefit claim。UCR-01 的固定场景 utility
  assertion 仍由 P7-T08 单独验收，不等同于 B06/B07 pass。
- `acceptance_requires`: B03 correctness；`promotion_requires`: GMVP-LINUX Gate composition
  只 requires B03。P4 implementation 可在 P3-T01/T02 stable ports 后并行。

---

## Phase 4 — Memory 与 Skill

### P4-T01 — Memory store、admission 与 policy

- `implementation_requires`: P3-T01/P3-T02 stable ports，不等待 P3-T06 benefit evidence。
- 复用现有 Memory schema/adapter域；新增 source/version/content digest、scope、purpose、provenance、confidence、freshness、retention、tombstone。
- user/Task/Agent 只能提出 Memory mutation proposal；deterministic policy admission 决定写入、更新、刷新与拒绝。
- 预计修改 store/runtime，新增 migrations/tests。
- 不新增 vector DB、graph DB或跨 workspace recall。
- 解锁 P4-T02/T03。

### P4-T02 — SQLite FTS5 + metadata filter baseline

- 对允许索引的 Memory source建立 SQLite FTS5；source row是权威，FTS5可重建。
- 查询先按 scope/purpose/freshness/retention metadata filter，再授权正文，最后 FTS 排序。
- 测 precision/recall corpus、stale index、delete/rebuild、latency。
- 解锁 P4-T05；embedding/vector/graph 不在本任务。

### P4-T03 — Memory lifecycle、冲突、retention 与 forget

- 更新生成版本；冲突显式；forget写 tombstone并失效派生索引。
- 敏感 Memory默认不外发 Provider；删除证据不含原文。
- 测 create/update/conflict/expire/forget/rebuild/audit；解锁 P4-T05。

### P4-T04 — Skill package、revision、local import 与 binding

- 定义 local Skill package/revision/digest/import lifecycle；Skill content 进入 Context 前必须 scope/auth 检查。
- 支持 Agent/Task/workspace binding、supersede/revoke 与 explain；binding 不是 capability，Skill 不自授权、不直接写 authority。
- 测 digest drift、unsafe path、revision mismatch、revoked binding、未授权 import 与 cross-workspace leakage。
- public `skill-manifest` 如确有需要由后续 Lane-CTR 单独登记；本任务不私造公共合同。解锁 P4-T05。

### P4-T05 — Memory/Skill API 与统一 projection

- `implementation_requires`: P4-T01/T02/T03/T04；明确不依赖 embedding。
- scheduler只处理已批准 Memory retention/refresh/summary 与 Skill lifecycle 工作。
- 摘要保存 source refs、transform version、loss declaration，不能替代 required source。
- Memory retrieval 与 Skill binding 通过同一 Context/Task application ports；CLI、Shell→sidecar 调相同 services 与 private versioned projection。
- 支持 remember/list/explain/update/forget 与 skill import/list/bind/revoke/explain。
- actual consumption 必须可追到同一 Task/Context/evidence trace。
- 不含自动从全部聊天抽取永久记忆；解锁 P4-T06。

### P4-T06 — B08 Memory+Skill correctness 与 UCR-01 consumption

- 执行 B08：Memory provenance/freshness/conflict/forget/privacy + Skill package/revision/binding/revoke，并证明 UCR-01 同一 Task trace 实际消费两类资源。
- stale/forgotten Memory、revoked Skill、projection-only display 或仅 API roundtrip 都不能满足 actual consumption。
- Embedding/vector/graph 明确 post-1.0；B08 与 P4-T06 acceptance 不依赖它们。

---

## Phase 5 — Managed Pi sidecar 与 post-1.0 Tool 生态

### P5-T01 — Agent + sidecar package acquisition 与安装生命周期

- 复用 runtime/store installation authority，形成 adapter-neutral Agent + sidecar acquisition/install/update/rollback/uninstall framework；不依赖 durable Memory 才能开始。
- Linux 1.0 首个实现从固定 official npm origin 获取 exact `@earendil-works/pi-coding-agent@0.81.1`，验证 package identity/version/SRI、package/dependency digest、Node compatibility 与 adapter digest。
- 同时固定 sidecar package/protocol/adapter digests；production release trust 签署 acquisition lock，签名只表示 CognitiveOS review/admission，不把 npm SRI 宣称为 publisher provenance。
- staging/commit/activation 分离；安装不自动产生 Tool/workspace/model/secret capability。首次使用可按 ADR-0026 产生 scoped capability lease，`cognitive grants` 可列出/撤销。
- upgrade 创建 immutable new installation；失败恢复旧 binding；uninstall 先 quiesce/fence/reconcile，再移除 package bytes，保留 policy 要求的 history/evidence。
- 测 floating/latest、错误 origin/redirect、identity/SRI/digest drift、lifecycle scripts、unsafe archive、tamper、dependency drift、interrupted update/rollback/uninstall。
- 解锁 P5-T02 与 B09 slice。

### P5-T02 — Sidecar contract、registration、instance/process identity 与 Pi foundation

- `implementation_requires`: P5-T01 与所需 P2-T03/P2-T06 supervision contracts；无需等待 B10/P5-T04。
- 以 Pi + exact sidecar 为 1.0 首个 managed Agent integration，定义 versioned sidecar contract/registration、Agent Definition/registry、实例健康、budget、tool/Memory/Skill/workspace scope 与 process-supervision binding。
- ShellSession、Pi Session、SidecarPackage/Installation/Session/process、AgentInstallation/Instance/Execution、process 与 Task 不得混用；安装不等于 activation，`agent_end`/process exit 不等于 completion。
- 支持 health/activate/pause/resume/stop/recover；resume 建立新 epoch 并重新授权，旧 instance/execution output 被 fence。
- Agent/sidecar concepts 先复用既有 package/installation/execution contracts；public `agent-adapter-manifest` 如需要必须另走 Lane-CTR，不在实现中私造合同。
- install ≠ permission 保留（REQ-AGENT-INSTALL-001/002）；低摩擦只改首用授予交互，不改默认拒绝底座（ADR-0026）。
- 解锁 P5-T05 的 B09 slice。

### P5-T03 — Post-1.0 MCP Tool adapter qualification

- Tool package manifest固定 schemas、operation descriptors、transport、risk、health、reconcile。
- MCP initialize/capability/version/timeout只建立 transport。
- server tool list变化生成候选，需重新 qualification，不自动启用。
- 测 malicious MCP、schema drift、prompt injection、direct endpoint bypass。
- 属于 1.0 后 B10 capability train，不阻塞 B09/GMVP-LINUX。
- 只解锁 P5-T04；不解锁或阻塞 P5-T05。

### P5-T04 — Post-1.0 dynamic Tool ecosystem 与 B10

- 每轮只暴露 TaskContract允许且当前健康的最小 tool集合。
- Composite Tool必须保留子操作Intent/Effect/evidence，不可隐藏 unknown outcome。
- Tool cache只用于明确纯读、版本绑定操作。
- 记录 Tool Schema Token Cost、result utilization、cache hit。
- 独立执行 B10 MCP/dynamic Tool campaign；不解锁或阻塞 P5-T05/B09/GMVP-LINUX。

### P5-T05 — B09 managed Pi + sidecar qualification

- B09 在 P5-T02 后执行：official Pi + sidecar acquisition lock、install/register、protocol/adapter/instance/process pins、health、activate、pause/resume、upgrade/rollback、stop/uninstall、recovery 与 identity/permission negatives。
- 本任务只负责 B09；完成条件与 B10 解耦。B10 在 P5-T04 独立执行且不阻塞 Linux 1.0。
- 所有 package 使用 exact version/ref/digest；任何 tampered case必须 0 activation/dispatch。
- Pi + sidecar 的 B09 证据不可资格化 OpenClaw、Hermes、Codex、WorkBuddy 或其他 adapter/sidecar。

---

## Phase 6 — Multi-Agent

> **2026-08-10 对齐注记：** 正式产品设计将 multi-agent 设为架构正线（ADR-0044），默认
> fail-closed；Linux 1.0 仍只资格化 Pi。下列 P6 卡保留为历史研究草案；实现主路径以
> Phase 8 adapter/qualification 与架构章 `multi-agent-orchestration.md` 为准，不得用
> 本节约束当前正式任务领取。

### P6-T01 — Delegation admission与AgentInstance/child Task

- 仅当可并行、上下文可隔离、需独立验证时允许。
- delegation是 child Task proposal；scope/tool/budget必须收窄。
- 无 transferable authority token；parent cancel传播有明确规则。
- 解锁 P6-T02。

### P6-T02 — Append-only Blackboard、Findings和Worktree

- blackboard只存 findings/evidence refs/status proposals，非 authority。
- 写任务使用独立 worktree；read-only研究可共享只读 snapshot。
- 冲突检测基于路径所有权和base commit；不得自动覆盖。
- 解锁 P6-T03。

### P6-T03 — Reviewer、Verifier、Integrator与join/cancel

- Worker不做最终 acceptance；Reviewer检查结果，Verifier执行criteria，Integrator只合并已验证产物。
- join需处理partial/fail/timeout/cancel/dead letter。
- 测重复工作、消息replay、stale agent lease、false child success。
- 解锁 P6-T04。

### P6-T04 — B11收益 Gate

- 与相同模型、预算、任务的单 Agent baseline对比。
- 建议启用条件：wall-clock speedup≥1.25x或verified quality显著提升，同时 coordination token overhead≤35%、duplicate work≤10%、merge conflict≤5%。
- 不满足则默认关闭，功能保持 experimental。

---

## Phase 7 — 产品化

### P7-T01 — Release pipeline、SBOM和attestation

- 新增 Linux x86_64 reproducible build、manifest、checksums、SBOM、GitHub attestation和verification test。
- release manifest 固定 Memory/Skill/Tool/Context/Task/Runtime 六类 schema/version/digest，以及 sidecar protocol/adapter、Skill package、Tool descriptor/catalog pins。
- 固定 Rust/Node/pnpm/Pi/sidecar/lockfiles、CI actions/toolchain/environment identities，并由 production trust 签署 product manifest 与 Pi acquisition lock。
- release artifact不得含 Key、test DB或开发路径。
- 解锁 P7-T02/T06。

### P7-T02 — Transactional update、rollback和uninstall

- update先验证→stage→migration preflight→health→atomic switch。
- downgrade只在数据兼容明确时允许；否则恢复旧 binary+DB backup。
- uninstall区分 binary/config/cache/data/secret，删除数据需要显式二次确认。
- 面向用户的 `cognitive backup`/`cognitive restore` 命令：覆盖 state/config/artifacts、Memory、Skill packages/revisions 与 Agent/Task/workspace bindings，排除 secret；restore 走 schema/migration preflight（ADR-0026）。
- B01增加upgrade/interruption/uninstall cleanliness。
- 解锁 P7-T06。

### P7-T03 — Doctor、support bundle和故障排查

- `cognitive doctor --bundle`只输出redacted facts/digests，不含secret和敏感正文。
- 覆盖六类 Resource health、daemon/DB/SecretStore/Provider、Pi+sidecar protocol/adapter/instance drift、Tool catalog、process cursor/supervision、pending Effect/reconcile 与 migrations。
- 实现并资格化 desktop Secret Service 与 headless encrypted-vault 两条同-port 路径：
  headless locked diagnostic start、SSH TTY unlock、可选 systemd encrypted credential
  vault-unlock material；Provider/user secret 不得进入 unit/credential/env/argv。
- 验证 Standard Workspace 与 Extended Home：选定 document/project roots + ordinary
  outbound network 可用，Secret/SSH/GPG/browser credentials、authority/bootstrap、
  Docker/system sockets、system directories 与 privilege management 始终拒绝。
- 可操作错误包含next step和stable error code。
- 解锁 P7-T06。

### P7-T04 — 完整性能 campaign和回归地板

- 固定硬件、Provider snapshot、模型、任务集、warm/cold cache、重复次数。
- 输出raw run、summary、confidence interval、baseline delta、non-claims。
- 回归阈值由测量后确定，不预设虚假达成。
- 解锁 RC。

### P7-T05 — 非阻塞 Web UI

- 仅在客户端 readiness gate、技术栈 ADR、法务和 daemon API稳定后启动。
- 计划路径为独立仓库 cognitiveos-clients 内的 `pc/web/`（本仓不落 `clients/`）；React/TS/Vite 已由 [ADR-0053](../adr/0053-personal-web-ui-stack.md) 接受，不是未裁决候选。
- 只渲染 system/provider/tasks/agents/processes/tokens/tools/memory/evidence projections。
- 不直接打开数据库、不做授权/完成判定。
- 不阻塞 RC CLI+Pi release。
- 设计文档：[Web UI 产品设计](../../personal/docs/product/web-ui-design.md)、[Web UI 架构](../../personal/docs/architecture/web-ui-architecture.md)、[P7-T05 任务卡](p7-t05-web-ui-task-card.md)、[route inventory](../../personal/docs/architecture/web-ui-route-inventory.json)。任务卡中的 D01-D10 是同一正式任务内的交付切片，不改变本任务的 post-1.0、non-blocking 边界。Owner 2026-08-23 将已关闭的 D01–D07 shell 重开为控制面板跟进；D08/D09 Linux-validated at kernel `881ebe82` / clients `c6b763b`（live key、binding CAS、Task admit、dsh Path B `assistant_ok`；HTTP cancel `not-run`）。Owner 2026-08-24 增补 D10：在 official clients checkout 中以 Apple-inspired 克制视觉完善 Provider shell/list/detail/status/action/loading/empty hierarchy，不改变 daemon API、SecretStore、binding CAS 或 Task 行为。Local clients `07f7513e` 已通过 29 tests、production build 与双 viewport review，但 official `agentkernel/cognitiveos-clients` push 被 GitHub HTTP 403 拒绝，remote branch / Draft PR 不存在；P7-T05 因此外部权限阻塞，恢复入口见 [D10 blocked closure](../checkpoints/20260824-personal-p7-t05-provider-webui-apple-theme-closure.md)。Approved checkout: `D:\cognitiveos-clients` (`pc/web/`)。

### P7-T06 — RC、文档、支持矩阵和B01-B12

- **目标/价值：** 为 Linux 1.0 发行工程冻结声明范围、digest 绑定证据与双语
  runbook；不新跑 B01 guest，不冒充 Profile，不发布生产 GitHub Release。
- **依赖：** P7-T08、P7-T04、P5-T05（均已 done）。
- **验收：** clean-VM suite 合成为 B01 successor `002` + P7-T01/T02 权威路径；所有
  release claims 指向 evidence digest；`implemented` 仍只按适用 MUST 计；发布
  install/init/provider/Pi/task/recovery/update/uninstall runbooks（无发明公开动词）；
  本 RC 范围 open critical risks = 0；P6 对本 RC 为明确 disabled-NO-GO。
- **切片：** D01 claim freeze + binder；D02 双语 runbook；D03 合成报告；D04 收口。
- **非声明：** 不设置 Gate 状态；不声称 Profile、Windows B01-W、生产签名仪式、
  B10/MCP、Web UI 或 Multi-Agent 启用。

### P7-T07 — Windows 安装面：credential 后端、installer/service 与 B01-W Gate

- **目标/价值：** 为 ADR-0025 已决定的 Windows x86_64 首发平台补齐安装面唯一落点；在此之前 Windows 仅为 daemon/CLI 产品路径。
- **依赖/不包含：** P1-T02（secret 边界）、P7-T01/T02（release/update 管线）；不包含 macOS/aarch64/WSL2、不阻塞 Linux RC。
- **验收：** Windows credential store 后端满足与 Linux 相同的 fail-closed 边界且无明文 fallback；可检查 installer/service；编写并执行专门 B01-W Gate（清洁 Windows VM install→first dialogue）。
- **安全：** 与 ADR-0018/0020 同一 secret 边界；ADR-0018 本机开发例外明确不适用于 Windows。
- **非声明：** 未执行 B01-W 前不得声称 Windows install parity（ADR-0025）；本卡完成不改变 Profile/Gate 结论。
- **回滚：** installer 失败原子回滚；不影响已发布 Linux bundle。

### P7-T08 — Public Linux 1.0 Gate（GMVP-LINUX）

- **目标/价值：** 由既有 `GMVP-LINUX` 汇合 Runtime Spine、Resource Value、Product Operability 并推广 Personal `1.0.0`；不新增 B13 或第二 release Gate。
- **依赖：** `acceptance_requires`: P1-T09、P2-T08、P3-T06、P4-T06、P5-T01/P5-T02/P5-T05 B09、P7-T01..T03；`promotion_requires`: **B01+B02+B03+B04+B05+B08+B09+B12**。B06/B07/B10/B11 不阻塞。
- **验收：** 六类最小真实 resource slice 与 UCR-01 same-Task consumption 和 fixed-scenario correctness/utility assertions、production trust、native user-systemd、desktop Secret Service/headless encrypted-vault locked/TTY/unattended paths、official Pi+sidecar lifecycle、update/rollback/uninstall、Memory/Skill backup/restore、six-resource doctor/support 均有 separately qualified executed evidence；open critical risk 为 0 或明确 NO-GO。
- **声明边界：** release manifest 列出六类 schema/version/digest、sidecar/adapter/skill/tool pins 与包含/排除项；只支持 Linux x86_64 + pinned Pi+sidecar。Embedding/vector/graph、MCP/dynamic Tool、Multi-Agent、Web UI、Windows 与 non-Pi adapter 不得被暗示为可用。`GMVP-LINUX` 不是 REQ、registry Gate 或 Profile。
- **失败语义：** 任一 exact promotion benchmark、trust/native service、六类 resource correctness、UCR-01 fixed-scenario assertion、Pi+sidecar、desktop/headless SecretStore、rollback/backup/doctor、secret redaction 或 independent evidence 缺失即 NO-GO；CI/WSL/fixture 不能替代。
- **回滚：** 保持最新可信 non-release artifact；不提升 B01、RC 或 Profile 状态。

---

## Phase 8 — 通用 Agent 适配与设计基线（研究卡；正式状态见台账）

> 正式定义与状态以 `PERSONAL-DEVELOPMENT-PLAN.md` Phase 8 为准。本卡只补充研究细节。
> 公理入口：[AXIOMS.md](../governance/AXIOMS.md)；决策：ADR-0041..0045。

### P8-T01 — 文档体系重构与 2.0 设计基线

- documentation-only；不改 core/specs/conformance。
- 交付：AXIOMS、白皮书 Personal 对齐章、product/architecture 扩展、ADR-0041+、台账修复。
- 切片 D01 公理/规则 → D02 设计文档 → D03 收口。

### P8-T02 — Universal Agent Adapter Contract 实现

- AKP 唯一适配；能力声明/注册/生命周期；candidate-only + 通道隔离负例。
- 对齐 A2A 发现语义，默认无公网 listener；Lane-CTR 登记 `agent-adapter-manifest`。

### P8-T03 — 首个非 Pi Agent 独立资格化

- 完整 acquisition/install/activation/rollback/uninstall；证据不继承 Pi；B09 模式泛化。

### P8-T04 — 确定性 hooks 与分级加载

- daemon-owned admission/pre-dispatch/post-effect/verification 拦截点。
- hooks 不得放松 A1–A8。

### P8-T05 — Context compaction 与自适应预算

- digest 绑定压缩产物 + 显式损失；UCR-01 可复验收益观察（非 Gate）。

### P8-T06 — 跨 episode 学习闭环

- Skill/Memory 候选 → deterministic admission；失败经验可解释可撤销；无自授权。

### P8-T07 — 独立双语 handbook 文档系统

- 独立 `personal/handbook/` 根：en 与 zh-CN 平行树，user/developer/reference/AI 四类读者入口；
  informative 派生层，不建第二事实源，动态状态只链接 `PROGRESS.md` 不复制。
- 机器模型：manifest、frontmatter JSON Schema、source-map、tracked-source coverage
  （全树分类，新文件未分类即失败）、per-page source fingerprint（内容 SHA-256 + 稳定
  symbol，存在性校验，不用行号）、source-set 记录（明确实现基线 revision，避免自引用
  HEAD）。
- 生成 reference：CLI usage、HTTP 路由、错误注册表、配置文件、env 变量、transition
  表、schema 清单、native tool 目录从实现/机器合同提取，注释文件双向防腐；生成页禁止
  手改（`--check` 字节比对）。
- 独立 `check-handbook` 检查器 + focused negative fixtures（缺源、陈旧指纹、断链、未
  映射新文件、非法状态、生成漂移、缺双语、secret-shaped、动态状态复制、History 引
  用）；`.cursor/rules/20` 同步适配、根 `llms.txt`、AGENTS 单指针、root
  `check:handbook` 脚本与命名 CI step。
- 与基线相比现有文档字节不变（最小治理记账与单指针除外，`--diff-base` 白名单证明）。

### P8-T08 — Handbook 同步强制化

- 把"提交/推送/合并前同步文档系统"升级为义务 + 机器门：docs-sync-contract §2 全档位
  handbook 联动、§5 机器门登记、§6 自查项；rule 10/20 与 AGENTS 检查点/收口协议同步。
- `docs-sync-gate`：`--staged`（pre-commit）/`--push`（pre-push）/`--range`（收口/CI 可选）
  三模式；用 source-map 路由改动路径，命中即运行 `check-handbook` + `generate --check`；
  映射源改动而手册未同步时 fail-closed，唯一逃生口 `DOCS_IMPACT_NONE=<理由>`（须记入
  commit/PR 描述）。
- 仓库内 `.githooks/`（sh + node，跨平台）+ `pnpm run hooks:install` 一次性注册 +
  `check:docs-sync` 聚合脚本；CI 既有 handbook 步骤保持无条件全量。

### P8-T09 — DeepSeek Harness candidate-only AKP adapter

- 钉住 dsh 精确 revision 与 AKP request-envelope schema digest；session fencing 与单调
  sequence；拒绝 authority-shaped / secret-shaped payload。
- TypeScript shim 只发 candidate/observation/lifecycle；JSONL/HTTP 有界 transport；
  daemon `POST /task/akp/dsh` 把 Workspace* 映射到既有 public candidate admission。
- dsh 不是 authority writer；dsh 响应不是 Task 完成。linux-002 真机与 Path A/B 计时
  只记 observation，不构成 Gate/B01/release/Profile/Agent-benefit。

### P8-T10 — Install dsh as the Personal product agent path

- `cognitive dsh configure` 钉住精确 dsh revision 并写入仅 candidate 的 adapter digest；
  `cognitive dsh launch` 是安装后的 Path B（dsh → AKP → daemon → Flash）。
- Pi 可保持 `not_configured`；doctor overall ready + system/database/secret/provider/daemon
  即可 launch。`--path a` 只用于配对测量，不是产品启动路径。
- linux-002：真实 WorkspaceRead/Search 与可丢弃 Write 走 Intent/Effect/verification/acceptance；
  同机 Path A/B n≥5。dsh 响应不是 Task 完成。不预设无损。

### P8-T11 — Provider streaming, dsh OS runtime inspect, real-task A/B

- Public Provider proxy 把 `stream:true` 以 SSE 透传到 selected Flash，禁止再经
  SSE-to-unary 桥等待完整 JSON；Pi 与 private-candidate 保持 unary。
- `GET /personal/dsh/runtime` 与 `cognitive dsh status` 观察 session/process/lifecycle。
- linux-002：raw Provider / Path A / 已安装 Path B；真实 read/search/write + 非 `pong`
  LLM 任务；n≥5。P8-T10 ~10.5 s 主因是 guest 上 tsx-from-source，不是 Provider TTFB。
  产品路径优先编译后的 `apps/cli/lib/bin.js`。dsh 响应不是 Task 完成。不预设无损。

### P8-T12 — Resource Manager common envelope

- 为 `personal/docs/product/resource-manager-design.md` 与
  `personal/docs/architecture/resource-manager-architecture.md` 落地已指定的
  `ResourceApplicationService` 词汇（list/inspect/watch/bind/unbind/enable/disable/revoke）。
- management HTTP 信封；task 通道与 generic create/install/execute/complete 失败闭合。
- list/inspect 只投影既有 authority 事实；Skill/Tool 变异接到既有 sinks。
- `cognitive resource` 是真实调用者。watch 不另开 SSE。无公开 generic Resource DTO。
- 真机实测走 `DEV-LINUX-NATIVE-01`，不是 B01 guest。

### P8-T13 — LLM Provider Control Plane

- 映射临时设计卡 PCP-T01..PCP-T07 到正式任务 `P8-T13`。官方 OpenAI / 官方 Anthropic /
  自定义 OpenAI 兼容端点；禁止第三方 Anthropic 兼容端点。
- 多命名账户；每个 agent 固定 account+provider+model binding；无回退/负载均衡/自动路由。
- API key 只进批准 Secret Store；daemon 是唯一 authority writer；不出现在 SQLite /
  普通配置 / argv / env / 服务单元 / 日志 / CLI / audit / 夹具 / 证据。
- 创建时前台发现一次；之后只在显式 refresh。发现失败保留账户/目录/binding。
- Pi 走 `/provider/v1/chat/completions`；DeepSeek harness 独立走 `/provider/v1/dsh/*`。
- 用量四类 token；`unknown` 不作 0；缺价 `cost_unavailable`。预算告警仅观察。
- 本阶段无 Web/Desktop 控制面。Cockpit/CC Switch 只作交互参考。
- 真机实测走 `DEV-LINUX-NATIVE-01`，不是 B01 guest。

### P8-T14 — Provider Control Plane 操作员使用说明

- 文档-only 跟进；不重开 `P8-T13`。双语 handbook 操作员用法对齐已交付 daemon API 与
  `cognitive` CLI（账户、密钥、trust flags、模型、binding、用量/审计投影、仅观察预算/
  告警、常见失败）。
- 本阶段仍无 Web/Desktop 控制面板。不发明未交付 flags。usage/audit query 无过滤器。
- 不修改产品代码、合同或测试来“补齐”文档。不声称 live Secret Store / Provider / Pi /
  dsh、Gate、Profile、B01 或 Web UI。

### P8-T15 — Native dsh Web UI control panel

- 产品命令 `cognitive dsh web` 启动原生 dsh 控制面板（`dsh --profile web --no-open`），
  默认 loopback `http://127.0.0.1:3080`。这不是 Personal `/ui/`（P7-T05）。
- 钉住 revision 不变，除非 web artifact 强制升 pin。安装 overlay 必须产出 `apps/web/dist`
  （`pnpm run build`，不只 `build:lib`）；缺 dist fail-closed。
- 只绑定 127.0.0.1；拒绝 `0.0.0.0`。无 TLS/auth。SSH guest 使用 `--no-open`。
- Path B 保持 dsh → AKP → daemon → Flash + SecretStore；不要把 API key 写入 dsh `.env`。
  不在 web 启动时 admit Workspace* one-shot tasks。
- `cognitive dsh status` 观察 web 进程；UI up 不是 Task 完成。
- 负例：缺 dist、非 loopback host、Path A 仍仅测量、headless `--print` 不回归。
- 真机 linux-002：`ss` 显示 `127.0.0.1:3080`，`GET /` 返回原生 SPA HTML。
- 声明上限 `hypothesis`；不提升 Gate/release/Profile/B01/EVAL/Agent-benefit。

---

## Phase 9 — 性能与结构演进（研究卡；正式状态见台账）

### P9-T01 — 异步事件底座决策门

- 以 P7-T04 stage 计时区分治理税/实现税；权威 SQLite 保持单写者。

### P9-T02 — 权威路径结构债拆分

- 拆分 oversized modules；行为不变；对照回归地板。

### P9-T03 — 存储访问与组合根优化

- 长生命周期 store；Personal 垂直逻辑下沉；stage 计时对照。

### P9-T04 — 全面性能与真实 Task campaign

- ADR-0051 注册的单一 B01 campaign；先完成 secret-free correlation、stage timing、Provider
  usage availability、transport/resource/evidence runner，再依序执行 L1--L5。
- 只记录实际获得的 B01 campaign evidence；无 streaming timestamp 时 TTFT 不可用，缺失
  Provider usage 必须为 `not_available`，L5 未达标时仍需完成 non-claim report 与 cleanup。

---

## Phase 10 — Personal 2.0 desktop Control Plane 与 MCP family（研究卡；正式状态见台账）

> 正式定义、typed dependencies 与状态以 `PERSONAL-DEVELOPMENT-PLAN.md` Phase 10
> 为准。本节只补充实施边界；Phase 10 不创建 Gate。

### P10-T01 — Personal 2.0 desktop + MCP semantic adoption

- **目的：** 以 ADR-0056/0057 固定 owner 已决定的 desktop primary entry、目标 IA、
  candidate-only global Agent Shell、vendor-specific conversation adapter/common internal
  capability projection、native Agent app coexistence 与 MCP seventh-family 语义。
- **Typed dependencies：** `implementation_requires` 为 owner-accepted 2026-08-27
  scope、ADR-0037/0043/0055 与 finalized Personal 1.0 baseline；ADR-0056/0057、
  ADR-0037 partial backlink、product/plan/trace/support/version sync 与 ADR-0055
  `Requires-backend` honesty 保持独立 `acceptance_requires`，不回写为实现前置。
- **边界：** Linux/Personal 1.0 保持 finalized six-family；ADR-0037 只对 2.0 family count
  增 backlink，不重写历史；ADR-0055 import 全部 per-source consent + daemon-only +
  SecretStore，未实现 surface 标 `Requires-backend`。
- **切片：** D01 ADR/canonical product semantics → D02 detailed product/architecture/bilingual
  alignment → D03 consistency/acceptance/closure。
- **不包含：** code、contract、schema、transition、negative、generated reference、Gate、
  release、Profile 或 support claim。

### P10-T02 — Lane-CTR contract and compatibility decision

- 决定 MCP family 与 common conversation projection 的 public/private boundary，
  identity/version compatibility、capability digest、binding、health/quarantine、P5-era
  migration 与 older-client fail-closed。
- [ADR-0058](../adr/0058-personal-2-0-mcp-conversation-private-projection.md)
  裁定两者保持 Personal-private versioned envelope，本批不改变 public machine
  contract；schema/generated bindings 因此不同批新增。

### P10-T03 — MCP family authority and product integration

- 在 P10-T02 后实现 daemon-owned server/package/connection/capability/binding/health/
  quarantine lifecycle，并把 Tool/Context/Skill candidate 接到各自 admission。
- 复用 P5-T03/P5-T04 transport/dynamic Tool 与 P8-T12 management envelope，但不得把
  既有 evidence 写成 seventh-family implementation；保持 SecretStore、Intent/Effect、
  fencing、reconcile 与 quarantine/requalification negatives。

### P10-T04 — Desktop Control Plane experience

- 以 desktop Control Plane 为 primary entry，交付 Home / Agents / Work / Library /
  Activity / Settings；Providers 与 System 进入 Settings。
- global Agent Shell 只建议、解释、导航和提案；installed-Agent conversations 经
  vendor-specific adapters 汇入 common internal projection/capability matrix，能力缺失
  诚实显示；native Agent app 仍可使用。
- credential import 只显示真实 backend capability；无 ADR-0055 implementation 时必须
  `Requires-backend`。现有 P7 Web UI/P8 Provider/dsh evidence 不自动构成 2.0 support。

### Phase 10 disposition (2026-08-27)

P10-T01/T02 remain completed historical facts. P10-T03..T18 retained their
original acceptance in the formal plan but were cancelled before implementation:
T03/T14 move to future advanced MCP/federation; T04/T12/T15 -> P11-T13;
T05 -> P11-T05/T07; T06/T08 -> P11-T03/T09; T07 -> P11-T08/T13;
T09 -> P11-T06/T07; T10/T11 -> P11-T12; T13 -> P11-T04; T16 -> future
independent adapter qualification; T17 -> P11-T15; T18 -> P11-T02.

## Phase 11 — Windows-first OPC 2.0 task cards

These cards elaborate the formal Phase 11 rows. They do not authorize
implementation. Each implementation task uses one task branch, lease, Draft
PR, focused negatives, supported CI, and the qualified environment named by its
formal acceptance.

### P11-T01 — OPC docs/ADR closure

- **Outcome:** one canonical Windows OPC product/design/architecture/plan/
  handbook graph with frozen legacy corpora and an interactive non-effect
  Canvas.
- **Boundaries:** documentation/design/prototype only; no product code,
  contracts, negatives, tests, support, Gate, or qualification.
- **Acceptance:** ADR-0059 partial supersession; P10 dispositions; P11 cards/
  trace/support/environment; bilingual handbook/source-map/fingerprints;
  generated docs unchanged/check; all static/required CI; closure.
- **Validation:** local Markdown/links/anchors/fences/terminology, Canvas TS,
  consistency, handbook, generator, docs-sync, diff/lint; required CI.

### P11-T02 — Windows host, tray, and background (hidden)

- **status:** `done` (merged PR [#292](https://github.com/agentkernel/cognitive-os/pull/292) at `main@cb66c7fb`; required CI [33358661063](https://github.com/agentkernel/cognitive-os/actions/runs/33358661063) **SUCCESS** at `19300b92`; Linux store 9/9 + HTTP 1/1 at `71c4824a`; native E2E **not-run**). Claim ceiling `hypothesis`.
- **2.0.0 表面:** 非一级 chrome；隐藏 Windows host 能力。不挡 T03。
- **依赖:** P11-T01；ADR-0052/P7-T07 fragments。**不**作为 T03 mutex。
- **垂直切片:** inspectable install → Personal Home `app/`/`data/` → daemon/tray
  → close background-or-pause → sleep/offline missed → ordered recovery。
- **Scene:** 无独立一级页；Settings/高级可暴露诊断，默认不画 native DSH UI。
- **acceptance:** 资格化宿主上的 install/ACL/SecretStore/process 负例绿；否则原生
  E2E 诚实 `not-run`。
- **不可做:** 把 GNU/WSL/Linux 写成 Windows 产品；B01-W 当日常开发机；假 background。
- **本仓 foundation:** 现有 installer/host fragments、SecretStore、daemon 生命周期。
- **禁止再造:** 第二套凭据平面、把 DSH web 当宿主壳。
- **validation environment:** `CI-WINDOWS-MSVC-01`。原生 E2E =
  `DEV-WINDOWS-NATIVE-OPC-01`（未资格化 = `Requires-environment` / `not-run`）。
  `DEV-WIN-GNU-01` 禁 Rust link。`B01-W` / `B01-DESKTOP-002` 非日常默认。
- **关闭门:** 资格化 Windows 上证明上述垂直路径；环境缺失不得编造 pass。
- **漂移检测负例:** 错误安装根、ACL 逃逸、raw secret/env/argv、重复 daemon、orphan
  DSH、restore-as-backup 声称；secret 不进日志/DOM。
- **硬门:** 适用 Phase 11 四条 + `TEST-REPORT-INCREMENTAL-01`。

### P11-T03 — Project aggregate walking skeleton (first knife)

- **2.0.0 表面:** Projects 列表/详情的权威来源；不是完整 Today 页、不是改装 `/work`。
- **依赖:** P11-T01；Task/Intent/Effect/verification。**不依赖 T02。**
- **垂直切片:** failure-first 负例 → 真 Project 聚合 → confirm-before-activate →
  Charter/Goal/Metric/Plan revision → Task/Attempt → Effect/evidence/独立 verification。
- **Scene:** 五段创建向导的权威后端；无权威则 empty/unavailable。
- **acceptance:** 真 Project 身份可查询；未确认不能 active；跨项目写失败。
- **不可做:** 完整 `/ui/` IA、改 canvas、领取 T02、假按钮。
- **本仓 foundation:** SessionGate、hash `/ui/`、Task/Intent/Effect/verification、
  SecretStore。外部：不把 Paperclip heartbeat 当权威。
- **禁止再造:** 通用 Resource 表、第二套完成定义、heartbeat 写权威。
- **validation environment:** `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01`（+ 需要时
  exact-revision `DEV-LINUX-NATIVE-01`）。`DEV-WIN-GNU-01` 仅 fmt/docs/TS。
  `B01-DESKTOP-002` 非日常默认。
- **关闭门:** 真 Project 聚合（非 Task 行冒充）在登记 CI/Linux 上通过负例与正向权威测试；
  不是完整 Today 页验收。
- **漂移检测负例:** 真 Project 而非 Task 行冒充；无权威则 empty、禁止假按钮；完成 ≠
  模型文本 / HTTP 200 / `agent_end`；未确认激活失败；跨项目写失败；secret 不进
  日志/argv/SQLite/聊天/DOM。
- **硬门:** 适用 Phase 11 四条 + `TEST-REPORT-INCREMENTAL-01`。

### P11-T04 — Role Blueprint, Assignment, and Employee

- **2.0.0 表面:** 成员先选后配；权威 id = Employee；chrome 可写 Member Runtime。
- **依赖:** P11-T03。
- **垂直切片:** 特化 Project Manager Blueprint → Assignment → Employee → 绑定兼容
  runtime；每活动 Project 一个 current manager。
- **Scene:** 项目「成员」子菜单的权威，不是 Agents 六族页改名。
- **acceptance:** employee≠runtime；Blueprint 无 Provider binding；升级 versioned +
  per-Project opt-in。
- **不可做:** Role=Agent 合并、聊天转移权威。
- **本仓 foundation:** adapter identity、现有 Agent 身份与 SessionGate。
- **禁止再造:** 第三套权威对象 id。
- **validation environment:** 同 T03 CI 默认；宿主 E2E 未资格化则 `not-run`。
- **关闭门:** Employee 身份与 runtime 可替换且不合并；one current manager 可证明。
- **漂移检测负例:** Role=Agent、Employee=process、聊天/handoff 转移权威、manager
  删除即丢历史、隐式 Blueprint 升级。
- **硬门:** 适用 Phase 11 四条。

### P11-T05 — Conversation new private version

- **2.0.0 表面:** 单 composer；聊天无 Approve。
- **依赖:** P11-T03、P11-T04；ADR-0058。
- **垂直切片:** 新 Personal-private projection version：append → archive/index →
  授权检索；禁止重解释 `conversation-projection/0.1`；不先开 Lane-CTR。
- **Scene:** 项目会话与助手会话分层参考 Codex `codex-rs/memories`（不搬执行引擎）。
- **acceptance:** 新 identifier；旧 `0.1` 客户端不得被静默 coerce。
- **不可做:** 把 Conversation 当 Task 完成；core 公共 schema 偷偷改。
- **本仓 foundation:** ADR-0058 信封、现有 conversation-projection 边界。
- **禁止再造:** 重解释 `0.1`、浏览器写 SQLite 权威。
- **validation environment:** 投影/检索负例 → CI；宿主 E2E 未资格化则 `not-run`。
- **关闭门:** 新 private version 存在且 `0.1` 语义未改；文档若仅契约则显式声明。
- **漂移检测负例:** 跨项目/员工读、secret-shape、无界 resume、全文档案注入、
  Conversation-as-completion、重解释 `0.1`。
- **硬门:** 适用 Phase 11 四条。

### P11-T06 — Hidden Pi Personal Assistant

- **2.0.0 表面:** 右栏助手；Pi 不进 Installed Agents chrome。
- **依赖:** P11-T03、P11-T05（用户建造顺序未点名，但是已拍板隐藏能力，不可删）。
- **垂直切片:** 当前对象/问题 → 有界 Context → exact Pi → 解释或变更 candidate →
  daemon preview。
- **Scene:** 右栏；candidate-only。
- **acceptance:** 无 authority/Secret/archive/Memory 写；default-deny tools。
- **不可做:** Pi Linux 资格转移 Windows；ambient shell。
- **本仓 foundation:** exact Pi client/Shell。
- **禁止再造:** 把 Pi 当 Member 执行引擎。
- **validation environment:** required CI；Linux Pi 不转移；宿主 Pi 路由 `not-run`
  until qualified。
- **关闭门:** 隐藏引擎只产 candidate；preview 走 daemon。
- **漂移检测负例:** 直连 SecretStore/DB、preview bypass、伪造 source、完成自报。
- **硬门:** 适用 Phase 11 四条。

### P11-T07 — Hidden hosted DSH engine

- **2.0.0 表面:** 不画原生 DSH UI / engine store / 可见 Installed Agents。
- **依赖:** P11-T03、P11-T04、P11-T12（诚实 usage）。**不**以 T02 为 mutex。
- **垂直切片:** exact audited artifact → isolated child/stdio broker → daemon
  Provider proxy → bounded Task candidate → health/update/rollback。
- **Scene:** 成员运行时的隐藏引擎；诊断可用独立 `cognitive dsh web`（非产品 `/ui/`）。
- **acceptance:** Personal 拥有 Conversation/Memory/Task/completion。
- **不可做:** 搬 `apps/web`、插件商店、harness 聊天 Approve、in-process loop 当权威。
- **本仓 foundation:** dsh Path B、AKP adapter、SecretStore、Provider CP。
  可借鉴（不可搬权威）：harness `packages/sandbox/sandbox-policy`、
  `sandbox-local`、`sandbox-windows-acl`、`packages/subprocess/*`、
  `packages/session/session-persistence-jsonl`。
- **禁止再造:** heartbeat 写权威、Codex/Claude 当可选 Member 引擎。
- **validation environment:** required CI。Windows sandbox E2E =
  `DEV-WINDOWS-NATIVE-OPC-01` / `not-run`。Linux Path B 不能冒充 Windows 托管资格。
- **关闭门:** 隐藏托管引擎可安装/隔离/代理/回滚；默认 chrome 无商店。
- **漂移检测负例:** digest 不匹配、env secret、native MCP/base tool/HMR、orphan、
  unknown=success、把 DSH web 嵌进 `/ui/`。
- **硬门:** 适用 Phase 11 四条。

### P11-T08 — Routine, Trigger, missed run

- **status:** done (merged PR [#290](https://github.com/agentkernel/cognitive-os/pull/290) at `main@bda740f6`; required CI [33340492220](https://github.com/agentkernel/cognitive-os/actions/runs/33340492220) **SUCCESS** at `7182a4fb`; Linux store 7/7 + HTTP 1/1 at `98bd61de`). Claim ceiling `hypothesis`. Unique next = `P11-T13` is **done** (PR [#291](https://github.com/agentkernel/cognitive-os/pull/291)); do not auto-claim T02.
- **2.0.0 表面:** 项目运行/例程；不是 Inbox 一级。
- **依赖:** P11-T03。**不挡 T09。**
- **垂直切片:** Routine revision + Trigger → no-overlap/queue-latest → missed ledger。
- **Scene:** 项目「运行」子菜单。
- **acceptance:** 无第二套 Temporal 调度器；checkpoint 非权威。
- **不可做:** 把 HITL 做成依赖本任务的 Inbox 一级。
- **本仓 foundation:** scheduler/Effect/recovery。
- **禁止再造:** 第二 scheduler。
- **validation environment:** required CI；clock/sleep E2E 未资格化则 `not-run`。
- **关闭门:** no-overlap 与可见 missed 可证明。
- **漂移检测负例:** overlap、静默丢、checkpoint 当完成、consequential auto-resume。
- **硬门:** 适用 Phase 11 四条。

### P11-T09 — HITL on canvas (not Inbox)

- **2.0.0 表面:** HITL 只在项目中心画布；Today 深链进入；聊天无 Approve。
- **依赖:** P11-T03。**不依赖 T08。**
- **垂直切片:** 结构化 daemon preview → edit/narrow/reject/confirm → Intent/Effect
  reconcile → receipt。聚合在画布，不是一级 Inbox queue。
- **Scene:** 中心画布 HITL + Today 深链；禁止独立 `#/hitl/:approvalId` 一级。
- **acceptance:** Conversation 只宣布，不批准。
- **不可做:** 聊天 Approve、假可点按钮、Inbox 一级页。
- **本仓 foundation:** preview/Effect/alert/recovery。
- **禁止再造:** harness approval 当产品 HITL。
- **validation environment:** required CI；宿主 UI E2E 未资格化则 `not-run`。
- **关闭门:** 画布 HITL 可确认/拒绝并留下 receipt；聊天路径不能完成批准。
- **漂移检测负例:** 聊天 Approve、stale/跨通道 preview、blind retry、unknown=success、
  无权威却渲染 active 控件。
- **硬门:** 适用 Phase 11 四条。

### P11-T10 — Knowledge and Markdown Vault

- **2.0.0 表面:** Knowledge 一级。
- **依赖:** P11-T03、P11-T05。
- **垂直切片:** import → rights/provenance → parse/index → conflict；index 可重建。
- **Scene:** Knowledge；项目 Vault 与 Owner 共享源分离。
- **acceptance:** file 不是 Project 权威。
- **不可做:** 捆绑 Obsidian；越权检索当功能。
- **本仓 foundation:** Memory/Skill/Context/Artifact。
- **禁止再造:** 文件当 CAS 权威。
- **validation environment:** 投影/检索负例 → CI；宿主 filesystem E2E `not-run`
  until qualified。
- **关闭门:** import/index/conflict 与越权负例在 CI 绿；宿主 E2E 诚实 `not-run` 若缺环境。
- **漂移检测负例:** 遍历、secret ingestion、检索越权、secret-shape、last-write-wins
  无冲突、file-as-authority。
- **硬门:** 适用 Phase 11 四条。

### P11-T11 — Memory admission, privacy, forget

- **2.0.0 表面:** 员工私有 Memory，非一级商店。
- **依赖:** P11-T05、P11-T10。
- **垂直切片:** candidate → admission → view/correct/forget → 非复活。
- **Scene:** 可检查 Memory；参考 Codex memories 分层，不搬写权威。
- **acceptance:** Letta/Mem0 不得直写。
- **不可做:** 默认 telemetry、env secret。
- **本仓 foundation:** 现有 Memory admission/forget。
- **禁止再造:** Agent 自 admission。
- **validation environment:** required CI；privacy E2E `not-run` until qualified。
- **关闭门:** admission 与 forget 后 index 不复活可证明。
- **漂移检测负例:** 跨 scope、secret/PII、poisoning、直写、tombstone 复活。
- **硬门:** 适用 Phase 11 四条。
- **status:** done (merged PR [#289](https://github.com/agentkernel/cognitive-os/pull/289) at `main@b5084e06`; required CI [33327844743](https://github.com/agentkernel/cognitive-os/actions/runs/33327844743) **SUCCESS** at `60844f51`; Linux store 4/4 + HTTP 2/2 at `f1dca3e0`). Claim ceiling `hypothesis`. T08 claimed in the same session.

### P11-T12 — Provider honest usage (no member budget chrome)

- **2.0.0 表面:** Settings 用量诚实；成员级预算 **不是** 当前 chrome（2.1 / Deferred）。
- **依赖:** P11-T03、P11-T04、Provider CP。**不依赖 T07。**
- **垂直切片:** account/quota 分离；binding global→Project→employee→Task；actual
  usage；unknown≠0。
- **Scene:** Settings 高级/用量；无成员硬停一级控件。
- **acceptance:** raw SecretStore 永不进 UI/DSH/Pi。
- **不可做:** unknown=0、静默 fallback、把成员预算当 2.0.0 关闭门。
- **本仓 foundation:** Provider Control Plane。
- **禁止再造:** agent 自停当权威预算。
- **validation environment:** required CI；宿主 SecretStore 路由 `not-run` until
  qualified。
- **关闭门:** 未知费用不显示为 0；binding 可解释。成员预算硬停不是本卡 done。
- **漂移检测负例:** unknown=0、secret 进 env/log/DB、静默 rebind、依赖 T07 才开始
  诚实 usage。
- **硬门:** 适用 Phase 11 四条。

### P11-T13 — OPC Control Plane IA closure

- **status:** done (merged PR [#291](https://github.com/agentkernel/cognitive-os/pull/291) at `main@46eebeca`; required CI [33347348125](https://github.com/agentkernel/cognitive-os/actions/runs/33347348125) **SUCCESS** at `e4f00179`; Dual Track L1; host dump-dom L1 **pass**; CDP Settings **partial**; NVDA/200%/host-theme hung **not-run**). Claim ceiling `hypothesis`. Unique next = do not auto-claim T02; do not unpark T14/T15.
- **2.0.0 表面:** Today / Projects / Knowledge + 底栏 Settings + 右栏助手。
  Team/Inbox **不是一级**。
- **依赖:** P11-T03 + Visual UI 规格。Dual Track `clients/pc/web` **仅**在 T03
  投影/HTTP 稳定后另租。完整 IA 收口在本任务；禁止先画完整假壳。
- **垂直切片:** daemon-served `/ui/` 替换六族 IA；无权威 empty；真实 caller 才可操作。
- **Scene:** 已定档 chrome；state-lab = Settings 高级默认隐藏。
- **acceptance:** Requires-backend honesty；Vite 不是产品源。NVDA/200%/host-theme
  contrast **挂单 `not-run`**。
- **不可做:** 无 Project 权威时冒充 T13 完成；改 IA；phase 4 重生 canvas。
- **本仓 foundation:** hash `/ui/`、SessionGate、P7-T05 inventory（不是 OPC 合同）。
- **禁止再造:** 假六族改名、假按钮。
- **validation environment:** TS/组件测试 + 合同 mock。产品源 daemon `/ui/`。
  NVDA/200%/contrast 挂单。`DEV-WINDOWS-NATIVE-OPC-01` 不合格则原生 E2E `not-run`。
- **关闭门:** 一级导航是 Today/Projects/Knowledge+Settings；无权威无假按钮。完整
  `/ui/` 不得提前冒充已验收。
- **漂移检测负例:** Vite 当产品、Team/Inbox 一级、假按钮、secret 进 DOM、Agent
  自验证、无视觉规格就整页编码。
- **硬门:** 适用 Phase 11 四条。

### P11-T14 — X/Twitter connector walking skeleton

- **status:** `done` (merged PR [#293](https://github.com/agentkernel/cognitive-os/pull/293) at `main@bc274bfd`; required CI [33364486699](https://github.com/agentkernel/cognitive-os/actions/runs/33364486699) **SUCCESS** at `53a35adf`; live X API **not-run**).
- **2.0.0 表面:** 隐藏能力，不是 chrome，**不是** P0 hero / default demo Project。
- **依赖:** P11-T03 Project、P11-T09 HITL confirm、P11-T12 honest unknown≠0。
- **垂直切片:** SecretStore-only bind → rights-safe original → digest preview →
  HITL confirm → persist-before-dispatch → readback or honest unknown。
- **Scene:** 非 v9 M-X chrome；无默认 X 项目。
- **acceptance:** Linux store/HTTP 负例 + required CI。live X API 可 `not-run`。
- **不可做:** fingerprint/CAPTCHA/anti-abuse 规避；raw secret；无 HITL 即 publish；
  receipt-as-completion；unknown=0；scraped content；把 Linux CI 写成平台资格。
- **本仓 foundation:** SecretStore、HITL preview/confirm、Provider unknown≠0。
- **禁止再造:** 第二套 credential plane、聊天 Approve、业务结果承诺。
- **validation environment:** `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01`；native
  store/HTTP = `DEV-LINUX-NATIVE-01`。live X = `Requires-environment` / `not-run`。
- **关闭门:** daemon-owned walking skeleton 如上；live X E2E 可诚实 `not-run`。
- **漂移检测负例:** evasion、raw token、无 HITL publish、receipt-as-completion、
  unknown metrics=0、scraped、P0 hero、secret 进日志/DOM。
- **硬门:** 适用 Phase 11 四条。

### P11-T15 — Fixed-denominator Windows OPC acceptance (unparked; Phase 13 验收出口)

- **2.0.0 表面:** unparked；N=15 固定分母预注册草案见下（`P13-T01` 登记，领取时
  冻结为 preregistration）。**不是** Phase 12 prototype completeness mutex。不自动
  release。
- **依赖:** `acceptance_requires` = **`P13-T02..T13` done** + `P13-T13` 已把
  `DEV-WINDOWS-NATIVE-OPC-01` 资格化。可与 P13 非重叠 lease 并行准备 preregistration
  文档，但任何 cell 执行前必须满足上述前置。
- **垂直切片:** 冻结 15 个场景 → 一 exact qualified Windows revision → 全部执行并
  retain every started cell → independent review → non-claim report。
- **Scene（N=15 预注册草案；领取时逐条冻结 oracle / 记录格式；分母不可替换）:**
  1. 干净 Windows 宿主 inspectable install → Personal Home `app/`/`data/` → daemon/tray 就位（P11-T02 / P13-T13）。
  2. 首次打开 `/ui/`：空 Home 只创建、右栏隐藏、无假按钮（P12-T02）。
  3. Settings Model Connections 完成一个真实 Provider 连接，raw secret 只经 SecretStore（P13-T08）。
  4. 五段创建向导 ①→⑤：助手真实研究/提案（P13-T03）、confirm-before-activate、G1/G2、成员顺序就位、④ 逐环测试、⑤ 联合验收进入 Today（P12-T02 / P11-T03 / P11-T04）。
  5. Project 四子菜单：详情只读流程轴；成员先选后配八标签；确认加入 = Intent（P12-T03 / P12-T04）。
  6. 一个 Member 经隐藏托管 DSH 真实执行一个 Task Attempt 并产出产物（P13-T02）。
  7. 独立验证 → `outputs` 可打开产物 → 末环验收（P13-T04）。
  8. Routine 按 ③ 声明武装，手动触发经 Intent，`runs` 显示 occurrence + Attempt 历史（P13-T05）。
  9. 关窗选择 background-or-pause → 睡眠/离线 → 唤醒后 missed/resume 事实可见且不重叠（P11-T02 / P13-T05 / P13-T13）。
  10. HITL：聊天只宣布，Today 深链进中心画布，批准 / 改窄 / 拒绝 / 停；stale 与 unknown 不能批（P12-T05 / P12-T06 / P11-T09）。
  11. external-send 发布包：画布 AUTONOMY packet 预览 → 确认 → receipt；planned ≠ published（P13-T04）。
  12. 项目群聊 `@manager` / `@member` 路由；成员发言规则；`@` 只进草稿；无 Approve（P13-T06）。
  13. Knowledge ingest → Why this fragment → 冲突/来源/权利可见；Memory inspect / correct / forget 后不复活（P12-T07 / P13-T07）。
  14. Skill/MCP 获取：评审 → exact Owner 画布 preview → 版本锁定 → 按范围 grant → 回滚（P13-T10）。
  15. 复制为 inactive 副本 → 归档停触发 → 删除影响预览 + 二次确认 → local restore point / 导出排除 secret（P13-T09）。
  以上每格记 `pass` / `fail` / `partial` / `not-run`（环境缺失时全部 `not-run`，不是
  产品失败）；zero critical A1–A8；a11y / 视觉资格化由 `P13-T12` 单独记账，不占本分母。
- **acceptance:** 15 格全部执行且保留、zero critical、independent review、non-claim
  report；signing / B01-W / 2.1 仍独立。T15 done ≠ release，≠ Gate，≠ Profile。
- **不可做:** 用 ordinary CI / Linux / WSL / GNU promotion；把本卡当 P12 mutex；替换
  或缩小分母；用 Linux 单元顶替 Windows 场景。
- **本仓 foundation:** Operating Model 固定分母实践；B01 successor `002` 的
  preregistration / evidence collector / independent verifier 模式。
- **禁止再造:** 把本卡当当前实现 mutex；把 `not-run` 写成 pass；第二套验收分母。
- **validation environment:** 仅已资格化 `DEV-WINDOWS-NATIVE-OPC-01`（P13-T13）。
  未资格化 = 整卡 `blocked` / `not-run`。`CI-WINDOWS-MSVC-01` 只证明编译。
- **关闭门:** 15 格在同一 exact qualified Windows revision 上全部执行并记账；
  environment 缺失不得编造 pass。
- **漂移检测负例:** 把 `not-run` 写成 pass；A7 提升；把 T15 写成 prototype done；
  替换分母；用 B01-DESKTOP-002 / Linux 冒充 Windows。
- **硬门:** 适用 Phase 11 四条 + `TEST-REPORT-INCREMENTAL-01`。

### P12-T01 — Phase 12 docs/plan registration (documentation-only)

- **Outcome:** Phase 12 + `P12-T02..T09` registered inside the existing
  `PERSONAL-DEVELOPMENT-PLAN.md` (three columns, negatives, Slices,
  `implementation_requires`); plan.md/trace/PROGRESS/handbook synced.
  Frozen-prototype **functional completeness** on daemon `/ui/`; not
  pixel-replica; not 2.1; not T15.
- **Boundaries:** documentation/plan only; no product code, contracts,
  negatives, tests, support, Gate, or qualification.
- **Acceptance:** Phase 12 cards complete; `check:consistency` /
  handbook / docs-sync / required CI; documentation-only.
- **Validation:** local Markdown/link/anchor/fence/terminology,
  `check:consistency`, `check:handbook`, generator `--check`, fingerprint,
  docs-sync, diff/lint; required Ubuntu/Windows CI.
- **不可做:** 新开平行计划/PRD；把 T15 写成 prototype mutex；把 T13 L1
  写成完整 `/ui/`；canvas/Vite 当产品源。
- **硬门:** documentation-only 出口写明；不触碰 code/contracts/tests。

### P12-T02 — Five-step create wizard + empty home only-create

- **status:** `done` (merged PR [#295](https://github.com/agentkernel/cognitive-os/pull/295) at `main@23646a84`; required CI [33373453242](https://github.com/agentkernel/cognitive-os/actions/runs/33373453242) **SUCCESS** at `69f5edb0`; Dual Track TS web **344/344**). Claim ceiling `hypothesis`. Unique next = `P12-T03` in-progress.

- **2.0.0 表面:** Scenes `empty-home`、`create-init`…`create-joint`。空 Home
  只创建、藏右栏。
- **依赖:** P12-T01；P11-T03/T04/T12 权威；P11-T13 Dual Track L1。
- **垂直切片:** 空 Home 诚实只创建 → 五段向导接到既有 confirm-before-activate
  preview → 无权威不写 Project。
- **acceptance:** 0 假 Create/Activate；confirm-before-activate 走既有 preview。
- **不可做:** 假 Create/Activate；向导绕过 preview 写权威；空 Home 画假决策包；
  Vite 当产品源；secret 进 DOM。
- **validation environment:** Dual Track TS（`DEV-WIN-GNU-01` 允许面）+
  `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01`。产品源 = daemon `/ui/`。NVDA/200%/
  host-theme **挂单 not-run**。`DEV-WINDOWS-NATIVE-OPC-01` 不合格则原生 UI
  E2E `not-run`。`DEV-WIN-GNU-01` 禁 Rust link。
- **关闭门:** 空 Home 只创建、藏右栏；五段向导接到既有 preview；无权威不写 Project。
- **漂移检测负例:** 假 Create/Activate；preview bypass；空 Home 假决策包；Vite
  当产品源；secret 进 DOM。
- **硬门:** 适用 Phase 12 四条 + Dual Track。

### P12-T03 — Project four submenus

- **status:** `done` merged PR [#296](https://github.com/agentkernel/cognitive-os/pull/296) at `main@1e736aae`; required CI [33378377579](https://github.com/agentkernel/cognitive-os/actions/runs/33378377579) **SUCCESS** at `43b3f092`. Dual Track TS **358/358**. Not T04 eight-tabs. Not T06 Confirm. Claim ceiling `hypothesis`.

- **2.0.0 表面:** `projects` / `project-detail` / `members` / `runs` / `outputs`。
- **依赖:** P12-T01；P11-T03/T04/T08 权威；P11-T13 L1。**不依赖 T02 mutex。**
- **垂直切片:** Projects 列表 → 四子菜单走 daemon Project 聚合 → 无权威诚实 empty。
- **acceptance:** 非 Task 行改名；无权威诚实 empty。
- **不可做:** `#/work` 改名冒充 Projects；无权威却渲染可点按钮；Team/Inbox 一级。
- **validation environment:** 同 T02 Dual Track + required CI。原生 UI E2E =
  `DEV-WINDOWS-NATIVE-OPC-01` / `not-run`。
- **关闭门:** 四子菜单走真实 Project 聚合。
- **漂移检测负例:** Task 行冒充；无权威可点按钮；Team/Inbox 一级。
- **硬门:** 适用 Phase 12 四条 + Dual Track。

### P12-T04 — Select-then-configure + eight tabs + add member

- **status:** `done` merged PR [#297](https://github.com/agentkernel/cognitive-os/pull/297) at `main@8c413648`; required CI [33383681338](https://github.com/agentkernel/cognitive-os/actions/runs/33383681338) **SUCCESS**. Dual Track TS **367/367**. Not T05 packets. Not T06 Confirm. Claim ceiling `hypothesis`.

- **2.0.0 表面:** `add-member`、`member-config`（八标签：duty/input/output/
  skills/tools/prompt/loop/perms）。
- **依赖:** P12-T03；P11-T04 Employee 权威。
- **垂直切片:** 先选后看 → 八标签 → 确认加入 = Intent；拒绝 = 未加入。
- **acceptance:** 无 Install 商店；成员级预算非 chrome。
- **不可做:** 未选即配置；Install 商店；成员级预算 chrome；Role=Agent 合并。
- **validation environment:** 同 T02 Dual Track + required CI。
- **关闭门:** 先选后看；确认加入 = Intent。
- **漂移检测负例:** 未选即配置；Install 商店；成员预算 chrome。
- **硬门:** 适用 Phase 12 四条 + Dual Track。

### P12-T05 — Today decision packets

- **status:** `done` merged PR [#298](https://github.com/agentkernel/cognitive-os/pull/298) at `main@bfc9aad6`; required CI [33391494827](https://github.com/agentkernel/cognitive-os/actions/runs/33391494827) **SUCCESS** at `c576e2f5`. Dual Track TS **373/373**. Creating-only = continue-create; live = pending-previews packets deep-linked to `/projects/:id?preview=`. No KPI wall. Chat has no Approve. Not T06 Confirm. Claim ceiling `hypothesis`.
- **2.0.0 表面:** `today-incomplete`、`today`。T13 明确未验收 packet canvas。
- **依赖:** P12-T01；P11-T03/T09/T13。
- **垂直切片:** 未验收只「继续创建」→ 已上线才日常决策包 → 拍板深链 HITL 画布。
- **acceptance:** 无 KPI 墙；无权威假包。
- **不可做:** 把 T13 empty chrome 写成 packet 已验收；KPI 墙；无权威假包。
- **validation environment:** 同 T02 Dual Track + required CI。
- **关闭门:** 未验收只继续创建；已上线才日常包；拍板深链 HITL。
- **漂移检测负例:** T13 冒充 packet 已验收；KPI 墙；无权威假包。
- **硬门:** 适用 Phase 12 四条 + Dual Track。

### P12-T06 — HITL canvas Confirm

- **status:** `done` merged PR [#299](https://github.com/agentkernel/cognitive-os/pull/299) at `main@a5265b22`; required CI [33396112669](https://github.com/agentkernel/cognitive-os/actions/runs/33396112669) **SUCCESS** at `89f85f16`. Dual Track TS **381/381**. Digest-bound Confirm/Narrow/Reject; Stop honest; chat has no Approve. Not T07 ingest. Claim ceiling `hypothesis`.
- **2.0.0 表面:** 画布批准 / 改窄 / 拒绝 / 停。聊天无 Approve。
- **依赖:** P12-T03；P11-T09 preview 权威。**不依赖 T08。**
- **垂直切片:** 画布 Confirm → persist-before-dispatch → stale/unknown 不能批。
- **acceptance:** 聊天无 Approve；独立 `#/hitl` 不是一级。
- **不可做:** 聊天 Approve；stale 仍批；unknown=success；无权威可点 Confirm。
- **validation environment:** 同 T02 Dual Track + required CI。confirm 走既有
  management HTTP。
- **关闭门:** 画布四动作；聊天无 Approve；stale/unknown 不能批。
- **漂移检测负例:** 聊天 Approve；独立 `#/hitl` 一级；stale 仍批；unknown=success。
- **硬门:** 适用 Phase 12 四条 + Dual Track。

### P12-T07 — Knowledge ingest UI

- **status:** `done` (merged PR [#300](https://github.com/agentkernel/cognitive-os/pull/300) at `main@081c40d0`). Required CI [33401268090](https://github.com/agentkernel/cognitive-os/actions/runs/33401268090) **SUCCESS** at `fefd6872`. Dual Track TS **391/391**. Owner-paste `vault.import` + Why this fragment from `vault.index`. Files ≠ Project authority. Import failure keeps original. Claim ceiling `hypothesis`.
- **2.0.0 表面:** ingest / Why this fragment。T13 现为只读 Knowledge。
- **依赖:** P12-T01；P11-T10 Vault 权威；P11-T13。
- **垂直切片:** ingest UI → Why this fragment 走 Vault 权威 → 导入失败保留原件。
- **acceptance:** files ≠ Project 权威；Obsidian 不进产品。
- **不可做:** 只读页假装 ingest；file-as-authority；secret ingestion。
- **validation environment:** 同 T02 Dual Track + required CI。宿主 FS E2E
  未资格化则 `not-run`。
- **关闭门:** ingest / Why this fragment 走 Vault；导入失败保留原件。
- **漂移检测负例:** 只读假装 ingest；file-as-authority；secret ingestion；捆绑 Obsidian。
- **硬门:** 适用 Phase 12 四条 + Dual Track。

### P12-T08 — Settings connections + don't-ask-again + CloseBackground

- **status:** `done` (merged PR [#301](https://github.com/agentkernel/cognitive-os/pull/301) at `main@4afc28b9`). Required CI [33418686755](https://github.com/agentkernel/cognitive-os/actions/runs/33418686755) **SUCCESS** at `21036106`. Dual Track TS web **405/405**.

- **2.0.0 表面:** 连接表 + 「本周不再问」可收回 + CloseBackgroundDialog。
  T12 list-only；T02 close-background 权威已存在。
- **依赖:** P12-T01；P11-T02/T12/T13。
- **垂直切片:** 诚实连接表 → 本周不再问可收回（非永久）→ CloseBackground 走 T02。
- **acceptance:** unknown≠0；native E2E 可 `not-run`。
- **不可做:** 假连接表；永久 Don't ask；unknown=0；raw secret 进 DOM/env。
- **validation environment:** Dual Track TS + required CI。native close/host
  E2E = `DEV-WINDOWS-NATIVE-OPC-01` / `not-run`。
- **关闭门:** 连接表诚实；本周不再问可收回；CloseBackground 走 T02 权威。
- **漂移检测负例:** 假连接表；永久 Don't ask；unknown=0；把 native not-run 写成 pass。
- **硬门:** 适用 Phase 12 四条 + Dual Track。

### P12-T09 — Right-rail edit → confirm → write canvas (no Approve)

- **status:** `done`. Merged PR [#302](https://github.com/agentkernel/cognitive-os/pull/302) at `main@3a563e7c`. Required CI [33432849969](https://github.com/agentkernel/cognitive-os/actions/runs/33432849969) **SUCCESS** at `f8581747`. Dual Track TS web **417/417**. Unique next = P12 Remaining = 0; wait for a fresh owner delivery instruction; do not auto-claim `P11-T15`.
- **2.0.0 表面:** 右栏「编辑 → 确认 → 写画布」无 Approve。empty-home 可藏聊天。
- **依赖:** P12-T01；P11-T05/T06。
- **垂直切片:** candidate 编辑 → 确认 → 写画布；无 authority/Secret 写。
- **acceptance:** candidate-only；Pi Linux 资格不转移 Windows。
- **不可做:** 聊天 Approve；助手直写权威/Secret/archive；empty-home 仍画假聊天。
- **validation environment:** 同 T02 Dual Track + required CI。
- **关闭门:** 编辑→确认→写画布且无 Approve；empty-home 可藏聊天。
- **漂移检测负例:** 聊天 Approve；助手直写权威；empty-home 假聊天；preview bypass。
- **硬门:** 适用 Phase 12 四条 + Dual Track。

## Phase 13 — Personal 2.0.0 completion task cards（walking skeleton → 原型程度 + 设计目标）

These cards elaborate the formal Phase 13 rows. They do not authorize
implementation by themselves. Each implementation task uses one task branch,
lease, Draft PR, focused negatives, supported CI, and the environment named by
its formal acceptance. Phase 13 is **not** release, **not** 2.1, **not** canvas
pixel-replica. Dual Track honesty continues: no authority → empty /
`Requires-backend`; zero fake Create / Activate / Approve / Connect / Install /
Publish. Product origin is daemon `/ui/`. Claim ceiling `hypothesis`.

差距来源（2026-09-02 对 `main@67ad05c0` 核对；写在卡上以便后续窗口不重审）：
`dsh.hosted.start` 只是 start 骨架、无完整 stdio broker
（[T07 closure](../checkpoints/2026-08-30-personal-p11-t07-dsh-closure.md)）；
`assistant.rs` `run_turn` 不调用 Pi，只把客户端 payload 注册为候选；
`ProjectRunsPage` / `ProjectOutputsPage` 只读 PlanRevision axis /
`output_contract`；`SettingsPage` 连接空态指路 `/providers`；`KnowledgePage`
Memory 列表只读（"Forget/remember stay on management HTTP"）；
`clients/docs/design/opc-2.0/` 无 Visual UI 规格文档；
`DEV-WINDOWS-NATIVE-OPC-01` not provisioned。

### P13-T01 — Phase 13 docs/plan registration + T15 N=15 预注册草案 (documentation-only)

- **status:** `done`. Merged PR [#305](https://github.com/agentkernel/cognitive-os/pull/305) at `main@aac6804f`. Required CI [33620332959](https://github.com/agentkernel/cognitive-os/actions/runs/33620332959) **SUCCESS** at `9724f67f`. Unique next = claim `P13-T02` and/or `P13-T03`; `P13-T12/D01` may run in parallel; do not claim `P11-T15`.
- **Outcome:** Phase 13 + `P13-T02..T13` registered inside the existing
  `PERSONAL-DEVELOPMENT-PLAN.md`（三栏、负例、Slices、`implementation_requires`）；
  `P11-T15` card elaborated with the N=15 preregistration draft and its
  `acceptance_requires` pointed at Phase 13；plan.md / trace / environments /
  PROGRESS / dev-prep index / handbook synced。
- **Boundaries:** documentation/plan only; no product code, contracts,
  negatives, tests, support, Gate, or qualification. Visual spec gap is
  registered as `P13-T12/D01`, not produced here.
- **Acceptance:** Phase 13 cards complete; T15 draft present; `check:consistency`
  / handbook / docs-sync / required CI; documentation-only.
- **Validation:** local Markdown/link/anchor/fence/terminology,
  `check:consistency`, `check:handbook`, generator `--check`, fingerprint,
  docs-sync, diff/lint; required Ubuntu/Windows CI.
- **不可做:** 新开平行计划/PRD；把 P13 写成 release；把 walking skeleton 写成
  产品级；把 T15 分母留空；canvas/Vite 当产品源。
- **硬门:** documentation-only 出口写明；不触碰 code/contracts/tests。

### P13-T02 — Hosted DSH real Attempt loop（完整 stdio broker + health/update/rollback）

- **status:** done (2026-09-03; merged PR [#310](https://github.com/agentkernel/cognitive-os/pull/310) at `main@d8a002ea`; lease closed; closure head required CI [33680357538](https://github.com/agentkernel/cognitive-os/actions/runs/33680357538) **SUCCESS** at `4c62bf9a`; implementation required CI [33676373077](https://github.com/agentkernel/cognitive-os/actions/runs/33676373077) **SUCCESS** at `f82bd437`; `DEV-LINUX-NATIVE-01` at `f82bd437`: store 9/9, runtime broker 7/7 with real `node` children, kernel-server 2/2, live daemon E2E product child → pinned dsh `528c682e` compiled-lib → daemon proxy → honest `failed` terminal, daemon SIGKILL → `unknown-outcome`, 0 secret leaks; [report](../checkpoints/2026-09-03-personal-p13-t02-hosted-dsh-attempt-report.md), [closure](../checkpoints/2026-09-03-personal-p13-t02-hosted-dsh-attempt-closure.md)). Claim ceiling `hypothesis`. Windows sandbox / ACL / supply-chain cells `not-run` until P13-T13; live Provider `done` leg `not-run` (no new SecretStore entry); child orphan window after a daemon *crash* is bounded by the child's own budget (recorded limitation, not containment). Implemented shape: v36 `p13_hosted_dsh_*` ledger (no `success` terminal; `completion_claimed` CHECK 0), `cognitive-runtime::hosted_dsh_broker`, kernel-server `dsh.hosted.attempt.run/list/detail` + `dsh.hosted.artifact.check/facts`, product child `personal/packages/dsh-akp-adapter/scripts/hosted-attempt-child.mjs`. Gap found for T05/T06: no product HTTP/CLI path applies a PlanRevision (roster needs one) — the live E2E seeded it as a fixture.
- **2.0.0 表面:** 无新一级 chrome；成员"运行"真实发生。引擎身份只在高级诊断
  （P13-T08）。不是 Installed Agent、不是原生 DSH UI、不是 engine store。
- **依赖:** P11-T07 骨架（v31 `p11_hosted_dsh_child`、`runtime_binding_ref`、
  `dsh.hosted.start` / `observe-exit`）；P11-T03/T04/T12；P2-T03/T06/T07
  Effect/WIA/verification。**不依赖 P13-T03。**
- **垂直切片:** D01：有界 Context payload → exact artifact child 真实 spawn
  （isolated；完整 stdio broker：request/response/observation 帧、超时、bounded
  redacted output）→ candidates/observations 流 → 终态观察写 Attempt；
  failure-first：process death ≠ completion、unknown ≠ success、secret 不进
  env/argv、child 直连 Provider 被拒、native MCP/base tool/HMR/home patch 被拒。
  D02：artifact health/update/rollback 事实 + 一个已就位 Member 对一个 Task 的
  完整 Attempt 经 Intent/Effect persist-before-dispatch 落 durable 终态；Attempt
  历史可被 `runs`（P13-T05）读取。
- **Scene:** `project-runs` 的数据源；`hitl` 的 external-send 由 T09 preview 承接。
- **acceptance:** 引擎无权威、无 secret、无完成权（14 §7.3 契约）；Personal 拥有
  Conversation/Memory/Task/completion；Attempt 终态只由 daemon 观察写入。
- **不可做:** heartbeat 写权威；in-process loop 当权威；把 Linux Path B 写成
  Windows 资格；Pi 当 Member 引擎；搬 harness `apps/web`。
- **本仓 foundation:** P11-T07 child identity + Provider proxy；P2-T06 process
  supervisor seam（PID ownership / timeout / orphan containment）；P2-T03 WIA。
  可借形状（不搬权威）：harness `packages/sandbox/*`、`packages/subprocess/*`、
  `session-persistence-jsonl`。
- **禁止再造:** 第二套 supervisor；第二套 Provider 凭据平面。
- **validation environment:** `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` + 已 push
  exact-revision `DEV-LINUX-NATIVE-01`（真实 child spawn）。`DEV-WIN-GNU-01` 仅
  fmt/docs/TS（`HOSTED_DSH_WIN_GNU_FENCE`）。Windows sandbox/ACL/supply-chain E2E =
  `DEV-WINDOWS-NATIVE-OPC-01` / `not-run` 直到 P13-T13。
- **关闭门:** 一个已就位 Member 对一个 Task 的 Attempt 经隐藏托管 DSH 真实 spawn、
  有界 Context、完整 stdio broker、candidates/observations 流、persist-before-dispatch、
  终态观察写 Attempt；health/update/rollback 事实可查。
- **漂移检测负例:** heartbeat 写权威；child 直连 Provider；orphan；process exit =
  complete；unknown output = success；secret 进 env/argv；Linux Path B 当 Windows
  资格；Pi 当 Member 引擎；把 start 骨架改名当完整循环。
- **硬门:** 适用 Phase 13 六条 + `TEST-REPORT-INCREMENTAL-01`。

### P13-T03 — Hidden Pi Assistant real inference（explain / navigate / research / propose 真调 exact Pi）

- **status:** done (2026-09-03; merged PR [#311](https://github.com/agentkernel/cognitive-os/pull/311) at `main@7f9b4115`; lease closed; closure head required CI [33699388684](https://github.com/agentkernel/cognitive-os/actions/runs/33699388684) **SUCCESS** at `b043aa9c`; implementation required CI [33676327318](https://github.com/agentkernel/cognitive-os/actions/runs/33676327318) **SUCCESS** at `7e6157f1` and [33682946140](https://github.com/agentkernel/cognitive-os/actions/runs/33682946140) **SUCCESS** at the merged head `93e6ed01`; `DEV-LINUX-NATIVE-01`: store 10/10 + 6/6, runtime 7/7, adapter 21/21 + 8/8, kernel-server assistant 7/7, clippy clean at `347ab54a` and `93e6ed01`; **live** at `347ab54a`: `propose` / `explain` / `research` / `navigate` all HTTP 200 through exact Pi 0.81.1 + daemon Provider proxy (`deepseek-v4-flash`, pre-existing Secret Service entry, no key created) registering typed-provenance chains `charter` / `business-brief` / `research-run` / `axis` with `provider_round_trips: 1`; unbound runtime → `assistant.status: provider_unbound` + turn 409 Settings pointer, no Pi process; ambient tool 403 before Pi; unpinned research target reported, never fetched; failure-first A/B: `main@84188aac` still echoes 200; [report](../checkpoints/2026-09-03-personal-p13-t03-pi-inference-report.md), [closure](../checkpoints/2026-09-03-personal-p13-t03-pi-inference-closure.md)). Claim ceiling `hypothesis`. Windows Pi route `not-run` until P13-T13; Linux Pi qualification does not transfer to Windows. Implemented shape: store `AssistantTurnSpec.inference` mandatory + `validate_inferred_object_chain` (single closed-kind chain validator, `{value, provenance}` per field, `sources[]` only from fetched / owner-supplied) + `admit_turn_request` + `provider_unbound_guidance()`; `cognitive-runtime::pi_inference` (frames, `assemble_bounded_context` on T10 `CONTEXT_INJECT_ORDER`, prompt, chain parser, `validate_research_target`); `pi-agent-adapter assistant-turn` (exact pin, no tools / extensions / skills / session, tool events refused); kernel-server `assistant_inference.rs` (`GET assistant.status`, `POST assistant.turn` → v26 `register_candidate` + `request_preview`, pending preview re-announced); web `CreateAssistantChat` + `RailCanvasWrite`. Observed, not decided: `deepseek-v4-flash` needed three structural parser hardenings (prose wrapper, raw control characters in strings, dropped closer) before all four kinds landed — each still ends in the closed-schema validator; pinned research origins for `task://personal/assistant-research` default to empty, so live research exercised the fetch path only as a refusal.
- **2.0.0 表面:** 右栏助手真的会想：创建向导 ①/② 的研究与提案、`propose` 候选
  带 typed 出处；Provider 未绑时渲染「去 Settings 连接」指路文案。Pi 仍不进
  Installed Agents chrome。
- **依赖:** P11-T06 candidate path（`run_turn`、`validate_assistant_provenance`、
  `reject_closed_candidate_schema`、`admit_tool`）；P11-T05 archive；P11-T10
  `inject_order`；P11-T12 Provider binding；P12-T09 右栏；exact Pi pin。
  **不依赖 P13-T02。**
- **垂直切片:** D01：`assistant.turn` 四类 turn 在 daemon 侧真实调用 exact Pi
  （Provider 经 daemon proxy；有界 Context 按 T10 inject_order 组装）→ Pi 输出解析为
  候选对象链（BusinessBrief → ResearchRun → Charter/Axis/Roster/Recipe）→ 每字段
  typed 出处（`sources[]` / owner-stated / assistant-assumption；无出处被拒）→
  注册候选 + research/propose 可选 preview；research fetch 只走
  `HttpFetchReadOnly` 只读族；Provider 未绑 → 指路 Settings（非假聊天框、非静默
  绑模型）。
- **Scene:** `create-init`…`create-joint` 右栏、`today` / `project-*` 右栏助手会话。
- **acceptance:** 仍 candidate-only；无 authority/Secret/archive/Memory 写；
  draft-apply 与 authority-approve 两层永不混用（14 §2.4）。
- **不可做:** 助手直写权威；伪造 sources；ambient shell；把 Pi Linux 资格写成
  Windows；聊天 Approve。
- **本仓 foundation:** exact Pi client/Shell、P2-T04 private candidate worker
  composition、P3-T02 Context builder、P11-T06 closed candidate schema。
- **禁止再造:** 第二套候选 schema；Pi 当 Member 执行引擎。
- **validation environment:** `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` + 已 push
  exact-revision `DEV-LINUX-NATIVE-01`（pinned Pi）。`DEV-WIN-GNU-01` 仅
  fmt/docs/TS。Windows Pi 路由 `not-run` 直到 P13-T13；Linux Pi qualification 不转移
  Windows。
- **关闭门:** 四类 turn 真实经 daemon 调用 exact Pi 并产出带 typed 出处的候选对象
  链；Provider 未绑时指路 Settings；`run_turn` 不再只回显客户端 payload。
- **漂移检测负例:** 直连 SecretStore/DB/Provider；伪造 `sources[]`；无出处候选注册；
  ambient tool；preview bypass；未绑 Provider 仍假装回答；完成自报。
- **硬门:** 适用 Phase 13 六条 + `TEST-REPORT-INCREMENTAL-01`。

### P13-T04 — Independent verification + openable outputs + publication package

- **status:** done (2026-09-03; merged PR [#313](https://github.com/agentkernel/cognitive-os/pull/313) at `main@a5d4040c`; lease closed; `P13-T04/D01` + `P13-T04/D02` done; required CI [33705526112](https://github.com/agentkernel/cognitive-os/actions/runs/33705526112) **SUCCESS** at `84b2467e`, closure head [33714235588](https://github.com/agentkernel/cognitive-os/actions/runs/33714235588) **SUCCESS** at `6b6cbe6a`; `DEV-LINUX-NATIVE-01` at `84b2467e`: store 7/7 + 8/8 + 19/19 + 11/11 + 9/9, runtime 6/6, kernel-server 2/2 (real `node` child) + 2/2 + 19/19, clippy clean; **live daemon E2E** U0–U15 with a real pinned dsh deliverable through the bound daemon Provider proxy → CAS → verifier evidence → `outputs.open` / `export` → intermediate ring 422 → last-ring acceptance via ApprovalPreview → packet `planned` / `published:false` → external-send `planned` → tamper `failed`; 0 secret leaks; 关闭门 (1)–(4) true on Linux, host file-open cell `not-run` until `P13-T13`; running report [2026-09-03 P13-T04 report](../checkpoints/2026-09-03-personal-p13-t04-artifacts-verifier-report.md), [closure](../checkpoints/2026-09-03-personal-p13-t04-artifacts-verifier-closure.md)). Implementation shape: v37 `p13_attempt_artifact` / `p13_artifact_evidence` / `p13_run_acceptance`; the daemon ingests each terminal Attempt's `DeliverableDraft` candidate into the P3-T03 `ArtifactStore` CAS; independent verifier `verifier://personal/attempt-artifact` (deterministic: CAS re-read digest, source-frame binding, terminal Attempt, format parse, non-empty, no secret shape) writes append-only evidence whose report bytes live in the same CAS; `StageTestOracle` is derived from durable facts, never from caller booleans; `run-acceptance` and `external-send` join the P11-T09 ApprovalPreview subject kinds (v37 rebuild of the CHECK, v30 precedent).
- **2.0.0 表面:** `project-outputs` select-then-view 显示真实产物并可打开；run 验收
  **只在末环**；发布包 = 完整 AUTONOMY packet 画布预览；planned ≠ published；
  聊天无 Confirm。
- **依赖:** P13-T02；P11-T03 StageAcceptanceSpec / StageTestPassed /
  `acceptance_decision`；P11-T09 ApprovalPreview（external-send）；P3-T03 CAS；
  P2-T07 independent verifier；P12-T03 outputs 页。
- **垂直切片:** D01：Attempt 产物入 CAS（digest / format / 来源 / 新鲜度）→ 独立
  verifier 对业务产物出 evidence → 末环验收需 StageTestPassed；failure-first：模型
  文本 / HTTP 200 / `agent_end` 当完成被拒、中间环验收被拒、file-as-authority 被拒。
  D02：`outputs` 页 select-then-view + 打开/下载（Personal Home `data/`）；
  publication package 画布预览 → 确认 → external-send 走 T09 preview → receipt；
  planned ≠ published。
- **Scene:** `project-outputs`、`hitl`（external-send）、`project-runs`（验收环）。
- **acceptance:** 完成 = 独立 verifier + daemon acceptance；产物 = CAS 引用；
  publish 前必有 preview。
- **不可做:** file 当权威；receipt 当完成；假 Publish 按钮；聊天 Confirm。
- **本仓 foundation:** P2-T07 verifier seam、P3-T03 CAS、P11-T03 acceptance、
  P11-T09 ApprovalPreview、P11-T14 connector preview 形状。
- **禁止再造:** 第二套 artifact store；第二套 verifier。
- **validation environment:** `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` +
  `DEV-LINUX-NATIVE-01`；Dual Track TS（`DEV-WIN-GNU-01` 允许面）。宿主打开文件
  E2E = `DEV-WINDOWS-NATIVE-OPC-01` / `not-run`。
- **关闭门:** 真实 Attempt 产物入 CAS 且 `outputs` 可选看/打开；独立 verifier 出
  evidence；末环验收需 StageTestPassed；发布包画布预览且 planned ≠ published。
- **漂移检测负例:** 模型文本 / HTTP 200 / `agent_end` 当完成；file-as-authority；
  中间环验收；无验证即 published；聊天 Confirm；receipt-as-completion。
- **硬门:** 适用 Phase 13 六条 + `TEST-REPORT-INCREMENTAL-01`。

### P13-T05 — Runs / Routine loop + Today run overview

- **status:** done (2026-09-03; merged PR [#315](https://github.com/agentkernel/cognitive-os/pull/315) at `main@90437cb4`; lease `lease/personal/P13-T05/routine-runs` closed → PARALLEL-LANES §3.1; `P13-T05/D01` + `P13-T05/D02` done; required CI [33723531536](https://github.com/agentkernel/cognitive-os/actions/runs/33723531536) **SUCCESS** at `bea18fb2`; live-validated required CI [33714624207](https://github.com/agentkernel/cognitive-os/actions/runs/33714624207) **SUCCESS** at `ecd35ab0`; `DEV-LINUX-NATIVE-01` live daemon E2E 11/11 at `ecd35ab0` — arm after G2 → daemon tick → 6 real hosted Attempts → ledger no-overlap / queue-latest / coalesced / missed → safe-point pause / resume / restart → `runs` + Today; running report [P13-T05 report](../checkpoints/2026-09-03-personal-p13-t05-routine-runs-report.md), closure [P13-T05 closure](../checkpoints/2026-09-03-personal-p13-t05-routine-runs-closure.md); Windows clock / sleep / restart host cells `not-run` until P13-T13). Scope decision recorded at claim: the PlanRevision / stage-test / G2-acceptance product HTTP path gap found by P13-T02 is **not** widened into this card — arming fails closed before G2 (`ROUTINE_ARM_BEFORE_G2`), and the live E2E seeds a G2 fixture exactly as P13-T02 did; the gap stays owned by `P13-T04` (stage test / last-ring acceptance) and `P13-T06` (`@manager` plan revision).
- **2.0.0 表面:** `project-runs` 显示真实 occurrence ledger + Attempt 历史；
  Routine/Trigger 按 ③「周期与触发」在 G2 后武装；`today` 每 live Project 一行
  （状态 / 今日完成次数 / 当前环节 / 时长）+ created/live/blocked 计数 + 周期切换；
  无待批时折叠决策包保留概览；无 KPI 墙。
- **依赖:** P11-T08 Routine/Trigger 骨架；P13-T02；P11-T02 close-background；
  P12-T03 runs 页；P12-T05 Today。
- **垂直切片:** D01：G2 后按声明武装 Routine/Trigger；手动触发经 Intent；
  occurrence ledger（no-overlap / queue-latest / missed / coalesced）真实产生；安全点
  continue / pause / restart 不静默注入 prompt；close-window background-or-pause 与
  offline/missed/resume 事实相连；failure-first：第二 scheduler、overlap、静默丢
  occurrence、checkpoint 当完成被拒。D02：`runs` UI + Today 概览行/计数/周期切换。
- **Scene:** `project-runs`、`today`。
- **acceptance:** daemon scheduler 唯一调度权威；checkpoint 非权威；Working ≠
  completion。
- **不可做:** Inbox 一级；第二套 Temporal；假 Start 按钮；KPI 墙。
- **本仓 foundation:** P11-T08 `scheduler_entries` 复用、P2-T03 scheduler/fencing、
  P11-T02 host status。
- **禁止再造:** 第二 scheduler。
- **validation environment:** `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` +
  `DEV-LINUX-NATIVE-01`；Dual Track TS。clock/sleep/restart 宿主 E2E =
  `DEV-WINDOWS-NATIVE-OPC-01` / `not-run`。
- **关闭门:** `runs` 显示真实 occurrence ledger + Attempt 历史；Routine 按 ③ 声明在
  G2 后武装；手动触发经 Intent；安全点 continue/pause/restart；Today 概览行 + 计数
  + 周期切换。
- **漂移检测负例:** 第二 scheduler；overlap；静默丢 occurrence；checkpoint 当完成；
  运行中静默 prompt 注入；进程退出 = cancel/complete；KPI 墙；假 Start。
- **硬门:** 适用 Phase 13 六条 + `TEST-REPORT-INCREMENTAL-01`。

### P13-T06 — Project group conversation + manager routing（`@manager` / `@member`）

- **2.0.0 表面:** 右栏在 Project 内是群聊（Owner / manager / Members），在 Project 外
  是 Personal Assistant；`@manager` / `@member` 路由；manager-default speech；成员
  proactive speech 仅 mentioned / delivering / handoff / blocked / decision-request；
  `@` 只插入未发送草稿；聊天无 Approve。
- **依赖:** P11-T05 记录类型词汇（`owner-message | assistant-proposal | announce |
  receipt | member-deliverable | handoff | blocked | decision-request`）；P11-T04；
  P13-T02（成员真的会 deliver）；P13-T03（助手真的会答）；P12-T09。
- **垂直切片:** 群聊与助手会话分层可切换 → `@manager` → daemon Task/PlanRevision
  候选 → preview（非直写）→ `@member` 只路由该成员 Task → 成员发言规则由 daemon
  记录类型强制 → `@` 只进草稿；failure-first：聊天 Approve、成员间转移权威、跨项目
  读、secret 进聊天被拒。
- **Scene:** 全部 `project-*` 右栏、`today` 右栏。
- **acceptance:** Conversation 只宣布不批准；无完成语义；SecretStore takeover。
- **不可做:** 聊天 Approve；成员自发 handoff 转移权威；Conversation 当 Task 完成。
- **本仓 foundation:** P11-T05 archive/index、P11-T09 announce + 深链、P12-T09
  `assistant.turn` → `draft.apply`。
- **禁止再造:** 第二套会话投影；重解释 `conversation-projection/0.1`。
- **validation environment:** `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` +
  `DEV-LINUX-NATIVE-01`；Dual Track TS。
- **关闭门:** 群聊与助手会话分层；`@manager` → 候选 → preview；`@member` 只路由该
  成员 Task；成员发言规则；`@` 只进草稿；聊天无 Approve。
- **漂移检测负例:** 聊天 Approve；成员间直接转移权威；成员随意发言；`@` 直写权威；
  Conversation 当完成；跨项目读；secret 进聊天。
- **硬门:** 适用 Phase 13 六条 + `TEST-REPORT-INCREMENTAL-01`。

### P13-T07 — Knowledge & Memory surface completeness

- **2.0.0 表面:** Knowledge 每条 fragment 显示 provenance / rights / freshness /
  exclusion / untrusted-observation；reindex 与导入失败保留原件可见；Memory
  inspect / correct / promote / forget 表面；聊天自动准入进可检查 Memory；跨 Project
  promote 需 Owner 确认。
- **依赖:** P11-T10/T11 权威；P12-T07；P13-T06（自动准入来源；**非 mutex**）。
  **不依赖 P13-T02。**
- **垂直切片:** Knowledge 标签/状态 → reindex/失败态 → Memory 四动作走 management
  HTTP（tombstone 不复活）→ 聊天自动准入 → 跨 Project promote preview。
- **Scene:** `knowledge`。
- **acceptance:** files ≠ Project 权威；Memory 须 admission；forget 后 index/cache
  不复活。
- **不可做:** file-as-authority；越权检索当功能；Agent 自 admission；捆绑 Obsidian。
- **本仓 foundation:** P11-T10 Vault import/index/conflict、P11-T11 admission/
  privacy/forget、P4-T03 tombstone/non-resurrection。
- **禁止再造:** 文件当 CAS 权威；第二套 Memory 写路径。
- **validation environment:** Dual Track TS + `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01`。
  宿主 FS/privacy E2E `not-run` 直到 P13-T13。
- **关闭门:** 标签可见；reindex/失败态可见；Memory 四动作走 management HTTP 且
  tombstone 不复活；跨 Project promote 需 Owner preview。
- **漂移检测负例:** file-as-authority；越权检索；tombstone 复活；Agent 自
  admission；secret/PII 进 Memory；last-write-wins 无冲突记录；捆绑 Obsidian。
- **硬门:** 适用 Phase 13 六条 + Dual Track。

### P13-T08 — Settings completeness（Model Connections / 通知恢复 / 高级诊断 / state-lab）

- **2.0.0 表面:** Settings **Model Connections**：主流 Provider 模板下拉 + 自定义
  URL / 兼容模式 / key / model；connected / failed 不露 raw secret；不再指路
  Linux-era `/providers`；usage source-labelled actual / estimated / unknown≠0；
  通知 / 恢复分组；高级诊断默认折叠（DSH / Pi exact version / health / update /
  rollback）；state-lab 九态 × 九表面用真实组件渲染，Settings 高级默认隐藏。
- **依赖:** P11-T12；P8-T13 Provider CP；P12-T08；P11-T02 host status；P13-T02
  engine health facts（**非 mutex**：缺则诚实 empty）。
- **垂直切片:** D01：Model Connections 写路径（SecretStore takeover；`/providers`
  能力迁入 OPC Settings）+ usage 标签。D02：通知/恢复分组 + 高级诊断 + state-lab。
- **Scene:** `settings`、`state-lab`。
- **acceptance:** raw secret 永不进 UI/DSH/Pi；unknown≠0；成员级预算硬停不是
  chrome；订阅/发票管理不做。
- **不可做:** 假 Connect 按钮；Installed Agents 商店；state-lab 一级。
- **本仓 foundation:** P8-T13 Provider Control Plane（account/secret/binding/usage）、
  P11-T12 labelled usage、P12-T08 连接表/时间盒撤销/CloseBackground、P11-T07
  hosted DSH identity。
- **禁止再造:** 第二套凭据平面；engine store。
- **validation environment:** Dual Track TS + `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01`
  + `DEV-LINUX-NATIVE-01`（SecretStore route）。Windows SecretStore 宿主 =
  `DEV-WINDOWS-NATIVE-OPC-01` / `not-run`。
- **关闭门:** OPC Settings 内完成一个真实 Provider 连接且不露 secret；不再指路
  `/providers`；usage 标签；通知/恢复分组；高级诊断默认折叠；state-lab 九态 × 九表面
  真实组件、默认隐藏。
- **漂移检测负例:** raw secret 进 DOM/log/env；unknown=0；成员级预算 chrome；
  Installed Agents 商店；state-lab 一级；假 Connect；订阅/发票产品。
- **硬门:** 适用 Phase 13 六条 + Dual Track。

### P13-T09 — Project lifecycle + local recovery（副本 / 归档 / 删除 / 还原点 / 导出）

- **2.0.0 表面:** copy-project 生成 inactive 副本；归档先停 Routine/Trigger；删除
  需影响预览 + 二次确认；同盘自动版本 = local restore points（明示非灾备）；手动导出
  默认排除 secret；每 Project 自动 `data/` 目录。
- **依赖:** P11-T03；P11-T08；P11-T02（Personal Home `app/`/`data/`、restore-point-
  not-backup）；P7-T02 backup/restore 边界。
- **垂直切片:** copy → archive（停触发）→ delete（影响预览 + 二次确认）→ restore
  point → export（排除 secret）；failure-first：副本继承 grant/就位、删除不停触发、
  导出含 secret 被拒。
- **Scene:** `projects`、`project-detail`、`settings`（还原点/导出入口）。
- **acceptance:** 副本不复制运行/grant/Member 就位；archive-first；secret 默认排除。
- **不可做:** restore-as-backup 声称；跨项目共享 Member；删除即物理删。
- **本仓 foundation:** P11-T03 Project aggregate、P11-T08 Trigger、P11-T02
  restore-point 事实、P7-T02 export 排除规则。
- **禁止再造:** 第二套备份平面。
- **validation environment:** `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` +
  `DEV-LINUX-NATIVE-01`；Dual Track TS。Windows FS E2E `not-run` 直到 P13-T13。
- **关闭门:** copy = inactive 副本；归档先停触发；删除影响预览 + 二次确认；restore
  points 明示非灾备；导出排除 secret；每 Project 自动 `data/`。
- **漂移检测负例:** 副本自动激活/继承 grant/就位；删除不停触发；restore-as-backup；
  导出含 secret；跨项目共享 Member。
- **硬门:** 适用 Phase 13 六条 + `TEST-REPORT-INCREMENTAL-01`。

### P13-T10 — Skill/MCP security-reviewed acquisition + scoped grant

- **2.0.0 表面:** 成员 skills / tools / perms 标签可发起获取；助手主导发现 → 结构化
  安全评审展示 → 首次安装或扩权前 exact Owner 画布 preview → 版本锁定 capability
  artifact → 独立 Project/Member grant → 更新评审 / 兼容测试 / 回滚。广泛
  marketplace / MCP family console / engine store **不做**。
- **依赖:** P11-T04 InstallFact/Grant 拆分；P11-T09 grant-expansion preview +
  时间盒；P13-T03 助手研究；P12-T04 标签；P4 Skill package/revision/binding；
  P5-T03/T04 MCP transport facts（不等于 MCP 第七族）。
- **垂直切片:** 发现候选（带 sources）→ 评审（来源 / 许可 / 隐藏指令 /
  prompt-injection / 文件·网络·命令意图；MCP 加依赖 / 可执行代码 / 网络 / Secret /
  工具权限 / 供应链）→ preview → InstallFact → Grant（Project/Member 范围、时间盒可选）
  → 更新评审 / 兼容测试 / 回滚；failure-first：安装即授权、未评审安装、聊天 Approve、
  ambient grant 被拒。
- **Scene:** `member-config`（skills / tools / perms）、`hitl`（grant-expansion）。
- **acceptance:** 配方 ≠ 授权；grant 有范围；exact Owner 确认在画布。
- **不可做:** generic Resource schema；DSH native MCP/base tool 自动启用；engine
  store。
- **本仓 foundation:** P11-T04 InstallFact/Grant、P11-T09 subject_kind
  grant-expansion、P4 Skill、P5 MCP transport、P11-T06 research 只读边界。
- **禁止再造:** MCP family console；第二套 grant 表。
- **validation environment:** `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` +
  `DEV-LINUX-NATIVE-01`；Dual Track TS。supply-chain 宿主 E2E `not-run` 直到 P13-T13。
- **关闭门:** 发现 → 评审 → exact Owner 画布 preview → 版本锁定 → 按范围 grant →
  更新评审/兼容测试/回滚 全链可走且负例绿。
- **漂移检测负例:** 安装即授权；未评审自动安装；聊天 Approve；ambient grant；
  generic Resource schema；engine store / marketplace；grant 无过期无范围。
- **硬门:** 适用 Phase 13 六条 + `TEST-REPORT-INCREMENTAL-01`。

### P13-T11 — Reflection + versioned Member Runtime improvement

- **2.0.0 表面:** 关键结果 / 日 / 周期 / 事件反思候选；Member Runtime 改进 = 新
  revision + Owner preview + 可回滚；跨 Project Role Template 提案需 Owner 确认。
  设计目标层（2.0 scope §3.3），不是原型 Scene 新增。
- **依赖:** P13-T02；P13-T04；P13-T05；P11-T04 versioned Employee/Blueprint、
  per-Project opt-in 升级。
- **垂直切片:** daemon 从 Attempt / verification / evidence 事实生成反思候选（非模型
  自报）→ 改进提案 = 新 revision preview → Owner 确认 → 生效 / 回滚；跨 Project
  提案需 Owner 确认；failure-first：模型自报即改进、隐式 Blueprint 升级、运行中
  prompt 注入被拒。
- **Scene:** `project-members`（revision 历史）、`hitl`（改进 preview）。
- **acceptance:** 反馈先是 Project evidence，再是 versioned 提案；不静默注入。
- **不可做:** 自动升级 Blueprint；跨 Project 静默复用 Member。
- **本仓 foundation:** P11-T04 versioned Employee/Blueprint、P11-T09 preview、
  P3-T04 loop-control observation（repeat/no-progress 事实）。
- **禁止再造:** 第二套 evaluation 平面（evaluation routing 仍 OFF）。
- **validation environment:** `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` +
  `DEV-LINUX-NATIVE-01`。
- **关闭门:** 反思候选由 daemon 从事实生成；改进 = 新 revision + Owner preview +
  可回滚；跨 Project 提案需 Owner 确认。
- **漂移检测负例:** 模型自报即改进；隐式 Blueprint 升级；运行中 prompt 注入；反思
  当完成；跨 Project 静默复用 Member。
- **硬门:** 适用 Phase 13 六条 + `TEST-REPORT-INCREMENTAL-01`。

### P13-T12 — Visual spec + accessibility / visual qualification

- **2.0.0 表面:** D01 补 Phase 11 未产出的 **Visual UI 规格**（Apple-led；不改 IA；
  不重生 canvas）+ 冻结 v9 逐模块 `/ui/` 对照清单。D02 在真实 `/ui/` 上执行：State
  Lab 九态 × 九表面真布局、键盘可达与焦点恢复、200% 与窄窗三栏横滚不堆叠、
  light / dark / high-contrast host-theme 对比、NVDA 关键路径朗读。
- **依赖:** D01：P11-T01 设计文档（`09-state-accessibility-and-visual-system.md`、
  `10-component-map-and-prototype-flows.md`）+ 冻结 canvas v9。D02：P12-T02..T09 +
  P13-T04..T09 表面存在。D01 可与任何实现并行。
- **垂直切片:** D01 documentation-only：视觉规格 + 对照清单。D02：逐格执行并记账
  pass / fail / not-run。
- **Scene:** 全部 19 Scene + `state-lab`。
- **acceptance:** 对照清单每模块有判定；a11y 每格有环境 ID + exact revision。
- **不可做:** canvas 截图当验收；改 IA；Vite 当产品源；跑 `personal-20-prototype-review`
  phase 4 重生 canvas。
- **本仓 foundation:** P7-T05 rendered browser review 方法（exclusive Chrome、
  cells/assertions、overflow/clipped/contrast 计数）、`clients/pc/web/src/tokens.css`、
  P11-T13 host dump-dom / CDP 检查。
- **禁止再造:** 第二套设计系统；平行 canvas。
- **validation environment:** D01：local Markdown/link、`check:consistency`、
  handbook/docs-sync。D02：rendered browser / NVDA / 200% / theme review 从登记宿主
  本机浏览器对已 push exact-revision guest daemon `/ui/`（`DEV-LINUX-NATIVE-01`
  SSH 隧道）= implementation evidence；Windows native chrome 证据 =
  `DEV-WINDOWS-NATIVE-OPC-01` / `not-run`；缺环境诚实 `not-run`；不发明环境 ID。
- **关闭门:** D01 视觉规格 + 对照清单成文；D02 九态 × 九表面、键盘/焦点、200%/窄窗、
  host-theme、NVDA 每格记账。
- **漂移检测负例:** canvas 截图当验收；跳过格写 pass；改 IA；Vite 当产品源；假
  State Lab 静态图；把 rendered review 写成 Windows native 资格。
- **硬门:** 适用 Phase 13 六条 + `TEST-REPORT-INCREMENTAL-01`（D01 documentation-only
  出口写明）。
- **D01 done（2026-09-03，merged PR [#308](https://github.com/agentkernel/cognitive-os/pull/308) at `main@3680b742`；lease `lease/personal/P13-T12/visual-spec` 已关闭；D02 `ready`）:** 视觉规格
  [`personal-2.0-opc-visual-ui-spec.md`](../../personal/docs/architecture/personal-2.0-opc-visual-ui-spec.md)
  与对照清单
  [`personal-2.0-opc-v9-ui-comparison-checklist.md`](../../personal/docs/architecture/personal-2.0-opc-v9-ui-comparison-checklist.md)
  已成文（表 A 19/19 模块；九态 × 九表面 81 格、键盘/焦点 57 格、200%/窄窗 36 格、
  主题 40 格、NVDA 10 条路径；全部判定 `not-run`）。规格全部以现有 `tokens.css`
  token 名与七类 `StateCategory` 表达；新增 token 只列为「proposed」，不改 CSS。
  观察到但未裁决的漂移记在规格 §13（含：现网 `app.css` ≤ 1279 px 叠栏 vs 产品
  「窄窗横滚不叠栏」——目前无 P13 卡拥有该 CSS 修正）。状态源见 PROGRESS。

### P13-T13 — Windows native host qualification + hung native E2E backfill

- **2.0.0 表面:** 无新 chrome；把所有 `Requires-environment` / `not-run` 变成真实
  pass / fail。
- **依赖:** owner 提供 Windows 11 x86_64 宿主；`PERSONAL-TEST-ENVIRONMENTS.md`
  登记修订；P11-T02；P13-T02；P13-T05；P13-T08；P10-T18 历史 unsigned dev path。
  **宿主未到位时本卡 `blocked`，不挡任何其他卡。**
- **垂直切片:** D01：provision + qualify `DEV-WINDOWS-NATIVE-OPC-01`（image / tools /
  pins 写回登记）；unsigned 开发安装路径可在该宿主运行。D02：挂单原生 E2E 回填并
  逐格记 pass / fail：T02 install / tray / sleep / SecretStore；P13-T02 sandbox / ACL /
  supply chain；P13-T05 clock / sleep / restart；P13-T08 SecretStore / proxy；P13-T04
  文件打开；UI 原生 E2E；live X（T14）可继续 `not-run`。
- **Scene:** 无；环境与证据。
- **acceptance:** 环境登记状态改变；每个挂单格有实际执行记录。
- **不可做:** CI / GNU / WSL / Linux 当原生资格；B01-W 当日常机；签名/release 声称。
- **本仓 foundation:** P11-T02 host walking skeleton（v34 + `host.*`）、P7-T07 D02
  inspectable bootstrap installer + Credential Manager 后端、P10-T18 免签名路径文档。
- **禁止再造:** 第二套安装器；用 B01-DESKTOP-002 冒充 Windows。
- **validation environment:** `DEV-WINDOWS-NATIVE-OPC-01`（本卡负责资格化并写回
  登记）；`CI-WINDOWS-MSVC-01` compile。
- **关闭门:** 环境登记从 not provisioned/qualified 变为 qualified；挂单原生 E2E 在该
  环境实际跑过并逐格记账；unsigned 开发安装路径可运行。
- **漂移检测负例:** CI/GNU/WSL/Linux 当原生资格；`not-run` 写 pass；B01-W 当日常机；
  签名/release 声称；用 B01-DESKTOP-002 冒充 Windows。
- **硬门:** 适用 Phase 13 六条 + `TEST-REPORT-INCREMENTAL-01`。

---

# 12. 机器可读 typed dependency 图

本节是正式计划的研究级细化，不拥有 current status。`implementation_requires`、
`acceptance_requires`、`promotion_requires` 的定义以正式计划为准；后两者不是 isolated
implementation mutex。PERS-PR 映射由
[personal-trace.yaml](personal-trace.yaml) 承载。

后续 public contract 只登记 Lane-CTR prerequisites：`skill-manifest`、
`operation-descriptor`、`agent-adapter-manifest`、TaskContract resource bindings、
server-issued preview 与 Memory codegen；本批不实施。统一 projection 先 private +
versioned，第二个真实 adapter/client 后才评估 public `ResourceSummary`。禁止新增
`Process` domain 或 giant `Resource` schema。

```yaml
phases:
  P0:
    gate: G0
  P1:
    implementation_requires: [G0]
    acceptance_requires: [B01]
    gate: G1_B01
  P2:
    implementation_requires: [P1_CONTRACTS]
    acceptance_requires: [B02, B04, B05, B12]
    gate: G2_B02_B04_B05_B12
  P3:
    implementation_requires: [P2-T01, P2-T02, P2_APPLICATION_CONTRACTS]
    acceptance_requires: [B03]
    observations: [B06, B07]
    gate: G3_B03
  P4:
    implementation_requires: [P3-T01, P3-T02, P3_STABLE_CONTEXT_PORTS]
    acceptance_requires: [B08]
    gate: G4_B08
  P5A_MANAGED_PI:
    implementation_requires: [P0-T06, P1-T08, P2_SUPERVISION_PORTS]
    acceptance_requires: [B09]
  P5B_TOOL_MCP:
    implementation_requires: [P2_TOOL_EFFECT_LOOP]
    acceptance_requires: [B10]
  P6:
    implementation_requires: [SINGLE_AGENT_BENCHMARK, PARALLEL_BENEFIT_HYPOTHESIS]
    acceptance_requires: [B11]
    gate: G6_B11
  P7:
    implementation_requires: [P1_INSTALL_FOUNDATION, P2_PRODUCT_INTERFACES, P3_CONTEXT_CORRECTNESS, P4_MEMORY_SKILL, P5A_MANAGED_PI]
    promotion_requires: [B01, B02, B03, B04, B05, B08, B09, B12]
    gate: GMVP_LINUX_then_G7_RC
  P10:
    implementation_requires: [OWNER_ACCEPTED_PERSONAL_2_0_SCOPE]
    acceptance_requires: [P10-T01, P10-T02]
    disposition: SUPERSEDED_BY_ADR_0059_PHASE_11
    claim_boundary: HISTORICAL_NO_NEW_GATE
  P11:
    implementation_requires: [P11-T01]
    acceptance_requires: [P11-T15]
    claim_boundary: WINDOWS_OPC_NON_CLAIM_UNTIL_SEPARATE_RELEASE
  P12:
    implementation_requires: [P12-T01]
    acceptance_requires: [P12-T02, P12-T03, P12-T04, P12-T05, P12-T06, P12-T07, P12-T08, P12-T09]
    claim_boundary: HYPOTHESIS_NOT_T15_NOT_GATE_NOT_PIXEL_REPLICA
  P13:
    implementation_requires: [P13-T01]
    acceptance_requires: [P13-T02, P13-T03, P13-T04, P13-T05, P13-T06, P13-T07, P13-T08, P13-T09, P13-T10, P13-T11, P13-T12, P13-T13]
    exit: P11-T15
    claim_boundary: HYPOTHESIS_NOT_RELEASE_NOT_SIGNING_NOT_B01W_NOT_2_1

linux_1_0_active_tracks:
  RUNTIME_SPINE: [P1-T09, P2-T01, P2-T02, P2-T03, P2-T04, P2-T05, P2-T06, P2-T07, P2-T08, P5-T01, P5-T02, P5-T05]
  RESOURCE_VALUE: [P3-T01, P3-T02, P3-T03, P3-T04, P3-T05, P3-T06, P4-T01, P4-T02, P4-T03, P4-T04, P4-T05, P4-T06]
  PRODUCT_OPERABILITY: [P7-T01, P7-T02, P7-T03, P7-T08]

post_1_0:
  EMBEDDING_SEMANTIC_VECTOR_GRAPH: []
  MCP_TOOL_ADAPTER_HISTORY: [P5-T03, P5-T04, B10]
  MCP_SEVENTH_FAMILY_ADVANCED: [P10-T01, P10-T02, FUTURE_MCP_SUCCESSOR]
  MULTI_AGENT: [P6-T01, P6-T02, P6-T03, P6-T04, B11]
  WEB_UI_WINDOWS: [P7-T05, P7-T07, B01-W]
  WINDOWS_OPC_2_0: [P11-T01, P11-T02, P11-T03, P11-T04, P11-T05, P11-T06, P11-T07, P11-T08, P11-T09, P11-T10, P11-T11, P11-T12, P11-T13, P11-T14, P11-T15]
  FROZEN_PROTOTYPE_UI_COMPLETENESS: [P12-T01, P12-T02, P12-T03, P12-T04, P12-T05, P12-T06, P12-T07, P12-T08, P12-T09]
  PERSONAL_2_0_0_COMPLETION: [P13-T01, P13-T02, P13-T03, P13-T04, P13-T05, P13-T06, P13-T07, P13-T08, P13-T09, P13-T10, P13-T11, P13-T12, P13-T13, P11-T15]

tasks:
  P0-T01: { implementation_requires: [] }
  P0-T02: { implementation_requires: [P0-T01] }
  P0-T03: { implementation_requires: [P0-T02] }
  P0-T04: { implementation_requires: [P0-T02] }
  P0-T05: { implementation_requires: [P0-T01] }
  P0-T06: { implementation_requires: [P0-T03] }
  P0-T07: { implementation_requires: [P0-T02] }

  P1-T01: { implementation_requires: [P0-T04] }
  P1-T02: { implementation_requires: [P0-T05, P1-T01] }
  P1-T03: { implementation_requires: [P1-T02] }
  P1-T04: { implementation_requires: [P0-T07, P1-T01] }
  P1-T05: { implementation_requires: [P1-T03, P1-T04] }
  P1-T06: { implementation_requires: [P1-T02, P1-T05] }
  P1-T07: { implementation_requires: [P0-T06, P1-T03, P1-T04, P1-T05] }
  P1-T08: { implementation_requires: [P0-T03, P1-T01, P1-T04, P1-T06, P1-T07] }
  P1-T09:
    implementation_requires: [P1-T08]
    acceptance_requires: [B01_CAMPAIGN_MIN_20_SUCCESS_GTE_90_ZERO_CRITICAL]
    promotion_requires: [B01]

  P2-T01:
    implementation_requires: [P1-T01, P1-T04]
    acceptance_requires: [P1-T09]
  P2-T02:
    implementation_requires: [P2-T01, P1-T07]
    acceptance_requires: [REAL_RESOURCE_TASK_API, PRIVATE_VERSIONED_PROJECTION, WATCH_RECOVERY, SIDECAR_CLI_SERVICE_PARITY, CHANNEL_ISOLATION]
    promotion_requires: [B02, B04, B05]
  P2-T03:
    implementation_requires: [P2-T01, P1-T01]
    acceptance_requires: [DURABLE_STOP, EFFECT_CLOSURE, WORKER_FENCING, CRASH_CLOCK_BUDGET_TESTS]
    promotion_requires: [B05, B12]
  P2-T04:
    implementation_requires: [P2-T02, P2-T03]
    acceptance_requires: [SCHEDULER_CONTEXT_PI_SIDECAR_BOUNDED_HARNESS]
  P2-T05:
    implementation_requires: [P2-T04]
    acceptance_requires: [NATIVE_TOOL_REGISTRY, WORKSPACE_READ_SEARCH_WRITE_PATCH, BOUNDED_PROCESS_CHECK, READ_ONLY_HTTP_FETCH]
  P2-T06:
    implementation_requires: [P2-T05]
    acceptance_requires: [TOOL_PROCESS_EXECUTOR, SUPERVISOR_CURSOR, FAULT_RECONCILE]
  P2-T07:
    implementation_requires: [P2-T03, P2-T04, P2-T06, ARTIFACT_CAS_PORT]
    acceptance_requires: [CHECKPOINT_ARTIFACT_EVIDENCE_INDEPENDENT_VERIFIER]
  P2-T08:
    implementation_requires: [P2-T07]
    acceptance_requires: [RUNTIME_SPINE_E2E, B02, B04, B05, B12]

  P3-T01:
    implementation_requires: [P2-T01, P2-T02, P2_APPLICATION_CONTRACTS]
    acceptance_requires: [REAL_CONTEXT_SOURCES, SCOPE_BEFORE_RANKING]
  P3-T02:
    implementation_requires: [P3-T01]
    acceptance_requires: [MINIMUM_CONTEXT_BUILDER, REQUIRED_FAIL_CLOSED, EXPLICIT_LOSS]
  P3-T03:
    implementation_requires: [P3-T02]
    acceptance_requires: [UNIQUE_ARTIFACT_CAS]
  P3-T04:
    implementation_requires: [P3-T02, P3-T03]
    acceptance_requires: [CONTEXT_DELTA, STABLE_PREFIX, GOVERNANCE_BOUND_CACHE, TELEMETRY]
  P3-T05:
    implementation_requires: [P3-T04]
    acceptance_requires: [UCR-01_RUNNER, RAW_NON_CLAIM_BASELINE]
  P3-T06:
    implementation_requires: [P3-T05]
    acceptance_requires: [B03]
    observations: [B06, B07]
    promotion_requires: [B03]

  P4-T01: { implementation_requires: [P3-T01, P3-T02, P3_STABLE_CONTEXT_PORTS] }
  P4-T02: { implementation_requires: [P4-T01] }
  P4-T03: { implementation_requires: [P4-T01] }
  P4-T04:
    implementation_requires: [P3-T02]
    acceptance_requires: [SKILL_PACKAGE_REVISION, LOCAL_IMPORT, RESOURCE_BINDING]
  P4-T05:
    implementation_requires: [P4-T01, P4-T02, P4-T03, P4-T04]
    acceptance_requires: [MEMORY_SKILL_API_PROJECTION, ACTUAL_CONSUMPTION]
  P4-T06:
    implementation_requires: [P4-T05]
    acceptance_requires: [B08, UCR-01_MEMORY_SKILL_CONSUMPTION]
    promotion_requires: [B08]

  P5-T01:
    implementation_requires: [P0-T04, P0-T06, P1-T08]
    acceptance_requires: [OFFICIAL_PI_ACQUISITION, SIDECAR_PACKAGE_PROTOCOL_ADAPTER_PINS, SIGNED_ACQUISITION_LOCK, INSTALL_UPGRADE_ROLLBACK_UNINSTALL_NEGATIVES]
    promotion_requires: [B09]
  P5-T02:
    implementation_requires: [P5-T01, P2-T03, P2-T06, P2_SUPERVISION_CONTRACTS]
    acceptance_requires: [SIDECAR_CONTRACT_REGISTRATION, PI_REGISTRY_INSTANCE_HEALTH, AGENT_SIDECAR_PROCESS_IDENTITY, SUPERVISION, PAUSE_RESUME_STOP]
    promotion_requires: [B09]
  P5-T03:
    implementation_requires: [P2-T05, P2-T08]
    acceptance_requires: [POST_1_0_MCP_ADAPTER_QUALIFICATION]
  P5-T04:
    implementation_requires: [P5-T03]
    acceptance_requires: [POST_1_0_DYNAMIC_TOOL_ECOSYSTEM, B10]
  P5-T05:
    implementation_requires: [P5-T02]
    acceptance_requires: [B09]
    promotion_requires: [B09]

  P6-T01: { implementation_requires: [P2-T08, SINGLE_AGENT_BENCHMARK, PARALLEL_BENEFIT_HYPOTHESIS], default: disabled, release: post-1.0 }
  P6-T02: { implementation_requires: [P6-T01] }
  P6-T03: { implementation_requires: [P6-T02] }
  P6-T04:
    implementation_requires: [P6-T03]
    acceptance_requires: [B11]

  P7-T01: { implementation_requires: [P0-T03, P1-T08, P2-T08] }
  P7-T02: { implementation_requires: [P1-T01, P1-T08, P2-T08, P7-T01] }
  P7-T03: { implementation_requires: [P1-T05, P2-T08, P7-T02] }
  P7-T04: { implementation_requires: [P3-T06, P4-T05, P5-T05] }
  P7-T05: { implementation_requires: [P2-T08, P7-T03] }
  P7-T06: { implementation_requires: [P7-T08, P7-T04, P5-T05] }
  P7-T07: { implementation_requires: [P1-T02, P7-T01, P7-T02] }
  P7-T08:
    implementation_requires: [P1-T09, P2-T08, P3-T06, P4-T06, P5-T01, P5-T02, P5-T05, P7-T01, P7-T02, P7-T03]
    acceptance_requires: [SIX_RESOURCE_RELEASE_MANIFEST, MANAGED_PI_SIDECAR, RESOURCE_ACTUAL_CONSUMPTION, PRODUCT_LIFECYCLE, MEMORY_SKILL_BACKUP_RESTORE, SIX_RESOURCE_DOCTOR_SUPPORT]
    promotion_requires: [B01, B02, B03, B04, B05, B08, B09, B12]

  P10-T01:
    implementation_requires: [OWNER_ACCEPTED_2026_08_27_SCOPE, ADR-0037, ADR-0043, ADR-0055, FINALIZED_PERSONAL_1_0_BASELINE]
    acceptance_requires: [ADR-0056, ADR-0057, ADR-0037_PARTIAL_BACKLINK, PRODUCT_PLAN_TRACE_SUPPORT_VERSION_SYNC, ADR-0055_REQUIRES_BACKEND_HONESTY]
  P10-T02:
    implementation_requires: [P10-T01]
    acceptance_requires: [LANE_CTR_MCP_CONVERSATION_CONTRACT_COMPATIBILITY_DECISION]
  P10-T03:
    implementation_requires: [P10-T02, P5-T03, P5-T04, P8-T12]
    acceptance_requires: [MCP_FAMILY_DAEMON_AUTHORITY, TOOL_CONTEXT_SKILL_ADMISSION, HEALTH_QUARANTINE_RECONCILIATION]
  P10-T04:
    implementation_requires: [P10-T02, P10-T03, P7-T05, P8-T13]
    acceptance_requires: [DESKTOP_PRIMARY_ENTRY, TARGET_IA, GLOBAL_AGENT_SHELL_CANDIDATE_ONLY, VENDOR_CONVERSATION_CAPABILITY_MATRIX, NATIVE_AGENT_APP_COEXISTENCE, ADR-0055_IMPORT_HONESTY]
    disposition: CANCELLED_SUPERSEDED_BY_P11_T13

  P11-T01:
    implementation_requires: [OWNER_APPROVED_OPC_BASELINE, ADR-0059, FINALIZED_PERSONAL_1_0_BASELINE]
    acceptance_requires: [OPC_PRODUCT_DESIGN_ARCHITECTURE_PLAN_TRACE_HANDBOOK_CANVAS_CLOSURE]
  P11-T02:
    implementation_requires: [P11-T01, ADR-0052, WINDOWS_INSTALL_FRAGMENTS]
    acceptance_requires: [WINDOWS_HOST_TRAY_BACKGROUND_SLEEP_MISSED_RECOVERY]
    notes: DOES_NOT_BLOCK_P11_T03
  P11-T03:
    implementation_requires: [P11-T01, TASK_INTENT_EFFECT_VERIFICATION_FOUNDATION]
    acceptance_requires: [PROJECT_CHARTER_GOAL_PLAN_TASK_ATTEMPT_AUTHORITY]
    notes: FIRST_KNIFE_NO_T02_MUTEX
  P11-T04:
    implementation_requires: [P11-T03, AGENT_ADAPTER_IDENTITY_FOUNDATION]
    acceptance_requires: [BLUEPRINT_ASSIGNMENT_EMPLOYEE_MANAGER_HANDOFF]
  P11-T05:
    implementation_requires: [P11-T03, P11-T04, ADR-0058_RETAINED_BOUNDARY]
    acceptance_requires: [PERSONAL_CONVERSATION_NEW_PRIVATE_VERSION]
    notes: NO_REINTERPRET_0_1_NO_FIRST_LANE_CTR
  P11-T06:
    implementation_requires: [P11-T03, P11-T05, EXACT_PI_FOUNDATION]
    acceptance_requires: [PI_BACKED_PERSONAL_ASSISTANT_CANDIDATE_ONLY]
  P11-T07:
    implementation_requires: [P11-T03, P11-T04, P11-T12, DSH_ADAPTER_FOUNDATION]
    acceptance_requires: [HIDDEN_HOSTED_DSH_ISOLATED_CHILD_PROXY]
    notes: NOT_VISIBLE_INSTALLED_AGENT_NO_T02_MUTEX
  P11-T08:
    implementation_requires: [P11-T03, SCHEDULER_EFFECT_RECOVERY_FOUNDATION]
    acceptance_requires: [ROUTINE_TRIGGER_NO_OVERLAP_QUEUE_LATEST_MISSED_RESUME]
    notes: DOES_NOT_BLOCK_P11_T09
  P11-T09:
    implementation_requires: [P11-T03, PREVIEW_EFFECT_ALERT_RECOVERY_FOUNDATION]
    acceptance_requires: [HITL_CANVAS_TODAY_DEEP_LINK_NO_CHAT_APPROVE]
    notes: NO_T08_MUTEX_NOT_INBOX_FIRST_LEVEL
  P11-T10:
    implementation_requires: [P11-T03, P11-T05, MEMORY_SKILL_CONTEXT_ARTIFACT_FOUNDATION]
    acceptance_requires: [KNOWLEDGE_VAULT_IMPORT_INDEX_CONFLICT]
  P11-T11:
    implementation_requires: [P11-T05, P11-T10, MEMORY_ADMISSION_FORGET_FOUNDATION]
    acceptance_requires: [EPISODIC_RETRIEVAL_MEMORY_PRIVACY_CORRECT_FORGET]
  P11-T12:
    implementation_requires: [P11-T03, P11-T04, PROVIDER_CONTROL_PLANE_FOUNDATION]
    acceptance_requires: [HONEST_USAGE_UNKNOWN_NEQ_ZERO]
    notes: NO_T07_MUTEX_MEMBER_BUDGET_DEFERRED_2_1
  P11-T13:
    implementation_requires: [P11-T03, VISUAL_UI_SPEC_BEFORE_CODING]
    acceptance_requires: [TODAY_PROJECTS_KNOWLEDGE_SETTINGS_UI]
    notes: DUAL_TRACK_AFTER_T03_HTTP_STABLE_NVDA_HUNG_NOT_RUN
  P11-T14:
    implementation_requires: [P11-T03, P11-T09, P11-T12]
    acceptance_requires: [X_CONNECTOR_WALKING_SKELETON_HITL_UNKNOWN_READBACK]
  notes: NOT_P0_HERO_LIVE_X_API_NOT_RUN_DONE_PR_293
  P11-T15:
    implementation_requires: [P11-T01, P13-T01]
    acceptance_requires: [P13-T02, P13-T03, P13-T04, P13-T05, P13-T06, P13-T07, P13-T08, P13-T09, P13-T10, P13-T11, P13-T12, P13-T13, FIXED_N15_WINDOWS_OPC]
    notes: UNPARKED_NOT_P12_MUTEX_PHASE_13_EXIT_REQUIRES_QUALIFIED_WINDOWS
    promotion_requires: [PRODUCTION_SIGNING, B01-W, OWNER_RELEASE_DISPOSITION]
  P12-T01:
    implementation_requires: [OWNER_2026_08_31_PHASE_12_AUTHORIZATION, P11-T01, P11-T13, P11-T14]
    acceptance_requires: [PHASE_12_CARDS_TRACE_PROGRESS_HANDBOOK_CONSISTENCY]
    notes: DOCUMENTATION_ONLY
  P12-T02:
    implementation_requires: [P12-T01, P11-T03, P11-T04, P11-T12, P11-T13]
    acceptance_requires: [EMPTY_HOME_ONLY_CREATE_FIVE_STEP_WIZARD]
  P12-T03:
    implementation_requires: [P12-T01, P11-T03, P11-T04, P11-T08, P11-T13]
    acceptance_requires: [PROJECT_FOUR_SUBMENUS]
    notes: NO_T02_MUTEX
  P12-T04:
    implementation_requires: [P12-T03, P11-T04]
    acceptance_requires: [SELECT_THEN_CONFIGURE_EIGHT_TABS_ADD_MEMBER]
  P12-T05:
    implementation_requires: [P12-T01, P11-T03, P11-T09, P11-T13]
    acceptance_requires: [TODAY_DECISION_PACKETS]
  P12-T06:
    implementation_requires: [P12-T03, P11-T09]
    acceptance_requires: [HITL_CANVAS_CONFIRM_NO_CHAT_APPROVE]
    notes: NO_T08_MUTEX
  P12-T07:
    implementation_requires: [P12-T01, P11-T10, P11-T13]
    acceptance_requires: [KNOWLEDGE_INGEST_WHY_THIS_FRAGMENT]
  P12-T08:
    implementation_requires: [P12-T01, P11-T02, P11-T12, P11-T13]
    acceptance_requires: [SETTINGS_CONNECTIONS_DONT_ASK_CLOSE_BACKGROUND]
  P12-T09:
    implementation_requires: [P12-T01, P11-T05, P11-T06]
    acceptance_requires: [RAIL_EDIT_CONFIRM_WRITE_NO_APPROVE]
  P13-T01:
    implementation_requires: [OWNER_2026_09_02_COMPLETION_INSTRUCTION, P11-T01, P12-T01, P12-T09, DOC_P12_ALIGN_DEBT_DONE]
    acceptance_requires: [PHASE_13_CARDS_T15_PREREGISTRATION_DRAFT_TRACE_PROGRESS_HANDBOOK_CONSISTENCY]
    notes: DOCUMENTATION_ONLY
  P13-T02:
    implementation_requires: [P13-T01, P11-T07, P11-T03, P11-T04, P11-T12, P2-T03, P2-T06, P2-T07]
    acceptance_requires: [HOSTED_DSH_REAL_ATTEMPT_LOOP_STDIO_BROKER_HEALTH_UPDATE_ROLLBACK]
    notes: FIRST_KNIFE_NO_T03_MUTEX_WINDOWS_SANDBOX_E2E_VIA_P13_T13
  P13-T03:
    implementation_requires: [P13-T01, P11-T06, P11-T05, P11-T10, P11-T12, P12-T09, EXACT_PI_FOUNDATION]
    acceptance_requires: [ASSISTANT_TURN_REAL_PI_INFERENCE_TYPED_PROVENANCE_CANDIDATES]
    notes: FIRST_KNIFE_NO_T02_MUTEX_PI_LINUX_NOT_WINDOWS
  P13-T04:
    implementation_requires: [P13-T02, P11-T03, P11-T09, P3-T03, P2-T07, P12-T03]
    acceptance_requires: [INDEPENDENT_VERIFY_CAS_OUTPUTS_OPENABLE_LAST_RING_ACCEPTANCE_PUBLICATION_PREVIEW]
  P13-T05:
    implementation_requires: [P11-T08, P13-T02, P11-T02, P12-T03, P12-T05]
    acceptance_requires: [RUNS_OCCURRENCE_LEDGER_ROUTINE_ARMING_SAFE_POINT_TODAY_OVERVIEW]
  P13-T06:
    implementation_requires: [P11-T05, P11-T04, P13-T02, P13-T03, P12-T09]
    acceptance_requires: [PROJECT_GROUP_CHAT_MANAGER_MEMBER_ROUTING_NO_APPROVE]
  P13-T07:
    implementation_requires: [P11-T10, P11-T11, P12-T07]
    acceptance_requires: [KNOWLEDGE_PROVENANCE_RIGHTS_MEMORY_INSPECT_CORRECT_PROMOTE_FORGET_UI]
    notes: P13_T06_AUTO_ADMISSION_SOURCE_NOT_MUTEX_NO_T02_MUTEX
  P13-T08:
    implementation_requires: [P11-T12, P8-T13, P12-T08, P11-T02]
    acceptance_requires: [SETTINGS_MODEL_CONNECTIONS_NOTIFICATIONS_DIAGNOSTICS_STATE_LAB]
    notes: P13_T02_ENGINE_HEALTH_NOT_MUTEX
  P13-T09:
    implementation_requires: [P11-T03, P11-T08, P11-T02, P7-T02]
    acceptance_requires: [PROJECT_COPY_ARCHIVE_DELETE_RESTORE_POINT_EXPORT]
  P13-T10:
    implementation_requires: [P11-T04, P11-T09, P13-T03, P12-T04, P4-T04, P5-T03, P5-T04]
    acceptance_requires: [SKILL_MCP_REVIEWED_ACQUISITION_SCOPED_GRANT_ROLLBACK]
  P13-T11:
    implementation_requires: [P13-T02, P13-T04, P13-T05, P11-T04]
    acceptance_requires: [REFLECTION_CANDIDATES_VERSIONED_MEMBER_RUNTIME_IMPROVEMENT]
  P13-T12:
    implementation_requires: [P11-T01, FROZEN_CANVAS_V9, P12-T02, P12-T03, P12-T04, P12-T05, P12-T06, P12-T07, P12-T08, P12-T09]
    acceptance_requires: [VISUAL_SPEC_D01, STATE_LAB_KEYBOARD_NVDA_200_HOST_THEME_D02]
    notes: D01_DOCUMENTATION_ONLY_PARALLEL_D02_AFTER_P13_T04_T05_T07_T08
  P13-T13:
    implementation_requires: [OWNER_PROVISIONED_WINDOWS_11_HOST, P11-T02, P13-T02, P13-T05, P13-T08]
    acceptance_requires: [DEV_WINDOWS_NATIVE_OPC_01_QUALIFIED, HUNG_NATIVE_E2E_BACKFILLED]
    notes: BLOCKED_UNTIL_HOST_DOES_NOT_BLOCK_OTHER_CARDS
    promotion_requires: [PRODUCTION_SIGNING, B01-W, OWNER_RELEASE_DISPOSITION]

# Linux 1.0 critical path 汇合 Runtime Spine、Resource Value、managed Pi sidecar
# 和 Product Operability。B06/B07/B10/B11、P6、P7-T05 与 P7-T07 不阻塞。
mvp_critical_path:
  - P0-T01
  - P0-T02
  - P0-T04
  - P1-T01
  - P1-T02
  - P1-T03
  - P1-T04
  - P1-T05
  - P1-T06
  - P1-T07
  - P1-T08
  - P1-T09
  - P2-T01
  - P2-T02
  - P2-T03
  - P2-T04
  - P2-T05
  - P2-T06
  - P2-T07
  - P2-T08
  - P3-T01
  - P3-T02
  - P3-T03
  - P3-T04
  - P3-T05
  - P3-T06
  - P4-T01
  - P4-T02
  - P4-T03
  - P4-T04
  - P4-T05
  - P4-T06
  - P5-T01
  - P5-T02
  - { task: P5-T05, acceptance_slice: B09 }
  - P7-T01
  - P7-T02
  - P7-T03
  - P7-T08

# Full RC 在公开 Linux 1.0 后汇合已选择的 post-1.0 能力列车。Multi-Agent 的明确
# NO-GO/disabled disposition 不要求 P6-T01..T04 成为强制路径；embedding 与 B10
# 也只在 release claim 选择它们时加入。
full_rc_critical_path:
  - P7-T08
  - P7-T04
  - P7-T06
```

## 12.1 可并行项

- P0-T03、P0-T04、P0-T05、P0-T07；
- P1-T02 与 P1-T04 在 P1-T01 后；
- P2-T02 与 P2-T03；
- P4-T02 与 P4-T03；
- P5-T02 与 P5-T03；
- P7-T01 与 P7-T04 的准备工作。

## 12.2 文件冲突热点

| 路径 | 冲突风险 |
|---|---|
| `personal/crates/cognitive-runtime/src/lib.rs` | Provider、Task、Scheduler、Process、Memory都可能触碰；必须按 phase串行 |
| `personal/crates/cognitive-store/src/sqlite.rs` | migrations、scheduler、memory；先拆模块再分别所有权 |
| `personal/apps/kernel-server/src/main.rs` | daemon/auth/routes；Phase 1后禁止并行大改 |
| `personal/apps/admin-cli/src/main.rs` | init/task/memory/agents/tools；先抽 shared client/service |
| `packages/sdk-ts` | API contract跟随 Lane-CTR，禁止独立造类型 |
| `docs/plan/PROGRESS.md` | 后合并者负责 rebase |
| migration目录 | migration编号必须单一分配，不可并行抢号 |

---

# 13. Benchmark Suite

| ID | 目标/前置与输入 | 步骤和预期转换 | 观测、成功/失败 | Token/时间/证据/自动化 |
|---|---|---|---|
| B01 | clean Linux VM、测试 DeepSeek account | install→init→secret→models→probe→daemon→Pi→response | ready必须来自真实checks；Key泄漏或仅HTTP 200即失败 | TTFC、probe tokens；VM runner+redacted bundle |
| B02 | 已初始化系统 | Shell查询system/task/process/agent/tool/memory | NL→management intent→真实projection | synthetic/cached错误状态失败；响应tokens/latency |
| B03 | 固定真实仓库快照 + real Context sources | scope-filter→minimum builder→Artifact refs→定位模块并输出 path/symbol evidence | required source、授权顺序、loss、revocation/cache correctness 全通过；收益非 pass 前提 | source/loss/access evidence；tokens/time 仅描述 |
| B04 | 有可复现bug fixture | reproduce→diagnose→change→test→verify | 只修部分、仅zero exit、自报成功均失败 | tokens/progress、tool calls、verified criteria |
| B05 | 运行中Task | 关Shell→重连→关daemon→restart→recover | fencing/replay/reconcile后继续；无重复Effect | recovery steps/time、effect evidence |
| B06 | 大日志、多文件、多轮工具 | externalize→delta/stable-prefix→dedup/cache→continue | 目标/约束/required evidence不可丢；采集但不阻塞 1.0 | repeated ratio、compression savings |
| B07 | 固定重复失败executor | repeated error→detect→switch→blocked | 预算内停止，不无限调用；采集但不阻塞 1.0 | no-progress、repeat action、guard latency |
| B08 | Memory corpus + local Skill package | save/import→bind→retrieve→actual Context/Task use→update/conflict/revoke/expire/forget | stale/forgotten Memory 与 revoked Skill 不得使用；仅展示不算 consumption | lifecycle/binding/consumption evidence；precision/latency仅描述 |
| B09 | exact official npm Pi + exact sidecar + production-signed acquisition lock | acquire→verify→install→register protocol/adapter/instance/process→health→activate→pause/resume→upgrade/rollback→stop→uninstall/recover | install ≠ permission；PiSession/SidecarSession ≠ AgentInstance；非 Pi 不继承资格 | acquisition/pins/lifecycle/recovery evidence、independent verifier |
| B10 | post-1.0 signed MCP/dynamic Tool fixture | qualify→discover→enable→call→quarantine/disable→uninstall | schema drift/disabled/bypass拒绝；不阻塞 Linux 1.0 | calls/failure/cache/result tokens |
| B11 | 可并行研究任务 | single baseline→2 workers→reviewer→integrator | 无重复/写冲突，verified结果不差 | speedup、coordination overhead、conflicts |
| B12 | executor timeout/uncertain | dispatch→timeout→OUTCOME_UNKNOWN→query/reconcile | 不换key、不盲重试、不完成Task | recovery time/steps、original key evidence |

**UCR-01 cross-Gate workload（不是新 Gate）：**
[UCR-01](../evaluation/personal-unified-cognitive-resource-workload.md) 在同一 Task trace
使用 Memory、Skill、Tool、Context、Task、Runtime，可分别为 B02/B03/B04/B05/B08/B09/B12
贡献证据。每个 Gate 必须单独 preregister exact workload digest、environment、threshold、
failure accounting、collector 和 independent verifier；一次 run 不自动 pass 多个 Gate，
也不创建 B13 或第二 release Gate。

所有 Benchmark：

- 固定 repository/task/provider/model/capability snapshot；
- 至少 30 次有效 run，安装等高成本场景至少 20 次；
- 同时报告中位数、p95、bootstrap 95% CI、失败样本；
- raw evidence 保存在 ignored `artifacts/evidence/personal/`；
- summary 可入库，但必须带 suite digest 和 non-claim。

---

# 14. 性能指标

## 14.1 公式和建议目标

| 指标 | 计算 | 建议目标 |
|---|---|---|
| Task Completion Rate | completed tasks / started tasks | baseline 后制定 |
| Verified Completion Rate | accepted with all criteria / started | B04 target ≥80%，视任务难度分层 |
| Partial Completion Rate | partial / terminal tasks | 报告，不隐藏 |
| False Completion Rate | claimed complete but verifier fails / claimed complete | gate suite = 0 |
| Recovery Completion Rate | recovered and accepted / recoverable interrupted | ≥95%；普通 daemon restart state恢复=100% |
| Human Intervention Rate | tasks requiring unplanned human action / started | 先测，不盲降 |
| Approval Interactions/Task | default-path 人工确认次数 / started | ≤1（准入预览；Tier-2 与首用授予除外，ADR-0026） |
| Total Tokens/Completed Task | total provider tokens / verified completed | 相对 baseline下降≥20%±5% |
| Repeated Context Ratio | duplicated input tokens / total input tokens | 相对下降≥25%±5% |
| Tokens/Progress Point | total tokens / accepted progress facts | 相对下降≥20% |
| Tokens/Verified Criterion | total tokens / passed criteria | 主质量指标之一 |
| Tool Calls/Completed Task | tool calls / verified completed | 分任务报告 |
| Redundant Tool Call Rate | calls with no new evidence/progress / calls | ≤10% |
| Tool Failure Rate | failed calls / calls | 依工具分层 |
| Tool Cache Hit Rate | valid cache hits / eligible reads | 报告，不追求虚高 |
| Tool Result Utilization | result referenced in next decision/evidence / calls | ≥70% |
| No-progress Step Ratio | no-progress iterations / all iterations | 相对下降≥30%±5% |
| Repeated Action Rate | same action-target-error repeats / steps | ≤5% |
| Strategy Switch Success | post-switch产生progress / switches | ≥50%初始建议 |
| Memory Precision | relevant retrieved / retrieved | ≥0.85±0.05 |
| Memory Recall | relevant retrieved / relevant corpus | ≥0.75±0.05，按类型分层 |
| Stale Memory Rate | stale used / retrieved used | ≤2% |
| Write Amplification | persisted derived bytes / source bytes | 设上限，Embedding实验单列 |
| Memory Token ROI | avoided/relevant tokens / retrieval+maintenance tokens | >1 |
| Parallel Speedup | single wall time / multi wall time | ≥1.25x才默认启用该场景 |
| Coordination Overhead | coordination tokens / total multi tokens | ≤35% |
| Clean Install Success | successful clean installs / attempts | ≥90%，目标提高到95% |
| TTFC | install start→first valid response | p95≤10 min建议目标 |
| Pi Startup Success | successful governed startup / attempts | ≥95% |
| Upgrade Success | healthy new version / valid upgrade attempts | ≥95% |
| Uninstall Cleanliness | expected removed and retained policy correct / runs | 100% gate fixtures |

在测量前，上述均为**建议目标**，不得写成已达成。

---

# 15. 测试策略

## 15.1 测试层

1. **合同测试**：schema、generated bindings、golden、API compatibility；
2. **纯逻辑单元测试**：admission、budget、loop、selection、redaction；
3. **SQLite 集成测试**：migration、CAS、lease、restart、WAL、backup；
4. **Provider contract tests**：recorded fixture + live opt-in；
5. **Pi contract tests**：Extension events、RPC framing、version mismatch；
6. **Security negatives**：secret leak、unauthorized body、tool bypass、path escape；
7. **Fault injection**：before/mid/after dispatch、daemon kill、disk failure；
8. **E2E**：B01-B12；
9. **Performance campaign**：固定环境、重复次数、CI非阻塞趋势和release blocking threshold分离；
10. **Wrong-implementation self-check**：每个关键 gate 至少一个故意错误实现必须 fail。

## 15.2 影响面验证矩阵

每批先声明 impact class，不再要求无关生态的全量命令作为 commit 前置：

- **Rust implementation-only:** affected Rust tests、focused integration/security negatives、affected Clippy、`cargo fmt --all -- --check`、`git diff --check`；
- **TS implementation-only:** affected package build/test/lint、`git diff --check`；
- **contract-affecting:** consistency、matrix、codegen、golden、conformance 与受影响双端测试；
- **cross-cutting/release:** 在受支持本地环境运行 full workspace；所有 code PR 在 merge 前仍须 protected CI 全绿；
- **docs-only governance/corrective:** Markdown links/consistency、`git diff --check` 与针对性状态核对。

提交前执行受影响检查；push 前执行可用的相关广域检查；merge/任务完成前执行 required CI。不可用或明确无影响的检查记录为 `not-run`/`not-affected` 及理由，不得虚报通过。根据实际影响追加 workspace、conformance、security、fault、E2E。

---

# 16. Risk Register

| ID | 风险 | 概率/影响 | 触发信号 | 缓解/应急 | 模块/任务 |
|---|---|---|---|---|---|
| R-01 | Pi 上游快速变化 | 高/高 | Extension/RPC fixture破坏 | exact pin、compat suite；回退上版 | Pi；P0-T06/P1-T07 |
| R-02 | SDK/RPC不兼容 | 中/高 | framing/event变化 | Interactive Extension主路径；RPC延后 | Pi；P6 |
| R-03 | Rust/Node边界复杂 | 中/高 | 双配置/双secret/双session | daemon拥有状态，Node仅UI | P1 |
| R-04 | DeepSeek模型/alias变化 | 高/中 | observed model不同/下架 | exact IDs、TTL snapshot、重新probe | P1-T03 |
| R-05 | Provider tool差异 | 高/高 | JSON合法但语义失败 | active probes、candidate validation | P1/P2 |
| R-06 | API Key泄漏 | 中/致命 | log/env/config命中 | native store、redaction、leak tests；立即rotate | P0-T05/P1-T02 |
| R-07 | Pi Bash绕过 | 高/致命 | built-in tool仍可调用 | governed mode禁用/替换；启动拒绝 | P1-T07 |
| R-08 | 两套Task状态 | 中/致命 | Pi session显示completed而DB非完成 | Session≠Task；只渲染projection | P2 |
| R-09 | 两套Memory | 中/高 | Pi summary和DB冲突 | Pi summary仅presentation | P3/P4 |
| R-10 | Session/Task混淆 | 高/高 | resume session被写成resume task | 明确mapping和negative tests | P1/P2 |
| R-11 | Context压缩丢约束 | 中/高 | required fragment消失 | required fail-closed、loss evidence | P3 |
| R-12 | Memory污染 | 中/高 | 未验证文本成为procedural fact | write admission/provenance/confidence | P4 |
| R-13 | 向量检索低精度 | 中/中 | precision低于FTS | embedding shadow gate；默认关闭 | P4-T06 |
| R-14 | Agent供应链 | 高/高 | floating ref/script/tamper | exact digest/attestation/quarantine | P5 |
| R-15 | Tool供应链 | 高/致命 | descriptor与binary不一致 | qualification digest/sandbox/health | P5 |
| R-16 | 后台孤儿进程 | 中/高 | daemon重启后进程存活 | epoch identity、reconcile、kill policy | P2-T06 |
| R-17 | Unknown Outcome | 高/高 | timeout无最终状态 | stable key、query/reconcile/quarantine | P2/B12 |
| R-18 | SQLite并发 | 中/高 | busy/duplicate writer | single daemon writer、lease、busy budget | P1/P2 |
| R-19 | 工作区越界 | 中/致命 | symlink/path escape | canonical path、root binding、negative tests | P2/P6 |
| R-20 | 多 Agent写冲突 | 高/高 | 同路径并行修改 | path ownership/worktree/integrator | P6 |
| R-21 | Token失控 | 高/中 | no-progress/context重复 | budget、metrics、loop guard | P3 |
| R-22 | 企业模块阻塞个人版 | 中/高 |每次本地动作要求复杂approval | ADR-0026 trust profile（Tier 0/1/2、准入预览唯一默认授权点、预算硬轨）、保留边界旁路 | 全阶段 |
| R-23 | 过度工程 | 高/高 | 引入Temporal/vector DB/K8s | phase gates、依赖引入需ADR amendment | 全阶段 |
| R-24 | 测试只依赖Mock | 高/高 | live/E2E缺失 | recorded+live opt-in+VM E2E | P1/P7 |
| R-25 | 跨平台安装差异 | 高/中 | Windows/macOS行为未知 | 首发只声明Linux；单独PoC | P7 |
| R-26 | 根License未决 | 中/致命 | 无发行授权 | P0-T03 release NO-GO | P0/P7 |
| R-27 | Headless Secret backend缺失 | 高/高 | Secret Service unavailable | 限定desktop；不提供明文fallback | P0/P1 |
| R-28 | HTTP loopback被本地恶意进程调用 | 中/高 | token复用/channel混淆 | short-lived channel-bound auth、socket perms | P0-T07/P1-T04 |

---

# 17. Deferred Backlog

| 内容 | 当前代码 | Personal处理 | 保留边界/恢复条件 | 当前禁区 |
|---|---|---|---|---|
| 多租户 | 规范/部分治理存在 | 默认单 owner旁路 | tenant字段不删除；企业Profile重启 | 不重写authz |
| 企业 RBAC | 部分 capability/authz | 不进入默认UX | 保留server gate | 不建管理员UI |
| 完整 ActorChain | domain能力存在 | local owner最小链 | 不删除类型 | 不扩展Personal流程 |
| Capability Intersection | 已有基础 | delegation前不暴露 | P6才消费 | P1-P5不重做 |
| 企业审批 | management基础 | 仅destructive confirmation（=ADR-0026 Tier 2） | 企业版另Profile | 不建审批链（ADR-0026 正式化） |
| 高保障 Profile | specified/局部实现 | non-claim | 全MUST证据后恢复 | 不宣称implemented |
| 全量 Conformance | 60/25当前口径 | Personal只跑相关+全回归 | 后续campaign | 不改向量迎合实现 |
| Kubernetes | 无 | 延后 | 多节点需求实测后 | 不引入 |
| 多节点/HA | 无 | 延后 | 单机瓶颈/可靠性需求后 | 不建分布式lease |
| 企业 Console | docs-only gate blocked | 延后 | client readiness通过 | 不在P0-P6写UI |
| 手机客户端 | docs-only | 延后 | daemon API稳定+平台gate | 不创建manifest/src |
| 复杂插件市场 | 无 | 延后 | package安全成熟 | 不自动发现 |
| Agent市场自动发现 | 无 | 延后 | verified publisher ecosystem | 不做推荐安装 |
| 默认云同步 | 无 | 延后 | privacy/encryption/conflict ADR | 不上传Memory |
| 企业 SSO | 无 | 延后 | 多用户产品目标 | 不阻塞local auth |
| 法务治理 | docs tracking | release前只做License/privacy | 企业版恢复 | 不伪造法律结论 |

---

# 18. Execution Contract

## 18.1 每个后续会话开始前

1. 读取 `AGENTS.md`、项目身份和 Development Operating Model；
2. 读取 Personal 正式计划；
3. 只读 `PROGRESS.md` 的 `Current snapshot`；
4. 只读 `PARALLEL-LANES.md` 的 active leases；
5. 读取所选任务的最新 matching handoff 与本文件对应 Task Card；
6. 验证 `implementation_requires` 可用；不得把 `acceptance_requires` 或
   `promotion_requires` 当作 isolated implementation mutex；
7. 确认无路径重叠并领取 exact-path lease；
8. 记录 base commit、branch、dirty state；
9. 运行或诚实记录该卡要求的 baseline tests；
10. 明确复述“不包含内容”和唯一允许修改路径；
11. 定位真实 REQ/schema/vector；
12. 若需越界，停止，不编码。

## 18.2 每个会话完成后

1. 运行任务指定 tests；
2. 运行相关回归；
3. 按真实验收结果更新正式 Task 状态；
4. 记录实际修改文件；
5. 记录测试命令、exit、关键输出；
6. 记录实际 public/internal/data/config变化；
7. 记录偏差和未决项；
8. 更新风险；
9. 只更新 PROGRESS Current snapshot 中的当前事实；
10. 写 matching handoff 并关闭或移交 lease；
11. 完成且全绿后准备 reviewable delivery；仅在明确要求或批准的 PR 流程中提交/push；
12. 默认不自动进入下一 Task。

## 18.3 Plan Amendment 触发条件

出现任一情况必须停止：

- 修改计划外核心模块；
- 现状证据错误；
- Pi固定版本API不兼容；
- 需要新对象族/REQ域/Profile；
- 数据模型需要跨阶段大改；
- 需引入Temporal/Postgres/Redis/vector DB等重基础设施；
- 性能目标不现实；
- Secret/Provider关键依赖不可用；
- 任务修改文件超过卡片范围；
- 阶段Gate无法满足。

Amendment 格式：

```text
Plan Amendment
原因:
新证据:
受影响任务:
候选修改:
风险:
是否改变产品目标:
建议决策:
批准人/状态:
```

## 18.4 Implementation Record

```text
Implementation Record
Task ID:
开始 Commit:
结束 Commit:
实际修改:
计划偏差:
测试命令:
测试结果:
性能结果:
Evidence refs:
遗留问题:
已知风险:
下一任务前提:
```

## 18.5 Handoff Record

```text
Handoff Record
当前状态:
已完成:
未完成:
关键决策:
关键文件:
运行命令:
失败命令:
恢复步骤:
下一步:
禁止重复尝试:
```

---

# 19. 安装目标和验收规范

## 19.1 首先支持的安装方式

选择：

```bash
curl -fsSL <official-install-url> -o install.sh
less install.sh
sh install.sh
```

但脚本只负责：

- 检查平台；
- 下载固定 release manifest；
- 验证 digest/attestation；
- 下载 Linux x86_64 bundle；
- stage；
- health check；
- atomic activate；
- 安装 user-level service。

不选择 Homebrew 作为首发，因为首发不是 macOS。
不选择 Docker Compose 作为首发，因为它使 Pi Interactive TUI、Secret Service、用户工作区和本地工具隔离更复杂；Compose 可作为后续 server/headless候选，但不能替代本地产品链路。

Windows x86_64 是首发产品平台（ADR-0025），但其安装面（credential 后端、installer/service、专门 B01-W Gate）统一延后到 P7-T07；本节安装方式仅覆盖 Linux bundle。

## 19.2 `cognitive init` 目标流程

```text
platform/toolchain check
→ XDG directories
→ migration preflight
→ Secret Service probe
→ hidden DeepSeek Key input
→ save opaque secret_ref
→ validate HTTPS endpoint
→ GET /models
→ active chat/stream/tool/cancel probes
→ exact model selection
→ persist capability snapshot
→ daemon start
→ pinned Pi verification
→ CognitiveOS Extension load
→ doctor
→ first conversation
```

失败必须保留可继续恢复的状态，不删除既有数据，不回显 Key。

## 19.3 启动

```bash
cognitive
```

应：

1. 查询/启动 daemon；
2. 验证 Provider snapshot未过期；
3. 验证固定 Pi版本和Package digest；
4. 加载 Extension；
5. 恢复Pi presentation session；
6. 从daemon查询最近Task projection；
7. 打开Pi Interactive TUI。

恢复 Pi Session 不等于恢复 Task；Task恢复由 daemon recovery flow完成。

## 19.4 更新

```text
verify release
→ stage binaries/Package
→ backup config/database metadata
→ migration preflight
→ stop accepting new work
→ reconcile/fence
→ switch
→ health/B01-smoke
→ commit update
```

失败即切回旧版本和兼容数据库备份。

## 19.5 卸载

```bash
cognitive uninstall
```

默认：

- 删除 binary、service、runtime socket和cache；
- 保留 data/config/history；
- 删除 Provider secret必须单独确认；
- `--purge-data` 必须二次确认并显示路径；
- 保留可审计 uninstall result，不保留 secret。

---

# 20. 最终结论

当前 CognitiveOS 最有价值的资产是确定性的 kernel、store、Intent/Effect、Context、TaskContract 和 recovery，而不是当前 app surface。Personal 计划必须围绕这些资产建立一个单 daemon 的产品闭环，而不是把 Pi 或新的 agent framework 放到 authority 中心。

最短正确路径是：

```text
迁移和Secret
→ Provider
→ daemon
→ CLI
→ Pi Extension
→ B01
→ Task Service
→ scheduler
→ 一个受限工具
→ verifier/recovery
→ B02/B04/B05/B12
→ Context效率
→ Memory
→ Agent/Tool生态
→ 有收益证据后Multi-Agent
→ release hardening
```

在以下条件完成前，不得声称 CognitiveOS Personal 已可用：

- B01 clean install真实通过；
- Pi governed mode无直接 mutating tool；
- API Key leak tests全负；
- daemon唯一writer；
- Task/Session分离；
- external operation走Intent/Effect；
- B05 restart recovery通过；
- B12 unknown outcome不盲重试；
- false-completion negative suite为0；
- release artifact可验证、可回滚；
- 所有性能提升有固定baseline和统计证据。
