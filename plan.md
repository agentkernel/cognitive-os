# CognitiveOS Personal 产品化研究与任务卡草案

> **文档状态：研究与任务卡草案；不代表实现已提供、测试已执行或 Profile 已符合。**
> **正式开发计划与进度台账：** [docs/plan/PERSONAL-DEVELOPMENT-PLAN.md](docs/plan/PERSONAL-DEVELOPMENT-PLAN.md)。后续开发完成任一部分时，必须更新该文件对应任务的状态、日期和证据。
> **研究与审计日期：2026-07-24。**
> **审计基线：`origin/main@9b53cf4c6c2b744a60283c3ea1431a9d1090aafd`。**
> **本草案不包含生产代码、规范或数据库 Schema 变更。**
> **落盘说明：** 正式计划与进度台账已保存于 `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`；本文件保留研究结论、详细任务卡和原始审计材料。

---

## 1. 文档层级与使用方式

`docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` 是 Personal 开发的唯一正式入口和进度台账：任务 ID 的名称、范围、依赖、状态和阶段 Gate 以该文件为准。

本文件保留研究结论、详细任务卡、ADR 候选和机器可读依赖图。开发会话领取任务时，应先在正式台账确认对应 `P*-T*` 的定义与状态，再阅读本文件中相同 ID 的实施细节；不得以本文件重新定义、交换或复用任务 ID。未来若需拆分本文件，必须先由 Lane-DOC 批准目录和迁移方案，且不得影响正式台账的 canonical 地位。

---

# 2. 执行摘要

## 2.1 当前真实状态

