# CognitiveOS — core / personal / enterprise / clients

本仓库按 [ADR-0054](docs/adr/0054-repository-subproject-structure-and-1.0.0-finalization.md)
组织为四个子项目目录加共享治理层，但仍然只有一个活动实现项目：

| 目录 | 子项目 | 状态 |
|---|---|---|
| [core/](core) | **cognitiveos-core** — 产品中立的机器合同与权威基底（specs、contracts、domain、kernel、AKP、conformance vectors、跨语言金样、架构白皮书） | **1.0.0 已定稿**（[边界与验收](core/docs/VERSION-1.0.0.md)，tag `core-v1.0.0`） |
| [personal/](personal) | **cognitiveos-personal** — 唯一活动实现项目：Rust daemon、六资源基座、Provider/SecretStore、CLI、Pi/dsh adapter、deploy、双语 handbook | **1.0.0 已定稿**（[边界与验收](personal/docs/VERSION-1.0.0.md)，tag `personal-v1.0.0`）；post-1.0 列车继续 |
| [enterprise/](enterprise) | **cognitiveos-enterprise** — 中央治理平面设计层 | **未开始**，仅有 [1.0.0 边界定义](enterprise/docs/VERSION-1.0.0.md) 与候选设计；未经 owner 激活不得实现 |
| [clients/](clients) | 客户端子项目（原独立仓库 `cognitiveos-clients` 于 2026-08-25 并入，历史保留）：PC Web Control Plane、mobile、shared、Agent Hub 文档域 | Web UI 实现在 [clients/pc/web](clients/pc/web)（ADR-0053） |

项目身份和默认工作范围以
[PROJECT-IDENTITY.md](docs/governance/PROJECT-IDENTITY.md) 为准；Personal 正式任务以
[PERSONAL-DEVELOPMENT-PLAN.md](docs/plan/PERSONAL-DEVELOPMENT-PLAN.md) 为准，当前事实
以 [PROGRESS.md](docs/plan/PROGRESS.md) 的 `Current snapshot` 为准。规范存在不代表
实现存在；实现存在不代表 Gate、release 或 Profile 符合。

依赖方向固定：`core → personal → clients`（clients 只消费 Personal API），
`core → enterprise`（设计期合同消费）；core 永不依赖产品目录。

## 分区导航

### 1. 规范与合同（core/）

| 路径 | 内容 |
|---|---|
| [core/specs/](core/specs) | companion 规范、registry、状态迁移表与 JSON Schema（draft 2020-12） |
| [core/crates/](core/crates) | `cognitive-contracts`（生成绑定）、`cognitive-domain`、`cognitive-kernel`、`cognitive-akp` |
| [core/packages/contracts-ts/](core/packages/contracts-ts) | TS 合同绑定与金样发射器 |
| [core/conformance/](core/conformance) | 声明式规范向量；执行结果由 runner/evidence 报告 |
| [core/tests/golden/](core/tests/golden) | 跨语言 canonical/digest 金样 |
| [core/docs/architecture/](core/docs/architecture) | 白皮书 v1.0.2、RFC-0001、两轮冻结评审 |

### 2. Personal 实现（唯一活动项目）

| 路径 | 内容 |
|---|---|
| [personal/crates/](personal/crates) | store、runtime、management、conformance runner（reference-IUT）、secret、provider-transport |
| [personal/apps/kernel-server](personal/apps/kernel-server) · [personal/apps/admin-cli](personal/apps/admin-cli) | 单节点组合根（daemon）· 确定性管理/产品 CLI |
| [personal/apps/agent-shell](personal/apps/agent-shell) · [personal/apps/pi-agent-adapter](personal/apps/pi-agent-adapter) | Task Shell 客户端核心 · Pi candidate adapter |
| [personal/packages/](personal/packages) | sdk-ts、pi-cognitiveos（Pi-hosted Shell Extension）、dsh-akp-adapter |
| [personal/deploy/](personal/deploy) | Linux systemd 与 Windows 安装面 |
| [personal/handbook/](personal/handbook) | 双语使用者/开发者/AI 手册（机器校验；AI 入口 [llms.txt](llms.txt)） |
| [personal/docs/product/](personal/docs/product) · [personal/docs/architecture/](personal/docs/architecture) | Personal canonical 产品设计 · 产品组合架构 |

### 3. 测试与证据

| 路径 | 内容 |
|---|---|
| [core/conformance/](core/conformance) · [personal/crates/cognitive-conformance](personal/crates/cognitive-conformance) | 规范向量 · 符合性 runner（局部通过不构成 Gate/Profile 结论） |
| [personal/tests/](personal/tests) | fixtures / e2e / faults / security（Personal 测试资产） |
| [tools/](tools) | 静态一致性检查、handbook 检查/生成、docs-sync 门、Gate 评估器（接入 CI） |
| `artifacts/evidence/` | 运行证据目录（gitignore，凭 digest 引用） |

### 4. 共享治理与文档（docs/）

| 路径 | 内容 |
|---|---|
| [AGENTS.md](AGENTS.md) | 开发代理入口：命令速查、硬纪律、会话协议 |
| [docs/governance/](docs/governance) | 项目身份、公理（A1–A8）、Development Operating Model |
| [docs/plan/](docs/plan) | Personal 正式计划、current snapshot、lease、支持矩阵、测试环境（历史计划在 [plan/archive/](docs/plan/archive)） |
| [docs/standards/](docs/standards) | 机器可判定行为标准（canonical/digest、状态迁移、错误合同、docs-sync 等） |
| [docs/adr/](docs/adr) | 架构与产品决策记录（子项目结构与 1.0.0 定稿见 ADR-0054） |
| [docs/traceability/](docs/traceability) | REQ 追溯矩阵 + findings 台账 |
| [docs/checkpoints/](docs/checkpoints) · [docs/evaluation/](docs/evaluation) | 会话交接/里程碑评审 · 评测合同与历史 campaign 报告 |
| [docs/research/](docs/research/README.md) | owner 分析归档（含 agent-work-system 发现文档）；不创建任务、Gate 或当前状态 |
| [docs/README.md](docs/README.md) | 文档地图与分类口径 |

## 四类状态用语（全仓强制）

任何文档与声明必须区分以下四类状态，不得混用（详见 [core/conformance/README.md](core/conformance/README.md)）：

1. **规范已登记（specified）**：REQ/schema/vector 在 registry 中存在。不代表任何实现存在。
2. **实现已提供（implementation available）**：代码存在且构建通过。不代表被测试证明。
3. **测试已执行（test executed）**：runner 真实执行向量并保留证据。schema-valid ≠ behavior-pass。
4. **Profile 已符合（implemented）**：该 Profile 全部适用 MUST 有通过证据或有据 not-applicable。安全负例不可被降级豁免。

当前 Personal task、implementation evidence、Gate 和 claim scope 以
[PROGRESS](docs/plan/PROGRESS.md) 的 `Current snapshot` 为准。README 不复制易漂移的
REQ、vector 或结果计数；任何 release/Profile 声明必须链接对应 campaign 与独立 verifier
证据。

## 快速开始

```powershell
# Rust（工具链钉在 rust-toolchain.toml；虚拟 workspace 跨 core/ 与 personal/）
cargo build --workspace ; cargo test --workspace

# TypeScript（Node >= 22，pnpm 10）
pnpm install ; pnpm -r build ; pnpm -r test

# 静态一致性检查（registry↔schema↔vector↔docs）
pnpm run check:consistency

# handbook 检查与生成页字节比对
pnpm run check:handbook

# 符合性 runner（输出五态报告、证据与样例 manifest）
cargo run -p cognitive-conformance --bin conformance-runner
```

`History/` 为冻结归档，不参与构建、schema bundle 与符合性声明，任何工具与文档不得引用。