CognitiveOS 当前不是 Personal 产品，而是一个具有较强确定性内核、规范合同和符合性基础的参考实现：

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
→ Secret Service 凭据后端
→ DeepSeek/OpenAI-compatible Provider 探测
→ 单一 Rust daemon
→ cognitive init / doctor / status
→ Pi 官方交互式 CLI + CognitiveOS Extension
→ 首次对话
→ TaskApplicationService
→ durable scheduler
→ 一个受限工具
→ checkpoint/recovery
→ independent verifier
→ B01/B02/B04/B05/B12
```

## 2.3 关键决策

1. **默认 Pi 集成不是 SDK，也不是 RPC。**
   - 默认交互路径选择官方支持的 **Pi Interactive CLI + 固定版本 CognitiveOS Package/Extension**。
   - Pi RPC 留给后续受监督的后台 Agent。
   - Pi SDK 只在未来确实需要自建 UI 且有测量收益时重评。

2. **daemon 使用 Rust。**
   - 复用现有 kernel/runtime/store/management；
   - daemon 是唯一 authority composition、scheduler 和 SQLite writer。

3. **SQLite 继续使用，但先补迁移和运维体系。**
   - 不引入 Temporal、PostgreSQL、Redis 或消息队列作为 Personal v1 前置条件。

4. **Provider Secret 只保存在原生 Secret Store。**
   - Pi、配置文件、SQLite、环境继承、命令参数和日志不得持有原始 API Key。

5. **MCP 只是工具传输适配器。**
   - MCP discovery、connection 和 protocol completion 都不等于 CognitiveOS 授权、Effect 提交或 Task 完成。

6. **Memory 首先采用 SQLite source-of-record + FTS。**
   - Embedding 是 Phase 4 后置实验，不是 P0/P1 依赖。

7. **Multi-Agent 不进入首个 Personal v1 关键路径。**
   - 只有单 Agent B01-B10 稳定后才能进入 Phase 6。

## 2.4 阶段数量

共八个阶段：

- Phase 0：研究、基线和决策；
- Phase 1：安装到首次对话；
- Phase 2：单 Agent 任务闭环；
- Phase 3：Context、Token 和 Loop 效率；
- Phase 4：Memory；
- Phase 5：Agent 与 Tool 生态；
- Phase 6：Multi-Agent；
- Phase 7：产品化和发布。

## 2.5 最大风险

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
| 服务入口 | `apps/kernel-server/src/main.rs` |
| 管理 CLI | `apps/admin-cli/src/main.rs` |
| Pi 候选适配器 | `apps/pi-agent-adapter/src/main.rs` |
| TS Shell | library，无 `bin` |
| CI | `.github/workflows/ci.yml`，Windows/Linux |
| License | **Apache-2.0**（P0-T03 / ADR-0025）；Rust `publish=false`；TS `private: true`；Pi/Node 不 vendor |
| 部署 | 无 Dockerfile/Compose、安装器、Homebrew、OS service、release workflow |
| 迁移 | 无版本化 SQL migrations 框架 |

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
| 安装 | L0 | L5 | 无 deploy/install/release | 无 bundle、verifier、rollback | P1-T08、P7-T01/T02 |
| 初始化 | L0 | L5 | `admin-cli` 无 init | 数据布局、Secret、Provider、daemon、Pi | P1-T01..T09 |
| CLI | L3 管理型 | L5 产品型 | `apps/admin-cli/src/main.rs` | 缺共享 Personal application service | P1-T05/T06 |
| daemon | L2 | L5 | `apps/kernel-server/src/main.rs` | 手写无界 parser、无 auth、canned routes | P1-T04 |
| Pi 集成 | L1/L2 | L5 Interactive | `apps/pi-agent-adapter/src/main.rs` | evaluator 与 admission 未连接 | P0-T06、P1-T07 |
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
| Web UI | L0 code/L1 docs | L4 later | `clients/**` docs only | gate、PoC、ADR、真实 API | P7-T05 |
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

---

# 8. 目标架构

```text
                    ┌──────────────────────────┐
                    │ Pi Interactive TUI       │
                    │ CognitiveOS Extension    │
                    │ Session = presentation   │
                    └────────────┬─────────────┘
                                 │ authenticated loopback
       ┌─────────────────────────┴──────────────────────────┐
       │            CognitiveOS Personal Daemon             │
       │                                                    │
       │  Readiness / Config / Provider / Task Application  │
       │  Scheduler / Loop / Context / Memory / Tool Router │
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

# 9. ADR 决策集

所有 ADR 都必须包含：状态、背景、仓库约束、候选、评价维度、决策、理由、后果、迁移成本、回滚和重评条件。

| ADR | 候选与决策 | 主要后果、回滚与重评条件 |
|---|---|---|
| 001 Pi 集成 | SDK / RPC / Interactive Extension；**选择 Interactive Extension** | 保留 Pi TUI；RPC 留给后台；若 Extension API 无法稳定固定或无法禁用 mutating tools，停止并重评 RPC/custom UI |
| 002 daemon 语言 | Rust / Node / Python；**Rust** | 直接复用 kernel/store；Node 只留 Pi 侧；若 OS/API ecosystem 阻塞，可新增窄 sidecar，不迁 authority |
| 003 SQLite | SQLite / Postgres / workflow DB；**SQLite 单 writer + migrations** | 适合本地；若写争用、恢复或数据量实测超预算再重评 |
| 004 Artifact Store | SQLite blob / filesystem CAS / object store；**filesystem CAS + SQLite metadata** | 大文件不塞主 DB；可通过 digest 重建引用 |
| 005 Secret Store | Pi auth.json / encrypted config / native store；**native Secret Store** | 首发依赖 Linux Secret Service PoC；headless 无可靠 backend 则不支持 |
| 006 Provider | Pi-owned / daemon OpenAI-compatible / vendor SDK；**daemon OpenAI-compatible** | DeepSeek 默认只是配置，不硬编码模型；vendor 差异放 adapter |
| 007 Task 状态机 | 新工作流 / Pi Session / 现有 transitions；**复用现有 Task state machine** | 不新增第二套 Task |
| 008 Event/Snapshot | snapshot-only / event-only / event+projection；**authority event + rebuildable projections** | snapshot 不成为独立 authority |
| 009 Memory | vector-first / files / SQLite+FTS；**SQLite source + FTS** | Embeddings 仅派生索引 |
| 010 Embedding | P0 / Phase 4 experiment / never；**Phase 4 实验 gate** | 未证明收益不得进入默认路径 |
| 011 MCP | direct MCP / MCP authority / adapter；**Tool Registry adapter** | MCP auth 与 CognitiveOS operation auth 分离 |
| 012 Process | Pi-owned / OS service per task / daemon supervisor；**daemon supervisor** | 进程必须有 task/attempt/epoch identity |
| 013 Agent Package | raw Pi package / arbitrary git / CognitiveOS manifest；**digest-pinned CognitiveOS manifest** | Pi package 可作为 payload，但 activation 由 daemon 决定 |
| 014 Tool Package | MCP discovery / arbitrary executable / qualified manifest；**qualified manifest** | schema、risk、sandbox、reconcile、health evidence 必须齐全 |
| 015 Blackboard | shared chat / mutable KV / append-only findings；**append-only non-authority findings** | 黑板消息不能提交状态 |
| 016 Git Worktree | shared cwd / mandatory worktree / selective worktree；**仅写任务选择性使用** | read-only research不强制；冲突时取消/重新派发 |
| 017 Intent-first | 仅高风险 / 所有 mutating external operations；**后者** | read-only 可短路径，但仍审计和 scope-check |
| 018 Web UI | Next/React/Vite/Tauri；**暂定 React+TS+Vite 静态客户端，gate 后重评** | 不进入 P0-P6；只能调用 daemon API |
| 019 安装更新 | Homebrew / Compose / attested bundle；**Linux attested bundle + inspectable script** | Brew/macOS 和 Compose 不首发；失败原子回滚 |

---

# 10. 阶段路线图和门禁

| Phase | Entry | Exit | Blocking tests/evidence | Rollback point | 禁止提前开发 |
|---|---|---|---|---|---|
| 0 基线 | 当前计划批准 | toolchain、ADR、platform、Secret/Pi PoC、benchmark spec 完成 | CI、Linux runner、PoC reports、plan consistency | `main@9b53cf4` | 功能实现、Memory/Multi-Agent/UI |
| 1 首次对话 | P0 全绿 | B01 在干净 Linux VM 真实通过 | install/init/secret/provider/daemon/Pi E2E；secret leak negatives | 未初始化快照/旧 binary | Task autonomy、Memory、MCP、多 Agent |
| 2 单 Agent 闭环 | B01 pass | B02/B04/B05/B12 pass | task/scheduler/tool/process/recovery/verification | Phase 1 release | Memory、embedding、多 Agent |
| 3 Context/效率 | Phase 2 completion 稳定 | B03/B06/B07 pass；指标采集完整 | context loss/revocation/cache/loop negative tests | Phase 2 release | Memory consolidation、多 Agent |
| 4 Memory | Phase 3 baseline frozen | B08 pass；FTS baseline；embedding go/no-go | provenance/freshness/conflict/forget/privacy | 无 Memory 的 Phase 3 | 自动跨工作区记忆 |
| 5 生态 | Phase 4 stable | B09/B10 pass | package verification、health、disable/uninstall、MCP negative | 内置 agent/tool set | 市场自动发现 |
| 6 Multi-Agent | 单 Agent benchmark稳定 | B11 相对 baseline 有收益且无权限/写冲突 | isolation/budget/cancel/merge/reviewer | 默认单 Agent | 无证据默认启用 |
| 7 产品化 | B01-B12 功能证据齐全 | attested RC、upgrade/uninstall、docs、support matrix | full CI + benchmark campaign + release checklist | 最近健康 release | 企业 Console、五平台客户端 |

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

---

## Phase 0

### P0-T01 — 固定可复现基线与支持工具链

- **优先级/目标/价值：** P0；让后续会话可复现 Rust、TS、CI 基线。
- **证据/研究：** 本机 LLVM-MinGW 缺 `libgcc`；远端双 OS CI green。2026-07-25 的正式台账记录确认：`01ceb93` 的 CI run 30140381194 在 Ubuntu 与 Windows/MSVC 均通过；本机 Windows GNU 在默认和已记录的 LLVM-MinGW/shim 处置下均于 linker exit 121 失败，故 GNU host 是非支持开发环境。详见 `tests/baseline/README.md`。
- **依赖/不包含：** 无；不改生产逻辑。
- **文件：** 修改 `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`、本研究报告及 CI/toolchain docs；计划新增 `tests/baseline/README.md`；不删文件。
- **数据/API/配置/迁移：** 无数据/API；明确 Linux runner、Windows GNU/MSVC 支持组合。
- **步骤：** 重跑 Git 状态、两套工具链、全部 baseline commands；记录 SHA、exit、耗时。
- **验收：** Linux clean runner 全绿；Windows 选择官方支持 linker 后全绿或明确 non-supported。
- **基准/性能：** 记录 build/test p50，不设改进目标。
- **安全/观测/回滚：** 不读取 secret；只生成 ignored artifacts；失败即恢复环境，不改源码。
- **解锁/风险/不确定：** 解锁全部任务；Windows GNU 是否继续支持需在此关闭。

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

- **状态：** **done**（2026-07-26 owner GO；正式台账见 `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`）。
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
- **证据：** `crates/cognitive-store/src/sqlite.rs`、`installation.rs` 无 schema version。
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
- **不确定：** headless Linux；未解决则首发限定 desktop user session。
- **解锁：** P1-T02。

### P0-T06 — Pi 版本、Extension 与 RPC 兼容性 PoC

- **目标：** 固定 Pi 版本、integrity、source commit、Extension API 和 RPC JSONL fixture。
- **证据：** registry 0.82.0；仓库 smoke 使用 0.81.1；API 快速变化。
- **依赖：** P0-T03；不启动 governed background Agent。
- **文件：** 修改 `apps/pi-agent-adapter` tests/docs；计划新增 `tests/golden/pi-rpc/` 和 Extension PoC。
- **API：** 固定 Extension command/provider/event subset；RPC 只做 contract fixture。
- **验收：** project trust、tool replacement、session event、strict LF framing、version mismatch 均有测试。
- **安全：** Extension 无 DB/secret；built-in write/edit/bash 在 governed mode 不可用。
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

---

## Phase 1 — 安装到首次对话

### P1-T01 — 版本化数据库迁移与 XDG 布局

- **目标：** 初始化 config/data/state/cache/runtime，安全迁移两套 SQLite。
- **证据：** inline schema、无 migration framework。
- **依赖：** P0-T04。
- **文件：** 修改 `crates/cognitive-store/src/{sqlite,installation,lib}.rs`；新增 `layout.rs`、`personal_db.rs`、`tests/p1_t01_layout_migrations.rs`；ADR-0017 补记 XDG 实现。
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
- **状态：** done（CI run 30156079691 Ubuntu/Windows-MSVC 全绿）。
- **文件：** 扩展 `crates/cognitive-secret`（ProviderConfig、ProviderKeyService、LinuxSecretToolStore、backend_select、secret_input、tests/p1_t02_provider_secret.rs）；ADR-0020；未改 management/runtime（避免 Lane-RUN 所有权冲突）。
- **数据：** Provider config 只存 provider、base URL、secret_ref、selected snapshot digest。
- **API：** put/rotate/delete/probe via ProviderKeyService；hidden-input helper `read_secret_material_from_reader`（CLI echo-off 归 P1-T06）。
- **验收：** rotation、locked store、deleted secret、daemon restart、redaction negatives（CI p1_t02_provider_secret 通过）。
- **不包含：** cloud secret manager、明文 fallback、Pi auth.json、真实 Provider Key、G0/Profile。
- **回滚：** 删除 ref；不自动删除用户未确认的数据。
- **解锁：** P1-T03/P1-T06。
### P1-T03 — OpenAI-compatible Provider、模型发现与能力快照
### P1-T03 — OpenAI-compatible Provider、模型发现与能力快照

- **目标：** DeepSeek 默认初始化，但模型 ID 动态发现和主动验证。
- **状态：** done（本地 typecheck/clippy 通过；行为测试由 CI Ubuntu/Windows-MSVC 执行 `p1_t03_provider_discovery`）。
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
- **状态：** in-progress（PR #95 已合入，CI Ubuntu/Windows-MSVC 已执行现有行为测试；timeout 与 concurrency 行为测试尚未提供，P1-T04 未完成）。
- **文件：** `apps/kernel-server/src/personal/{mod,auth,bounds,lifecycle,server}.rs`、`main.rs --personal`、`tests/p1_t04_personal_daemon.rs`；layout daemon 路径；ADR-0022。
- **配置：** loopback-only bind、ADR-0019 body/header/concurrency bounds、single-instance `daemon.lock`、runtime bootstrap secret。
- **API：** `POST /local/session`；channel-scoped bearer on `/management/*` and `/task/*`；`GET /personal/health`（non-claim）。
- **验收：** oversized body、bad auth、wrong channel、cookie/host reject、second-instance lock、restart。
- **不包含：** Task scheduler、Memory、MCP、full readiness projection（P1-T05）、UDS product default path（design remains ADR-0019）。
- **回滚：** 未认证/超限请求 fail-closed；无 authority mutation from this front door.
- **解锁：** P1-T05/T06/T07。

### P1-T05 — Readiness、status 和 doctor 应用服务

- **目标：** CLI、Pi、未来 UI 共用同一事实源。
- **状态：** done（CI Ubuntu/Windows-MSVC SUCCESS：30164114878 / 30164113787）。
- **文件：** `apps/kernel-server/src/personal/readiness.rs`、`server.rs` 路由、`tests/p1_t05_personal_readiness.rs`、ADR-0023。**未**修改 `cognitive-management`（Lane-RUN 所有权；Personal 组合根承载 projection）。
- **API：** management-channel `GET /personal/status`、`GET /personal/readiness`、`GET /personal/doctor`；组件 system/database/secret/provider/daemon/pi；返回事实、duration、source、error_class 与 non-claim。
- **验收：** degraded/blocked/ready 分离；静态检查通过不写成 runtime ready（`static_check_is_not_runtime_ready`）；secret_ref/bootstrap 不入投影。
- **观测：** 每项 check duration/error class；doctor guidance 可操作。
- **不包含：** CLI 产品入口（P1-T06）、Pi package（P1-T07）、G0/B01-B12/Profile、registry/schema/vector。
- **解锁：** P1-T06/T07/T09。

### P1-T06 — `cognitive init/doctor/status/daemon`

- **状态（2026-07-25）：** done；PR #98 / `main@adbb0e5`；CI 30167503487 Ubuntu/Windows-MSVC green；ADR-0024；非 G0/Profile。
- **目标：** 将 `admin-cli` 演进为 `cognitive` 产品入口。
- **文件：** 修改 `apps/admin-cli/Cargo.toml`、`src/main.rs`、tests；计划添加 `cognitive` bin，保留 `admin-cli` 兼容。
- **API：** 只调用 daemon/application service，不直接编排 SQLite。
- **步骤：** env check→dirs→secret→provider→model→daemon→Pi check→self-test。
- **验收：** hidden input、URL修正、重试、手填 model、可操作错误、重复 init 幂等。
- **不包含：** Task/Memory/Tool management。
- **回滚：** 初始化失败清理临时状态，不删除既有数据/secret。
- **解锁：** P1-T09。

### P1-T07 — CognitiveOS Pi Package/Extension 与 Provider proxy

- **目标：** 复用 Pi TUI，实现 `cognitive` 首次会话。
- **文件：** 计划新增 `packages/pi-cognitiveos/`；修改 pnpm workspace/lock；新增 Extension tests。
- **API：** daemon-authenticated provider proxy、`/cognitive-status`、task placeholders；Session 仅 presentation metadata。
- **配置：** fixed Pi version/integrity/package path/project trust。
- **验收：** direct bash/write/edit 禁用；无 API Key/env/SQLite path；daemon unavailable 时明确失败。
- **性能：** Extension startup 建议 <2 s，首个 proxy token另测。
- **回滚：** disable/remove Extension 不影响 authority data。
- **解锁：** P1-T09、P2-T02。

### P1-T08 — 可检查 Linux bundle installer 与 user service

- **目标：** 支持 `curl -o install.sh; less; sh`，不是 `curl|sh`。
- **文件：** 新增 `deploy/linux/install.sh`、manifest、systemd user unit、uninstall skeleton；修改 CI release dry-run。
- **数据：** staged install state；不迁移用户数据直到 verifier通过。
- **步骤：** platform check→download manifest/artifacts→digest/attestation→stage→health→atomic switch。
- **验收：** tamper、interrupted download/install、no Node、wrong Pi integrity、existing version。
- **不包含：** Homebrew、Docker、root service、自动更新。
- **回滚：** 切回前一版本；保留数据备份。
- **解锁：** P1-T09；为 P7-T01/P7-T02 提供安装器输入。

### P1-T09 — B01 首次安装到首次对话 Gate

- **目标：** 干净 Linux VM 完成安装、init、DeepSeek、daemon、Pi、首个响应。
- **文件：** 新增 `tests/e2e/personal/b01-*` 和 evidence schema/runner。
- **验收：** 连续至少 20 次 clean-run；建议成功率 ≥90%，95% CI 同时报告；TTFC 建议 p95 ≤10 min（含人工 Key 输入但不含下载网络异常）。
- **失败条件：** Key 泄漏、工具未禁用、daemon synthetic ready、模型仅凭 `/models`、无法卸载临时安装。
- **证据：** logs、timestamps、versions、snapshot digests，绝不含 Key。
- **解锁：** Phase 2。

---

## Phase 2 — 单 Agent 任务闭环

### P2-T01 — TaskApplicationService

- 复用 `intent_chain.rs`、TaskContract、budgets；新增 proposal/clarify/preview/admit/control/query service。
- 修改 `cognitive-management`/`cognitive-runtime` ports；不得新增平行 Task 类型。
- 验收 raw intent 先持久化、preview digest 绑定、修订产生新 epoch 并 fence 旧任务。
- 不含 scheduler、Memory、多 Agent；解锁 P2-T02/T03。

### P2-T02 — 真实 Task API、watch 与自然语言管理映射

- 替换 canned proposal/attach/detach/cancel/watch routes；TS SDK和 Pi Extension消费同一 API。
- 修改 `kernel-server`、`packages/sdk-ts`、`apps/agent-shell`、Pi package；合同变化经 Lane-CTR。
- 验收 detach不cancel、watch resume/dedup、cancel只产生 authority request、错误 HTTP status 真实。
- 解锁 B02 和 P2-T04。

### P2-T03 — durable scheduler、lease 和 timer

- 新增 scheduler repository/service；持久化 runnable、lease owner/epoch、next eligible、attempt、cancel request。
- 数据迁移必须由 P1-T01框架执行。
- 测 worker crash、duplicate lease、clock shift、deadline/retry/step/cost ceiling。
- 不引入 Temporal/queue server；解锁 P2-T04/P2-T07。

### P2-T04 — 单 Agent worker 与 BoundedHarness 接入

- 连接 scheduler→TaskContract→Context→candidate→progress→LoopDriver。
- 每轮必须重新加载 contract/governance/lease；模型 self-report 不算 progress。
- 测 no-progress、strategy switch、wait-user、budget stop、stale lease。
- 不含 Memory/MCP/background Pi；解锁 P2-T05/T07。

### P2-T05 — Tool Registry 与第一个安全 operation

- 建立 immutable descriptor：schema digest、risk、effect class、query/idempotency、sandbox、verification、health、state。
- 第一个 operation 应选择可查询、可幂等、影响范围窄的 workspace-local操作；不得选择 generic Bash。
- 未注册、schema drift、disabled/quarantined、伪造 capability 均拒绝且 dispatch=0。
- 解锁 P2-T06。

### P2-T06 — Process supervisor 和首个 executor

- 新增 stable process/task/attempt/epoch identity、CWD、stdout/stderr cursor、timeout、stop、restart、reconcile。
- executor 必须 persist-before-dispatch；stdout zero exit 只作为 evidence。
- 测 crash before/mid/after dispatch、orphan、output limit、secret redaction、same-key/different-input。
- 解锁 P2-T07。

### P2-T07 — Checkpoint、Artifact/Evidence 和独立 Completion Verifier

- 把 checkpoint、effect closure、artifacts、criteria results、verification event 接入 task closure。
- verifier 与执行 agent分离；每个 criterion 记录 pass/fail/unknown/evidence digest。
- Partial completion 不得升级为 completed；remote done和receipt不够。
- recovery 顺序严格使用现有 `recovery.rs`。
- 解锁 P2-T08。

### P2-T08 — Phase 2 E2E Gate

- 自动化 B02、B04、B05、B12。
- 必须覆盖 Shell关闭、daemon关闭、OUTCOME_UNKNOWN、不盲重试、false completion negative。
- 建议普通重启 authority state recovery=100%；False Completion Rate 在 gate suite 中必须 0/所有故意错误案例。
- 解锁 Phase 3。

---

## Phase 3 — Context、Token 与 Loop 效率

### P3-T01 — Context source/retrieval port

- 为现有 `context.rs::resolve` 提供真实 workspace/task/evidence source。
- candidate references 先 scope-filter，正文授权后才交给 ranker。
- 测 revoked/out-of-scope/rank-before-auth/cache-key variants。
- 不含 Memory semantics；解锁 P3-T02。

### P3-T02 — 最小充分 Context Builder 与预算

- 建立 System/Shell/Task/Working/Evidence fragments、required/optional、dedup、freshness、token budget。
- Required fragment缺失或超预算 fail-closed；loss显式。
- 记录 fragment source digest、included/excluded reason、token estimate/actual。
- 解锁 P3-T03/T04。

### P3-T03 — Artifact Store 和 Context externalization

- filesystem CAS + SQLite metadata；限制 size、retention、access、content-type。
- 大日志/工具输出外部化，prompt只持摘要和引用。
- 测 digest mismatch、partial write、orphan GC、unauthorized fetch。
- 解锁 P3-T04。

### P3-T04 — Loop telemetry、progress 与 strategy controls

- 记录 model tokens/cache/latency/cost、tool calls/failures、progress points、retries、loop signatures。
- Loop detection比较 action+target+error+evidence digest；触发 switch/wait/block，不无限 retry。
- Tool result先结构化过滤，再摘要；cache key含工具/输入/治理/版本。Pi compaction仅压 presentation session；TaskContract/criteria/evidence不从 Pi summary恢复。
- 测 constraint loss、stale cache、revocation、required source removed、no-progress/repeat/strategy 控制可观测。
- Cardinality和日志体积有预算；不记录 secret/raw sensitive body。
- 解锁 P3-T06。

### P3-T05 — Benchmark harness 与性能基线

- 建立 raw run、稳定基线、CI 采集和 non-claim 报告；冻结 baseline，再比较 Context Builder/压缩/Loop Guard。
- 建议目标：Repeated Context Ratio 相对 baseline下降≥25%±5%；No-progress Step Ratio下降≥30%±5%；constraint retention=100% gate cases。
- 结果若无显著改善，保留安全功能但不声称性能提升。
- 解锁 P3-T06。

### P3-T06 — Phase 3 E2E Gate

- 执行 B03/B06/B07，验证 Context 正确性、预算/loss、缓存、telemetry 与策略控制，并保留 benchmark raw evidence。
- 仅在 Gate 通过且指标可采集后解锁 Phase 4；性能结果必须保留 non-claim。

---

## Phase 4 — Memory

### P4-T01 — Memory source model和repository

- 复用现有 Memory schema/adapter域；新增 source/version/content digest、scope、purpose、provenance、confidence、freshness、retention、tombstone。
- 预计修改 store/runtime，新增 migrations/tests。
- 不新增 vector DB、graph DB或跨 workspace recall。
- 解锁 P4-T02/T03。

### P4-T02 — SQLite FTS retrieval baseline

- 对允许索引的 Memory source建立 FTS；source row是权威，FTS可重建。
- 查询先 scope/purpose/pre-filter，再授权正文，再排序。
- 测 precision/recall corpus、stale index、delete/rebuild、latency。
- 解锁 P4-T04。

### P4-T03 — Write、更新、冲突、遗忘与retention policy

- user/task只能提出 memory mutation；确定性 policy admission。
- 更新生成版本；冲突显式；forget写 tombstone并失效派生索引。
- 敏感 Memory默认不外发 Provider；删除证据不含原文。
- 解锁 P4-T04/T05。

### P4-T04 — Embedding 实验 Gate

- 只做离线 shadow experiment，不进入 authority 或默认检索路径。
- 比较 FTS、embedding、hybrid 的 recall、precision、latency、storage、token ROI、删除传播。
- 采纳条件：关键 recall 提升≥10个百分点且 precision不降>3个百分点、p95满足预算、可完整失效。
- 不满足则关闭，Personal 默认继续 FTS；记录明确 go/no-go 后解锁 P4-T05。

### P4-T05 — Memory API、CLI/Pi projection 与 B08

- scheduler只处理已批准 retention/refresh/summary工作。
- 摘要保存 source refs、transform version、loss declaration，不能替代 required source。
- Retrieval结果通过现有 Context resolver。
- 支持 remember/list/explain/update/forget，通过同一 Memory service。
- B08覆盖保存、检索、实际使用、过期、冲突、删除、不可恢复展示。
- 建议：Memory Precision ≥0.85±0.05；Stale Memory Rate ≤2%；Memory Token ROI >1，均须先测 baseline。
- 不含自动从全部聊天抽取永久记忆；解锁 P4-T06。

### P4-T06 — Phase 4 E2E Gate

- 执行 Memory 回归与 benchmark，保留 B08 的 provenance、freshness、conflict、forget/privacy 证据。
- 通过后解锁 Phase 5；Embedding 结论只按 P4-T04 的明确 go/no-go 进入后续范围。

---

## Phase 5 — Agent 和 Tool 生态

### P5-T01 — Agent package manifest 与安装生命周期

- 复用 installation store；补 activation/disable/update/uninstall/rollback lifecycle。
- source/lockfile/manifest/compatibility/sandbox digests持久化。
- 卸载保留历史 evidence，不保留运行权限。
- 测浮动依赖、lifecycle scripts、tamper、interrupted update。
- 解锁 P5-T02/P5-T05。

### P5-T02 — Agent registry 与 instance lifecycle

- 定义 versioned Agent Definition、实例健康、budget、tool scope、memory policy、workspace scope。
- 不把 Pi Session 当 AgentInstance；安装不等于 activation。
- 内置 Agent只做最小 general/reviewer/verifier集合；capability 默认拒绝并验证实例隔离。
- 解锁 P5-T05。

### P5-T03 — Tool package格式和MCP adapter

- Tool package manifest固定 schemas、operation descriptors、transport、risk、health、reconcile。
- MCP initialize/capability/version/timeout只建立 transport。
- server tool list变化生成候选，需重新 qualification，不自动启用。
- 测 malicious MCP、schema drift、prompt injection、direct endpoint bypass。
- 解锁 P5-T04/T05。

### P5-T04 — Dynamic exposure、composite tools、cache和health

- 每轮只暴露 TaskContract允许且当前健康的最小 tool集合。
- Composite Tool必须保留子操作Intent/Effect/evidence，不可隐藏 unknown outcome。
- Tool cache只用于明确纯读、版本绑定操作。
- 记录 Tool Schema Token Cost、result utilization、cache hit。
- 解锁 P5-T05。

### P5-T05 — 供应链和B09/B10 Gate

- 安装、验证、使用、禁用、卸载 Agent/Tool；保留历史。
- 所有 package使用 exact version/ref/digest；attestation验证和publisher policy分开。
- 建议 install success≥95%测试fixture；任何 tampered case必须0 activation。
- 解锁 Phase 6/P7。

---

## Phase 6 — Multi-Agent

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
- 固定 Rust/Node/pnpm/Pi/lockfiles。
- release artifact不得含 Key、test DB或开发路径。
- 解锁 P7-T02/T06。

### P7-T02 — Transactional update、rollback和uninstall

- update先验证→stage→migration preflight→health→atomic switch。
- downgrade只在数据兼容明确时允许；否则恢复旧 binary+DB backup。
- uninstall区分 binary/config/cache/data/secret，删除数据需要显式二次确认。
- B01增加upgrade/interruption/uninstall cleanliness。
- 解锁 P7-T06。

### P7-T03 — Doctor、support bundle和故障排查

- `cognitive doctor --bundle`只输出redacted facts/digests，不含secret和敏感正文。
- 覆盖 daemon、DB、Secret、Provider、Pi、Tools、Processes、migrations。
- 可操作错误包含next step和stable error code。
- 解锁 P7-T06。

### P7-T04 — 完整性能 campaign和回归地板

- 固定硬件、Provider snapshot、模型、任务集、warm/cold cache、重复次数。
- 输出raw run、summary、confidence interval、baseline delta、non-claims。
- 回归阈值由测量后确定，不预设虚假达成。
- 解锁 RC。

### P7-T05 — 非阻塞 Web UI

- 仅在客户端 readiness gate、技术栈 ADR、法务和 daemon API稳定后启动。
- 计划路径 `clients/pc/web/`；React/TS/Vite候选。
- 只渲染 system/provider/tasks/agents/processes/tokens/tools/memory/evidence projections。
- 不直接打开数据库、不做授权/完成判定。
- 不阻塞 RC CLI+Pi release。

### P7-T06 — RC、文档、支持矩阵和B01-B12

- clean Linux VM执行完整 suite；所有 release claims指向 evidence digest。
- `implemented` 仍只能按适用 MUST证据计算，Personal release不得冒充 Profile。
- 发布 install/init/provider/Pi/task/recovery/update/uninstall runbooks。
- 所有 open critical risks为0，或明确NO-GO。

---

# 12. 机器可读依赖图

本节只定义阶段和任务依赖。PERS-PR 到任务、Gate/benchmark 和既有 REQ 的映射由 [docs/plan/personal-trace.yaml](docs/plan/personal-trace.yaml) 单独承载，以免把产品规划 ID 混入 registry matrix。

```yaml
phases:
  P0:
    gate: G0
  P1:
    depends_on: [G0]
    gate: G1_B01
  P2:
    depends_on: [G1_B01]
    gate: G2_B02_B04_B05_B12
  P3:
    depends_on: [G2_B02_B04_B05_B12]
    gate: G3_B03_B06_B07
  P4:
    depends_on: [G3_B03_B06_B07]
    gate: G4_B08
  P5:
    depends_on: [G4_B08]
    gate: G5_B09_B10
  P6:
    depends_on: [G5_B09_B10]
    gate: G6_B11
  P7:
    depends_on: [G6_B11]
    gate: G7_RC

tasks:
  P0-T01: { depends_on: [] }
  P0-T02: { depends_on: [P0-T01] }
  P0-T03: { depends_on: [P0-T02] }
  P0-T04: { depends_on: [P0-T02] }
  P0-T05: { depends_on: [P0-T01] }
  P0-T06: { depends_on: [P0-T03] }
  P0-T07: { depends_on: [P0-T02] }

  P1-T01: { depends_on: [P0-T04] }
  P1-T02: { depends_on: [P0-T05, P1-T01] }
  P1-T03: { depends_on: [P1-T02] }
  P1-T04: { depends_on: [P0-T07, P1-T01] }
  P1-T05: { depends_on: [P1-T03, P1-T04] }
  P1-T06: { depends_on: [P1-T02, P1-T05] }
  P1-T07: { depends_on: [P0-T06, P1-T03, P1-T04, P1-T05] }
  P1-T08: { depends_on: [P0-T03, P1-T01, P1-T04, P1-T06, P1-T07] }
  P1-T09: { depends_on: [P1-T08] }

  P2-T01: { depends_on: [P1-T09] }
  P2-T02: { depends_on: [P2-T01, P1-T07] }
  P2-T03: { depends_on: [P2-T01, P1-T01] }
  P2-T04: { depends_on: [P2-T02, P2-T03] }
  P2-T05: { depends_on: [P2-T04] }
  P2-T06: { depends_on: [P2-T05] }
  P2-T07: { depends_on: [P2-T03, P2-T04, P2-T06] }
  P2-T08: { depends_on: [P2-T07] }

  P3-T01: { depends_on: [P2-T08] }
  P3-T02: { depends_on: [P3-T01] }
  P3-T03: { depends_on: [P3-T02] }
  P3-T04: { depends_on: [P3-T02, P3-T03] }
  P3-T05: { depends_on: [P3-T04] }
  P3-T06: { depends_on: [P3-T05] }

  P4-T01: { depends_on: [P3-T06] }
  P4-T02: { depends_on: [P4-T01] }
  P4-T03: { depends_on: [P4-T01] }
  P4-T04: { depends_on: [P4-T02, P4-T03] }
  P4-T05: { depends_on: [P4-T04] }
  P4-T06: { depends_on: [P4-T05] }

  P5-T01: { depends_on: [P4-T05] }
  P5-T02: { depends_on: [P5-T01] }
  P5-T03: { depends_on: [P2-T05, P4-T05] }
  P5-T04: { depends_on: [P5-T03] }
  P5-T05: { depends_on: [P5-T02, P5-T04] }

  P6-T01: { depends_on: [P5-T05] }
  P6-T02: { depends_on: [P6-T01] }
  P6-T03: { depends_on: [P6-T02] }
  P6-T04: { depends_on: [P6-T03] }

  P7-T01: { depends_on: [P5-T05, P0-T03] }
  P7-T02: { depends_on: [P7-T01, P1-T01] }
  P7-T03: { depends_on: [P7-T02] }
  P7-T04: { depends_on: [P3-T06, P4-T05, P5-T05] }
  P7-T05: { depends_on: [P2-T08, P7-T03] }
  P7-T06: { depends_on: [P7-T02, P7-T03, P7-T04, P5-T05, P6-T04] }

critical_path:
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
  - P4-T04
  - P4-T05
  - P5-T01
  - P5-T02
  - P5-T05
  - P7-T01
  - P7-T02
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
| `crates/cognitive-runtime/src/lib.rs` | Provider、Task、Scheduler、Process、Memory都可能触碰；必须按 phase串行 |
| `crates/cognitive-store/src/sqlite.rs` | migrations、scheduler、memory；先拆模块再分别所有权 |
| `apps/kernel-server/src/main.rs` | daemon/auth/routes；Phase 1后禁止并行大改 |
| `apps/admin-cli/src/main.rs` | init/task/memory/agents/tools；先抽 shared client/service |
| `packages/sdk-ts` | API contract跟随 Lane-CTR，禁止独立造类型 |
| `docs/plan/PROGRESS.md` | 后合并者负责 rebase |
| migration目录 | migration编号必须单一分配，不可并行抢号 |

---

# 13. Benchmark Suite

| ID | 目标/前置与输入 | 步骤和预期转换 | 观测、成功/失败 | Token/时间/证据/自动化 |
|---|---|---|---|
| B01 | clean Linux VM、测试 DeepSeek account | install→init→secret→models→probe→daemon→Pi→response | ready必须来自真实checks；Key泄漏或仅HTTP 200即失败 | TTFC、probe tokens；VM runner+redacted bundle |
| B02 | 已初始化系统 | Shell查询system/task/process/agent/tool/memory | NL→management intent→真实projection | synthetic/cached错误状态失败；响应tokens/latency |
| B03 | 固定真实仓库快照 | 定位模块并输出path/symbol evidence | 证据准确、无越界、Context预算内 | total/input/repeated context、wall time |
| B04 | 有可复现bug fixture | reproduce→diagnose→change→test→verify | 只修部分、仅zero exit、自报成功均失败 | tokens/progress、tool calls、verified criteria |
| B05 | 运行中Task | 关Shell→重连→关daemon→restart→recover | fencing/replay/reconcile后继续；无重复Effect | recovery steps/time、effect evidence |
| B06 | 大日志、多文件、多轮工具 | externalize→dedup→compress→continue | 目标/约束/required evidence不可丢 | repeated ratio、compression savings |
| B07 | 固定重复失败executor | repeated error→detect→switch→blocked | 预算内停止，不无限调用 | no-progress、repeat action、guard latency |
| B08 | memory corpus | save→retrieve→use→update→conflict→expire→forget | stale/forgotten不得返回 | precision/recall/latency/token ROI |
| B09 | signed agent fixture | install→health→activate→use→stop→uninstall | 无隐式capability；历史保留 | install time、health、evidence |
| B10 | signed tool/MCP fixture | qualify→enable→call→disable→uninstall | schema drift/disabled调用拒绝 | calls/failure/cache/result tokens |
| B11 | 可并行研究任务 | single baseline→2 workers→reviewer→integrator | 无重复/写冲突，verified结果不差 | speedup、coordination overhead、conflicts |
| B12 | executor timeout/uncertain | dispatch→timeout→OUTCOME_UNKNOWN→query/reconcile | 不换key、不盲重试、不完成Task | recovery time/steps、original key evidence |

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

## 15.2 每次任务最低命令

```text
cargo fmt --all -- --check
cargo test -p <affected-package>
cargo clippy -p <affected-package> --all-targets -- -D warnings
pnpm -r build
pnpm -r test
pnpm run check:consistency
node tools/src/gen-matrix.mjs --check
git diff --check
```

根据影响面追加 workspace、conformance、security、fault、E2E。

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
| R-22 | 企业模块阻塞个人版 | 中/高 |每次本地动作要求复杂approval | lightweight policy profile、保留边界旁路 | 全阶段 |
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
| 企业审批 | management基础 | 仅destructive confirmation | 企业版另Profile | 不建审批链 |
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

1. 读取 `AGENTS.md`；
2. 读取 `docs/plan/PROGRESS.md`；
3. 读取最近 handoff；
4. 读取 Personal 总计划和当前 Phase；
5. 读取唯一对应 Task Card；
6. 验证 `depends_on` 全部完成；
7. 记录 base commit、branch、dirty state；
8. 运行该卡要求的 baseline tests；
9. 明确复述“不包含内容”；
10. 列出唯一允许修改路径；
11. 定位真实 REQ/schema/vector；
12. 若需越界，停止，不编码。

## 18.2 每个会话完成后

1. 运行任务指定 tests；
2. 运行相关回归；
3. 更新 Task 状态；
4. 记录实际修改文件；
5. 记录测试命令、exit、关键输出；
6. 记录实际 public/internal/data/config变化；
7. 记录偏差和未决项；
8. 更新风险；
9. 更新 PROGRESS；
10. 写 handoff；
11. 完成且全绿才提交/push；
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
